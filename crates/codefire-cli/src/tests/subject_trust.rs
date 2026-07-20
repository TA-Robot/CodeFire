use super::*;

#[test]
fn evidence_capture_redacts_secrets_and_records_a_subject() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let options = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--cwd".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf 'API_TOKEN=alpha Authorization: Bearer bravo'".to_string(),
    ])
    .unwrap();

    let result = run_evidence_add(&options).unwrap();
    let evidence = codefire_store::read_object(
        &repo_root.join(".codefire").join("objects"),
        &result.evidence_id,
    )
    .unwrap();
    let serialized = serde_json::to_string(&evidence).unwrap();
    assert!(!serialized.contains("alpha"));
    assert!(!serialized.contains("bravo"));
    assert_eq!(
        evidence["command"]["stdout"],
        "API_TOKEN=<redacted> Authorization: Bearer <redacted>"
    );
    assert_eq!(evidence["command"]["cwd"], ".");
    assert_eq!(
        evidence["command"]["environment_policy"],
        "minimal_allowlist"
    );
    assert_eq!(evidence["command"]["phase"], "evidence_capture");
    assert_eq!(evidence["command"]["portability"], "portable_redacted");
    assert!(evidence["command"]["input_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(evidence["command"]["environment_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(evidence["command"]["subject"]["subject_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(result
        .diagnostics
        .iter()
        .any(|item| item["kind"] == "evidence_secret_redacted"));
}

#[test]
fn evidence_external_cwd_and_inherited_secret_environment_stay_out_of_the_object() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let outside = temp.path().join("private-worktree");
    init_repo(&repo_root, false).unwrap();
    fs::create_dir(&outside).unwrap();
    let options = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--cwd".to_string(),
        outside.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "printf '%s|%s' \"$PWD\" \"${OPENAI_API_KEY-unset}\"".to_string(),
    ])
    .unwrap();

    let result = run_evidence_add(&options).unwrap();
    let evidence = codefire_store::read_object(
        &repo_root.join(".codefire").join("objects"),
        &result.evidence_id,
    )
    .unwrap();
    let serialized = serde_json::to_string(&evidence).unwrap();
    assert!(!serialized.contains(&outside.to_string_lossy().to_string()));
    if let Ok(secret) = std::env::var("OPENAI_API_KEY") {
        if !secret.is_empty() {
            assert!(!serialized.contains(&secret));
        }
    }
    assert_eq!(evidence["command"]["cwd"], "<redacted>");
    assert_eq!(evidence["command"]["cwd_kind"], "external_redacted");
    assert_eq!(evidence["command"]["portability"], "local_only_redacted");
    assert_eq!(evidence["command"]["stdout"], "<redacted-path>|unset");
}

#[test]
fn evidence_capture_streams_large_output_with_bounded_storage() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let options = parse_evidence_add_args(&[
        "--path".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--cwd".to_string(),
        repo_root.to_string_lossy().into_owned(),
        "--from-command".to_string(),
        "i=0; while [ $i -lt 50000 ]; do printf x; i=$((i+1)); done".to_string(),
        "--max-output-bytes=31".to_string(),
    ])
    .unwrap();

    let result = run_evidence_add(&options).unwrap();
    let evidence = codefire_store::read_object(
        &repo_root.join(".codefire").join("objects"),
        &result.evidence_id,
    )
    .unwrap();
    assert_eq!(evidence["command"]["stdout_bytes_seen"], 50_000);
    assert_eq!(evidence["command"]["stdout_bytes_stored"], 31);
    assert_eq!(evidence["command"]["stdout"].as_str().unwrap().len(), 31);
    assert_eq!(evidence["command"]["stdout_truncated"], true);
}

#[test]
fn verification_rejects_a_subject_changed_by_a_passing_command() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\nverification:\n  - id: mutate\n    command: printf changed > generated.txt\n    cwd: .\n",
    )
    .unwrap();

    let verification = run_verify(&open_dir).unwrap();

    assert_eq!(verification.result, "failed");
    assert_eq!(verification.failed_checks.len(), 1);
    assert!(verification.failed_checks[0]
        .output
        .contains("verification subject changed"));
    assert_eq!(verification.execution.len(), 1);
    assert!(!verification.execution[0].subject_unchanged);
    assert!(!verification.execution[0].success);
    assert_eq!(verification.execution[0].cwd, ".");
    assert_eq!(verification.execution[0].phase, "verification_command");
    assert!(verification.execution[0]
        .input_digest
        .starts_with("sha256:"));
    assert!(verification.execution[0]
        .environment_digest
        .starts_with("sha256:"));
    assert!(verification.subject.is_some());
    assert!(verification
        .subject
        .as_ref()
        .unwrap()
        .worktree_digest
        .as_deref()
        .unwrap()
        .starts_with("sha256:"));

    let data = automation::verification_data_json(&verification);
    assert_eq!(data["profile"], "local_verification_v1");
    assert_eq!(data["phase"], "post_execution");
    assert_eq!(
        data["subject"]["subject_digest"],
        verification.subject.as_ref().unwrap().subject_digest
    );
    assert_eq!(data["execution"][0]["phase"], "verification_command");
}

#[test]
fn commit_rejects_a_post_verification_worktree_change_before_domain_writes() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "verified bytes\n").unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let objects_before = non_branch_object_store_file_set(&objects);
    let branch_before =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    set_commit_worktree_mutation_for_test(
        &open_dir,
        "after_verify",
        Path::new("README.md"),
        b"changed after verification\n",
    );

    let error = run_commit(&commit_options(&open_dir, "Reject stale subject")).unwrap_err();

    assert!(error
        .to_string()
        .contains("assessment subject no longer matches the worktree after verification"));
    assert_eq!(non_branch_object_store_file_set(&objects), objects_before);
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        branch_before
    );
    assert!(!active_state_path(&open_dir)
        .join("commit_transaction.json")
        .exists());
}

