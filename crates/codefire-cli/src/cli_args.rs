//! Argument parsing for the commands dispatched from main.rs.

use crate::command_registry::supports_metrics;
use crate::pagination::{parse_page_limit, PageRequest};
use crate::remote::{
    RemoteProjectOptions, RequestApplyOptions, RequestMergeOptions, RequestReviewOptions,
    UploadOptions,
};
use crate::view::{
    DiffAlgorithm, DiffOptions, PatchExportOptions, ReviewPackOptions,
    DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES, DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES,
};
use crate::{
    is_remote_project_url, open_next_step, BranchShowOptions, CliError, CloneOptions, CloseOptions,
    CommitOptions, DiffArgs, ExtinguishOptions, InitOptions, ListOptions, ListTarget, LockOptions,
    MergeOptions, OpenOptions, PatchExportArgs, PatchImportOptions, PathJsonOptions,
    ReadOnlyDebugOptions, ReviewPackArgs, ServeOptions, ShowOptions,
};
use std::env;
use std::path::PathBuf;

pub(crate) fn parse_show_args(args: &[String]) -> Result<ShowOptions, CliError> {
    let mut target = None;
    let mut json_output = false;
    for value in args {
        match value.as_str() {
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported show option: {option}"
                )));
            }
            value if target.is_none() => target = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected show argument: {value}"
                )));
            }
        }
    }
    Ok(ShowOptions {
        target: target.ok_or_else(|| {
            CliError::Usage("usage: codefire show <branch-or-commit> [--json]".to_string())
        })?,
        json_output,
    })
}

pub(crate) fn parse_list_args(args: &[String]) -> Result<ListOptions, CliError> {
    let mut positional = Vec::new();
    let mut json_output = false;
    let mut page = PageRequest::default();
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--limit" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--limit requires a value".to_string()))?;
                page.limit = Some(parse_page_limit(raw)?);
            }
            value if value.starts_with("--limit=") => {
                page.limit = Some(parse_page_limit(value.trim_start_matches("--limit="))?);
            }
            "--cursor" => {
                index += 1;
                page.cursor = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--cursor requires a value".to_string()))?
                        .clone(),
                );
            }
            value if value.starts_with("--cursor=") => {
                page.cursor = Some(value.trim_start_matches("--cursor=").to_string());
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported list option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    let target = match positional.as_slice() {
        [] => ListTarget::Local(env::current_dir()?),
        [value] if is_remote_project_url(value) => ListTarget::Remote(value.clone()),
        [value] => ListTarget::Local(PathBuf::from(value)),
        _ => {
            return Err(CliError::Usage(
                "usage: codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]".to_string(),
            ));
        }
    };
    Ok(ListOptions {
        target,
        json_output,
        page,
    })
}

pub(crate) fn parse_branch_show_args(args: &[String]) -> Result<BranchShowOptions, CliError> {
    let mut path = None;
    let mut branch = None;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--path" => {
                index += 1;
                let path_value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                set_single_path(&mut path, path_value)?;
            }
            "--metrics" => {
                return Err(CliError::Usage(
                    "branch show does not support --metrics".to_string(),
                ));
            }
            value if value.starts_with("--path=") => {
                set_single_path(&mut path, value.trim_start_matches("--path="))?;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported branch show option: {option}"
                )));
            }
            value if branch.is_none() => branch = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected branch show argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(BranchShowOptions {
        path: path.unwrap_or(env::current_dir()?),
        branch,
        json_output,
    })
}

pub(crate) fn parse_read_only_debug_args(
    args: &[String],
    command: &str,
) -> Result<ReadOnlyDebugOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--path" => {
                index += 1;
                let path_value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                set_single_path(&mut path, path_value)?;
            }
            "--metrics" => {
                return Err(CliError::Usage(format!(
                    "{command} does not support --metrics"
                )));
            }
            value if value.starts_with("--path=") => {
                set_single_path(&mut path, value.trim_start_matches("--path="))?;
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {value}"
                )));
            }
            value => set_single_path(&mut path, value)?,
        }
        index += 1;
    }
    Ok(ReadOnlyDebugOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
    })
}

pub(crate) fn set_single_path(path: &mut Option<PathBuf>, value: &str) -> Result<(), CliError> {
    if path.is_some() {
        return Err(CliError::Usage(
            "path may only be specified once".to_string(),
        ));
    }
    *path = Some(PathBuf::from(value));
    Ok(())
}

