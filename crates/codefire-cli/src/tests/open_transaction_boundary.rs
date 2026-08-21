use super::*;
use crate::open_transaction::{
    open_transaction_path, set_foreign_open_marker_before_save_for_test,
    set_open_marker_swap_before_removal_for_test, set_open_transaction_failure_for_test,
};
#[cfg(unix)]
use crate::open_transaction::{
    set_open_target_ancestor_swap_after_validation_for_test,
    set_open_target_ancestor_swap_before_use_for_test,
    set_open_target_swap_after_fingerprint_for_test,
};

fn open_options(target: &Path) -> OpenOptions {
    OpenOptions {
        branch: "main".to_string(),
        path: target.to_path_buf(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    }
}

#[cfg(unix)]
#[test]
fn open_transaction_retains_target_capability_after_fingerprint_until_metadata_publication() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let moved = temp.path().join("main-open-moved");
    init_repo(&repo_root, false).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    let active_root = repo_root.join(".codefire").join("active");
    let active_count_before = fs::read_dir(&active_root).unwrap().count();
    set_open_target_swap_after_fingerprint_for_test(&repo_root, &target, &moved);
    let options = OpenOptions {
        branch: "main".to_string(),
        path: target.clone(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };

    let error = open_branch_from(&repo_root, &options).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("before opened-registry publication"),
        "{error}"
    );
    assert!(fs::symlink_metadata(&target)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(moved.join(".codefire-open").is_file());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(
        fs::read_dir(&active_root).unwrap().count(),
        active_count_before
    );
    let marker = read_json(&open_transaction_path(&repo_root)).unwrap();
    assert_eq!(marker["phase"], "target_published");

    let doctor = run_doctor(&DoctorOptions {
        start: repo_root,
        quick: true,
        json_output: true,
    })
    .unwrap();
    assert!(doctor
        .issues
        .iter()
        .any(|issue| issue.kind == "invalid_open_transaction"));
}

#[cfg(unix)]
#[test]
fn open_transaction_rejects_real_ancestor_replacement_before_staging() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let repo_root = workspace.join("repo");
    let ancestor = workspace.join("targets");
    let target = ancestor.join("nested").join("main-open");
    let moved_original = workspace.join("targets-original");
    let replacement = temp.path().join("replacement-targets");
    fs::create_dir_all(&ancestor).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(replacement.join("sentinel.txt"), b"replacement sentinel\n").unwrap();
    init_repo(&repo_root, false).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    let active_root = repo_root.join(".codefire").join("active");
    let active_count_before = fs::read_dir(&active_root).unwrap().count();
    set_open_target_ancestor_swap_before_use_for_test(
        &target,
        &ancestor,
        &moved_original,
        &replacement,
    );

    let error = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: target.clone(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("ancestor changed before staging"),
        "{error}"
    );
    assert_eq!(
        fs::read(ancestor.join("sentinel.txt")).unwrap(),
        b"replacement sentinel\n"
    );
    assert!(!ancestor.join("nested").exists());
    assert!(!moved_original.join("nested").exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(
        fs::read_dir(&active_root).unwrap().count(),
        active_count_before
    );
    assert!(!open_transaction_path(&repo_root).exists());
}

#[cfg(unix)]
#[test]
fn open_transaction_rejects_real_ancestor_replacement_after_validation_before_prepare() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let repo_root = workspace.join("repo");
    let ancestor = workspace.join("targets");
    let target = ancestor.join("nested").join("main-open");
    let moved_original = workspace.join("targets-original");
    let replacement = temp.path().join("replacement-targets");
    fs::create_dir_all(&ancestor).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(replacement.join("sentinel.txt"), b"replacement sentinel\n").unwrap();
    init_repo(&repo_root, false).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    set_open_target_ancestor_swap_after_validation_for_test(
        &target,
        &ancestor,
        &moved_original,
        &replacement,
    );

    let error = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: target,
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("identity changed"), "{error}");
    assert_eq!(
        fs::read(ancestor.join("sentinel.txt")).unwrap(),
        b"replacement sentinel\n"
    );
    assert_eq!(fs::read_dir(&ancestor).unwrap().count(), 1);
    assert!(!moved_original.join("nested").exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert!(!open_transaction_path(&repo_root).exists());
}

#[cfg(unix)]
#[test]
fn open_transaction_recovery_rejects_replaced_recorded_ancestor_without_writing_replacement() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let repo_root = workspace.join("repo");
    let ancestor = workspace.join("targets");
    let target = ancestor.join("main-open");
    let moved_original = workspace.join("targets-original");
    let replacement = temp.path().join("replacement-targets");
    fs::create_dir_all(&ancestor).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(replacement.join("sentinel.txt"), b"replacement sentinel\n").unwrap();
    init_repo(&repo_root, false).unwrap();
    let options = OpenOptions {
        branch: "main".to_string(),
        path: target.clone(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &options).unwrap_err();
    let marker_before = fs::read(open_transaction_path(&repo_root)).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    fs::rename(&ancestor, &moved_original).unwrap();
    fs::rename(&replacement, &ancestor).unwrap();

    let error = open_branch_from(&repo_root, &options).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("recorded target ancestor identity changed"),
        "{error}"
    );
    assert_eq!(
        fs::read(ancestor.join("sentinel.txt")).unwrap(),
        b"replacement sentinel\n"
    );
    assert_eq!(fs::read_dir(&ancestor).unwrap().count(), 1);
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(
        fs::read(open_transaction_path(&repo_root)).unwrap(),
        marker_before
    );
}

