use super::*;

#[cfg(unix)]
fn copy_directory_tree(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            copy_directory_tree(&source_path, &target_path);
        } else if file_type.is_file() {
            fs::copy(source_path, target_path).unwrap();
        } else {
            panic!("unexpected non-regular object-store entry");
        }
    }
}

#[cfg(unix)]
#[test]
fn verify_carries_validated_store_token_and_rejects_equal_root_replacement_before_persist() {
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
    let objects = repo_root.join(".codefire").join("objects");
    let moved_objects = repo_root
        .join(".codefire")
        .join("objects-validated-original");
    let replacement_objects = temp.path().join("objects-equal-replacement");
    copy_directory_tree(&objects, &replacement_objects);
    let registry_path = opened_registry_path(&repo_root, "main");
    let registry_before = read_json(&registry_path).unwrap();
    let branch_before = load_branch_record(&repo_root, "main").unwrap();
    let active = active_state_path(&open_dir);
    let state_before = read_json(&active.join("state.json")).unwrap();
    set_validated_store_swap_before_verify_persist_for_test(
        &repo_root,
        &objects,
        &moved_objects,
        &replacement_objects,
    );

    let error = run_verify(&open_dir).unwrap_err();

    assert!(matches!(error, CliError::StoreContext { .. }), "{error}");
    assert!(error.to_string().contains("identity changed"), "{error}");
    assert!(!active.join("verification.json").exists());
    assert!(!active.join("state_transaction.json").exists());
    assert_eq!(read_json(&registry_path).unwrap(), registry_before);
    assert_eq!(
        load_branch_record(&repo_root, "main").unwrap(),
        branch_before
    );
    assert_eq!(read_json(&active.join("state.json")).unwrap(), state_before);
    assert!(moved_objects.is_dir());
    let head = required_string(&branch_before, &["head"]).unwrap();
    codefire_store::validate_sealed_commit(&objects, &head).unwrap();
}
