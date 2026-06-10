use crate::command_registry::COMMAND_NAMES;
use crate::completion::{bash_completion_script, help_text, zsh_completion_script};

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
