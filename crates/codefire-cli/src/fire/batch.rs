use super::{
    default_manual_fire_severity, required_arg, run_manual_fire_specs, validate_manual_fire_spec,
    FireResult, ManualFireSpec,
};
use crate::{limited_yaml, parse_lock_option, CliError, LockOptions};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const YAML_CONTEXT: &str = "fire batch YAML";

#[derive(Debug)]
pub(crate) struct FireBatchOptions {
    pub(crate) path: PathBuf,
    pub(crate) batch_path: PathBuf,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
}

#[derive(Debug, Clone, Default)]
struct FireBatchDefaults {
    reason: Option<String>,
    severity: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct FireBatchSpec {
    source_atom: Option<String>,
    target_atom: Option<String>,
    reason: Option<String>,
    severity: Option<String>,
}

#[derive(Debug)]
struct FireBatchFile {
    version: u64,
    defaults: FireBatchDefaults,
    fires: Vec<FireBatchSpec>,
}

pub(crate) fn parse_fire_batch_args(args: &[String]) -> Result<FireBatchOptions, CliError> {
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
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--batch" => {
                index += 1;
                batch_path = Some(PathBuf::from(required_arg(args, index, "--batch")?));
            }
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--batch=") => {
                batch_path = Some(PathBuf::from(value.trim_start_matches("--batch=")));
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported fire batch option: {option}"
                )));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected fire batch argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(FireBatchOptions {
        path: path.unwrap_or(std::env::current_dir()?),
        batch_path: batch_path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--json]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
    })
}

pub(crate) fn run_fire_batch(options: &FireBatchOptions) -> Result<FireResult, CliError> {
    let batch = parse_fire_batch_file(&options.batch_path)?;
    if batch.version != 1 {
        return Err(CliError::Usage(format!(
            "unsupported fire batch version: {}",
            batch.version
        )));
    }
    if batch.fires.is_empty() {
        return Err(CliError::Usage(
            "fire batch requires at least one fire".to_string(),
        ));
    }
    let specs = resolve_fire_batch(&batch)?;
    run_manual_fire_specs(
        &options.path,
        &specs,
        options.dry_run,
        &options.lock,
        "fire-batch",
    )
}

pub(crate) fn fire_batch_data_json(result: &FireResult) -> Value {
    json!({
        "type": "codefire_fire_batch_result",
        "version": 1,
        "dry_run": result.dry_run,
        "item_count": result.item_count,
        "branch": &result.branch,
        "open_dir": &result.open_dir,
        "plan": &result.plan,
        "fires": result.fires.iter().map(|fire| {
            json!({
                "display_id": &fire.display_id,
                "fire_uid": &fire.fire_uid,
                "source_atom": &fire.source.atom_id,
                "target_atom": &fire.target.atom_id,
                "reason": &fire.reason,
                "severity": &fire.severity,
                "status": &fire.status,
            })
        }).collect::<Vec<_>>(),
    })
}

fn resolve_fire_batch(batch: &FireBatchFile) -> Result<Vec<ManualFireSpec>, CliError> {
    batch
        .fires
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            validate_manual_fire_spec(ManualFireSpec {
                source_atom: required_fire_field(spec.source_atom.as_deref(), index, "from")?
                    .to_string(),
                target_atom: required_fire_field(spec.target_atom.as_deref(), index, "to")?
                    .to_string(),
                reason: spec
                    .reason
                    .clone()
                    .or_else(|| batch.defaults.reason.clone())
                    .ok_or_else(|| missing_fire_field(index, "reason"))?,
                severity: spec
                    .severity
                    .clone()
                    .or_else(|| batch.defaults.severity.clone())
                    .unwrap_or_else(default_manual_fire_severity),
            })
        })
        .collect()
}

fn required_fire_field<'a>(
    value: Option<&'a str>,
    index: usize,
    field: &str,
) -> Result<&'a str, CliError> {
    let value = value.ok_or_else(|| missing_fire_field(index, field))?;
    if value.trim().is_empty() {
        return Err(missing_fire_field(index, field));
    }
    Ok(value)
}

fn missing_fire_field(index: usize, field: &str) -> CliError {
    CliError::Usage(format!("fire batch item {} missing {field}", index + 1))
}

fn parse_fire_batch_file(path: &Path) -> Result<FireBatchFile, CliError> {
    let text = fs::read_to_string(path)?;
    if text.trim_start().starts_with('{') {
        parse_fire_batch_json(&text)
    } else {
        parse_fire_batch_yaml(&text)
    }
}