pub(crate) fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
    let mut path = None;
    let mut force = false;
    for arg in args {
        if arg == "--force" {
            force = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err(CliError::Usage(format!("unexpected init argument: {arg}")));
        }
    }
    Ok(InitOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        force,
    })
}

pub(crate) fn parse_open_args(args: &[String]) -> Result<OpenOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported open option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [branch, path] => Ok(OpenOptions {
            branch: branch.to_string(),
            path: PathBuf::from(path),
            dry_run,
            json_output,
            lock,
            idempotency_key,
        }),
        [branch] => Err(CliError::Usage(format!(
            "open needs the directory to work in: codefire open {branch} <path>{}",
            open_next_step(branch)
                .map(|line| format!("\n{line}"))
                .unwrap_or_default()
        ))),
        _ => Err(CliError::Usage(
            "usage: codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

pub(crate) fn parse_path_json_args(
    args: &[String],
    command: &str,
) -> Result<PathJsonOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut metrics = false;
    let mut full = false;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if command == "scan" && parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        if parse_path_json_metrics_option(arg, &mut json_output, &mut metrics) {
            if arg == "--metrics" && !supports_metrics(command) {
                return Err(CliError::Usage(format!(
                    "{command} does not support --metrics"
                )));
            }
            index += 1;
            continue;
        }
        match arg.as_str() {
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--full" if command == "scan" => {
                full = true;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value if path.is_none() => {
                path = Some(PathBuf::from(value));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected {command} argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(PathJsonOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
        metrics,
        full,
        lock,
    })
}

pub(crate) fn parse_extinguish_args(args: &[String]) -> Result<ExtinguishOptions, CliError> {
    let mut path = None;
    let mut fire_id = None;
    let mut resolution = "addressed".to_string();
    let mut rationale = String::new();
    let mut evidence = String::new();
    let mut evidence_refs = Vec::new();
    let mut refresh = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut edit_rationale = false;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--resolution" => {
                index += 1;
                resolution = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--resolution requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--resolution=") => {
                resolution = value.trim_start_matches("--resolution=").to_string();
            }
            "--rationale" => {
                index += 1;
                rationale = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--rationale requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--rationale=") => {
                rationale = value.trim_start_matches("--rationale=").to_string();
            }
            "--evidence" => {
                index += 1;
                evidence = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--evidence requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--evidence=") => {
                evidence = value.trim_start_matches("--evidence=").to_string();
            }
            "--evidence-ref" => {
                index += 1;
                evidence_refs.push(
                    args.get(index)
                        .ok_or_else(|| {
                            CliError::Usage("--evidence-ref requires a value".to_string())
                        })?
                        .to_string(),
                );
            }
            "--refresh" => refresh = true,
            "--edit-rationale" => edit_rationale = true,
            value if value.starts_with("--evidence-ref=") => {
                evidence_refs.push(value.trim_start_matches("--evidence-ref=").to_string());
            }
            value if fire_id.is_none() => fire_id = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected extinguish argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(ExtinguishOptions {
        path: path.unwrap_or(env::current_dir()?),
        fire_id: fire_id.ok_or_else(|| {
            CliError::Usage("usage: codefire extinguish <fire-id> [--path <open-dir>] [--refresh] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>)".to_string())
        })?,
        resolution,
        rationale,
        evidence,
        evidence_refs,
        refresh,
        dry_run,
        json_output,
        lock,
        idempotency_key,
        edit_rationale,
    })
}

pub(crate) fn parse_commit_args(args: &[String]) -> Result<CommitOptions, CliError> {
    let mut path = None;
    let mut message = None;
    let mut dry_run = false;
    let mut full_output = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut signer = None;
    let mut key_id = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--path" => {
                path = Some(PathBuf::from(take_option_value(
                    args, &mut index, "--path",
                )?));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--full" => {
                full_output = true;
            }
            "-m" | "--message" => {
                message = Some(take_option_value(args, &mut index, "-m")?);
            }
            "--signer" => {
                signer = Some(take_option_value(args, &mut index, "--signer")?);
            }
            value if value.starts_with("--signer=") => {
                signer = Some(value.trim_start_matches("--signer=").to_string());
            }
            "--key-id" => {
                key_id = Some(take_option_value(args, &mut index, "--key-id")?);
            }
            value if value.starts_with("--key-id=") => {
                key_id = Some(value.trim_start_matches("--key-id=").to_string());
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected commit argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(CommitOptions {
        path: path.unwrap_or(env::current_dir()?),
        message: message.unwrap_or_else(|| "CodeFire commit".to_string()),
        dry_run,
        full_output,
        json_output,
        lock,
        idempotency_key,
        signer,
        key_id,
    })
}

pub(crate) fn parse_clone_args(args: &[String]) -> Result<CloneOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported clone option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [source, new_branch] => Ok(CloneOptions {
            source: source.to_string(),
            new_branch: new_branch.to_string(),
            dry_run,
            json_output,
            lock,
            idempotency_key,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

pub(crate) fn parse_close_args(
    args: &[String],
    discard_command: bool,
) -> Result<CloseOptions, CliError> {
    let mut branch = None;
    let mut discard = discard_command;
    let mut keep_dir = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--discard" if !discard_command => discard = true,
            "--discard" => {
                return Err(CliError::Usage(
                    "discard command does not accept --discard".to_string(),
                ));
            }
            "--keep-dir" => keep_dir = true,
            option if option.starts_with("--") => {
                let command = if discard_command { "discard" } else { "close" };
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value if branch.is_none() => branch = Some(value.to_string()),
            value => {
                let command = if discard_command { "discard" } else { "close" };
                return Err(CliError::Usage(format!(
                    "unexpected {command} argument: {value}"
                )));
            }
        }
        index += 1;
    }
    if idempotency_key.is_some() {
        let command = if discard_command { "discard" } else { "close" };
        return Err(CliError::Usage(format!(
            "{command} does not support --idempotency-key"
        )));
    }
    let usage = if discard_command {
        "usage: codefire discard <branch> [--keep-dir] [--dry-run] [--json]"
    } else {
        "usage: codefire close <branch> [--discard] [--keep-dir] [--dry-run] [--json]"
    };
    Ok(CloseOptions {
        branch: branch.ok_or_else(|| CliError::Usage(usage.to_string()))?,
        discard,
        keep_dir,
        dry_run,
        json_output,
        lock,
    })
}

pub(crate) fn parse_diff_args(args: &[String]) -> Result<DiffArgs, CliError> {
    let mut positional = Vec::new();
    let mut algorithm = DiffAlgorithm::Myers;
    let mut context_lines = DiffOptions::default().context_lines;
    let mut rename_detection = false;
    let mut atom_diff = false;
    let mut trace_diff = false;
    let mut impact_diff = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--algorithm" {
            index += 1;
            let name = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--algorithm requires a value".to_string()))?;
            algorithm = parse_diff_algorithm(name)?;
        } else if let Some(name) = value.strip_prefix("--algorithm=") {
            algorithm = parse_diff_algorithm(name)?;
        } else if value == "--context" {
            index += 1;
            let raw = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--context requires a value".to_string()))?;
            context_lines = parse_diff_context(raw)?;
        } else if let Some(raw) = value.strip_prefix("--context=") {
            context_lines = parse_diff_context(raw)?;
        } else if value == "--rename-detection" {
            rename_detection = true;
        } else if value == "--atoms" {
            atom_diff = true;
        } else if value == "--trace" {
            trace_diff = true;
        } else if value == "--impact" {
            impact_diff = true;
        } else if value == "--json" {
            json_output = true;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!("unsupported diff option: {value}")));
        } else {
            positional.push(value.clone());
        }
        index += 1;
    }
    match positional.as_slice() {
        [left, right] => Ok(DiffArgs {
            left: left.clone(),
            right: right.clone(),
            diff: DiffOptions {
                algorithm,
                context_lines,
                rename_detection,
                atom_diff,
                trace_diff,
                impact_diff,
                json_output,
            },
        }),
        _ => Err(CliError::Usage(
            "usage: codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <left> <right>".to_string(),
        )),
    }
}