#[test]
fn commit_rejects_a_manifest_planning_race_before_immutable_storage() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "planned bytes\n").unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let objects_before = non_branch_object_store_file_set(&objects);
    let branch_before =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    set_commit_worktree_mutation_for_test(
        &open_dir,
        "after_manifest_plan",
        Path::new("README.md"),
        b"changed during manifest planning\n",
    );

    let error = run_commit(&commit_options(&open_dir, "Reject manifest race")).unwrap_err();

    assert!(error
        .to_string()
        .contains("assessment subject no longer matches the worktree after manifest planning"));
    assert_eq!(non_branch_object_store_file_set(&objects), objects_before);
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        branch_before
    );
    let transaction =
        read_json(&active_state_path(&open_dir).join("commit_transaction.json")).unwrap();
    assert_eq!(transaction["phase"], "started");
    assert_eq!(transaction["objects"], json!([]));
}

#[test]
fn commit_rechecks_the_subject_immediately_before_branch_publication() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "candidate bytes\n").unwrap();
    let branch_before =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    let registry_before = required_string(
        &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
        &["open", "current_base_commit"],
    )
    .unwrap();
    set_commit_worktree_mutation_for_test(
        &open_dir,
        "before_branch_publish",
        Path::new("README.md"),
        b"changed before branch publication\n",
    );

    let error = run_commit(&commit_options(&open_dir, "Reject publication race")).unwrap_err();

    assert!(error
        .to_string()
        .contains("assessment subject no longer matches the worktree before branch publication"));
    assert_eq!(
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap(),
        branch_before
    );
    assert_eq!(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "current_base_commit"]
        )
        .unwrap(),
        registry_before
    );
    let transaction =
        read_json(&active_state_path(&open_dir).join("commit_transaction.json")).unwrap();
    assert_eq!(transaction["phase"], "publishing_branch");
    assert!(transaction["objects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|object| object["state"] == "stored"));
}

#[test]
fn verification_rejects_persistable_secret_literals_before_active_publication() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
    let before = directory_files(&active);

    for policy in [
        "verification:\n  - id: inline-secret\n    command: printf API_TOKEN=alpha\n",
        "verification:\n  - id: env-secret\n    command: true\n    env: API_TOKEN=alpha\n",
    ] {
        fs::write(open_dir.join("codefire.policy.yaml"), policy).unwrap();
        let error = run_verify(&open_dir).unwrap_err();
        assert!(error.to_string().contains("secret-like"));
        assert!(!error.to_string().contains("alpha"));
        assert_eq!(directory_files(&active), before);
    }
}

#[cfg(unix)]
#[test]
fn verification_cwd_escape_is_rejected_before_active_publication() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
    let outside = temp.path().join("outside");
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
    fs::create_dir(&outside).unwrap();
    symlink(&outside, open_dir.join("escape")).unwrap();
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "verification:\n  - id: escape\n    command: true\n    cwd: escape\n",
    )
    .unwrap();
    let active = active_state_path(&open_dir);
    let before = directory_files(&active);

    let error = run_verify(&open_dir).unwrap_err();

    assert!(error
        .to_string()
        .contains("verification cwd escapes the open directory"));
    assert_eq!(directory_files(&active), before);
}

#[test]
fn context_and_explain_share_the_same_live_subject() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    let context = build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Branch,
        depth: 1,
        limit: 10,
        json_output: true,
    })
    .unwrap();
    let explain = run_explain(&explain::ExplainOptions {
        path: open_dir,
        target: explain::ExplainTarget::VerifyFailure,
        json_output: true,
        depth: 1,
        large_threshold_bytes: 1_048_576,
    })
    .unwrap();

    assert_eq!(
        context.data["subject"]["subject_digest"],
        explain.data["subject"]["subject_digest"]
    );
    assert_eq!(context.data["subject"]["trust_kind"], "local_unisolated");
}

#[test]
fn sealed_commit_certificate_preserves_the_verified_subject_and_phase() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
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
        open_dir.join("codefire.policy.yaml"),
        "commit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "subject-bound commit\n").unwrap();
    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "passed");
    run_commit(&CommitOptions {
        path: open_dir,
        message: "Seal subject".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();

    let branch = load_branch_record(&repo_root, "main").unwrap();
    let head = required_string(&branch, &["head"]).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit = codefire_store::read_object(&objects, &head).unwrap();
    let verification_id = required_string(&commit, &["roots", "verification"]).unwrap();
    let sealed_verification = codefire_store::read_object(&objects, &verification_id).unwrap();

    assert_eq!(commit["certificate"]["profile"], "local_verification_v1");
    assert_eq!(commit["certificate"]["phase"], "pre_commit");
    assert_eq!(
        commit["certificate"]["subject"],
        sealed_verification["subject"]
    );
    assert_eq!(
        commit["certificate"]["subject"]["subject_digest"],
        verification.subject.unwrap().subject_digest
    );
    codefire_store::validate_sealed_commit(&objects, &head).unwrap();
}

fn directory_files(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().unwrap().is_file())
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn commit_options(path: &Path, message: &str) -> CommitOptions {
    CommitOptions {
        path: path.to_path_buf(),
        message: message.to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    }
}

fn non_branch_object_store_file_set(objects: &Path) -> BTreeSet<String> {
    object_store_file_set(objects)
        .into_iter()
        .filter(|path| !path.starts_with("branches/"))
        .collect()
}
