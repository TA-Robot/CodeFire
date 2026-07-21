use super::*;

const SCAN_FAILPOINT_PHASES: &[&str] = &[
    "started",
    "verification_invalidated",
    "fires_written",
    "scan_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];

const VERIFY_FAILPOINT_PHASES: &[&str] = &[
    "started",
    "verification_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];

const COMMIT_ACTIVE_RESET_FAILPOINTS: &[&str] = &[
    "after_active_reset_remove_fires",
    "after_active_reset_remove_resolutions",
    "after_active_reset_remove_scan",
    "after_active_reset_remove_verification",
    "after_active_reset_remove_active_ledger_transaction",
    "after_active_reset_remove_state_transaction",
    "after_active_reset_state_write",
];

const COMMIT_ACTIVE_RESET_FILES: &[&str] = &[
    "fires.json",
    "resolutions.json",
    "scan.json",
    "verification.json",
    crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE,
    "state_transaction.json",
];

#[test]
fn scan_state_transaction_recovers_every_publication_phase() {
    for phase in SCAN_FAILPOINT_PHASES {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let open_dir = temp.path().join(format!("main-open-{phase}"));
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

        // Give scan a real verification authority to invalidate. This keeps
        // the early phases observable instead of testing only absent files.
        assert_eq!(run_verify(&open_dir).unwrap().result, "passed");
        let active_path = active_state_path(&open_dir);
        set_state_transaction_failure_for_test(&active_path, "scan", phase);

        let error = run_scan(&open_dir).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("scan state transaction failure at {phase}")),
            "{phase}: {error}"
        );
        let marker = read_json(&active_path.join("state_transaction.json")).unwrap();
        assert_eq!(marker["operation"], "scan", "{phase}");
        assert_eq!(marker["phase"], *phase, "{phase}");

        if *phase != "started" {
            set_state_transaction_failure_for_test(&active_path, "scan", "started");
            let retry_error = run_scan(&open_dir).unwrap_err();
            assert!(
                retry_error
                    .to_string()
                    .contains("scan state transaction failure at started"),
                "{phase}: {retry_error}"
            );
            let preserved = read_json(&active_path.join("state_transaction.json")).unwrap();
            assert_eq!(preserved["phase"], *phase, "{phase}");
            open_context_for_recovery(&open_dir, "scan").unwrap();
        }

        run_scan(&open_dir).unwrap();
        assert_final_state_authorities(&repo_root, &open_dir, false, phase);
    }
}

#[test]
fn verify_state_transaction_recovers_every_publication_phase() {
    for phase in VERIFY_FAILPOINT_PHASES {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let open_dir = temp.path().join(format!("main-open-{phase}"));
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

        run_scan(&open_dir).unwrap();
        let active_path = active_state_path(&open_dir);
        set_state_transaction_failure_for_test(&active_path, "verify", phase);

        let error = run_verify(&open_dir).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("verify state transaction failure at {phase}")),
            "{phase}: {error}"
        );
        let marker = read_json(&active_path.join("state_transaction.json")).unwrap();
        assert_eq!(marker["operation"], "verify", "{phase}");
        assert_eq!(marker["phase"], *phase, "{phase}");

        if *phase != "started" {
            set_state_transaction_failure_for_test(&active_path, "verify", "started");
            let retry_error = run_verify(&open_dir).unwrap_err();
            assert!(
                retry_error
                    .to_string()
                    .contains("verify state transaction failure at started"),
                "{phase}: {retry_error}"
            );
            let preserved = read_json(&active_path.join("state_transaction.json")).unwrap();
            assert_eq!(preserved["phase"], *phase, "{phase}");
            open_context_for_recovery(&open_dir, "verify").unwrap();
        }

        assert_eq!(run_verify(&open_dir).unwrap().result, "passed");
        assert_final_state_authorities(&repo_root, &open_dir, true, phase);
    }
}

