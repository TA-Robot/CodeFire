use super::*;
use crate::close_transaction::{
    close_transaction_path, set_close_branch_parent_swap_for_test,
    set_close_transaction_failure_for_test,
};
use crate::open_transaction::{open_transaction_path, set_open_transaction_failure_for_test};
#[cfg(unix)]
use crate::open_transaction::{
    set_open_target_swap_before_fingerprint_for_test,
    set_open_target_symlink_before_fingerprint_for_test,
};
#[cfg(unix)]
use crate::secure_worktree::set_manifest_file_swap_before_rebind_for_test;

fn transactional_open_options(target: &Path) -> OpenOptions {
    OpenOptions {
        branch: "main".to_string(),
        path: target.to_path_buf(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: Some("open-transaction-once".to_string()),
    }
}

#[test]
fn open_transaction_recovers_every_publication_boundary() {
    for failpoint in [
        "after_prepared",
        "after_target_publish_before_phase",
        "after_target_published",
        "after_registry_write_before_phase",
        "after_registry_written",
        "after_active_directory_create_before_state",
        "after_active_state_write_before_phase",
        "after_active_state_written",
        "after_branch_object_before_reference",
        "after_branch_reference_before_phase",
        "after_branch_written",
        "after_idempotency_write_before_phase",
        "after_idempotency_written",
        "after_complete_archive_before_marker_removal",
        "after_marker_removed",
    ] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let target = temp.path().join(format!("main-open-{failpoint}"));
        init_repo(&repo_root, false).unwrap();
        let options = transactional_open_options(&target);
        set_open_transaction_failure_for_test(&repo_root, failpoint);

        let interrupted = open_branch_from(&repo_root, &options).unwrap_err();
        assert!(
            interrupted.to_string().contains(failpoint),
            "{failpoint}: {interrupted}"
        );
        let recovered = open_branch_from(&repo_root, &options).unwrap();
        let replayed = open_branch_from(&repo_root, &options).unwrap();

        assert_eq!(recovered.branch, "main", "{failpoint}");
        assert_eq!(recovered.open_dir, target, "{failpoint}");
        assert_eq!(replayed.open_dir, recovered.open_dir, "{failpoint}");
        assert_eq!(replayed.plan, recovered.plan, "{failpoint}");
        assert!(target.join(".codefire-open").is_file(), "{failpoint}");
        assert!(
            opened_registry_path(&repo_root, "main").is_file(),
            "{failpoint}"
        );
        assert_eq!(
            branch_state(&load_branch_record(&repo_root, "main").unwrap()),
            "open-clean",
            "{failpoint}"
        );
        assert!(!open_transaction_path(&repo_root).exists(), "{failpoint}");
    }
}

#[cfg(unix)]
#[test]
fn open_transaction_rejects_target_name_swap_before_fingerprint() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let moved = temp.path().join("main-open-moved");
    init_repo(&repo_root, false).unwrap();
    set_open_target_swap_before_fingerprint_for_test(&repo_root, &target, &moved);

    let error = open_branch_from(&repo_root, &transactional_open_options(&target)).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("identity changed during verification"),
        "{error}"
    );
    assert!(fs::symlink_metadata(&target)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(moved.join(".codefire-open").is_file());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_ne!(
        branch_state(&load_branch_record(&repo_root, "main").unwrap()),
        "open-clean"
    );
    let marker = read_json(&open_transaction_path(&repo_root)).unwrap();
    assert_eq!(marker["phase"], "prepared");
}

#[cfg(unix)]
#[test]
fn open_transaction_rejects_symlink_inserted_before_fingerprint() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let outside = temp.path().join("outside.txt");
    let outside_before = b"outside sentinel\n";
    fs::write(&outside, outside_before).unwrap();
    init_repo(&repo_root, false).unwrap();
    set_open_target_symlink_before_fingerprint_for_test(
        &repo_root,
        &target,
        "escape-link",
        &outside,
    );

    let error = open_branch_from(&repo_root, &transactional_open_options(&target)).unwrap_err();

    assert!(
        error.to_string().contains("rejects symlink entry"),
        "{error}"
    );
    assert_eq!(fs::read(&outside).unwrap(), outside_before);
    assert!(fs::symlink_metadata(target.join("escape-link"))
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_ne!(
        branch_state(&load_branch_record(&repo_root, "main").unwrap()),
        "open-clean"
    );
    let marker = read_json(&open_transaction_path(&repo_root)).unwrap();
    assert_eq!(marker["phase"], "prepared");
}

