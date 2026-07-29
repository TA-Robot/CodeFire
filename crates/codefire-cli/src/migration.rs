use super::{
    command_arg, find_repo_root, parse_lock_option, read_json, CliError, LockOptions, RepoLock,
};
use crate::automation::{cli_command, next_action as automation_next_action};
use crate::repo_layout::{object_subdir_requirement, LayoutRequirement, REPO_DIRS};
use crate::secure_worktree::{self, StableFsIdentity};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct MigrateOptions {
    pub(crate) start: PathBuf,
    pub(crate) mode: MigrateMode,
    pub(crate) target_format: String,
    pub(crate) json_output: bool,
    pub(crate) quick: bool,
    pub(crate) lock: LockOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MigrateMode {
    Check,
    DryRun,
    Apply,
}

#[derive(Debug, Clone)]
pub(crate) struct MigrationReport {
    pub(crate) repo_root: PathBuf,
    pub(crate) mode: MigrateMode,
    pub(crate) target_format: String,
    pub(crate) supported_target_formats: Vec<String>,
    pub(crate) repository_version: Option<i64>,
    pub(crate) compatible: bool,
    pub(crate) checked_objects: usize,
    pub(crate) checked_branches: usize,
    pub(crate) quick: bool,
    pub(crate) skipped_checks: Vec<String>,
    pub(crate) blockers: Vec<MigrationIssue>,
    pub(crate) warnings: Vec<MigrationIssue>,
    pub(crate) planned_actions: Vec<MigrationAction>,
    pub(crate) applied_actions: Vec<MigrationAction>,
    pub(crate) skipped_actions: Vec<MigrationAction>,
    pub(crate) remaining_actions: Vec<MigrationAction>,
    pub(crate) would_write: bool,
    pub(crate) did_write: bool,
    pub(crate) format_write_performed: bool,
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
    relative: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootstrapOutcome {
    NotNeeded,
    Created,
    ConcurrentlyCreated,
}

struct MigrationSubject {
    repo_root: PathBuf,
    repo: Dir,
    codefire: Dir,
    codefire_identity: StableFsIdentity,
    objects: Dir,
    objects_identity: StableFsIdentity,
}

#[derive(Debug)]
struct MigrationActionApplyError {
    source: io::Error,
    directory_created: bool,
}

impl std::fmt::Display for MigrationActionApplyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

pub(crate) fn parse_migrate_args(args: &[String]) -> Result<MigrateOptions, CliError> {
    let (mode, rest) =
        match args.first().map(String::as_str) {
            Some("check") => (MigrateMode::Check, &args[1..]),
            Some("dry-run") => (MigrateMode::DryRun, &args[1..]),
            Some("apply") => (MigrateMode::Apply, &args[1..]),
            Some(command) => {
                return Err(CliError::Usage(format!(
                    "unsupported migrate command: {command}"
                )))
            }
            None => return Err(CliError::Usage(
                "usage: codefire migrate (check|dry-run|apply) [path] [--json] [--target-format current|v0.6] [--quick|--full] [--wait-lock] [--lock-timeout <duration>]"
                    .to_string(),
            )),
        };
    let mut start = None;
    let mut target_format = CURRENT_FORMAT.to_string();
    let mut json_output = false;
    let mut quick = false;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < rest.len() {
        if parse_lock_option(rest, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match rest[index].as_str() {
            "--json" => json_output = true,
            "--quick" => quick = true,
            "--full" => quick = false,
            "--target-format" => {
                index += 1;
                target_format = required_arg(rest, index, "--target-format")?.to_string();
            }
            "--path" => {
                index += 1;
                let value = required_arg(rest, index, "--path")?;
                set_single_start(&mut start, value)?;
            }
            value if value.starts_with("--target-format=") => {
                target_format = value.trim_start_matches("--target-format=").to_string();
            }
            value if value.starts_with("--path=") => {
                set_single_start(&mut start, value.trim_start_matches("--path="))?;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported migrate option: {option}"
                )));
            }
            value => set_single_start(&mut start, value)?,
        }
        index += 1;
    }
    if mode != MigrateMode::Apply && lock != LockOptions::default() {
        return Err(CliError::Usage(
            "--wait-lock and --lock-timeout are only supported by migrate apply".to_string(),
        ));
    }
    if mode == MigrateMode::Apply && quick {
        return Err(CliError::Usage(
            "migrate apply requires a full compatibility scan; remove --quick or pass --full"
                .to_string(),
        ));
    }
    Ok(MigrateOptions {
        start: start.unwrap_or(std::env::current_dir()?),
        mode,
        target_format,
        json_output,
        quick,
        lock,
    })
}

