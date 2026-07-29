use super::*;
use std::collections::BTreeMap;

#[test]
fn commit_recovery_rejects_a_different_request_before_any_write() {
    for failpoint in [
        "after_content_manifest",
        "after_branch_publish_before_registry",
    ] {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join(format!("repo-{failpoint}"));
        let open_dir = temp.path().join(format!("open-{failpoint}"));
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
            open_dir.join("README.md"),
            format!("commit recovery request identity: {failpoint}\n"),
        )
        .unwrap();

        let original = commit_options(&open_dir, "original request", Some("original-key"));
        let active = active_state_path(&open_dir);
        set_commit_transaction_failure_for_test(&active, failpoint);
        let interrupted = run_commit(&original).unwrap_err();
        assert!(interrupted.to_string().contains(failpoint));

        let before = authority_snapshot(&repo_root, &open_dir);
        let different = commit_options(&open_dir, "different request", Some("different-key"));
        let error = run_commit(&different).unwrap_err();
        assert!(
            matches!(
                &error,
                CliError::OpenRecoveryPending(pending)
                    if pending.operation == "commit"
                        && pending.phase
                        == if failpoint == "after_content_manifest" {
                            "stored_content_manifest"
                            } else {
                                "publishing_branch"
                            }
            ),
            "unexpected recovery error at {failpoint}: {error:?}"
        );
        assert_eq!(
            authority_snapshot(&repo_root, &open_dir),
            before,
            "different request mutated pending commit authority at {failpoint}"
        );

        let recovered = run_commit(&original).unwrap();
        assert!(!recovered.commit_id.is_empty());
        assert!(!active.join("commit_transaction.json").exists());
    }
}

fn commit_options(path: &Path, message: &str, idempotency_key: Option<&str>) -> CommitOptions {
    CommitOptions {
        path: path.to_path_buf(),
        message: message.to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: idempotency_key.map(str::to_string),
        signer: None,
        key_id: None,
    }
}

fn authority_snapshot(repo_root: &Path, open_dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    let codefire = repo_root.join(".codefire");
    for relative in ["objects", "branches", "opened", "active", "idempotency"] {
        snapshot_tree(repo_root, &codefire.join(relative), &mut snapshot);
    }
    let marker = open_dir.join(".codefire-open");
    if marker.is_file() {
        snapshot.insert(
            PathBuf::from("open/.codefire-open"),
            fs::read(marker).unwrap(),
        );
    }
    snapshot
}

fn snapshot_tree(root: &Path, path: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
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
            snapshot_tree(root, &entry, snapshot);
        } else if metadata.is_file() {
            snapshot.insert(
                entry.strip_prefix(root).unwrap().to_path_buf(),
                fs::read(entry).unwrap(),
            );
        }
    }
}
