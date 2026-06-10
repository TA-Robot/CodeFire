use super::{
    parse_duration_ms, run_evidence_add, validate_evidence_add_options, EvidenceAddOptions,
    EvidenceAddResult, DEFAULT_MAX_OUTPUT_BYTES,
};
use crate::{find_repo_root, limited_yaml, read_batch_file, CliError};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const YAML_CONTEXT: &str = "evidence batch YAML";

#[derive(Debug)]
pub(crate) struct EvidenceBatchResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) item_count: usize,
    pub(crate) dry_run: bool,
    pub(crate) plan: Value,
    pub(crate) results: Vec<EvidenceAddResult>,
}

#[derive(Debug, Clone, Default)]
struct EvidenceBatchDefaults {
    label: Option<String>,
    command_cwd: Option<PathBuf>,
    command_timeout_ms: Option<u64>,
    max_output_bytes: Option<usize>,
}

#[derive(Debug, Clone, Default)]
struct EvidenceBatchItem {
    label: Option<String>,
    artifact_path: Option<PathBuf>,
    artifact_uri: Option<String>,
    command: Option<String>,
    command_argv: Option<Vec<String>>,
    command_cwd: Option<PathBuf>,
    command_timeout_ms: Option<u64>,
    max_output_bytes: Option<usize>,
}

#[derive(Debug)]
struct EvidenceBatchFile {
    version: u64,
    defaults: EvidenceBatchDefaults,
    items: Vec<EvidenceBatchItem>,
}

pub(crate) fn run_evidence_batch(
    options: &EvidenceAddOptions,
) -> Result<EvidenceBatchResult, CliError> {
    let batch_path = options
        .batch_path
        .as_ref()
        .ok_or_else(|| CliError::Usage("evidence batch requires --batch".to_string()))?;
    let repo_root = find_repo_root(&options.start)?;
    let batch = parse_evidence_batch_file(batch_path)?;
    if batch.version != 1 {
        return Err(CliError::Usage(format!(
            "unsupported evidence batch version: {}",
            batch.version
        )));
    }
    if batch.items.is_empty() {
        return Err(CliError::Usage(
            "evidence batch requires at least one item".to_string(),
        ));
    }

    let resolved = batch
        .items
        .iter()
        .map(|item| resolve_item(options, &batch.defaults, item))
        .collect::<Vec<_>>();
    for item in &resolved {
        validate_evidence_add_options(item)?;
    }

    let plan = evidence_batch_operation_plan(options, batch_path, &resolved);
    if options.dry_run {
        return Ok(EvidenceBatchResult {
            repo_root,
            item_count: resolved.len(),
            dry_run: true,
            plan,
            results: Vec::new(),
        });
    }

    let mut results = Vec::with_capacity(resolved.len());
    for item in &resolved {
        results.push(run_evidence_add(item)?);
    }
    Ok(EvidenceBatchResult {
        repo_root,
        item_count: results.len(),
        dry_run: false,
        plan,
        results,
    })
}

pub(crate) fn evidence_batch_data_json(result: &EvidenceBatchResult) -> Value {
    json!({
        "type": "codefire_evidence_batch_result",
        "version": 1,
        "dry_run": result.dry_run,
        "item_count": result.item_count,
        "plan": &result.plan,
        "results": result.results.iter().map(evidence_result_json).collect::<Vec<_>>(),
    })
}

fn evidence_result_json(result: &EvidenceAddResult) -> Value {
    let evidence_id = if result.evidence_id.is_empty() {
        Value::Null
    } else {
        Value::String(result.evidence_id.clone())
    };
    json!({
        "dry_run": result.dry_run,
        "created": !result.dry_run,
        "evidence_id": evidence_id,
        "artifact_ref_id": &result.artifact_ref_id,
        "object_path": &result.object_path,
        "command_exit_code": result.command_exit_code,
        "command_timed_out": result.command_timed_out,
        "command_stdout_summary": &result.command_stdout_summary,
        "command_stdout_truncated": result.command_stdout_truncated,
        "command_stderr_summary": &result.command_stderr_summary,
        "command_stderr_truncated": result.command_stderr_truncated,
        "diagnostics": &result.diagnostics,
    })
}