pub(crate) fn parse_diff_context(value: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::Usage("--context requires a non-negative integer".to_string()))
}

pub(crate) fn parse_diff_algorithm(value: &str) -> Result<DiffAlgorithm, CliError> {
    DiffAlgorithm::parse(value).ok_or_else(|| {
        CliError::Usage("--algorithm must be one of: myers, patience, histogram".to_string())
    })
}

pub(crate) fn parse_review_pack_args(args: &[String]) -> Result<ReviewPackArgs, CliError> {
    let mut source = None;
    let mut base = None;
    let mut output = None;
    let mut json_output = false;
    let mut algorithm = DiffAlgorithm::Myers;
    let mut rename_detection = true;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--base" {
            index += 1;
            base = Some(
                args.get(index)
                    .ok_or_else(|| CliError::Usage("--base requires a value".to_string()))?
                    .to_string(),
            );
        } else if value == "--output" {
            index += 1;
            output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                CliError::Usage("--output requires a value".to_string())
            })?));
        } else if value == "--json" {
            json_output = true;
        } else if value == "--algorithm" {
            index += 1;
            let name = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--algorithm requires a value".to_string()))?;
            algorithm = parse_diff_algorithm(name)?;
        } else if let Some(name) = value.strip_prefix("--algorithm=") {
            algorithm = parse_diff_algorithm(name)?;
        } else if value == "--rename-detection" {
            rename_detection = true;
        } else if value == "--no-rename-detection" {
            rename_detection = false;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!(
                "unsupported review-pack option: {value}"
            )));
        } else if source.is_none() {
            source = Some(value.clone());
        } else {
            return Err(CliError::Usage(format!(
                "unexpected review-pack argument: {value}"
            )));
        }
        index += 1;
    }
    Ok(ReviewPackArgs {
        review: ReviewPackOptions {
            source: source.ok_or_else(|| {
                CliError::Usage(
                    "usage: codefire review-pack <source> [--base <base>] [--output <path>] [--algorithm myers|patience|histogram] [--no-rename-detection]"
                        .to_string(),
                )
            })?,
            base,
            algorithm,
            rename_detection,
        },
        output,
        json_output,
    })
}

