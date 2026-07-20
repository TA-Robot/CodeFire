use super::*;
use std::collections::{BTreeMap, BTreeSet};

const DIVERGENCE_KIND: &str = "open_snapshot_divergence";

#[derive(Debug, Clone, Copy)]
enum DivergenceDirection {
    BranchNewRegistryOld,
    BranchOldRegistryNew,
}

impl DivergenceDirection {
    fn label(self) -> &'static str {
        match self {
            Self::BranchNewRegistryOld => "branch-new-registry-old",
            Self::BranchOldRegistryNew => "branch-old-registry-new",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OpenConsumer {
    Status,
    Scan,
    Verify,
    CommitDryRun,
    CommitLive,
    FireLive,
    FireBatchLive,
    ExtinguishLive,
    ExtinguishBatchLive,
}

impl OpenConsumer {
    fn label(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Scan => "scan",
            Self::Verify => "verify",
            Self::CommitDryRun => "commit-dry-run",
            Self::CommitLive => "commit-live",
            Self::FireLive => "fire-live",
            Self::FireBatchLive => "fire-batch-live",
            Self::ExtinguishLive => "extinguish-live",
            Self::ExtinguishBatchLive => "extinguish-batch-live",
        }
    }
}

struct DivergentOpenFixture {
    _temp: tempfile::TempDir,
    repo_root: PathBuf,
    open_dir: PathBuf,
    fire_batch_path: PathBuf,
    extinguish_batch_path: PathBuf,
    branch_head: String,
    opened_base: String,
}

struct PendingStateFixture {
    _temp: tempfile::TempDir,
    repo_root: PathBuf,
    open_dir: PathBuf,
    active_path: PathBuf,
    marker_path: PathBuf,
    marker: Value,
}

struct PendingCommitFixture {
    _temp: tempfile::TempDir,
    repo_root: PathBuf,
    open_dir: PathBuf,
    marker_path: PathBuf,
    options: CommitOptions,
    candidate: String,
}

#[test]
fn branch_head_opened_base_mismatch_is_rejected_prewrite_by_every_open_consumer() {
    let mut failures = Vec::new();
    for direction in [
        DivergenceDirection::BranchNewRegistryOld,
        DivergenceDirection::BranchOldRegistryNew,
    ] {
        for consumer in [
            OpenConsumer::Status,
            OpenConsumer::Scan,
            OpenConsumer::Verify,
            OpenConsumer::CommitDryRun,
            OpenConsumer::CommitLive,
            OpenConsumer::FireLive,
            OpenConsumer::FireBatchLive,
            OpenConsumer::ExtinguishLive,
            OpenConsumer::ExtinguishBatchLive,
        ] {
            let fixture = divergent_open_fixture(direction);
            let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
            let result = run_open_consumer(consumer, &fixture);
            let after = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
            let case = format!("{}/{}", direction.label(), consumer.label());

            match result {
                Ok(()) => failures.push(format!("{case}: accepted divergent open authority")),
                Err(error) => {
                    if !matches!(&error, CliError::OpenSnapshotDivergence(_)) {
                        failures.push(format!(
                            "{case}: error was not typed OpenSnapshotDivergence: {error}"
                        ));
                    }
                    let diagnostic = error.diagnostic();
                    if diagnostic["kind"].as_str() != Some(DIVERGENCE_KIND) {
                        failures.push(format!(
                            "{case}: diagnostic kind was {:?}, expected {DIVERGENCE_KIND}",
                            diagnostic["kind"]
                        ));
                    }
                    if error.exit_code() != ExitCode::RepositoryCorruption.code() {
                        failures.push(format!(
                            "{case}: exit code was {}, expected {}",
                            error.exit_code(),
                            ExitCode::RepositoryCorruption.code()
                        ));
                    }
                    for (field, expected) in [
                        ("phase", "steady"),
                        ("authority", "branch_head"),
                        ("expected", fixture.branch_head.as_str()),
                        ("actual", fixture.opened_base.as_str()),
                    ] {
                        if diagnostic[field].as_str() != Some(expected) {
                            failures.push(format!(
                                "{case}: diagnostic {field} was {:?}, expected {expected}",
                                diagnostic[field]
                            ));
                        }
                    }
                }
            }

            if before != after {
                failures.push(format!(
                    "{case}: changed authority bytes: {:?}",
                    changed_authority_paths(&before, &after)
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "open consumers did not share a pre-write divergence rejection:\n{}",
        failures.join("\n")
    );
}

#[test]
fn doctor_reports_the_same_open_snapshot_divergence_for_both_mismatch_directions() {
    let mut failures = Vec::new();
    for direction in [
        DivergenceDirection::BranchNewRegistryOld,
        DivergenceDirection::BranchOldRegistryNew,
    ] {
        let fixture = divergent_open_fixture(direction);
        let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
        let report = run_doctor(&DoctorOptions {
            start: fixture.repo_root.clone(),
            quick: false,
            json_output: true,
        })
        .unwrap();
        let after = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
        let diagnostics = doctor_report_diagnostics_json(&report);
        let case = direction.label();

        let Some(diagnostic) = diagnostics
            .iter()
            .find(|diagnostic| diagnostic["kind"].as_str() == Some(DIVERGENCE_KIND))
        else {
            failures.push(format!(
                "{case}: doctor omitted {DIVERGENCE_KIND}; kinds={:?}",
                diagnostics
                    .iter()
                    .filter_map(|diagnostic| diagnostic["kind"].as_str())
                    .collect::<Vec<_>>()
            ));
            continue;
        };

        for (field, expected) in [
            ("phase", "steady"),
            ("authority", "branch_head"),
            ("expected", fixture.branch_head.as_str()),
            ("actual", fixture.opened_base.as_str()),
        ] {
            if diagnostic[field].as_str() != Some(expected) {
                failures.push(format!(
                    "{case}: doctor {field} was {:?}, expected {expected}",
                    diagnostic[field]
                ));
            }
        }
        if !diagnostic["blocking"].as_bool().unwrap_or(false) {
            failures.push(format!("{case}: doctor divergence was not blocking"));
        }
        let safe_action = doctor_report_next_actions(&report)
            .into_iter()
            .any(|action| {
                action["id"] == "inspect_open_snapshot_divergence"
                    && action["command"]
                        .as_str()
                        .is_some_and(|command| command.contains("doctor"))
            });
        if !safe_action {
            failures.push(format!(
                "{case}: doctor omitted the read-only inspect_open_snapshot_divergence action"
            ));
        }
        if before != after {
            failures.push(format!(
                "{case}: doctor changed authority bytes: {:?}",
                changed_authority_paths(&before, &after)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "doctor did not share the open-consumer divergence contract:\n{}",
        failures.join("\n")
    );
}

#[test]
fn pending_commit_publication_window_is_not_a_steady_open_snapshot_divergence() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "pending publication window\n").unwrap();
    let active = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active, "after_branch_publish_before_registry");
    let options = CommitOptions {
        path: open_dir,
        message: "Characterize pending publication".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    };

    let interrupted = run_commit(&options).unwrap_err();
    assert!(interrupted
        .to_string()
        .contains("after_branch_publish_before_registry"));
    let marker_path = active.join("commit_transaction.json");
    let marker = read_json(&marker_path).unwrap();
    assert_eq!(marker["phase"], "publishing_branch");
    let candidate = marker["candidate_commit"]["object_id"]
        .as_str()
        .unwrap()
        .to_string();
    let branch_head =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    let opened_base = required_string(
        &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        &["open", "current_base_commit"],
    )
    .unwrap();
    assert_eq!(branch_head, candidate);
    assert_ne!(branch_head, opened_base);

    let pending_before = authority_snapshot(&repo_root, &options.path);
    let status_error = read_status(&options.path).unwrap_err();
    assert_open_recovery_pending(&status_error, "commit", "publishing_branch");
    assert_eq!(
        authority_snapshot(&repo_root, &options.path),
        pending_before,
        "status mutated a valid pending commit"
    );
    let unrelated_scan = run_scan(&options.path).unwrap_err();
    assert_open_recovery_pending(&unrelated_scan, "commit", "publishing_branch");
    assert_eq!(
        authority_snapshot(&repo_root, &options.path),
        pending_before,
        "unrelated scan mutated a valid pending commit"
    );

    let report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        quick: false,
        json_output: true,
    })
    .unwrap();
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.kind == "pending_commit_transaction"));
    assert!(!report
        .issues
        .iter()
        .any(|issue| issue.kind == DIVERGENCE_KIND));

    let recovered = run_commit(&options).unwrap();
    assert_eq!(recovered.commit_id, candidate);
    assert!(!marker_path.exists());
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        candidate
    );
    assert_eq!(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "current_base_commit"],
        )
        .unwrap(),
        candidate
    );
}