fn parse_fire_batch_json(text: &str) -> Result<FireBatchFile, CliError> {
    let value: Value = serde_json::from_str(text)?;
    let version = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    let defaults = parse_json_defaults(value.get("defaults"))?;
    let fires = value
        .get("fires")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("fire batch JSON missing fires array".to_string()))?
        .iter()
        .map(parse_json_fire)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FireBatchFile {
        version,
        defaults,
        fires,
    })
}

fn parse_json_defaults(value: Option<&Value>) -> Result<FireBatchDefaults, CliError> {
    let Some(value) = value else {
        return Ok(FireBatchDefaults::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("fire batch defaults must be an object".to_string()))?;
    Ok(FireBatchDefaults {
        reason: optional_json_string(object.get("reason"), "defaults.reason")?,
        severity: optional_json_string(object.get("severity"), "defaults.severity")?,
    })
}

fn parse_json_fire(value: &Value) -> Result<FireBatchSpec, CliError> {
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("fire batch item must be an object".to_string()))?;
    Ok(FireBatchSpec {
        source_atom: optional_json_string(object.get("from"), "fire.from")?,
        target_atom: optional_json_string(object.get("to"), "fire.to")?,
        reason: optional_json_string(object.get("reason"), "fire.reason")?,
        severity: optional_json_string(object.get("severity"), "fire.severity")?,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FireBatchYamlSection {
    Root,
    Defaults,
    Fires,
}

fn parse_fire_batch_yaml(text: &str) -> Result<FireBatchFile, CliError> {
    let mut version = None;
    let mut defaults = FireBatchDefaults::default();
    let mut fires = Vec::new();
    let mut current_fire = None;
    let mut section = FireBatchYamlSection::Root;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        if raw_line.contains('\t') {
            return Err(CliError::Usage(format!(
                "invalid fire batch YAML: tabs are not supported at line {line_number}"
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
            (0, "defaults:", _) => section = FireBatchYamlSection::Defaults,
            (0, "fires:", _) => {
                push_current_fire(&mut fires, &mut current_fire);
                section = FireBatchYamlSection::Fires;
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
                    section = FireBatchYamlSection::Root;
                } else {
                    return Err(CliError::Usage(format!(
                        "invalid fire batch YAML: unsupported root key '{key}' at line {line_number}"
                    )));
                }
            }
            (2, _, FireBatchYamlSection::Defaults) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_default(&mut defaults, &key, value, line_number)?;
            }
            (2, _, FireBatchYamlSection::Fires) if text.starts_with("- ") => {
                push_current_fire(&mut fires, &mut current_fire);
                let rest = text[2..].trim();
                let mut fire = FireBatchSpec::default();
                if !rest.is_empty() {
                    let (key, value) =
                        limited_yaml::parse_key_value(rest, line_number, YAML_CONTEXT)?;
                    apply_yaml_fire_field(&mut fire, &key, value, line_number)?;
                }
                current_fire = Some(fire);
            }
            (4, _, FireBatchYamlSection::Fires) => {
                let fire = current_fire.as_mut().ok_or_else(|| {
                    CliError::Usage(format!(
                        "invalid fire batch YAML: fire field before list item at line {line_number}"
                    ))
                })?;
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_fire_field(fire, &key, value, line_number)?;
            }
            _ => {
                return Err(CliError::Usage(format!(
                    "invalid fire batch YAML indentation or section at line {line_number}"
                )));
            }
        }
    }
    push_current_fire(&mut fires, &mut current_fire);

    Ok(FireBatchFile {
        version: version
            .ok_or_else(|| CliError::Usage("fire batch YAML missing version".to_string()))?,
        defaults,
        fires,
    })
}

fn push_current_fire(fires: &mut Vec<FireBatchSpec>, current: &mut Option<FireBatchSpec>) {
    if let Some(fire) = current.take() {
        fires.push(fire);
    }
}

fn apply_yaml_default(
    defaults: &mut FireBatchDefaults,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "reason" => defaults.reason = Some(value),
        "severity" => defaults.severity = Some(value),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid fire batch YAML: unsupported defaults key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn apply_yaml_fire_field(
    fire: &mut FireBatchSpec,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "from" => fire.source_atom = Some(value),
        "to" => fire.target_atom = Some(value),
        "reason" => fire.reason = Some(value),
        "severity" => fire.severity = Some(value),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid fire batch YAML: unsupported fire key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}
