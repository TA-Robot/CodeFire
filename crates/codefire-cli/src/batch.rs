use super::{parse_lock_option, run_extinguish, CliError, ExtinguishOptions, LockOptions};
use crate::{limited_yaml, read_batch_file};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const YAML_CONTEXT: &str = "batch YAML";

#[derive(Debug)]
pub(super) struct BatchExtinguishOptions {
    pub(super) path: PathBuf,
    pub(super) batch_path: PathBuf,
    pub(super) dry_run: bool,
    pub(super) json_output: bool,
    pub(super) lock: LockOptions,
}

#[derive(Debug)]
pub(super) struct BatchExtinguishResult {
    pub(super) item_count: usize,
    pub(super) plan: Value,
}

#[derive(Debug, Clone, Default)]
struct BatchFireDefaults {
    resolution: Option<String>,
    rationale: Option<String>,
    evidence: Option<String>,
    evidence_refs: Vec<String>,
    refresh: Option<bool>,
}

#[derive(Debug, Clone, Default)]
struct BatchFireSpec {
    id: Option<String>,
    resolution: Option<String>,
    rationale: Option<String>,
    evidence: Option<String>,
    evidence_refs: Vec<String>,
    refresh: Option<bool>,
}

#[derive(Debug)]
struct BatchExtinguishFile {
    version: u64,
    defaults: BatchFireDefaults,
    fires: Vec<BatchFireSpec>,
}

#[derive(Debug)]
struct ResolvedBatchFire {
    id: String,
    resolution: String,
    rationale: String,
    evidence: String,
    evidence_refs: Vec<String>,
    refresh: bool,
    validation_plan: Value,
}

pub(super) fn has_batch_extinguish_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--batch")
}

pub(super) fn parse_extinguish_batch_args(
    args: &[String],
) -> Result<BatchExtinguishOptions, CliError> {
    let mut path = None;
    let mut batch_path = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--batch" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--batch requires a value".to_string()))?;
                batch_path = Some(PathBuf::from(value));
            }
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported batch extinguish option: {value}"
                )));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected batch extinguish argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(BatchExtinguishOptions {
        path: path.unwrap_or(std::env::current_dir()?),
        batch_path: batch_path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire extinguish --batch <file> [--path <open-dir>] [--dry-run] [--json]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
    })
}

