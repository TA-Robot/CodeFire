use super::*;
use crate::doctor::DoctorSeverity;

#[test]
fn doctor_guides_legacy_semantic_profile_bridge_without_treating_it_as_proof() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("team's open workspace");
    let initialized = init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");

    let mut legacy_payload = codefire_store::read_object(&objects, &initialized.main_commit)
        .expect("initial commit payload");
    legacy_payload
        .as_object_mut()
        .expect("commit object")
        .remove("semantic_validation_profile");
    let legacy_commit = codefire_store::store_object(&objects, "commit", legacy_payload.clone())
        .expect("legacy commit");
    codefire_store::validate_sealed_commit_pinned(&objects, &legacy_commit)
        .expect("legacy commit remains readable");

    let branch_path = branch_record_path(&repo_root, "main");
    let mut branch = read_json(&branch_path).unwrap();
    branch["head"] = Value::String(legacy_commit.clone());
    write_json_atomic(&branch_path, &branch).unwrap();
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

    let legacy_report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert!(legacy_report.ok, "{:?}", legacy_report.issues);
    let branch_warning = legacy_report
        .issues
        .iter()
        .find(|issue| issue.kind == "legacy_branch_head_semantic_profile")
        .expect("legacy branch warning");
    assert_eq!(branch_warning.severity, DoctorSeverity::Warning);
    assert_eq!(branch_warning.path.as_deref(), Some(branch_path.as_path()));
    let open_warning = legacy_report
        .issues
        .iter()
        .find(|issue| issue.kind == "legacy_open_base_semantic_profile")
        .expect("legacy open-base warning");
    assert_eq!(open_warning.severity, DoctorSeverity::Warning);
    assert_eq!(open_warning.path.as_deref(), Some(open_dir.as_path()));

    let diagnostics = doctor_report_diagnostics_json(&legacy_report);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["kind"] == "legacy_open_base_semantic_profile"
            && diagnostic["category"] == "opened_registry"
            && diagnostic["blocking"] == false
            && diagnostic["repairable"] == true
            && diagnostic["path"] == open_dir.to_string_lossy().as_ref()
    }));
    let actions = doctor_report_next_actions(&legacy_report);
    assert_eq!(actions[0]["id"], "scan_legacy_open_base");
    let quoted_open_path = format!("'{}'", open_dir.to_string_lossy().replace('\'', "'\\''"));
    assert_eq!(
        actions[0]["command"],
        format!("codefire scan --path {quoted_open_path} --json")
    );
    assert!(actions
        .iter()
        .any(|action| action["id"] == "inspect_legacy_branch_profile"));

    let roots = legacy_payload["roots"]
        .as_object()
        .expect("legacy roots")
        .clone();
    let certificate = legacy_payload["certificate"]
        .as_object()
        .expect("legacy certificate")
        .clone();
    let bridge_commit = codefire_store::store_object(
        &objects,
        "commit",
        codefire_store::commit_payload(vec![legacy_commit], roots, certificate),
    )
    .expect("current-profile bridge commit");
    codefire_store::validate_sealed_commit_pinned(&objects, &bridge_commit)
        .expect("current bridge may retain a legacy ancestor");

    let mut branch = read_json(&branch_path).unwrap();
    branch["head"] = Value::String(bridge_commit.clone());
    write_json_atomic(&branch_path, &branch).unwrap();
    let registry_path = opened_registry_path(&repo_root, "main");
    let mut registry = read_json(&registry_path).unwrap();
    registry["open"]["current_base_commit"] = Value::String(bridge_commit.clone());
    write_json_atomic(&registry_path, &registry).unwrap();

    let bridged_report = run_doctor(&DoctorOptions {
        start: repo_root.clone(),
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert!(bridged_report.ok, "{:?}", bridged_report.issues);
    assert!(!bridged_report.issues.iter().any(|issue| matches!(
        issue.kind.as_str(),
        "legacy_branch_head_semantic_profile" | "legacy_open_base_semantic_profile"
    )));
    assert!(doctor_report_next_actions(&bridged_report).is_empty());

    let mut unknown_payload = codefire_store::read_object(&objects, &bridge_commit).unwrap();
    unknown_payload["semantic_validation_profile"] =
        Value::String("sealed-snapshot-future".to_string());
    let unknown_commit = codefire_store::store_object(&objects, "commit", unknown_payload)
        .expect("content-addressed unknown-profile fixture");
    let mut branch = read_json(&branch_path).unwrap();
    branch["head"] = Value::String(unknown_commit.clone());
    write_json_atomic(&branch_path, &branch).unwrap();
    let mut registry = read_json(&registry_path).unwrap();
    registry["open"]["current_base_commit"] = Value::String(unknown_commit);
    write_json_atomic(&registry_path, &registry).unwrap();

    let unknown_report = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert!(!unknown_report.ok);
    assert!(unknown_report
        .issues
        .iter()
        .any(|issue| issue.kind == "invalid_branch_head"));
    assert!(unknown_report
        .issues
        .iter()
        .any(|issue| issue.kind == "invalid_open_base_commit"));
    assert!(!unknown_report.issues.iter().any(|issue| matches!(
        issue.kind.as_str(),
        "legacy_branch_head_semantic_profile" | "legacy_open_base_semantic_profile"
    )));
}

#[cfg(unix)]
#[test]
fn doctor_active_inventory_rejects_symlinked_instance_without_following_it() {
    use std::os::unix::fs::symlink;

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
    let active = active_state_path(&open_dir);
    let moved = active.parent().unwrap().join("moved-active-instance");
    let outside = temp.path().join("outside-active");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("state.json"), b"{\"state\":\"foreign\"}\n").unwrap();
    let outside_before = fs::read(outside.join("state.json")).unwrap();
    fs::rename(&active, &moved).unwrap();
    symlink(&outside, &active).unwrap();

    let report = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: true,
        json_output: true,
    })
    .unwrap();

    assert!(report
        .issues
        .iter()
        .any(|issue| issue.kind == "invalid_active_state_dir"));
    assert!(!report.issues.iter().any(|issue| {
        issue.kind == "invalid_active_state_shape"
            && issue
                .path
                .as_deref()
                .is_some_and(|path| path.starts_with(&outside))
    }));
    let diagnostics = doctor_report_diagnostics_json(&report);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["kind"] == "invalid_active_state_dir" && diagnostic["category"] == "active_state"
    }));
    assert!(doctor_report_next_actions(&report)
        .iter()
        .any(|action| action["id"] == "reopen_branch_workspace"));
    assert_eq!(
        fs::read(outside.join("state.json")).unwrap(),
        outside_before
    );
}