const CURRENT_FORMAT: &str = "current";
const LEGACY_V06_FORMAT: &str = "v0.6";

fn supported_target_formats() -> Vec<String> {
    vec![CURRENT_FORMAT.to_string(), LEGACY_V06_FORMAT.to_string()]
}

fn set_single_start(start: &mut Option<PathBuf>, value: &str) -> Result<(), CliError> {
    match start {
        Some(_) => Err(CliError::Usage(
            "path may only be specified once".to_string(),
        )),
        None => {
            *start = Some(PathBuf::from(value));
            Ok(())
        }
    }
}

pub(crate) fn run_migrate(options: &MigrateOptions) -> Result<MigrationReport, CliError> {
    if options.mode == MigrateMode::Apply {
        return run_migrate_apply_with_hook(options, |_| Ok(()));
    }
    inspect_migration(options)
}

fn inspect_migration(options: &MigrateOptions) -> Result<MigrationReport, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    let cf = repo_root.join(".codefire");
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut planned_actions = Vec::new();

    if !supported_target_formats()
        .iter()
        .any(|target| target == &options.target_format)
    {
        blockers.push(MigrationIssue {
            kind: "unsupported_target_format".to_string(),
            message: format!(
                "unsupported target format: {}; supported: {}",
                options.target_format,
                supported_target_formats().join(", ")
            ),
            path: None,
        });
    }

    let repository_version = check_repo_json(&cf, &mut blockers)?;
    let mut skipped_checks = Vec::new();
    let checked_objects = if options.quick {
        skipped_checks.push("object_record_integrity".to_string());
        0
    } else {
        check_objects(&cf.join("objects"), &mut blockers, &mut warnings)?
    };
    let checked_branches = check_branches(&cf, &mut blockers, &mut warnings)?;
    collect_layout_findings(&cf, &mut blockers, &mut planned_actions);
    let compatible = blockers.is_empty();
    let would_write =
        options.mode == MigrateMode::DryRun && compatible && !planned_actions.is_empty();

    Ok(MigrationReport {
        repo_root,
        mode: options.mode,
        target_format: options.target_format.clone(),
        supported_target_formats: supported_target_formats(),
        repository_version,
        compatible,
        checked_objects,
        checked_branches,
        quick: options.quick,
        skipped_checks,
        blockers,
        warnings,
        remaining_actions: planned_actions.clone(),
        planned_actions,
        applied_actions: Vec::new(),
        skipped_actions: Vec::new(),
        would_write,
        did_write: false,
        format_write_performed: false,
    })
}