pub(super) fn run_extinguish_batch(
    options: &BatchExtinguishOptions,
) -> Result<BatchExtinguishResult, CliError> {
    let batch = parse_batch_extinguish_file(&options.batch_path)?;
    if batch.version != 1 {
        return Err(CliError::Usage(format!(
            "unsupported batch extinguish version: {}",
            batch.version
        )));
    }
    if batch.fires.is_empty() {
        return Err(CliError::Usage(
            "batch extinguish requires at least one fire".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    let mut resolved = Vec::with_capacity(batch.fires.len());
    for fire in batch.fires {
        let id = fire
            .id
            .ok_or_else(|| CliError::Usage("batch fire entry missing id".to_string()))?;
        if !seen.insert(id.clone()) {
            return Err(CliError::Usage(format!("duplicate batch fire id: {id}")));
        }
        let resolution = fire
            .resolution
            .or_else(|| batch.defaults.resolution.clone())
            .unwrap_or_else(|| "addressed".to_string());
        let rationale = fire
            .rationale
            .or_else(|| batch.defaults.rationale.clone())
            .unwrap_or_default();
        let evidence = fire
            .evidence
            .or_else(|| batch.defaults.evidence.clone())
            .unwrap_or_default();
        let evidence_refs = if fire.evidence_refs.is_empty() {
            batch.defaults.evidence_refs.clone()
        } else {
            fire.evidence_refs
        };
        let refresh = fire.refresh.or(batch.defaults.refresh).unwrap_or(false);
        let validation = run_extinguish(&ExtinguishOptions {
            path: options.path.clone(),
            fire_id: id.clone(),
            resolution: resolution.clone(),
            rationale: rationale.clone(),
            evidence: evidence.clone(),
            evidence_refs: evidence_refs.clone(),
            refresh,
            dry_run: true,
            json_output: true,
            lock: options.lock,
            idempotency_key: None,
            edit_rationale: false,
        })?;
        resolved.push(ResolvedBatchFire {
            id,
            resolution,
            rationale,
            evidence,
            evidence_refs,
            refresh,
            validation_plan: validation.plan,
        });
    }

    let plan = batch_extinguish_operation_plan(options, &resolved);
    if options.dry_run {
        return Ok(BatchExtinguishResult {
            item_count: resolved.len(),
            plan,
        });
    }

    for fire in &resolved {
        run_extinguish(&ExtinguishOptions {
            path: options.path.clone(),
            fire_id: fire.id.clone(),
            resolution: fire.resolution.clone(),
            rationale: fire.rationale.clone(),
            evidence: fire.evidence.clone(),
            evidence_refs: fire.evidence_refs.clone(),
            refresh: fire.refresh,
            dry_run: false,
            json_output: false,
            lock: options.lock,
            idempotency_key: None,
            edit_rationale: false,
        })?;
    }

    Ok(BatchExtinguishResult {
        item_count: resolved.len(),
        plan,
    })
}

fn batch_extinguish_operation_plan(
    options: &BatchExtinguishOptions,
    fires: &[ResolvedBatchFire],
) -> Value {
    let items = fires
        .iter()
        .map(|fire| {
            json!({
                "fire_id": &fire.id,
                "resolution": &fire.resolution,
                "refresh": fire.refresh,
                "has_rationale": !fire.rationale.is_empty(),
                "has_evidence": !fire.evidence.is_empty() || !fire.evidence_refs.is_empty(),
                "evidence_refs": &fire.evidence_refs,
                "validation": &fire.validation_plan,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "extinguish-batch",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "open_dir": &options.path,
        "batch_file": &options.batch_path,
        "item_count": fires.len(),
        "items": items,
        "operations": [
            {"kind": "validate_all_batch_items", "count": fires.len()},
            {"kind": "update_fire_statuses", "status": "extinguished"},
            {"kind": "append_resolutions", "count": fires.len()},
        ],
        "next_actions": [
            {"kind": "verify", "command": "codefire verify --details --json", "target": {"path": &options.path}},
        ],
    })
}

fn parse_batch_extinguish_file(path: &Path) -> Result<BatchExtinguishFile, CliError> {
    let text = read_batch_file(path, "extinguish batch")?;
    if starts_json_document(&text) {
        parse_batch_extinguish_json(&text)
    } else {
        parse_batch_extinguish_yaml(&text)
    }
}

fn parse_batch_extinguish_json(text: &str) -> Result<BatchExtinguishFile, CliError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| CliError::Usage(format!("invalid batch extinguish JSON: {error}")))?;
    let object = value.as_object().ok_or_else(|| {
        CliError::Usage(
            "batch extinguish JSON must be an object with version and fires array; top-level arrays are not supported. Use {\"version\":1,\"fires\":[...]} wrapper".to_string(),
        )
    })?;
    let version = object.get("version").and_then(Value::as_u64).unwrap_or(0);
    let defaults = parse_json_defaults(object.get("defaults"))?;
    let fires = object
        .get("fires")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("batch extinguish JSON missing fires array".to_string()))?
        .iter()
        .map(parse_json_fire)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BatchExtinguishFile {
        version,
        defaults,
        fires,
    })
}

fn starts_json_document(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn parse_json_defaults(value: Option<&Value>) -> Result<BatchFireDefaults, CliError> {
    let Some(value) = value else {
        return Ok(BatchFireDefaults::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("batch defaults must be an object".to_string()))?;
    Ok(BatchFireDefaults {
        resolution: optional_json_string(object.get("resolution"), "defaults.resolution")?,
        rationale: optional_json_string(object.get("rationale"), "defaults.rationale")?,
        evidence: optional_json_string(object.get("evidence"), "defaults.evidence")?,
        evidence_refs: optional_json_string_array(
            object.get("evidence_refs"),
            object.get("evidence_ref"),
            "defaults.evidence_refs",
        )?,
        refresh: optional_json_bool(object.get("refresh"), "defaults.refresh")?,
    })
}

fn parse_json_fire(value: &Value) -> Result<BatchFireSpec, CliError> {
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("batch fire entry must be an object".to_string()))?;
    Ok(BatchFireSpec {
        id: optional_json_string(object.get("id"), "fire.id")?,
        resolution: optional_json_string(object.get("resolution"), "fire.resolution")?,
        rationale: optional_json_string(object.get("rationale"), "fire.rationale")?,
        evidence: optional_json_string(object.get("evidence"), "fire.evidence")?,
        evidence_refs: optional_json_string_array(
            object.get("evidence_refs"),
            object.get("evidence_ref"),
            "fire.evidence_refs",
        )?,
        refresh: optional_json_bool(object.get("refresh"), "fire.refresh")?,
    })
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

fn optional_json_bool(value: Option<&Value>, field: &str) -> Result<Option<bool>, CliError> {
    value
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| CliError::Usage(format!("{field} must be a boolean")))
        })
        .transpose()
}

