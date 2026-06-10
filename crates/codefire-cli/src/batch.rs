use super::{
    compute_scan, now_iso_utc, open_context, parse_lock_option, read_json, required_string,
    stable_hash_48, validate_evidence_refs, write_json_atomic, CliError, LockOptions, RepoLock,
};
use crate::{limited_yaml, read_batch_file};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const YAML_CONTEXT: &str = "batch YAML";
const DEFAULT_BATCH_DETAIL_LIMIT: usize = 20;

#[derive(Debug)]
pub(super) struct BatchExtinguishOptions {
    pub(super) path: PathBuf,
    pub(super) batch_path: PathBuf,
    pub(super) dry_run: bool,
    pub(super) full_output: bool,
    pub(super) json_output: bool,
    pub(super) lock: LockOptions,
}

#[derive(Debug)]
pub(super) struct BatchExtinguishResult {
    pub(super) repo_root: PathBuf,
    pub(super) branch: String,
    pub(super) open_dir: PathBuf,
    pub(super) dry_run: bool,
    pub(super) item_count: usize,
    pub(super) applied_count: usize,
    pub(super) fire_ids: Vec<String>,
    pub(super) resolution_uids: Vec<String>,
    pub(super) remaining_open_fire_count: usize,
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
    display_id: String,
    fire_uid: String,
    resolution_uid: String,
    resolution: String,
    rationale: String,
    evidence: String,
    evidence_refs: Vec<String>,
    refresh: bool,
    source_atom: String,
    target_atom: String,
    fire_index: usize,
    resolution_record: codefire_core::Resolution,
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
    let mut full_output = false;
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
            "--full" => full_output = true,
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
                "usage: codefire extinguish --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]"
                    .to_string(),
            )
        })?,
        dry_run,
        full_output,
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
    let context = open_context(&options.path)?;
    let _lock = RepoLock::acquire_with_options(&context.repo_root, &options.lock)?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let policy = codefire_core::parse_verification_policy(&context.open_dir)?;
    let scan_execution = compute_scan(&context.open_dir, !options.dry_run)?;
    let scan = scan_execution.scan;
    let mut fires = scan_execution.fires;
    let mut resolved = resolve_batch_fires(&batch, &policy, &scan, &fires, &context.repo_root)?;
    let plan = batch_extinguish_operation_plan(options, &resolved);

    if options.dry_run {
        return Ok(BatchExtinguishResult {
            repo_root: context.repo_root,
            branch: context.branch,
            open_dir: context.open_dir,
            dry_run: true,
            item_count: resolved.len(),
            applied_count: 0,
            fire_ids: resolved
                .iter()
                .map(|fire| fire.display_id.clone())
                .collect(),
            resolution_uids: resolved
                .iter()
                .map(|fire| fire.resolution_uid.clone())
                .collect(),
            remaining_open_fire_count: scan.open_fires.len(),
            plan,
        });
    }

    let resolutions_path = active_state_path.join("resolutions.json");
    let mut resolutions: Vec<codefire_core::Resolution> = if resolutions_path.exists() {
        serde_json::from_value(read_json(&resolutions_path)?)?
    } else {
        Vec::new()
    };
    for fire in &resolved {
        fires[fire.fire_index].status = "extinguished".to_string();
        fires[fire.fire_index].resolution_uid = Some(fire.resolution_uid.clone());
        if fire.refresh {
            for existing in &mut resolutions {
                if existing.fire_uid == fire.fire_uid && existing.status == "active" {
                    existing.status = "superseded".to_string();
                }
            }
        }
    }
    let fire_ids = resolved
        .iter()
        .map(|fire| fire.display_id.clone())
        .collect::<Vec<_>>();
    let resolution_uids = resolved
        .iter()
        .map(|fire| fire.resolution_uid.clone())
        .collect::<Vec<_>>();
    let applied_count = resolved.len();
    resolutions.extend(resolved.drain(..).map(|fire| fire.resolution_record));
    write_json_atomic(
        &active_state_path.join("fires.json"),
        &serde_json::to_value(&fires)?,
    )?;
    write_json_atomic(&resolutions_path, &serde_json::to_value(&resolutions)?)?;
    let remaining_open_fire_count = fires.iter().filter(|fire| fire.status == "open").count();

    Ok(BatchExtinguishResult {
        repo_root: context.repo_root,
        branch: context.branch,
        open_dir: context.open_dir,
        dry_run: false,
        item_count: applied_count,
        applied_count,
        fire_ids,
        resolution_uids,
        remaining_open_fire_count,
        plan,
    })
}

