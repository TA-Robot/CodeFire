use super::*;

#[test]
fn file_remote_upload_and_clone_preserve_legacy_short_commit_identity() {
    let temp = tempdir().unwrap();
    let source_repo = temp.path().join("source");
    let clone_repo = temp.path().join("clone");
    let received_objects = temp.path().join("received-objects");
    let server = temp.path().join("server");
    init_repo(&source_repo, false).unwrap();
    init_repo(&clone_repo, false).unwrap();

    let source_objects = source_repo.join(".codefire/objects");
    let mut branch = load_branch_record(&source_repo, "main").unwrap();
    let current_head = required_string(&branch, &["head"]).unwrap();
    let commit_payload = codefire_store::read_object(&source_objects, &current_head).unwrap();
    let mut legacy_record = codefire_store::object_record("commit", commit_payload).unwrap();
    legacy_record.object_id = format!(
        "{}-{}",
        codefire_store::object_prefix("commit"),
        &legacy_record.hash[..12]
    );
    let legacy_head = legacy_record.object_id.clone();
    assert_eq!(legacy_head.rsplit_once('-').unwrap().1.len(), 12);
    codefire_store::store_object_record(&source_objects, legacy_record.clone()).unwrap();
    codefire_store::validate_sealed_commit(&source_objects, &legacy_head).unwrap();
    branch["head"] = Value::String(legacy_head.clone());
    branch["state"] = Value::String("closed".to_string());
    save_branch_record(&source_repo, &branch).unwrap();

    let remote_url = format!("cf://{}/org/app/main", server.display());
    let upload = upload_branch(
        &source_repo,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: remote_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(upload.head, legacy_head);

    let remote = parse_cf_url(&remote_url).unwrap();
    let remote_objects = remote_dirs(&remote.project_root).objects;
    let remote_record = codefire_store::read_object_record(&remote_objects, &legacy_head).unwrap();
    assert_eq!(remote_record, legacy_record);
    let remote_path = codefire_store::object_record_path(&remote_objects, &legacy_head).unwrap();
    let remote_bytes = fs::read(&remote_path).unwrap();

    upload_branch(
        &source_repo,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: remote_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(fs::read(&remote_path).unwrap(), remote_bytes);

    clone_branch(
        &clone_repo,
        &CloneOptions {
            source: remote_url,
            new_branch: "legacy-from-remote".to_string(),
            dry_run: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    let cloned = load_branch_record(&clone_repo, "legacy-from-remote").unwrap();
    assert_eq!(required_string(&cloned, &["head"]).unwrap(), legacy_head);
    codefire_store::validate_sealed_commit(&clone_repo.join(".codefire/objects"), &legacy_head)
        .unwrap();

    let transferred = serde_json::to_value(legacy_record.clone()).unwrap();
    write_object_records(&received_objects, [transferred.clone()]).unwrap();
    let received_path =
        codefire_store::object_record_path(&received_objects, &legacy_head).unwrap();
    let received_bytes = fs::read(&received_path).unwrap();
    write_object_records(&received_objects, [transferred]).unwrap();
    assert_eq!(fs::read(&received_path).unwrap(), received_bytes);
    assert_eq!(
        codefire_store::read_object_record(&received_objects, &legacy_head).unwrap(),
        legacy_record
    );
}
