use super::*;
use serde_json::{json, Map};
use tempfile::tempdir;

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
    codefire_store::validate_sealed_commit(
        &repo_root.join(".codefire").join("objects"),
        &result.main_commit,
    )
    .unwrap();
    assert!(init_repo(&repo_root, false).is_err());
    init_repo(&repo_root, true).unwrap();
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
    let args = vec![
        "--algorithm".to_string(),
        "patience".to_string(),
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
    assert!(parsed.diff.rename_detection);
    assert!(parsed.diff.atom_diff);
    assert!(parsed.diff.trace_diff);
    assert!(parsed.diff.impact_diff);
    assert!(parsed.diff.json_output);

    let args = vec![
        "main".to_string(),
        "feature".to_string(),
        "--algorithm=histogram".to_string(),
    ];
    let parsed = parse_diff_args(&args).unwrap();
    assert_eq!(parsed.diff.algorithm, DiffAlgorithm::Histogram);
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
        &["/tmp/example".to_string(), "--json".to_string()],
        "status",
    )
    .unwrap();
    assert_eq!(status.path, PathBuf::from("/tmp/example"));
    assert!(status.json_output);

    let verify = parse_verify_args(&[
        "--details".to_string(),
        "--json".to_string(),
        "/tmp/example".to_string(),
    ])
    .unwrap();
    assert_eq!(verify.path, PathBuf::from("/tmp/example"));
    assert!(verify.details);
    assert!(verify.json_output);
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
fn context_pack_returns_atom_changed_and_fire_views() {
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
        json_output: true,
    })
    .unwrap();
    assert_eq!(atom_pack.data["type"], "codefire_context_pack");
    assert_eq!(atom_pack.data["selector"]["kind"], "atom");
    assert_eq!(atom_pack.data["atoms"].as_array().unwrap().len(), 2);
    assert_eq!(atom_pack.data["trace_links"].as_array().unwrap().len(), 1);

    let changed_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir.clone(),
        selector: context::ContextSelector::Changed,
        depth: 1,
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

    let fire_pack = context::build_context_pack(&context::ContextOptions {
        path: open_dir,
        selector: context::ContextSelector::Fire("FIRE-001".to_string()),
        depth: 1,
        json_output: true,
    })
    .unwrap();
    assert_eq!(fire_pack.data["selector"]["kind"], "fire");
    assert_eq!(fire_pack.data["fires"].as_array().unwrap().len(), 1);
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
    ];
    let parsed = parse_merge_args(&args).unwrap();
    assert_eq!(parsed.source_branch, "feature-session");
    assert_eq!(parsed.target_branch, "main");
    assert!(parsed.dry_run);
    assert!(parsed.json_output);
    assert!(parsed.lock.wait);
    assert_eq!(parsed.lock.timeout_ms, Some(500));
}