pub(crate) fn parse_patch_export_args(args: &[String]) -> Result<PatchExportArgs, CliError> {
    let mut source = None;
    let mut base = None;
    let mut output = None;
    let mut json_output = false;
    let mut max_file_bytes = DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES;
    let mut max_payload_bytes = DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES;
    let mut include_large_files = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--base" {
            index += 1;
            base = Some(
                args.get(index)
                    .ok_or_else(|| CliError::Usage("--base requires a value".to_string()))?
                    .to_string(),
            );
        } else if value == "--output" {
            index += 1;
            output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                CliError::Usage("--output requires a value".to_string())
            })?));
        } else if value == "--json" {
            json_output = true;
        } else if value == "--include-large-files" {
            include_large_files = true;
        } else if value == "--max-file-bytes" {
            index += 1;
            max_file_bytes = parse_usize_arg(
                args.get(index).ok_or_else(|| {
                    CliError::Usage("--max-file-bytes requires a value".to_string())
                })?,
                "--max-file-bytes",
            )?;
        } else if let Some(bytes) = value.strip_prefix("--max-file-bytes=") {
            max_file_bytes = parse_usize_arg(bytes, "--max-file-bytes")?;
        } else if value == "--max-payload-bytes" {
            index += 1;
            max_payload_bytes = parse_usize_arg(
                args.get(index).ok_or_else(|| {
                    CliError::Usage("--max-payload-bytes requires a value".to_string())
                })?,
                "--max-payload-bytes",
            )?;
        } else if let Some(bytes) = value.strip_prefix("--max-payload-bytes=") {
            max_payload_bytes = parse_usize_arg(bytes, "--max-payload-bytes")?;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!(
                "unsupported patch export option: {value}"
            )));
        } else if source.is_none() {
            source = Some(value.clone());
        } else {
            return Err(CliError::Usage(format!(
                "unexpected patch export argument: {value}"
            )));
        }
        index += 1;
    }
    Ok(PatchExportArgs {
        patch: PatchExportOptions {
            source: source.ok_or_else(|| {
                CliError::Usage(
                    "usage: codefire patch export <source> [--base <base>] [--output <path>]"
                        .to_string(),
                )
            })?,
            base,
            max_file_bytes,
            max_payload_bytes,
            include_large_files,
        },
        output,
        json_output,
    })
}

pub(crate) fn parse_usize_arg(value: &str, name: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::Usage(format!("{name} must be a non-negative integer")))
}