fn run_migrate_apply_with_hook<F>(
    options: &MigrateOptions,
    after_locked_plan: F,
) -> Result<MigrationReport, CliError>
where
    F: FnOnce(&MigrationReport) -> Result<(), CliError>,
{
    debug_assert_eq!(options.mode, MigrateMode::Apply);

    // Do the complete read-only validation first. A repository that is already known to be
    // incompatible must not even bootstrap its operational lock directory.
    let mut preflight = inspect_migration(options)?;
    preflight.mode = MigrateMode::Apply;
    preflight.would_write = preflight.compatible && !preflight.planned_actions.is_empty();
    if !preflight.compatible {
        return Ok(preflight);
    }

    let subject = match MigrationSubject::bind(&preflight.repo_root) {
        Ok(subject) => subject,
        Err(error) => {
            let codefire_path = preflight.repo_root.join(".codefire");
            add_apply_blocker(
                &mut preflight,
                "migration_subject_invalid",
                error.to_string(),
                Some(codefire_path),
            );
            return Ok(preflight);
        }
    };

    let lock_action = preflight
        .planned_actions
        .iter()
        .find(|action| action.relative == Path::new("locks"))
        .cloned();
    let bootstrap = match subject.bootstrap_lock_directory(lock_action.is_some()) {
        Ok(outcome) => outcome,
        Err(error) => {
            let locks_path = preflight.repo_root.join(".codefire/locks");
            add_apply_blocker(
                &mut preflight,
                "migration_lock_bootstrap_failed",
                error.to_string(),
                Some(locks_path),
            );
            return Ok(preflight);
        }
    };
    if let Some(action) = &lock_action {
        match bootstrap {
            BootstrapOutcome::Created => preflight.applied_actions.push(action.clone()),
            BootstrapOutcome::ConcurrentlyCreated => preflight.skipped_actions.push(action.clone()),
            BootstrapOutcome::NotNeeded => {}
        }
    }
    preflight.did_write = bootstrap == BootstrapOutcome::Created;

    let lock = RepoLock::acquire_with_options(&preflight.repo_root, &options.lock)?;
    if let Err(error) = subject.ensure_identity() {
        let repo_root = preflight.repo_root.clone();
        add_apply_blocker(
            &mut preflight,
            "migration_subject_changed",
            error.to_string(),
            Some(repo_root),
        );
        return Ok(preflight);
    }
    lock.ensure_repo_root(&preflight.repo_root)?;

    // The preflight plan is advisory only. The execution authority is always a fresh full report
    // made while holding the persistent repository lock.
    let mut locked = inspect_migration(options)?;
    locked.mode = MigrateMode::Apply;
    locked.applied_actions = preflight.applied_actions;
    locked.skipped_actions = preflight.skipped_actions;
    locked.did_write = preflight.did_write;
    locked.would_write = locked.compatible && preflight.would_write;
    if !locked.compatible {
        return Ok(locked);
    }

    let expected_after_bootstrap = action_keys(
        preflight
            .planned_actions
            .iter()
            .filter(|action| action.relative != Path::new("locks")),
    );
    if expected_after_bootstrap != action_keys(locked.planned_actions.iter()) {
        let codefire_path = locked.repo_root.join(".codefire");
        add_apply_blocker(
            &mut locked,
            "migration_plan_changed",
            "migration plan changed between preflight and locked reload; rerun migrate dry-run"
                .to_string(),
            Some(codefire_path),
        );
        return Ok(locked);
    }

    after_locked_plan(&locked)?;
    if let Err(error) = subject.ensure_identity() {
        let repo_root = locked.repo_root.clone();
        add_apply_blocker(
            &mut locked,
            "migration_subject_changed",
            error.to_string(),
            Some(repo_root),
        );
        return Ok(locked);
    }
    lock.ensure_repo_root(&locked.repo_root)?;

    let (layout_blockers, reloaded_actions) = reload_layout_plan(&locked.repo_root);
    if let Err(error) = subject.ensure_identity() {
        let repo_root = locked.repo_root.clone();
        add_apply_blocker(
            &mut locked,
            "migration_subject_changed",
            error.to_string(),
            Some(repo_root),
        );
        return Ok(locked);
    }
    lock.ensure_repo_root(&locked.repo_root)?;
    if !layout_blockers.is_empty()
        || action_keys(reloaded_actions.iter()) != action_keys(locked.planned_actions.iter())
    {
        locked.blockers.extend(layout_blockers);
        let codefire_path = locked.repo_root.join(".codefire");
        add_apply_blocker(
            &mut locked,
            "migration_plan_changed",
            "migration plan changed after the locked plan was prepared; no planned action was applied"
                .to_string(),
            Some(codefire_path),
        );
        locked.remaining_actions = reloaded_actions;
        return Ok(locked);
    }

    let execution_plan = locked.planned_actions.clone();
    for action in &execution_plan {
        if let Err(error) = subject.validate_missing_action(action) {
            add_apply_blocker(
                &mut locked,
                "migration_action_precondition_changed",
                error.to_string(),
                Some(action.path.clone()),
            );
            return Ok(locked);
        }
    }

    for (index, action) in execution_plan.iter().enumerate() {
        if let Err(error) = subject.create_registered_directory(action) {
            if error.directory_created {
                locked.applied_actions.push(action.clone());
                locked.did_write = true;
                locked.remaining_actions = execution_plan[index + 1..].to_vec();
            } else {
                locked.remaining_actions = execution_plan[index..].to_vec();
            }
            add_apply_blocker(
                &mut locked,
                "migration_apply_failed",
                error.to_string(),
                Some(action.path.clone()),
            );
            return Ok(locked);
        }
        locked.applied_actions.push(action.clone());
        locked.did_write = true;
        locked.remaining_actions = execution_plan[index + 1..].to_vec();
        if let Err(error) = subject.ensure_identity() {
            let repo_root = locked.repo_root.clone();
            add_apply_blocker(
                &mut locked,
                "migration_subject_changed",
                error.to_string(),
                Some(repo_root),
            );
            return Ok(locked);
        }
        lock.ensure_repo_root(&locked.repo_root)?;
    }

    if let Err(error) = subject.ensure_identity() {
        let repo_root = locked.repo_root.clone();
        add_apply_blocker(
            &mut locked,
            "migration_subject_changed",
            error.to_string(),
            Some(repo_root),
        );
        return Ok(locked);
    }
    lock.ensure_repo_root(&locked.repo_root)?;
    let (layout_blockers, remaining_actions) = reload_layout_plan(&locked.repo_root);
    if let Err(error) = subject.ensure_identity() {
        let repo_root = locked.repo_root.clone();
        add_apply_blocker(
            &mut locked,
            "migration_subject_changed",
            error.to_string(),
            Some(repo_root),
        );
        return Ok(locked);
    }
    lock.ensure_repo_root(&locked.repo_root)?;
    if !layout_blockers.is_empty() || !remaining_actions.is_empty() {
        locked.blockers.extend(layout_blockers);
        locked.remaining_actions = remaining_actions;
        let codefire_path = locked.repo_root.join(".codefire");
        add_apply_blocker(
            &mut locked,
            "migration_postcondition_failed",
            "repository layout did not match the applied locked plan".to_string(),
            Some(codefire_path),
        );
        return Ok(locked);
    }
    locked.remaining_actions.clear();
    locked.compatible = true;
    Ok(locked)
}