#[test]
fn parse_mutating_dry_run_json_args() {
    let commit = parse_commit_args(&[
        "--dry-run".to_string(),
        "--json".to_string(),
        "--path".to_string(),
        "/tmp/open".to_string(),
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
        "--path".to_string(),
        "/tmp/open".to_string(),
        "--resolution".to_string(),
        "addressed".to_string(),
        "--rationale".to_string(),
        "fixed".to_string(),
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
    assert!(extinguish.lock.wait);
    assert_eq!(extinguish.lock.timeout_ms, Some(2000));
    assert_eq!(extinguish.idempotency_key.as_deref(), Some("ext-key-1"));

    let open = parse_open_args(&[
        "main".to_string(),
        "/tmp/open".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--wait-lock".to_string(),
    ])
    .unwrap();
    assert_eq!(open.branch, "main");
    assert_eq!(open.path, PathBuf::from("/tmp/open"));
    assert!(open.dry_run);
    assert!(open.json_output);
    assert!(open.lock.wait);

    let clone = parse_clone_args(&[
        "main".to_string(),
        "feature".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--lock-timeout".to_string(),
        "1s".to_string(),
    ])
    .unwrap();
    assert_eq!(clone.source, "main");
    assert_eq!(clone.new_branch, "feature");
    assert!(clone.dry_run);
    assert!(clone.json_output);
    assert!(clone.lock.wait);
    assert_eq!(clone.lock.timeout_ms, Some(1000));
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
        },
    )
    .unwrap();
    let before = load_branch_record(&repo_root, "main").unwrap();
    let before_head = required_string(&before, &["head"]).unwrap();

    let result = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Dry run".to_string(),
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
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
        },
    )
    .unwrap();

    let first = run_commit(&CommitOptions {
        path: open_dir.clone(),
        message: "Seal once".to_string(),
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
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
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
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
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("seal-main-once".to_string()),
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

    let result = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        refresh: false,
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
    })
    .unwrap();
    let active = active_state_path(&open_dir);

    assert_eq!(result.display_id, "FIRE-001");
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

    let first = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
    })
    .unwrap();
    assert_eq!(first.display_id, "FIRE-001");

    let replay = run_extinguish(&ExtinguishOptions {
        path: open_dir.clone(),
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: "fixed".to_string(),
        evidence: String::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
    })
    .unwrap();
    assert_eq!(replay.display_id, first.display_id);
    assert_eq!(replay.plan["command"], "extinguish");

    let conflict = run_extinguish(&ExtinguishOptions {
        path: open_dir,
        fire_id: "FIRE-001".to_string(),
        resolution: "addressed".to_string(),
        rationale: "different rationale".to_string(),
        evidence: String::new(),
        refresh: false,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
        idempotency_key: Some("extinguish-fire-once".to_string()),
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
    fs::write(
        &batch_path,
        r#"version: 1
defaults:
  resolution: addressed
  evidence: "cargo test --workspace: passed"
fires:
  - id: FIRE-001
    rationale: "REQ to DES reviewed"
  - id: FIRE-002
    rationale: "DES to REQ reviewed"
"#,
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
    assert!(parsed.json_output);

    let dry_run = run_extinguish_batch(&parsed).unwrap();
    let active = active_state_path(&open_dir);
    assert_eq!(dry_run.item_count, 2);
    assert_eq!(dry_run.plan["type"], "codefire_operation_plan");
    assert_eq!(dry_run.plan["command"], "extinguish-batch");
    assert_eq!(dry_run.plan["dry_run"], true);
    assert!(!active.join("fires.json").exists());
    assert!(!active.join("resolutions.json").exists());

    let applied = run_extinguish_batch(&batch::BatchExtinguishOptions {
        path: open_dir.clone(),
        batch_path: parsed.batch_path,
        dry_run: false,
        json_output: false,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert_eq!(applied.item_count, 2);
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
fn parse_review_pack_args_accepts_base_output_and_algorithm() {
    let args = vec![
        "feature-session".to_string(),
        "--base".to_string(),
        "main".to_string(),
        "--output".to_string(),
        "review-pack.json".to_string(),
        "--algorithm=patience".to_string(),
        "--no-rename-detection".to_string(),
    ];
    let parsed = parse_review_pack_args(&args).unwrap();
    assert_eq!(parsed.review.source, "feature-session");
    assert_eq!(parsed.review.base.as_deref(), Some("main"));
    assert_eq!(parsed.review.algorithm, DiffAlgorithm::Patience);
    assert!(!parsed.review.rename_detection);
    assert_eq!(parsed.output, Some(PathBuf::from("review-pack.json")));
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
}

#[test]
fn parse_patch_args_accept_export_and_import_options() {
    let export = parse_patch_export_args(&[
        "feature-session".to_string(),
        "--base".to_string(),
        "main".to_string(),
        "--output".to_string(),
        "change.cfpatch.json".to_string(),
    ])
    .unwrap();
    assert_eq!(export.patch.source, "feature-session");
    assert_eq!(export.patch.base.as_deref(), Some("main"));
    assert_eq!(export.output, Some(PathBuf::from("change.cfpatch.json")));

    let import = parse_patch_import_args(&[
        "change.cfpatch.json".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .unwrap();
    assert_eq!(import.path, PathBuf::from("change.cfpatch.json"));
    assert!(import.dry_run);
    assert!(import.json_output);
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
        },
    )
    .unwrap();
    let patch = patch_export_with_options(
        Some(&repo_root),
        &PatchExportOptions {
            source: "feature-session".to_string(),
            base: Some("main".to_string()),
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
        },
    )
    .unwrap();
    assert_eq!(upload_dry_run.plan["type"], "codefire_operation_plan");
    assert_eq!(upload_dry_run.plan["command"], "upload");
    assert!(!remote_project_root.exists());

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
            dry_run: false,
            json_output: false,
        },
    )
    .unwrap();
    let branches = list_remote_branches(&project_url).unwrap();
    assert_eq!(branches[0].name, "main");
    assert_eq!(
        branches[0].head,
        load_branch_record(&repo_root, "main").unwrap()["head"]
    );

    clone_branch(
        &clone_repo,
        &CloneOptions {
            source: main_url.clone(),
            new_branch: "main-from-remote".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
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
    })
    .unwrap();
    assert!(mr.id.starts_with("MR-"));
    let listed = list_merge_requests(&project_url).unwrap();
    assert_eq!(listed[0].status, "open");
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
            state: "open-consistent".to_string(),
            base: commit_id,
            open_fires: 1
        }
    );
}

#[test]
fn branch_list_reads_repo_from_open_marker_and_validates_heads() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("worktree");
    let cf = repo_root.join(".codefire");
    fs::create_dir_all(cf.join("branches")).unwrap();
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

    let branches = list_branches(&open_dir).unwrap();

    assert_eq!(
        branches,
        vec![Branch {
            name: branch_name.to_string(),
            head: commit_id,
            state: "open-clean".to_string()
        }]
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
    assert_eq!(format_unix_seconds_utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(format_unix_seconds_utc(951_782_400), "2000-02-29T00:00:00Z");
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
