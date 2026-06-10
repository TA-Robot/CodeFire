use super::*;
use crate::doctor::DoctorOptions;
use crate::remote::{next_remote_generation, remote_idempotency_key, RemoteGenerationLock};
use serde_json::{json, Map};
use std::sync::{Arc, Barrier};
use tempfile::tempdir;

mod docs;

#[test]
fn command_help_routes_before_mutating_parsers() {
    for (args, expected) in [
        (vec!["init", "--help"], "usage: codefire init"),
        (vec!["open", "--help"], "usage: codefire open"),
        (vec!["clone", "--help"], "usage: codefire clone"),
        (vec!["upload", "--help"], "usage: codefire upload"),
        (
            vec!["request-merge", "--help"],
            "usage: codefire request-merge",
        ),
        (
            vec!["branch", "list", "--help"],
            "usage: codefire branch list",
        ),
        (
            vec!["evidence", "add", "--help"],
            "usage: codefire evidence add",
        ),
        (
            vec!["patch", "export", "--help"],
            "usage: codefire patch export",
        ),
        (vec!["context", "--help"], "usage: codefire context"),
        (vec!["explain", "--help"], "usage: codefire explain"),
        (vec!["migrate", "--help"], "usage: codefire migrate"),
        (
            vec!["migrate", "check", "--help"],
            "usage: codefire migrate check",
        ),
        (vec!["atom-index", "--help"], "usage: codefire atom-index"),
        (vec!["trace-graph", "--help"], "usage: codefire trace-graph"),
        (
            vec!["missing-links", "--help"],
            "usage: codefire missing-links",
        ),
    ] {
        let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
        let help = command_help_for_args(&args).expect("help should be routed");
        assert!(
            help.contains(expected),
            "help for {:?} did not contain {expected}: {help}",
            args
        );
    }

    let init_options = parse_init_args(&["--help".to_string()]).unwrap();
    assert_eq!(init_options.path, PathBuf::from("--help"));

    let branch_list_help = command_help_for_args(&[
        "branch".to_string(),
        "list".to_string(),
        "--help".to_string(),
    ])
    .expect("branch list help should be routed");
    assert!(branch_list_help.contains("--metrics"));
    let branch_help = command_help_for_args(&["branch".to_string(), "--help".to_string()])
        .expect("branch help should be routed");
    assert!(branch_help.contains("branch show"));
    let branch_show_help = command_help_for_args(&[
        "branch".to_string(),
        "show".to_string(),
        "--help".to_string(),
    ])
    .expect("branch show help should be routed");
    assert!(branch_show_help.contains("usage: codefire branch show"));
}

#[test]
fn read_only_debug_args_accept_path_and_json() {
    let options = parse_read_only_debug_args(
        &[
            "--path".to_string(),
            "/tmp/open".to_string(),
            "--json".to_string(),
        ],
        "atom-index",
    )
    .unwrap();
    assert_eq!(options.path, PathBuf::from("/tmp/open"));
    assert!(options.json_output);

    let options = parse_read_only_debug_args(
        &["--path=/tmp/open".to_string(), "--json".to_string()],
        "missing-links",
    )
    .unwrap();
    assert_eq!(options.path, PathBuf::from("/tmp/open"));
    assert!(options.json_output);

    let options = parse_read_only_debug_args(
        &["/tmp/open".to_string(), "--json".to_string()],
        "trace-graph",
    )
    .unwrap();
    assert_eq!(options.path, PathBuf::from("/tmp/open"));
    assert!(options.json_output);

    assert!(parse_read_only_debug_args(
        &["--path".to_string(), "/a".to_string(), "/b".to_string()],
        "atom-index",
    )
    .is_err());
}

#[test]
fn missing_evidence_ref_error_uses_json_envelope() {
    let repo_root = PathBuf::from("/repo");
    let envelope = cli_error_envelope(
        "extinguish",
        &CliError::MissingEvidenceRef {
            evidence_id: "CF-EVIDENCE-missing".to_string(),
            repo_root: repo_root.clone(),
        },
    );

    assert_eq!(envelope["schema"], "codefire.command_result.v1");
    assert_eq!(envelope["command"], "extinguish");
    assert_eq!(envelope["ok"], false);
    assert_eq!(
        envelope["exit_code"],
        ExitCode::ObjectReferenceInvalid.code()
    );
    assert_eq!(envelope["repo"], repo_root.to_string_lossy().as_ref());
    assert_eq!(envelope["diagnostics"][0]["kind"], "missing_evidence_ref");
    assert_eq!(
        envelope["diagnostics"][0]["evidence_id"],
        "CF-EVIDENCE-missing"
    );
    assert_eq!(envelope["next_actions"][0]["kind"], "create_evidence");
}

#[test]
fn unknown_target_errors_use_json_failure_envelope() {
    let context_error = CliError::Usage("unknown atom: REQ-missing".to_string());
    let context_envelope = cli_error_envelope("context", &context_error);
    assert_eq!(context_envelope["schema"], "codefire.command_result.v1");
    assert_eq!(context_envelope["command"], "context");
    assert_eq!(context_envelope["ok"], false);
    assert_eq!(
        context_envelope["exit_code"],
        ExitCode::InvalidUsageOrConfig.code()
    );
    assert_eq!(context_envelope["diagnostics"][0]["kind"], "unknown_target");
    assert_eq!(
        context_envelope["diagnostics"][0]["target"]["id"],
        "REQ-missing"
    );
    assert_eq!(context_envelope["next_actions"][0]["kind"], "refresh_scan");

    let explain_error = CliError::Usage("unknown fire: FIRE-missing".to_string());
    let explain_envelope = cli_error_envelope("explain", &explain_error);
    assert_eq!(explain_envelope["command"], "explain");
    assert_eq!(explain_envelope["diagnostics"][0]["kind"], "unknown_target");
    assert_eq!(explain_envelope["diagnostics"][0]["target"]["kind"], "fire");
}

#[test]
fn batch_file_read_errors_use_json_failure_envelope() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let missing = temp.path().join("missing-batch.json");
    init_repo(&repo_root, false).unwrap();

    let evidence_error = run_evidence_batch(&evidence::EvidenceAddOptions {
        start: repo_root,
        json_output: true,
        dry_run: true,
        batch_path: Some(missing.clone()),
        label: None,
        artifact_path: None,
        artifact_uri: None,
        command: None,
        command_argv: None,
        command_cwd: None,
        command_timeout_ms: 300_000,
        max_output_bytes: 64 * 1024,
        allow_failed_command: false,
    })
    .unwrap_err();
    assert_batch_missing_envelope("evidence-add-batch", &evidence_error);

    let link_error = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: temp.path().to_path_buf(),
        batch_path: missing.clone(),
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_batch_missing_envelope("link-batch", &link_error);

    let fire_error = fire::run_fire_batch(&fire::FireBatchOptions {
        path: temp.path().to_path_buf(),
        batch_path: missing.clone(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_batch_missing_envelope("fire-batch", &fire_error);

    let extinguish_error = run_extinguish_batch(&batch::BatchExtinguishOptions {
        path: temp.path().to_path_buf(),
        batch_path: missing,
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_batch_missing_envelope("extinguish-batch", &extinguish_error);
}

#[test]
fn batch_json_arrays_report_schema_errors_not_yaml_errors() {
    let temp = tempdir().unwrap();
    let batch_path = temp.path().join("fires.json");
    fs::write(&batch_path, "[]").unwrap();

    let error = run_extinguish_batch(&batch::BatchExtinguishOptions {
        path: temp.path().to_path_buf(),
        batch_path,
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    let envelope = cli_error_envelope("extinguish-batch", &error);

    assert_eq!(envelope["diagnostics"][0]["kind"], "batch_schema_error");
    assert!(error
        .to_string()
        .contains("batch extinguish JSON must be an object"));
    assert!(!error.to_string().contains("YAML"));
    assert_eq!(envelope["next_actions"][0]["kind"], "show_batch_schema");
}

#[test]
fn extinguish_help_documents_batch_wrapper_schema() {
    let help = subcommand_help("extinguish");

    assert!(help.contains("Batch JSON example"));
    assert!(help.contains("{\"version\":1,\"fires\""));
}

fn assert_batch_missing_envelope(command: &str, error: &CliError) {
    let envelope = cli_error_envelope(command, error);

    assert_eq!(envelope["schema"], "codefire.command_result.v1");
    assert_eq!(envelope["command"], command);
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["exit_code"], ExitCode::InvalidUsageOrConfig.code());
    assert_eq!(envelope["diagnostics"][0]["kind"], "batch_file_missing");
    assert_eq!(envelope["next_actions"][0]["kind"], "check_batch_path");
    assert_eq!(envelope["next_actions"][1]["kind"], "show_batch_schema");
}

#[test]
fn init_creates_python_compatible_repo_layout() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");

    let result = init_repo(&repo_root, false).unwrap();
    let branches = list_branches(&repo_root).unwrap();

    assert_eq!(result.repo_root, repo_root);
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert_eq!(branches[0].head, result.main_commit);
    assert_eq!(branches[0].state, "closed");
    for relative in [
        "objects",
        "branches",
        "opened",
        "active",
        "cache",
        "locks",
        "remotes",
        "idempotency",
    ] {
        assert!(repo_root.join(".codefire").join(relative).is_dir());
    }
    for subdir in codefire_store::known_object_subdirs() {
        assert!(repo_root
            .join(".codefire")
            .join("objects")
            .join(subdir)
            .is_dir());
    }
    codefire_store::validate_sealed_commit(
        &repo_root.join(".codefire").join("objects"),
        &result.main_commit,
    )
    .unwrap();
    assert!(init_repo(&repo_root, false).is_err());
    init_repo(&repo_root, true).unwrap();
}

#[test]
fn doctor_reports_clean_repo_and_corrupted_object_record() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let init = init_repo(&repo_root, false).unwrap();

    let clean = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: false,
        quick: false,
    })
    .unwrap();
    assert!(clean.ok);
    assert!(clean.checked_objects > 0);
    assert!(doctor_report_next_actions(&clean).is_empty());

    let commit_path = codefire_store::object_record_path(
        &repo_root.join(".codefire").join("objects"),
        &init.main_commit,
    )
    .unwrap();
    let mut commit_record = read_json(&commit_path).unwrap();
    commit_record["hash"] = Value::String("sha256-corrupted".to_string());
    write_json_atomic(&commit_path, &commit_record).unwrap();

    let corrupted = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: false,
        quick: false,
    })
    .unwrap();
    assert!(!corrupted.ok);
    assert!(corrupted
        .issues
        .iter()
        .any(|issue| issue.kind == "object_integrity_error"));
    assert!(doctor_report_diagnostics_json(&corrupted)
        .iter()
        .any(|issue| issue["kind"] == "object_integrity_error"
            && issue["category"] == "object_store"
            && issue["blocking"] == true
            && issue["repairable"] == false));
    let data = doctor_report_data_json(&corrupted);
    assert_eq!(data["mode"], "full");
    assert!(data["issue_counts"]["blocking"].as_u64().unwrap() >= 1);
    let doctor_actions = doctor_report_next_actions(&corrupted);
    assert!(doctor_actions
        .iter()
        .any(|action| action["id"] == "inspect_object_store_corruption"));
    assert_common_next_action_schema(&doctor_actions[0]);
}

#[test]
fn doctor_quick_mode_skips_full_object_record_scan() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let loose_invalid = repo_root
        .join(".codefire")
        .join("objects")
        .join("blobs")
        .join("CF-BLOB-invalid.json");
    fs::write(loose_invalid, "{not json").unwrap();

    let full = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: false,
        quick: false,
    })
    .unwrap();
    assert!(!full.ok);
    assert!(full
        .issues
        .iter()
        .any(|issue| issue.kind == "invalid_object_record"));

    let quick = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: false,
        quick: true,
    })
    .unwrap();
    assert!(quick.ok);
    assert_eq!(quick.checked_objects, 0);
    assert_eq!(quick.skipped_checks, vec!["object_store_integrity"]);
    let data = doctor_report_data_json(&quick);
    assert_eq!(data["mode"], "quick");
    assert_eq!(data["skipped_checks"][0], "object_store_integrity");
    assert!(quick
        .issues
        .iter()
        .all(|issue| issue.kind != "invalid_object_record"));
}

#[test]
fn doctor_checks_layout_active_state_and_open_marker_shape() {
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

    fs::remove_dir_all(repo_root.join(".codefire").join("locks")).unwrap();

    let registry_path = opened_registry_path(&repo_root, "main");
    let registry = read_json(&registry_path).unwrap();
    let active_state_path = PathBuf::from(
        registry["open"]["active_state_path"]
            .as_str()
            .expect("active state path"),
    );
    write_json_atomic(
        &active_state_path.join("fires.json"),
        &json!([{"display_id": "FIRE-1"}]),
    )
    .unwrap();

    let mut marker = read_json(&open_dir.join(".codefire-open")).unwrap();
    marker["open"]["open_instance_id"] = Value::String("open_wrong".to_string());
    write_json_atomic(&open_dir.join(".codefire-open"), &marker).unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: false,
        quick: true,
    })
    .unwrap();
    assert!(!report.ok);
    for expected in [
        "missing_repairable_directory",
        "invalid_active_state_shape",
        "open_marker_instance_mismatch",
    ] {
        assert!(
            report.issues.iter().any(|issue| issue.kind == expected),
            "missing doctor issue: {expected}"
        );
    }
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "reopen_branch_workspace"));
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "review_migration_plan"));
}

#[test]
fn doctor_reports_active_state_missing_file_and_invariant_violation() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    let registry_path = opened_registry_path(&repo_root, "main");
    let registry = read_json(&registry_path).unwrap();
    let active_state_path = PathBuf::from(
        registry["open"]["active_state_path"]
            .as_str()
            .expect("active state path"),
    );
    fs::remove_file(active_state_path.join("state.json")).unwrap();
    let missing = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: false,
        quick: true,
    })
    .unwrap();
    assert!(missing
        .issues
        .iter()
        .any(|issue| issue.kind == "missing_active_state_file"));

    write_json_atomic(
        &active_state_path.join("state.json"),
        &json!({"state": "open-clean"}),
    )
    .unwrap();
    write_json_atomic(
        &active_state_path.join("fires.json"),
        &json!([{
            "type": "fire",
            "version": 1,
            "fire_uid": "fire_test",
            "display_id": "FIRE-TEST",
            "status": "open",
            "severity": "required",
            "source": {"atom_id": "REQ-session"},
            "target": {"atom_id": "DES-session"},
            "reason": "required trace link missing",
            "trace_path": [],
            "created_by": "test",
            "created_at": "2026-06-10T00:00:00Z",
            "key": "fire-test"
        }]),
    )
    .unwrap();
    let inconsistent = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: false,
        quick: true,
    })
    .unwrap();
    assert!(inconsistent
        .issues
        .iter()
        .any(|issue| issue.kind == "active_state_invariant_violation"));
}

#[test]
fn doctor_detects_invalid_branch_head_and_open_registry() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    let branch_path = branch_record_path(&repo_root, "main");
    let mut branch = read_json(&branch_path).unwrap();
    branch["head"] = Value::String("CF-COMMIT-missing".to_string());
    write_json_atomic(&branch_path, &branch).unwrap();

    let registry_path = opened_registry_path(&repo_root, "main");
    let mut registry = read_json(&registry_path).unwrap();
    registry["open"]["current_base_commit"] = Value::String("CF-COMMIT-missing".to_string());
    registry["open"]["active_state_path"] =
        Value::String(temp.path().join("missing-active").display().to_string());
    write_json_atomic(&registry_path, &registry).unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        json_output: false,
        quick: false,
    })
    .unwrap();
    assert!(!report.ok);
    for expected in [
        "invalid_branch_head",
        "invalid_open_base_commit",
        "missing_active_state_dir",
    ] {
        assert!(
            report.issues.iter().any(|issue| issue.kind == expected),
            "missing doctor issue: {expected}"
        );
    }
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "repair_branch_head"));
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "reopen_branch_workspace"));
}

#[test]
fn open_materializes_manifest_and_updates_registry() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_file_commit(&objects, "docs/readme.md", "hello\n");
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": "main",
        "head": commit_id,
        "state": "closed",
        "created_at": "2026-06-04T00:00:00Z"
    });
    save_branch_record(&repo_root, &branch).unwrap();

    let result = open_branch_from(
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
    let status = read_status(&open_dir).unwrap();
    let branches = list_branches(&open_dir).unwrap();

    assert_eq!(result.open_dir, open_dir);
    assert_eq!(
        fs::read_to_string(result.open_dir.join("docs").join("readme.md")).unwrap(),
        "hello\n"
    );
    assert_eq!(
        status,
        Status {
            branch: "main".to_string(),
            state: "open-clean".to_string(),
            base: commit_id.clone(),
            open_fires: 0,
        }
    );
    assert_eq!(branches[0].state, "open-clean");
}

#[test]
fn clone_branch_creates_closed_branch_and_rejects_burning_source() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_file_commit(&objects, "docs/readme.md", "hello\n");
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": commit_id,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature/session".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let cloned = load_branch_record(&repo_root, "feature/session").unwrap();
    assert_eq!(
        required_string(&cloned, &["head"]).unwrap(),
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap()
    );
    assert_eq!(cloned.get("state").and_then(Value::as_str), Some("closed"));

    let mut burning = cloned;
    burning["name"] = Value::String("burning".to_string());
    burning["state"] = Value::String("open-burning".to_string());
    save_branch_record(&repo_root, &burning).unwrap();
    let error = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "burning".to_string(),
            new_branch: "copy".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("cannot clone from branch 'burning' while it is open-burning"));
}

#[test]
fn show_and_diff_local_branches() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "src/session.py", "def ttl():\n    return 30\n");
    let feature_commit =
        write_file_commit(&objects, "src/session.py", "def ttl():\n    return 15\n");
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": main_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let show = show_commitish(Some(&repo_root), "feature-session").unwrap();
    assert!(show.contains("Object: feature-session@CF-COMMIT-"));
    assert!(show.contains("Message: test commit"));
    assert!(show.contains("Files: 1"));
    assert!(show.contains("Certificate: consistent"));

    let show_data = show_commitish_data(Some(&repo_root), "feature-session").unwrap();
    assert_eq!(show_data["type"], "codefire_show_result");
    assert_eq!(show_data["show"]["target"], "feature-session");
    assert_eq!(show_data["show"]["message"], "test commit");
    assert_eq!(show_data["show"]["files"], 1);
    assert_eq!(show_data["show"]["atoms"], 0);
    assert_eq!(show_data["show"]["certificate"], "consistent");
    assert_eq!(show_data["show"]["signature"]["present"], false);
    assert_eq!(show_data["show"]["signature"]["summary"], "(none)");
    let show_envelope = command_result_envelope(
        "show",
        true,
        0,
        Some(&repo_root),
        show_data,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(show_envelope["schema"], "codefire.command_result.v1");
    assert_eq!(show_envelope["command"], "show");
    assert_eq!(show_envelope["data"]["show"]["target"], "feature-session");

    let diff = diff_commitish_with_options(
        Some(&repo_root),
        "main",
        "feature-session",
        &DiffOptions::default(),
    )
    .unwrap();
    assert!(diff.contains("--- main@CF-COMMIT-"));
    assert!(diff.contains("+++ feature-session@CF-COMMIT-"));
    assert!(diff.contains("-    return 30"));
    assert!(diff.contains("+    return 15"));

    let json_options = DiffOptions {
        json_output: true,
        ..DiffOptions::default()
    };
    let diff_json =
        diff_commitish_with_options(Some(&repo_root), "main", "feature-session", &json_options)
            .unwrap();
    let diff_envelope = diff_result_envelope(Some(&repo_root), &diff_json).unwrap();
    assert_eq!(diff_envelope["schema"], "codefire.command_result.v1");
    assert_eq!(diff_envelope["command"], "diff");
    assert_eq!(diff_envelope["ok"], true);
    assert_eq!(diff_envelope["data"]["type"], "codefire_diff");
    assert_eq!(diff_envelope["data"]["options"]["impact"], true);
    assert_eq!(
        diff_envelope["data"]["options"]["requested"]["impact"],
        false
    );
    assert_eq!(diff_envelope["data"]["included_sections"]["impact"], true);
    assert!(diff_envelope["data"]["left"]["label"]
        .as_str()
        .unwrap()
        .starts_with("main@CF-COMMIT-"));

    let missing =
        diff_commitish_with_options(Some(&repo_root), "main", "DOES-NOT-EXIST", &json_options)
            .unwrap_err();
    let missing_envelope = cli_error_envelope("diff", &missing);
    assert_eq!(missing_envelope["schema"], "codefire.command_result.v1");
    assert_eq!(missing_envelope["command"], "diff");
    assert_eq!(missing_envelope["ok"], false);
    assert_eq!(missing_envelope["diagnostics"][0]["kind"], "command_error");
    assert_eq!(missing_envelope["next_actions"][0]["kind"], "list_branches");
}