#[cfg(unix)]
#[test]
fn open_transaction_rejects_file_replaced_after_capability_read() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let moved_original = temp.path().join("original-a-first.txt");
    let replacement = temp.path().join("replacement-a-first.txt");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_two_file_commit(&objects);
    let mut branch = load_branch_record(&repo_root, "main").unwrap();
    branch["head"] = Value::String(commit_id);
    save_branch_record(&repo_root, &branch).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    fs::write(&replacement, b"replacement\n").unwrap();
    set_manifest_file_swap_before_rebind_for_test(
        &target,
        Path::new("a-first.txt"),
        &moved_original,
        &replacement,
    );

    let error = open_branch_from(&repo_root, &transactional_open_options(&target)).unwrap_err();

    assert!(error.to_string().contains("identity changed"), "{error}");
    assert_eq!(fs::read(&moved_original).unwrap(), b"first\n");
    assert_eq!(
        fs::read(target.join("a-first.txt")).unwrap(),
        b"replacement\n"
    );
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    let marker = read_json(&open_transaction_path(&repo_root)).unwrap();
    assert_eq!(marker["phase"], "prepared");
}

#[test]
fn pending_open_rejects_a_different_request_before_writing() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let other_target = temp.path().join("other-open");
    init_repo(&repo_root, false).unwrap();
    let options = transactional_open_options(&target);
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &options).unwrap_err();
    let marker_before = fs::read(open_transaction_path(&repo_root)).unwrap();

    let conflict =
        open_branch_from(&repo_root, &transactional_open_options(&other_target)).unwrap_err();

    assert_eq!(conflict.exit_code(), ExitCode::IdempotencyConflict.code());
    assert_eq!(
        fs::read(open_transaction_path(&repo_root)).unwrap(),
        marker_before
    );
    assert!(!other_target.exists());
    open_branch_from(&repo_root, &options).unwrap();
}

#[test]
fn pending_open_branch_publication_rejects_nonexistent_branch_before_lookup() {
    for failpoint in [
        "after_branch_object_before_reference",
        "after_branch_reference_before_phase",
    ] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let target = temp.path().join(format!("main-open-{failpoint}"));
        init_repo(&repo_root, false).unwrap();
        let options = transactional_open_options(&target);
        set_open_transaction_failure_for_test(&repo_root, failpoint);
        open_branch_from(&repo_root, &options).unwrap_err();
        let marker_before = fs::read(open_transaction_path(&repo_root)).unwrap();
        let mut conflict_options = transactional_open_options(&target);
        conflict_options.branch = "branch-that-does-not-exist".to_string();

        let conflict = open_branch_from(&repo_root, &conflict_options).unwrap_err();

        assert!(matches!(conflict, CliError::IdempotencyConflict(_)));
        assert_eq!(
            fs::read(open_transaction_path(&repo_root)).unwrap(),
            marker_before,
            "{failpoint}"
        );
        open_branch_from(&repo_root, &options).unwrap();
    }
}

#[test]
fn open_dry_run_does_not_create_a_transaction_marker() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let mut options = transactional_open_options(&target);
    options.dry_run = true;

    let result = open_branch_from(&repo_root, &options).unwrap();

    assert_eq!(result.plan["dry_run"], true);
    assert!(!target.exists());
    assert!(!open_transaction_path(&repo_root).exists());
}

#[test]
fn doctor_reports_pending_open_transaction_evidence() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let options = transactional_open_options(&target);
    set_open_transaction_failure_for_test(&repo_root, "after_target_publish_before_phase");
    open_branch_from(&repo_root, &options).unwrap_err();

    let report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: true,
        quick: true,
    })
    .unwrap();

    let issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_open_transaction")
        .expect("pending open diagnostic");
    assert!(issue.message.contains("phase prepared"));
    assert!(issue.message.contains("target=Exact"));
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "resume_pending_open"));
    open_branch_from(&repo_root, &options).unwrap();
}

