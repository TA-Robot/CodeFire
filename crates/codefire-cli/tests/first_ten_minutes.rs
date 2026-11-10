//! The first ten minutes of a new user, driven only by what the CLI prints.
//!
//! init -> open -> write files -> scan -> extinguish -> verify -> commit, then a
//! change that fires again, and a policy change that makes the recorded
//! resolutions stale. Every fire id used here comes from a `next_actions`
//! entry of the previous command, never from an earlier remembered output.

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

const EXIT_STALE_RESOLUTION: i32 = 12;

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_codefire-rs"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run CodeFire command")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "{args:?}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn success(cwd: &Path, args: &[&str]) -> Output {
    let output = run(cwd, args);
    assert!(
        output.status.success(),
        "command failed: {}",
        describe(args, &output)
    );
    output
}

fn json(cwd: &Path, args: &[&str]) -> (Option<i32>, Value) {
    let output = run(cwd, args);
    let document = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!("stdout is not JSON ({error}): {}", describe(args, &output))
    });
    (output.status.code(), document)
}

fn actions<'a>(document: &'a Value, kind: &str) -> Vec<&'a Value> {
    document["next_actions"]
        .as_array()
        .expect("next_actions array")
        .iter()
        .filter(|action| action["kind"] == kind)
        .collect()
}

/// Turns an action's `argv_template` into arguments, filling named inputs.
/// The program name is dropped; the test binary path is used instead.
fn materialize(action: &Value, inputs: &[(&str, &str)]) -> Vec<String> {
    let template = action["execution"]["argv_template"]
        .as_array()
        .unwrap_or_else(|| panic!("action has no argv_template: {action}"));
    template
        .iter()
        .skip(1)
        .map(|part| match part.as_str() {
            Some(literal) => literal.to_string(),
            None => {
                let name = part["input"].as_str().expect("named input");
                inputs
                    .iter()
                    .find(|(input, _)| *input == name)
                    .unwrap_or_else(|| panic!("no value for input {name} in {action}"))
                    .1
                    .to_string()
            }
        })
        .collect()
}

/// Runs every action of `kind` that `command` offers, asking again until it
/// offers none. `next_actions` is a capped list, so one pass is not enough.
fn drain_actions(cwd: &Path, command: &[&str], kind: &str, rationale: &str) -> usize {
    let mut total = 0;
    loop {
        let (_, document) = json(cwd, command);
        let selected = actions(&document, kind);
        if selected.is_empty() {
            return total;
        }
        for action in &selected {
            let mut argv = materialize(action, &[("resolution", "changed")]);
            argv.extend(["--rationale".to_string(), rationale.to_string()]);
            let argv = argv.iter().map(String::as_str).collect::<Vec<_>>();
            success(cwd, &argv);
        }
        total += selected.len();
        assert!(total <= 100, "{kind} actions never ran out");
    }
}

fn write_project(open_dir: &Path, minutes: u32, check_command: &str) {
    for dir in ["docs/spec", "docs/design", "src", "tests"] {
        fs::create_dir_all(open_dir.join(dir)).unwrap();
    }
    fs::write(
        open_dir.join("docs/spec/auth.md"),
        format!(
            "## REQ-AUTH-001: Session expiration\n\nSessions expire after {minutes} minutes.\n"
        ),
    )
    .unwrap();
    fs::write(
        open_dir.join("docs/design/auth.md"),
        format!("## DES-AUTH-001: Session policy\n\nSessionPolicy returns {minutes} minutes.\n"),
    )
    .unwrap();
    fs::write(
        open_dir.join("src/auth.py"),
        format!(
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    def expires_after_minutes(self):\n        return {minutes}\n"
        ),
    )
    .unwrap();
    fs::write(
        open_dir.join("tests/test_auth.py"),
        format!(
            "from src.auth import SessionPolicy\n\n# cf-atom: TEST-session-expiration\ndef test_session_expiration():\n    assert SessionPolicy().expires_after_minutes() == {minutes}\n"
        ),
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.yaml"),
        "version: 1\nartifacts:\n  requirements:\n    - path: docs/spec/**/*.md\n      kind: requirement_document\n  designs:\n    - path: docs/design/**/*.md\n      kind: design_document\n  code:\n    - path: src/**/*.py\n      kind: python_code\n  tests:\n    - path: tests/**/*.py\n      kind: pytest_test\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "version: 1\nlinks:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n    type: refined_by\n  - from: DES-AUTH-001\n    to: CODE-SessionPolicy\n    type: implemented_by\n  - from: REQ-AUTH-001\n    to: TEST-session-expiration\n    type: verified_by\n",
    )
    .unwrap();
    write_policy(open_dir, check_command);
}