#[test]
fn active_ledger_state_transaction_recovers_every_publication_phase() {
    for operation in ["fire", "fire-batch", "extinguish", "extinguish-batch"] {
        for failpoint in ACTIVE_LEDGER_STATE_TRANSACTION_PHASES
            .iter()
            .copied()
            .chain(std::iter::once("after_registry_write_before_phase"))
        {
            let temp = tempdir().unwrap();
            let repo_root = temp.path().join("repo");
            let open_dir = temp.path().join(format!("{operation}-{failpoint}"));
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

            let context = open_context(&open_dir).unwrap();
            let owning_request = active_ledger_state_request(operation, "owner");
            let durable_result = json!({
                "type": "test_active_ledger_state_result",
                "version": 1,
                "operation": operation,
            });
            apply_active_ledger_transition(
                &context.active_state,
                operation,
                &owning_request,
                vec![ActiveLedgerWrite::new(
                    "fires.json",
                    None,
                    json!([{"operation": operation, "owner": "owner"}]),
                )],
                &durable_result,
            )
            .unwrap();
            set_state_transaction_failure_for_test(
                context.active_state.path(),
                operation,
                failpoint,
            );
            let error =
                set_open_state_transactional(&context, operation, "open-burning", &owning_request)
                    .unwrap_err();
            assert!(
                error.to_string().contains(&format!(
                    "{operation} state transaction failure at {failpoint}"
                )),
                "{operation}/{failpoint}: {error}"
            );
            let transaction_path = context
                .active_state
                .display_path(Path::new("state_transaction.json"));
            let marker = read_json(&transaction_path).unwrap();
            let expected_phase = if failpoint == "after_registry_write_before_phase" {
                "started"
            } else {
                failpoint
            };
            assert_eq!(marker["version"], 2, "{operation}/{failpoint}");
            assert_eq!(marker["operation"], operation, "{operation}/{failpoint}");
            assert_eq!(marker["phase"], expected_phase, "{operation}/{failpoint}");
            assert_eq!(
                marker["request_hash"],
                active_ledger_request_hash(&owning_request).unwrap(),
                "{operation}/{failpoint}"
            );
            if failpoint == "after_registry_write_before_phase" {
                let pre_state = marker["pre_state"]["branch"].as_str().unwrap();
                assert_eq!(
                    read_json(&opened_registry_path(&repo_root, "main")).unwrap()["state"]
                        ["last_known"],
                    "open-burning",
                    "{operation}/{failpoint}"
                );
                assert_eq!(
                    read_active_json_required(&context.active_state, Path::new("state.json"))
                        .unwrap()["state"],
                    pre_state,
                    "{operation}/{failpoint}"
                );
                assert_eq!(
                    load_branch_record(&repo_root, "main").unwrap()["state"],
                    pre_state,
                    "{operation}/{failpoint}"
                );
            }

            let retry = open_context_for_recovery(&open_dir, operation).unwrap();
            let before_foreign_retry =
                active_ledger_state_publication_snapshot(&repo_root, &retry.active_state);
            let foreign_request = active_ledger_state_request(operation, "foreign");
            let foreign_error =
                set_open_state_transactional(&retry, operation, "open-burning", &foreign_request)
                    .unwrap_err();
            assert!(
                matches!(foreign_error, CliError::IdempotencyConflict(_)),
                "{operation}/{failpoint}: {foreign_error}"
            );
            assert_eq!(
                active_ledger_state_publication_snapshot(&repo_root, &retry.active_state),
                before_foreign_retry,
                "{operation}/{failpoint}: foreign request changed publication authority"
            );

            if expected_phase != "started" {
                set_state_transaction_failure_for_test(
                    retry.active_state.path(),
                    operation,
                    "started",
                );
                let retry_error = set_open_state_transactional(
                    &retry,
                    operation,
                    "open-burning",
                    &owning_request,
                )
                .unwrap_err();
                assert!(
                    retry_error
                        .to_string()
                        .contains(&format!("{operation} state transaction failure at started")),
                    "{operation}/{failpoint}: {retry_error}"
                );
                assert_eq!(
                    read_json(&transaction_path).unwrap()["phase"],
                    expected_phase,
                    "{operation}/{failpoint}: exact retry rewound the marker"
                );
                open_context_for_recovery(&open_dir, operation).unwrap();
            }

            let final_retry = open_context_for_recovery(&open_dir, operation).unwrap();
            assert_eq!(
                recover_active_ledger_transition(
                    &final_retry.active_state,
                    operation,
                    &owning_request,
                )
                .unwrap(),
                Some(durable_result)
            );
            set_open_state_transactional(&final_retry, operation, "open-burning", &owning_request)
                .unwrap();
            assert!(!transaction_path.exists(), "{operation}/{failpoint}");
            let registry = read_json(&opened_registry_path(&repo_root, "main")).unwrap();
            let branch = load_branch_record(&repo_root, "main").unwrap();
            let active =
                read_active_json_required(&final_retry.active_state, Path::new("state.json"))
                    .unwrap();
            assert_eq!(
                registry["state"]["last_known"], "open-burning",
                "{operation}/{failpoint}"
            );
            assert_eq!(branch["state"], "open-burning", "{operation}/{failpoint}");
            assert_eq!(active["state"], "open-burning", "{operation}/{failpoint}");
        }
    }
}