#[cfg(test)]
pub(crate) fn run_migrate_apply_with_locked_plan_hook<F>(
    options: &MigrateOptions,
    hook: F,
) -> Result<MigrationReport, CliError>
where
    F: FnOnce(&MigrationReport) -> Result<(), CliError>,
{
    run_migrate_apply_with_hook(options, hook)
}

fn add_apply_blocker(
    report: &mut MigrationReport,
    kind: &str,
    message: String,
    path: Option<PathBuf>,
) {
    report.compatible = false;
    report.would_write = false;
    report.blockers.push(MigrationIssue {
        kind: kind.to_string(),
        message,
        path,
    });
}

fn action_keys<'a>(actions: impl IntoIterator<Item = &'a MigrationAction>) -> BTreeSet<PathBuf> {
    actions
        .into_iter()
        .map(|action| action.relative.clone())
        .collect()
}

fn reload_layout_plan(repo_root: &Path) -> (Vec<MigrationIssue>, Vec<MigrationAction>) {
    let mut blockers = Vec::new();
    let mut actions = Vec::new();
    collect_layout_findings(&repo_root.join(".codefire"), &mut blockers, &mut actions);
    (blockers, actions)
}

pub(crate) fn migration_report_data_json(report: &MigrationReport) -> Value {
    json!({
        "type": "codefire_migration_report",
        "version": 1,
        "mode": mode_name(report.mode),
        "target_format": &report.target_format,
        "target_format_semantics": "compatibility_profile_only",
        "current_format": CURRENT_FORMAT,
        "supported_target_formats": &report.supported_target_formats,
        "repository_version": report.repository_version,
        "compatible": report.compatible,
        "scan_mode": if report.quick { "quick" } else { "full" },
        "skipped_checks": &report.skipped_checks,
        "checked_objects": report.checked_objects,
        "checked_branches": report.checked_branches,
        "blockers": report.blockers.iter().map(issue_json).collect::<Vec<_>>(),
        "warnings": report.warnings.iter().map(issue_json).collect::<Vec<_>>(),
        "planned_actions": report.planned_actions.iter().map(action_json).collect::<Vec<_>>(),
        "applied_actions": report.applied_actions.iter().map(action_json).collect::<Vec<_>>(),
        "skipped_actions": report.skipped_actions.iter().map(action_json).collect::<Vec<_>>(),
        "remaining_actions": report.remaining_actions.iter().map(action_json).collect::<Vec<_>>(),
        "applied_count": report.applied_actions.len(),
        "skipped_count": report.skipped_actions.len(),
        "remaining_count": report.remaining_actions.len(),
        "would_write": report.would_write,
        "did_write": report.did_write,
        "format_write_supported": false,
        "format_write_performed": report.format_write_performed,
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
    let path = command_arg(&report.repo_root.to_string_lossy());
    let target_format = command_arg(&report.target_format);
    let full_apply_command = || {
        cli_command(format!(
            "migrate apply --path {path} --target-format {target_format} --full --json"
        ))
    };

    if report.compatible {
        match report.mode {
            MigrateMode::Apply => {
                if report.remaining_actions.is_empty() {
                    return Vec::new();
                }
                return vec![automation_next_action(
                    "retry_remaining_migration_actions",
                    full_apply_command(),
                    "revalidate and apply only the layout repairs that remain",
                    json!({
                        "remaining_actions": report.remaining_actions.len(),
                        "applied_actions": report.applied_actions.len(),
                    }),
                )];
            }
            MigrateMode::DryRun => {
                if report.planned_actions.is_empty() {
                    return Vec::new();
                }
                return vec![automation_next_action(
                    "apply_migration_plan",
                    full_apply_command(),
                    "apply the validated layout-only migration plan under the repository lock",
                    json!({"planned_actions": report.planned_actions.len()}),
                )];
            }
            MigrateMode::Check => {}
        }

        if report.planned_actions.is_empty() {
            return Vec::new();
        }
        let scan_mode = if report.quick { "--quick" } else { "--full" };
        return vec![automation_next_action(
            "review_migration_plan",
            cli_command(format!(
                "migrate dry-run --path {path} --target-format {target_format} {scan_mode} --json"
            )),
            "review migration planned actions",
            json!({"planned_actions": report.planned_actions.len()}),
        )];
    }

    if report.mode == MigrateMode::Apply && !report.remaining_actions.is_empty() {
        return vec![automation_next_action(
            "retry_remaining_migration_actions",
            full_apply_command(),
            "revalidate blockers and retry only the layout repairs that remain",
            json!({
                "remaining_actions": report.remaining_actions.len(),
                "applied_actions": report.applied_actions.len(),
                "blockers": report.blockers.len(),
            }),
        )];
    }

    vec![automation_next_action(
        "inspect_migration_blockers",
        cli_command(format!(
            "migrate check --path {path} --target-format {target_format} --full --json"
        )),
        "inspect migration compatibility blockers",
        json!({"blockers": report.blockers.len()}),
    )]
}

pub(crate) fn print_migration_report(report: &MigrationReport) {
    println!("CodeFire migration {}", mode_name(report.mode));
    println!("repo: {}", report.repo_root.display());
    println!("target format: {}", report.target_format);
    println!("scan mode: {}", if report.quick { "quick" } else { "full" });
    if !report.skipped_checks.is_empty() {
        println!("skipped checks: {}", report.skipped_checks.join(", "));
    }
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
    println!("would write: {}", report.would_write);
    println!("did write: {}", report.did_write);
    println!("format write performed: {}", report.format_write_performed);
    println!("applied actions: {}", report.applied_actions.len());
    for action in &report.applied_actions {
        println!(
            "  {}: {} ({})",
            action.kind,
            action.description,
            action.path.display()
        );
    }
    println!("skipped actions: {}", report.skipped_actions.len());
    println!("remaining actions: {}", report.remaining_actions.len());
}

fn check_repo_json(cf: &Path, blockers: &mut Vec<MigrationIssue>) -> Result<Option<i64>, CliError> {
    let path = cf.join("repo.json");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            blockers.push(MigrationIssue {
                kind: "invalid_repo_json_subject".to_string(),
                message: ".codefire/repo.json must be a regular no-follow file".to_string(),
                path: Some(path),
            });
            return Ok(None);
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            blockers.push(MigrationIssue {
                kind: "missing_repo_json".to_string(),
                message: "missing .codefire/repo.json".to_string(),
                path: Some(path),
            });
            return Ok(None);
        }
        Err(error) => return Err(CliError::Io(error)),
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
    codefire_store::scan_object_store_record_documents(objects, |entry| {
        let path = entry.path;
        let record = match entry.document {
            Ok(document) => document.record,
            Err(error) => {
                blockers.push(MigrationIssue {
                    kind: "invalid_object_record".to_string(),
                    message: error.to_string(),
                    path: Some(path),
                });
                return;
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
    })?;
    if checked == 0 {
        warnings.push(MigrationIssue {
            kind: "empty_object_store".to_string(),
            message: "object store has no valid object records".to_string(),
            path: Some(objects.to_path_buf()),
        });
    }
    Ok(checked)
}

fn check_branches(
    cf: &Path,
    blockers: &mut Vec<MigrationIssue>,
    warnings: &mut Vec<MigrationIssue>,
) -> Result<usize, CliError> {
    check_branches_with_hook(cf, blockers, warnings, || Ok(()))
}

fn check_branches_with_hook<F>(
    cf: &Path,
    blockers: &mut Vec<MigrationIssue>,
    warnings: &mut Vec<MigrationIssue>,
    before_finalize: F,
) -> Result<usize, CliError>
where
    F: FnOnce() -> Result<(), CliError>,
{
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
    let (mut validation, validation_begin_error) =
        match codefire_store::begin_sealed_commit_validation_session(&objects) {
            Ok(session) => (Some(session), None),
            Err(error) => (None, Some(error.to_string())),
        };
    let mut successful = Vec::<(String, PathBuf, usize)>::new();
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
        let validation_result = match validation.as_mut() {
            Some(session) => session
                .validate_commit_recoverable(head)
                .map_err(|error| error.to_string()),
            None => Err(validation_begin_error
                .clone()
                .unwrap_or_else(|| "sealed validation session is unavailable".to_string())),
        };
        match validation_result {
            Ok(()) => successful.push((head.to_string(), path, blockers.len())),
            Err(message) => blockers.push(MigrationIssue {
                kind: "invalid_branch_head".to_string(),
                message,
                path: Some(path),
            }),
        }
    }
    before_finalize()?;
    let finalization_error = validation
        .as_mut()
        .and_then(|session| session.finalize().err())
        .map(|error| error.to_string());
    let mut checked = 0usize;
    let mut inserted = 0usize;
    for (head, path, blocker_index) in successful {
        if let Some(message) = finalization_error.as_ref() {
            blockers.insert(
                blocker_index + inserted,
                MigrationIssue {
                    kind: "invalid_branch_head".to_string(),
                    message: message.clone(),
                    path: Some(path),
                },
            );
            inserted += 1;
            continue;
        }
        match validation
            .as_ref()
            .expect("successful migration validation retains its session")
            .validated_commit_payload(&head)
        {
            Ok(commit) => {
                checked += 1;
                if commit.get("semantic_validation_profile").is_none() {
                    warnings.push(MigrationIssue {
                        kind: "legacy_semantic_validation_profile".to_string(),
                        message: format!(
                            "branch head {head} has no semantic_validation_profile; create a current-profile bridge commit before relying on current semantic validation"
                        ),
                        path: Some(path),
                    });
                }
            }
            Err(error) => {
                blockers.insert(
                    blocker_index + inserted,
                    MigrationIssue {
                        kind: "invalid_branch_head".to_string(),
                        message: error.to_string(),
                        path: Some(path),
                    },
                );
                inserted += 1;
            }
        }
    }
    Ok(checked)
}

fn collect_layout_findings(
    cf: &Path,
    blockers: &mut Vec<MigrationIssue>,
    actions: &mut Vec<MigrationAction>,
) {
    for entry in REPO_DIRS {
        let path = cf.join(entry.relative);
        match layout_directory_state(&path) {
            Ok(LayoutDirectoryState::Directory) => {}
            Ok(LayoutDirectoryState::Missing)
                if entry.requirement == LayoutRequirement::AutoCreate =>
            {
                actions.push(create_directory_action(
                    cf,
                    Path::new(entry.relative),
                    format!("create .codefire/{}", entry.relative),
                ));
            }
            Ok(LayoutDirectoryState::Missing) => blockers.push(MigrationIssue {
                kind: "missing_required_directory".to_string(),
                message: format!("missing required .codefire/{} directory", entry.relative),
                path: Some(path),
            }),
            Ok(LayoutDirectoryState::Invalid) => blockers.push(MigrationIssue {
                kind: "invalid_layout_directory".to_string(),
                message: format!(
                    ".codefire/{} must be a real no-follow directory",
                    entry.relative
                ),
                path: Some(path),
            }),
            Err(error) => blockers.push(MigrationIssue {
                kind: "layout_directory_inspection_failed".to_string(),
                message: error.to_string(),
                path: Some(path),
            }),
        }
    }
    let objects = cf.join("objects");
    for subdir in codefire_store::known_object_subdirs() {
        let path = objects.join(subdir);
        match layout_directory_state(&path) {
            Ok(LayoutDirectoryState::Directory) => {}
            Ok(LayoutDirectoryState::Missing)
                if object_subdir_requirement(subdir) == LayoutRequirement::AutoCreate =>
            {
                actions.push(create_directory_action(
                    cf,
                    &Path::new("objects").join(subdir),
                    format!("create .codefire/objects/{subdir}"),
                ));
            }
            Ok(LayoutDirectoryState::Missing) => blockers.push(MigrationIssue {
                kind: "missing_required_object_subdirectory".to_string(),
                message: format!("missing required .codefire/objects/{subdir} directory"),
                path: Some(path),
            }),
            Ok(LayoutDirectoryState::Invalid) => blockers.push(MigrationIssue {
                kind: "invalid_layout_object_subdirectory".to_string(),
                message: format!(".codefire/objects/{subdir} must be a real no-follow directory"),
                path: Some(path),
            }),
            Err(error) => blockers.push(MigrationIssue {
                kind: "layout_object_subdirectory_inspection_failed".to_string(),
                message: error.to_string(),
                path: Some(path),
            }),
        }
    }
    actions.sort_by(|left, right| left.relative.cmp(&right.relative));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutDirectoryState {
    Missing,
    Directory,
    Invalid,
}

fn layout_directory_state(path: &Path) -> io::Result<LayoutDirectoryState> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            Ok(LayoutDirectoryState::Directory)
        }
        Ok(_) => Ok(LayoutDirectoryState::Invalid),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LayoutDirectoryState::Missing),
        Err(error) => Err(error),
    }
}