fn resolve_item(
    options: &EvidenceAddOptions,
    defaults: &EvidenceBatchDefaults,
    item: &EvidenceBatchItem,
) -> EvidenceAddOptions {
    EvidenceAddOptions {
        start: options.start.clone(),
        json_output: false,
        dry_run: false,
        batch_path: None,
        label: item.label.clone().or_else(|| defaults.label.clone()),
        artifact_path: item.artifact_path.clone(),
        artifact_uri: item.artifact_uri.clone(),
        command: item.command.clone(),
        command_argv: item.command_argv.clone(),
        command_cwd: item
            .command_cwd
            .clone()
            .or_else(|| defaults.command_cwd.clone()),
        command_timeout_ms: item
            .command_timeout_ms
            .or(defaults.command_timeout_ms)
            .unwrap_or(options.command_timeout_ms),
        max_output_bytes: item
            .max_output_bytes
            .or(defaults.max_output_bytes)
            .unwrap_or(DEFAULT_MAX_OUTPUT_BYTES),
        allow_failed_command: options.allow_failed_command,
    }
}

fn evidence_batch_operation_plan(
    options: &EvidenceAddOptions,
    batch_path: &Path,
    items: &[EvidenceAddOptions],
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "evidence-add-batch",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "batch_file": batch_path,
        "item_count": items.len(),
        "items": items.iter().map(evidence_batch_item_plan).collect::<Vec<_>>(),
        "operations": [
            {"kind": "validate_all_batch_items", "count": items.len()},
            {"kind": "store_artifact_refs"},
            {"kind": "store_evidence_objects", "count": items.len()},
        ],
        "next_actions": [
            {"kind": "storage_report", "command": "codefire storage report --json"},
            {"kind": "verify", "command": "codefire verify --details --json"},
        ],
    })
}

fn evidence_batch_item_plan(item: &EvidenceAddOptions) -> Value {
    json!({
        "label": &item.label,
        "has_artifact": item.artifact_path.is_some(),
        "artifact": &item.artifact_path,
        "artifact_uri": &item.artifact_uri,
        "has_command": item.command.is_some(),
        "command": &item.command,
        "has_command_argv": item.command_argv.is_some(),
        "command_argv": &item.command_argv,
        "cwd": &item.command_cwd,
        "timeout_ms": item.command_timeout_ms,
        "max_output_bytes": item.max_output_bytes,
    })
}

fn parse_evidence_batch_file(path: &Path) -> Result<EvidenceBatchFile, CliError> {
    let text = read_batch_file(path, "evidence batch")?;
    if starts_json_document(&text) {
        parse_evidence_batch_json(&text)
    } else {
        parse_evidence_batch_yaml(&text)
    }
}

fn parse_evidence_batch_json(text: &str) -> Result<EvidenceBatchFile, CliError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| CliError::Usage(format!("invalid evidence batch JSON: {error}")))?;
    let object = value.as_object().ok_or_else(|| {
        CliError::Usage(
            "evidence batch JSON must be an object with version and items array; top-level arrays are not supported. Use {\"version\":1,\"items\":[...]} wrapper".to_string(),
        )
    })?;
    let version = object.get("version").and_then(Value::as_u64).unwrap_or(0);
    let defaults = parse_json_defaults(object.get("defaults"))?;
    let items = object
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("evidence batch JSON missing items array".to_string()))?
        .iter()
        .map(parse_json_item)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EvidenceBatchFile {
        version,
        defaults,
        items,
    })
}