#[test]
fn active_ledger_and_state_marker_must_share_operation_request_and_status() {
    for mismatch in ["operation", "request_hash"] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join(format!("repo-{mismatch}"));
        let open_dir = temp.path().join(format!("open-{mismatch}"));
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
        let context = open_context(&open_dir).unwrap();
        let request = active_ledger_state_request("fire", "owner");
        apply_active_ledger_transition(
            &context.active_state,
            "fire",
            &request,
            vec![ActiveLedgerWrite::new(
                "fires.json",
                None,
                json!([{"owner": "owner"}]),
            )],
            &json!({"type": "pair-test-result", "version": 1}),
        )
        .unwrap();
        set_state_transaction_failure_for_test(context.active_state.path(), "fire", "started");
        set_open_state_transactional(&context, "fire", "open-burning", &request).unwrap_err();
        let marker_path = context
            .active_state
            .display_path(Path::new("state_transaction.json"));
        let mut marker = read_json(&marker_path).unwrap();
        match mismatch {
            "operation" => marker["operation"] = Value::String("fire-batch".to_string()),
            "request_hash" => {
                marker["request_hash"] = Value::String(
                    active_ledger_request_hash(&active_ledger_state_request("fire", "foreign"))
                        .unwrap(),
                )
            }
            _ => unreachable!(),
        }
        validate_state_transaction_record(&marker).unwrap();
        write_json_atomic(&marker_path, &marker).unwrap();
        let before = active_ledger_state_publication_snapshot(&repo_root, &context.active_state);

        let error = match open_context_for_recovery(&open_dir, "fire") {
            Err(error) => error,
            Ok(_) => panic!("{mismatch}: mismatched pair was accepted"),
        };
        assert_active_ledger_pair_divergence(&error, mismatch);
        assert_eq!(
            active_ledger_state_publication_snapshot(&repo_root, &context.active_state),
            before,
            "{mismatch}: mismatched pair changed authority"
        );
    }

    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo-status");
    let open_dir = temp.path().join("open-status");
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
    let context = open_context(&open_dir).unwrap();
    let request = active_ledger_state_request("fire", "pending-status");
    crate::fire::active_ledger_transaction::set_failure(&context.active_state, "after_prepare");
    apply_active_ledger_transition(
        &context.active_state,
        "fire",
        &request,
        vec![ActiveLedgerWrite::new(
            "fires.json",
            None,
            json!([{"owner": "pending-status"}]),
        )],
        &json!({"type": "pair-test-result", "version": 1}),
    )
    .unwrap_err();
    let request_hash = active_ledger_request_hash(&request).unwrap();
    begin_state_transaction_with_request_hash(
        &context,
        "fire",
        "open-burning",
        Some(&request_hash),
    )
    .unwrap();
    let before = active_ledger_state_publication_snapshot(&repo_root, &context.active_state);

    let error = match open_context_for_recovery(&open_dir, "fire") {
        Err(error) => error,
        Ok(_) => panic!("pending ledger/state status mismatch was accepted"),
    };
    assert_active_ledger_pair_divergence(&error, "ledger-status");
    assert_eq!(
        active_ledger_state_publication_snapshot(&repo_root, &context.active_state),
        before,
        "pending ledger/state pair changed authority"
    );
}

#[test]
fn final_scan_authority_predicate_preserves_marker_on_divergence() {
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
    run_scan(&open_dir).unwrap();

    let context = open_context(&open_dir).unwrap();
    let fires: Vec<codefire_core::Fire> = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("fires.json")).unwrap(),
    )
    .unwrap();
    let scan: codefire_core::ScanResult = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("scan.json")).unwrap(),
    )
    .unwrap();
    stage_complete_state_transaction(&context, "scan", "open-clean");

    let mut corrupted = serde_json::to_value(&scan).unwrap();
    corrupted["base_commit"] = Value::String("CF-COMMIT-divergent".to_string());
    write_active_json_atomic(&context.active_state, Path::new("scan.json"), &corrupted).unwrap();

    let error = complete_state_transaction(
        &context,
        "open-clean",
        StateTransactionAuthority::Scan {
            scan: &scan,
            fires: &fires,
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("persisted scan differs"));
    assert!(context
        .active_state
        .exists(Path::new("state_transaction.json"))
        .unwrap());
}

