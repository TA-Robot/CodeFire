use super::idempotency::{
    idempotency_payload_hash, idempotency_record_path, require_idempotency_key,
    verify_idempotency_record,
};
use super::{
    absolute_path, common_ancestor, load_branch_record, opened_registry_path, read_json,
    read_optional_json, required_string, write_json_atomic, CliError, MergeAction, MergeFileAction,
    MergeOptions, MergeResult, OpenContext, PatchImportOptions, SemanticConflictCandidate,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(super) fn merge_idempotency_payload(
    repo_root: &Path,
    options: &MergeOptions,
) -> Result<Value, CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    let source = load_branch_record(repo_root, &options.source_branch)?;
    let source_head = required_string(&source, &["head"])?;
    let source_state = source
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed");
    if source_state == "open-burning" {
        return Err(CliError::Usage(format!(
            "cannot merge from branch '{}' while it is open-burning",
            options.source_branch
        )));
    }
    let target = load_branch_record(repo_root, &options.target_branch)?;
    let target_head = required_string(&target, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &source_head)?;
    codefire_store::validate_sealed_commit(&objects, &target_head)?;
    let registry_path = opened_registry_path(repo_root, &options.target_branch);
    if !registry_path.exists() {
        return Err(CliError::Usage(format!(
            "target branch must be open: {}",
            options.target_branch
        )));
    }
    let registry = read_json(&registry_path)?;
    let target_dir = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let base = common_ancestor(&objects, &source_head, &target_head)?
        .ok_or_else(|| CliError::Usage("no common ancestor found".to_string()))?;
    Ok(json!({
        "command": "merge",
        "repo": repo_root,
        "source_branch": &options.source_branch,
        "target_branch": &options.target_branch,
        "source_head": source_head,
        "target_head": target_head,
        "target_dir": target_dir,
        "base": base,
    }))
}

pub(super) fn load_merge_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<MergeResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "merge", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "merge", key, payload)?;
    Ok(Some(merge_result_from_record(&record, &path)?))
}

pub(super) fn save_merge_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &MergeResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "merge", key);
    let record = json!({
        "version": 1,
        "command": "merge",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": merge_result_json(result),
        "created_at": super::now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

pub(super) fn patch_import_idempotency_payload(
    options: &PatchImportOptions,
    context: &OpenContext,
    patch: &Value,
    patch_base: &str,
    paths: &[String],
) -> Result<Value, CliError> {
    Ok(json!({
        "command": "patch_import",
        "repo": &context.repo_root,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "patch_file": absolute_path(&options.path)?,
        "patch_base": patch_base,
        "current_base": required_string(&context.registry, &["open", "current_base_commit"])?,
        "patch_source": patch.get("source").cloned().unwrap_or_else(|| json!(null)),
        "patch": patch,
        "paths": paths,
    }))
}

pub(super) fn load_patch_import_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<Value>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "patch_import", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "patch_import", key, payload)?;
    Ok(Some(record_result(&record, &path)?))
}

