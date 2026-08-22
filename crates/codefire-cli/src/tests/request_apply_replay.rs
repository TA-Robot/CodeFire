use super::*;

#[test]
fn file_backed_request_apply_replay_and_applied_list_status() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "base\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "feature\n",
        vec![main_commit.clone()],
    );
    for (name, head) in [("main", &main_commit), ("feature", &feature_commit)] {
        save_branch_record(
            &repo_root,
            &json!({
                "type": "branch",
                "version": 1,
                "name": name,
                "head": head,
                "state": "closed",
                "created_at": "2026-08-21T00:00:00Z"
            }),
        )
        .unwrap();
    }
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature");
    for (branch, remote_url) in [("main", &main_url), ("feature", &feature_url)] {
        upload_branch(
            &repo_root,
            &UploadOptions {
                branch: branch.to_string(),
                remote_url: remote_url.clone(),
                dry_run: false,
                json_output: true,
                idempotency_key: None,
                request_key_id: None,
                lock: LockOptions::default(),
            },
        )
        .unwrap();
    }

    let merge = request_merge(&RequestMergeOptions {
        source_url: feature_url,
        target_url: main_url.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(list_merge_requests(&project_url).unwrap()[0].status, "open");

    review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "ready".to_string(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "approved"
    );

    let apply_options = RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: merge.id.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: Some("apply-once".to_string()),
        request_key_id: None,
        lock: LockOptions::default(),
    };
    let live = apply_merge_request(&apply_options).unwrap();
    assert_eq!(live.head, feature_commit);
    assert_eq!(live.target_branch, "main");
    assert_eq!(live.plan["command"], "request-apply");
    assert_eq!(live.plan["would_apply"], true);
    assert!(live.plan.get("replayed").is_none());
    assert_eq!(
        live.plan["operations"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0),
        5
    );
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "applied"
    );

    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;
    let dirs = remote_dirs(&remote_project_root);
    let mr_path = dirs.merge_requests.join(format!("{}.json", merge.id));
    let branch_path = crate::remote::remote_branch_path(&remote_project_root, "main");
    let generation_path = remote_project_root.join("gc_state.json");
    let receipt_path = fs::read_dir(dirs.idempotency.join("v2").join("remote_request_apply"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let persisted_mr = fs::read(&mr_path).unwrap();
    let persisted_branch = fs::read(&branch_path).unwrap();
    let persisted_generation = fs::read(&generation_path).unwrap();
    let persisted_receipt = fs::read(&receipt_path).unwrap();
    let persisted_mr_json = read_json(&mr_path).unwrap();
    assert_eq!(persisted_mr_json["status"], "applied");
    assert_eq!(persisted_mr_json["applied_head"], feature_commit);

    let replay = apply_merge_request(&apply_options).unwrap();
    assert_eq!(replay.head, live.head);
    assert_eq!(replay.target_branch, live.target_branch);
    assert_eq!(replay.plan["type"], "codefire_operation_plan");
    assert_eq!(replay.plan["command"], "request-apply");
    assert_eq!(replay.plan["replayed"], true);
    assert_eq!(replay.plan["would_apply"], false);
    assert_eq!(replay.plan["dry_run"], false);
    assert_eq!(replay.plan["transport"], "file");
    assert_eq!(replay.plan["head"], live.head);
    assert_eq!(replay.plan["target_branch"], "main");
    assert_eq!(replay.plan["merge_request_id"], merge.id);
    assert_eq!(replay.plan["operations"], json!([]));
    assert_eq!(replay.plan["next_actions"], json!([]));
    assert_ne!(replay.plan, live.plan);

    assert_eq!(fs::read(&mr_path).unwrap(), persisted_mr);
    assert_eq!(fs::read(&branch_path).unwrap(), persisted_branch);
    assert_eq!(fs::read(&generation_path).unwrap(), persisted_generation);
    assert_eq!(fs::read(&receipt_path).unwrap(), persisted_receipt);
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "applied"
    );
}

#[test]
fn file_backed_request_list_keeps_stale_projection_for_open_mr() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "base\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "feature\n",
        vec![main_commit.clone()],
    );
    let advanced_main = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "advanced\n",
        vec![main_commit.clone()],
    );
    for (name, head) in [
        ("main", &main_commit),
        ("feature", &feature_commit),
        ("main-advanced", &advanced_main),
    ] {
        save_branch_record(
            &repo_root,
            &json!({
                "type": "branch",
                "version": 1,
                "name": name,
                "head": head,
                "state": "closed",
                "created_at": "2026-08-21T00:00:00Z"
            }),
        )
        .unwrap();
    }
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature");
    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature".to_string(),
            remote_url: feature_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    let merge = request_merge(&RequestMergeOptions {
        source_url: feature_url,
        target_url: main_url.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(list_merge_requests(&project_url).unwrap()[0].status, "open");

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main-advanced".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "stale"
    );
    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;
    let mr_path = remote_dirs(&remote_project_root)
        .merge_requests
        .join(format!("{}.json", merge.id));
    assert_eq!(read_json(&mr_path).unwrap()["status"], "open");
}

