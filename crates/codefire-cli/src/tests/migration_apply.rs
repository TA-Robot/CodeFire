use super::*;
use crate::migration::{
    migration_report_data_json, migration_report_next_actions, parse_migrate_args, run_migrate,
    run_migrate_apply_with_locked_plan_hook, validate_migration_action_path_for_test, MigrateMode,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn migrate_options(repo: &Path, mode: &str) -> migration::MigrateOptions {
    parse_migrate_args(&[
        mode.to_string(),
        repo.to_string_lossy().into_owned(),
        "--full".to_string(),
        "--json".to_string(),
    ])
    .unwrap()
}

fn repository_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                visit(root, &path, output);
            } else if file_type.is_file() {
                output.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            } else if file_type.is_symlink() {
                output.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read_link(path)
                        .unwrap()
                        .as_os_str()
                        .as_encoded_bytes()
                        .to_vec(),
                );
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

#[test]
fn migrate_apply_parser_accepts_locked_full_surface_and_rejects_quick_apply() {
    let help = command_help_for_args(&[
        "migrate".to_string(),
        "apply".to_string(),
        "--help".to_string(),
    ])
    .unwrap();
    assert!(help.contains("codefire migrate apply"));
    assert!(help.contains("layout-only"));
    assert!(completion::bash_completion_script().contains("check dry-run apply"));
    assert!(completion::zsh_completion_script().contains("(check dry-run apply)"));

    let parsed = parse_migrate_args(&[
        "apply".to_string(),
        "--path=/tmp/example".to_string(),
        "--target-format=v0.6".to_string(),
        "--full".to_string(),
        "--json".to_string(),
        "--wait-lock".to_string(),
        "--lock-timeout=250ms".to_string(),
    ])
    .unwrap();
    assert_eq!(parsed.mode, MigrateMode::Apply);
    assert_eq!(parsed.start, PathBuf::from("/tmp/example"));
    assert_eq!(parsed.target_format, "v0.6");
    assert!(parsed.json_output);
    assert!(parsed.lock.wait);
    assert_eq!(parsed.lock.timeout_ms, Some(250));

    let quick = parse_migrate_args(&[
        "apply".to_string(),
        "/tmp/example".to_string(),
        "--quick".to_string(),
    ])
    .unwrap_err();
    assert!(quick
        .to_string()
        .contains("requires a full compatibility scan"));

    let check_lock = parse_migrate_args(&[
        "check".to_string(),
        "/tmp/example".to_string(),
        "--wait-lock".to_string(),
    ])
    .unwrap_err();
    assert!(check_lock
        .to_string()
        .contains("only supported by migrate apply"));
}

#[test]
fn apply_executes_only_locked_registered_plan_and_exact_replay_is_noop() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let codefire = repo.join(".codefire");
    fs::remove_dir_all(codefire.join("locks")).unwrap();
    fs::remove_dir_all(codefire.join("idempotency")).unwrap();
    fs::remove_dir_all(codefire.join("objects/artifact_refs")).unwrap();

    let before_dry_run = repository_bytes(&repo);
    let dry_run = run_migrate(&migrate_options(&repo, "dry-run")).unwrap();
    assert!(dry_run.compatible);
    assert!(dry_run.would_write);
    assert!(!dry_run.did_write);
    assert_eq!(repository_bytes(&repo), before_dry_run);

    let apply = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(apply.compatible, "{:?}", apply.blockers);
    assert!(apply.did_write);
    assert!(!apply.format_write_performed);
    assert!(codefire.join("locks").is_dir());
    assert!(codefire.join("idempotency").is_dir());
    assert!(codefire.join("objects/artifact_refs").is_dir());
    assert!(apply
        .applied_actions
        .iter()
        .any(|action| action.path.ends_with("locks")));
    assert!(apply
        .applied_actions
        .iter()
        .any(|action| action.path.ends_with("idempotency")));
    assert!(apply.remaining_actions.is_empty());
    assert!(migration_report_next_actions(&apply).is_empty());

    let replay = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(replay.compatible);
    assert!(!replay.would_write);
    assert!(!replay.did_write);
    assert!(replay.applied_actions.is_empty());
    assert!(replay.remaining_actions.is_empty());
    assert!(migration_report_next_actions(&replay).is_empty());
}