pub(super) fn save_patch_import_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &Value,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "patch_import", key);
    let record = json!({
        "version": 1,
        "command": "patch_import",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": result,
        "created_at": super::now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

fn merge_result_json(result: &MergeResult) -> Value {
    json!({
        "source_branch": &result.source_branch,
        "target_branch": &result.target_branch,
        "source_head": &result.source_head,
        "target_head": &result.target_head,
        "base": &result.base,
        "target_dir": &result.target_dir,
        "file_actions": result.file_actions.iter().map(merge_file_action_record).collect::<Vec<_>>(),
        "conflicts": &result.conflicts,
        "semantic_conflicts": result.semantic_conflicts.iter().map(semantic_conflict_record).collect::<Vec<_>>(),
        "dry_run": result.dry_run,
        "applied": result.applied,
    })
}

fn merge_result_from_record(record: &Value, path: &Path) -> Result<MergeResult, CliError> {
    let result = record_result(record, path)?;
    Ok(MergeResult {
        source_branch: required_string(&result, &["source_branch"])?,
        target_branch: required_string(&result, &["target_branch"])?,
        source_head: required_string(&result, &["source_head"])?,
        target_head: required_string(&result, &["target_head"])?,
        base: required_string(&result, &["base"])?,
        target_dir: PathBuf::from(required_string(&result, &["target_dir"])?),
        file_actions: merge_file_actions_from_result(&result, path)?,
        conflicts: string_array(&result, "conflicts", path)?,
        semantic_conflicts: semantic_conflicts_from_result(&result, path)?,
        dry_run: required_bool(&result, "dry_run", path)?,
        applied: required_bool(&result, "applied", path)?,
    })
}

fn merge_file_action_record(action: &MergeFileAction) -> Value {
    json!({
        "path": &action.path,
        "action": action.action.as_str(),
        "content": &action.content,
    })
}

fn semantic_conflict_record(candidate: &SemanticConflictCandidate) -> Value {
    json!({
        "atom_id": &candidate.atom_id,
        "base_hash": &candidate.base_hash,
        "source_hash": &candidate.source_hash,
        "target_hash": &candidate.target_hash,
        "source_path": &candidate.source_path,
        "target_path": &candidate.target_path,
    })
}

fn merge_file_actions_from_result(
    result: &Value,
    record_path: &Path,
) -> Result<Vec<MergeFileAction>, CliError> {
    let actions = result
        .get("file_actions")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_record(record_path, "merge result file_actions must be a list"))?;
    actions
        .iter()
        .map(|action| {
            Ok(MergeFileAction {
                path: required_string(action, &["path"])?,
                action: merge_action_from_str(&required_string(action, &["action"])?)?,
                content: optional_byte_array(action.get("content"), record_path)?,
            })
        })
        .collect()
}

fn semantic_conflicts_from_result(
    result: &Value,
    record_path: &Path,
) -> Result<Vec<SemanticConflictCandidate>, CliError> {
    let candidates = result
        .get("semantic_conflicts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            invalid_record(
                record_path,
                "merge result semantic_conflicts must be a list",
            )
        })?;
    candidates
        .iter()
        .map(|candidate| {
            Ok(SemanticConflictCandidate {
                atom_id: required_string(candidate, &["atom_id"])?,
                base_hash: optional_string(candidate.get("base_hash"), record_path)?,
                source_hash: optional_string(candidate.get("source_hash"), record_path)?,
                target_hash: optional_string(candidate.get("target_hash"), record_path)?,
                source_path: optional_string(candidate.get("source_path"), record_path)?,
                target_path: optional_string(candidate.get("target_path"), record_path)?,
            })
        })
        .collect()
}

fn merge_action_from_str(value: &str) -> Result<MergeAction, CliError> {
    match value {
        "write_source" => Ok(MergeAction::WriteSource),
        "delete_target" => Ok(MergeAction::DeleteTarget),
        "write_conflict_markers" => Ok(MergeAction::WriteConflictMarkers),
        _ => Err(CliError::InvalidRepository(format!(
            "unknown merge action in idempotency record: {value}"
        ))),
    }
}

fn record_result(record: &Value, path: &Path) -> Result<Value, CliError> {
    record.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "idempotency record missing result: {}",
            path.display()
        ))
    })
}

fn string_array(value: &Value, key: &str, record_path: &Path) -> Result<Vec<String>, CliError> {
    let array = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_record(record_path, &format!("{key} must be a list")))?;
    array
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                invalid_record(record_path, &format!("{key} entries must be strings"))
            })
        })
        .collect()
}

fn required_bool(value: &Value, key: &str, record_path: &Path) -> Result<bool, CliError> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| invalid_record(record_path, &format!("{key} must be a boolean")))
}

fn optional_string(value: Option<&Value>, record_path: &Path) -> Result<Option<String>, CliError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(invalid_record(
            record_path,
            "optional field must be a string",
        )),
    }
}

fn optional_byte_array(
    value: Option<&Value>,
    record_path: &Path,
) -> Result<Option<Vec<u8>>, CliError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => {
            let mut bytes = Vec::with_capacity(items.len());
            for item in items {
                let byte = item.as_u64().ok_or_else(|| {
                    invalid_record(record_path, "merge action content must contain byte values")
                })?;
                if byte > u8::MAX as u64 {
                    return Err(invalid_record(
                        record_path,
                        "merge action content byte value out of range",
                    ));
                }
                bytes.push(byte as u8);
            }
            Ok(Some(bytes))
        }
        Some(_) => Err(invalid_record(
            record_path,
            "merge action content must be null or a byte array",
        )),
    }
}

fn invalid_record(path: &Path, message: &str) -> CliError {
    CliError::InvalidRepository(format!("{message}: {}", path.display()))
}