fn create_directory_action(
    codefire: &Path,
    relative: &Path,
    description: String,
) -> MigrationAction {
    MigrationAction {
        kind: "create_directory".to_string(),
        path: codefire.join(relative),
        description,
        relative: relative.to_path_buf(),
    }
}

impl MigrationSubject {
    fn bind(repo_root: &Path) -> io::Result<Self> {
        let repo_root = fs::canonicalize(repo_root)?;
        let repo = secure_worktree::open_absolute_dir_nofollow(&repo_root)?;
        let codefire = repo.open_dir_nofollow(".codefire")?;
        let codefire_identity = StableFsIdentity::for_directory(&codefire)?;
        let objects = codefire.open_dir_nofollow("objects")?;
        let objects_identity = StableFsIdentity::for_directory(&objects)?;
        let subject = Self {
            repo_root,
            repo,
            codefire,
            codefire_identity,
            objects,
            objects_identity,
        };
        subject.ensure_identity()?;
        Ok(subject)
    }

    fn ensure_identity(&self) -> io::Result<()> {
        secure_worktree::ensure_absolute_dir_identity(&self.repo, &self.repo_root)?;
        let codefire = self.repo.open_dir_nofollow(".codefire")?;
        self.codefire_identity
            .ensure_directory(&codefire, "migration .codefire directory")?;
        secure_worktree::ensure_same_directory(&self.codefire, &codefire)?;
        let objects = self.codefire.open_dir_nofollow("objects")?;
        self.objects_identity
            .ensure_directory(&objects, "migration objects directory")?;
        secure_worktree::ensure_same_directory(&self.objects, &objects)
    }