#[test]
fn file_backed_request_list_projects_stale_for_drifted_rejected_mr() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "base\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "feature\n",
        vec![main_commit.clone()],
    );
    let advanced_main = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "advanced\n",
        vec![main_commit.clone()],
    );
    for (name, head) in [
        ("main", &main_commit),
        ("feature", &feature_commit),
        ("main-advanced", &advanced_main),
    ] {
        save_branch_record(
            &repo_root,
            &json!({
                "type": "branch",
                "version": 1,
                "name": name,
                "head": head,
                "state": "closed",
                "created_at": "2026-08-21T00:00:00Z"
            }),
        )
        .unwrap();
    }
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature");
    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature".to_string(),
            remote_url: feature_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    let merge = request_merge(&RequestMergeOptions {
        source_url: feature_url,
        target_url: main_url.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "reject".to_string(),
        comment: "not yet".to_string(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "rejected"
    );

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main-advanced".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: true,
            idempotency_key: None,
            request_key_id: None,
            lock: LockOptions::default(),
        },
    )
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "stale"
    );
    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;
    let mr_path = remote_dirs(&remote_project_root)
        .merge_requests
        .join(format!("{}.json", merge.id));
    assert_eq!(read_json(&mr_path).unwrap()["status"], "rejected");
}

#[test]
fn file_backed_request_apply_pending_recovery_is_not_receipt_replay() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "base\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "feature\n",
        vec![main_commit.clone()],
    );
    for (name, head) in [("main", &main_commit), ("feature", &feature_commit)] {
        save_branch_record(
            &repo_root,
            &json!({
                "type": "branch",
                "version": 1,
                "name": name,
                "head": head,
                "state": "closed",
                "created_at": "2026-08-21T00:00:00Z"
            }),
        )
        .unwrap();
    }
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature");
    for (branch, remote_url) in [("main", &main_url), ("feature", &feature_url)] {
        upload_branch(
            &repo_root,
            &UploadOptions {
                branch: branch.to_string(),
                remote_url: remote_url.clone(),
                dry_run: false,
                json_output: true,
                idempotency_key: None,
                request_key_id: None,
                lock: LockOptions::default(),
            },
        )
        .unwrap();
    }
    let merge = request_merge(&RequestMergeOptions {
        source_url: feature_url,
        target_url: main_url.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();
    review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: merge.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "ready".to_string(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    })
    .unwrap();

    let apply_options = RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: merge.id.clone(),
        dry_run: false,
        json_output: true,
        idempotency_key: None,
        request_key_id: None,
        lock: LockOptions::default(),
    };
    crate::remote::inject_publication_fail_after_prepare_once();
    let interrupted = apply_merge_request(&apply_options).unwrap_err();
    assert!(interrupted
        .to_string()
        .contains("injected publication failure after pending prepare"));

    let remote_project_root = parse_cf_url(&main_url).unwrap().project_root;
    let pending_path = remote_project_root.join("pending_transaction.json");
    assert!(pending_path.is_file());
    let branch_path = crate::remote::remote_branch_path(&remote_project_root, "main");
    let mr_path = remote_dirs(&remote_project_root)
        .merge_requests
        .join(format!("{}.json", merge.id));
    assert_eq!(
        required_string(&read_json(&branch_path).unwrap(), &["head"]).unwrap(),
        main_commit
    );
    assert_eq!(read_json(&mr_path).unwrap()["status"], "approved");
    assert!(!remote_dirs(&remote_project_root)
        .idempotency
        .join("v2")
        .join("remote_request_apply")
        .exists());

    let recovered = apply_merge_request(&apply_options).unwrap();
    assert_eq!(recovered.head, feature_commit);
    assert_eq!(recovered.target_branch, "main");
    assert!(recovered.plan.get("replayed").is_none());
    assert_eq!(recovered.plan["would_apply"], true);
    assert_eq!(
        recovered.plan["operations"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0),
        5
    );
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "applied"
    );
    assert!(!pending_path.is_file());
    assert_eq!(
        required_string(&read_json(&branch_path).unwrap(), &["head"]).unwrap(),
        feature_commit
    );
    assert!(!remote_dirs(&remote_project_root)
        .idempotency
        .join("v2")
        .join("remote_request_apply")
        .exists());
}