#[test]
fn final_verify_authority_predicate_preserves_marker_on_divergence() {
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
    assert_eq!(run_verify(&open_dir).unwrap().result, "passed");

    let context = open_context(&open_dir).unwrap();
    let fires: Vec<codefire_core::Fire> = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("fires.json")).unwrap(),
    )
    .unwrap();
    let scan: codefire_core::ScanResult = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("scan.json")).unwrap(),
    )
    .unwrap();
    let verification: codefire_core::Verification = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("verification.json")).unwrap(),
    )
    .unwrap();
    stage_complete_state_transaction(&context, "verify", "open-clean");

    let mut corrupted = serde_json::to_value(&verification).unwrap();
    corrupted["open_required_fires"] = Value::Number(1_u64.into());
    write_active_json_atomic(
        &context.active_state,
        Path::new("verification.json"),
        &corrupted,
    )
    .unwrap();

    let error = complete_state_transaction(
        &context,
        "open-clean",
        StateTransactionAuthority::Verify {
            scan: &scan,
            fires: &fires,
            verification: &verification,
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("persisted verification differs"));
    assert!(context
        .active_state
        .exists(Path::new("state_transaction.json"))
        .unwrap());
}

#[test]
fn commit_active_reset_recovers_every_apply_before_progress_gap_exactly_once() {
    for failpoint in COMMIT_ACTIVE_RESET_FAILPOINTS {
        let temp = tempdir().unwrap();
        let (repo_root, open_dir, options) =
            prepare_commit_active_reset_fixture(temp.path(), failpoint, false);
        let active_path = active_state_path(&open_dir);
        let marker_path = active_path.join("commit_transaction.json");
        set_commit_transaction_failure_for_test(&active_path, failpoint);

        let error = run_commit(&options).unwrap_err();
        assert!(
            error.to_string().contains(failpoint),
            "{failpoint}: {error}"
        );
        let interrupted_marker = read_json(&marker_path).unwrap();
        validate_commit_transaction_record(&interrupted_marker).unwrap();
        assert_eq!(
            interrupted_marker["phase"], "resetting_active_state",
            "{failpoint}"
        );
        let candidate = interrupted_marker["candidate_commit"]["object_id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(
            required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
            candidate,
            "{failpoint}"
        );
        assert_eq!(
            required_string(
                &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
                &["open", "current_base_commit"]
            )
            .unwrap(),
            candidate,
            "{failpoint}"
        );

        let interrupted_removals = interrupted_marker["active_reset"]["removals"]
            .as_array()
            .unwrap();
        if *failpoint == "after_active_reset_state_write" {
            assert!(interrupted_removals
                .iter()
                .all(|entry| entry["progress"] == "removed"));
        } else {
            let interrupted_index = COMMIT_ACTIVE_RESET_FAILPOINTS
                .iter()
                .position(|candidate| candidate == failpoint)
                .unwrap();
            for (index, removal) in interrupted_removals.iter().enumerate() {
                let expected = if index < interrupted_index {
                    "removed"
                } else {
                    "planned"
                };
                assert_eq!(removal["progress"], expected, "{failpoint}: {index}");
            }
        }
        assert_eq!(
            interrupted_marker["active_reset"]["state_file"]["progress"], "planned",
            "{failpoint}"
        );

        let recovered = run_commit(&options).unwrap();
        assert_eq!(recovered.commit_id, candidate, "{failpoint}");
        assert!(!marker_path.exists(), "{failpoint}");
        for path in COMMIT_ACTIVE_RESET_FILES {
            assert!(!active_path.join(path).exists(), "{failpoint}: {path}");
        }
        assert_eq!(
            read_json(&active_path.join("state.json")).unwrap(),
            json!({"state": "open-clean"}),
            "{failpoint}"
        );
        let transaction_id = interrupted_marker["transaction_id"].as_str().unwrap();
        let archive = read_json(&active_path.join(format!(
            "commit_transaction.recovered.{transaction_id}.json"
        )))
        .unwrap();
        assert!(commit_active_reset_complete(&archive["active_reset"]));
        assert_eq!(archive["candidate_commit"]["object_id"], candidate);
    }
}