fn starts_json_document(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn parse_json_defaults(value: Option<&Value>) -> Result<EvidenceBatchDefaults, CliError> {
    let Some(value) = value else {
        return Ok(EvidenceBatchDefaults::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("evidence batch defaults must be an object".to_string()))?;
    Ok(EvidenceBatchDefaults {
        label: optional_json_string(object.get("label"), "defaults.label")?,
        command_cwd: optional_json_path(object.get("cwd"), "defaults.cwd")?,
        command_timeout_ms: optional_json_duration_ms(
            object.get("timeout_ms"),
            "defaults.timeout_ms",
        )?,
        max_output_bytes: optional_json_usize(
            object.get("max_output_bytes"),
            "defaults.max_output_bytes",
        )?,
    })
}

fn parse_json_item(value: &Value) -> Result<EvidenceBatchItem, CliError> {
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("evidence batch item must be an object".to_string()))?;
    Ok(EvidenceBatchItem {
        label: optional_json_string(object.get("label"), "item.label")?,
        artifact_path: optional_json_path(object.get("artifact"), "item.artifact")?,
        artifact_uri: optional_json_string(object.get("artifact_uri"), "item.artifact_uri")?,
        command: optional_json_string(object.get("from_command"), "item.from_command")?,
        command_argv: optional_json_string_array(object.get("from_argv"), "item.from_argv")?,
        command_cwd: optional_json_path(object.get("cwd"), "item.cwd")?,
        command_timeout_ms: optional_json_duration_ms(object.get("timeout_ms"), "item.timeout_ms")?,
        max_output_bytes: optional_json_usize(
            object.get("max_output_bytes"),
            "item.max_output_bytes",
        )?,
    })
}

fn optional_json_string_array(
    value: Option<&Value>,
    field: &str,
) -> Result<Option<Vec<String>>, CliError> {
    value
        .map(|value| {
            let items = value
                .as_array()
                .ok_or_else(|| CliError::Usage(format!("{field} must be a string array")))?;
            if items.is_empty() {
                return Err(CliError::Usage(format!("{field} must not be empty")));
            }
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| CliError::Usage(format!("{field} must be a string array")))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
}

fn optional_json_string(value: Option<&Value>, field: &str) -> Result<Option<String>, CliError> {
    value
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| CliError::Usage(format!("{field} must be a string")))
        })
        .transpose()
}

fn optional_json_path(value: Option<&Value>, field: &str) -> Result<Option<PathBuf>, CliError> {
    Ok(optional_json_string(value, field)?.map(PathBuf::from))
}

fn optional_json_usize(value: Option<&Value>, field: &str) -> Result<Option<usize>, CliError> {
    value
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| CliError::Usage(format!("{field} must be an unsigned integer")))
        })
        .transpose()
}

