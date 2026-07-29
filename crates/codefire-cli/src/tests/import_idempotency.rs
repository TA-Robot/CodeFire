use super::*;
use std::collections::BTreeMap;

#[test]
fn exact_import_replay_preserves_domain_state_bytes() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("project");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("README.md"), "import once\n").unwrap();
    let options = ImportOptions {
        path: target.clone(),
        branch: "main".to_string(),
        dry_run: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: Some("byte-stable-import".to_string()),
    };

    let first = import_existing_project(&options).unwrap();
    let before = domain_state_snapshot(&target);
    let replay = import_existing_project(&options).unwrap();
    let after = domain_state_snapshot(&target);

    assert_eq!(replay.repo_root, first.repo_root);
    assert_eq!(replay.open_dir, first.open_dir);
    assert_eq!(replay.branch, first.branch);
    assert_eq!(replay.base_commit, first.base_commit);
    assert_eq!(replay.initial_files, first.initial_files);
    assert_eq!(replay.initial_state, first.initial_state);
    assert_eq!(replay.plan["replayed"], true);
    assert_eq!(after, before);
}

fn domain_state_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    collect_domain_state(root, root, &mut snapshot);
    snapshot
}

fn collect_domain_state(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let mut entries = fs::read_dir(current)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if relative == Path::new(".codefire/locks/repo.lock.owner.json") {
            // Lock ownership diagnostics are expected to be refreshed for the
            // replaying process. They are not import result/domain state.
            continue;
        }
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            collect_domain_state(root, &path, snapshot);
        } else if file_type.is_file() {
            snapshot.insert(relative, fs::read(path).unwrap());
        }
    }
}