#[test]
fn pending_state_phase_disallowed_authority_tuple_is_rejected_prewrite() {
    let fixture = pending_scan_fixture("registry_written");
    let pre_state = fixture.marker["pre_state"]["branch"]
        .as_str()
        .unwrap()
        .to_string();
    let intended_state = fixture.marker["intended_state"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(pre_state, intended_state);

    // `registry_written` permits (pre, intended, pre) and
    // (pre, intended, intended). This crossed tuple belongs to neither
    // transition edge even though the v2 transaction subject is authentic.
    write_state_authorities(
        &fixture.repo_root,
        &fixture.active_path,
        &intended_state,
        &pre_state,
        &intended_state,
    );
    let marker = read_json(&fixture.marker_path).unwrap();
    assert_eq!(marker["subject"], fixture.marker["subject"]);
    validate_state_transaction_record(&marker).unwrap();

    let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
    let status_error = read_status(&fixture.open_dir).unwrap_err();
    assert_transaction_divergence(
        &status_error,
        "pending_state_transaction",
        "registry_written",
    );
    assert_eq!(
        authority_snapshot(&fixture.repo_root, &fixture.open_dir),
        before,
        "status changed a phase-disallowed state tuple"
    );

    let owning_retry = run_scan(&fixture.open_dir).unwrap_err();
    assert_transaction_divergence(
        &owning_retry,
        "pending_state_transaction",
        "registry_written",
    );
    assert_eq!(
        authority_snapshot(&fixture.repo_root, &fixture.open_dir),
        before,
        "owning retry changed a phase-disallowed state tuple"
    );
}

#[test]
fn pending_state_foreign_open_instance_and_base_are_rejected_prewrite() {
    for (label, pointer, foreign) in [
        (
            "open-instance",
            "/subject/open_instance_id",
            "open_foreign_0123456789abcdef",
        ),
        (
            "current-base",
            "/subject/current_base_commit",
            "CF-COMMIT-foreign-subject",
        ),
    ] {
        let fixture = pending_scan_fixture("fires_written");
        let mut marker = fixture.marker.clone();
        let subject = marker
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("{label}: missing subject field {pointer}"));
        assert_ne!(subject.as_str(), Some(foreign));
        *subject = Value::String(foreign.to_string());
        validate_state_transaction_record(&marker).unwrap();
        write_json_atomic(&fixture.marker_path, &marker).unwrap();

        let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
        let error = run_scan(&fixture.open_dir).unwrap_err();
        assert_transaction_divergence(&error, "pending_state_transaction", "fires_written");
        assert_eq!(
            authority_snapshot(&fixture.repo_root, &fixture.open_dir),
            before,
            "{label}: foreign state subject was mutated before rejection"
        );
    }
}

