use super::*;
use crate::merge_resolution::MergeContinueOptions;
use std::collections::BTreeMap;
use tempfile::tempdir;

fn snapshot_authority(repo_root: &Path, include_active: bool) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    let codefire = repo_root.join(".codefire");
    for relative in ["objects", "branches", "opened"] {
        snapshot_authority_tree(&codefire, &codefire.join(relative), &mut snapshot);
    }
    if include_active {
        snapshot_authority_tree(&codefire, &codefire.join("active"), &mut snapshot);
    }
    snapshot
}

fn snapshot_authority_tree(base: &Path, path: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if !path.exists() {
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        let metadata = fs::symlink_metadata(&entry).unwrap();
        if metadata.is_dir() {
            snapshot_authority_tree(base, &entry, snapshot);
        } else if metadata.is_file() {
            snapshot.insert(
                entry.strip_prefix(base).unwrap().to_path_buf(),
                fs::read(entry).unwrap(),
            );
        }
    }
}

fn opened_base(repo_root: &Path, branch: &str) -> String {
    required_string(
        &read_json(&opened_registry_path(repo_root, branch)).unwrap(),
        &["open", "current_base_commit"],
    )
    .unwrap()
}

fn assert_blocked_commits_do_not_publish(
    repo_root: &Path,
    open_dir: &Path,
    expected_opened_base: &str,
    label: &str,
) {
    for dry_run in [true, false] {
        let before = snapshot_authority(repo_root, false);
        let blocked = run_commit(&CommitOptions {
            path: open_dir.to_path_buf(),
            message: format!("blocked {label}"),
            dry_run,
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
            signer: None,
            key_id: None,
        })
        .unwrap();

        assert!(blocked.blocked);
        assert_eq!(blocked.exit_code, ExitCode::MergeConflict);
        assert_eq!(blocked.exit_code.code(), 15);
        assert_eq!(snapshot_authority(repo_root, false), before);
        assert_eq!(opened_base(repo_root, "main"), expected_opened_base);
    }
}

fn assert_blocked_continue_does_not_mutate(
    repo_root: &Path,
    open_dir: &Path,
    accept_semantic_conflicts: bool,
) {
    for dry_run in [true, false] {
        let before = snapshot_authority(repo_root, true);
        let blocked = run_merge_continue(&MergeContinueOptions {
            path: open_dir.to_path_buf(),
            accept_semantic_conflicts,
            dry_run,
            json_output: true,
            lock: LockOptions::default(),
        })
        .unwrap();

        assert!(blocked.blocked);
        assert!(!blocked.applied);
        assert_eq!(blocked.exit_code(), ExitCode::MergeConflict);
        assert_eq!(blocked.exit_code().code(), 15);
        assert_eq!(snapshot_authority(repo_root, true), before);
    }
}