    fn bootstrap_lock_directory(&self, expected_missing: bool) -> io::Result<BootstrapOutcome> {
        self.ensure_identity()?;
        let outcome = match self.codefire.symlink_metadata("locks") {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                let locks = self.codefire.open_dir_nofollow("locks")?;
                let rebound = self.codefire.open_dir_nofollow("locks")?;
                secure_worktree::ensure_same_directory(&locks, &rebound)?;
                if expected_missing {
                    BootstrapOutcome::ConcurrentlyCreated
                } else {
                    BootstrapOutcome::NotNeeded
                }
            }
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "migration lock path is not a real no-follow directory",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if !expected_missing {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "migration lock directory disappeared after the preflight plan",
                    ));
                }
                match self.codefire.create_dir("locks") {
                    Ok(()) => {
                        sync_capability_directory(&self.codefire)?;
                        BootstrapOutcome::Created
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                        BootstrapOutcome::ConcurrentlyCreated
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        };
        let locks = self.codefire.open_dir_nofollow("locks")?;
        let locks_identity = StableFsIdentity::for_directory(&locks)?;
        let rebound = self.codefire.open_dir_nofollow("locks")?;
        locks_identity.ensure_directory(&rebound, "migration lock directory")?;
        self.ensure_identity()?;
        Ok(outcome)
    }

    fn validate_missing_action(&self, action: &MigrationAction) -> io::Result<()> {
        let (parent, name, expected_parent_identity) = self.action_parent(action)?;
        self.ensure_identity()?;
        expected_parent_identity.ensure_directory(parent, "migration action parent")?;
        match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "planned migration target became a symlink: {}",
                        action.path.display()
                    ),
                ))
            }
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "planned migration target already exists: {}",
                        action.path.display()
                    ),
                ))
            }
            Err(error) => return Err(error),
        }
        self.ensure_identity()
    }

    fn create_registered_directory(
        &self,
        action: &MigrationAction,
    ) -> Result<(), MigrationActionApplyError> {
        self.validate_missing_action(action)
            .map_err(|source| MigrationActionApplyError {
                source,
                directory_created: false,
            })?;
        let (parent, name, expected_parent_identity) =
            self.action_parent(action)
                .map_err(|source| MigrationActionApplyError {
                    source,
                    directory_created: false,
                })?;
        expected_parent_identity
            .ensure_directory(parent, "migration action parent")
            .map_err(|source| MigrationActionApplyError {
                source,
                directory_created: false,
            })?;
        parent.create_dir(&name).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                MigrationActionApplyError {
                    source: io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "migration plan changed concurrently before create: {}",
                            action.path.display()
                        ),
                    ),
                    directory_created: false,
                }
            } else {
                MigrationActionApplyError {
                    source: error,
                    directory_created: false,
                }
            }
        })?;
        (|| {
            let created = parent.open_dir_nofollow(&name)?;
            let created_identity = StableFsIdentity::for_directory(&created)?;
            let rebound = parent.open_dir_nofollow(&name)?;
            created_identity.ensure_directory(&rebound, "created migration directory")?;
            expected_parent_identity.ensure_directory(parent, "migration action parent")?;
            sync_capability_directory(parent)?;
            self.ensure_identity()
        })()
        .map_err(|source| MigrationActionApplyError {
            source,
            directory_created: true,
        })
    }

    fn action_parent(
        &self,
        action: &MigrationAction,
    ) -> io::Result<(&Dir, OsString, StableFsIdentity)> {
        if action.kind != "create_directory" || !is_registered_auto_create(&action.relative) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "migration action is not a registered auto-create directory: {}",
                    action.relative.display()
                ),
            ));
        }
        let expected_path = self.repo_root.join(".codefire").join(&action.relative);
        if action.path != expected_path
            || !action.path.starts_with(self.repo_root.join(".codefire"))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "migration action escapes or disagrees with its repository subject: {}",
                    action.path.display()
                ),
            ));
        }
        let name = action.relative.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "migration action has no name")
        })?;
        let relative_parent = action.relative.parent().unwrap_or_else(|| Path::new(""));
        if relative_parent == Path::new("objects") {
            return Ok((&self.objects, name.to_os_string(), self.objects_identity));
        }
        if relative_parent.as_os_str().is_empty() {
            return Ok((&self.codefire, name.to_os_string(), self.codefire_identity));
        }
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "migration action has an unsupported parent: {}",
                action.relative.display()
            ),
        ))
    }
}