#[test]
fn pending_commit_with_forged_parents_is_rejected_prewrite() {
    let fixture = pending_commit_fixture("after_registry_publish_before_phase");
    let mut marker = read_json(&fixture.marker_path).unwrap();
    let original_parent = marker["parents"][0].as_str().unwrap().to_string();
    let forged_parent = "CF-COMMIT-forged-parent";
    assert_ne!(original_parent, forged_parent);
    marker["parents"][0] = Value::String(forged_parent.to_string());
    validate_commit_transaction_record(&marker).unwrap();
    write_json_atomic(&fixture.marker_path, &marker).unwrap();

    let objects = fixture.repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects, &fixture.candidate).unwrap();
    let candidate = codefire_store::read_object(&objects, &fixture.candidate).unwrap();
    assert_ne!(candidate["parents"], marker["parents"]);

    let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);
    let error = run_commit(&fixture.options).unwrap_err();
    assert_transaction_divergence(&error, "pending_commit_transaction", "publishing_branch");
    assert_eq!(
        authority_snapshot(&fixture.repo_root, &fixture.open_dir),
        before,
        "forged pending commit mutated authority before rejection"
    );
}

#[test]
fn valid_pending_state_blocks_nonowners_and_exact_owner_retry_converges() {
    let fixture = pending_scan_fixture("fires_written");
    let intended_state = fixture.marker["intended_state"]
        .as_str()
        .unwrap()
        .to_string();
    let before = authority_snapshot(&fixture.repo_root, &fixture.open_dir);

    let status_error = read_status(&fixture.open_dir).unwrap_err();
    assert_open_recovery_pending(&status_error, "scan", "fires_written");
    assert_eq!(
        authority_snapshot(&fixture.repo_root, &fixture.open_dir),
        before,
        "status mutated a valid pending state transaction"
    );

    let unrelated_verify = run_verify(&fixture.open_dir).unwrap_err();
    assert_open_recovery_pending(&unrelated_verify, "scan", "fires_written");
    assert_eq!(
        authority_snapshot(&fixture.repo_root, &fixture.open_dir),
        before,
        "unrelated verify mutated a valid pending state transaction"
    );

    run_scan(&fixture.open_dir).unwrap();
    assert!(!fixture.marker_path.exists());
    assert_state_authorities(&fixture.repo_root, &fixture.active_path, &intended_state);
    read_status(&fixture.open_dir).unwrap();
}

