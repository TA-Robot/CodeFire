use super::automation::{verification_diagnostics_json, verification_next_actions};
use super::context::{build_context_pack, ContextOptions, ContextSelector};
use super::storage::{
    run_storage_report, storage_report_diagnostics_json, storage_report_next_actions,
    StorageReportOptions,
};
use super::{compute_scan, compute_verify, open_context, CliError};
use serde_json::{json, Value};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct ExplainOptions {
    pub(crate) path: PathBuf,
    pub(crate) target: ExplainTarget,
    pub(crate) json_output: bool,
    pub(crate) depth: usize,
    pub(crate) large_threshold_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExplainTarget {
    Fire(String),
    Atom(String),
    VerifyFailure,
    StorageWarning,
}

pub(crate) struct ExplainResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) data: Value,
    pub(crate) diagnostics: Vec<Value>,
    pub(crate) next_actions: Vec<Value>,
}

pub(crate) fn parse_explain_args(args: &[String]) -> Result<ExplainOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut depth = 1usize;
    let mut large_threshold_bytes = 1_048_576u64;
    let mut positional = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--depth" => {
                index += 1;
                depth = required_arg(args, index, "--depth")?
                    .parse::<usize>()
                    .map_err(|_| {
                        CliError::Usage("--depth must be a non-negative integer".to_string())
                    })?;
            }
            "--large-threshold" => {
                index += 1;
                large_threshold_bytes =
                    parse_byte_size(required_arg(args, index, "--large-threshold")?)?;
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--depth=") => {
                depth = value
                    .trim_start_matches("--depth=")
                    .parse::<usize>()
                    .map_err(|_| {
                        CliError::Usage("--depth must be a non-negative integer".to_string())
                    })?;
            }
            value if value.starts_with("--large-threshold=") => {
                large_threshold_bytes =
                    parse_byte_size(value.trim_start_matches("--large-threshold="))?;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported explain option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    let target = match positional.as_slice() {
        [kind, value] if kind == "fire" => ExplainTarget::Fire(value.clone()),
        [kind, value] if kind == "atom" => ExplainTarget::Atom(value.clone()),
        [kind] if kind == "verify-failure" => ExplainTarget::VerifyFailure,
        [kind] if kind == "storage-warning" => ExplainTarget::StorageWarning,
        _ => {
            return Err(CliError::Usage(
                "usage: codefire-rs explain (fire <id>|atom <id>|verify-failure|storage-warning) [--path <path>] [--json]".to_string(),
            ))
        }
    };
    Ok(ExplainOptions {
        path: path.unwrap_or(env::current_dir()?),
        target,
        json_output,
        depth,
        large_threshold_bytes,
    })
}

pub(crate) fn run_explain(options: &ExplainOptions) -> Result<ExplainResult, CliError> {
    match &options.target {
        ExplainTarget::Fire(value) => explain_fire(options, value),
        ExplainTarget::Atom(value) => explain_atom(options, value),
        ExplainTarget::VerifyFailure => explain_verify_failure(options),
        ExplainTarget::StorageWarning => explain_storage_warning(options),
    }
}

pub(crate) fn print_explain_result(result: &ExplainResult) {
    println!(
        "Explain: {}",
        result.data["target"]["kind"]
            .as_str()
            .unwrap_or("(unknown)")
    );
    if let Some(summary) = result.data["summary"].as_str() {
        println!("{summary}");
    }
    if !result.diagnostics.is_empty() {
        println!("Diagnostics: {}", result.diagnostics.len());
    }
    if !result.next_actions.is_empty() {
        println!("Next actions:");
        for action in &result.next_actions {
            println!(
                "  {}: {}",
                action["id"].as_str().unwrap_or("(action)"),
                action["command"].as_str().unwrap_or("")
            );
        }
    }
}

fn explain_fire(options: &ExplainOptions, value: &str) -> Result<ExplainResult, CliError> {
    let scan_execution = compute_scan(&options.path, false)?;
    let fire = scan_execution
        .scan
        .open_fires
        .iter()
        .find(|fire| fire.display_id == value || fire.fire_uid == value)
        .ok_or_else(|| CliError::Usage(format!("unknown fire: {value}")))?;
    let data = json!({
        "type": "codefire_explain",
        "version": 1,
        "target": {"kind": "fire", "value": value},
        "summary": format!("{} links {} to {} because {}", fire.display_id, fire.source.atom_id, fire.target.atom_id, fire.reason),
        "fire": {
            "display_id": &fire.display_id,
            "fire_uid": &fire.fire_uid,
            "severity": &fire.severity,
            "reason": &fire.reason,
            "source_atom": &fire.source.atom_id,
            "target_atom": &fire.target.atom_id,
            "trace_path": &fire.trace_path,
        },
    });
    Ok(ExplainResult {
        repo_root: scan_execution.context.repo_root,
        diagnostics: vec![json!({
            "kind": "open_fire",
            "display_id": &fire.display_id,
            "source_atom": &fire.source.atom_id,
            "target_atom": &fire.target.atom_id,
            "reason": &fire.reason,
        })],
        next_actions: vec![
            next_action(
                "context_fire",
                format!("codefire-rs context --fire {} --json", fire.display_id),
                "inspect source, target, and trace context for this fire",
                json!({"display_id": &fire.display_id}),
            ),
            next_action(
                "extinguish_fire",
                format!(
                    "codefire-rs extinguish {} --resolution <type> --rationale <text>",
                    fire.display_id
                ),
                "record the resolution when the fire is addressed",
                json!({"display_id": &fire.display_id}),
            ),
        ],
        data,
    })
}