fn transactional_close_options(branch: &str) -> CloseOptions {
    CloseOptions {
        branch: branch.to_string(),
        discard: true,
        keep_dir: false,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    }
}

fn prepare_open_feature(repo_root: &Path, branch: &str, target: &Path) {
    clone_branch(
        repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: branch.to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    open_branch_from(
        repo_root,
        &OpenOptions {
            branch: branch.to_string(),
            path: target.to_path_buf(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
}

#[test]
fn close_transaction_recovers_every_publication_boundary() {
    for failpoint in [
        "after_prepared",
        "after_worktree_action_before_phase",
        "after_worktree_applied",
        "after_active_state_remove_before_phase",
        "after_active_state_removed",
        "after_registry_remove_before_phase",
        "after_registry_removed",
        "after_branch_reference_before_phase",
        "after_branch_written",
        "after_complete_archive_before_marker_removal",
        "after_pending_marker_removed",
    ] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let target = temp.path().join(format!("feature-open-{failpoint}"));
        let branch = "feature/close-transaction";
        init_repo(&repo_root, false).unwrap();
        prepare_open_feature(&repo_root, branch, &target);
        let options = transactional_close_options(branch);
        set_close_transaction_failure_for_test(&repo_root, failpoint);

        let interrupted = close_branch(&repo_root, &options).unwrap_err();
        assert!(
            interrupted.to_string().contains(failpoint),
            "{failpoint}: {interrupted}"
        );
        let recovered = close_branch(&repo_root, &options).unwrap();
        let replayed = close_branch(&repo_root, &options).unwrap();

        assert_eq!(recovered.branch, branch, "{failpoint}");
        assert_eq!(recovered.open_dir, target, "{failpoint}");
        assert_eq!(replayed.plan, recovered.plan, "{failpoint}");
        assert!(!target.exists(), "{failpoint}");
        assert!(
            !opened_registry_path(&repo_root, branch).exists(),
            "{failpoint}"
        );
        assert_eq!(
            branch_state(&load_branch_record(&repo_root, branch).unwrap()),
            "closed",
            "{failpoint}"
        );
        assert!(!close_transaction_path(&repo_root).exists(), "{failpoint}");
        assert!(
            repo_root
                .join(".codefire/close_transaction.completed.json")
                .is_file(),
            "{failpoint}"
        );
    }
}

#[test]
fn close_transaction_recovers_object_first_branch_gap() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("feature-open");
    let branch = "feature/object-first-close";
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, branch, &target);
    let mut after = load_branch_record(&repo_root, branch).unwrap();
    after["state"] = Value::String("closed".to_string());
    let object_id = codefire_store::object_id("branch", &after).unwrap();
    let object_path =
        codefire_store::object_record_path(&repo_root.join(".codefire/objects"), &object_id)
            .unwrap();
    fs::remove_file(object_path).unwrap();
    let options = transactional_close_options(branch);
    set_close_transaction_failure_for_test(&repo_root, "after_branch_object_before_reference");

    let error = close_branch(&repo_root, &options).unwrap_err();
    assert!(error
        .to_string()
        .contains("after_branch_object_before_reference"));
    assert_ne!(
        branch_state(&load_branch_record(&repo_root, branch).unwrap()),
        "closed"
    );

    close_branch(&repo_root, &options).unwrap();
    assert_eq!(
        branch_state(&load_branch_record(&repo_root, branch).unwrap()),
        "closed"
    );
    assert_eq!(
        codefire_store::read_object(&repo_root.join(".codefire/objects"), &object_id).unwrap(),
        after
    );
}

#[test]
fn close_dry_run_reports_pending_authority_without_mutation() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("feature-open");
    let branch = "feature/close-dry-run";
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, branch, &target);
    let options = transactional_close_options(branch);
    set_close_transaction_failure_for_test(&repo_root, "after_worktree_action_before_phase");
    close_branch(&repo_root, &options).unwrap_err();
    let marker_before = fs::read(close_transaction_path(&repo_root)).unwrap();
    let registry_before = fs::read(opened_registry_path(&repo_root, branch)).unwrap();
    let mut dry_run = transactional_close_options(branch);
    dry_run.dry_run = true;

    let error = close_branch(&repo_root, &dry_run).unwrap_err();

    assert!(error
        .to_string()
        .contains("rerun the exact discard request"));
    assert_eq!(
        fs::read(close_transaction_path(&repo_root)).unwrap(),
        marker_before
    );
    assert_eq!(
        fs::read(opened_registry_path(&repo_root, branch)).unwrap(),
        registry_before
    );
    close_branch(&repo_root, &options).unwrap();
}