fn pending_scan_fixture(phase: &str) -> PendingStateFixture {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "pending state subject\n").unwrap();
    let active_path = active_state_path(&open_dir);
    set_state_transaction_failure_for_test(&active_path, "scan", phase);
    let interrupted = run_scan(&open_dir).unwrap_err();
    assert!(
        interrupted
            .to_string()
            .contains(&format!("scan state transaction failure at {phase}")),
        "{phase}: {interrupted}"
    );
    let marker_path = active_path.join("state_transaction.json");
    let marker = read_json(&marker_path).unwrap();
    validate_state_transaction_record(&marker).unwrap();
    assert_eq!(marker["version"], 2);
    assert_eq!(marker["operation"], "scan");
    assert_eq!(marker["phase"], phase);
    open_context_for_recovery(&open_dir, "scan").unwrap();

    PendingStateFixture {
        _temp: temp,
        repo_root,
        open_dir,
        active_path,
        marker_path,
        marker,
    }
}

fn pending_commit_fixture(failpoint: &str) -> PendingCommitFixture {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "pending commit subject\n").unwrap();
    let options = CommitOptions {
        path: open_dir.clone(),
        message: "Pending commit subject".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    };
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, failpoint);
    let interrupted = run_commit(&options).unwrap_err();
    assert!(
        interrupted.to_string().contains(failpoint),
        "{failpoint}: {interrupted}"
    );
    let marker_path = active_path.join("commit_transaction.json");
    let marker = read_json(&marker_path).unwrap();
    validate_commit_transaction_record(&marker).unwrap();
    assert_eq!(marker["phase"], "publishing_branch");
    let candidate = marker["candidate_commit"]["object_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        candidate
    );
    assert_eq!(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "current_base_commit"],
        )
        .unwrap(),
        candidate
    );
    open_context_for_recovery(&open_dir, "commit").unwrap();

    PendingCommitFixture {
        _temp: temp,
        repo_root,
        open_dir,
        marker_path,
        options,
        candidate,
    }
}