pub(crate) fn parse_patch_import_args(args: &[String]) -> Result<PatchImportOptions, CliError> {
    let mut path = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported patch import option: {value}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected patch import argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(PatchImportOptions {
        path: path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}

pub(crate) fn parse_upload_args(args: &[String]) -> Result<UploadOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [branch, remote_url] => Ok(UploadOptions {
            branch: branch.clone(),
            remote_url: remote_url.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire upload <branch> <cf-url> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

pub(crate) fn parse_remote_project_args(
    args: &[String],
    command: &str,
) -> Result<RemoteProjectOptions, CliError> {
    let mut positional = Vec::new();
    let mut json_output = false;
    let mut page = PageRequest::default();
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--limit" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--limit requires a value".to_string()))?;
                page.limit = Some(parse_page_limit(raw)?);
            }
            value if value.starts_with("--limit=") => {
                page.limit = Some(parse_page_limit(value.trim_start_matches("--limit="))?);
            }
            "--cursor" => {
                index += 1;
                page.cursor = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--cursor requires a value".to_string()))?
                        .clone(),
                );
            }
            value if value.starts_with("--cursor=") => {
                page.cursor = Some(value.trim_start_matches("--cursor=").to_string());
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [project_url] => Ok(RemoteProjectOptions {
            project_url: project_url.clone(),
            json_output,
            page,
        }),
        _ => Err(CliError::Usage(format!(
            "usage: codefire {command} <cf-project-url> [--json] [--limit N] [--cursor CURSOR]"
        ))),
    }
}

pub(crate) fn parse_request_merge_args(args: &[String]) -> Result<RequestMergeOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [source_url, target_url] => Ok(RequestMergeOptions {
            source_url: source_url.clone(),
            target_url: target_url.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

pub(crate) fn parse_request_review_args(args: &[String]) -> Result<RequestReviewOptions, CliError> {
    let mut positional = Vec::new();
    let mut reviewer = None;
    let mut decision = "approve".to_string();
    let mut comment = String::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            "--reviewer" => {
                index += 1;
                reviewer = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--reviewer requires a value".to_string()))?
                        .to_string(),
                );
            }
            "--decision" => {
                index += 1;
                decision = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--decision requires a value".to_string()))?
                    .to_string();
            }
            "--comment" => {
                index += 1;
                comment = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--comment requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    if decision != "approve" && decision != "reject" {
        return Err(CliError::Usage(
            "--decision must be approve or reject".to_string(),
        ));
    }
    match positional.as_slice() {
        [project_url, mr_id] => Ok(RequestReviewOptions {
            project_url: project_url.clone(),
            mr_id: mr_id.clone(),
            reviewer: reviewer.unwrap_or_else(|| "reviewer".to_string()),
            decision,
            comment,
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-review <project-url> <mr-id> [--reviewer <name>] [--decision approve|reject] [--comment <text>] [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

pub(crate) fn parse_request_apply_args(args: &[String]) -> Result<RequestApplyOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [project_url, mr_id] => Ok(RequestApplyOptions {
            project_url: project_url.clone(),
            mr_id: mr_id.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-apply <project-url> <mr-id> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

pub(crate) fn parse_serve_args(args: &[String]) -> Result<ServeOptions, CliError> {
    let mut storage_root = None;
    let mut host = "127.0.0.1".to_string();
    let mut port = 8080u16;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut tls_client_ca = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                index += 1;
                host = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--host requires a value".to_string()))?
                    .to_string();
            }
            "--port" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--port requires a value".to_string()))?;
                port = value.parse::<u16>().map_err(|_| {
                    CliError::Usage("--port must be an integer from 0 to 65535".to_string())
                })?;
            }
            "--tls-cert" => {
                index += 1;
                tls_cert = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-cert requires a value".to_string())
                })?));
            }
            "--tls-key" => {
                index += 1;
                tls_key = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-key requires a value".to_string())
                })?));
            }
            "--tls-client-ca" => {
                index += 1;
                tls_client_ca = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-client-ca requires a value".to_string())
                })?));
            }
            value if storage_root.is_none() => storage_root = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected serve argument: {value}"
                )))
            }
        }
        index += 1;
    }
    if tls_cert.is_some() != tls_key.is_some() {
        return Err(CliError::Usage(
            "--tls-cert and --tls-key must be provided together".to_string(),
        ));
    }
    if tls_client_ca.is_some() && tls_cert.is_none() {
        return Err(CliError::Usage(
            "--tls-client-ca requires --tls-cert and --tls-key".to_string(),
        ));
    }
    Ok(ServeOptions {
        storage_root: storage_root.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>] [--tls-client-ca <ca-pem>]"
                    .to_string(),
            )
        })?,
        host,
        port,
        tls_cert,
        tls_key,
        tls_client_ca,
    })
}