#[test]
fn sequential_distinct_closes_rotate_the_completed_authority() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let first = "feature/first-close";
    let second = "feature/second-close";
    let first_target = temp.path().join("first-open");
    let second_target = temp.path().join("second-open");
    prepare_open_feature(&repo_root, first, &first_target);
    prepare_open_feature(&repo_root, second, &second_target);

    close_branch(&repo_root, &transactional_close_options(first)).unwrap();
    close_branch(&repo_root, &transactional_close_options(second)).unwrap();

    assert!(!first_target.exists());
    assert!(!second_target.exists());
    let archive = read_json(&repo_root.join(".codefire/close_transaction.completed.json")).unwrap();
    assert_eq!(archive["request"]["branch"], second);
}

#[test]
fn pending_close_is_reported_by_doctor() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("feature-open");
    let branch = "feature/doctor-close";
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, branch, &target);
    let options = transactional_close_options(branch);
    set_close_transaction_failure_for_test(&repo_root, "after_registry_remove_before_phase");
    close_branch(&repo_root, &options).unwrap_err();

    let report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: true,
        quick: true,
    })
    .unwrap();

    let issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_close_transaction")
        .expect("pending close diagnostic");
    assert!(issue.message.contains("registry=Missing"));
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "resume_pending_close"));
    close_branch(&repo_root, &options).unwrap();
}