#[cfg(unix)]
#[test]
fn open_transaction_recovery_rejects_replaced_created_parent_without_writing_replacement() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let repo_root = workspace.join("repo");
    let ancestor = workspace.join("targets");
    let created_parent = ancestor.join("created-parent");
    let target = created_parent.join("main-open");
    let moved_parent = ancestor.join("created-parent-original");
    let replacement = temp.path().join("replacement-created-parent");
    fs::create_dir_all(&ancestor).unwrap();
    fs::create_dir_all(&replacement).unwrap();
    fs::write(replacement.join("sentinel.txt"), b"replacement sentinel\n").unwrap();
    init_repo(&repo_root, false).unwrap();
    let options = OpenOptions {
        branch: "main".to_string(),
        path: target,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    };
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &options).unwrap_err();
    let marker_before = fs::read(open_transaction_path(&repo_root)).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    fs::rename(&created_parent, &moved_parent).unwrap();
    fs::rename(&replacement, &created_parent).unwrap();

    let error = open_branch_from(&repo_root, &options).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("recorded staging-parent identity changed"),
        "{error}"
    );
    assert_eq!(
        fs::read(created_parent.join("sentinel.txt")).unwrap(),
        b"replacement sentinel\n"
    );
    assert_eq!(fs::read_dir(&created_parent).unwrap().count(), 1);
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(
        fs::read(open_transaction_path(&repo_root)).unwrap(),
        marker_before
    );
}

#[test]
fn open_transaction_initial_marker_publication_never_replaces_foreign_racer() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    let foreign = b"foreign marker bytes\n";
    set_foreign_open_marker_before_save_for_test(&repo_root, foreign);

    let error = open_branch_from(&repo_root, &open_options(&target)).unwrap_err();

    assert!(error
        .to_string()
        .contains("pending transaction already exists"));
    assert_eq!(
        fs::read(open_transaction_path(&repo_root)).unwrap(),
        foreign
    );
    assert!(!target.exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
}

#[test]
fn open_dry_run_reports_pending_transaction_without_mutation() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let options = open_options(&target);
    set_open_transaction_failure_for_test(&repo_root, "after_prepared");
    open_branch_from(&repo_root, &options).unwrap_err();
    let marker_before = fs::read(open_transaction_path(&repo_root)).unwrap();
    let mut dry_run = open_options(&target);
    dry_run.dry_run = true;

    let error = open_branch_from(&repo_root, &dry_run).unwrap_err();

    assert!(error.to_string().contains("exact live retry"), "{error}");
    assert_eq!(
        fs::read(open_transaction_path(&repo_root)).unwrap(),
        marker_before
    );
    assert!(!target.exists());
    assert!(!opened_registry_path(&repo_root, "main").exists());
}

#[test]
fn open_transaction_completed_archive_is_exact_or_create() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let options = open_options(&target);
    set_open_transaction_failure_for_test(&repo_root, "after_idempotency_written");
    open_branch_from(&repo_root, &options).unwrap_err();
    let marker = read_json(&open_transaction_path(&repo_root)).unwrap();
    assert_eq!(marker["phase"], "idempotency_written");
    let transaction_id = marker["transaction_id"].as_str().unwrap();
    let suffix = transaction_id.strip_prefix("open_tx_").unwrap();
    let active_state_file = PathBuf::from(marker["active_state"]["path"].as_str().unwrap());
    let archive_path = active_state_file
        .parent()
        .unwrap()
        .join(format!("open_transaction.completed.{suffix}.json"));
    let divergent = json!({"foreign": true});
    write_json_atomic(&archive_path, &divergent).unwrap();

    let error = open_branch_from(&repo_root, &options).unwrap_err();

    assert!(error.to_string().contains("divergent contents"), "{error}");
    assert_eq!(read_json(&archive_path).unwrap(), divergent);
    assert!(open_transaction_path(&repo_root).exists());
}

#[test]
fn open_transaction_marker_identity_swap_before_removal_is_preserved_and_rejected() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let target = temp.path().join("main-open");
    let moved_marker = repo_root
        .join(".codefire")
        .join("open_transaction.original.json");
    init_repo(&repo_root, false).unwrap();
    set_open_marker_swap_before_removal_for_test(&repo_root, &moved_marker);

    let error = open_branch_from(&repo_root, &open_options(&target)).unwrap_err();

    assert!(error.to_string().contains("identity changed"), "{error}");
    let canonical = open_transaction_path(&repo_root);
    assert!(canonical.is_file());
    assert!(moved_marker.is_file());
    assert_eq!(
        fs::read(&canonical).unwrap(),
        fs::read(&moved_marker).unwrap()
    );
    assert!(opened_registry_path(&repo_root, "main").is_file());
    assert_eq!(
        branch_state(&load_branch_record(&repo_root, "main").unwrap()),
        "open-clean"
    );
}