#[test]
fn commit_active_reset_applier_recovers_each_present_file_and_state_write_gap() {
    for (failpoint_index, failpoint) in COMMIT_ACTIVE_RESET_FAILPOINTS.iter().enumerate() {
        let temp = tempdir().unwrap();
        let active_path = temp.path().join(format!("active-{failpoint_index}"));
        let active = active_state::ActiveStateDir::create(&active_path).unwrap();
        for (index, path) in COMMIT_ACTIVE_RESET_FILES.iter().enumerate() {
            active
                .write_atomic(
                    Path::new(path),
                    format!("{{\"file\":{index}}}\n").as_bytes(),
                    REPOSITORY_JSON_BYTE_LIMIT,
                )
                .unwrap();
        }
        active
            .write_atomic(
                Path::new("state.json"),
                b"{\"state\":\"open-dirty\"}\n",
                REPOSITORY_JSON_BYTE_LIMIT,
            )
            .unwrap();
        let transaction_id = format!("commit_tx_{failpoint_index:032x}");
        let transaction = json!({
            "version": 1,
            "transaction_id": transaction_id,
            "status": "pending",
            "phase": "resetting_active_state",
            "branch": "main",
            "open_dir": temp.path(),
            "parents": ["CF-COMMIT-parent"],
            "message": "active reset applier",
            "request_payload": {},
            "idempotency_key": null,
            "operation_plan": {},
            "active_reset": build_commit_active_reset_authority(&active).unwrap(),
            "objects": ["CF-OBJECT-legacy"],
            "started_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        });
        validate_commit_transaction_record(&transaction).unwrap();
        write_pending_commit_transaction(&active, &transaction).unwrap();
        set_commit_transaction_failure_for_test(&active_path, failpoint);

        let error = reset_commit_active_state(&active, &transaction_id).unwrap_err();
        assert!(
            error.to_string().contains(failpoint),
            "{failpoint}: {error}"
        );
        let interrupted = read_commit_transaction(&active).unwrap();
        if failpoint_index < COMMIT_ACTIVE_RESET_FILES.len() {
            assert!(!active
                .exists(Path::new(COMMIT_ACTIVE_RESET_FILES[failpoint_index]))
                .unwrap());
            assert_eq!(
                interrupted["active_reset"]["removals"][failpoint_index]["progress"], "planned",
                "{failpoint}"
            );
        } else {
            assert_eq!(
                active
                    .read_required(Path::new("state.json"), REPOSITORY_JSON_BYTE_LIMIT)
                    .unwrap(),
                commit_active_reset_state_postimage_bytes().unwrap()
            );
            assert_eq!(
                interrupted["active_reset"]["state_file"]["progress"],
                "planned"
            );
        }

        reset_commit_active_state(&active, &transaction_id).unwrap();
        let completed = read_commit_transaction(&active).unwrap();
        assert!(commit_active_reset_complete(&completed["active_reset"]));
        for path in COMMIT_ACTIVE_RESET_FILES {
            assert!(
                !active.exists(Path::new(path)).unwrap(),
                "{failpoint}: {path}"
            );
        }
        assert_eq!(
            active
                .read_required(Path::new("state.json"), REPOSITORY_JSON_BYTE_LIMIT)
                .unwrap(),
            commit_active_reset_state_postimage_bytes().unwrap()
        );
    }
}

#[test]
fn commit_active_reset_upgrades_a_legacy_publishing_marker_before_reset() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "legacy-publishing", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_branch_publish_before_registry");
    run_commit(&options).unwrap_err();
    let mut legacy_marker = read_json(&active_path.join("commit_transaction.json")).unwrap();
    assert_eq!(legacy_marker["phase"], "publishing_branch");
    let candidate = legacy_marker["candidate_commit"]["object_id"]
        .as_str()
        .unwrap()
        .to_string();
    legacy_marker
        .as_object_mut()
        .unwrap()
        .remove("active_reset");
    validate_commit_transaction_record(&legacy_marker).unwrap();
    let active = active_state::ActiveStateDir::bind(&active_path).unwrap();
    write_pending_commit_transaction(&active, &legacy_marker).unwrap();

    let recovered = run_commit(&options).unwrap();

    assert_eq!(recovered.commit_id, candidate);
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        candidate
    );
    assert_eq!(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "current_base_commit"]
        )
        .unwrap(),
        candidate
    );
    let transaction_id = legacy_marker["transaction_id"].as_str().unwrap();
    let archive = read_json(&active_path.join(format!(
        "commit_transaction.recovered.{transaction_id}.json"
    )))
    .unwrap();
    assert!(commit_active_reset_complete(&archive["active_reset"]));
}