#[test]
fn migration_next_actions_follow_mode_and_current_postcondition() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo with space");
    init_repo(&repo, false).unwrap();
    fs::remove_dir_all(repo.join(".codefire/cache")).unwrap();

    let check = run_migrate(&migrate_options(&repo, "check")).unwrap();
    let check_actions = migration_report_next_actions(&check);
    assert_eq!(check_actions.len(), 1);
    assert_eq!(check_actions[0]["id"], "review_migration_plan");
    assert!(check_actions[0]["command"]
        .as_str()
        .unwrap()
        .contains("migrate dry-run"));

    let dry_run = run_migrate(&migrate_options(&repo, "dry-run")).unwrap();
    let dry_run_actions = migration_report_next_actions(&dry_run);
    assert_eq!(dry_run_actions.len(), 1);
    assert_eq!(dry_run_actions[0]["id"], "apply_migration_plan");
    assert!(dry_run_actions[0]["command"]
        .as_str()
        .unwrap()
        .contains("--path '/"));

    let apply = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(apply.did_write);
    assert!(!apply.planned_actions.is_empty());
    assert!(apply.remaining_actions.is_empty());
    assert!(migration_report_next_actions(&apply).is_empty());
}

#[test]
fn failed_apply_recovery_action_is_derived_from_remaining_actions() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    fs::remove_dir_all(repo.join(".codefire/idempotency")).unwrap();
    let mut branch = load_branch_record(&repo, "main").unwrap();
    branch["head"] = Value::String("CF-COMMIT-missing".to_string());
    save_branch_record(&repo, &branch).unwrap();

    let report = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(!report.compatible);
    assert!(!report.remaining_actions.is_empty());
    let actions = migration_report_next_actions(&report);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0]["id"], "retry_remaining_migration_actions");
    assert_eq!(
        actions[0]["target"]["remaining_actions"],
        report.remaining_actions.len()
    );
    assert_eq!(actions[0]["target"]["applied_actions"], 0);
    assert_eq!(actions[0]["target"]["blockers"], report.blockers.len());
}

#[test]
fn v06_apply_is_explicitly_layout_only_and_never_rewrites_repo_format() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let repo_json = repo.join(".codefire/repo.json");
    let original = fs::read(&repo_json).unwrap();
    fs::remove_dir_all(repo.join(".codefire/cache")).unwrap();

    let options = parse_migrate_args(&[
        "apply".to_string(),
        repo.to_string_lossy().into_owned(),
        "--target-format=v0.6".to_string(),
        "--full".to_string(),
    ])
    .unwrap();
    let report = run_migrate(&options).unwrap();
    let data = migration_report_data_json(&report);
    assert!(report.compatible);
    assert!(report.did_write);
    assert_eq!(fs::read(repo_json).unwrap(), original);
    assert_eq!(data["target_format"], "v0.6");
    assert_eq!(data["format_write_supported"], false);
    assert_eq!(data["format_write_performed"], false);
}

#[test]
fn blocker_returns_report_without_any_repository_write() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    fs::remove_dir_all(repo.join(".codefire/idempotency")).unwrap();
    let mut branch = load_branch_record(&repo, "main").unwrap();
    branch["head"] = Value::String("CF-COMMIT-missing".to_string());
    save_branch_record(&repo, &branch).unwrap();
    let before = repository_bytes(&repo);

    let report = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(!report.compatible);
    assert!(!report.did_write);
    assert!(report.applied_actions.is_empty());
    assert!(report
        .blockers
        .iter()
        .any(|issue| issue.kind == "invalid_branch_head"));
    assert_eq!(repository_bytes(&repo), before);
    assert!(!repo.join(".codefire/idempotency").exists());
}

