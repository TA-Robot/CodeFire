use crate::command_registry::COMMAND_NAMES;
use crate::completion::{bash_completion_script, help_text, zsh_completion_script};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[test]
fn help_and_completion_scripts_cover_rust_default_cli() {
    let help = help_text();
    assert!(help.contains("usage: codefire <command> [args]"));
    assert!(help.contains("codefire completion <bash|zsh>"));
    assert!(help.contains("codefire doctor [path] [--quick] [--json]"));

    let bash = bash_completion_script();
    assert!(bash.contains("complete -F _codefire_complete codefire"));
    assert!(bash.contains("request-apply"));
    assert!(bash.contains("--details --blocking-only --json --metrics"));
    assert!(bash.contains("--quick --full --json"));
    assert!(bash.contains("--tls-cert --tls-key --tls-client-ca"));
    assert!(!bash.contains("token-hash"));

    let zsh = zsh_completion_script();
    assert!(zsh.contains("#compdef codefire"));
    assert!(zsh.contains("completion\\:completion"));
    assert!(zsh.contains("--details[show failed verification diagnostic details]"));
    assert!(zsh.contains("--quick[skip full object store integrity scan]"));
    assert!(zsh.contains("--tls-client-ca[required client CA PEM]"));
    assert!(zsh.contains("--request-key-id[remote request signing key id]"));
    assert!(!zsh.contains("token-hash"));
}

#[test]
fn known_limitations_python_fallback_commands_stay_out_of_rust_cli() {
    let known_limitations = include_str!("../../../../docs/development/known-limitations.md");
    for required_policy_line in [
        "Python fallback freeze policy:",
        "New mainline features must land in Rust first.",
        "Python fallback changes are limited to compatibility fixtures, critical security/bug fixes, installer fallback behavior, and narrowly-scoped Python/Rust parity tests.",
        "Python fallback must not gain a new user-facing command, flag, output schema, remote behavior, or repository format unless the Rust CLI already owns that behavior and the temporary fallback delta is documented here.",
        "Golden compatibility tests should cover only repository/object/remote formats that Python-created projects still need for migration or incident response.",
    ] {
        assert!(
            known_limitations.contains(required_policy_line),
            "known-limitations.md is missing Python fallback freeze policy line: {required_policy_line}"
        );
    }

    let documented_fallback = documented_python_fallback_commands();
    assert_eq!(
        documented_fallback,
        ["close", "discard", "gc", "token-hash"]
            .into_iter()
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>()
    );

    let rust_commands = COMMAND_NAMES
        .iter()
        .map(|command| (*command).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let help_commands = help_command_set(&help_text());
    assert_eq!(help_commands, rust_commands);

    for command in documented_fallback {
        assert!(
            !rust_commands.contains(&command),
            "{command} is documented as Python fallback-only but is present in Rust command list"
        );
        assert!(
            !help_commands.contains(&command),
            "{command} is documented as Python fallback-only but is present in Rust help"
        );
    }
}

#[test]
fn issue_root_cause_map_covers_open_issues() {
    let repo_root = repo_root();
    let map_text =
        std::fs::read_to_string(repo_root.join("docs/development/issue-root-cause-map.json"))
            .expect("issue-root-cause-map.json must be readable");
    let map: Value =
        serde_json::from_str(&map_text).expect("issue-root-cause-map.json must be valid JSON");
    assert_eq!(
        map.get("type").and_then(Value::as_str),
        Some("codefire.issue_root_cause_map.v1")
    );

    let mut mapped_issues = BTreeMap::new();
    let issues = map
        .get("issues")
        .and_then(Value::as_array)
        .expect("issue-root-cause-map.json must contain an issues array");
    assert!(
        !issues.is_empty(),
        "issue-root-cause-map.json must contain at least one audited issue"
    );

    for issue in issues {
        let id = required_str(issue, "id");
        assert!(
            mapped_issues.insert(id.to_string(), issue).is_none(),
            "duplicate issue root cause map entry for {id}"
        );
        for field in [
            "status",
            "root_cause",
            "fix_group",
            "target_version",
            "owner_module",
        ] {
            let value = required_str(issue, field);
            assert!(!value.trim().is_empty(), "{id} has empty {field}");
        }
        let evidence = issue
            .get("close_evidence")
            .and_then(Value::as_array)
            .expect("mapped issue must contain close_evidence array");
        assert!(!evidence.is_empty(), "{id} has no close_evidence");
        assert!(
            issue.get("next_plan_mapping").is_some(),
            "{id} has no next_plan_mapping"
        );
    }

    assert!(
        mapped_issues.contains_key("CFR-120"),
        "CFR-120 close evidence must remain in the root cause map"
    );
    assert!(
        mapped_issues["CFR-120"]
            .get("close_evidence")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|value| value.contains("issue_root_cause_map_covers_open_issues")),
        "CFR-120 must cite this consistency test as close evidence"
    );

    let detail_statuses = detail_issue_statuses(&repo_root);
    for (id, status) in &detail_statuses {
        if status == "open" {
            assert!(
                mapped_issues.contains_key(id),
                "{id} is open but missing from issue-root-cause-map.json"
            );
        }
    }
    for (id, issue) in mapped_issues {
        if let Some(detail_status) = detail_statuses.get(&id) {
            assert_eq!(
                required_str(issue, "status"),
                detail_status,
                "{id} status differs between detail file and root cause map"
            );
        }
    }
}

fn documented_python_fallback_commands() -> std::collections::BTreeSet<String> {
    let docs = include_str!("../../../../docs/development/known-limitations.md");
    let mut commands = std::collections::BTreeSet::new();
    let mut in_section = false;
    let mut in_list = false;
    for line in docs.lines() {
        if line.contains("The Python fallback still owns Python-only maintenance commands") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_list {
                break;
            }
            continue;
        }
        if let Some(command) = trimmed
            .strip_prefix("- `")
            .and_then(|value| value.strip_suffix('`'))
        {
            in_list = true;
            commands.insert(command.to_string());
            continue;
        }
        if in_list {
            break;
        }
    }
    assert!(
        in_section,
        "known-limitations.md fallback command section is missing"
    );
    assert!(
        !commands.is_empty(),
        "known-limitations.md fallback command list is empty"
    );
    commands
}

fn help_command_set(help: &str) -> std::collections::BTreeSet<String> {
    let mut lines = help.lines();
    for line in lines.by_ref() {
        if line.trim() == "commands:" {
            break;
        }
    }
    lines
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("codefire-cli crate must live under crates/codefire-cli")
        .to_path_buf()
}

fn required_str<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("mapped issue is missing string field {field}"))
}

fn detail_issue_statuses(repo_root: &Path) -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    for dir in [
        "docs/development/bug-issues",
        "docs/development/code-review-issues",
    ] {
        let path = repo_root.join(dir);
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.expect("issue detail directory entry must be readable");
            let issue_path = entry.path();
            if issue_path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = issue_path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if !(stem.starts_with("cfb-") || stem.starts_with("cfr-")) {
                continue;
            }
            let id = stem.to_ascii_uppercase();
            let text = std::fs::read_to_string(&issue_path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", issue_path.display()));
            let status = parse_detail_status(&text)
                .unwrap_or_else(|| panic!("{} has no Summary status row", issue_path.display()));
            statuses.insert(id, status);
        }
    }
    statuses
}

fn parse_detail_status(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| Status |") {
            continue;
        }
        let mut cells = trimmed.trim_matches('|').split('|').map(|cell| cell.trim());
        if cells.next() == Some("Status") {
            return cells.next().map(str::to_string);
        }
    }
    None
}