#[test]
fn commit_active_reset_legacy_partial_reset_requires_inspection_without_writes() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "legacy-partial", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_active_reset_remove_fires");
    run_commit(&options).unwrap_err();
    let mut legacy_marker = read_json(&active_path.join("commit_transaction.json")).unwrap();
    assert_eq!(legacy_marker["phase"], "resetting_active_state");
    legacy_marker
        .as_object_mut()
        .unwrap()
        .remove("active_reset");
    validate_commit_transaction_record(&legacy_marker).unwrap();
    let active = active_state::ActiveStateDir::bind(&active_path).unwrap();
    write_pending_commit_transaction(&active, &legacy_marker).unwrap();
    let active_before_retry = snapshot_directory_files(&active_path);
    let branch_before_retry = load_branch_record(&repo_root, "main").unwrap();
    let registry_before_retry = read_json(&opened_registry_path(&repo_root, "main")).unwrap();

    let error = run_commit(&options).unwrap_err();

    assert!(error.to_string().contains("requires explicit inspection"));
    assert_eq!(snapshot_directory_files(&active_path), active_before_retry);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before_retry
    );
    assert_eq!(
        read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        registry_before_retry
    );
}

#[test]
fn commit_active_reset_legacy_completed_reset_can_finalize() {
    let temp = tempdir().unwrap();
    let (_repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "legacy-complete", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_active_reset_before_marker_clear");
    run_commit(&options).unwrap_err();
    let mut legacy_marker = read_json(&active_path.join("commit_transaction.json")).unwrap();
    assert_eq!(legacy_marker["phase"], "resetting_active_state");
    let candidate = legacy_marker["candidate_commit"]["object_id"]
        .as_str()
        .unwrap()
        .to_string();
    legacy_marker
        .as_object_mut()
        .unwrap()
        .remove("active_reset");
    validate_commit_transaction_record(&legacy_marker).unwrap();
    let active = active_state::ActiveStateDir::bind(&active_path).unwrap();
    write_pending_commit_transaction(&active, &legacy_marker).unwrap();

    let recovered = run_commit(&options).unwrap();

    assert_eq!(recovered.commit_id, candidate);
    assert!(!active_path.join("commit_transaction.json").exists());
}

#[test]
fn commit_active_reset_divergent_planned_file_fails_before_any_additional_reset_write() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "planned-divergence", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_active_reset_remove_fires");
    run_commit(&options).unwrap_err();

    fs::write(
        active_path.join("scan.json"),
        b"{\n  \"divergent\": true\n}\n",
    )
    .unwrap();
    let active_before_retry = snapshot_directory_files(&active_path);
    let branch_before_retry = load_branch_record(&repo_root, "main").unwrap();
    let registry_before_retry = read_json(&opened_registry_path(&repo_root, "main")).unwrap();

    let error = run_commit(&options).unwrap_err();
    assert!(error
        .to_string()
        .contains("durable commit active-state reset found divergent content"));
    assert!(error.to_string().contains("scan.json"));
    assert_eq!(snapshot_directory_files(&active_path), active_before_retry);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before_retry
    );
    assert_eq!(
        read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        registry_before_retry
    );
}

#[test]
fn commit_active_reset_reappearing_removed_file_fails_closed() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "removed-reappearance", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_active_reset_remove_resolutions");
    run_commit(&options).unwrap_err();
    let marker = read_json(&active_path.join("commit_transaction.json")).unwrap();
    assert_eq!(marker["active_reset"]["removals"][0]["progress"], "removed");
    assert_eq!(marker["active_reset"]["removals"][1]["progress"], "planned");

    fs::write(active_path.join("fires.json"), b"[]\n").unwrap();
    let active_before_retry = snapshot_directory_files(&active_path);
    let branch_before_retry = load_branch_record(&repo_root, "main").unwrap();
    let registry_before_retry = read_json(&opened_registry_path(&repo_root, "main")).unwrap();

    let error = run_commit(&options).unwrap_err();
    assert!(error
        .to_string()
        .contains("durable commit active-state reset found divergent content"));
    assert!(error.to_string().contains("fires.json"));
    assert_eq!(snapshot_directory_files(&active_path), active_before_retry);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before_retry
    );
    assert_eq!(
        read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        registry_before_retry
    );
}