#[test]
fn migration_continues_one_snapshot_after_invalid_branch_to_legacy_branch() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let initialized = init_repo(&repo, false).unwrap();
    let objects = repo.join(".codefire/objects");
    let mut legacy = codefire_store::read_object(&objects, &initialized.main_commit).unwrap();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("semantic_validation_profile");
    legacy["message"] = Value::String("valid legacy after corrupt branch".to_string());
    let legacy_head = codefire_store::store_object(&objects, "commit", legacy).unwrap();
    save_branch_record(
        &repo,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "z-legacy",
            "head": legacy_head,
            "state": "closed",
            "created_at": "2026-07-20T00:00:00Z"
        }),
    )
    .unwrap();
    let mut main = load_branch_record(&repo, "main").unwrap();
    main["head"] = Value::String("CF-COMMIT-missing".to_string());
    save_branch_record(&repo, &main).unwrap();

    let report = run_migrate(&migrate_options(&repo, "check")).unwrap();

    assert!(!report.compatible);
    assert_eq!(report.checked_branches, 1);
    assert_eq!(
        report
            .blockers
            .iter()
            .filter(|issue| issue.kind == "invalid_branch_head")
            .count(),
        1
    );
    assert!(report
        .warnings
        .iter()
        .any(|issue| issue.kind == "legacy_semantic_validation_profile"
            && issue
                .path
                .as_ref()
                .is_some_and(|path| path.ends_with("z-legacy.json"))));
}

#[test]
fn migration_reports_duplicate_invalid_head_for_each_branch_in_path_order() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let missing = "CF-COMMIT-missing";
    let mut main = load_branch_record(&repo, "main").unwrap();
    main["head"] = Value::String(missing.to_string());
    save_branch_record(&repo, &main).unwrap();
    save_branch_record(
        &repo,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "z-duplicate",
            "head": missing,
            "state": "closed",
            "created_at": "2026-07-20T00:00:00Z"
        }),
    )
    .unwrap();

    let report = run_migrate(&migrate_options(&repo, "check")).unwrap();

    let invalid_paths = report
        .blockers
        .iter()
        .filter(|issue| issue.kind == "invalid_branch_head")
        .map(|issue| {
            issue
                .path
                .as_ref()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(invalid_paths, ["main.json", "z-duplicate.json"]);
}

#[test]
fn check_and_dry_run_are_byte_for_byte_non_mutating() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    fs::remove_dir_all(repo.join(".codefire/remotes")).unwrap();
    let before = repository_bytes(&repo);

    let check = run_migrate(&migrate_options(&repo, "check")).unwrap();
    assert!(check.compatible);
    assert_eq!(repository_bytes(&repo), before);
    let dry_run = run_migrate(&migrate_options(&repo, "dry-run")).unwrap();
    assert!(dry_run.compatible);
    assert!(dry_run.would_write);
    assert_eq!(repository_bytes(&repo), before);
}

#[cfg(unix)]
#[test]
fn symlink_and_non_directory_targets_are_blockers_and_write_nothing_outside() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    init_repo(&repo, false).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::remove_dir_all(repo.join(".codefire/cache")).unwrap();
    symlink(&outside, repo.join(".codefire/cache")).unwrap();
    fs::remove_dir_all(repo.join(".codefire/idempotency")).unwrap();
    fs::write(repo.join(".codefire/idempotency"), b"foreign\n").unwrap();
    let before = repository_bytes(&repo);

    let report = run_migrate(&migrate_options(&repo, "apply")).unwrap();
    assert!(!report.compatible);
    assert!(!report.did_write);
    assert!(
        report
            .blockers
            .iter()
            .filter(|issue| issue.kind == "invalid_layout_directory")
            .count()
            >= 2
    );
    assert_eq!(repository_bytes(&repo), before);
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
}

#[test]
fn forged_outside_root_action_is_rejected_by_capability_subject() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let outside = temp.path().join("outside-idempotency");
    let error = validate_migration_action_path_for_test(&repo, Path::new("idempotency"), &outside)
        .unwrap_err();
    assert!(error.to_string().contains("escapes or disagrees"));
    assert!(!outside.exists());
}

#[test]
fn concurrent_locked_plan_change_fails_before_codefire_applies_any_action() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let remotes = repo.join(".codefire/remotes");
    let idempotency = repo.join(".codefire/idempotency");
    fs::remove_dir_all(&remotes).unwrap();
    fs::remove_dir_all(&idempotency).unwrap();

    let report = run_migrate_apply_with_locked_plan_hook(&migrate_options(&repo, "apply"), |_| {
        fs::create_dir(&remotes)?;
        Ok(())
    })
    .unwrap();
    assert!(!report.compatible);
    assert!(report
        .blockers
        .iter()
        .any(|issue| issue.kind == "migration_plan_changed"));
    assert!(!idempotency.exists());
    assert!(report.applied_actions.is_empty());
}