fn is_registered_auto_create(relative: &Path) -> bool {
    if REPO_DIRS.iter().any(|entry| {
        entry.requirement == LayoutRequirement::AutoCreate && relative == Path::new(entry.relative)
    }) {
        return true;
    }
    relative
        .parent()
        .is_some_and(|parent| parent == Path::new("objects"))
        && relative
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|subdir| {
                codefire_store::known_object_subdirs().any(|known| known == subdir)
                    && object_subdir_requirement(subdir) == LayoutRequirement::AutoCreate
            })
}

fn sync_capability_directory(directory: &Dir) -> io::Result<()> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
pub(crate) fn validate_migration_action_path_for_test(
    repo_root: &Path,
    relative: &Path,
    path: &Path,
) -> Result<(), CliError> {
    let subject = MigrationSubject::bind(repo_root)?;
    let action = MigrationAction {
        kind: "create_directory".to_string(),
        path: path.to_path_buf(),
        description: "test action".to_string(),
        relative: relative.to_path_buf(),
    };
    subject
        .action_parent(&action)
        .map(|_| ())
        .map_err(CliError::Io)
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

fn mode_name(mode: MigrateMode) -> &'static str {
    match mode {
        MigrateMode::Check => "check",
        MigrateMode::DryRun => "dry-run",
        MigrateMode::Apply => "apply",
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migration_branch_session_turns_final_subject_change_into_ordered_blocker() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let initialized = crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let objects = cf.join("objects");
        let commit_path =
            codefire_store::object_record_path(&objects, &initialized.main_commit).unwrap();
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();

        let checked = check_branches_with_hook(&cf, &mut blockers, &mut warnings, || {
            let mut bytes = fs::read(&commit_path)?;
            bytes.push(b' ');
            fs::write(&commit_path, bytes)?;
            Ok(())
        })
        .unwrap();

        assert_eq!(checked, 0);
        assert!(warnings.is_empty());
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].kind, "invalid_branch_head");
        assert!(blockers[0]
            .message
            .contains("sealed validation subject changed"));
    }
}
