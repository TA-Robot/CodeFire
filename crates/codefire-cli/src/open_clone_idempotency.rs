use super::idempotency::{
    idempotency_payload_hash, idempotency_record_path, idempotency_result_plan,
    require_idempotency_key, verify_idempotency_record,
};
use super::{
    now_iso_utc, read_optional_json, required_string, write_json_atomic, CliError, CloneOptions,
    CloneResult, OpenOptions, OpenResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(super) fn open_idempotency_payload(
    options: &OpenOptions,
    repo_root: &Path,
    target: &Path,
    branch_head: &str,
) -> Value {
    json!({
        "command": "open",
        "repo": repo_root,
        "branch": &options.branch,
        "path": target,
        "branch_head": branch_head,
    })
}

pub(super) fn load_open_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<OpenResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "open", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "open", key, payload)?;
    Ok(Some(OpenResult {
        branch: required_string(&record, &["result", "branch"])?,
        open_dir: PathBuf::from(required_string(&record, &["result", "open_dir"])?),
        plan: idempotency_result_plan(&record, &path)?,
    }))
}

pub(super) fn save_open_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &OpenResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "open", key);
    let record = json!({
        "version": 1,
        "command": "open",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "branch": &result.branch,
            "open_dir": &result.open_dir,
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

pub(super) fn clone_idempotency_payload(options: &CloneOptions, repo_root: &Path) -> Value {
    json!({
        "command": "clone",
        "repo": repo_root,
        "source": &options.source,
        "new_branch": &options.new_branch,
    })
}

pub(super) fn load_clone_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<CloneResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "clone", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "clone", key, payload)?;
    Ok(Some(CloneResult {
        plan: idempotency_result_plan(&record, &path)?,
    }))
}

pub(super) fn save_clone_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &CloneResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "clone", key);
    let record = json!({
        "version": 1,
        "command": "clone",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}