fn write_state_authorities(
    repo_root: &Path,
    active_path: &Path,
    branch_state: &str,
    registry_state: &str,
    active_state: &str,
) {
    let branch_path = branch_record_path(repo_root, "main");
    let mut branch = read_json(&branch_path).unwrap();
    branch["state"] = Value::String(branch_state.to_string());
    write_json_atomic(&branch_path, &branch).unwrap();

    let registry_path = opened_registry_path(repo_root, "main");
    let mut registry = read_json(&registry_path).unwrap();
    registry["state"]["last_known"] = Value::String(registry_state.to_string());
    write_json_atomic(&registry_path, &registry).unwrap();

    let state_path = active_path.join("state.json");
    let mut state = read_json(&state_path).unwrap();
    state["state"] = Value::String(active_state.to_string());
    write_json_atomic(&state_path, &state).unwrap();
}

fn assert_state_authorities(repo_root: &Path, active_path: &Path, expected: &str) {
    assert_eq!(
        load_branch_record(repo_root, "main").unwrap()["state"],
        expected
    );
    assert_eq!(
        read_json(&opened_registry_path(repo_root, "main")).unwrap()["state"]["last_known"],
        expected
    );
    assert_eq!(
        read_json(&active_path.join("state.json")).unwrap()["state"],
        expected
    );
}

fn assert_transaction_divergence(error: &CliError, authority: &str, phase: &str) {
    match error {
        CliError::OpenSnapshotDivergence(divergence) => {
            assert_eq!(divergence.authority, authority);
            assert_eq!(divergence.phase, phase);
        }
        other => panic!(
            "expected typed open snapshot divergence, got {other}: {:?}",
            other.diagnostic()
        ),
    }
    let diagnostic = error.diagnostic();
    assert_eq!(diagnostic["kind"], DIVERGENCE_KIND);
    assert_eq!(diagnostic["authority"], authority);
    assert_eq!(diagnostic["phase"], phase);
    assert_eq!(
        error.exit_code(),
        ExitCode::RepositoryCorruption.code(),
        "{diagnostic:?}"
    );
}

fn assert_open_recovery_pending(error: &CliError, operation: &str, phase: &str) {
    match error {
        CliError::OpenRecoveryPending(pending) => {
            assert_eq!(pending.operation, operation);
            assert_eq!(pending.phase, phase);
        }
        other => panic!(
            "expected typed open recovery pending, got {other}: {:?}",
            other.diagnostic()
        ),
    }
    let diagnostic = error.diagnostic();
    assert_eq!(diagnostic["kind"], "open_recovery_pending");
    assert_eq!(diagnostic["operation"], operation);
    assert_eq!(diagnostic["phase"], phase);
    assert_eq!(diagnostic["blocking"], true);
}