#[test]
fn close_rejects_pending_active_transactions_without_writes() {
    for marker in ["state_transaction.json", "commit_transaction.json"] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let target = temp.path().join(format!("feature-open-{marker}"));
        let branch = "feature/pending-active-close";
        init_repo(&repo_root, false).unwrap();
        prepare_open_feature(&repo_root, branch, &target);
        let registry_path = opened_registry_path(&repo_root, branch);
        let registry_before = fs::read(&registry_path).unwrap();
        let registry: Value = serde_json::from_slice(&registry_before).unwrap();
        let active = validated_active_state_dir(&repo_root, &registry).unwrap();
        let pending = if marker == "state_transaction.json" {
            json!({
                "version": 1,
                "transaction_id": "state_tx_0123456789abcdef0123456789abcdef",
                "status": "pending",
                "operation": "scan",
                "phase": "started",
                "intended_state": "open-clean",
                "started_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            })
        } else {
            json!({
                "version": 1,
                "transaction_id": "commit_tx_0123456789abcdef0123456789abcdef",
                "status": "pending",
                "phase": "started",
                "branch": branch,
                "open_dir": target,
                "parents": [],
                "message": "pending",
                "request_payload": {},
                "idempotency_key": null,
                "operation_plan": {},
                "objects": [],
                "started_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            })
        };
        if marker == "state_transaction.json" {
            validate_state_transaction_record(&pending).unwrap();
        } else {
            validate_commit_transaction_record(&pending).unwrap();
        }
        let mut bytes = serde_json::to_vec_pretty(&pending).unwrap();
        bytes.push(b'\n');
        active
            .write_atomic(Path::new(marker), &bytes, 1024 * 1024)
            .unwrap();
        let branch_before = load_branch_record(&repo_root, branch).unwrap();

        let error = close_branch(&repo_root, &transactional_close_options(branch)).unwrap_err();

        assert!(error.to_string().contains("pending"), "{marker}: {error}");
        assert!(!close_transaction_path(&repo_root).exists(), "{marker}");
        assert!(target.join(".codefire-open").is_file(), "{marker}");
        assert_eq!(
            fs::read(&registry_path).unwrap(),
            registry_before,
            "{marker}"
        );
        assert_eq!(
            load_branch_record(&repo_root, branch).unwrap(),
            branch_before,
            "{marker}"
        );
    }
}

#[test]
fn close_and_discard_reject_pending_active_ledger_publication_without_writes() {
    for (discard, ledger_pending) in [(false, false), (true, false), (false, true), (true, true)] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join(format!("repo-{discard}-{ledger_pending}"));
        let target = temp
            .path()
            .join(format!("feature-open-{discard}-{ledger_pending}"));
        let branch = "feature/pending-active-ledger-close";
        init_repo(&repo_root, false).unwrap();
        prepare_open_feature(&repo_root, branch, &target);
        let registry_path = opened_registry_path(&repo_root, branch);
        let registry: Value = read_json(&registry_path).unwrap();
        let active = validated_active_state_dir(&repo_root, &registry).unwrap();
        let request = json!({
            "type": "test_close_active_ledger_request",
            "version": 1,
            "branch": branch,
            "discard": discard,
        });
        if ledger_pending {
            crate::fire::active_ledger_transaction::set_failure(&active, "after_prepare");
        }
        let transition = apply_active_ledger_transition(
            &active,
            "fire",
            &request,
            vec![ActiveLedgerWrite::new(
                "fires.json",
                None,
                json!([{"owner": "pending-close"}]),
            )],
            &json!({"type": "test_close_active_ledger_result", "version": 1}),
        );
        if ledger_pending {
            transition.unwrap_err();
        } else {
            transition.unwrap();
        }
        let ledger_path = active.display_path(Path::new(
            crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE,
        ));
        let fires_path = active.display_path(Path::new("fires.json"));
        let before = (
            fs::read(&registry_path).unwrap(),
            fs::read(branch_record_path(&repo_root, branch)).unwrap(),
            fs::read(&ledger_path).unwrap(),
            fs::read(&fires_path).ok(),
            fs::read(target.join(".codefire-open")).unwrap(),
        );
        let mut options = transactional_close_options(branch);
        options.discard = discard;
        let expected_phase = if ledger_pending {
            "prepared"
        } else {
            "state_publication_pending"
        };

        let error = close_branch(&repo_root, &options).unwrap_err();

        assert!(matches!(
            &error,
            CliError::OpenRecoveryPending(pending)
                if pending.operation == "fire" && pending.phase == expected_phase
        ));
        assert!(!close_transaction_path(&repo_root).exists());
        assert_eq!(fs::read(&registry_path).unwrap(), before.0);
        assert_eq!(
            fs::read(branch_record_path(&repo_root, branch)).unwrap(),
            before.1
        );
        assert_eq!(fs::read(&ledger_path).unwrap(), before.2);
        assert_eq!(fs::read(&fires_path).ok(), before.3);
        assert_eq!(fs::read(target.join(".codefire-open")).unwrap(), before.4);
    }
}

#[test]
fn open_and_close_transactions_block_each_other() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let feature_target = temp.path().join("feature-open");
    let pending_target = temp.path().join("pending-open");
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, "feature/cross-protocol-close", &feature_target);
    clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature/pending-open".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let pending_open = OpenOptions {
        branch: "feature/pending-open".to_string(),
        path: pending_target,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &pending_open).unwrap_err();

    let error = close_branch(
        &repo_root,
        &transactional_close_options("feature/cross-protocol-close"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("open transaction is pending"));
    assert!(!close_transaction_path(&repo_root).exists());

    open_branch_from(&repo_root, &pending_open).unwrap();
    let close_options = transactional_close_options("feature/cross-protocol-close");
    set_close_transaction_failure_for_test(&repo_root, "after_prepared");
    close_branch(&repo_root, &close_options).unwrap_err();
    let another_open = OpenOptions {
        branch: "main".to_string(),
        path: temp.path().join("main-open"),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    let error = open_branch_from(&repo_root, &another_open).unwrap_err();
    assert!(error.to_string().contains("close transaction is pending"));
    let primary_error = close_branch(&repo_root, &transactional_close_options("main")).unwrap_err();
    assert!(
        primary_error
            .to_string()
            .contains("pending close transaction")
            || primary_error.exit_code() == ExitCode::IdempotencyConflict.code()
    );
    close_branch(&repo_root, &close_options).unwrap();
}

#[test]
fn doctor_reports_simultaneous_open_and_close_authorities() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let feature_target = temp.path().join("feature-open");
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, "feature/conflicting-close", &feature_target);
    clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature/conflicting-open".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let open_options = OpenOptions {
        branch: "feature/conflicting-open".to_string(),
        path: temp.path().join("conflicting-open"),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &open_options).unwrap_err();
    let open_path = open_transaction_path(&repo_root);
    let held_open = repo_root.join(".codefire/held-open-transaction.json");
    fs::rename(&open_path, &held_open).unwrap();
    let close_options = transactional_close_options("feature/conflicting-close");
    set_close_transaction_failure_for_test(&repo_root, "after_prepared");
    close_branch(&repo_root, &close_options).unwrap_err();
    fs::rename(&held_open, &open_path).unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: true,
        quick: true,
    })
    .unwrap();
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.kind == "conflicting_repository_transactions"));
    let error = close_branch(&repo_root, &close_options).unwrap_err();
    assert!(error
        .to_string()
        .contains("simultaneous pending open and close"));
}