fn optional_json_string_array(
    array_value: Option<&Value>,
    single_value: Option<&Value>,
    field: &str,
) -> Result<Vec<String>, CliError> {
    let mut values = Vec::new();
    if let Some(value) = single_value {
        values.push(
            value
                .as_str()
                .ok_or_else(|| CliError::Usage(format!("{field} must be a string or array")))?
                .to_string(),
        );
    }
    if let Some(value) = array_value {
        let array = value
            .as_array()
            .ok_or_else(|| CliError::Usage(format!("{field} must be an array")))?;
        for item in array {
            values.push(
                item.as_str()
                    .ok_or_else(|| CliError::Usage(format!("{field} entries must be strings")))?
                    .to_string(),
            );
        }
    }
    Ok(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatchYamlSection {
    Root,
    Defaults,
    Fires,
}

fn parse_batch_extinguish_yaml(text: &str) -> Result<BatchExtinguishFile, CliError> {
    let mut version = None;
    let mut defaults = BatchFireDefaults::default();
    let mut fires = Vec::new();
    let mut current_fire = None;
    let mut section = BatchYamlSection::Root;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        if raw_line.contains('\t') {
            return Err(CliError::Usage(format!(
                "invalid batch YAML: tabs are not supported at line {line_number}"
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
            (0, "defaults:", _) => section = BatchYamlSection::Defaults,
            (0, "fires:", _) => {
                push_current_fire(&mut fires, &mut current_fire);
                section = BatchYamlSection::Fires;
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
                    section = BatchYamlSection::Root;
                } else {
                    return Err(CliError::Usage(format!(
                        "invalid batch YAML: unsupported root key '{key}' at line {line_number}"
                    )));
                }
            }
            (2, _, BatchYamlSection::Defaults) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_default(&mut defaults, &key, value, line_number)?;
            }
            (2, _, BatchYamlSection::Fires) if text.starts_with("- ") => {
                push_current_fire(&mut fires, &mut current_fire);
                let rest = text[2..].trim();
                let mut fire = BatchFireSpec::default();
                if !rest.is_empty() {
                    let (key, value) =
                        limited_yaml::parse_key_value(rest, line_number, YAML_CONTEXT)?;
                    apply_yaml_fire_field(&mut fire, &key, value, line_number)?;
                }
                current_fire = Some(fire);
            }
            (4, _, BatchYamlSection::Fires) => {
                let fire = current_fire.as_mut().ok_or_else(|| {
                    CliError::Usage(format!(
                        "invalid batch YAML: fire field before list item at line {line_number}"
                    ))
                })?;
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_fire_field(fire, &key, value, line_number)?;
            }
            _ => {
                return Err(CliError::Usage(format!(
                    "invalid batch YAML indentation or section at line {line_number}"
                )));
            }
        }
    }
    push_current_fire(&mut fires, &mut current_fire);

    Ok(BatchExtinguishFile {
        version: version
            .ok_or_else(|| CliError::Usage("batch YAML missing version".to_string()))?,
        defaults,
        fires,
    })
}

fn push_current_fire(fires: &mut Vec<BatchFireSpec>, current_fire: &mut Option<BatchFireSpec>) {
    if let Some(fire) = current_fire.take() {
        fires.push(fire);
    }
}

fn apply_yaml_default(
    defaults: &mut BatchFireDefaults,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "resolution" => defaults.resolution = Some(value),
        "rationale" => defaults.rationale = Some(value),
        "evidence" => defaults.evidence = Some(value),
        "evidence_ref" => defaults.evidence_refs.push(value),
        "refresh" => defaults.refresh = Some(parse_yaml_bool(&value, "refresh", line_number)?),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid batch YAML: unsupported defaults key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn apply_yaml_fire_field(
    fire: &mut BatchFireSpec,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "id" => fire.id = Some(value),
        "resolution" => fire.resolution = Some(value),
        "rationale" => fire.rationale = Some(value),
        "evidence" => fire.evidence = Some(value),
        "evidence_ref" => fire.evidence_refs.push(value),
        "refresh" => fire.refresh = Some(parse_yaml_bool(&value, "refresh", line_number)?),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid batch YAML: unsupported fire key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn parse_yaml_bool(value: &str, field: &str, line_number: usize) -> Result<bool, CliError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CliError::Usage(format!(
            "invalid batch YAML: {field} must be true or false at line {line_number}"
        ))),
    }
}