#[test]
fn diff_manifest_matches_golden_fixture() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 30\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-base",
        )],
        vec![],
    );
    let feature_commit = write_file_commit_with_atoms(
        &objects,
        &[
            (
                "docs/spec/session.md",
                "## REQ-session: Requirement\nTTL 15\n",
            ),
            ("src/app.py", "print('hi')\n"),
        ],
        &[
            (
                "REQ-session",
                "requirement",
                "docs/spec/session.md",
                "sha256:req-feature",
            ),
            ("CODE-app", "code", "src/app.py", "sha256:code-app"),
        ],
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let diff = diff_commitish_with_options(
        Some(&repo_root),
        "main",
        "feature-session",
        &DiffOptions::default(),
    )
    .unwrap();
    let normalized = normalize_diff_labels(&diff, "main", "feature-session");

    assert_eq!(
        normalized.trim_end(),
        include_str!("../tests/fixtures/diff_manifest.golden").trim_end()
    );
}

#[test]
fn cli_error_next_actions_are_contextual() {
    let unsupported = cli_error_envelope(
        "migrate",
        &CliError::Usage("unsupported migrate option: --metrics".to_string()),
    );
    assert_eq!(unsupported["next_actions"][0]["kind"], "show_help");
    assert_eq!(
        unsupported["next_actions"][0]["command"],
        "codefire migrate --help"
    );
    assert_eq!(
        unsupported["next_actions"][0]["target"]["command"],
        "migrate"
    );

    let invalid = cli_error_envelope(
        "status",
        &CliError::InvalidRepository("missing .codefire".to_string()),
    );
    let kinds = invalid["next_actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|action| action["kind"].as_str())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"doctor"));
    assert!(kinds.contains(&"migrate_check"));
}

fn assert_common_next_action_schema(action: &Value) {
    for field in ["kind", "id", "command", "reason", "description"] {
        assert!(
            action.get(field).and_then(Value::as_str).is_some(),
            "missing string next_action field {field}: {action}"
        );
    }
    assert!(action.get("target").is_some(), "missing target: {action}");
    assert!(action.get("context").is_some(), "missing context: {action}");
}

#[test]
fn codefire_text_diff_matches_git_payload_lines() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let git_base = temp.path().join("git-base");
    let git_feature = temp.path().join("git-feature");
    fs::create_dir_all(git_base.join("docs/spec")).unwrap();
    fs::create_dir_all(git_feature.join("docs/spec")).unwrap();
    fs::write(
        git_base.join("docs/spec/session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        git_feature.join("docs/spec/session.md"),
        "## REQ-session: Requirement\nTTL 15\n",
    )
    .unwrap();

    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 30\n",
    );
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 15\n",
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let codefire_diff = diff_commitish_with_options(
        Some(&repo_root),
        "main",
        "feature-session",
        &DiffOptions::default(),
    )
    .unwrap();
    let git_output = Command::new("git")
        .args([
            "diff",
            "--no-index",
            "--no-color",
            "--",
            git_base
                .join("docs/spec/session.md")
                .to_str()
                .expect("utf-8 path"),
            git_feature
                .join("docs/spec/session.md")
                .to_str()
                .expect("utf-8 path"),
        ])
        .output()
        .unwrap();
    assert!(!git_output.status.success());
    let git_diff = String::from_utf8(git_output.stdout).unwrap();

    assert_eq!(
        diff_payload_lines(&codefire_diff),
        diff_payload_lines(&git_diff)
    );
}

#[test]
fn parse_diff_args_accepts_algorithm_forms() {
    let show = parse_show_args(&["main".to_string(), "--json".to_string()]).unwrap();
    assert_eq!(show.target, "main");
    assert!(show.json_output);
    assert!(parse_show_args(&["main".to_string(), "--metrics".to_string()]).is_err());
    let branch_show = parse_branch_show_args(&[
        "--path=/tmp/open".to_string(),
        "--json".to_string(),
        "main".to_string(),
    ])
    .unwrap();
    assert_eq!(branch_show.path, PathBuf::from("/tmp/open"));
    assert_eq!(branch_show.branch.as_deref(), Some("main"));
    assert!(branch_show.json_output);
    assert!(parse_branch_show_args(&["--metrics".to_string()]).is_err());
    let local_list = parse_list_args(&["--json".to_string()]).unwrap();
    assert!(matches!(local_list.target, ListTarget::Local(_)));
    assert!(local_list.json_output);
    let remote_list =
        parse_list_args(&["cf:///tmp/server/org/app".to_string(), "--json".to_string()]).unwrap();
    assert_eq!(
        remote_list.target,
        ListTarget::Remote("cf:///tmp/server/org/app".to_string())
    );
    assert!(remote_list.json_output);
    let request_list = parse_remote_project_args(
        &["cf:///tmp/server/org/app".to_string(), "--json".to_string()],
        "request-list",
    )
    .unwrap();
    assert_eq!(request_list.project_url, "cf:///tmp/server/org/app");
    assert!(request_list.json_output);

    let args = vec![
        "--algorithm".to_string(),
        "patience".to_string(),
        "--context".to_string(),
        "2".to_string(),
        "--rename-detection".to_string(),
        "--atoms".to_string(),
        "--trace".to_string(),
        "--impact".to_string(),
        "--json".to_string(),
        "main".to_string(),
        "feature".to_string(),
    ];
    let parsed = parse_diff_args(&args).unwrap();
    assert_eq!(parsed.left, "main");
    assert_eq!(parsed.right, "feature");
    assert_eq!(parsed.diff.algorithm, DiffAlgorithm::Patience);
    assert_eq!(parsed.diff.context_lines, 2);
    assert!(parsed.diff.rename_detection);
    assert!(parsed.diff.atom_diff);
    assert!(parsed.diff.trace_diff);
    assert!(parsed.diff.impact_diff);
    assert!(parsed.diff.json_output);

    let args = vec![
        "main".to_string(),
        "feature".to_string(),
        "--algorithm=histogram".to_string(),
        "--context=1".to_string(),
    ];
    let parsed = parse_diff_args(&args).unwrap();
    assert_eq!(parsed.diff.algorithm, DiffAlgorithm::Histogram);
    assert_eq!(parsed.diff.context_lines, 1);
    assert!(!parsed.diff.rename_detection);
    assert!(!parsed.diff.atom_diff);
    assert!(!parsed.diff.trace_diff);
    assert!(!parsed.diff.impact_diff);
    assert!(!parsed.diff.json_output);

    let error = parse_diff_args(&[
        "--algorithm".to_string(),
        "minimal".to_string(),
        "main".to_string(),
        "feature".to_string(),
    ])
    .unwrap_err()
    .to_string();
    assert!(error.contains("--algorithm must be one of"));
}

#[test]
fn parse_state_diagnostic_json_args() {
    let status = parse_path_json_args(
        &[
            "--path".to_string(),
            "/tmp/example".to_string(),
            "--json".to_string(),
            "--metrics".to_string(),
        ],
        "status",
    )
    .unwrap();
    assert_eq!(status.path, PathBuf::from("/tmp/example"));
    assert!(status.json_output);
    assert!(status.metrics);

    let verify = parse_verify_args(&[
        "--details".to_string(),
        "--blocking-only".to_string(),
        "--path=/tmp/example".to_string(),
        "--json".to_string(),
        "--metrics".to_string(),
    ])
    .unwrap();
    assert_eq!(verify.path, PathBuf::from("/tmp/example"));
    assert!(verify.details);
    assert!(verify.blocking_only);
    assert_eq!(verify.diagnostic_filter(), "blocking_only");
    assert!(verify.json_output);
    assert!(verify.metrics);
}

#[test]
fn parse_health_recovery_commands_accept_common_path_option() {
    let storage = parse_storage_report_args(&[
        "--path=/tmp/example".to_string(),
        "--json".to_string(),
        "--quick".to_string(),
    ])
    .unwrap();
    assert_eq!(storage.start, PathBuf::from("/tmp/example"));
    assert!(storage.json_output);
    assert!(storage.quick);

    let doctor = parse_doctor_args(&[
        "--path".to_string(),
        "/tmp/example".to_string(),
        "--json".to_string(),
        "--quick".to_string(),
    ])
    .unwrap();
    assert_eq!(doctor.start, PathBuf::from("/tmp/example"));
    assert!(doctor.json_output);
    assert!(doctor.quick);

    let migrate = parse_migrate_args(&[
        "check".to_string(),
        "--path=/tmp/example".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    assert_eq!(migrate.start, PathBuf::from("/tmp/example"));
    assert!(migrate.json_output);

    let metrics_error = parse_migrate_args(&[
        "check".to_string(),
        "--path=/tmp/example".to_string(),
        "--json".to_string(),
        "--metrics".to_string(),
    ])
    .unwrap_err();
    let envelope = cli_error_envelope("migrate", &metrics_error);
    assert_eq!(envelope["schema"], "codefire.command_result.v1");
    assert_eq!(envelope["command"], "migrate");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["diagnostics"][0]["kind"], "command_error");
}

#[test]
fn cli_error_exit_codes_follow_stable_taxonomy() {
    assert_eq!(
        CliError::Usage("bad args".to_string()).exit_code(),
        ExitCode::InvalidUsageOrConfig.code()
    );
    assert_eq!(
        CliError::Usage("HTTP remote error: rejected".to_string()).exit_code(),
        ExitCode::RemoteRejected.code()
    );
    assert_eq!(
        CliError::LockContention("CodeFire repository is locked".to_string()).exit_code(),
        ExitCode::LockContention.code()
    );
    assert_eq!(
        CliError::IdempotencyConflict("conflict".to_string()).exit_code(),
        ExitCode::IdempotencyConflict.code()
    );
    assert_eq!(
        CliError::InvalidRepository("missing branch head".to_string()).exit_code(),
        ExitCode::RepositoryCorruption.code()
    );
    assert_eq!(
        CliError::Store(codefire_store::StoreError::ObjectNotFound(
            "CF-BLOB-missing".to_string()
        ))
        .exit_code(),
        ExitCode::ObjectReferenceInvalid.code()
    );
    assert_eq!(
        CliError::Store(codefire_store::StoreError::InvalidSealedCommit(
            "missing root".to_string()
        ))
        .exit_code(),
        ExitCode::SealedCommitInvalid.code()
    );
    assert_eq!(
        CliError::VerificationFailed(ExitCode::MissingRequiredLinks).exit_code(),
        ExitCode::MissingRequiredLinks.code()
    );
}

#[test]
fn repo_lock_wait_timeout_returns_lock_contention_with_owner_metadata() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let lock_path = repo_root.join(".codefire").join("locks").join("repo.lock");
    fs::write(
        lock_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "pid": 4242,
            "created_at": "2026-06-04T00:00:00Z",
        }))
        .unwrap(),
    )
    .unwrap();

    let error = RepoLock::acquire_with_options(
        &repo_root,
        &LockOptions {
            wait: true,
            timeout_ms: Some(0),
        },
    )
    .unwrap_err();
    let message = error.to_string();
    assert_eq!(error.exit_code(), ExitCode::LockContention.code());
    assert!(message.contains("timed out after 0ms"));
    assert!(message.contains("owner pid 4242"));
    assert!(message.contains("locked at 2026-06-04T00:00:00Z"));
}

#[test]
fn remote_resource_lock_wait_timeout_returns_owner_metadata_and_json_diagnostics() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let main_url = format!("cf://{}/org/app/main", server.display());
    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;
    remote::ensure_remote_layout(&remote_project_root).unwrap();
    let lock_path = remote_dirs(&remote_project_root)
        .locks
        .join("branch-main.lock");
    fs::write(
        lock_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "pid": 7777,
            "created_at": "2026-06-05T00:00:00Z",
        }))
        .unwrap(),
    )
    .unwrap();

    let error = upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url,
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions {
                wait: true,
                timeout_ms: Some(0),
            },
        },
    )
    .unwrap_err();
    let message = error.to_string();
    assert_eq!(error.exit_code(), ExitCode::LockContention.code());
    assert!(message.contains("CodeFire resource is locked"));
    assert!(message.contains("timed out after 0ms"));
    assert!(message.contains("owner pid 7777"));
    assert!(message.contains("locked at 2026-06-05T00:00:00Z"));

    let envelope = lock_contention_envelope("upload", &error);
    assert_eq!(envelope["schema"], "codefire.command_result.v1");
    assert_eq!(envelope["command"], "upload");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["exit_code"], ExitCode::LockContention.code());
    assert_eq!(envelope["diagnostics"][0]["kind"], "lock_contention");
    assert_eq!(envelope["next_actions"][0]["kind"], "retry_with_wait_lock");
}

#[test]
fn legacy_plan_json_outputs_are_wrapped_in_command_result_envelope() {
    let plan = json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "repo_root": "/tmp/codefire-repo",
        "dry_run": true,
        "next_actions": [
            {"kind": "verify", "command": "codefire verify --json"}
        ],
    });

    let envelope = plan_result_envelope("commit", &plan);

    assert_eq!(envelope["schema"], "codefire.command_result.v1");
    assert_eq!(envelope["command"], "commit");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["exit_code"], 0);
    assert_eq!(envelope["repo"], "/tmp/codefire-repo");
    assert_eq!(envelope["next_actions"][0]["kind"], "verify");
    assert_eq!(envelope["data"]["type"], "codefire_plan_result");
    assert_eq!(envelope["data"]["plan"], plan);
}

#[test]
fn local_workflow_commands_are_registered_through_dispatch_table() {
    for command in ["scan", "verify", "fire", "extinguish", "commit"] {
        assert!(
            local_workflow_handler(command).is_some(),
            "{command} should be routed through the local workflow dispatch table"
        );
    }
    assert!(local_workflow_handler("clone").is_none());
}

#[test]
fn remote_generation_allocation_is_locked_across_threads() {
    let temp = tempdir().unwrap();
    let project_root = temp.path().join("remote").join("org").join("app");
    remote::ensure_remote_layout(&project_root).unwrap();
    let workers = 16;
    let barrier = Arc::new(Barrier::new(workers));
    let options = LockOptions {
        wait: true,
        timeout_ms: Some(5_000),
    };
    let handles = (0..workers)
        .map(|_| {
            let project_root = project_root.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let lock = RemoteGenerationLock::acquire(&project_root, &options).unwrap();
                next_remote_generation(&lock).unwrap()
            })
        })
        .collect::<Vec<_>>();
    let mut generations = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();

    generations.sort_unstable();
    assert_eq!(generations, (1..=workers as u64).collect::<Vec<_>>());
    let state = read_json(&project_root.join("gc_state.json")).unwrap();
    assert_eq!(state["current_generation"], workers);
}

#[test]
fn context_pack_returns_atom_changed_and_fire_views() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[
            (
                "docs/requirements/session.md",
                "## REQ-session: Requirement\nTTL 30\n",
            ),
            (
                "docs/design/session.md",
                "## DES-session: Design\nClock policy\n",
            ),
            (
                "codefire.links.yaml",
                "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
            ),
            (
                "codefire.policy.yaml",
                "commit_policy:\n  require_trace_completeness: false\n",
            ),
        ],
        &[
            (
                "REQ-session",
                "requirement",
                "docs/requirements/session.md",
                "sha256:req",
            ),
            (
                "DES-session",
                "design",
                "docs/design/session.md",
                "sha256:des",
            ),
        ],
        Vec::new(),
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-05T00:00:00Z"
        }),
    )
    .unwrap();
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let atom_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Atom("REQ-session".to_string()),
        depth: 1,
        limit: 100,
        json_output: true,
    })
    .unwrap();
    assert_eq!(atom_pack.data["type"], "codefire_context_pack");
    assert_eq!(atom_pack.data["selector"]["kind"], "atom");
    assert_eq!(atom_pack.data["limits"]["limit"], 100);
    assert_eq!(atom_pack.data["truncated"]["atoms"], false);
    assert_eq!(
        atom_pack.data["scan"]["snapshot_source"],
        "preview_recomputed"
    );
    assert_eq!(atom_pack.data["scan"]["preview_recomputed"], true);
    assert_eq!(atom_pack.data["scan"]["fire_source"], "preview");
    assert_eq!(atom_pack.data["atoms"].as_array().unwrap().len(), 2);
    assert_eq!(atom_pack.data["trace_links"].as_array().unwrap().len(), 1);

    let changed_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Changed,
        depth: 1,
        limit: 100,
        json_output: true,
    })
    .unwrap();
    assert_eq!(
        changed_pack.data["scan"]["changed_atoms"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(changed_pack.data["changed_count"], 2);
    assert_eq!(changed_pack.data["open_fire_count"], 2);
    assert_eq!(
        changed_pack.data["changed_atoms"],
        changed_pack.data["scan"]["changed_atoms"]
    );
    assert_eq!(
        changed_pack.data["open_fires"],
        changed_pack.data["scan"]["open_fires"]
    );
    let branch_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Branch,
        depth: 1,
        limit: 100,
        json_output: true,
    })
    .unwrap();
    assert_eq!(branch_pack.data["selector"]["kind"], "branch");
    assert!(branch_pack.data["atoms"].as_array().unwrap().len() >= 2);
    let status = read_status(&open_dir).unwrap();
    let status_json = status_data_json_with_prediction(&status, &open_dir);
    assert_eq!(status_json["scan_prediction"]["changed_count"], 2);
    assert_eq!(status_json["scan_prediction"]["open_fire_count"], 2);
    let bounded_changed_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Changed,
        depth: 1,
        limit: 1,
        json_output: true,
    })
    .unwrap();
    assert_eq!(bounded_changed_pack.data["summary"]["atoms"]["total"], 1);
    assert_eq!(bounded_changed_pack.data["summary"]["atoms"]["returned"], 1);
    assert_eq!(bounded_changed_pack.data["summary"]["atoms"]["omitted"], 0);
    assert_eq!(
        bounded_changed_pack.data["summary"]["trace_links"]["with_omitted_endpoints"],
        1
    );
    assert_eq!(bounded_changed_pack.next_actions.len(), 1);
    assert_eq!(
        bounded_changed_pack.next_actions[0]["kind"],
        "context_expand"
    );
    assert_eq!(
        bounded_changed_pack.next_actions[0]["target"]["omitted"]
            ["trace_links_with_omitted_endpoints"],
        1
    );
    assert!(bounded_changed_pack.data["trace_link_endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|endpoint| endpoint["from"]["presence"] == "omitted"
            || endpoint["to"]["presence"] == "omitted"));

    let persisted_scan = run_scan(&open_dir).unwrap();
    let fire_id = persisted_scan
        .open_fires
        .first()
        .unwrap()
        .display_id
        .clone();
    let fire_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Fire(fire_id.clone()),
        depth: 1,
        limit: 100,
        json_output: true,
    })
    .unwrap();
    assert_eq!(fire_pack.data["selector"]["kind"], "fire");
    assert_eq!(fire_pack.data["scan"]["snapshot_source"], "active_scan");
    assert_eq!(fire_pack.data["scan"]["preview_recomputed"], false);
    assert_eq!(fire_pack.data["scan"]["fire_source"], "active");
    assert_eq!(
        fire_pack.data["scan"]["open_fires"][0]["display_id"],
        fire_id
    );
    assert_eq!(fire_pack.data["fires"].as_array().unwrap().len(), 1);

    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 45\n",
    )
    .unwrap();
    let stale_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir,
        selector: context::ContextSelector::Changed,
        depth: 1,
        limit: 100,
        json_output: true,
    })
    .unwrap();
    assert_eq!(
        stale_pack.data["scan"]["snapshot_source"],
        "stale_active_scan_preview"
    );
    assert_eq!(stale_pack.data["scan"]["active_scan_fresh"], false);
    assert_eq!(stale_pack.data["scan"]["preview_recomputed"], true);
}