fn explain_atom(options: &ExplainOptions, value: &str) -> Result<ExplainResult, CliError> {
    let pack = build_context_pack(&ContextOptions {
        path: options.path.clone(),
        selector: ContextSelector::Atom(value.to_string()),
        depth: options.depth,
        json_output: true,
    })?;
    let atoms = pack.data["atoms"].as_array().map(Vec::len).unwrap_or(0);
    let fires = pack.data["fires"].as_array().map(Vec::len).unwrap_or(0);
    let data = json!({
        "type": "codefire_explain",
        "version": 1,
        "target": {"kind": "atom", "value": value},
        "summary": format!("{value} has {atoms} atom(s) in context and {fires} related open fire(s)"),
        "context": pack.data,
    });
    Ok(ExplainResult {
        repo_root: pack.repo_root,
        diagnostics: Vec::new(),
        next_actions: vec![next_action(
            "context_atom",
            format!(
                "codefire-rs context --atom {value} --depth {} --json",
                options.depth
            ),
            "inspect neighboring trace context for this atom",
            json!({"atom_id": value, "depth": options.depth}),
        )],
        data,
    })
}

fn explain_verify_failure(options: &ExplainOptions) -> Result<ExplainResult, CliError> {
    let context = open_context(&options.path)?;
    let execution = compute_verify(&options.path, false)?;
    let verification = execution.verification;
    let blocker_count = verification.open_required_fires
        + verification.missing_required_links.len()
        + verification.stale_resolutions.len()
        + verification.duplicate_atom_ids.len()
        + verification.failed_checks.len();
    let data = json!({
        "type": "codefire_explain",
        "version": 1,
        "target": {"kind": "verify-failure", "value": null},
        "summary": format!("verification result is {} with {blocker_count} blocker(s)", verification.result),
        "verification": super::automation::verification_data_json(&verification),
    });
    Ok(ExplainResult {
        repo_root: context.repo_root,
        diagnostics: verification_diagnostics_json(&verification),
        next_actions: verification_next_actions(&verification),
        data,
    })
}

fn explain_storage_warning(options: &ExplainOptions) -> Result<ExplainResult, CliError> {
    let report = run_storage_report(&StorageReportOptions {
        start: options.path.clone(),
        json_output: true,
        large_threshold_bytes: options.large_threshold_bytes,
    })?;
    let data = json!({
        "type": "codefire_explain",
        "version": 1,
        "target": {"kind": "storage-warning", "value": null},
        "summary": format!("storage report has {} warning(s)", report.warnings.len()),
        "warnings": report.warnings.iter().map(|warning| {
            json!({
                "kind": &warning.kind,
                "message": &warning.message,
                "path": &warning.path,
                "bytes": warning.bytes,
            })
        }).collect::<Vec<_>>(),
        "largest_objects": report.largest_objects.iter().map(|object| {
            json!({
                "object_id": &object.object_id,
                "type": &object.type_tag,
                "path": &object.path,
                "bytes": object.bytes,
            })
        }).collect::<Vec<_>>(),
        "external_artifacts": {
            "refs": report.external_artifacts.refs,
            "referenced_bytes": report.external_artifacts.referenced_bytes,
            "payload_bytes_stored": report.external_artifacts.payload_bytes_stored,
        },
    });
    Ok(ExplainResult {
        repo_root: report.repo_root.clone(),
        diagnostics: storage_report_diagnostics_json(&report),
        next_actions: storage_report_next_actions(&report),
        data,
    })
}

fn next_action(id: &str, command: impl Into<String>, description: &str, context: Value) -> Value {
    json!({
        "id": id,
        "command": command.into(),
        "description": description,
        "context": context,
    })
}

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn parse_byte_size(value: &str) -> Result<u64, CliError> {
    let trimmed = value.trim();
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split_at);
    if number.is_empty() {
        return Err(CliError::Usage(format!("invalid byte size: {value}")));
    }
    let base = number
        .parse::<u64>()
        .map_err(|_| CliError::Usage(format!("invalid byte size: {value}")))?;
    let multiplier = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        _ => {
            return Err(CliError::Usage(format!(
                "unsupported byte size unit: {value}"
            )))
        }
    };
    base.checked_mul(multiplier)
        .ok_or_else(|| CliError::Usage(format!("byte size is too large: {value}")))
}