#[test]
fn commit_active_reset_divergent_state_postimage_fails_closed() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "state-divergence", false);
    let active_path = active_state_path(&open_dir);
    set_commit_transaction_failure_for_test(&active_path, "after_active_reset_state_write");
    run_commit(&options).unwrap_err();
    let marker = read_json(&active_path.join("commit_transaction.json")).unwrap();
    assert!(marker["active_reset"]["removals"]
        .as_array()
        .unwrap()
        .iter()
        .all(|removal| removal["progress"] == "removed"));
    assert_eq!(marker["active_reset"]["state_file"]["progress"], "planned");

    fs::write(
        active_path.join("state.json"),
        b"{\n  \"state\": \"open-divergent\"\n}\n",
    )
    .unwrap();
    let active_before_retry = snapshot_directory_files(&active_path);
    let branch_before_retry = load_branch_record(&repo_root, "main").unwrap();
    let registry_before_retry = read_json(&opened_registry_path(&repo_root, "main")).unwrap();

    let error = run_commit(&options).unwrap_err();
    assert!(error
        .to_string()
        .contains("durable commit active-state reset found divergent content"));
    assert!(error.to_string().contains("state.json"));
    assert_eq!(snapshot_directory_files(&active_path), active_before_retry);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before_retry
    );
    assert_eq!(
        read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        registry_before_retry
    );
}

#[test]
fn commit_dry_run_does_not_create_or_advance_active_reset_authority() {
    let temp = tempdir().unwrap();
    let (repo_root, open_dir, options) =
        prepare_commit_active_reset_fixture(temp.path(), "dry-run", true);
    let active_path = active_state_path(&open_dir);
    let active_before = snapshot_directory_files(&active_path);
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    let registry_before = read_json(&opened_registry_path(&repo_root, "main")).unwrap();

    let result = run_commit(&options).unwrap();

    assert!(result.dry_run);
    assert!(result.commit_id.is_empty());
    assert_eq!(snapshot_directory_files(&active_path), active_before);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(
        read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        registry_before
    );
    assert!(!active_path.join("commit_transaction.json").exists());
}