fn write_policy(open_dir: &Path, check_command: &str) {
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        format!(
            "version: 1\nverification:\n  required:\n    - id: files-present\n      command: \"{check_command}\"\n      cwd: \".\"\n"
        ),
    )
    .unwrap();
}

#[test]
fn new_user_reaches_sealed_commits_using_only_cli_output() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    let open_dir = root.path().join("main");
    let open = open_dir.to_str().unwrap();

    success(root.path(), &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", open]);

    // First commit: everything is new, so every trace link fires.
    write_project(&open_dir, 30, "test -f src/auth.py");
    let fired = drain_actions(
        &open_dir,
        &["scan", "--json"],
        "extinguish_fire",
        "initial import",
    );
    assert!(fired > 0, "initial import must open fires");
    success(&open_dir, &["verify"]);
    success(
        &open_dir,
        &["commit", "-m", "Initial consistent auth sample"],
    );

    // A requirement change fires again and is sealed the same way.
    write_project(&open_dir, 15, "test -f src/auth.py");
    let fired = drain_actions(
        &open_dir,
        &["scan", "--json"],
        "extinguish_fire",
        "expire after 15 minutes",
    );
    assert!(fired > 0, "changed requirement must open fires");

    // Changing the policy after extinguishing makes those resolutions stale
    // while no fire is open. The refusal itself must name what to refresh.
    write_policy(&open_dir, "test -f src/auth.py -a -f tests/test_auth.py");
    let (code, verify) = json(&open_dir, &["verify", "--json"]);
    assert_eq!(code, Some(EXIT_STALE_RESOLUTION), "{verify}");
    let stale = verify["data"]["stale_resolutions"]
        .as_array()
        .expect("stale_resolutions array");
    assert_eq!(stale.len(), fired, "{verify}");
    for item in stale {
        assert!(
            item["fire_display_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("FIRE-")),
            "stale resolution must name its fire: {item}"
        );
        assert!(item["source_atom_id"].is_string() && item["target_atom_id"].is_string());
    }
    let details = success_or_code(&open_dir, &["verify", "--details"], EXIT_STALE_RESOLUTION);
    let first_fire = stale[0]["fire_display_id"].as_str().unwrap();
    assert!(
        details.contains(first_fire) && details.contains("--refresh"),
        "text details must name the fire and the refresh command:\n{details}"
    );

    // The refusal names the fire without asking for a second command, on
    // every path a user may take from here.
    let plain = success_or_code(&open_dir, &["verify"], EXIT_STALE_RESOLUTION);
    assert!(plain.contains(first_fire), "{plain}");
    let blocked = success_or_code(
        &open_dir,
        &["commit", "-m", "too early"],
        EXIT_STALE_RESOLUTION,
    );
    assert!(
        blocked.contains(first_fire) && blocked.contains("Nothing was sealed"),
        "{blocked}"
    );
    let scan = String::from_utf8_lossy(&success(&open_dir, &["scan"]).stdout).into_owned();
    assert!(scan.contains("Stale resolutions:"), "{scan}");

    let refreshed = drain_actions(
        &open_dir,
        &["verify", "--json"],
        "refresh_resolution",
        "policy check updated",
    );
    assert_eq!(refreshed, fired);
    success(&open_dir, &["verify"]);
    success(
        &open_dir,
        &["commit", "-m", "Change session expiration to 15 minutes"],
    );

    let (code, status) = json(&open_dir, &["status", "--json"]);
    assert_eq!(code, Some(0), "{status}");
    assert_eq!(status["data"]["state"], "open-clean", "{status}");
}

fn success_or_code(cwd: &Path, args: &[&str], expected: i32) -> String {
    let output = run(cwd, args);
    assert_eq!(
        output.status.code(),
        Some(expected),
        "unexpected exit: {}",
        describe(args, &output)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn extinguish_help_documents_refresh() {
    let root = tempdir().unwrap();
    let output = run(root.path(), &["extinguish", "--help"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("[--refresh]"), "{text}");
}

#[test]
fn failed_check_details_show_exit_code_and_the_end_of_the_output() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    let open_dir = root.path().join("main");
    success(root.path(), &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", open_dir.to_str().unwrap()]);
    write_project(
        &open_dir,
        30,
        "echo first-line && echo cause-of-failure && exit 3",
    );
    drain_actions(
        &open_dir,
        &["scan", "--json"],
        "extinguish_fire",
        "initial import",
    );

    let output = run(&open_dir, &["verify", "--details"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "{text}");
    for expected in ["files-present:", "exit code 3", "| cause-of-failure"] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

#[test]
fn rejected_open_targets_say_where_an_open_directory_belongs() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    success(root.path(), &["init", repo.to_str().unwrap()]);

    let inside = run(
        &repo,
        &["open", "main", repo.join("main").to_str().unwrap()],
    );
    let text = String::from_utf8_lossy(&inside.stderr);
    assert!(!inside.status.success());
    let suggestion = root.path().canonicalize().unwrap().join("main");
    assert!(
        text.contains("outside the repository root") && text.contains(suggestion.to_str().unwrap()),
        "{text}"
    );

    let parent = run(&repo, &["open", "main", "../main"]);
    let text = String::from_utf8_lossy(&parent.stderr);
    assert!(!parent.status.success());
    assert!(text.contains("use an absolute path without '..'"), "{text}");

    // The suggested directory is accepted as given.
    success(&repo, &["open", "main", suggestion.to_str().unwrap()]);
}

#[test]
fn status_points_at_scan_when_files_changed_since_the_last_scan() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    let open_dir = root.path().join("main");
    success(root.path(), &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", open_dir.to_str().unwrap()]);

    let clean = String::from_utf8_lossy(&success(&open_dir, &["status"]).stdout).into_owned();
    assert!(!clean.contains("Unscanned changes"), "{clean}");

    write_project(&open_dir, 30, "test -f src/auth.py");
    let edited = String::from_utf8_lossy(&success(&open_dir, &["status"]).stdout).into_owned();
    assert!(
        edited.contains("Unscanned changes") && edited.contains("codefire scan"),
        "{edited}"
    );

    success(&open_dir, &["scan"]);
    let scanned = String::from_utf8_lossy(&success(&open_dir, &["status"]).stdout).into_owned();
    assert!(!scanned.contains("Unscanned changes"), "{scanned}");
}

#[test]
fn batch_template_keeps_the_rationale_given_on_the_command_line() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    let open_dir = root.path().join("main");
    success(root.path(), &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", open_dir.to_str().unwrap()]);
    write_project(&open_dir, 30, "test -f src/auth.py");

    let batch = root.path().join("batch.json");
    success(
        &open_dir,
        &[
            "extinguish",
            "--batch-template",
            "--rationale",
            "initial import",
            "--output",
            batch.to_str().unwrap(),
        ],
    );
    let template: Value = serde_json::from_slice(&fs::read(&batch).unwrap()).unwrap();
    assert_eq!(template["defaults"]["rationale"], "initial import");
    let fires = template["fires"].as_array().unwrap();
    assert!(!fires.is_empty());
    assert!(
        fires.iter().all(|fire| fire.get("rationale").is_none()),
        "per-fire placeholders must not override the given rationale: {template}"
    );

    success(
        &open_dir,
        &["extinguish", "--batch", batch.to_str().unwrap()],
    );
    success(&open_dir, &["verify"]);
}

#[test]
fn help_and_scan_teach_the_daily_loop() {
    let root = tempdir().unwrap();
    let repo = root.path().join("repo");
    let open_dir = root.path().join("main");
    let help = String::from_utf8_lossy(&success(root.path(), &["--help"]).stdout).into_owned();
    for step in [
        "codefire scan",
        "codefire extinguish",
        "codefire verify",
        "codefire commit -m",
    ] {
        assert!(help.contains(step), "help must name {step}:\n{help}");
    }
    assert!(help.contains("A fire is"), "{help}");

    success(root.path(), &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", open_dir.to_str().unwrap()]);
    write_project(&open_dir, 30, "test -f src/auth.py");
    let scan = String::from_utf8_lossy(&success(&open_dir, &["scan"]).stdout).into_owned();
    assert!(
        scan.contains("Next:") && scan.contains("codefire extinguish"),
        "{scan}"
    );
}

/// Seals `notes.txt` with the given lines on `main`, clones `feature`, and
/// returns (repo, main open dir, feature open dir).
fn two_branches(
    root: &Path,
    lines: &[&str],
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let repo = root.join("repo");
    let main_dir = root.join("main");
    let feature_dir = root.join("feature");
    success(root, &["init", repo.to_str().unwrap()]);
    success(&repo, &["open", "main", main_dir.to_str().unwrap()]);
    fs::write(main_dir.join("notes.txt"), lines.join("\n") + "\n").unwrap();
    success(&main_dir, &["commit", "-m", "base"]);
    success(&repo, &["clone", "main", "feature"]);
    success(&repo, &["open", "feature", feature_dir.to_str().unwrap()]);
    (repo, main_dir, feature_dir)
}

fn replace_line(dir: &Path, from: &str, to: &str) {
    let path = dir.join("notes.txt");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from));
    fs::write(path, text.replace(from, to)).unwrap();
}

#[test]
fn merge_combines_changes_to_different_lines_of_one_file() {
    let root = tempdir().unwrap();
    let (repo, main_dir, feature_dir) =
        two_branches(root.path(), &["one", "two", "three", "four", "five", "six"]);
    replace_line(&feature_dir, "two", "two from feature");
    success(&feature_dir, &["commit", "-m", "feature edit"]);
    replace_line(&main_dir, "five", "five from main");
    success(&main_dir, &["commit", "-m", "main edit"]);

    let merge = success(&repo, &["merge", "feature", "--into", "main"]);
    let report = String::from_utf8_lossy(&merge.stdout);
    assert!(!report.contains("conflict"), "{report}");
    assert_eq!(
        fs::read_to_string(main_dir.join("notes.txt")).unwrap(),
        "one\ntwo from feature\nthree\nfour\nfive from main\nsix\n"
    );
    success(&main_dir, &["verify"]);
    success(&main_dir, &["commit", "-m", "merge feature"]);
}

#[test]
fn merge_marks_only_the_region_both_branches_changed() {
    let root = tempdir().unwrap();
    let (repo, main_dir, feature_dir) =
        two_branches(root.path(), &["one", "two", "three", "four", "five", "six"]);
    replace_line(&feature_dir, "two", "two from feature");
    replace_line(&feature_dir, "six", "six from feature");
    success(&feature_dir, &["commit", "-m", "feature edit"]);
    replace_line(&main_dir, "two", "two from main");
    success(&main_dir, &["commit", "-m", "main edit"]);

    let merge = run(&repo, &["merge", "feature", "--into", "main"]);
    let report = String::from_utf8_lossy(&merge.stdout);
    assert!(report.contains("conflict: notes.txt"), "{report}");
    assert_eq!(
        fs::read_to_string(main_dir.join("notes.txt")).unwrap(),
        "one\n<<<<<<< target\ntwo from main\n=======\ntwo from feature\n>>>>>>> source\nthree\nfour\nfive\nsix from feature\n"
    );
}

#[test]
fn branch_workflow_is_discoverable_from_command_output() {
    let root = tempdir().unwrap();
    let (repo, main_dir, feature_dir) = two_branches(root.path(), &["one", "two", "three"]);
    success(&repo, &["close", "feature"]);
    assert!(!feature_dir.exists());

    let help = String::from_utf8_lossy(&success(&repo, &["branch", "--help"]).stdout).into_owned();
    assert!(help.contains("codefire clone"), "{help}");

    // clone names the open command, and that exact command works.
    let clone =
        String::from_utf8_lossy(&success(&repo, &["clone", "main", "topic"]).stdout).into_owned();
    let next = clone
        .lines()
        .find_map(|line| line.strip_prefix("Next: codefire "))
        .unwrap_or_else(|| panic!("clone must say what to run next:\n{clone}"));
    let argv = next.split_whitespace().take(3).collect::<Vec<_>>();
    assert_eq!(&argv[..2], ["open", "topic"], "{clone}");
    success(&repo, &argv);

    // open without a directory explains itself instead of printing bare usage.
    let bare = run(&repo, &["open", "feature"]);
    let text = String::from_utf8_lossy(&bare.stderr);
    assert!(!bare.status.success());
    assert!(text.contains("codefire open feature "), "{text}");

    // A conflicting merge says what to do next, and so does status.
    replace_line(&root.path().join("topic"), "two", "two from topic");
    success(&root.path().join("topic"), &["commit", "-m", "topic edit"]);
    replace_line(&main_dir, "two", "two from main");
    success(&main_dir, &["commit", "-m", "main edit"]);
    let merge = String::from_utf8_lossy(&run(&repo, &["merge", "topic", "--into", "main"]).stdout)
        .into_owned();
    assert!(
        merge.contains("codefire merge --continue") && merge.contains("marker lines"),
        "{merge}"
    );
    let status = String::from_utf8_lossy(&success(&main_dir, &["status"]).stdout).into_owned();
    assert!(status.contains("Merge in progress"), "{status}");
    let blocked = run(&main_dir, &["merge", "--continue"]);
    let text = String::from_utf8_lossy(&blocked.stdout);
    assert!(!blocked.status.success());
    assert!(text.contains("delete the"), "{text}");

    fs::write(main_dir.join("notes.txt"), "one\ntwo merged\nthree\n").unwrap();
    success(&main_dir, &["merge", "--continue"]);
    success(&main_dir, &["verify"]);
    success(&main_dir, &["commit", "-m", "merge topic"]);
    let status = String::from_utf8_lossy(&success(&main_dir, &["status"]).stdout).into_owned();
    assert!(!status.contains("Merge in progress"), "{status}");
}