#[test]
fn branch_parent_swap_before_close_publication_writes_nowhere() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("feature-open");
    let branch = "feature/branch-parent-swap";
    init_repo(&repo_root, false).unwrap();
    prepare_open_feature(&repo_root, branch, &target);
    let options = transactional_close_options(branch);
    set_close_transaction_failure_for_test(&repo_root, "after_registry_removed");
    close_branch(&repo_root, &options).unwrap_err();
    let parent = repo_root.join(".codefire/branches");
    let moved = repo_root.join(".codefire/moved-branches");
    let before = load_branch_record(&repo_root, branch).unwrap();
    set_close_branch_parent_swap_for_test(&repo_root, &parent, &moved);

    let error = close_branch(&repo_root, &options).unwrap_err();

    assert!(error.to_string().contains("identity"));
    assert_eq!(
        read_json(&moved.join(format!("{}.json", ref_file_name(branch)))).unwrap(),
        before
    );
    assert_eq!(
        fs::read(parent.join("outside-sentinel.txt")).unwrap(),
        b"outside\n"
    );
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
    fs::remove_dir_all(&parent).unwrap();
    fs::rename(&moved, &parent).unwrap();
    close_branch(&repo_root, &options).unwrap();
}

#[test]
fn doctor_rejects_divergent_published_open_target() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let options = transactional_open_options(&target);
    set_open_transaction_failure_for_test(&repo_root, "after_target_publish_before_phase");
    open_branch_from(&repo_root, &options).unwrap_err();
    fs::write(target.join("foreign.txt"), b"foreign\n").unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: true,
        quick: true,
    })
    .unwrap();

    let issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "invalid_open_transaction")
        .expect("divergent published target diagnostic");
    assert!(issue.message.contains("differs from the sealed base"));
    assert!(!report
        .issues
        .iter()
        .any(|issue| issue.kind == "pending_open_transaction"));
}