#[test]
fn apply_contends_on_the_shared_persistent_repository_lock() {
    use fs2::FileExt;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let remotes = repo.join(".codefire/remotes");
    fs::remove_dir_all(&remotes).unwrap();
    let lock_path = repo.join(".codefire/locks/repo.lock");
    let (file, _) = secure_worktree::open_lock_file_nofollow(&lock_path).unwrap();
    file.lock_exclusive().unwrap();

    let error = run_migrate(&migrate_options(&repo, "apply")).unwrap_err();
    assert!(matches!(error, CliError::RepositoryLockContention { .. }));
    assert!(!remotes.exists());
    FileExt::unlock(&file).unwrap();
}

#[cfg(unix)]
#[test]
fn replaced_repository_parent_is_never_adopted_as_apply_subject() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let moved = temp.path().join("repo-original");
    init_repo(&repo, false).unwrap();
    fs::remove_dir_all(repo.join(".codefire/remotes")).unwrap();

    let report = run_migrate_apply_with_locked_plan_hook(&migrate_options(&repo, "apply"), |_| {
        fs::rename(&repo, &moved)?;
        fs::create_dir(&repo)?;
        fs::create_dir(repo.join(".codefire"))?;
        Ok(())
    })
    .unwrap();
    assert!(!report.compatible);
    assert!(report
        .blockers
        .iter()
        .any(|issue| issue.kind == "migration_subject_changed"));
    assert!(!repo.join(".codefire/remotes").exists());
    assert!(!moved.join(".codefire/remotes").exists());
}

#[cfg(unix)]
#[test]
fn replaced_objects_parent_is_never_adopted_as_apply_subject() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let objects = repo.join(".codefire/objects");
    let moved = repo.join(".codefire/objects-original");
    fs::remove_dir_all(objects.join("artifact_refs")).unwrap();

    let report = run_migrate_apply_with_locked_plan_hook(&migrate_options(&repo, "apply"), |_| {
        fs::rename(&objects, &moved)?;
        fs::create_dir(&objects)?;
        Ok(())
    })
    .unwrap();
    assert!(!report.compatible);
    assert!(report
        .blockers
        .iter()
        .any(|issue| issue.kind == "migration_subject_changed"));
    assert!(!objects.join("artifact_refs").exists());
    assert!(!moved.join("artifact_refs").exists());
}

#[test]
fn migration_warns_only_when_the_validated_branch_head_itself_is_legacy() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo, false).unwrap();
    let objects = repo.join(".codefire/objects");
    let mut branch = load_branch_record(&repo, "main").unwrap();
    let current_head = required_string(&branch, &["head"]).unwrap();
    let mut legacy = codefire_store::read_object(&objects, &current_head).unwrap();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("semantic_validation_profile");
    legacy["message"] = Value::String("legacy profile fixture".to_string());
    let legacy_head = codefire_store::store_object(&objects, "commit", legacy.clone()).unwrap();
    codefire_store::validate_sealed_commit(&objects, &legacy_head).unwrap();
    branch["head"] = Value::String(legacy_head.clone());
    save_branch_record(&repo, &branch).unwrap();

    let legacy_report = run_migrate(&migrate_options(&repo, "check")).unwrap();
    assert!(legacy_report.compatible);
    assert!(legacy_report
        .warnings
        .iter()
        .any(|issue| issue.kind == "legacy_semantic_validation_profile"));

    let mut bridge = legacy;
    bridge["semantic_validation_profile"] =
        Value::String(codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE.to_string());
    bridge["parents"] = json!([legacy_head]);
    bridge["message"] = Value::String("current profile bridge".to_string());
    let bridge_head = codefire_store::store_object(&objects, "commit", bridge).unwrap();
    codefire_store::validate_sealed_commit(&objects, &bridge_head).unwrap();
    branch["head"] = Value::String(bridge_head);
    save_branch_record(&repo, &branch).unwrap();

    let bridge_report = run_migrate(&migrate_options(&repo, "check")).unwrap();
    assert!(bridge_report.compatible);
    assert!(!bridge_report
        .warnings
        .iter()
        .any(|issue| issue.kind == "legacy_semantic_validation_profile"));
}
