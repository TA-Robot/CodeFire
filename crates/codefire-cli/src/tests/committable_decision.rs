use super::*;
use std::collections::BTreeMap;

#[test]
fn unsafe_committable_policy_fails_before_verify_or_commit_state_writes() {
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
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "version: 1\ncommit_policy:\n  require_no_required_fires: false\n  require_trace_completeness: false\n",
    )
    .unwrap();
    let before = snapshot_mutable_authority(&repo_root);

    let verify_error = run_verify(&open_dir).unwrap_err();
    assert!(verify_error
        .to_string()
        .contains("commit_policy.require_no_required_fires must be true"));
    assert_eq!(snapshot_mutable_authority(&repo_root), before);

    for dry_run in [true, false] {
        let error = run_commit(&CommitOptions {
            path: open_dir.clone(),
            message: "must remain blocked".to_string(),
            dry_run,
            full_output: false,
            json_output: true,
            lock: LockOptions::default(),
            idempotency_key: None,
            signer: None,
            key_id: None,
        })
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("commit_policy.require_no_required_fires must be true"));
        assert_eq!(snapshot_mutable_authority(&repo_root), before);
    }
}

fn snapshot_mutable_authority(repo_root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    let base = repo_root.join(".codefire");
    for relative in ["objects", "branches", "opened", "active", "idempotency"] {
        let path = base.join(relative);
        snapshot_tree(&base, &path, &mut snapshot);
    }
    snapshot
}

fn snapshot_tree(base: &Path, path: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
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
            snapshot_tree(base, &entry, snapshot);
        } else if metadata.is_file() {
            snapshot.insert(
                entry.strip_prefix(base).unwrap().to_path_buf(),
                fs::read(entry).unwrap(),
            );
        }
    }
}