#[cfg(unix)]
#[test]
fn open_partial_materialization_failure_cleans_stage_and_retry_succeeds() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let repo_root = workspace.join("repo");
    let target = workspace.join("main-open");
    let outside = temp.path().join("outside");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_two_file_commit(&objects);
    let mut branch = load_branch_record(&repo_root, "main").unwrap();
    branch["head"] = Value::String(commit_id);
    save_branch_record(&repo_root, &branch).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();

    set_open_target_nested_symlink_injection_for_test(
        &target,
        &target,
        Path::new("docs"),
        &outside,
    );
    let options = || OpenOptions {
        branch: "main".to_string(),
        path: target.clone(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    let error = open_branch_from(&repo_root, &options()).unwrap_err();

    assert!(matches!(
        &error,
        CliError::OpenTargetResolution(details)
            if details.reason == "identity_changed_during_use"
    ));
    assert!(!target.exists());
    assert!(!outside.join("readme.md").exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert!(open_staging_leftovers(&workspace).is_empty());

    let result = open_branch_from(&repo_root, &options()).unwrap();

    assert_eq!(result.open_dir, target);
    assert!(result.plan["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| operation["kind"] == "publish_open_target"
            && operation["visibility"] == "single_sibling_rename"));
    assert_eq!(
        fs::read_to_string(result.open_dir.join("a-first.txt")).unwrap(),
        "first\n"
    );
    assert_eq!(
        fs::read_to_string(result.open_dir.join("docs/readme.md")).unwrap(),
        "second\n"
    );
    assert!(result.open_dir.join(".codefire-open").is_file());
    assert!(opened_registry_path(&repo_root, "main").is_file());
    assert!(open_staging_leftovers(&workspace).is_empty());
}

#[cfg(unix)]
fn write_two_file_commit(objects: &Path) -> String {
    let first = codefire_store::store_object(
        objects,
        "blob",
        json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(b"first\n"),
        }),
    )
    .unwrap();
    let second = codefire_store::store_object(
        objects,
        "blob",
        json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(b"second\n"),
        }),
    )
    .unwrap();
    let manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({
            "type": "content_manifest",
            "version": 1,
            "entries": [
                {"path": "a-first.txt", "kind": "file", "mode": "100644", "blob": first},
                {"path": "docs/readme.md", "kind": "file", "mode": "100644", "blob": second},
            ],
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(objects);
    roots.insert("content_manifest".to_string(), Value::String(manifest));
    codefire_store::store_object(
        objects,
        "commit",
        codefire_store::commit_payload(vec![], roots, consistent_certificate()),
    )
    .unwrap()
}

#[cfg(unix)]
fn open_staging_leftovers(workspace: &Path) -> Vec<PathBuf> {
    let mut leftovers = fs::read_dir(workspace)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".codefire-open-stage-"))
        })
        .collect::<Vec<_>>();
    leftovers.sort();
    leftovers
}

#[test]
fn close_target_swap_preserves_replacement_and_repository_state() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("feature-open");
    let moved_original = temp.path().join("moved-feature-open");
    init_repo(&repo_root, false).unwrap();
    clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature/close-race".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "feature/close-race".to_string(),
            path: target.clone(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(target.join("original.txt"), b"original\n").unwrap();
    let registry_path = opened_registry_path(&repo_root, "feature/close-race");
    let registry_before = fs::read(&registry_path).unwrap();
    let branch_before = load_branch_record(&repo_root, "feature/close-race").unwrap();
    let active_state = validated_active_state_dir(
        &repo_root,
        &serde_json::from_slice(&registry_before).unwrap(),
    )
    .unwrap();
    let active_path = active_state.path().to_path_buf();
    set_close_target_swap_injection_for_test(&target, &moved_original);

    let error = close_branch(
        &repo_root,
        &CloseOptions {
            branch: "feature/close-race".to_string(),
            discard: true,
            keep_dir: false,
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("close worktree identity changed"),
        "unexpected close error: {error}"
    );
    assert_eq!(
        fs::read_to_string(target.join("victim.txt")).unwrap(),
        "victim\n"
    );
    assert_eq!(
        fs::read_to_string(moved_original.join("original.txt")).unwrap(),
        "original\n"
    );
    assert!(moved_original.join(".codefire-open").is_file());
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert_eq!(
        load_branch_record(&repo_root, "feature/close-race").unwrap(),
        branch_before
    );
    assert!(active_path.is_dir());

    fs::remove_dir_all(&target).unwrap();
    fs::rename(&moved_original, &target).unwrap();
    close_branch(
        &repo_root,
        &CloseOptions {
            branch: "feature/close-race".to_string(),
            discard: true,
            keep_dir: false,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert!(!target.exists());
    assert!(!registry_path.exists());
}
