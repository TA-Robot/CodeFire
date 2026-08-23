use super::*;
use crate::doctor::{
    DoctorPendingAuthorityKind, DoctorPendingOperation, DoctorPendingRequestIdentity,
};

fn open_main(repo_root: &Path, open_dir: &Path) {
    init_repo(repo_root, false).unwrap();
    open_branch_from(
        repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.to_path_buf(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
}

fn diagnostic_for(report: &crate::doctor::DoctorReport, kind: &str) -> Value {
    doctor_report_diagnostics_json(report)
        .into_iter()
        .find(|diagnostic| diagnostic["kind"] == kind)
        .unwrap_or_else(|| panic!("missing doctor diagnostic {kind}"))
}

fn action_for(report: &crate::doctor::DoctorReport, kind: &str) -> Value {
    doctor_report_next_actions(report)
        .into_iter()
        .find(|action| action["kind"] == kind)
        .unwrap_or_else(|| panic!("missing doctor action {kind}"))
}

#[test]
fn extinguished_manual_fire_in_open_clean_state_is_doctor_clean() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    open_main(&repo_root, &open_dir);
    fs::create_dir_all(open_dir.join("docs/requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs/design")).unwrap();
    fs::write(
        open_dir.join("docs/requirements/doctor-clean.md"),
        "## REQ-doctor-clean: Requirement\nDoctor clean fixture\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs/design/doctor-clean.md"),
        "## DES-doctor-clean: Design\nDoctor clean fixture\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Seed doctor clean Fire atoms".to_string(),
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();

    let opened = fire::run_fire(&fire::FireOptions {
        path: open_dir.clone(),
        source_atom: "REQ-doctor-clean".to_string(),
        target_atom: "DES-doctor-clean".to_string(),
        reason: "doctor must count only open Fires".to_string(),
        severity: "required".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: opened.fires[0].display_id.clone(),
        resolution: "obsolete".to_string(),
        rationale: "manual Fire is intentionally extinguished".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        refresh: false,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        edit_rationale: false,
    })
    .unwrap();
    assert_eq!(read_status(&open_dir).unwrap().state, "open-clean");
    let fires = read_json(&active_state_path(&open_dir).join("fires.json")).unwrap();
    assert_eq!(fires.as_array().unwrap().len(), 1);
    assert_eq!(fires[0]["status"], "extinguished");

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert!(report.ok, "doctor issues: {:?}", report.issues);
    assert!(!report
        .issues
        .iter()
        .any(|issue| issue.kind == "active_state_invariant_violation"));
}

#[test]
fn doctor_commit_pending_contract_is_typed_bounded_and_read_only() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    open_main(&repo_root, &open_dir);
    fs::write(open_dir.join("README.md"), "pending commit fixture\n").unwrap();

    let active = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active, "after_content_manifest");
    let secret_message = "DO_NOT_EXPOSE_PENDING_COMMIT_MESSAGE";
    let secret_key = "DO_NOT_EXPOSE_PENDING_COMMIT_IDEMPOTENCY_KEY";
    let error = run_commit(&CommitOptions {
        path: open_dir,
        message: secret_message.to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: Some(secret_key.to_string()),
        signer: None,
        key_id: None,
    })
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("injected commit transaction failure"));

    let marker_path = active.join("commit_transaction.json");
    let marker_before = fs::read(&marker_path).unwrap();
    let marker: Value = serde_json::from_slice(&marker_before).unwrap();
    let transaction_id = marker["transaction_id"].as_str().unwrap();
    let phase = marker["phase"].as_str().unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert_eq!(fs::read(&marker_path).unwrap(), marker_before);

    let issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_commit_transaction")
        .expect("doctor must retain typed pending commit authority");
    let pending = issue.pending_authority.as_ref().unwrap();
    assert_eq!(pending.authority, DoctorPendingAuthorityKind::Commit);
    assert_eq!(pending.operation, DoctorPendingOperation::Commit);
    assert_eq!(pending.phase, phase);
    assert_eq!(
        pending.request_identity,
        DoctorPendingRequestIdentity::TransactionId {
            transaction_id: transaction_id.to_string(),
        }
    );

    let diagnostic = diagnostic_for(&report, "pending_commit_transaction");
    assert_eq!(diagnostic["authority"], "commit_transaction");
    assert_eq!(diagnostic["operation"], "commit");
    assert_eq!(diagnostic["phase"], phase);
    assert_eq!(diagnostic["request_identity"]["kind"], "transaction_id");
    assert_eq!(
        diagnostic["request_identity"]["transaction_id"],
        transaction_id
    );
    let diagnostic_text = serde_json::to_string(&diagnostic).unwrap();
    assert!(!diagnostic_text.contains(secret_message));
    assert!(!diagnostic_text.contains(secret_key));
    assert!(!diagnostic_text.contains("request_payload"));
    assert!(!diagnostic_text.contains("operation_plan"));

    let action = action_for(&report, "inspect_pending_commit_transaction");
    assert!(action["command"]
        .as_str()
        .unwrap()
        .starts_with("codefire doctor "));
    assert!(!action["command"]
        .as_str()
        .unwrap()
        .contains("codefire commit"));
    assert_eq!(
        action["target"]["type"],
        "codefire_pending_authority_target"
    );
    assert_eq!(action["target"]["operation"], "commit");
    assert_eq!(action["target"]["phase"], phase);
    assert_eq!(action["target"]["owner"]["command"], "commit");
    assert_eq!(
        action["target"]["owner"]["retry_contract"],
        "exact_request_only"
    );
    assert_eq!(action["target"]["inspection"]["mode"], "read_only");
    assert_eq!(action["target"]["automatic_repair"], false);
}