#[test]
fn unresolved_text_merge_conflict_blocks_verification_and_commit_preview() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "README.md", "base\n");
    let target_commit = write_file_commit_with_parents(
        &objects,
        "README.md",
        "target\n",
        vec![base_commit.clone()],
    );
    let source_commit =
        write_file_commit_with_parents(&objects, "README.md", "source\n", vec![base_commit]);
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": target_commit.clone(),
            "state": "closed",
            "created_at": "2026-07-12T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature",
            "head": source_commit,
            "state": "closed",
            "created_at": "2026-07-12T00:00:00Z"
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

    let merge = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    assert_eq!(merge.conflicts, vec!["README.md"]);

    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.unresolved_merge_conflicts.len(), 1);
    assert_eq!(
        verification.unresolved_merge_conflicts[0].kind,
        codefire_core::MergeConflictKind::Text
    );
    assert_eq!(
        verification_exit_code(&verification),
        ExitCode::MergeConflict
    );
    assert!(
        !codefire_core::committable_decision(&verification).is_committable(),
        "unresolved merge conflict was incorrectly considered committable"
    );
    assert_eq!(opened_base(&repo_root, "main"), target_commit);
    assert_blocked_commits_do_not_publish(&repo_root, &open_dir, &target_commit, "text merge");
    assert_blocked_continue_does_not_mutate(&repo_root, &open_dir, false);

    // A Markdown setext underline is legitimate content, not sufficient by
    // itself to prove that generated merge boundaries remain.
    fs::write(open_dir.join("README.md"), "resolved\n=======\n").unwrap();
    let metadata_still_blocking = run_verify(&open_dir).unwrap();
    assert_eq!(
        metadata_still_blocking.unresolved_merge_conflicts,
        verification.unresolved_merge_conflicts
    );
    assert_eq!(
        verification_exit_code(&metadata_still_blocking),
        ExitCode::MergeConflict
    );
    let state_path = active_state_path(&open_dir).join("state.json");
    let state_before_preview = read_json(&state_path).unwrap();
    let continue_preview = run_merge_continue(&MergeContinueOptions {
        path: open_dir.clone(),
        accept_semantic_conflicts: false,
        dry_run: true,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(!continue_preview.blocked);
    assert!(!continue_preview.applied);
    assert_eq!(read_json(&state_path).unwrap(), state_before_preview);

    let continued = run_merge_continue(&MergeContinueOptions {
        path: open_dir.clone(),
        accept_semantic_conflicts: false,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(continued.applied);
    let continued_state = read_json(&state_path).unwrap();
    assert!(continued_state["merge_conflicts"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(continued_state["pending_merge_parent"].is_string());
    assert_eq!(continued_state["merge_resolution"]["version"], 1);
    let replay = run_merge_continue(&MergeContinueOptions {
        path: open_dir.clone(),
        accept_semantic_conflicts: false,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(replay.already_resolved);
    assert!(!replay.applied);
    assert_eq!(read_json(&state_path).unwrap(), continued_state);

    let resolved_verification = run_verify(&open_dir).unwrap();
    assert!(resolved_verification.unresolved_merge_conflicts.is_empty());
    assert!(codefire_core::committable_decision(&resolved_verification).is_committable());
    let committed = run_commit(&CommitOptions {
        path: open_dir,
        message: "resolved merge".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();
    let commit = codefire_store::read_object(&objects, &committed.commit_id).unwrap();
    let parents = commit["parents"].as_array().unwrap();
    assert_eq!(parents.len(), 2);
    assert_eq!(parents[0], target_commit);
    assert_eq!(parents[1], merge.source_head);
}

#[test]
fn binary_merge_conflict_requires_sidecar_removal_before_continue() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_bytes_commit(&objects, "assets/model.bin", b"\0base");
    let target_commit = write_file_bytes_commit_with_parents(
        &objects,
        "assets/model.bin",
        b"\0target",
        vec![base_commit.clone()],
    );
    let source_commit = write_file_bytes_commit_with_parents(
        &objects,
        "assets/model.bin",
        b"\0source",
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({"type":"branch","version":1,"name":"main","head":target_commit.clone(),"state":"closed","created_at":"2026-07-12T00:00:00Z"}),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({"type":"branch","version":1,"name":"feature","head":source_commit,"state":"closed","created_at":"2026-07-12T00:00:00Z"}),
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
    merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();

    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.unresolved_merge_conflicts.len(), 1);
    assert_eq!(
        verification.unresolved_merge_conflicts[0].kind,
        codefire_core::MergeConflictKind::Binary
    );
    assert_eq!(
        verification_exit_code(&verification),
        ExitCode::MergeConflict
    );
    assert_blocked_commits_do_not_publish(&repo_root, &open_dir, &target_commit, "binary merge");
    assert_blocked_continue_does_not_mutate(&repo_root, &open_dir, false);
    assert_eq!(
        read_json(&active_state_path(&open_dir).join("state.json")).unwrap()
            ["binary_merge_conflicts"][0]["path"],
        "assets/model.bin"
    );

    fs::remove_file(
        open_dir
            .join(".codefire-conflicts/assets/model.bin")
            .join("target"),
    )
    .unwrap();
    // Either recorded side is sufficient to keep the transition blocked.
    assert_blocked_continue_does_not_mutate(&repo_root, &open_dir, false);
    fs::remove_file(
        open_dir
            .join(".codefire-conflicts/assets/model.bin")
            .join("source"),
    )
    .unwrap();
    let continued = run_merge_continue(&MergeContinueOptions {
        path: open_dir.clone(),
        accept_semantic_conflicts: false,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(continued.applied);
    let verification = run_verify(&open_dir).unwrap();
    assert!(verification.unresolved_merge_conflicts.is_empty());
    assert!(!codefire_core::committable_decision(&verification)
        .is_blocking(codefire_core::CommittableReason::UnresolvedMergeConflicts));
}

#[test]
fn semantic_merge_conflict_requires_explicit_acknowledgement() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let files = &[(
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 30\n",
    )];
    let base_commit = write_file_commit_with_atoms(
        &objects,
        files,
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:base",
        )],
        vec![],
    );
    let target_commit = write_file_commit_with_atoms(
        &objects,
        files,
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:target",
        )],
        vec![base_commit.clone()],
    );
    let source_commit = write_file_commit_with_atoms(
        &objects,
        files,
        &[(
            "REQ-session",
            "requirement",
            "docs/spec/session.md",
            "sha256:source",
        )],
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({"type":"branch","version":1,"name":"main","head":target_commit.clone(),"state":"closed","created_at":"2026-07-12T00:00:00Z"}),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({"type":"branch","version":1,"name":"feature","head":source_commit,"state":"closed","created_at":"2026-07-12T00:00:00Z"}),
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
    let merge = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    assert!(merge.conflicts.is_empty());
    assert_eq!(merge.semantic_conflicts.len(), 1);

    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(
        verification.unresolved_merge_conflicts[0].kind,
        codefire_core::MergeConflictKind::Semantic
    );
    assert_eq!(
        verification_exit_code(&verification),
        ExitCode::MergeConflict
    );
    assert_blocked_commits_do_not_publish(&repo_root, &open_dir, &target_commit, "semantic merge");
    assert_blocked_continue_does_not_mutate(&repo_root, &open_dir, false);

    let continued = run_merge_continue(&MergeContinueOptions {
        path: open_dir.clone(),
        accept_semantic_conflicts: true,
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
    })
    .unwrap();
    assert!(continued.applied);
    let verification = run_verify(&open_dir).unwrap();
    assert!(verification.unresolved_merge_conflicts.is_empty());
    assert!(!codefire_core::committable_decision(&verification)
        .is_blocking(codefire_core::CommittableReason::UnresolvedMergeConflicts));
}

#[test]
fn malformed_merge_conflict_path_fails_closed() {
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
    let state_path = active_state_path(&open_dir).join("state.json");
    let mut state = read_json(&state_path).unwrap();
    state["pending_merge_parent"] = Value::String("CF-COMMIT-invalid".to_string());
    state["merge_conflicts"] = json!(["../../outside"]);
    write_json_atomic(&state_path, &state).unwrap();
    let state_before = fs::read(&state_path).unwrap();

    let error = run_verify(&open_dir).unwrap_err();
    assert!(error.to_string().contains("manifest path escapes target"));
    assert_eq!(fs::read(&state_path).unwrap(), state_before);
    assert!(!active_state_path(&open_dir).join("scan.json").exists());
    assert!(!active_state_path(&open_dir)
        .join("verification.json")
        .exists());
    assert!(!temp.path().join("outside").exists());
}

#[test]
fn malformed_binary_side_path_fails_closed_without_authority_writes() {
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
    let state_path = active_state_path(&open_dir).join("state.json");
    let mut state = read_json(&state_path).unwrap();
    state["pending_merge_parent"] = Value::String("CF-COMMIT-invalid".to_string());
    state["merge_conflicts"] = json!(["assets/model.bin"]);
    state["binary_merge_conflicts"] = json!([{
        "path": "assets/model.bin",
        "target_path": ".codefire-conflicts/other.bin/target",
        "source_path": ".codefire-conflicts/assets/model.bin/source"
    }]);
    write_json_atomic(&state_path, &state).unwrap();
    let before = snapshot_authority(&repo_root, true);

    let verify_error = run_verify(&open_dir).unwrap_err();
    assert!(verify_error
        .to_string()
        .contains("does not match assets/model.bin"));
    assert_eq!(snapshot_authority(&repo_root, true), before);

    for dry_run in [true, false] {
        let continue_error = run_merge_continue(&MergeContinueOptions {
            path: open_dir.clone(),
            accept_semantic_conflicts: false,
            dry_run,
            json_output: true,
            lock: LockOptions::default(),
        })
        .unwrap_err();
        assert!(continue_error
            .to_string()
            .contains("does not match assets/model.bin"));
        assert_eq!(snapshot_authority(&repo_root, true), before);
    }
}

#[cfg(unix)]
#[test]
fn merge_continue_does_not_follow_a_symlinked_conflict_file() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let outside = temp.path().join("outside.md");
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
    fs::write(&outside, "<<<<<<< target\noutside\n>>>>>>> source\n").unwrap();
    symlink(&outside, open_dir.join("README.md")).unwrap();
    let state_path = active_state_path(&open_dir).join("state.json");
    let mut state = read_json(&state_path).unwrap();
    state["pending_merge_parent"] = Value::String("CF-COMMIT-invalid".to_string());
    state["merge_conflicts"] = json!(["README.md"]);
    write_json_atomic(&state_path, &state).unwrap();
    let before = snapshot_authority(&repo_root, true);
    let outside_before = fs::read(&outside).unwrap();

    for dry_run in [true, false] {
        let error = run_merge_continue(&MergeContinueOptions {
            path: open_dir.clone(),
            accept_semantic_conflicts: false,
            dry_run,
            json_output: true,
            lock: LockOptions::default(),
        })
        .unwrap_err();
        assert!(matches!(error, CliError::Io(_)));
        assert_eq!(snapshot_authority(&repo_root, true), before);
        assert_eq!(fs::read(&outside).unwrap(), outside_before);
    }
}

#[test]
fn parse_merge_continue_accepts_resolution_controls() {
    let options = crate::merge_resolution::parse_merge_continue_args(&[
        "--continue".to_string(),
        "--path".to_string(),
        "/tmp/open".to_string(),
        "--accept-semantic-conflicts".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--wait-lock".to_string(),
        "--lock-timeout".to_string(),
        "2s".to_string(),
    ])
    .unwrap();

    assert_eq!(options.path, PathBuf::from("/tmp/open"));
    assert!(options.accept_semantic_conflicts);
    assert!(options.dry_run);
    assert!(options.json_output);
    assert!(options.lock.wait);
    assert_eq!(options.lock.timeout_ms, Some(2_000));
}