#[test]
fn verify_diagnostics_filter_nonblocking_missing_required_links() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[
            (
                "docs/requirements/session.md",
                "## REQ-session: Requirement\nTTL 30\n",
            ),
            (
                "docs/design/session.md",
                "## DES-session: Design\nClock policy\n",
            ),
            (
                "codefire.policy.yaml",
                "commit_policy:\n  require_trace_completeness: false\n",
            ),
        ],
        &[
            (
                "REQ-session",
                "requirement",
                "docs/requirements/session.md",
                "sha256:req",
            ),
            (
                "DES-session",
                "design",
                "docs/design/session.md",
                "sha256:des",
            ),
        ],
        Vec::new(),
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-05T00:00:00Z"
        }),
    )
    .unwrap();
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

    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "passed");
    assert!(!verification.trace_completeness_required);
    assert!(!verification.missing_required_links.is_empty());
    assert_eq!(verification_exit_code(&verification), ExitCode::Success);

    let all = crate::automation::verification_diagnostics_json(&verification);
    assert!(!all.is_empty());
    assert!(all.iter().all(|diagnostic| {
        diagnostic["kind"] == "missing_required_link"
            && diagnostic["severity"] == "warning"
            && diagnostic["blocking"] == false
    }));
    assert!(
        crate::automation::verification_diagnostics_json_with_filter(&verification, true)
            .is_empty()
    );
    let filtered_data =
        crate::automation::verification_data_json_with_filter(&verification, "blocking_only");
    assert!(filtered_data["missing_required_links"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!filtered_data["all_missing_required_links"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(filtered_data["blocking_missing_required_links"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        filtered_data["diagnostic_summary"]["warning_count"],
        all.len()
    );
    assert!(verification::render_verification(&verification, true, true)
        .contains("Blocking checks: none"));

    let explain = run_explain(
        &parse_explain_args(&[
            "verify-failure".to_string(),
            "--path".to_string(),
            open_dir.to_string_lossy().into_owned(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(explain.data["diagnostic_summary"]["blocking_count"], 0);
    assert_eq!(
        explain.data["diagnostic_summary"]["warning_count"],
        all.len()
    );
    assert!(explain.data["summary"]
        .as_str()
        .unwrap()
        .contains("verification passed"));
    assert!(!explain.data["summary"]
        .as_str()
        .unwrap()
        .contains("8 blocker"));
}

#[test]
fn link_batch_validates_all_items_before_writing_links() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();

    let batch_path = temp.path().join("links.json");
    fs::write(
        &batch_path,
        serde_json::to_string(&json!({
            "version": 1,
            "defaults": {"type": "refined_by"},
            "links": [{"from": "REQ-session", "to": "DES-session"}]
        }))
        .unwrap(),
    )
    .unwrap();

    let dry_run = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(dry_run.item_count, 1);
    assert!(dry_run.dry_run);
    assert!(dry_run.valid);
    assert!(dry_run.diagnostics.is_empty());
    assert_eq!(dry_run.plan["type"], "codefire_operation_plan");
    assert!(!open_dir.join("codefire.links.yaml").exists());

    let applied = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.item_count, 1);
    assert_eq!(applied.added_links.len(), 1);
    let graph = codefire_core::current_trace_graph(&open_dir).unwrap();
    assert_eq!(graph.links.len(), 1);
    assert_eq!(graph.links[0].from, "REQ-session");
    assert_eq!(graph.links[0].to, "DES-session");
    assert_eq!(graph.links[0].link_type, "refined_by");
    assert!(fs::read_to_string(open_dir.join("codefire.links.yaml"))
        .unwrap()
        .contains("links:\n  - from: \"REQ-session\""));

    let invalid_path = temp.path().join("invalid-links.json");
    fs::write(
        &invalid_path,
        serde_json::to_string(&json!({
            "version": 1,
            "defaults": {"type": "verified_by"},
            "links": [
                {"from": "REQ-session", "to": "DES-session"},
                {"from": "REQ-session", "to": "REQ-session"},
                {"from": "REQ-session", "to": "TEST-missing"},
                {"from": "REQ-session", "to": "TEST-missing"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let invalid_dry_run = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir.clone(),
        batch_path: invalid_path.clone(),
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(!invalid_dry_run.valid);
    assert!(invalid_dry_run.diagnostics.len() >= 4);
    assert!(invalid_dry_run
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "self_link" && diagnostic["item_index"] == 1));
    assert!(
        invalid_dry_run
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "unknown_to_atom"
                && diagnostic["item_index"] == 2)
    );
    assert!(invalid_dry_run
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "duplicate_batch_link"
            && diagnostic["item_index"] == 3));
    assert_eq!(invalid_dry_run.plan["valid"], false);
    assert_eq!(
        invalid_dry_run.plan["diagnostics"]
            .as_array()
            .unwrap()
            .len(),
        invalid_dry_run.diagnostics.len()
    );
    let invalid_envelope = command_result_envelope(
        "link-batch",
        invalid_dry_run.valid,
        ExitCode::InvalidUsageOrConfig.code(),
        Some(&invalid_dry_run.repo_root),
        link_batch::link_batch_data_json(&invalid_dry_run),
        invalid_dry_run.diagnostics.clone(),
        Vec::new(),
    );
    assert_eq!(invalid_envelope["ok"], false);
    assert_eq!(
        invalid_envelope["exit_code"],
        ExitCode::InvalidUsageOrConfig.code()
    );
    assert_eq!(invalid_envelope["data"]["valid"], false);
    assert!(invalid_envelope["diagnostics"].as_array().unwrap().len() >= 4);
    let error = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir.clone(),
        batch_path: invalid_path,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert!(error.to_string().contains("TEST-missing"));
    assert!(error.to_string().contains("issue(s)"));
    let graph_after_error = codefire_core::current_trace_graph(&open_dir).unwrap();
    assert_eq!(graph_after_error.links.len(), 1);
}

#[test]
fn link_batch_appends_inside_existing_links_section_and_rejects_unsupported_yaml() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::create_dir_all(open_dir.join("tests")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("tests").join("test_session.py"),
        "# cf-atom: TEST-session\nassert True\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "version: 1\nlinks:\n  - from: \"REQ-session\"\n    to: \"DES-session\"\n    type: \"refined_by\"\nmetadata:\n  owner: qa\n",
    )
    .unwrap();
    let batch_path = temp.path().join("links.json");
    fs::write(
        &batch_path,
        serde_json::to_string(&json!({
            "version": 1,
            "links": [{"from": "DES-session", "to": "TEST-session", "type": "verified_by"}]
        }))
        .unwrap(),
    )
    .unwrap();

    let applied = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.item_count, 1);
    let updated = fs::read_to_string(open_dir.join("codefire.links.yaml")).unwrap();
    let inserted = updated
        .find("DES-session\"\n    to: \"TEST-session")
        .unwrap();
    let metadata = updated.find("metadata:").unwrap();
    assert!(inserted < metadata);

    fs::write(
        open_dir.join("codefire.links.yaml"),
        "version: 1\nmetadata:\n  owner: qa\n",
    )
    .unwrap();
    let error = link_batch::run_link_batch(&link_batch::LinkBatchOptions {
        path: open_dir,
        batch_path,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported codefire.links.yaml"));
}

#[test]
fn manual_fire_and_batch_validate_before_writing_fires() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::create_dir_all(open_dir.join("tests")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("tests").join("test_session.py"),
        "# cf-atom: TEST-session\nassert True\n",
    )
    .unwrap();

    let dry_run = fire::run_fire(&fire::FireOptions {
        path: open_dir.clone(),
        source_atom: "REQ-session".to_string(),
        target_atom: "DES-session".to_string(),
        reason: "manual concern".to_string(),
        severity: "required".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(dry_run.item_count, 1);
    assert!(dry_run.dry_run);
    assert_eq!(dry_run.plan["type"], "codefire_operation_plan");
    assert_eq!(dry_run.plan["item_count"], 1);
    assert_eq!(dry_run.plan["items_omitted"], 0);
    assert_eq!(dry_run.plan["truncated"], false);
    assert!(dry_run.plan["next_actions"][0]["command"]
        .as_str()
        .unwrap()
        .contains("--path"));
    let active_path = active_state_path(&open_dir);
    assert!(!active_path.join("fires.json").exists());

    let applied = fire::run_fire(&fire::FireOptions {
        path: open_dir.clone(),
        source_atom: "REQ-session".to_string(),
        target_atom: "DES-session".to_string(),
        reason: "manual concern".to_string(),
        severity: "required".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.fires.len(), 1);
    assert_eq!(applied.fires[0].created_by, "manual");
    assert!(applied.fires[0].display_id.starts_with("FIRE-"));
    assert_eq!(applied.fires[0].display_id.len(), "FIRE-".len() + 12);
    assert_ne!(applied.fires[0].display_id, "FIRE-001");
    assert!(applied.fires[0].fire_uid.starts_with("fire_sha256_"));
    assert_eq!(applied.fires[0].fire_uid.len(), "fire_sha256_".len() + 32);
    assert_eq!(read_status(&open_dir).unwrap().state, "open-burning");
    let fires_after_single = read_active_fires(&open_dir);
    assert_eq!(fires_after_single.len(), 1);

    let batch_path = temp.path().join("fires.json");
    fs::write(
        &batch_path,
        serde_json::to_string(&json!({
            "version": 1,
            "defaults": {"reason": "manual batch", "severity": "required"},
            "fires": [
                {"from": "REQ-session", "to": "TEST-session"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let batch_dry_run = fire::run_fire_batch(&fire::FireBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(batch_dry_run.item_count, 1);
    assert_eq!(read_active_fires(&open_dir).len(), 1);

    let batch_applied = fire::run_fire_batch(&fire::FireBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(batch_applied.fires.len(), 1);
    assert_eq!(read_active_fires(&open_dir).len(), 2);

    let invalid_path = temp.path().join("invalid-fires.json");
    fs::write(
        &invalid_path,
        serde_json::to_string(&json!({
            "version": 1,
            "defaults": {"reason": "invalid batch"},
            "fires": [
                {"from": "DES-session", "to": "REQ-session"},
                {"from": "REQ-session", "to": "TEST-missing"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let error = fire::run_fire_batch(&fire::FireBatchOptions {
        path: open_dir.clone(),
        batch_path: invalid_path,
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert!(error.to_string().contains("TEST-missing"));
    assert_eq!(read_active_fires(&open_dir).len(), 2);
}

#[test]
fn manual_fire_cleanup_without_source_changes_returns_open_clean() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir.join("docs").join("requirements").join("clean.md"),
        "## REQ-clean-manual: Requirement\nManual cleanup fixture\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("clean.md"),
        "## DES-clean-manual: Design\nManual cleanup fixture\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Seed manual fire atoms".to_string(),
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    assert_eq!(read_status(&open_dir).unwrap().state, "open-clean");

    let opened = fire::run_fire(&fire::FireOptions {
        path: open_dir.clone(),
        source_atom: "REQ-clean-manual".to_string(),
        target_atom: "DES-clean-manual".to_string(),
        reason: "manual cleanup probe".to_string(),
        severity: "required".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(read_status(&open_dir).unwrap().state, "open-burning");

    run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: opened.fires[0].display_id.clone(),
        resolution: "obsolete".to_string(),
        rationale: "probe cleanup".to_string(),
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
    let status = read_status(&open_dir).unwrap();
    assert_eq!(status.open_fires, 0);
    assert_eq!(status.state, "open-clean");
}

#[test]
fn fire_batch_dry_run_plan_is_bounded_by_default() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let batch_path = temp.path().join("manual-fires.json");
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir.join("docs").join("requirements").join("base.md"),
        "## REQ-manual-batch: Requirement\nManual batch fixture\n",
    )
    .unwrap();
    let mut fires = Vec::new();
    for index in 0..25 {
        fs::write(
            open_dir
                .join("docs")
                .join("design")
                .join(format!("manual-{index}.md")),
            format!("## DES-manual-batch-{index}: Design\nManual batch fixture {index}\n"),
        )
        .unwrap();
        fires.push(json!({
            "from": "REQ-manual-batch",
            "to": format!("DES-manual-batch-{index}"),
            "reason": "manual batch dry-run bounded",
        }));
    }
    fs::write(
        &batch_path,
        serde_json::to_string(&json!({"version": 1, "fires": fires})).unwrap(),
    )
    .unwrap();

    let bounded = fire::run_fire_batch(&fire::FireBatchOptions {
        path: open_dir.clone(),
        batch_path: batch_path.clone(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(bounded.item_count, 25);
    assert_eq!(bounded.plan["item_count"], 25);
    assert_eq!(bounded.plan["items"].as_array().unwrap().len(), 20);
    assert_eq!(bounded.plan["sample_items"].as_array().unwrap().len(), 20);
    assert_eq!(bounded.plan["items_omitted"], 5);
    assert_eq!(bounded.plan["truncated"], true);

    let full = fire::run_fire_batch(&fire::FireBatchOptions {
        path: open_dir,
        batch_path,
        dry_run: true,
        full_output: true,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(full.plan["items"].as_array().unwrap().len(), 25);
    assert_eq!(full.plan["items_omitted"], 0);
    assert_eq!(full.plan["truncated"], false);
}

#[test]
fn parse_merge_args_accepts_dry_run_json() {
    let args = vec![
        "feature-session".to_string(),
        "--into".to_string(),
        "main".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--lock-timeout".to_string(),
        "500ms".to_string(),
        "--idempotency-key=merge-key-1".to_string(),
    ];
    let parsed = parse_merge_args(&args).unwrap();
    assert_eq!(parsed.source_branch, "feature-session");
    assert_eq!(parsed.target_branch, "main");
    assert!(parsed.dry_run);
    assert!(parsed.json_output);
    assert!(parsed.lock.wait);
    assert_eq!(parsed.lock.timeout_ms, Some(500));
    assert_eq!(parsed.idempotency_key.as_deref(), Some("merge-key-1"));
}

#[test]
fn parse_mutating_dry_run_json_args() {
    let commit = parse_commit_args(&[
        "--dry-run".to_string(),
        "--json".to_string(),
        "--path=/tmp/open".to_string(),
        "-m".to_string(),
        "Seal".to_string(),
        "--wait-lock".to_string(),
        "--lock-timeout=250ms".to_string(),
        "--idempotency-key".to_string(),
        "commit-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(commit.path, PathBuf::from("/tmp/open"));
    assert_eq!(commit.message, "Seal");
    assert!(commit.dry_run);
    assert!(commit.json_output);
    assert!(commit.lock.wait);
    assert_eq!(commit.lock.timeout_ms, Some(250));
    assert_eq!(commit.idempotency_key.as_deref(), Some("commit-key-1"));

    let extinguish = parse_extinguish_args(&[
        "FIRE-001".to_string(),
        "--path=/tmp/open".to_string(),
        "--resolution=addressed".to_string(),
        "--rationale=fixed".to_string(),
        "--evidence-ref".to_string(),
        "CF-EVIDENCE-test".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--lock-timeout".to_string(),
        "2".to_string(),
        "--idempotency-key=ext-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(extinguish.path, PathBuf::from("/tmp/open"));
    assert_eq!(extinguish.fire_id, "FIRE-001");
    assert!(extinguish.dry_run);
    assert!(extinguish.json_output);
    assert_eq!(extinguish.evidence_refs, vec!["CF-EVIDENCE-test"]);
    assert!(extinguish.lock.wait);
    assert_eq!(extinguish.lock.timeout_ms, Some(2000));
    assert_eq!(extinguish.idempotency_key.as_deref(), Some("ext-key-1"));

    let open = parse_open_args(&[
        "main".to_string(),
        "/tmp/open".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--wait-lock".to_string(),
        "--idempotency-key=open-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(open.branch, "main");
    assert_eq!(open.path, PathBuf::from("/tmp/open"));
    assert!(open.dry_run);
    assert!(open.json_output);
    assert!(open.lock.wait);
    assert_eq!(open.idempotency_key.as_deref(), Some("open-key-1"));

    let clone = parse_clone_args(&[
        "main".to_string(),
        "feature".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--lock-timeout".to_string(),
        "1s".to_string(),
        "--idempotency-key".to_string(),
        "clone-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(clone.source, "main");
    assert_eq!(clone.new_branch, "feature");
    assert!(clone.dry_run);
    assert!(clone.json_output);
    assert!(clone.lock.wait);
    assert_eq!(clone.lock.timeout_ms, Some(1000));
    assert_eq!(clone.idempotency_key.as_deref(), Some("clone-key-1"));

    let upload = parse_upload_args(&[
        "main".to_string(),
        "cf:///tmp/server/org/app/main".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--idempotency-key=upload-key-1".to_string(),
        "--lock-timeout=750ms".to_string(),
    ])
    .unwrap();
    assert_eq!(upload.branch, "main");
    assert!(upload.dry_run);
    assert!(upload.json_output);
    assert_eq!(upload.idempotency_key.as_deref(), Some("upload-key-1"));
    assert!(upload.lock.wait);
    assert_eq!(upload.lock.timeout_ms, Some(750));

    let request_merge = parse_request_merge_args(&[
        "cf:///tmp/server/org/app/feature".to_string(),
        "cf:///tmp/server/org/app/main".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--idempotency-key".to_string(),
        "request-merge-key-1".to_string(),
        "--lock-timeout".to_string(),
        "2s".to_string(),
    ])
    .unwrap();
    assert!(request_merge.dry_run);
    assert!(request_merge.json_output);
    assert_eq!(
        request_merge.idempotency_key.as_deref(),
        Some("request-merge-key-1")
    );
    assert!(request_merge.lock.wait);
    assert_eq!(request_merge.lock.timeout_ms, Some(2000));

    let request_review = parse_request_review_args(&[
        "cf:///tmp/server/org/app".to_string(),
        "MR-abc123".to_string(),
        "--reviewer".to_string(),
        "alice".to_string(),
        "--decision".to_string(),
        "approve".to_string(),
        "--comment".to_string(),
        "ready".to_string(),
        "--wait-lock".to_string(),
        "--idempotency-key=request-review-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(request_review.reviewer, "alice");
    assert_eq!(request_review.decision, "approve");
    assert_eq!(
        request_review.idempotency_key.as_deref(),
        Some("request-review-key-1")
    );
    assert!(request_review.lock.wait);

    let request_apply = parse_request_apply_args(&[
        "cf:///tmp/server/org/app".to_string(),
        "MR-abc123".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--idempotency-key=request-apply-key-1".to_string(),
        "--lock-timeout=1s".to_string(),
    ])
    .unwrap();
    assert!(request_apply.dry_run);
    assert!(request_apply.json_output);
    assert_eq!(
        request_apply.idempotency_key.as_deref(),
        Some("request-apply-key-1")
    );
    assert!(request_apply.lock.wait);
    assert_eq!(request_apply.lock.timeout_ms, Some(1000));
}

#[test]
fn parse_https_remote_urls_and_tls_serve_args() {
    let branch_url = http::parse_cf_http_url("cf+https://127.0.0.1:8443/org/app/main").unwrap();
    assert_eq!(branch_url.endpoint, "https://127.0.0.1:8443");
    assert_eq!(branch_url.endpoint_scheme(), "https");
    assert_eq!(branch_url.org, "org");
    assert_eq!(branch_url.app, "app");
    assert_eq!(branch_url.branch, "main");

    let project_url = http::parse_cf_http_project_url("cf+https://127.0.0.1:8443/org/app").unwrap();
    assert_eq!(project_url.endpoint, "https://127.0.0.1:8443");
    assert_eq!(project_url.endpoint_scheme(), "https");
    assert_eq!(project_url.org, "org");
    assert_eq!(project_url.app, "app");
    assert!(http::is_cf_http_url("cf+https://127.0.0.1:8443/org/app"));

    let serve = parse_serve_args(&[
        "/tmp/server".to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        "8443".to_string(),
        "--tls-cert".to_string(),
        "/tmp/server.crt".to_string(),
        "--tls-key".to_string(),
        "/tmp/server.key".to_string(),
        "--tls-client-ca".to_string(),
        "/tmp/client-ca.crt".to_string(),
    ])
    .unwrap();
    assert_eq!(serve.storage_root, PathBuf::from("/tmp/server"));
    assert_eq!(serve.host, "127.0.0.1");
    assert_eq!(serve.port, 8443);
    assert_eq!(serve.tls_cert, Some(PathBuf::from("/tmp/server.crt")));
    assert_eq!(serve.tls_key, Some(PathBuf::from("/tmp/server.key")));
    assert_eq!(
        serve.tls_client_ca,
        Some(PathBuf::from("/tmp/client-ca.crt"))
    );

    let missing_key = parse_serve_args(&[
        "/tmp/server".to_string(),
        "--tls-cert".to_string(),
        "/tmp/server.crt".to_string(),
    ])
    .unwrap_err();
    assert!(missing_key
        .to_string()
        .contains("--tls-cert and --tls-key must be provided together"));

    let client_ca_without_server_tls = parse_serve_args(&[
        "/tmp/server".to_string(),
        "--tls-client-ca".to_string(),
        "/tmp/client-ca.crt".to_string(),
    ])
    .unwrap_err();
    assert!(client_ca_without_server_tls
        .to_string()
        .contains("--tls-client-ca requires --tls-cert and --tls-key"));
}

#[test]
fn http_remote_urls_reject_extra_path_segments() {
    let branch_error =
        http::parse_cf_http_url("cf+https://127.0.0.1:8443/org/app/main/extra").unwrap_err();
    assert!(branch_error.to_string().contains("HTTP remote URL must be"));

    let project_error =
        http::parse_cf_http_project_url("cf+https://127.0.0.1:8443/org/app/extra").unwrap_err();
    assert!(project_error
        .to_string()
        .contains("HTTP remote project URL must be"));
}

#[test]
fn tls_insecure_env_requires_explicit_truthy_value() {
    assert!(!http_tls::insecure_env_value_enabled(None));
    assert!(!http_tls::insecure_env_value_enabled(Some(
        std::ffi::OsStr::new("")
    )));
    assert!(!http_tls::insecure_env_value_enabled(Some(
        std::ffi::OsStr::new("0")
    )));
    assert!(http_tls::insecure_env_value_enabled(Some(
        std::ffi::OsStr::new("1")
    )));
    assert!(http_tls::insecure_env_value_enabled(Some(
        std::ffi::OsStr::new("true")
    )));
}

#[test]
fn tls_private_key_errors_name_supported_formats() {
    let temp = tempdir().unwrap();
    let key_path = temp.path().join("unsupported.key");
    fs::write(&key_path, "not a supported PEM private key\n").unwrap();
    let error = http_tls::load_private_key(&key_path).unwrap_err();
    assert!(error.to_string().contains("PKCS#8, RSA, SEC1 EC"));
}

#[test]
fn remote_idempotency_key_ignores_empty_string() {
    assert_eq!(remote_idempotency_key(&json!({})), None);
    assert_eq!(
        remote_idempotency_key(&json!({"idempotency_key": ""})),
        None
    );
    assert_eq!(
        remote_idempotency_key(&json!({"idempotency_key": "remote-once"})).as_deref(),
        Some("remote-once")
    );
}

#[test]
fn json_atomic_temp_paths_are_unique_for_same_target() {
    let path = PathBuf::from("state.json");
    let first = unique_temp_path(&path);
    let second = unique_temp_path(&path);
    assert_ne!(first, second);
    assert!(first.to_string_lossy().contains(".tmp"));
    assert!(second.to_string_lossy().contains(".tmp"));
}

#[test]
fn scan_and_verify_text_outputs_include_empty_summaries() {
    let scan = codefire_core::ScanResult {
        type_tag: "scan".to_string(),
        version: 1,
        base_commit: "CF-COMMIT-base".to_string(),
        atom_index: codefire_core::AtomIndex {
            type_tag: "atom_index".to_string(),
            version: 1,
            hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
            atoms: Vec::new(),
            duplicate_atom_ids: Vec::new(),
        },
        trace_graph: codefire_core::TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: 1,
            links: Vec::new(),
        },
        changed_atoms: Vec::new(),
        non_atom_changed_files: Vec::new(),
        open_fires: Vec::new(),
        tool_migration: None,
    };
    let scan_text = render_scan(&scan);
    assert!(scan_text.contains("Changed atoms: none"));
    assert!(scan_text.contains("Open fires: none"));

    let verification = codefire_core::Verification {
        type_tag: "verification".to_string(),
        version: 1,
        result: "passed".to_string(),
        trace_completeness_required: true,
        open_required_fires: 0,
        failed_checks: Vec::new(),
        missing_required_links: Vec::new(),
        stale_resolutions: Vec::new(),
        missing_evidence_refs: Vec::new(),
        duplicate_atom_ids: Vec::new(),
        verified_at: "2026-06-06T00:00:00Z".to_string(),
    };
    let verify_text = verification::render_verification(&verification, false, false);
    assert!(verify_text.contains("Verification passed."));
    assert!(verify_text.contains("Blocking checks: none"));
    assert!(verify_text.contains("Open fires: 0"));
    assert!(verify_text.contains("Failed checks: 0"));
}

#[test]
fn non_atom_file_changes_are_reported_in_scan_context_and_commit() {
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

    fs::write(open_dir.join("README.md"), "plain project notes\n").unwrap();
    let scan = compute_scan(&open_dir, true).unwrap().scan;
    assert!(scan.changed_atoms.is_empty());
    assert_eq!(scan.non_atom_changed_files.len(), 1);
    assert_eq!(scan.non_atom_changed_files[0].path, "README.md");
    assert_eq!(scan.non_atom_changed_files[0].status, "added");

    let scan_text = render_scan(&scan);
    assert!(scan_text.contains("Changed atoms: none"));
    assert!(scan_text.contains("Non-atom changed files:\n  added README.md"));
    let scan_json = scan_data_json_with_full(&scan, false);
    assert_eq!(scan_json["branch_state"], "open-burning");
    assert_eq!(scan_json["changed_count"], 1);
    assert_eq!(scan_json["changed_atom_count"], 0);
    assert_eq!(scan_json["non_atom_changed_file_count"], 1);
    assert_eq!(scan_json["non_atom_changed_files"][0]["path"], "README.md");

    let context = build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Changed,
        depth: 1,
        limit: 10,
        json_output: true,
    })
    .unwrap();
    assert_eq!(context.data["changed_count"], 1);
    assert_eq!(context.data["changed_atom_count"], 0);
    assert_eq!(context.data["non_atom_changed_file_count"], 1);
    assert_eq!(
        context.data["non_atom_changed_files"][0]["path"],
        "README.md"
    );

    let verification = compute_verify(&open_dir, true).unwrap();
    assert_eq!(verification.verification.result, "passed");
    assert_eq!(read_status(&open_dir).unwrap().state, "open-consistent");

    let commit = run_commit(&CommitOptions {
        path: open_dir,
        message: "Seal non-atom docs".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let commit_data = commit_result_data_json(&commit);
    assert_eq!(commit_data["changed_atom_count"], 0);
    assert_eq!(commit_data["non_atom_changed_file_count"], 1);
    assert_eq!(
        commit_data["non_atom_changed_files"][0]["path"],
        "README.md"
    );
}

#[test]
fn open_and_clone_dry_run_return_plans_without_structural_changes() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let main = load_branch_record(&repo_root, "main").unwrap();
    let main_head = required_string(&main, &["head"]).unwrap();

    let open_result = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: true,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert_eq!(open_result.plan["type"], "codefire_operation_plan");
    assert_eq!(open_result.plan["command"], "open");
    assert_eq!(open_result.plan["dry_run"], true);
    assert_eq!(open_result.plan["base_commit"], main_head);
    assert!(!open_dir.exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap()["state"],
        "closed"
    );

    let clone_result = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature".to_string(),
            dry_run: true,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert_eq!(clone_result.plan["type"], "codefire_operation_plan");
    assert_eq!(clone_result.plan["command"], "clone");
    assert_eq!(clone_result.plan["dry_run"], true);
    assert_eq!(clone_result.plan["source"]["head"], main_head);
    assert!(!branch_record_path(&repo_root, "feature").exists());
}

#[test]
fn open_and_clone_idempotency_replay_same_payload_and_reject_conflicts() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let other_open_dir = temp.path().join("other-open");
    init_repo(&repo_root, false).unwrap();

    let first_open = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("open-main-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(first_open.branch, "main");
    assert!(open_dir.join(".codefire-open").exists());

    let replay_open = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("open-main-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(replay_open.open_dir, first_open.open_dir);
    assert_eq!(replay_open.plan["command"], "open");

    let open_conflict = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: other_open_dir,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("open-main-once".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(
        open_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );

    let first_clone = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("clone-feature-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(first_clone.plan["command"], "clone");
    assert!(branch_record_path(&repo_root, "feature").exists());

    let replay_clone = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("clone-feature-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(replay_clone.plan["command"], "clone");

    let clone_conflict = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "other-feature".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("clone-feature-once".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(
        clone_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );
}

#[test]
fn commit_dry_run_returns_plan_without_updating_branch() {
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
    let before = load_branch_record(&repo_root, "main").unwrap();
    let before_head = required_string(&before, &["head"]).unwrap();

    let result = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Dry run".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let after = load_branch_record(&repo_root, "main").unwrap();
    let after_head = required_string(&after, &["head"]).unwrap();

    assert_eq!(result.commit_id, "");
    assert_eq!(result.plan["type"], "codefire_operation_plan");
    assert_eq!(result.plan["command"], "commit");
    assert_eq!(result.plan["dry_run"], true);
    assert_eq!(before_head, after_head);
    assert!(!active_state_path(&open_dir)
        .join("verification.json")
        .exists());
}

#[test]
fn commit_dry_run_json_reports_blockers_and_bounds_changed_atoms() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    for index in 0..60 {
        fs::write(
            open_dir
                .join("docs")
                .join("requirements")
                .join(format!("bulk-{index}.md")),
            format!("## REQ-commit-bulk-{index}: Requirement\nBulk changed atom {index}\n"),
        )
        .unwrap();
    }
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();

    let bounded = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Bounded dry run".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let bounded_data = commit_result_data_json(&bounded);
    assert_eq!(bounded_data["type"], "codefire_commit_result");
    assert_eq!(bounded_data["dry_run"], true);
    assert_eq!(bounded_data["changed_atom_count"], 60);
    assert_eq!(bounded_data["changed_atoms"].as_array().unwrap().len(), 50);
    assert_eq!(bounded_data["changed_atoms_omitted"], 10);
    assert_eq!(bounded_data["message"], "Bounded dry run");
    assert_eq!(bounded_data["extinguished_fire_count"], 0);
    assert_eq!(bounded_data["active_resolution_count"], 0);
    assert_eq!(bounded_data["evidence_ref_count"], 0);
    assert_eq!(bounded.plan["changed_atoms_truncated"], true);

    let full = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Full dry run".to_string(),
        dry_run: true,
        full_output: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    assert_eq!(full.plan["changed_atoms"].as_array().unwrap().len(), 60);
    assert_eq!(full.plan["changed_atoms_omitted"], 0);

    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-commit-bulk-0\n    to: REQ-commit-bulk-1\n    type: refined_by\n",
    )
    .unwrap();
    let blocked = run_commit(&CommitOptions {
        path: open_dir,
        message: "Blocked dry run".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    assert!(blocked.blocked);
    assert_ne!(blocked.exit_code.code(), 0);
    let blocked_data = commit_result_data_json(&blocked);
    assert_eq!(blocked_data["blocked"], true);
    assert_eq!(blocked_data["verification"]["result"], "failed");
    assert_eq!(blocked.plan["next_actions"][0]["kind"], "verify");
}

#[test]
fn commit_idempotency_key_replays_same_payload_and_rejects_conflict() {
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

    let first = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Seal once".to_string(),
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
        signer: None,
        key_id: None,
    })
    .unwrap();
    let branch_after_first = load_branch_record(&repo_root, "main").unwrap();
    assert_eq!(
        required_string(&branch_after_first, &["head"]).unwrap(),
        first.commit_id
    );

    let replay = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Seal once".to_string(),
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
        signer: None,
        key_id: None,
    })
    .unwrap();
    assert_eq!(replay.commit_id, first.commit_id);
    assert_eq!(replay.branch, first.branch);
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        first.commit_id
    );

    let conflict = run_commit(&CommitOptions {
        path: open_dir,
        message: "Different payload".to_string(),
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
        signer: None,
        key_id: None,
    })
    .unwrap_err();
    assert_eq!(conflict.exit_code(), ExitCode::IdempotencyConflict.code());
}

#[test]
fn extinguish_dry_run_returns_plan_without_writing_ledgers() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let fire = compute_scan(&open_dir, false)
        .unwrap()
        .scan
        .open_fires
        .first()
        .unwrap()
        .clone();
    let result = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: fire.display_id.clone(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        refresh: false,
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        edit_rationale: false,
    })
    .unwrap();
    let active = active_state_path(&open_dir);

    assert_eq!(result.display_id, fire.display_id);
    assert_eq!(result.fire_uid, fire.fire_uid);
    assert_eq!(result.plan["type"], "codefire_operation_plan");
    assert_eq!(result.plan["command"], "extinguish");
    assert_eq!(result.plan["dry_run"], true);
    assert!(!active.join("fires.json").exists());
    assert!(!active.join("resolutions.json").exists());
    assert_eq!(read_status(&open_dir).unwrap().state, "open-clean");
}

#[test]
fn extinguish_idempotency_key_replays_same_payload_and_rejects_conflict() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let fire = compute_scan(&open_dir, false)
        .unwrap()
        .scan
        .open_fires
        .first()
        .unwrap()
        .clone();
    let first = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: fire.display_id.clone(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
        edit_rationale: false,
    })
    .unwrap();
    assert_eq!(first.display_id, fire.display_id);
    assert_eq!(first.fire_uid, fire.fire_uid);

    let replay = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: fire.display_id.clone(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
        edit_rationale: false,
    })
    .unwrap();
    assert_eq!(replay.display_id, first.display_id);
    assert_eq!(replay.fire_uid, first.fire_uid);
    assert_eq!(replay.plan["command"], "extinguish");

    let conflict = run_extinguish(&ExtinguishOptions {
        path: open_dir,
        fire_id: fire.display_id,
        resolution: "addressed".to_string(),
        rationale: "different rationale".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
        edit_rationale: false,
    })
    .unwrap_err();
    assert_eq!(conflict.exit_code(), ExitCode::IdempotencyConflict.code());
}

#[test]
fn extinguish_batch_validates_all_items_before_writing_ledgers() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let batch_path = temp.path().join("fires.yaml");
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();
    let scan = compute_scan(&open_dir, false).unwrap().scan;
    let first_fire = scan.open_fires.first().unwrap().display_id.clone();
    let second_fire = scan.open_fires.get(1).unwrap().display_id.clone();
    fs::write(
        &batch_path,
        format!(
            r#"version: 1
defaults:
  resolution: addressed
  evidence: "cargo test --workspace: passed"
fires:
  - id: {first_fire}
    rationale: "REQ to DES reviewed"
  - id: {second_fire}
    rationale: "DES to REQ reviewed"
"#
        ),
    )
    .unwrap();

    let parsed = parse_extinguish_batch_args(&[
        "--batch".to_string(),
        batch_path.display().to_string(),
        "--path".to_string(),
        open_dir.display().to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    assert_eq!(parsed.batch_path, batch_path);
    assert_eq!(parsed.path, open_dir);
    assert!(parsed.dry_run);
    assert!(!parsed.full_output);
    assert!(parsed.json_output);

    let dry_run = run_extinguish_batch(&parsed).unwrap();
    let active = active_state_path(&open_dir);
    assert_eq!(dry_run.item_count, 2);
    assert_eq!(dry_run.plan["type"], "codefire_operation_plan");
    assert_eq!(dry_run.plan["command"], "extinguish-batch");
    assert_eq!(dry_run.plan["dry_run"], true);
    assert_eq!(dry_run.applied_count, 0);
    assert_eq!(dry_run.remaining_open_fire_count, scan.open_fires.len());
    assert!(!active.join("fires.json").exists());
    assert!(!active.join("resolutions.json").exists());

    let applied = run_extinguish_batch(&batch::BatchExtinguishOptions {
        path: open_dir.clone(),
        batch_path: parsed.batch_path,
        dry_run: false,
        full_output: false,
        json_output: false,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.item_count, 2);
    assert_eq!(applied.applied_count, 2);
    assert_eq!(applied.fire_ids.len(), 2);
    assert_eq!(applied.resolution_uids.len(), 2);
    assert_eq!(applied.remaining_open_fire_count, scan.open_fires.len() - 2);
    let applied_data = batch_extinguish_data_json(&applied);
    assert_eq!(applied_data["type"], "codefire_extinguish_batch_result");
    assert_eq!(applied_data["applied_count"], 2);
    assert_eq!(applied_data["plan"]["items_omitted"], 0);
    let fires: Vec<codefire_core::Fire> =
        serde_json::from_value(read_json(&active.join("fires.json")).unwrap()).unwrap();
    let resolutions: Vec<codefire_core::Resolution> =
        serde_json::from_value(read_json(&active.join("resolutions.json")).unwrap()).unwrap();
    assert_eq!(
        fires
            .iter()
            .filter(|fire| fire.status == "extinguished")
            .count(),
        2
    );
    assert_eq!(resolutions.len(), 2);
}

#[test]
fn extinguish_batch_dry_run_json_is_bounded_by_default() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let batch_path = temp.path().join("large-fires.json");
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir.join("docs").join("requirements").join("base.md"),
        "## REQ-batch-base: Requirement\nTTL 30\n",
    )
    .unwrap();
    for index in 0..25 {
        fs::write(
            open_dir
                .join("docs")
                .join("design")
                .join(format!("session-{index}.md")),
            format!("## DES-session-{index}: Design\nClock policy {index}\n"),
        )
        .unwrap();
    }
    let mut fires = Vec::new();
    for index in 0..25 {
        let result = run_fire(&fire::FireOptions {
            path: open_dir.clone(),
            source_atom: "REQ-batch-base".to_string(),
            target_atom: format!("DES-session-{index}"),
            reason: "large batch dry-run fixture".to_string(),
            severity: "required".to_string(),
            dry_run: false,
            full_output: false,
            json_output: false,
            lock: LockOptions::default(),
        })
        .unwrap();
        fires.push(result.fires[0].display_id.clone());
    }
    let batch_fires = fires
        .iter()
        .map(|fire_id| {
            json!({
                "id": fire_id,
                "resolution": "addressed",
                "rationale": "large batch dry-run bounded",
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        &batch_path,
        serde_json::to_string_pretty(&json!({"version": 1, "fires": batch_fires})).unwrap(),
    )
    .unwrap();

    let result = run_extinguish_batch(&batch::BatchExtinguishOptions {
        path: open_dir,
        batch_path,
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    let data = batch_extinguish_data_json(&result);

    assert_eq!(result.item_count, 25);
    assert_eq!(result.plan["item_detail_limit"], 20);
    assert_eq!(result.plan["items_omitted"], 5);
    assert_eq!(result.plan["items"].as_array().unwrap().len(), 20);
    assert_eq!(data["fire_ids"].as_array().unwrap().len(), 20);
    assert_eq!(data["fire_ids_omitted"], 5);
}

#[test]
fn interactive_extinguish_plan_lists_open_fires_and_recent_evidence() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();
    let evidence_id = codefire_store::store_object(
        &repo_root.join(".codefire").join("objects"),
        "evidence",
        json!({
            "type": "evidence",
            "version": 1,
            "label": "pytest",
            "artifact_ref": null,
            "command": {
                "command": "pytest -q",
                "exit_code": 0,
                "success": true
            },
            "created_at": "2026-06-05T00:00:00Z"
        }),
    )
    .unwrap();

    let options = extinguish_ux::parse_interactive_extinguish_args(&[
        "--interactive".to_string(),
        "--path".to_string(),
        open_dir.display().to_string(),
        "--json".to_string(),
        "--evidence-limit".to_string(),
        "1".to_string(),
    ])
    .unwrap();
    let result = extinguish_ux::run_interactive_extinguish(&options).unwrap();

    assert_eq!(result.plan["type"], "codefire_extinguish_interactive_plan");
    assert_eq!(result.plan["command"], "extinguish-interactive");
    assert_eq!(result.plan["evidence_candidates"][0]["id"], evidence_id);
    assert!(result.plan["open_fires"].as_array().unwrap().len() >= 2);
    assert!(result.plan["draft_commands"][0]["command"]
        .as_str()
        .unwrap()
        .contains(&evidence_id));
}

#[test]
fn extinguish_batch_template_generates_batch_wrapper_from_open_fires() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let output = temp.path().join("fires-to-extinguish.json");
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let options = batch::parse_batch_template_args(&[
        "--batch-template".to_string(),
        "--path".to_string(),
        open_dir.display().to_string(),
        "--evidence".to_string(),
        "cargo test --workspace: passed".to_string(),
        "--output".to_string(),
        output.display().to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    let result = batch::run_batch_template(&options).unwrap();
    let template: Value = serde_json::from_str(&result.template).unwrap();

    assert_eq!(template["version"], 1);
    assert_eq!(
        template["defaults"]["evidence"],
        "cargo test --workspace: passed"
    );
    assert!(template["fires"].as_array().unwrap().len() >= 2);
    assert!(result.plan["next_actions"][0]["command"]
        .as_str()
        .unwrap()
        .contains("extinguish --batch"));
    assert!(output.exists());

    let scan = compute_scan(&open_dir, false).unwrap().scan;
    let actions = scan_next_actions(&scan, Some(&open_dir));
    assert!(actions
        .iter()
        .any(|action| action["kind"] == "extinguish_batch_template"));
}

#[test]
fn all_matching_extinguish_extinguishes_source_target_subset() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let options = extinguish_ux::parse_all_matching_extinguish_args(&[
        "--all-matching".to_string(),
        "REQ-session -> DES-session".to_string(),
        "--path".to_string(),
        open_dir.display().to_string(),
        "--rationale".to_string(),
        "source target link reviewed".to_string(),
    ])
    .unwrap();
    let result = extinguish_ux::run_all_matching_extinguish(&options).unwrap();

    assert_eq!(result.item_count, 1);
    assert_eq!(result.plan["command"], "extinguish-all-matching");
    let fires: Vec<codefire_core::Fire> = serde_json::from_value(
        read_json(&active_state_path(&open_dir).join("fires.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        fires
            .iter()
            .filter(|fire| fire.status == "extinguished")
            .count(),
        1
    );
    assert!(fires.iter().any(|fire| fire.status == "open"));
}

#[test]
fn all_matching_extinguish_args_accept_equals_form_and_editor_flag() {
    let parsed = extinguish_ux::parse_all_matching_extinguish_args(&[
        "--all-matching=REQ-session -> DES-session".to_string(),
        "--path=/tmp/open".to_string(),
        "--edit-rationale".to_string(),
        "--evidence".to_string(),
        "pytest passed".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .unwrap();

    assert_eq!(parsed.query, "REQ-session -> DES-session");
    assert_eq!(parsed.path, PathBuf::from("/tmp/open"));
    assert!(parsed.edit_rationale);
    assert!(parsed.dry_run);
    assert!(parsed.json_output);
}

#[cfg(unix)]
#[test]
fn edit_rationale_reads_configured_editor_output() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let editor = temp.path().join("editor.sh");
    fs::write(
        &editor,
        "#!/bin/sh\nprintf 'edited rationale from editor\\n' > \"$1\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&editor).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&editor, permissions).unwrap();

    let rationale =
        extinguish_ux::resolve_rationale_with_editor("initial rationale", &editor).unwrap();

    assert_eq!(rationale, "edited rationale from editor");
}

#[test]
fn storage_report_counts_objects_by_type_and_warns_large_objects() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let blob_id = codefire_store::store_object(
        &repo_root.join(".codefire").join("objects"),
        "blob",
        json!({"path": "large.bin", "content": "x".repeat(512)}),
    )
    .unwrap();
    let blob_path =
        codefire_store::object_record_path(&repo_root.join(".codefire").join("objects"), &blob_id)
            .unwrap();
    let mut blob_record = read_json(&blob_path).unwrap();
    blob_record["payload"]["type"] = Value::String("payload_blob".to_string());
    write_json_atomic(&blob_path, &blob_record).unwrap();

    let parsed = parse_storage_report_args(&[
        repo_root.to_string_lossy().into_owned(),
        "--json".to_string(),
        "--large-threshold=128".to_string(),
    ])
    .unwrap();
    let report = run_storage_report(&parsed).unwrap();
    let data = storage_report_data_json(&report);

    assert!(parsed.json_output);
    assert_eq!(parsed.large_threshold_bytes, 128);
    assert!(!parsed.quick);
    assert_eq!(data["mode"], "full");
    assert!(report.objects.files >= 10);
    assert!(report.active_state.files > 0);
    assert!(report
        .object_types
        .iter()
        .any(|stats| stats.type_tag == "blob" && stats.files == 1));
    assert!(report
        .largest_objects
        .iter()
        .any(|stats| stats.object_id == blob_id));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == "large_object"));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == "payload_type_mismatch"));
    assert_eq!(data["type"], "codefire_storage_report");
    assert!(data["objects"]["by_type"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stats| stats["type"] == "blob"));
    assert!(!storage_report_diagnostics_json(&report).is_empty());
    let storage_actions = storage_report_next_actions(&report);
    assert!(!storage_actions.is_empty());
    assert_common_next_action_schema(&storage_actions[0]);

    let quick = run_storage_report(
        &parse_storage_report_args(&[
            repo_root.to_string_lossy().into_owned(),
            "--quick".to_string(),
            "--json".to_string(),
            "--large-threshold=128".to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    let quick_data = storage_report_data_json(&quick);
    assert!(quick.quick);
    assert_eq!(quick_data["mode"], "quick");
    assert_eq!(quick_data["coverage"]["object_json_validation"], false);
    assert_eq!(
        quick_data["coverage"]["largest_object_type_source"],
        "id_prefix_or_path"
    );
    assert_eq!(quick_data["objects"]["largest_limit"], 10);
    assert!(quick.object_types.is_empty());
    assert_eq!(quick_data["skipped_checks"][0], "object_json_validation");
    assert!(quick
        .largest_objects
        .iter()
        .any(|object| object.type_tag == "blob" && object.type_confidence == "id_prefix"));
    assert!(quick_data["objects"]["largest"]
        .as_array()
        .unwrap()
        .iter()
        .any(|object| object["type"] == "blob" && object["type_confidence"] == "id_prefix"));
    assert!(quick
        .warnings
        .iter()
        .all(|warning| warning.kind != "payload_type_mismatch"));
}

#[test]
fn storage_report_counts_file_remote_project_storage() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let project_url = format!("cf://{}/org/app", server.display());
    let remote_project_root = crate::remote::parse_cf_project_url(&project_url)
        .unwrap()
        .project_root;
    let dirs = remote_dirs(&remote_project_root);
    fs::create_dir_all(&dirs.objects).unwrap();
    fs::create_dir_all(&dirs.branches).unwrap();
    fs::create_dir_all(&dirs.merge_requests).unwrap();
    fs::create_dir_all(&dirs.idempotency).unwrap();
    write_json_atomic(
        &remote_project_root.join("server_policy.json"),
        &json!({"gc": {
            "retention_seconds": 3600,
            "retention_generations": 2,
            "idempotency_retention_seconds": 1
        }}),
    )
    .unwrap();
    write_json_atomic(
        &remote_project_root.join("gc_state.json"),
        &json!({"current_generation": 7}),
    )
    .unwrap();
    fs::write(
        dirs.objects.join("object.json"),
        serde_json::to_string_pretty(&json!({
            "remote": {"last_seen_generation": 7}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(dirs.branches.join("main.json"), "{}\n").unwrap();
    fs::write(dirs.merge_requests.join("mr_1.json"), "{}\n").unwrap();
    write_json_atomic(
        &remote_project_root.join("request_nonce_cache.json"),
        &json!({
            "version": 1,
            "entries": {
                "nonce-a": {"seen_at": 1}
            },
            "entry_count": 1,
            "max_entries": 10000,
            "ttl_seconds": 300,
            "updated_at": "2026-06-07T00:00:00Z"
        }),
    )
    .unwrap();
    fs::create_dir_all(dirs.idempotency.join("remote_upload")).unwrap();
    fs::write(
        dirs.idempotency
            .join("remote_upload")
            .join("old-upload.json"),
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "command": "remote_upload",
            "key": "old-upload",
            "payload_hash": "sha256-old",
            "payload_mode": "hash_only",
            "result": {"head": "CF-COMMIT-old"},
            "created_at": "1970-01-01T00:00:00Z"
        }))
        .unwrap(),
    )
    .unwrap();

    let parsed = parse_storage_report_args(&[
        repo_root.to_string_lossy().into_owned(),
        "--remote".to_string(),
        project_url.clone(),
        "--remote=cf:///tmp/missing/org/app".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    let report = run_storage_report(&parsed).unwrap();
    let data = storage_report_data_json(&report);

    assert_eq!(parsed.remotes.len(), 2);
    assert_eq!(report.remotes.len(), 2);
    let missing = report
        .remotes
        .iter()
        .find(|remote| remote.url == "cf:///tmp/missing/org/app")
        .unwrap();
    assert_eq!(missing.status, "invalid_layout");
    assert!(!missing.missing_required_paths.is_empty());
    let remote = report
        .remotes
        .iter()
        .find(|remote| remote.url == project_url)
        .unwrap();
    assert_eq!(remote.objects.files, 1);
    assert_eq!(remote.branches.files, 1);
    assert_eq!(remote.merge_requests.files, 1);
    assert_eq!(remote.idempotency.files, 1);
    assert_eq!(remote.retention.retention_seconds, 3600);
    assert_eq!(remote.retention.retention_generations, 2);
    assert_eq!(remote.retention.idempotency_retention_seconds, 1);
    assert_eq!(remote.retention.current_generation, 7);
    assert!(remote.nonce_cache.present);
    assert_eq!(remote.nonce_cache.entry_count, 1);
    assert_eq!(remote.nonce_cache.max_entries, 10000);
    assert_eq!(remote.nonce_cache.ttl_seconds, 300);
    assert_eq!(remote.idempotency_retention.expired_files, 1);
    assert_eq!(
        remote.idempotency_retention.oldest_created_at.as_deref(),
        Some("1970-01-01T00:00:00Z")
    );
    assert!(remote
        .objects_by_generation
        .iter()
        .any(|stats| stats.generation == Some(7) && stats.files == 1));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == "remote_idempotency_retention"));
    assert_eq!(data["remotes"].as_array().unwrap().len(), 2);
    assert_eq!(data["remotes"][0]["retention"]["current_generation"], 7);
    assert_eq!(data["remotes"][0]["idempotency"]["expired_files"], 1);
    assert_eq!(data["remotes"][0]["nonce_cache"]["entry_count"], 1);
    assert_eq!(data["remotes"][0]["nonce_cache"]["max_entries"], 10000);
    assert!(data["remotes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|remote| remote["url"] == "cf:///tmp/missing/org/app"
            && remote["status"] == "invalid_layout"
            && !remote["missing_required_paths"]
                .as_array()
                .unwrap()
                .is_empty()));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == "remote_layout_invalid"));
    assert_eq!(
        data["remotes"][0]["objects_by_generation"][0]["generation"],
        7
    );
}

#[test]
fn evidence_add_records_artifact_ref_and_command_capture() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let artifact_path = temp.path().join("model.bin");
    init_repo(&repo_root, false).unwrap();
    fs::write(&artifact_path, b"artifact-payload").unwrap();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--artifact".to_string(),
        artifact_path.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf okay".to_string(),
        "--label=training-smoke".to_string(),
        "--max-output-bytes=2".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    let result = run_evidence_add(&parsed).unwrap();
    let data = evidence_add_data_json(&result);

    assert!(parsed.json_output);
    assert!(result.evidence_id.starts_with("CF-EVIDENCE-"));
    let artifact_ref_id = result.artifact_ref_id.as_ref().unwrap();
    assert!(artifact_ref_id.starts_with("CF-ARTIFACT-"));
    assert_eq!(result.command_exit_code, Some(0));
    assert!(!result.command_timed_out);
    assert_eq!(data["command_cwd_source"], "caller_cwd");
    assert_eq!(
        data["command_cwd"].as_str(),
        Some(std::env::current_dir().unwrap().to_string_lossy().as_ref())
    );
    assert_eq!(data["type"], "codefire_evidence_add_result");
    assert_eq!(data["dry_run"], false);
    assert_eq!(data["created"], true);
    assert_eq!(
        data["evidence_id"].as_str(),
        Some(result.evidence_id.as_str())
    );
    assert_eq!(data["command_timed_out"], false);
    assert_eq!(data["command_stdout_summary"], "ok");
    assert_eq!(data["command_stdout_truncated"], true);
    assert!(data["object_path"]
        .as_str()
        .unwrap()
        .contains("CF-EVIDENCE-"));

    let objects = repo_root.join(".codefire").join("objects");
    let artifact_ref = codefire_store::read_object(&objects, artifact_ref_id).unwrap();
    assert_eq!(artifact_ref["type"], "artifact_ref");
    assert_eq!(artifact_ref["label"], "training-smoke");
    assert_eq!(artifact_ref["size_bytes"], 16);
    assert_eq!(artifact_ref["path_kind"], "redacted");
    assert_eq!(artifact_ref["path"], "<redacted>");
    assert_eq!(artifact_ref["local_path_redacted"], true);
    assert!(artifact_ref["uri"]
        .as_str()
        .unwrap()
        .starts_with("artifact://redacted/"));
    assert_eq!(artifact_ref["hash_streaming"], true);
    assert_eq!(artifact_ref["hash_chunk_bytes"], 65536);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "artifact_path_redacted"));
    assert_eq!(
        artifact_ref["content_hash"],
        "sha256:5c6fd60a6ad0ce3fffdf2f2c61fbf1e9677f780c64a1ee33563bb2a40f29ef80"
    );
    let evidence = codefire_store::read_object(&objects, &result.evidence_id).unwrap();
    assert_eq!(evidence["type"], "evidence");
    assert_eq!(evidence["proof_status"], "command-passed");
    assert_eq!(
        evidence["artifact_ref"].as_str(),
        Some(artifact_ref_id.as_str())
    );
    assert_eq!(evidence["command"]["exit_code"], 0);
    assert_eq!(evidence["command"]["mode"], "shell");
    assert_eq!(evidence["command"]["shell"], true);
    assert_eq!(evidence["command"]["timed_out"], false);
    assert_eq!(evidence["command"]["timeout_ms"], 300000);
    assert_eq!(
        evidence["command"]["cwd"].as_str(),
        Some(std::env::current_dir().unwrap().to_string_lossy().as_ref())
    );
    assert_eq!(evidence["command"]["stdout"], "ok");
    assert_eq!(evidence["command"]["stdout_truncated"], true);
    assert!(!objects
        .join("blobs")
        .join(format!("{artifact_ref_id}.json"))
        .exists());

    let storage = run_storage_report(&storage::StorageReportOptions {
        start: repo_root,
        json_output: false,
        large_threshold_bytes: 1024,
        remotes: Vec::new(),
        quick: false,
    })
    .unwrap();
    assert_eq!(storage.external_artifacts.refs, 1);
    assert_eq!(storage.external_artifacts.referenced_bytes, 16);
    assert_eq!(storage.external_artifacts.payload_bytes_stored, 0);
}

#[test]
fn evidence_add_single_dry_run_returns_plan_without_writing_object() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let before = fs::read_dir(objects.join("evidence")).unwrap().count();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf dry-run".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    let result = run_evidence_add(&parsed).unwrap();
    let data = evidence_add_data_json(&result);

    assert!(result.dry_run);
    assert_eq!(result.evidence_id, "");
    assert_eq!(data["dry_run"], true);
    assert_eq!(data["created"], false);
    assert!(data["evidence_id"].is_null());
    assert_eq!(data["plan"]["command"], "evidence-add");
    assert_eq!(data["plan"]["would_apply"], false);
    assert_eq!(data["plan"]["cwd_source"], "caller_cwd");
    assert!(data["plan"]["resolved_cwd"].as_str().is_some());
    assert_eq!(
        fs::read_dir(objects.join("evidence")).unwrap().count(),
        before
    );
}

#[test]
fn evidence_artifact_ref_uses_repo_relative_path_and_warns_for_large_artifact() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let artifact_dir = repo_root.join("artifacts");
    let artifact_path = artifact_dir.join("large.bin");
    init_repo(&repo_root, false).unwrap();
    fs::create_dir_all(&artifact_dir).unwrap();
    fs::write(&artifact_path, vec![b'x'; 1_048_576]).unwrap();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--artifact".to_string(),
        artifact_path.to_string_lossy().into_owned(),
    ])
    .unwrap();
    let result = run_evidence_add(&parsed).unwrap();

    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "large_artifact"));
    let data = evidence_add_data_json(&result);
    assert!(data["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "large_artifact"));
    let artifact_ref_id = result.artifact_ref_id.as_ref().unwrap();
    let artifact_ref = codefire_store::read_object(
        &repo_root.join(".codefire").join("objects"),
        artifact_ref_id,
    )
    .unwrap();
    assert_eq!(artifact_ref["path_kind"], "repo_relative");
    assert_eq!(artifact_ref["path"], "artifacts/large.bin");
    assert_eq!(artifact_ref["uri"], "repo://artifacts/large.bin");
    assert_eq!(artifact_ref["local_path_redacted"], false);
    assert_eq!(artifact_ref["large_artifact"], true);
    assert_eq!(artifact_ref["large_artifact_threshold_bytes"], 1_048_576);
    assert_eq!(artifact_ref["hash_streaming"], true);
    assert_eq!(artifact_ref["hash_chunk_bytes"], 65536);
    assert_eq!(artifact_ref["size_bytes"], 1_048_576);
}

#[test]
fn evidence_command_failure_is_blocking_by_default_and_allowable_explicitly() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();

    let failing = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf nope >&2; exit 7".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    let error = run_evidence_add(&failing).unwrap_err();
    let envelope = cli_error_envelope("evidence-add", &error);
    assert_eq!(
        envelope["diagnostics"][0]["kind"],
        "evidence_command_failed"
    );
    assert_eq!(envelope["diagnostics"][0]["exit_code"], 7);
    assert_eq!(
        fs::read_dir(repo_root.join(".codefire").join("objects").join("evidence"))
            .unwrap()
            .count(),
        0
    );

    let allowed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf nope >&2; exit 7".to_string(),
        "--allow-failed-command".to_string(),
    ])
    .unwrap();
    let result = run_evidence_add(&allowed).unwrap();

    assert_eq!(result.command_exit_code, Some(7));
    assert!(!result.command_timed_out);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "evidence_command_failed"));
    let objects = repo_root.join(".codefire").join("objects");
    let evidence = codefire_store::read_object(&objects, &result.evidence_id).unwrap();
    assert_eq!(evidence["proof_status"], "failed-command-captured");
    assert_eq!(evidence["command"]["success"], false);
    assert_eq!(evidence["command"]["exit_code"], 7);
}

#[test]
fn evidence_command_timeout_requires_allow_failed_command_to_store() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "while :; do :; done".to_string(),
        "--timeout-ms=20".to_string(),
    ])
    .unwrap();
    let error = run_evidence_add(&parsed).unwrap_err();
    assert_eq!(error.exit_code(), ExitCode::InvalidUsageOrConfig.code());

    let allowed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "while :; do :; done".to_string(),
        "--timeout-ms=20".to_string(),
        "--allow-failed-command".to_string(),
    ])
    .unwrap();
    let result = run_evidence_add(&allowed).unwrap();
    assert_eq!(result.command_exit_code, None);
    assert!(result.command_timed_out);
}

#[test]
fn evidence_argv_command_capture_avoids_shell_expansion() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-argv".to_string(),
        "printf".to_string(),
        "--argv=%s".to_string(),
        "--argv".to_string(),
        "literal; echo injected".to_string(),
    ])
    .unwrap();
    let result = run_evidence_add(&parsed).unwrap();

    assert_eq!(result.command_exit_code, Some(0));
    let objects = repo_root.join(".codefire").join("objects");
    let evidence = codefire_store::read_object(&objects, &result.evidence_id).unwrap();
    assert_eq!(evidence["command"]["mode"], "argv");
    assert_eq!(evidence["command"]["shell"], false);
    assert_eq!(
        evidence["command"]["argv"],
        json!(["printf", "%s", "literal; echo injected"])
    );
    assert_eq!(evidence["command"]["stdout"], "literal; echo injected");
}

#[test]
fn evidence_batch_validates_all_items_before_writing_objects() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let artifact_path = temp.path().join("model.bin");
    let batch_path = temp.path().join("evidence-batch.json");
    let invalid_batch_path = temp.path().join("invalid-evidence-batch.json");
    init_repo(&repo_root, false).unwrap();
    fs::write(&artifact_path, b"artifact-payload").unwrap();
    fs::write(
        &batch_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "defaults": {"label": "batch-smoke", "max_output_bytes": 2},
            "items": [
                {"artifact": artifact_path, "artifact_uri": "artifact://model.bin"},
                {"from_command": "printf okay", "label": "command-smoke"},
                {"from_argv": ["printf", "%s", "argv-okay"], "label": "argv-smoke"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &invalid_batch_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "items": [
                {"from_command": "printf should-not-run"},
                {"artifact": temp.path().join("missing.bin")}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let parsed = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--batch".to_string(),
        batch_path.to_string_lossy().into_owned(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    assert!(parsed.dry_run);
    assert!(parsed.json_output);
    assert_eq!(parsed.batch_path.as_deref(), Some(batch_path.as_path()));

    let dry_run = run_evidence_batch(&parsed).unwrap();
    let dry_run_data = evidence_batch_data_json(&dry_run);
    assert_eq!(dry_run.item_count, 3);
    assert_eq!(dry_run_data["type"], "codefire_evidence_batch_result");
    assert_eq!(dry_run_data["dry_run"], true);
    assert_eq!(dry_run_data["plan"]["command"], "evidence-add-batch");
    let objects = repo_root.join(".codefire").join("objects");
    assert_eq!(fs::read_dir(objects.join("evidence")).unwrap().count(), 0);
    assert_eq!(
        fs::read_dir(objects.join("artifact_refs")).unwrap().count(),
        0
    );

    let applied = run_evidence_batch(&evidence::EvidenceAddOptions {
        start: repo_root.clone(),
        json_output: false,
        dry_run: false,
        batch_path: Some(batch_path),
        label: None,
        artifact_path: None,
        artifact_uri: None,
        command: None,
        command_argv: None,
        command_cwd: None,
        command_timeout_ms: 300_000,
        max_output_bytes: 64 * 1024,
        allow_failed_command: false,
    })
    .unwrap();
    assert_eq!(applied.item_count, 3);
    assert_eq!(applied.results.len(), 3);
    assert!(applied.results[0].artifact_ref_id.is_some());
    assert_eq!(applied.results[1].command_exit_code, Some(0));
    assert_eq!(applied.results[2].command_exit_code, Some(0));
    assert_eq!(fs::read_dir(objects.join("evidence")).unwrap().count(), 3);
    assert_eq!(
        fs::read_dir(objects.join("artifact_refs")).unwrap().count(),
        1
    );

    let before_invalid_evidence_count = fs::read_dir(objects.join("evidence")).unwrap().count();
    let invalid = run_evidence_batch(&evidence::EvidenceAddOptions {
        start: repo_root,
        json_output: false,
        dry_run: false,
        batch_path: Some(invalid_batch_path),
        label: None,
        artifact_path: None,
        artifact_uri: None,
        command: None,
        command_argv: None,
        command_cwd: None,
        command_timeout_ms: 300_000,
        max_output_bytes: 64 * 1024,
        allow_failed_command: false,
    })
    .unwrap_err();
    assert!(invalid.to_string().contains("missing.bin"));
    assert_eq!(
        fs::read_dir(objects.join("evidence")).unwrap().count(),
        before_invalid_evidence_count
    );
}

#[test]
fn extinguish_evidence_ref_links_resolution_and_verify_detects_missing_ref() {
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

    let evidence = run_evidence_add(&evidence::EvidenceAddOptions {
        start: open_dir.clone(),
        json_output: true,
        dry_run: false,
        batch_path: None,
        label: Some("verify evidence".to_string()),
        artifact_path: None,
        artifact_uri: None,
        command: Some("printf ok".to_string()),
        command_argv: None,
        command_cwd: None,
        command_timeout_ms: 300_000,
        max_output_bytes: 128,
        allow_failed_command: false,
    })
    .unwrap();
    let active = active_state_path(&open_dir);
    write_json_atomic(
        &active.join("fires.json"),
        &json!([{
            "type": "fire",
            "version": 1,
            "fire_uid": "fire_evidence_ref_test",
            "display_id": "FIRE-001",
            "status": "open",
            "severity": "required",
            "source": {"atom_id": "REQ-session"},
            "target": {"atom_id": "DES-session"},
            "reason": "needs explicit evidence",
            "trace_path": [],
            "created_by": "manual",
            "created_at": "2026-06-05T00:00:00Z",
            "key": "evidence-ref-test"
        }]),
    )
    .unwrap();
    let dry_run = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: String::new(),
        evidence: String::new(),
        evidence_refs: vec![evidence.evidence_id.clone()],
        refresh: false,
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        edit_rationale: false,
    })
    .unwrap();
    assert_eq!(dry_run.plan["resolution"]["has_evidence"], true);
    assert_eq!(
        dry_run.plan["resolution"]["evidence_refs"],
        json!([evidence.evidence_id.clone()])
    );
    let dry_run_envelope = plan_result_envelope("extinguish", &dry_run.plan);
    assert_eq!(
        dry_run_envelope["repo"],
        repo_root.to_string_lossy().as_ref()
    );
    assert_eq!(dry_run_envelope["next_actions"][0]["kind"], "verify");

    run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: String::new(),
        evidence: String::new(),
        evidence_refs: vec![evidence.evidence_id.clone()],
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: None,
        edit_rationale: false,
    })
    .unwrap();

    let resolutions: Vec<codefire_core::Resolution> = serde_json::from_value(
        read_json(&active_state_path(&open_dir).join("resolutions.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        resolutions[0].evidence_refs,
        vec![evidence.evidence_id.clone()]
    );
    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "passed");
    assert!(verification.missing_evidence_refs.is_empty());

    let commit = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Evidence-backed resolution".to_string(),
        dry_run: true,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let commit_data = commit_result_data_json(&commit);
    assert_eq!(commit_data["message"], "Evidence-backed resolution");
    assert_eq!(commit_data["extinguished_fire_count"], 1);
    assert_eq!(commit_data["active_resolution_count"], 1);
    assert_eq!(commit_data["evidence_ref_count"], 1);
    assert_eq!(commit.plan["extinguished_fire_count"], 1);
    assert_eq!(commit.plan["active_resolution_count"], 1);
    assert_eq!(commit.plan["evidence_ref_count"], 1);

    fs::remove_file(
        repo_root
            .join(".codefire")
            .join("objects")
            .join("evidence")
            .join(format!("{}.json", evidence.evidence_id)),
    )
    .unwrap();
    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "failed");
    assert_eq!(verification.missing_evidence_refs.len(), 1);
    assert_eq!(
        verification_exit_code(&verification),
        ExitCode::ObjectReferenceInvalid
    );
}

#[test]
fn remote_object_graph_copies_resolution_evidence_refs() {
    let temp = tempdir().unwrap();
    let source_objects = temp.path().join("source").join("objects");
    let target_objects = temp.path().join("target").join("objects");
    let artifact_ref = codefire_store::store_object(
        &source_objects,
        "artifact_ref",
        json!({
            "type": "artifact_ref",
            "version": 1,
            "path": "/tmp/model.bin",
            "uri": "artifact://model.bin",
            "hash_algorithm": "sha256",
            "content_hash": "sha256:abc",
            "size_bytes": 3,
            "captured_at": "2026-06-05T00:00:00Z",
        }),
    )
    .unwrap();
    let evidence = codefire_store::store_object(
        &source_objects,
        "evidence",
        json!({
            "type": "evidence",
            "version": 1,
            "label": "remote evidence",
            "artifact_ref": artifact_ref.clone(),
            "command": null,
            "created_at": "2026-06-05T00:00:00Z",
        }),
    )
    .unwrap();
    let resolution_ledger = codefire_store::store_object(
        &source_objects,
        "resolution_ledger",
        json!({
            "type": "resolution_ledger",
            "version": 1,
            "resolutions": [{
                "type": "resolution",
                "version": 1,
                "resolution_uid": "res_remote_evidence",
                "fire_uid": "fire_remote_evidence",
                "resolution_type": "addressed",
                "rationale": "",
                "evidence": "",
                "evidence_refs": [evidence.clone()],
                "basis": {
                    "source_atom": {"atom_id": "REQ-session", "content_hash": null},
                    "target_atom": {"atom_id": "DES-session", "content_hash": null},
                    "trace_links": [],
                    "policy_hash": "sha256:policy"
                },
                "resolved_at": "2026-06-05T00:00:00Z",
                "status": "active"
            }]
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(&source_objects);
    roots.insert(
        "resolution_ledger".to_string(),
        Value::String(resolution_ledger),
    );
    let commit = codefire_store::store_object(
        &source_objects,
        "commit",
        codefire_store::commit_payload(Vec::new(), roots, consistent_certificate()),
    )
    .unwrap();
    let records = crate::remote::collect_object_records(&source_objects, &commit).unwrap();
    let diagnostics = crate::remote::upload_object_graph_diagnostics(&records);
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "artifact_path_sensitive"));

    copy_object_graph(&source_objects, &target_objects, &commit).unwrap();

    let copied_commit = codefire_store::read_object(&target_objects, &commit).unwrap();
    assert_eq!(copied_commit["type"], "commit");
    let copied_evidence = codefire_store::read_object(&target_objects, &evidence).unwrap();
    assert_eq!(copied_evidence["type"], "evidence");
    let copied_artifact = codefire_store::read_object(&target_objects, &artifact_ref).unwrap();
    assert_eq!(copied_artifact["type"], "artifact_ref");
}

#[test]
fn explain_fire_atom_verify_and_storage_targets_return_actions() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("docs").join("design").join("session.md"),
        "## DES-session: Design\nClock policy\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();

    let fire_id = compute_scan(&open_dir, false)
        .unwrap()
        .scan
        .open_fires
        .first()
        .unwrap()
        .display_id
        .clone();
    let fire_options = parse_explain_args(&[
        "fire".to_string(),
        fire_id.clone(),
        "--path".to_string(),
        open_dir.to_string_lossy().into_owned(),
        "--json".to_string(),
    ])
    .unwrap();
    assert!(fire_options.json_output);
    assert_eq!(fire_options.target, explain::ExplainTarget::Fire(fire_id));
    let fire = run_explain(&fire_options).unwrap();
    assert_eq!(fire.data["type"], "codefire_explain");
    assert_eq!(fire.data["target"]["kind"], "fire");
    assert!(fire
        .next_actions
        .iter()
        .any(|action| action["id"] == "extinguish_fire"));
    assert_common_next_action_schema(&fire.next_actions[0]);

    let atom = run_explain(
        &parse_explain_args(&[
            "atom".to_string(),
            "REQ-session".to_string(),
            "--path".to_string(),
            open_dir.to_string_lossy().into_owned(),
            "--depth=2".to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(atom.data["target"]["kind"], "atom");
    assert!(atom.data["context"]["atoms"].as_array().unwrap().len() >= 2);

    fs::remove_file(open_dir.join("codefire.links.yaml")).unwrap();
    let verify = run_explain(
        &parse_explain_args(&[
            "verify-failure".to_string(),
            "--path".to_string(),
            open_dir.to_string_lossy().into_owned(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(verify.data["target"]["kind"], "verify-failure");
    assert!(!verify.diagnostics.is_empty());
    assert!(!verify.next_actions.is_empty());

    let storage = run_explain(
        &parse_explain_args(&[
            "storage-warning".to_string(),
            "--path".to_string(),
            repo_root.to_string_lossy().into_owned(),
            "--large-threshold=1".to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(storage.data["target"]["kind"], "storage-warning");
    assert!(!storage.diagnostics.is_empty());
    assert!(!storage.next_actions.is_empty());
}

#[test]
fn migrate_check_and_dry_run_report_compatibility_and_blockers() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();

    let parsed = parse_migrate_args(&[
        "check".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--json".to_string(),
    ])
    .unwrap();
    assert_eq!(parsed.mode, migration::MigrateMode::Check);
    assert!(parsed.json_output);
    let report = run_migrate(&parsed).unwrap();
    let data = migration_report_data_json(&report);

    assert!(report.compatible);
    assert!(report.checked_objects >= 8);
    assert_eq!(report.checked_branches, 1);
    assert_eq!(data["type"], "codefire_migration_report");
    assert_eq!(data["target_format"], "current");
    assert!(data["supported_target_formats"]
        .as_array()
        .unwrap()
        .iter()
        .any(|format| format == "current"));
    assert_eq!(data["compatible"], true);
    assert!(migration_report_diagnostics_json(&report).is_empty());
    assert!(migration_report_next_actions(&report).is_empty());

    let quick = run_migrate(
        &parse_migrate_args(&[
            "check".to_string(),
            repo_root.to_string_lossy().into_owned(),
            "--quick".to_string(),
            "--json".to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert!(quick.compatible);
    assert_eq!(quick.checked_objects, 0);
    assert_eq!(migration_report_data_json(&quick)["scan_mode"], "quick");
    assert_eq!(quick.skipped_checks, vec!["object_record_integrity"]);

    let dry_run = run_migrate(
        &parse_migrate_args(&[
            "dry-run".to_string(),
            repo_root.to_string_lossy().into_owned(),
            "--target-format=v0.6".to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(dry_run.mode, migration::MigrateMode::DryRun);
    assert!(dry_run.compatible);
    assert!(dry_run.planned_actions.is_empty());

    fs::remove_dir_all(repo_root.join(".codefire").join("idempotency")).unwrap();
    fs::remove_dir_all(
        repo_root
            .join(".codefire")
            .join("objects")
            .join("artifact_refs"),
    )
    .unwrap();
    let doctor = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        json_output: false,
        quick: true,
    })
    .unwrap();
    assert!(doctor.ok);
    assert!(doctor
        .issues
        .iter()
        .any(|issue| issue.kind == "missing_repairable_directory"));
    assert!(doctor
        .issues
        .iter()
        .any(|issue| issue.kind == "missing_repairable_object_subdirectory"));
    let repair_plan = run_migrate(&parsed).unwrap();
    assert!(repair_plan.compatible);
    assert!(repair_plan
        .planned_actions
        .iter()
        .any(|action| action.path.ends_with("idempotency")));
    assert!(repair_plan
        .planned_actions
        .iter()
        .any(|action| action.path.ends_with("artifact_refs")));

    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": "broken",
        "head": "CF-COMMIT-missing",
        "state": "closed",
        "created_at": "2026-06-05T00:00:00Z"
    });
    save_branch_record(&repo_root, &branch).unwrap();
    let broken = run_migrate(&parsed).unwrap();

    assert!(!broken.compatible);
    assert!(broken
        .blockers
        .iter()
        .any(|issue| issue.kind == "invalid_branch_head"));
    assert!(!migration_report_diagnostics_json(&broken).is_empty());
    assert_eq!(
        migration_report_next_actions(&broken)[0]["id"],
        "inspect_migration_blockers"
    );
    assert_common_next_action_schema(&migration_report_next_actions(&broken)[0]);
}

#[test]
fn parse_review_pack_args_accepts_base_output_and_algorithm() {
    let args = vec![
        "feature-session".to_string(),
        "--base".to_string(),
        "main".to_string(),
        "--output".to_string(),
        "review-pack.json".to_string(),
        "--algorithm=patience".to_string(),
        "--no-rename-detection".to_string(),
        "--json".to_string(),
    ];
    let parsed = parse_review_pack_args(&args).unwrap();
    assert_eq!(parsed.review.source, "feature-session");
    assert_eq!(parsed.review.base.as_deref(), Some("main"));
    assert_eq!(parsed.review.algorithm, DiffAlgorithm::Patience);
    assert!(!parsed.review.rename_detection);
    assert_eq!(parsed.output, Some(PathBuf::from("review-pack.json")));
    assert!(parsed.json_output);
}

#[test]
fn review_pack_exports_file_atom_verification_and_next_actions() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 30\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-base",
        )],
        vec![],
    );
    let feature_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 15\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-feature",
        )],
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let pack = review_pack_with_options(
        Some(&repo_root),
        &ReviewPackOptions {
            source: "feature-session".to_string(),
            base: None,
            algorithm: DiffAlgorithm::Myers,
            rename_detection: true,
        },
    )
    .unwrap();
    let json: Value = serde_json::from_str(&pack).unwrap();

    assert_eq!(json["type"], "codefire_review_pack");
    assert_eq!(json["version"], 1);
    assert!(json["source"]["label"]
        .as_str()
        .unwrap()
        .starts_with("feature-session@"));
    assert!(json["base"]["label"]
        .as_str()
        .unwrap()
        .starts_with("parent@"));
    assert!(json["file_diff"]["text"]
        .as_str()
        .unwrap()
        .contains("-TTL 30"));
    assert!(json["file_diff"]["text"]
        .as_str()
        .unwrap()
        .contains("+TTL 15"));
    assert_eq!(json["semantic_diff"]["type"], "codefire_diff");
    assert_eq!(
        json["semantic_diff"]["atoms"]["changed"][0]["atom_id"],
        "REQ-session"
    );
    assert!(json["semantic_diff"]["trace"].is_object());
    assert!(json["verification"]["source"].is_object());
    assert!(json["next_actions"].is_array());

    let result = review_pack_result_data(Some(&PathBuf::from("review-pack.json")), &pack).unwrap();
    assert_eq!(result["type"], "codefire_review_pack_result");
    assert_eq!(result["output_path"], "review-pack.json");
    assert_eq!(result["payload_in_envelope"], false);
    assert!(result["included_sections"]
        .as_array()
        .unwrap()
        .iter()
        .any(|section| section == "semantic_diff"));
}

#[test]
fn parse_patch_args_accept_export_and_import_options() {
    let export = parse_patch_export_args(&[
        "feature-session".to_string(),
        "--base".to_string(),
        "main".to_string(),
        "--json".to_string(),
        "--max-file-bytes=4096".to_string(),
        "--max-payload-bytes".to_string(),
        "8192".to_string(),
        "--include-large-files".to_string(),
        "--output".to_string(),
        "change.cfpatch.json".to_string(),
    ])
    .unwrap();
    assert_eq!(export.patch.source, "feature-session");
    assert_eq!(export.patch.base.as_deref(), Some("main"));
    assert!(export.json_output);
    assert_eq!(export.patch.max_file_bytes, 4096);
    assert_eq!(export.patch.max_payload_bytes, 8192);
    assert!(export.patch.include_large_files);
    assert_eq!(export.output, Some(PathBuf::from("change.cfpatch.json")));

    let import = parse_patch_import_args(&[
        "change.cfpatch.json".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--idempotency-key".to_string(),
        "patch-key-1".to_string(),
    ])
    .unwrap();
    assert_eq!(import.path, PathBuf::from("change.cfpatch.json"));
    assert!(import.dry_run);
    assert!(import.json_output);
    assert_eq!(import.idempotency_key.as_deref(), Some("patch-key-1"));
}

#[test]
fn patch_export_json_metadata_and_large_file_omission_are_bounded() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "model.bin", "small\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "model.bin",
        "0123456789abcdef\n",
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-large",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let patch = patch_export_with_options(
        Some(&repo_root),
        &PatchExportOptions {
            source: "feature-large".to_string(),
            base: Some(base_commit),
            max_file_bytes: 4,
            max_payload_bytes: DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES,
            include_large_files: false,
        },
    )
    .unwrap();
    let patch_json: Value = serde_json::from_str(&patch).unwrap();
    assert_eq!(patch_json["summary"]["omitted_entries"], 1);
    assert_eq!(patch_json["entries"][0]["action"], "omit");
    assert_eq!(
        patch_json["entries"][0]["reason"],
        "file_exceeds_max_file_bytes"
    );
    assert!(patch_json["entries"][0].get("content").is_none());

    let result = patch_export_result_data(None, &patch).unwrap();
    assert_eq!(result["type"], "codefire_patch_export_result");
    assert_eq!(result["payload_in_envelope"], false);
    assert_eq!(result["summary"]["max_file_bytes"], 4);
    assert_eq!(result["omissions"].as_array().unwrap().len(), 1);
    assert_eq!(
        result["next_actions"][0]["kind"],
        "patch_export_include_large_files"
    );
}

#[test]
fn patch_export_import_applies_manifest_delta_to_open_directory() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let patch_path = temp.path().join("change.cfpatch.json");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[
            (
                "docs/spec/session.md",
                "## REQ-session: Requirement\nTTL 30\n",
            ),
            ("docs/obsolete.md", "old\n"),
        ],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-base",
        )],
        vec![],
    );
    let feature_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 15\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-feature",
        )],
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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
    let patch = patch_export_with_options(
        Some(&repo_root),
        &PatchExportOptions {
            source: "feature-session".to_string(),
            base: Some("main".to_string()),
            max_file_bytes: DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES,
            max_payload_bytes: DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES,
            include_large_files: false,
        },
    )
    .unwrap();
    fs::write(&patch_path, &patch).unwrap();
    let patch_json: Value = serde_json::from_str(&patch).unwrap();
    assert_eq!(patch_json["type"], "codefire_patch");
    assert_eq!(patch_json["entries"].as_array().unwrap().len(), 2);

    let dry_run = import_patch(
        &open_dir,
        &PatchImportOptions {
            path: patch_path.clone(),
            dry_run: true,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    assert_eq!(dry_run["applied"], false);
    assert_eq!(
        fs::read_to_string(open_dir.join("docs/spec/session.md")).unwrap(),
        "## REQ-session: Requirement\nTTL 30\n"
    );
    assert!(open_dir.join("docs/obsolete.md").exists());
    assert_eq!(read_status(&open_dir).unwrap().state, "open-clean");

    let applied = import_patch(
        &open_dir,
        &PatchImportOptions {
            path: patch_path,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(
        fs::read_to_string(open_dir.join("docs/spec/session.md")).unwrap(),
        "## REQ-session: Requirement\nTTL 15\n"
    );
    assert!(!open_dir.join("docs/obsolete.md").exists());
    assert_eq!(read_status(&open_dir).unwrap().state, "open-burning");
}

#[test]
fn patch_import_idempotency_replays_same_payload_and_rejects_conflict() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let patch_path = temp.path().join("change.cfpatch.json");
    let other_patch_path = temp.path().join("other-change.cfpatch.json");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "docs/spec/session.md", "TTL 30\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/spec/session.md",
        "TTL 15\n",
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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
    let patch = patch_export_with_options(
        Some(&repo_root),
        &PatchExportOptions {
            source: "feature-session".to_string(),
            base: Some("main".to_string()),
            max_file_bytes: DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES,
            max_payload_bytes: DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES,
            include_large_files: false,
        },
    )
    .unwrap();
    fs::write(&patch_path, &patch).unwrap();
    let mut other_patch: Value = serde_json::from_str(&patch).unwrap();
    other_patch["source"]["commit"] = Value::String("CF-COMMIT-different".to_string());
    fs::write(
        &other_patch_path,
        serde_json::to_string_pretty(&other_patch).unwrap(),
    )
    .unwrap();

    let first = import_patch(
        &open_dir,
        &PatchImportOptions {
            path: patch_path.clone(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("patch-import-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(first["applied"], true);
    assert_eq!(read_status(&open_dir).unwrap().state, "open-burning");

    let replay = import_patch(
        &open_dir,
        &PatchImportOptions {
            path: patch_path,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("patch-import-once".to_string()),
        },
    )
    .unwrap();
    assert_eq!(replay["applied"], true);
    assert_eq!(replay["branch"], "main");

    let conflict = import_patch(
        &open_dir,
        &PatchImportOptions {
            path: other_patch_path,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("patch-import-once".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(conflict.exit_code(), ExitCode::IdempotencyConflict.code());
}

#[test]
fn file_remote_upload_clone_show_diff_and_merge_request_flow() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let clone_repo = temp.path().join("clone-repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    init_repo(&clone_repo, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "hello 30\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "hello 15\n",
        vec![main_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": main_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature-session");
    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;

    let upload_dry_run = upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: true,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(upload_dry_run.plan["type"], "codefire_operation_plan");
    assert_eq!(upload_dry_run.plan["command"], "upload");
    let upload_envelope = plan_result_envelope("upload", &upload_dry_run.plan);
    assert_eq!(
        upload_envelope["repo"],
        repo_root.to_string_lossy().as_ref()
    );
    assert_eq!(upload_envelope["next_actions"][0]["kind"], "list_remote");
    assert!(!remote_project_root.exists());

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    let branches = list_remote_branches(&project_url).unwrap();
    assert_eq!(branches[0].name, "main");
    assert_eq!(
        branches[0].head,
        load_branch_record(&repo_root, "main").unwrap()["head"]
    );
    let remote_list = remote_branch_list_data_json(&project_url, &branches);
    assert_eq!(remote_list["type"], "codefire_remote_branch_list");
    assert_eq!(remote_list["project_url"], project_url);
    assert_eq!(remote_list["branches"][0]["name"], "main");

    clone_branch(
        &clone_repo,
        &CloneOptions {
            source: main_url.clone(),
            new_branch: "main-from-remote".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let cloned = load_branch_record(&clone_repo, "main-from-remote").unwrap();
    assert_eq!(
        required_string(&cloned, &["head"]).unwrap(),
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap()
    );

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature-session".to_string(),
            remote_url: feature_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    let show = show_commitish(None, &feature_url).unwrap();
    assert!(show.contains(&format!("Object: {feature_url}@CF-COMMIT-")));
    assert!(show.contains("Certificate: consistent"));
    let diff = diff_commitish_with_options(None, &main_url, &feature_url, &DiffOptions::default())
        .unwrap();
    assert!(diff.contains(&format!("--- {main_url}@CF-COMMIT-")));
    assert!(diff.contains(&format!("+++ {feature_url}@CF-COMMIT-")));
    assert!(diff.contains("+hello 15"));

    let request_dir = remote_dirs(&remote_project_root).merge_requests;
    assert_eq!(fs::read_dir(&request_dir).unwrap().count(), 0);
    let mr_dry_run = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: main_url.clone(),
        dry_run: true,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(mr_dry_run.plan["command"], "request-merge");
    assert!(mr_dry_run.id.starts_with("MR-"));
    assert_eq!(fs::read_dir(&request_dir).unwrap().count(), 0);

    let mr = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: main_url.clone(),
        dry_run: false,
        json_output: false,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(mr.id.starts_with("MR-"));
    let listed = list_merge_requests(&project_url).unwrap();
    assert_eq!(listed[0].status, "open");
    let request_list = merge_request_list_data_json(&project_url, &listed);
    assert_eq!(request_list["type"], "codefire_merge_request_list");
    assert_eq!(request_list["project_url"], project_url);
    assert_eq!(request_list["merge_requests"][0]["status"], "open");
    let mr_path = request_dir.join(format!("{}.json", mr.id));
    let open_mr_before_review = read_json(&mr_path).unwrap();
    let review_dry_run = review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: mr.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "sealed source is ready".to_string(),
        dry_run: true,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(review_dry_run.plan["command"], "request-review");
    assert_eq!(read_json(&mr_path).unwrap(), open_mr_before_review);
    review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: mr.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "sealed source is ready".to_string(),
        dry_run: false,
        json_output: false,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "approved"
    );
    let approved_mr_before_apply = read_json(&mr_path).unwrap();
    let main_before_apply = required_string(
        &load_remote_branch(&parse_cf_url(&main_url).unwrap()).unwrap(),
        &["head"],
    )
    .unwrap();
    let apply_dry_run = apply_merge_request(&RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: mr.id.clone(),
        dry_run: true,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(apply_dry_run.plan["command"], "request-apply");
    assert_eq!(
        required_string(
            &load_remote_branch(&parse_cf_url(&main_url).unwrap()).unwrap(),
            &["head"]
        )
        .unwrap(),
        main_before_apply
    );
    assert_eq!(read_json(&mr_path).unwrap(), approved_mr_before_apply);
    let applied = apply_merge_request(&RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: mr.id,
        dry_run: false,
        json_output: false,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.target_branch, "main");
    assert_eq!(
        applied.head,
        required_string(
            &load_remote_branch(&parse_cf_url(&feature_url).unwrap()).unwrap(),
            &["head"]
        )
        .unwrap()
    );
    assert_eq!(
        required_string(
            &load_remote_branch(&parse_cf_url(&main_url).unwrap()).unwrap(),
            &["head"]
        )
        .unwrap(),
        applied.head
    );
}

#[test]
fn remote_mutator_idempotency_replays_same_payload_and_rejects_conflicts() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "hello 30\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "hello 15\n",
        vec![main_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": main_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature-session");
    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;

    let first_upload = upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: Some("upload-main-once".to_string()),
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    let replay_upload = upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: Some("upload-main-once".to_string()),
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(replay_upload.head, first_upload.head);
    assert_eq!(replay_upload.plan["replayed"], true);
    let upload_record = read_json(
        &remote_dirs(&remote_project_root)
            .idempotency
            .join("remote_upload")
            .join("upload-main-once.json"),
    )
    .unwrap();
    assert_eq!(upload_record["payload_mode"], "hash_only");
    assert!(upload_record.get("payload").is_none());
    assert!(upload_record["result"].get("plan").is_none());
    let upload_conflict = upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature-session".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: Some("upload-main-once".to_string()),
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap_err();
    assert_eq!(
        upload_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature-session".to_string(),
            remote_url: feature_url.clone(),
            dry_run: false,
            json_output: false,
            idempotency_key: Some("upload-feature-once".to_string()),
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();

    let first_merge = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: main_url.clone(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("request-merge-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    let replay_merge = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: main_url.clone(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("request-merge-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(replay_merge.id, first_merge.id);
    let merge_conflict = request_merge(&RequestMergeOptions {
        source_url: main_url.clone(),
        target_url: feature_url.clone(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("request-merge-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_eq!(
        merge_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );

    let mr_path = remote_dirs(&remote_project_root)
        .merge_requests
        .join(format!("{}.json", first_merge.id));
    let first_review = review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: first_merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "ready".to_string(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("review-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    let replay_review = review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: first_merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "ready".to_string(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("review-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(replay_review.decision, first_review.decision);
    assert_eq!(
        read_json(&mr_path).unwrap()["reviews"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let review_conflict = review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: first_merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "changed comment".to_string(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("review-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_eq!(
        review_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );

    let first_apply = apply_merge_request(&RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: first_merge.id.clone(),
        dry_run: false,
        json_output: false,
        idempotency_key: Some("apply-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    let replay_apply = apply_merge_request(&RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: first_merge.id,
        dry_run: false,
        json_output: false,
        idempotency_key: Some("apply-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(replay_apply.head, first_apply.head);

    let other_merge = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: feature_url,
        dry_run: false,
        json_output: false,
        idempotency_key: Some("other-request-merge".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    let apply_conflict = apply_merge_request(&RequestApplyOptions {
        project_url,
        mr_id: other_merge.id,
        dry_run: false,
        json_output: false,
        idempotency_key: Some("apply-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap_err();
    assert_eq!(
        apply_conflict.exit_code(),
        ExitCode::IdempotencyConflict.code()
    );
}

#[test]
fn merge_branch_writes_source_changes_and_marks_target_burning() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 30\n\n## DES-session: Design\nClock policy\n",
    );
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 15\n\n## DES-session: Design\nClock policy\n",
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert!(result.conflicts.is_empty());
    assert_eq!(
        fs::read_to_string(open_dir.join("docs/spec/session.md")).unwrap(),
        "## REQ-session: Requirement\nTTL 15\n\n## DES-session: Design\nClock policy\n"
    );
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();
    let scan = run_scan(&open_dir).unwrap();
    assert!(scan
        .open_fires
        .iter()
        .any(|fire| fire.reason == "merge_changed"));
    let status = read_status(&open_dir).unwrap();
    assert_eq!(status.state, "open-burning");
    let active_state_path = PathBuf::from(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "active_state_path"],
        )
        .unwrap(),
    );
    let state = read_json(&active_state_path.join("state.json")).unwrap();
    let expected_parent = required_string(
        &load_branch_record(&repo_root, "feature-session").unwrap(),
        &["head"],
    )
    .unwrap();
    assert_eq!(
        state.get("pending_merge_parent").and_then(Value::as_str),
        Some(expected_parent.as_str())
    );
}

#[test]
fn merge_idempotency_replays_same_payload_and_rejects_conflict() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let main_open_dir = temp.path().join("main-open");
    let other_open_dir = temp.path().join("other-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "docs/spec/session.md", "TTL 30\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/spec/session.md",
        "TTL 15\n",
        vec![base_commit.clone()],
    );
    for branch in ["main", "other"] {
        save_branch_record(
            &repo_root,
            &json!({
                "type": "branch",
                "version": 1,
                "name": branch,
                "head": base_commit,
                "state": "closed",
                "created_at": "2026-06-04T00:00:00Z"
            }),
        )
        .unwrap();
    }
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: main_open_dir.clone(),
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
            branch: "other".to_string(),
            path: other_open_dir,
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    let first = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("merge-feature-once".to_string()),
        },
    )
    .unwrap();
    assert!(first.applied);
    assert_eq!(read_status(&main_open_dir).unwrap().state, "open-burning");

    let replay = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("merge-feature-once".to_string()),
        },
    )
    .unwrap();
    assert!(replay.applied);
    assert_eq!(replay.target_branch, "main");
    assert_eq!(
        replay.file_actions[0].content.as_deref(),
        Some("TTL 15\n".as_bytes())
    );

    let conflict = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "other".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("merge-feature-once".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(conflict.exit_code(), ExitCode::IdempotencyConflict.code());
}

#[test]
fn merge_branch_writes_conflict_markers_for_divergent_changes() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "src/session.py", "def ttl():\n    return 30\n");
    let target_commit = write_file_commit_with_parents(
        &objects,
        "src/session.py",
        "def ttl():\n    return 45\n",
        vec![base_commit.clone()],
    );
    let source_commit = write_file_commit_with_parents(
        &objects,
        "src/session.py",
        "def ttl():\n    return 15\n",
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": target_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": source_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert_eq!(result.conflicts, vec!["src/session.py".to_string()]);
    let content = fs::read_to_string(open_dir.join("src").join("session.py")).unwrap();
    assert!(content.contains("<<<<<<< target"));
    assert!(content.contains("return 45"));
    assert!(content.contains("return 15"));
    assert!(content.contains(">>>>>>> source"));
}

#[test]
fn merge_branch_writes_binary_conflict_sides_without_lossy_markers() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_bytes_commit(&objects, "assets/model.bin", b"\x00base");
    let target_commit = write_file_bytes_commit_with_parents(
        &objects,
        "assets/model.bin",
        b"\x00target",
        vec![base_commit.clone()],
    );
    let source_commit = write_file_bytes_commit_with_parents(
        &objects,
        "assets/model.bin",
        b"\x00source",
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": target_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-binary",
            "head": source_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-binary".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert_eq!(result.conflicts, vec!["assets/model.bin".to_string()]);
    assert_eq!(result.binary_conflicts.len(), 1);
    let conflict = &result.binary_conflicts[0];
    assert_eq!(conflict.target_bytes, Some(7));
    assert_eq!(conflict.source_bytes, Some(7));
    assert_eq!(
        fs::read(open_dir.join("assets").join("model.bin")).unwrap(),
        b"\x00target"
    );
    assert_eq!(
        fs::read(
            open_dir
                .join(".codefire-conflicts")
                .join("assets")
                .join("model.bin")
                .join("target")
        )
        .unwrap(),
        b"\x00target"
    );
    assert_eq!(
        fs::read(
            open_dir
                .join(".codefire-conflicts")
                .join("assets")
                .join("model.bin")
                .join("source")
        )
        .unwrap(),
        b"\x00source"
    );
    let state = read_json(&active_state_path(&open_dir).join("state.json")).unwrap();
    assert_eq!(
        state["binary_merge_conflicts"][0]["path"],
        "assets/model.bin"
    );
    let rendered = render_merge_result_json(&result).unwrap();
    let json: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(json["binary_conflicts"][0]["path"], "assets/model.bin");
    assert_eq!(json["next_actions"][0]["binary"], true);
}

#[test]
fn common_ancestor_handles_deep_generation_graph() {
    let temp = tempdir().unwrap();
    let objects = temp.path().join("objects");
    let root = write_generation_commit(&objects, Vec::new(), 0);
    let mut base = root;
    for generation in 1..=40 {
        base = write_generation_commit(&objects, vec![base], generation);
    }
    let split = base.clone();
    let mut left = split.clone();
    for generation in 41..=80 {
        left = write_generation_commit_with_label(&objects, vec![left], generation, "left");
    }
    let mut right = split.clone();
    for generation in 41..=70 {
        right = write_generation_commit_with_label(&objects, vec![right], generation, "right");
    }

    let ancestor = common_ancestor(&objects, &left, &right).unwrap();

    assert_eq!(ancestor.as_deref(), Some(split.as_str()));
}

#[test]
fn merge_dry_run_reports_plan_without_writing_target() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 30\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-base",
        )],
        vec![],
    );
    let target_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 45\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-target",
        )],
        vec![base_commit.clone()],
    );
    let source_commit = write_file_commit_with_atoms(
        &objects,
        &[(
            "docs/spec/session.md",
            "## REQ-session: Requirement\nTTL 15\n",
        )],
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:req-source",
        )],
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": target_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": source_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
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

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
            dry_run: true,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    assert!(!result.applied);
    assert_eq!(result.conflicts, vec!["docs/spec/session.md".to_string()]);
    assert_eq!(
        result.file_actions[0].action,
        MergeAction::WriteConflictMarkers
    );
    assert_eq!(result.semantic_conflicts[0].atom_id, "REQ-session");
    assert_eq!(
        fs::read_to_string(open_dir.join("docs/spec/session.md")).unwrap(),
        "## REQ-session: Requirement\nTTL 45\n"
    );
    assert_eq!(read_status(&open_dir).unwrap().state, "open-clean");
    let rendered = render_merge_result_json(&result).unwrap();
    let json: Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(json["type"], "codefire_merge_plan");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["file_actions"][0]["path"], "docs/spec/session.md");
    assert_eq!(json["semantic_conflicts"][0]["atom_id"], "REQ-session");
    assert!(json["next_actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action["kind"] == "resolve_file_conflict"));

    assert_eq!(
        normalize_merge_dry_run(&render_merge_dry_run(&result)).trim_end(),
        include_str!("../tests/fixtures/merge_dry_run.golden").trim_end()
    );
}

#[test]
fn status_reads_python_compatible_open_directory() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();
    let open_dir = repo_root.join("main");
    let cf = repo_root.join(".codefire");
    fs::create_dir_all(&open_dir).unwrap();
    fs::create_dir_all(cf.join("opened")).unwrap();
    fs::create_dir_all(cf.join("active").join("open_123")).unwrap();

    let commit_id = write_valid_commit(&cf.join("objects"));
    fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": "main", "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();
    fs::write(
        repo_root.join(".codefire").join("opened").join("main.json"),
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "branch": {"name": "main"},
            "open": {
                "path": open_dir,
                "open_instance_id": "open_123",
                "opened_from_commit": commit_id,
                "current_base_commit": commit_id,
                "active_state_path": repo_root.join(".codefire").join("active").join("open_123")
            },
            "state": {"last_known": "open-consistent"}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        repo_root
            .join(".codefire")
            .join("active")
            .join("open_123")
            .join("fires.json"),
        serde_json::to_string_pretty(&json!([
            {"display_id": "FIRE-001", "status": "open"},
            {"display_id": "FIRE-002", "status": "extinguished"}
        ]))
        .unwrap(),
    )
    .unwrap();

    let status = read_status(&open_dir).unwrap();

    assert_eq!(
        status,
        Status {
            branch: "main".to_string(),
            state: "open-burning".to_string(),
            base: commit_id,
            open_fires: 1
        }
    );
}

#[test]
fn verify_clean_branch_preserves_clean_state_without_degrading_status() {
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

    let before = read_status(&open_dir).unwrap();
    assert_eq!(before.state, "open-clean");

    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "passed");

    let after = read_status(&open_dir).unwrap();
    assert_eq!(after.state, "open-clean");
    assert_eq!(after.open_fires, 0);
    let active = active_state_path(&open_dir);
    assert!(active.join("scan.json").exists());
    assert!(active.join("fires.json").exists());
    assert!(active.join("verification.json").exists());
    assert_eq!(
        read_json(&active.join("state.json")).unwrap()["state"],
        "open-clean"
    );
}

#[test]
fn metrics_attach_to_status_scan_and_verify_data() {
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
    fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
    fs::write(
        open_dir
            .join("docs")
            .join("requirements")
            .join("session.md"),
        "## REQ-session: Requirement\nTTL 30\n",
    )
    .unwrap();

    let status = read_status(&open_dir).unwrap();
    let status_data = attach_metrics(
        status_data_json(&status),
        Some(&status_metrics(Duration::from_millis(3), &status)),
    );
    assert_eq!(status_data["metrics"]["type"], "codefire_metrics");
    assert_eq!(status_data["metrics"]["command"], "status");
    assert_eq!(status_data["metrics"]["phase_timings"]["status_load_ms"], 3);

    let scan = run_scan(&open_dir).unwrap();
    let scan_data = attach_metrics(
        scan_data_json_with_full(&scan, false),
        Some(&scan_metrics(Duration::from_millis(5), &scan)),
    );
    assert_eq!(scan_data["metrics"]["command"], "scan");
    assert_eq!(scan_data["metrics"]["phase_timings"]["total_ms"], 5);
    assert_eq!(scan_data["metrics"]["phase_timings"]["scan_pipeline_ms"], 5);
    assert_eq!(
        scan_data["metrics"]["phase_timings"]["atom_extraction_ms"],
        Value::Null
    );
    assert!(scan_data["metrics"]["phases"]
        .as_array()
        .unwrap()
        .iter()
        .any(|phase| phase["name"] == "trace_parse" && phase["measured"] == false));
    assert!(scan_data["metrics"]["counters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|counter| counter["name"] == "atoms" && counter["value"] == 1));
    assert_eq!(scan_data["metrics"]["cache_status"], "unavailable");
    assert!(scan_data["metrics"].get("cache").is_none());

    let branch_data = attach_metrics(
        json!({
            "type": "codefire_branch_list",
            "version": 1,
            "branches": [],
        }),
        Some(&branch_list_metrics(
            Duration::from_millis(4),
            &[
                Branch {
                    name: "main".to_string(),
                    head: "CF-COMMIT-main".to_string(),
                    state: "closed".to_string(),
                },
                Branch {
                    name: "feature".to_string(),
                    head: "CF-COMMIT-feature".to_string(),
                    state: "open-clean".to_string(),
                },
            ],
        )),
    );
    assert_eq!(branch_data["metrics"]["command"], "branch-list");
    assert_eq!(
        branch_data["metrics"]["phase_timings"]["branch_list_load_ms"],
        4
    );
    assert!(branch_data["metrics"]["counters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|counter| counter["name"] == "branches" && counter["value"] == 2));
    assert!(branch_data["metrics"]["counters"]
        .as_array()
        .unwrap()
        .iter()
        .any(|counter| counter["name"] == "open_branches" && counter["value"] == 1));

    let verification = run_verify(&open_dir).unwrap();
    let verify_data = attach_metrics(
        verification_data_json_with_filter(&verification, "blocking_only"),
        Some(&verification_metrics(
            Duration::from_millis(7),
            &verification,
        )),
    );
    assert_eq!(verify_data["diagnostic_filter"], "blocking_only");
    assert_eq!(verify_data["metrics"]["command"], "verify");
    assert_eq!(verify_data["metrics"]["phase_timings"]["total_ms"], 7);
    assert_eq!(
        verify_data["metrics"]["phase_timings"]["verification_pipeline_ms"],
        7
    );
    assert!(verify_data["metrics"]["phases"]
        .as_array()
        .unwrap()
        .iter()
        .any(|phase| phase["name"] == "verification_commands" && phase["measured"] == false));
}

#[test]
fn branch_list_reads_repo_from_open_marker_and_validates_heads() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("worktree");
    let cf = repo_root.join(".codefire");
    fs::create_dir_all(cf.join("branches")).unwrap();
    fs::create_dir_all(cf.join("opened")).unwrap();
    fs::create_dir_all(&open_dir).unwrap();

    let commit_id = write_valid_commit(&cf.join("objects"));
    let branch_name = "feature/a b%雪";
    fs::write(
        cf.join("branches")
            .join(format!("{}.json", ref_file_name(branch_name))),
        serde_json::to_string_pretty(&json!({
            "type": "branch",
            "version": 1,
            "name": branch_name,
            "head": commit_id,
            "state": "open-clean",
            "created_at": "2026-06-04T00:00:00Z"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": branch_name, "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();
    fs::write(
        cf.join("opened")
            .join(format!("{}.json", ref_file_name(branch_name))),
        serde_json::to_string_pretty(&json!({
            "type": "opened",
            "version": 1,
            "branch": {"name": branch_name, "opened_from_commit": commit_id},
            "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "path": open_dir}
        }))
        .unwrap(),
    )
    .unwrap();

    let branches = list_branches(&open_dir).unwrap();

    assert_eq!(
        branches,
        vec![Branch {
            name: branch_name.to_string(),
            head: commit_id.clone(),
            state: "open-clean".to_string()
        }]
    );

    let opened = list_opened_registries(&repo_root).unwrap();
    let local_list = local_list_data_json(&branches, opened);
    assert_eq!(local_list["type"], "codefire_local_list");
    assert_eq!(local_list["branches"][0]["name"], branch_name);
    assert_eq!(local_list["opened"][0]["branch"], branch_name);
    assert_eq!(
        local_list["opened"][0]["path"],
        open_dir.to_string_lossy().as_ref()
    );

    let (detail_repo, detail) = branch_show_data(&open_dir, None).unwrap();
    assert_eq!(detail_repo, repo_root);
    assert_eq!(detail["type"], "codefire_branch_detail");
    assert_eq!(detail["branch"]["name"], branch_name);
    assert_eq!(detail["branch"]["head"], commit_id);
    assert_eq!(detail["branch"]["state"], "open-clean");
    assert_eq!(detail["branch"]["sealed_validation"]["ok"], true);
    assert_eq!(detail["branch"]["opened"]["present"], true);
    assert_eq!(
        detail["branch"]["opened"]["open"]["open_instance_id"],
        "open_123"
    );

    let (_, explicit_detail) = branch_show_data(&detail_repo, Some(branch_name)).unwrap();
    assert_eq!(explicit_detail["branch"]["name"], branch_name);

    let unsupported = branch_unsupported_command_envelope("unknown", Some(&repo_root), 2);
    assert_eq!(unsupported["schema"], "codefire.command_result.v1");
    assert_eq!(unsupported["command"], "branch");
    assert_eq!(unsupported["ok"], false);
    assert_eq!(unsupported["repo"], repo_root.to_string_lossy().as_ref());
    assert_eq!(unsupported["data"]["type"], "codefire_branch_error");
    assert_eq!(unsupported["data"]["unsupported_subcommand"], "unknown");
    assert_eq!(
        unsupported["data"]["supported_subcommands"],
        json!(["list", "show"])
    );
    assert_eq!(
        unsupported["diagnostics"][0]["kind"],
        "unsupported_subcommand"
    );
    assert_eq!(
        unsupported["next_actions"][0]["command"],
        "codefire branch --help"
    );
    assert_eq!(
        unsupported["next_actions"][1]["command"],
        "codefire branch list --json"
    );
    assert_eq!(
        unsupported["next_actions"][2]["command"],
        "codefire branch show --json"
    );
}

#[test]
fn ref_file_name_matches_python_quote_safe_empty() {
    assert_eq!(
        ref_file_name("feature/a b%雪"),
        "feature%2Fa%20b%25%E9%9B%AA"
    );
}

#[test]
fn unix_timestamp_format_uses_utc_iso_seconds() {
    assert_eq!(
        codefire_util::format_unix_seconds_utc(0),
        "1970-01-01T00:00:00Z"
    );
    assert_eq!(
        codefire_util::format_unix_seconds_utc(951_782_400),
        "2000-02-29T00:00:00Z"
    );
}

#[test]
fn base64_decoder_handles_padding_and_rejects_invalid_input() {
    assert_eq!(decode_base64("aGVsbG8K").unwrap(), b"hello\n");
    assert_eq!(decode_base64("Zg==").unwrap(), b"f");
    assert!(decode_base64("Z===").is_err());
}

fn normalize_diff_labels(output: &str, left_branch: &str, right_branch: &str) -> String {
    output
        .lines()
        .map(|line| {
            normalize_diff_label_line(line, "--- ", left_branch, "<LEFT>")
                .or_else(|| normalize_diff_label_line(line, "+++ ", right_branch, "<RIGHT>"))
                .unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn normalize_diff_label_line(
    line: &str,
    marker: &str,
    branch: &str,
    replacement: &str,
) -> Option<String> {
    let prefix = format!("{marker}{branch}@CF-COMMIT-");
    if !line.starts_with(&prefix) {
        return None;
    }
    let slash = line[prefix.len()..].find('/')? + prefix.len();
    Some(format!("{marker}{replacement}{}", &line[slash..]))
}

fn diff_payload_lines(diff: &str) -> Vec<String> {
    diff.lines()
        .filter(|line| {
            (line.starts_with('+') || line.starts_with('-'))
                && !line.starts_with("+++")
                && !line.starts_with("---")
        })
        .map(str::to_string)
        .collect()
}

fn normalize_merge_dry_run(output: &str) -> String {
    output
        .lines()
        .map(|line| {
            if line.starts_with("  base: ") {
                "  base: <BASE>".to_string()
            } else if line.starts_with("  source: ") {
                "  source: <SOURCE>".to_string()
            } else if line.starts_with("  target: ") {
                "  target: <TARGET>".to_string()
            } else if line.starts_with("  target dir: ") {
                "  target dir: <TARGET_DIR>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn write_valid_commit(objects: &Path) -> String {
    let roots = write_required_roots(objects);
    let certificate = consistent_certificate();
    let commit = codefire_store::commit_payload(vec![], roots, certificate);
    codefire_store::store_object(objects, "commit", commit).unwrap()
}

fn write_generation_commit(objects: &Path, parents: Vec<String>, generation: u64) -> String {
    write_generation_commit_with_label(objects, parents, generation, "")
}

fn write_generation_commit_with_label(
    objects: &Path,
    parents: Vec<String>,
    generation: u64,
    label: &str,
) -> String {
    let mut commit = codefire_store::commit_payload(
        parents,
        write_required_roots(objects),
        consistent_certificate(),
    );
    commit["generation"] = json!(generation);
    commit["generation_test_label"] = json!(label);
    codefire_store::store_object(objects, "commit", commit).unwrap()
}

fn write_file_commit(objects: &Path, path: &str, contents: &str) -> String {
    write_file_commit_with_parents(objects, path, contents, vec![])
}

fn write_file_commit_with_parents(
    objects: &Path,
    path: &str,
    contents: &str,
    parents: Vec<String>,
) -> String {
    let blob = codefire_store::store_object(
        objects,
        "blob",
        json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(contents.as_bytes()),
        }),
    )
    .unwrap();
    let manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({
            "type": "content_manifest",
            "version": 1,
            "entries": [{"path": path, "kind": "file", "mode": "100644", "blob": blob}],
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(objects);
    roots.insert("content_manifest".to_string(), Value::String(manifest));
    codefire_store::store_object(
        objects,
        "commit",
        codefire_store::commit_payload(parents, roots, consistent_certificate()),
    )
    .unwrap()
}

fn write_file_bytes_commit(objects: &Path, path: &str, contents: &[u8]) -> String {
    write_file_bytes_commit_with_parents(objects, path, contents, vec![])
}

fn write_file_bytes_commit_with_parents(
    objects: &Path,
    path: &str,
    contents: &[u8],
    parents: Vec<String>,
) -> String {
    let blob = codefire_store::store_object(
        objects,
        "blob",
        json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(contents),
        }),
    )
    .unwrap();
    let manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({
            "type": "content_manifest",
            "version": 1,
            "entries": [{"path": path, "kind": "file", "mode": "100644", "blob": blob}],
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(objects);
    roots.insert("content_manifest".to_string(), Value::String(manifest));
    codefire_store::store_object(
        objects,
        "commit",
        codefire_store::commit_payload(parents, roots, consistent_certificate()),
    )
    .unwrap()
}

fn write_file_commit_with_atoms(
    objects: &Path,
    files: &[(&str, &str)],
    atoms: &[(&str, &str, &str, &str)],
    parents: Vec<String>,
) -> String {
    let entries = files
        .iter()
        .map(|(path, contents)| {
            let blob = codefire_store::store_object(
                objects,
                "blob",
                json!({
                    "type": "blob",
                    "version": 1,
                    "encoding": "base64",
                    "content": encode_base64(contents.as_bytes()),
                }),
            )
            .unwrap();
            json!({"path": path, "kind": "file", "mode": "100644", "blob": blob})
        })
        .collect::<Vec<_>>();
    let manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": entries}),
    )
    .unwrap();
    let atom_values = atoms
        .iter()
        .map(|(atom_id, kind, artifact_path, content_hash)| {
            json!({
                "atom_id": atom_id,
                "kind": kind,
                "artifact_path": artifact_path,
                "selector": {"type": "heading", "value": atom_id},
                "content_hash": content_hash,
            })
        })
        .collect::<Vec<_>>();
    let atom_index = codefire_store::store_object(
        objects,
        "atom_index",
        json!({
            "type": "atom_index",
            "version": 1,
            "atoms": atom_values,
            "duplicate_atom_ids": [],
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(objects);
    roots.insert("content_manifest".to_string(), Value::String(manifest));
    roots.insert("atom_index".to_string(), Value::String(atom_index));
    codefire_store::store_object(
        objects,
        "commit",
        codefire_store::commit_payload(parents, roots, consistent_certificate()),
    )
    .unwrap()
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(ALPHABET[(a >> 2) as usize] as char);
        encoded.push(ALPHABET[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            encoded.push(ALPHABET[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() == 3 {
            encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn write_required_roots(objects: &Path) -> Map<String, Value> {
    let content_manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": []}),
    )
    .unwrap();
    let atom_index = codefire_store::store_object(
        objects,
        "atom_index",
        json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
    )
    .unwrap();
    let trace_graph = codefire_store::store_object(
        objects,
        "trace_graph",
        json!({"type": "trace_graph", "version": 1, "links": []}),
    )
    .unwrap();
    let fire_delta = codefire_store::store_object(
        objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": []}),
    )
    .unwrap();
    let verification = codefire_store::store_object(
        objects,
        "verification",
        json!({"type": "verification", "version": 1, "checks": []}),
    )
    .unwrap();
    let policy = codefire_store::store_object(
        objects,
        "policy",
        json!({"type": "policy", "version": 1, "policy": {}}),
    )
    .unwrap();

    let mut roots = Map::new();
    roots.insert(
        "content_manifest".to_string(),
        Value::String(content_manifest),
    );
    roots.insert("atom_index".to_string(), Value::String(atom_index));
    roots.insert("trace_graph".to_string(), Value::String(trace_graph));
    roots.insert("fire_delta".to_string(), Value::String(fire_delta));
    roots.insert("verification".to_string(), Value::String(verification));
    roots.insert("policy".to_string(), Value::String(policy));
    roots
}

fn consistent_certificate() -> Map<String, Value> {
    let mut certificate = Map::new();
    certificate.insert(
        "result".to_string(),
        Value::String("consistent".to_string()),
    );
    certificate.insert("open_required_fires".to_string(), Value::Number(0.into()));
    certificate.insert("failed_checks".to_string(), Value::Number(0.into()));
    certificate.insert(
        "missing_required_links".to_string(),
        Value::Number(0.into()),
    );
    certificate.insert("stale_resolutions".to_string(), Value::Number(0.into()));
    certificate.insert("duplicate_atom_ids".to_string(), Value::Number(0.into()));
    certificate
}

fn active_state_path(open_dir: &Path) -> PathBuf {
    let marker = read_json(&open_dir.join(".codefire-open")).unwrap();
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"]).unwrap());
    let repo_root = repo_dir.parent().unwrap();
    let branch = required_string(&marker, &["branch", "name"]).unwrap();
    let registry = read_json(&opened_registry_path(repo_root, &branch)).unwrap();
    PathBuf::from(required_string(&registry, &["open", "active_state_path"]).unwrap())
}

fn read_active_fires(open_dir: &Path) -> Vec<codefire_core::Fire> {
    let path = active_state_path(open_dir).join("fires.json");
    if path.exists() {
        serde_json::from_value(read_json(&path).unwrap()).unwrap()
    } else {
        Vec::new()
    }
}
