use super::*;

#[test]
fn required_signed_candidate_with_tampered_digest_is_rejected_before_reference_publication() {
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
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "tampered signature boundary\n").unwrap();
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_commit_signature: true\n",
    )
    .unwrap();

    let active = active_state_path(&open_dir);
    let registry_path = opened_registry_path(&repo_root, "main");
    let branch_head_before =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    let registry_before = read_json(&registry_path).unwrap();
    let registry_base_before =
        required_string(&registry_before, &["open", "current_base_commit"]).unwrap();
    let _signing_key =
        signatures::override_commit_signing_key_for_test(Some("trusted-local-signing-key"));
    set_commit_candidate_corruption_for_test(
        &active,
        CommitCandidateCorruptionKind::InvalidSignature,
    );

    let error = run_commit(&CommitOptions {
        path: open_dir,
        message: "Tampered signed candidate must not publish".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: Some("local-signer".to_string()),
        key_id: Some("local-key".to_string()),
    })
    .unwrap_err();

    assert!(matches!(error, CliError::AuthenticationOrSignature(_)));
    assert!(error.to_string().contains("invalid signature"), "{error}");
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        branch_head_before
    );
    assert_eq!(
        required_string(
            &read_json(&registry_path).unwrap(),
            &["open", "current_base_commit"]
        )
        .unwrap(),
        registry_base_before
    );
    let transaction_path = active.join("commit_transaction.json");
    let transaction = read_json(&transaction_path).unwrap();
    assert_eq!(transaction["status"], "pending");
    assert_eq!(transaction["phase"], "validating_candidate_commit");
    assert_eq!(commit_transaction_state_counts(&transaction), (10, 1));
    let candidate = commit_transaction_object_entry(&transaction, "candidate_commit", None);
    assert_eq!(candidate["state"], "stored");
    let candidate_id = candidate["object_id"].as_str().unwrap();
    let candidate_record =
        codefire_store::read_object(&repo_root.join(".codefire").join("objects"), candidate_id)
            .unwrap();
    assert_eq!(
        candidate_record["signature"]["signature"],
        Value::String("0".repeat(64))
    );
    let envelope = cli_error_envelope("commit", &error);
    assert_eq!(
        envelope["diagnostics"][0]["kind"],
        "authentication_or_signature_error"
    );

    let doctor = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: false,
        json_output: true,
    })
    .unwrap();
    let pending = doctor
        .issues
        .iter()
        .find(|issue| issue.kind == "pending_commit_transaction")
        .expect("tampered signed candidate must remain diagnosable");
    assert!(pending.message.contains("validating_candidate_commit"));
}