fn divergent_open_fixture(direction: DivergenceDirection) -> DivergentOpenFixture {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let fire_batch_path = temp.path().join("divergent-fire-batch.json");
    let extinguish_batch_path = temp.path().join("divergent-extinguish-batch.json");
    fs::write(
        &fire_batch_path,
        serde_json::to_vec(&json!({
            "version": 1,
            "fires": [{
                "from": "REQ-divergent",
                "to": "DES-divergent",
                "reason": "reject divergent open snapshot",
            }],
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &extinguish_batch_path,
        serde_json::to_vec(&json!({
            "version": 1,
            "fires": [{
                "id": "FIRE-divergent",
                "resolution": "addressed",
                "rationale": "reject divergent open snapshot",
            }],
        }))
        .unwrap(),
    )
    .unwrap();
    init_repo(&repo_root, false).unwrap();
    let old_head =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "second valid commit\n").unwrap();
    let committed = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Create second valid commit".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let new_head = committed.commit_id;
    assert_ne!(old_head, new_head);

    match direction {
        DivergenceDirection::BranchNewRegistryOld => {
            let path = opened_registry_path(&repo_root, "main");
            let mut registry = read_json(&path).unwrap();
            registry["open"]["current_base_commit"] = Value::String(old_head.clone());
            write_json_atomic(&path, &registry).unwrap();
        }
        DivergenceDirection::BranchOldRegistryNew => {
            let path = branch_record_path(&repo_root, "main");
            let mut branch = read_json(&path).unwrap();
            branch["head"] = Value::String(old_head.clone());
            write_json_atomic(&path, &branch).unwrap();
        }
    }

    let branch_head =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    let opened_base = required_string(
        &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        &["open", "current_base_commit"],
    )
    .unwrap();
    assert_ne!(branch_head, opened_base);
    let objects = repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects, &branch_head).unwrap();
    codefire_store::validate_sealed_commit(&objects, &opened_base).unwrap();

    DivergentOpenFixture {
        _temp: temp,
        repo_root,
        open_dir,
        fire_batch_path,
        extinguish_batch_path,
        branch_head,
        opened_base,
    }
}

fn run_open_consumer(
    consumer: OpenConsumer,
    fixture: &DivergentOpenFixture,
) -> Result<(), CliError> {
    let open_dir = &fixture.open_dir;
    match consumer {
        OpenConsumer::Status => read_status(open_dir).map(|_| ()),
        OpenConsumer::Scan => run_scan(open_dir).map(|_| ()),
        OpenConsumer::Verify => run_verify(open_dir).map(|_| ()),
        OpenConsumer::CommitDryRun | OpenConsumer::CommitLive => run_commit(&CommitOptions {
            path: open_dir.to_path_buf(),
            message: format!("Reject {} divergence", consumer.label()),
            dry_run: matches!(consumer, OpenConsumer::CommitDryRun),
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
            signer: None,
            key_id: None,
        })
        .map(|_| ()),
        OpenConsumer::FireLive => fire::run_fire(&fire::FireOptions {
            path: open_dir.clone(),
            source_atom: "REQ-divergent".to_string(),
            target_atom: "DES-divergent".to_string(),
            reason: "reject divergent open snapshot".to_string(),
            severity: "required".to_string(),
            dry_run: false,
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
        })
        .map(|_| ()),
        OpenConsumer::FireBatchLive => fire::run_fire_batch(&fire::FireBatchOptions {
            path: open_dir.clone(),
            batch_path: fixture.fire_batch_path.clone(),
            dry_run: false,
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
        })
        .map(|_| ()),
        OpenConsumer::ExtinguishLive => run_extinguish(&ExtinguishOptions {
            path: open_dir.clone(),
            fire_id: "FIRE-divergent".to_string(),
            resolution: "addressed".to_string(),
            rationale: "reject divergent open snapshot".to_string(),
            evidence: String::new(),
            evidence_refs: Vec::new(),
            refresh: false,
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
            edit_rationale: false,
        })
        .map(|_| ()),
        OpenConsumer::ExtinguishBatchLive => run_extinguish_batch(&batch::BatchExtinguishOptions {
            path: open_dir.clone(),
            batch_path: fixture.extinguish_batch_path.clone(),
            dry_run: false,
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
        })
        .map(|_| ()),
    }
}

fn authority_snapshot(repo_root: &Path, open_dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    let codefire = repo_root.join(".codefire");
    for relative in ["objects", "branches", "opened", "active"] {
        snapshot_tree(repo_root, &codefire.join(relative), &mut snapshot);
    }
    let marker = open_dir.join(".codefire-open");
    if marker.is_file() {
        snapshot.insert(
            PathBuf::from("open-marker").join(".codefire-open"),
            fs::read(marker).unwrap(),
        );
    }
    snapshot
}

fn snapshot_tree(repo_root: &Path, path: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if !path.exists() {
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        let metadata = fs::symlink_metadata(&entry).unwrap();
        if metadata.is_dir() {
            snapshot_tree(repo_root, &entry, snapshot);
        } else if metadata.is_file() {
            snapshot.insert(
                entry.strip_prefix(repo_root).unwrap().to_path_buf(),
                fs::read(entry).unwrap(),
            );
        } else {
            panic!("unexpected authority entry type: {}", entry.display());
        }
    }
}

fn changed_authority_paths(
    before: &BTreeMap<PathBuf, Vec<u8>>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
) -> Vec<PathBuf> {
    before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before.get(path) != after.get(path))
        .collect()
}