fn resolve_batch_fires(
    batch: &BatchExtinguishFile,
    policy: &codefire_core::VerificationPolicy,
    scan: &codefire_core::ScanResult,
    fires: &[codefire_core::Fire],
    repo_root: &Path,
) -> Result<Vec<ResolvedBatchFire>, CliError> {
    let mut seen_input_ids = BTreeSet::new();
    let mut seen_fire_uids = BTreeSet::new();
    let mut resolved = Vec::with_capacity(batch.fires.len());
    for fire in &batch.fires {
        let id = fire
            .id
            .clone()
            .ok_or_else(|| CliError::Usage("batch fire entry missing id".to_string()))?;
        if !seen_input_ids.insert(id.clone()) {
            return Err(CliError::Usage(format!("duplicate batch fire id: {id}")));
        }
        let resolution = fire
            .resolution
            .clone()
            .or_else(|| batch.defaults.resolution.clone())
            .unwrap_or_else(|| "addressed".to_string());
        let rationale = fire
            .rationale
            .clone()
            .or_else(|| batch.defaults.rationale.clone())
            .unwrap_or_default();
        let evidence = fire
            .evidence
            .clone()
            .or_else(|| batch.defaults.evidence.clone())
            .unwrap_or_default();
        let evidence_refs = if fire.evidence_refs.is_empty() {
            batch.defaults.evidence_refs.clone()
        } else {
            fire.evidence_refs.clone()
        };
        let refresh = fire.refresh.or(batch.defaults.refresh).unwrap_or(false);
        validate_batch_resolution_fields(
            &resolution,
            &rationale,
            &evidence,
            &evidence_refs,
            policy,
        )?;
        validate_evidence_refs(repo_root, &evidence_refs)?;
        let fire_index = fires
            .iter()
            .position(|candidate| candidate.display_id == id || candidate.fire_uid == id)
            .ok_or_else(|| CliError::Usage(format!("unknown fire: {id}")))?;
        let current_fire = &fires[fire_index];
        if !seen_fire_uids.insert(current_fire.fire_uid.clone()) {
            return Err(CliError::Usage(format!(
                "duplicate batch fire target: {}",
                current_fire.display_id
            )));
        }
        if current_fire.status != "open" && !refresh {
            return Err(CliError::Usage(format!("fire is not open: {id}")));
        }
        let resolved_at = now_iso_utc();
        let resolution_uid = format!(
            "res_{:012x}",
            stable_hash_48(format!("{}:{resolved_at}", current_fire.fire_uid).as_bytes())
        );
        let resolution_record = codefire_core::build_resolution(
            current_fire,
            &scan.atom_index,
            &scan.trace_graph,
            policy,
            codefire_core::ResolutionRequest {
                resolution_uid: resolution_uid.clone(),
                resolution_type: resolution.clone(),
                rationale: rationale.clone(),
                evidence: evidence.clone(),
                evidence_refs: evidence_refs.clone(),
                resolved_at,
            },
        )?;
        resolved.push(ResolvedBatchFire {
            id,
            display_id: current_fire.display_id.clone(),
            fire_uid: current_fire.fire_uid.clone(),
            resolution_uid,
            resolution,
            rationale,
            evidence,
            evidence_refs,
            refresh,
            source_atom: current_fire.source.atom_id.clone(),
            target_atom: current_fire.target.atom_id.clone(),
            fire_index,
            resolution_record,
        });
    }
    Ok(resolved)
}

fn validate_batch_resolution_fields(
    resolution: &str,
    rationale: &str,
    evidence: &str,
    evidence_refs: &[String],
    policy: &codefire_core::VerificationPolicy,
) -> Result<(), CliError> {
    if resolution == "no-change-required"
        && policy.no_change_required_requires_rationale
        && rationale.is_empty()
    {
        return Err(CliError::Usage(
            "no-change-required requires --rationale".to_string(),
        ));
    }
    if rationale.is_empty() && evidence.is_empty() && evidence_refs.is_empty() {
        return Err(CliError::Usage(
            "extinguish requires --rationale, --evidence, or --evidence-ref".to_string(),
        ));
    }
    Ok(())
}

fn batch_extinguish_operation_plan(
    options: &BatchExtinguishOptions,
    fires: &[ResolvedBatchFire],
) -> Value {
    let items = fires
        .iter()
        .take(batch_detail_limit(options, fires.len()))
        .map(|fire| {
            json!({
                "input_id": &fire.id,
                "display_id": &fire.display_id,
                "fire_uid": &fire.fire_uid,
                "source_atom": &fire.source_atom,
                "target_atom": &fire.target_atom,
                "resolution": &fire.resolution,
                "resolution_uid": &fire.resolution_uid,
                "refresh": fire.refresh,
                "has_rationale": !fire.rationale.is_empty(),
                "has_evidence": !fire.evidence.is_empty() || !fire.evidence_refs.is_empty(),
                "evidence_refs": &fire.evidence_refs,
            })
        })
        .collect::<Vec<_>>();
    let detail_limit = batch_detail_limit(options, fires.len());
    let omitted = fires.len().saturating_sub(detail_limit);
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "extinguish-batch",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "open_dir": &options.path,
        "batch_file": &options.batch_path,
        "item_count": fires.len(),
        "valid_item_count": fires.len(),
        "invalid_item_count": 0,
        "item_detail_limit": detail_limit,
        "items_omitted": omitted,
        "full_output": options.full_output,
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

fn batch_detail_limit(options: &BatchExtinguishOptions, item_count: usize) -> usize {
    if options.full_output {
        item_count
    } else {
        item_count.min(DEFAULT_BATCH_DETAIL_LIMIT)
    }
}

pub(super) fn batch_extinguish_data_json(result: &BatchExtinguishResult) -> Value {
    let fire_id_limit = if result.fire_ids.len() <= DEFAULT_BATCH_DETAIL_LIMIT {
        result.fire_ids.len()
    } else {
        DEFAULT_BATCH_DETAIL_LIMIT
    };
    json!({
        "type": "codefire_extinguish_batch_result",
        "version": 1,
        "dry_run": result.dry_run,
        "branch": &result.branch,
        "open_dir": &result.open_dir,
        "item_count": result.item_count,
        "applied_count": result.applied_count,
        "fire_ids": result.fire_ids.iter().take(fire_id_limit).collect::<Vec<_>>(),
        "fire_ids_omitted": result.fire_ids.len().saturating_sub(fire_id_limit),
        "resolution_uids": result.resolution_uids.iter().take(fire_id_limit).collect::<Vec<_>>(),
        "resolution_uids_omitted": result.resolution_uids.len().saturating_sub(fire_id_limit),
        "remaining_open_fire_count": result.remaining_open_fire_count,
        "plan": &result.plan,
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