#[test]
fn commit_active_reset_authority_shape_rejects_digest_order_and_progress_tampering() {
    let temp = tempdir().unwrap();
    let (_repo_root, open_dir, _options) =
        prepare_commit_active_reset_fixture(temp.path(), "shape", false);
    let context = open_context(&open_dir).unwrap();
    let authority = build_commit_active_reset_authority(&context.active_state).unwrap();
    let transaction = json!({
        "version": 1,
        "transaction_id": "commit_tx_0123456789abcdef0123456789abcdef",
        "status": "pending",
        "phase": "started",
        "branch": "main",
        "open_dir": open_dir,
        "parents": [],
        "message": "shape",
        "request_payload": {},
        "idempotency_key": null,
        "operation_plan": {},
        "active_reset": authority,
        "objects": [],
        "started_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    });
    validate_commit_transaction_record(&transaction).unwrap();

    let mut bad_digest = transaction.clone();
    bad_digest["active_reset"]["state_file"]["postimage"]["sha256"] =
        Value::String("not-a-digest".to_string());
    assert!(validate_commit_transaction_record(&bad_digest)
        .unwrap_err()
        .contains("64-character hex digest"));

    let mut bad_order = transaction.clone();
    bad_order["active_reset"]["removals"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert!(validate_commit_transaction_record(&bad_order)
        .unwrap_err()
        .contains("removal path must be fires.json"));

    let mut out_of_order_progress = transaction.clone();
    out_of_order_progress["active_reset"]["removals"][1]["progress"] =
        Value::String("removed".to_string());
    assert!(validate_commit_transaction_record(&out_of_order_progress)
        .unwrap_err()
        .contains("ordered removed prefix"));

    let mut early_state_write = transaction.clone();
    early_state_write["active_reset"]["state_file"]["progress"] =
        Value::String("written".to_string());
    assert!(validate_commit_transaction_record(&early_state_write)
        .unwrap_err()
        .contains("before every removal completes"));

    let mut bad_progress = transaction;
    bad_progress["active_reset"]["removals"][0]["progress"] = Value::String("removed".to_string());
    assert!(validate_commit_transaction_record(&bad_progress)
        .unwrap_err()
        .contains("cannot contain active_reset progress"));
}

fn active_ledger_state_request(operation: &str, owner: &str) -> Value {
    json!({
        "type": "test_active_ledger_state_request",
        "version": 1,
        "operation": operation,
        "owner": owner,
    })
}

fn assert_active_ledger_pair_divergence(error: &CliError, case: &str) {
    match error {
        CliError::OpenSnapshotDivergence(divergence) => assert_eq!(
            divergence.authority, "pending_active_ledger_transaction",
            "{case}"
        ),
        other => panic!(
            "{case}: expected active-ledger snapshot divergence, got {other}: {:?}",
            other.diagnostic()
        ),
    }
    assert_eq!(error.diagnostic()["kind"], "open_snapshot_divergence");
}

fn active_ledger_state_publication_snapshot(
    repo_root: &Path,
    active_state: &active_state::ActiveStateDir,
) -> (
    Vec<u8>,
    Vec<u8>,
    std::collections::BTreeMap<String, Vec<u8>>,
) {
    (
        fs::read(branch_record_path(repo_root, "main")).unwrap(),
        fs::read(opened_registry_path(repo_root, "main")).unwrap(),
        snapshot_directory_files(active_state.path()),
    )
}

fn stage_complete_state_transaction(context: &OpenContext, operation: &str, intended_state: &str) {
    begin_state_transaction(context, operation, intended_state).unwrap();
    let phases = match operation {
        "scan" => &SCAN_FAILPOINT_PHASES[1..],
        "verify" => &VERIFY_FAILPOINT_PHASES[1..],
        other => panic!("unsupported test operation: {other}"),
    };
    for phase in phases {
        update_state_transaction_phase(&context.active_state, phase).unwrap();
    }
}

fn prepare_commit_active_reset_fixture(
    root: &Path,
    name: &str,
    dry_run: bool,
) -> (PathBuf, PathBuf, CommitOptions) {
    let repo_root = root.join(format!("repo-{name}"));
    let open_dir = root.join(format!("main-open-{name}"));
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
    fs::write(
        open_dir.join("README.md"),
        format!("commit active reset fixture {name}\n"),
    )
    .unwrap();
    assert_eq!(run_verify(&open_dir).unwrap().result, "passed");
    let context = open_context(&open_dir).unwrap();
    write_active_json_atomic(
        &context.active_state,
        Path::new("resolutions.json"),
        &json!([]),
    )
    .unwrap();
    drop(context);
    let options = CommitOptions {
        path: open_dir.clone(),
        message: format!("Commit active reset {name}"),
        dry_run,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    };
    (repo_root, open_dir, options)
}

fn snapshot_directory_files(path: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}

fn assert_final_state_authorities(
    repo_root: &Path,
    open_dir: &Path,
    expect_verification: bool,
    phase: &str,
) {
    let registry = read_json(&opened_registry_path(repo_root, "main")).unwrap();
    let active_path = active_state_path(open_dir);
    let active_record = read_json(&active_path.join("state.json")).unwrap();
    let branch = load_branch_record(repo_root, "main").unwrap();
    let fires: Vec<codefire_core::Fire> =
        serde_json::from_value(read_json(&active_path.join("fires.json")).unwrap()).unwrap();
    let scan: codefire_core::ScanResult =
        serde_json::from_value(read_json(&active_path.join("scan.json")).unwrap()).unwrap();
    let open_fires = fires
        .iter()
        .filter(|fire| fire.status == "open")
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(scan.open_fires, open_fires, "{phase}");
    assert_eq!(
        registry["open"]["current_base_commit"], scan.base_commit,
        "{phase}"
    );
    assert_eq!(branch["head"], scan.base_commit, "{phase}");

    let expected_state = if expect_verification {
        let verification: codefire_core::Verification =
            serde_json::from_value(read_json(&active_path.join("verification.json")).unwrap())
                .unwrap();
        assert_eq!(
            verification.open_required_fires,
            scan.open_fires
                .iter()
                .filter(|fire| fire.status == "open" && fire.severity == "required")
                .count(),
            "{phase}"
        );
        assert_eq!(
            verification.duplicate_atom_ids, scan.atom_index.duplicate_atom_ids,
            "{phase}"
        );
        verification_open_state(&verification, &scan)
    } else {
        assert!(!active_path.join("verification.json").exists(), "{phase}");
        scan_branch_state(scan_change_count(&scan), scan.open_fires.len())
    };
    assert_eq!(registry["state"]["last_known"], expected_state, "{phase}");
    assert_eq!(active_record["state"], expected_state, "{phase}");
    assert_eq!(branch["state"], expected_state, "{phase}");
    assert!(
        !active_path.join("state_transaction.json").exists(),
        "{phase}"
    );
}