#[test]
fn doctor_state_pending_contract_uses_request_hash_and_exact_owner_without_payload() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    open_main(&repo_root, &open_dir);
    fs::create_dir_all(open_dir.join("docs/requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs/design")).unwrap();
    fs::write(
        open_dir.join("docs/requirements/pending.md"),
        "## REQ-doctor-pending: Requirement\nPending authority fixture\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs/design/pending.md"),
        "## DES-doctor-pending: Design\nPending authority fixture\n",
    )
    .unwrap();

    let active = active_state_path(&open_dir);
    set_state_transaction_failure_for_test(&active, "fire", "branch_written");
    let secret_reason = "DO_NOT_EXPOSE_PENDING_FIRE_REQUEST_PAYLOAD";
    let error = fire::run_fire(&fire::FireOptions {
        path: open_dir,
        source_atom: "REQ-doctor-pending".to_string(),
        target_atom: "DES-doctor-pending".to_string(),
        reason: secret_reason.to_string(),
        severity: "required".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("fire state transaction failure at branch_written"));

    let marker_path = active.join("state_transaction.json");
    let ledger_path =
        active.join(crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE);
    let marker_before = fs::read(&marker_path).unwrap();
    let ledger_before = fs::read(&ledger_path).unwrap();
    let marker: Value = serde_json::from_slice(&marker_before).unwrap();
    let request_hash = marker["request_hash"].as_str().unwrap();
    assert_eq!(marker["operation"], "fire");
    assert_eq!(marker["phase"], "branch_written");

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert_eq!(fs::read(&marker_path).unwrap(), marker_before);
    assert_eq!(fs::read(&ledger_path).unwrap(), ledger_before);

    let issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_state_transaction")
        .expect("doctor must retain typed pending state authority");
    let pending = issue.pending_authority.as_ref().unwrap();
    assert_eq!(pending.authority, DoctorPendingAuthorityKind::State);
    assert_eq!(pending.operation, DoctorPendingOperation::Fire);
    assert_eq!(pending.phase, "branch_written");
    assert_eq!(pending.intended_state.as_deref(), Some("open-burning"));
    assert_eq!(
        pending.request_identity,
        DoctorPendingRequestIdentity::RequestHash {
            request_hash: request_hash.to_string(),
        }
    );

    let diagnostic = diagnostic_for(&report, "pending_state_transaction");
    assert_eq!(diagnostic["authority"], "state_transaction");
    assert_eq!(diagnostic["operation"], "fire");
    assert_eq!(diagnostic["phase"], "branch_written");
    assert_eq!(diagnostic["intended_state"], "open-burning");
    assert_eq!(diagnostic["request_identity"]["kind"], "request_hash");
    assert_eq!(diagnostic["request_identity"]["request_hash"], request_hash);
    let diagnostic_text = serde_json::to_string(&diagnostic).unwrap();
    assert!(!diagnostic_text.contains(secret_reason));
    assert!(!diagnostic_text.contains("request_payload"));

    let action = action_for(&report, "inspect_pending_state_transaction");
    assert!(action["command"]
        .as_str()
        .unwrap()
        .starts_with("codefire doctor "));
    assert!(!action["command"]
        .as_str()
        .unwrap()
        .contains("codefire fire"));
    assert_eq!(action["target"]["operation"], "fire");
    assert_eq!(action["target"]["phase"], "branch_written");
    assert_eq!(action["target"]["owner"]["command"], "fire");
    assert_eq!(action["target"]["owner"]["mode"], "single");
    assert_eq!(
        action["target"]["owner"]["retry_contract"],
        "exact_request_only"
    );
    assert_eq!(action["target"]["inspection"]["mode"], "read_only");
    assert_eq!(action["target"]["automatic_repair"], false);

    let ledger_issue = report
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_active_ledger_transaction")
        .expect("doctor must retain typed pending active-ledger authority");
    let ledger_pending = ledger_issue.pending_authority.as_ref().unwrap();
    assert_eq!(
        ledger_pending.authority,
        DoctorPendingAuthorityKind::ActiveLedger
    );
    assert_eq!(ledger_pending.operation, DoctorPendingOperation::Fire);
    assert_eq!(ledger_pending.phase, "complete");
    assert_eq!(
        ledger_pending.request_identity,
        DoctorPendingRequestIdentity::RequestHash {
            request_hash: request_hash.to_string(),
        }
    );

    let ledger_diagnostic = diagnostic_for(&report, "pending_active_ledger_transaction");
    assert_eq!(ledger_diagnostic["authority"], "active_ledger_transaction");
    assert_eq!(ledger_diagnostic["operation"], "fire");
    assert_eq!(ledger_diagnostic["phase"], "complete");
    assert_eq!(
        ledger_diagnostic["request_identity"]["request_hash"],
        request_hash
    );
    assert_eq!(ledger_diagnostic["repairable"], false);
    let ledger_diagnostic_text = serde_json::to_string(&ledger_diagnostic).unwrap();
    assert!(!ledger_diagnostic_text.contains(secret_reason));
    assert!(!ledger_diagnostic_text.contains("result"));
    assert!(!ledger_diagnostic_text.contains("effects"));

    let ledger_action = action_for(&report, "inspect_pending_active_ledger_transaction");
    assert!(ledger_action["command"]
        .as_str()
        .unwrap()
        .starts_with("codefire doctor "));
    assert!(!ledger_action["command"]
        .as_str()
        .unwrap()
        .contains("codefire fire"));
    assert_eq!(ledger_action["target"]["operation"], "fire");
    assert_eq!(ledger_action["target"]["phase"], "complete");
    assert_eq!(ledger_action["target"]["owner"]["command"], "fire");
    assert_eq!(
        ledger_action["target"]["owner"]["retry_contract"],
        "exact_request_only"
    );
    assert_eq!(ledger_action["target"]["inspection"]["mode"], "read_only");
    assert_eq!(ledger_action["target"]["automatic_repair"], false);
}