pub(crate) fn skip_ignored_option(args: &[String], index: &mut usize) -> Result<(), CliError> {
    match args[*index].as_str() {
        "--actor" | "--token" | "--request-key-id" | "--request-key" => {
            *index += 1;
            if args.get(*index).is_none() {
                return Err(CliError::Usage(format!(
                    "{} requires a value",
                    args[*index - 1]
                )));
            }
            Ok(())
        }
        "--dry-run" => Ok(()),
        option => Err(CliError::Usage(format!("unsupported option: {option}"))),
    }
}

pub(crate) fn parse_common_mutation_option(
    args: &[String],
    index: &mut usize,
    dry_run: &mut bool,
    json_output: &mut bool,
    lock: &mut LockOptions,
    idempotency_key: &mut Option<String>,
) -> Result<bool, CliError> {
    if parse_lock_option(args, index, lock)? {
        return Ok(true);
    }
    match args[*index].as_str() {
        "--dry-run" => {
            *dry_run = true;
            Ok(true)
        }
        "--json" => {
            *json_output = true;
            Ok(true)
        }
        "--idempotency-key" => {
            *idempotency_key = Some(take_option_value(args, index, "--idempotency-key")?);
            Ok(true)
        }
        value if value.starts_with("--idempotency-key=") => {
            *idempotency_key = Some(value.trim_start_matches("--idempotency-key=").to_string());
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(crate) fn parse_path_json_metrics_option(
    arg: &str,
    json_output: &mut bool,
    metrics: &mut bool,
) -> bool {
    match arg {
        "--json" => {
            *json_output = true;
            true
        }
        "--metrics" => {
            *metrics = true;
            true
        }
        _ => false,
    }
}

pub(crate) fn take_option_value(
    args: &[String],
    index: &mut usize,
    option: &str,
) -> Result<String, CliError> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

pub(crate) fn parse_lock_option(
    args: &[String],
    index: &mut usize,
    lock: &mut LockOptions,
) -> Result<bool, CliError> {
    match args[*index].as_str() {
        "--wait-lock" => {
            lock.wait = true;
            Ok(true)
        }
        "--lock-timeout" => {
            *index += 1;
            let value = args
                .get(*index)
                .ok_or_else(|| CliError::Usage("--lock-timeout requires a value".to_string()))?;
            lock.timeout_ms = Some(parse_lock_timeout_ms(value)?);
            lock.wait = true;
            Ok(true)
        }
        value if value.starts_with("--lock-timeout=") => {
            lock.timeout_ms = Some(parse_lock_timeout_ms(
                value.trim_start_matches("--lock-timeout="),
            )?);
            lock.wait = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(crate) fn parse_lock_timeout_ms(value: &str) -> Result<u64, CliError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CliError::Usage(
            "--lock-timeout requires a duration".to_string(),
        ));
    }
    if let Some(ms) = value.strip_suffix("ms") {
        return parse_u64_duration(ms, value);
    }
    if let Some(seconds) = value.strip_suffix('s') {
        return parse_u64_duration(seconds, value).map(|seconds| seconds.saturating_mul(1000));
    }
    parse_u64_duration(value, value).map(|seconds| seconds.saturating_mul(1000))
}

pub(crate) fn parse_u64_duration(raw: &str, display: &str) -> Result<u64, CliError> {
    raw.parse::<u64>().map_err(|_| {
        CliError::Usage(format!(
            "invalid --lock-timeout duration: {display}; use <seconds>, <seconds>s, or <milliseconds>ms"
        ))
    })
}

pub(crate) fn parse_merge_args(args: &[String]) -> Result<MergeOptions, CliError> {
    let mut source_branch = None;
    let mut target_branch = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--into" => {
                target_branch = Some(take_option_value(args, &mut index, "--into")?);
            }
            value if source_branch.is_none() => source_branch = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected merge argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(MergeOptions {
        source_branch: source_branch.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                    .to_string(),
            )
        })?,
        target_branch: target_branch.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}
