use super::{find_repo_root, read_json, CliError};
use crate::automation::cli_command;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct MigrateOptions {
    pub(crate) start: PathBuf,
    pub(crate) mode: MigrateMode,
    pub(crate) target_format: String,
    pub(crate) json_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MigrateMode {
    Check,
    DryRun,
}

#[derive(Debug)]
pub(crate) struct MigrationReport {
    pub(crate) repo_root: PathBuf,
    pub(crate) mode: MigrateMode,
    pub(crate) target_format: String,
    pub(crate) repository_version: Option<i64>,
    pub(crate) compatible: bool,
    pub(crate) checked_objects: usize,
    pub(crate) checked_branches: usize,
    pub(crate) blockers: Vec<MigrationIssue>,
    pub(crate) warnings: Vec<MigrationIssue>,
    pub(crate) planned_actions: Vec<MigrationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MigrationIssue {
    pub(crate) kind: String,
    pub(crate) message: String,
    pub(crate) path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MigrationAction {
    pub(crate) kind: String,
    pub(crate) path: PathBuf,
    pub(crate) description: String,
}

pub(crate) fn parse_migrate_args(args: &[String]) -> Result<MigrateOptions, CliError> {
    let (mode, rest) =
        match args.first().map(String::as_str) {
            Some("check") => (MigrateMode::Check, &args[1..]),
            Some("dry-run") => (MigrateMode::DryRun, &args[1..]),
            Some(command) => {
                return Err(CliError::Usage(format!(
                    "unsupported migrate command: {command}"
                )))
            }
            None => return Err(CliError::Usage(
                "usage: codefire migrate (check|dry-run) [path] [--json] [--target-format v0.6]"
                    .to_string(),
            )),
        };
    let mut start = None;
    let mut target_format = "v0.6".to_string();
    let mut json_output = false;
    let mut index = 0usize;
    while index < rest.len() {
        match rest[index].as_str() {
            "--json" => json_output = true,
            "--target-format" => {
                index += 1;
                target_format = required_arg(rest, index, "--target-format")?.to_string();
            }
            value if value.starts_with("--target-format=") => {
                target_format = value.trim_start_matches("--target-format=").to_string();
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported migrate option: {option}"
                )));
            }
            value if start.is_none() => start = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected migrate argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(MigrateOptions {
        start: start.unwrap_or(std::env::current_dir()?),
        mode,
        target_format,
        json_output,
    })
}

pub(crate) fn run_migrate(options: &MigrateOptions) -> Result<MigrationReport, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    let cf = repo_root.join(".codefire");
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut planned_actions = Vec::new();

    if options.target_format != "v0.6" {
        blockers.push(MigrationIssue {
            kind: "unsupported_target_format".to_string(),
            message: format!("unsupported target format: {}", options.target_format),
            path: None,
        });
    }

    let repository_version = check_repo_json(&cf, &mut blockers)?;
    let checked_objects = check_objects(&cf.join("objects"), &mut blockers, &mut warnings)?;
    let checked_branches = check_branches(&cf, &mut blockers)?;
    collect_missing_v06_dirs(&cf, &mut planned_actions);
    let compatible = blockers.is_empty();

    Ok(MigrationReport {
        repo_root,
        mode: options.mode,
        target_format: options.target_format.clone(),
        repository_version,
        compatible,
        checked_objects,
        checked_branches,
        blockers,
        warnings,
        planned_actions,
    })
}

pub(crate) fn migration_report_data_json(report: &MigrationReport) -> Value {
    json!({
        "type": "codefire_migration_report",
        "version": 1,
        "mode": mode_name(report.mode),
        "target_format": &report.target_format,
        "repository_version": report.repository_version,
        "compatible": report.compatible,
        "checked_objects": report.checked_objects,
        "checked_branches": report.checked_branches,
        "blockers": report.blockers.iter().map(issue_json).collect::<Vec<_>>(),
        "warnings": report.warnings.iter().map(issue_json).collect::<Vec<_>>(),
        "planned_actions": report.planned_actions.iter().map(action_json).collect::<Vec<_>>(),
        "would_write": false,
    })
}

pub(crate) fn migration_report_diagnostics_json(report: &MigrationReport) -> Vec<Value> {
    report
        .blockers
        .iter()
        .chain(report.warnings.iter())
        .map(issue_json)
        .collect()
}

pub(crate) fn migration_report_next_actions(report: &MigrationReport) -> Vec<Value> {
    if report.compatible {
        if report.planned_actions.is_empty() {
            return Vec::new();
        }
        return vec![next_action(
            "review_migration_plan",
            cli_command("migrate dry-run --json"),
            "review v0.6 migration planned actions",
            json!({"planned_actions": report.planned_actions.len()}),
        )];
    }
    vec![next_action(
        "inspect_migration_blockers",
        cli_command("migrate check --json"),
        "inspect migration compatibility blockers",
        json!({"blockers": report.blockers.len()}),
    )]
}

pub(crate) fn print_migration_report(report: &MigrationReport) {
    println!("CodeFire migration {}", mode_name(report.mode));
    println!("repo: {}", report.repo_root.display());
    println!("target format: {}", report.target_format);
    println!("compatible: {}", report.compatible);
    println!("checked objects: {}", report.checked_objects);
    println!("checked branches: {}", report.checked_branches);
    println!("blockers: {}", report.blockers.len());
    for issue in &report.blockers {
        print_issue(issue);
    }
    println!("warnings: {}", report.warnings.len());
    for issue in &report.warnings {
        print_issue(issue);
    }
    println!("planned actions: {}", report.planned_actions.len());
    for action in &report.planned_actions {
        println!(
            "  {}: {} ({})",
            action.kind,
            action.description,
            action.path.display()
        );
    }
}

fn check_repo_json(cf: &Path, blockers: &mut Vec<MigrationIssue>) -> Result<Option<i64>, CliError> {
    let path = cf.join("repo.json");
    if !path.exists() {
        blockers.push(MigrationIssue {
            kind: "missing_repo_json".to_string(),
            message: "missing .codefire/repo.json".to_string(),
            path: Some(path),
        });
        return Ok(None);
    }
    let repo = match read_json(&path) {
        Ok(repo) => repo,
        Err(error) => {
            blockers.push(MigrationIssue {
                kind: "invalid_repo_json".to_string(),
                message: error.to_string(),
                path: Some(path),
            });
            return Ok(None);
        }
    };
    let version = repo.get("version").and_then(Value::as_i64);
    if version != Some(1) {
        blockers.push(MigrationIssue {
            kind: "unsupported_repository_version".to_string(),
            message: format!("expected repository version 1, got {version:?}"),
            path: Some(path.clone()),
        });
    }
    if repo.get("repository_id").and_then(Value::as_str).is_none() {
        blockers.push(MigrationIssue {
            kind: "missing_repository_id".to_string(),
            message: "repo.json must contain repository_id".to_string(),
            path: Some(path),
        });
    }
    Ok(version)
}

fn check_objects(
    objects: &Path,
    blockers: &mut Vec<MigrationIssue>,
    warnings: &mut Vec<MigrationIssue>,
) -> Result<usize, CliError> {
    if !objects.exists() {
        blockers.push(MigrationIssue {
            kind: "missing_objects_dir".to_string(),
            message: "missing .codefire/objects".to_string(),
            path: Some(objects.to_path_buf()),
        });
        return Ok(0);
    }
    let mut checked = 0usize;
    for path in collect_json_files(objects)? {
        let record = match fs::read_to_string(&path)
            .map_err(CliError::from)
            .and_then(|text| {
                serde_json::from_str::<codefire_store::ObjectRecord>(&text).map_err(CliError::from)
            }) {
            Ok(record) => record,
            Err(error) => {
                blockers.push(MigrationIssue {
                    kind: "invalid_object_record".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                });
                continue;
            }
        };
        match codefire_store::validate_object_record(&record, None, Some(&path)) {
            Ok(()) => checked += 1,
            Err(error) => blockers.push(MigrationIssue {
                kind: "object_integrity_error".to_string(),
                message: error.to_string(),
                path: Some(path),
            }),
        }
    }
    if checked == 0 {
        warnings.push(MigrationIssue {
            kind: "empty_object_store".to_string(),
            message: "object store has no valid object records".to_string(),
            path: Some(objects.to_path_buf()),
        });
    }
    Ok(checked)
}

fn check_branches(cf: &Path, blockers: &mut Vec<MigrationIssue>) -> Result<usize, CliError> {
    let branches = cf.join("branches");
    if !branches.exists() {
        blockers.push(MigrationIssue {
            kind: "missing_branches_dir".to_string(),
            message: "missing .codefire/branches".to_string(),
            path: Some(branches),
        });
        return Ok(0);
    }
    let objects = cf.join("objects");
    let mut checked = 0usize;
    for path in collect_json_files(&branches)? {
        let branch = match read_json(&path) {
            Ok(branch) => branch,
            Err(error) => {
                blockers.push(MigrationIssue {
                    kind: "invalid_branch_record".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                });
                continue;
            }
        };
        let head = match branch.get("head").and_then(Value::as_str) {
            Some(head) => head,
            None => {
                blockers.push(MigrationIssue {
                    kind: "missing_branch_head".to_string(),
                    message: "branch record must contain head".to_string(),
                    path: Some(path),
                });
                continue;
            }
        };
        match codefire_store::validate_sealed_commit(&objects, head) {
            Ok(()) => checked += 1,
            Err(error) => blockers.push(MigrationIssue {
                kind: "invalid_branch_head".to_string(),
                message: error.to_string(),
                path: Some(path),
            }),
        }
    }
    Ok(checked)
}

fn collect_missing_v06_dirs(cf: &Path, actions: &mut Vec<MigrationAction>) {
    for relative in [
        "objects/artifact_refs",
        "objects/evidence",
        "idempotency",
        "locks",
        "remotes",
    ] {
        let path = cf.join(relative);
        if !path.exists() {
            actions.push(MigrationAction {
                kind: "create_directory".to_string(),
                path,
                description: format!("create .codefire/{relative}"),
            });
        }
    }
}

fn collect_json_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn issue_json(issue: &MigrationIssue) -> Value {
    json!({
        "kind": &issue.kind,
        "message": &issue.message,
        "path": &issue.path,
    })
}

fn action_json(action: &MigrationAction) -> Value {
    json!({
        "kind": &action.kind,
        "path": &action.path,
        "description": &action.description,
    })
}

fn next_action(id: &str, command: String, description: &str, context: Value) -> Value {
    json!({
        "id": id,
        "command": command,
        "description": description,
        "context": context,
    })
}

fn mode_name(mode: MigrateMode) -> &'static str {
    match mode {
        MigrateMode::Check => "check",
        MigrateMode::DryRun => "dry-run",
    }
}

fn print_issue(issue: &MigrationIssue) {
    match &issue.path {
        Some(path) => println!("  {}: {} ({})", issue.kind, issue.message, path.display()),
        None => println!("  {}: {}", issue.kind, issue.message),
    }
}

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}