fn optional_json_duration_ms(value: Option<&Value>, field: &str) -> Result<Option<u64>, CliError> {
    value
        .map(|value| {
            value
                .as_u64()
                .filter(|value| *value > 0)
                .ok_or_else(|| CliError::Usage(format!("{field} must be a positive integer")))
        })
        .transpose()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceBatchYamlSection {
    Root,
    Defaults,
    Items,
}

fn parse_evidence_batch_yaml(text: &str) -> Result<EvidenceBatchFile, CliError> {
    let mut version = None;
    let mut defaults = EvidenceBatchDefaults::default();
    let mut items = Vec::new();
    let mut current_item = None;
    let mut section = EvidenceBatchYamlSection::Root;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        if raw_line.contains('\t') {
            return Err(CliError::Usage(format!(
                "invalid evidence batch YAML: tabs are not supported at line {line_number}"
            )));
        }
        let without_comment = limited_yaml::strip_comment(raw_line);
        if without_comment.trim().is_empty() {
            continue;
        }
        let indent = without_comment
            .as_bytes()
            .iter()
            .take_while(|byte| **byte == b' ')
            .count();
        let text = without_comment.trim();
        match (indent, text, section) {
            (0, "defaults:", _) => section = EvidenceBatchYamlSection::Defaults,
            (0, "items:", _) => {
                push_current_item(&mut items, &mut current_item);
                section = EvidenceBatchYamlSection::Items;
            }
            (0, _, _) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                if key == "version" {
                    version = Some(limited_yaml::parse_u64(
                        &value,
                        "version",
                        line_number,
                        YAML_CONTEXT,
                    )?);
                    section = EvidenceBatchYamlSection::Root;
                } else {
                    return Err(CliError::Usage(format!(
                        "invalid evidence batch YAML: unsupported root key '{key}' at line {line_number}"
                    )));
                }
            }
            (2, _, EvidenceBatchYamlSection::Defaults) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_default(&mut defaults, &key, value, line_number)?;
            }
            (2, _, EvidenceBatchYamlSection::Items) if text.starts_with("- ") => {
                push_current_item(&mut items, &mut current_item);
                let rest = text[2..].trim();
                let mut item = EvidenceBatchItem::default();
                if !rest.is_empty() {
                    let (key, value) =
                        limited_yaml::parse_key_value(rest, line_number, YAML_CONTEXT)?;
                    apply_yaml_item_field(&mut item, &key, value, line_number)?;
                }
                current_item = Some(item);
            }
            (4, _, EvidenceBatchYamlSection::Items) => {
                let item = current_item.as_mut().ok_or_else(|| {
                    CliError::Usage(format!(
                        "invalid evidence batch YAML: item field before list item at line {line_number}"
                    ))
                })?;
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_item_field(item, &key, value, line_number)?;
            }
            _ => {
                return Err(CliError::Usage(format!(
                    "invalid evidence batch YAML indentation or section at line {line_number}"
                )));
            }
        }
    }
    push_current_item(&mut items, &mut current_item);

    Ok(EvidenceBatchFile {
        version: version
            .ok_or_else(|| CliError::Usage("evidence batch YAML missing version".to_string()))?,
        defaults,
        items,
    })
}

fn push_current_item(
    items: &mut Vec<EvidenceBatchItem>,
    current_item: &mut Option<EvidenceBatchItem>,
) {
    if let Some(item) = current_item.take() {
        items.push(item);
    }
}

fn apply_yaml_default(
    defaults: &mut EvidenceBatchDefaults,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "label" => defaults.label = Some(value),
        "cwd" => defaults.command_cwd = Some(PathBuf::from(value)),
        "timeout_ms" => defaults.command_timeout_ms = Some(parse_duration_ms(&value)?),
        "max_output_bytes" => {
            defaults.max_output_bytes = Some(limited_yaml::parse_usize(
                &value,
                "max_output_bytes",
                line_number,
                YAML_CONTEXT,
            )?);
        }
        _ => {
            return Err(CliError::Usage(format!(
                "invalid evidence batch YAML: unsupported defaults key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn apply_yaml_item_field(
    item: &mut EvidenceBatchItem,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "label" => item.label = Some(value),
        "artifact" => item.artifact_path = Some(PathBuf::from(value)),
        "artifact_uri" => item.artifact_uri = Some(value),
        "from_command" => item.command = Some(value),
        "from_argv" => {
            item.command_argv = Some(parse_yaml_string_array(&value, "from_argv", line_number)?);
        }
        "cwd" => item.command_cwd = Some(PathBuf::from(value)),
        "timeout_ms" => item.command_timeout_ms = Some(parse_duration_ms(&value)?),
        "max_output_bytes" => {
            item.max_output_bytes = Some(limited_yaml::parse_usize(
                &value,
                "max_output_bytes",
                line_number,
                YAML_CONTEXT,
            )?);
        }
        _ => {
            return Err(CliError::Usage(format!(
                "invalid evidence batch YAML: unsupported item key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn parse_yaml_string_array(
    value: &str,
    field: &str,
    line_number: usize,
) -> Result<Vec<String>, CliError> {
    let parsed = serde_json::from_str::<Value>(value).map_err(|_| {
        CliError::Usage(format!(
            "invalid evidence batch YAML: {field} must be an inline JSON string array at line {line_number}"
        ))
    })?;
    optional_json_string_array(Some(&parsed), field)?.ok_or_else(|| {
        CliError::Usage(format!(
            "invalid evidence batch YAML: {field} must not be empty at line {line_number}"
        ))
    })
}
