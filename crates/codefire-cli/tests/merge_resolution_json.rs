use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::tempdir;

fn codefire() -> &'static str {
    env!("CARGO_BIN_EXE_codefire-rs")
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(codefire())
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run CodeFire command")
}

fn assert_success(cwd: &Path, args: &[&str]) -> Output {
    let output = run(cwd, args);
    assert!(
        output.status.success(),
        "command failed: {args:?}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn json_document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout must contain one JSON document: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn setup_divergent_text_merge(root: &Path) -> (PathBuf, PathBuf, String, String) {
    let repo = root.join("repo");
    let main_open = root.join("main-open");
    let feature_open = root.join("feature-open");
    assert_success(root, &["init", repo.to_str().unwrap()]);
    assert_success(&repo, &["open", "main", main_open.to_str().unwrap()]);
    fs::write(main_open.join("README.md"), "base\n").unwrap();
    let base = json_document(&assert_success(
        &repo,
        &[
            "commit",
            "--path",
            main_open.to_str().unwrap(),
            "-m",
            "base",
            "--json",
        ],
    ))["data"]["commit_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!base.is_empty());
    assert_success(&repo, &["clone", "main", "feature"]);
    assert_success(&repo, &["open", "feature", feature_open.to_str().unwrap()]);
    fs::write(feature_open.join("README.md"), "source\n").unwrap();
    let source = json_document(&assert_success(
        &repo,
        &[
            "commit",
            "--path",
            feature_open.to_str().unwrap(),
            "-m",
            "source",
            "--json",
        ],
    ))["data"]["commit_id"]
        .as_str()
        .unwrap()
        .to_string();
    fs::write(main_open.join("README.md"), "target\n").unwrap();
    let target = json_document(&assert_success(
        &repo,
        &[
            "commit",
            "--path",
            main_open.to_str().unwrap(),
            "-m",
            "target",
            "--json",
        ],
    ))["data"]["commit_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_success(&repo, &["merge", "feature", "--into", "main"]);
    (repo, main_open, target, source)
}

fn snapshot_authority(repo: &Path, include_active: bool) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    let codefire = repo.join(".codefire");
    for relative in ["objects", "branches", "opened"] {
        snapshot_tree(&codefire, &codefire.join(relative), &mut snapshot);
    }
    if include_active {
        snapshot_tree(&codefire, &codefire.join("active"), &mut snapshot);
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

fn opened_base(repo: &Path, branch: &str) -> String {
    let opened: Value = serde_json::from_slice(
        &fs::read(repo.join(".codefire/opened").join(format!("{branch}.json"))).unwrap(),
    )
    .unwrap();
    opened["open"]["current_base_commit"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn merge_conflict_json_is_single_exit_15_document_until_continue() {
    let temp = tempdir().unwrap();
    let (repo, main_open, target, source) = setup_divergent_text_merge(temp.path());

    let human = run(
        &repo,
        &[
            "verify",
            "--path",
            main_open.to_str().unwrap(),
            "--details",
            "--blocking-only",
        ],
    );
    assert_eq!(human.status.code(), Some(15));
    let human_stdout = String::from_utf8(human.stdout).unwrap();
    assert!(human_stdout.contains("Verification failed."));
    assert!(human_stdout.contains("Blocking checks: unresolved merge conflicts"));
    assert!(human_stdout.contains("text: README.md"));

    let explain = assert_success(
        &repo,
        &[
            "explain",
            "verify-failure",
            "--path",
            main_open.to_str().unwrap(),
            "--json",
        ],
    );
    let explain_json = json_document(&explain);
    assert_eq!(
        explain_json["data"]["verification"]["decision"]["primary_reason"],
        "unresolved_merge_conflicts"
    );
    assert!(explain_json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "unresolved_merge_conflict"));
    assert!(explain_json["next_actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action["kind"] == "continue_merge"));

    let verify = run(
        &repo,
        &["verify", "--path", main_open.to_str().unwrap(), "--json"],
    );
    assert_eq!(verify.status.code(), Some(15));
    assert!(verify.stderr.is_empty());
    let verify_json = json_document(&verify);
    assert_eq!(verify_json["schema"], "codefire.command_result.v1");
    assert_eq!(verify_json["command"], "verify");
    assert_eq!(verify_json["ok"], false);
    assert_eq!(verify_json["exit_code"], 15);
    assert_eq!(
        verify_json["data"]["decision"]["primary_reason"],
        "unresolved_merge_conflicts"
    );
    assert_eq!(
        verify_json["diagnostics"][0]["kind"],
        "unresolved_merge_conflict"
    );
    assert!(verify_json["next_actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action["kind"] == "continue_merge"));

    let publication_before = snapshot_authority(&repo, false);
    assert_eq!(opened_base(&repo, "main"), target);
    let commit = run(
        &repo,
        &[
            "commit",
            "--path",
            main_open.to_str().unwrap(),
            "-m",
            "unsafe",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(commit.status.code(), Some(15));
    let commit_json = json_document(&commit);
    assert_eq!(commit_json["command"], "commit");
    assert_eq!(commit_json["ok"], false);
    assert_eq!(commit_json["data"]["blocked"], true);
    assert_eq!(snapshot_authority(&repo, false), publication_before);
    assert_eq!(opened_base(&repo, "main"), target);

    let live_commit = run(
        &repo,
        &[
            "commit",
            "--path",
            main_open.to_str().unwrap(),
            "-m",
            "unsafe live",
            "--json",
        ],
    );
    assert_eq!(live_commit.status.code(), Some(15));
    assert!(live_commit.stderr.is_empty());
    let live_commit_json = json_document(&live_commit);
    assert_eq!(live_commit_json["command"], "commit");
    assert_eq!(live_commit_json["exit_code"], 15);
    assert_eq!(live_commit_json["data"]["dry_run"], false);
    assert_eq!(live_commit_json["data"]["blocked"], true);
    assert_eq!(live_commit_json["data"]["applied"], false);
    assert_eq!(snapshot_authority(&repo, false), publication_before);
    assert_eq!(opened_base(&repo, "main"), target);

    let full_authority_before_continue = snapshot_authority(&repo, true);
    let blocked_continue = run(
        &repo,
        &[
            "merge",
            "--continue",
            "--path",
            main_open.to_str().unwrap(),
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(blocked_continue.status.code(), Some(15));
    assert!(blocked_continue.stderr.is_empty());
    let blocked_json = json_document(&blocked_continue);
    assert_eq!(blocked_json["command"], "merge");
    assert_eq!(
        blocked_json["data"]["type"],
        "codefire_merge_continue_result"
    );
    assert_eq!(blocked_json["data"]["blocked"], true);
    assert_eq!(blocked_json["data"]["applied"], false);
    assert_eq!(
        snapshot_authority(&repo, true),
        full_authority_before_continue
    );

    let live_blocked_continue = run(
        &repo,
        &[
            "merge",
            "--continue",
            "--path",
            main_open.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(live_blocked_continue.status.code(), Some(15));
    assert!(live_blocked_continue.stderr.is_empty());
    let live_blocked_json = json_document(&live_blocked_continue);
    assert_eq!(live_blocked_json["data"]["blocked"], true);
    assert_eq!(live_blocked_json["data"]["applied"], false);
    assert_eq!(
        snapshot_authority(&repo, true),
        full_authority_before_continue
    );

    fs::write(main_open.join("README.md"), "resolved\n").unwrap();
    let ready = assert_success(
        &repo,
        &[
            "merge",
            "--continue",
            "--path",
            main_open.to_str().unwrap(),
            "--dry-run",
            "--json",
        ],
    );
    let ready_json = json_document(&ready);
    assert_eq!(ready_json["ok"], true);
    assert_eq!(ready_json["data"]["applied"], false);

    let continued = assert_success(
        &repo,
        &[
            "merge",
            "--continue",
            "--path",
            main_open.to_str().unwrap(),
            "--json",
        ],
    );
    let continued_json = json_document(&continued);
    assert_eq!(continued_json["data"]["applied"], true);

    let resolved_verify = assert_success(
        &repo,
        &["verify", "--path", main_open.to_str().unwrap(), "--json"],
    );
    let resolved_json = json_document(&resolved_verify);
    assert_eq!(resolved_json["data"]["result"], "passed");
    assert!(resolved_json["data"]["unresolved_merge_conflicts"]
        .as_array()
        .unwrap()
        .is_empty());

    let committed = assert_success(
        &repo,
        &[
            "commit",
            "--path",
            main_open.to_str().unwrap(),
            "-m",
            "resolved merge",
            "--json",
        ],
    );
    let committed_json = json_document(&committed);
    assert_eq!(committed_json["ok"], true);
    let commit_id = committed_json["data"]["commit_id"].as_str().unwrap();
    let show = assert_success(&repo, &["show", commit_id, "--json"]);
    let show_json = json_document(&show);
    assert_eq!(show_json["data"]["show"]["parents"][0], target);
    assert_eq!(show_json["data"]["show"]["parents"][1], source);
}
