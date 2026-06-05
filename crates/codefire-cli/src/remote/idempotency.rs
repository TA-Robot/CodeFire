use super::super::idempotency::{
    idempotency_payload_hash, require_idempotency_key, verify_idempotency_record,
};
use super::super::{now_iso_utc, read_optional_json, required_string, write_json_atomic, CliError};
use super::{
    ref_file_name, remote_dirs, RequestApplyResult, RequestMergeResult, RequestReviewResult,
    UploadResult,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(crate) fn remote_idempotency_key(request: &Value) -> Option<String> {
    request
        .get("idempotency_key")
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(crate) fn remote_upload_payload(
    project_root: &Path,
    branch_name: &str,
    local_branch: &str,
    head: &str,
    object_records_hash: Option<&str>,
) -> Value {
    json!({
        "command": "remote_upload",
        "project": project_root,
        "remote_branch": branch_name,
        "local_branch": local_branch,
        "head": head,
        "object_records_hash": object_records_hash,
    })
}

pub(crate) fn remote_merge_payload(
    project_root: &Path,
    source_url: &str,
    source_head: &str,
    target_url: &str,
    target_head: &str,
) -> Value {
    json!({
        "command": "remote_request_merge",
        "project": project_root,
        "source_url": source_url,
        "source_head": source_head,
        "target_url": target_url,
        "target_head": target_head,
    })
}

pub(crate) fn remote_review_payload(
    project_root: &Path,
    mr: &Value,
    mr_id: &str,
    reviewer: &str,
    decision: &str,
    comment: &str,
) -> Result<Value, CliError> {
    Ok(json!({
        "command": "remote_request_review",
        "project": project_root,
        "mr_id": mr_id,
        "source_url": required_string(mr, &["source_url"])?,
        "source_head": required_string(mr, &["source_head"])?,
        "target_url": required_string(mr, &["target_url"])?,
        "target_head_at_request": required_string(mr, &["target_head_at_request"])?,
        "reviewer": reviewer,
        "decision": decision,
        "comment": comment,
    }))
}

pub(crate) fn remote_apply_payload(
    project_root: &Path,
    mr: &Value,
    mr_id: &str,
) -> Result<Value, CliError> {
    Ok(json!({
        "command": "remote_request_apply",
        "project": project_root,
        "mr_id": mr_id,
        "source_url": required_string(mr, &["source_url"])?,
        "source_head": required_string(mr, &["source_head"])?,
        "target_url": required_string(mr, &["target_url"])?,
        "target_head_at_request": required_string(mr, &["target_head_at_request"])?,
    }))
}

pub(crate) fn load_remote_idempotency_result(
    project_root: &Path,
    command: &str,
    key: &str,
    payload: &Value,
) -> Result<Option<Value>, CliError> {
    require_idempotency_key(key)?;
    let path = remote_idempotency_record_path(project_root, command, key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, command, key, payload)?;
    Ok(Some(
        record
            .get("result")
            .cloned()
            .ok_or_else(|| missing_result_error(&path))?,
    ))
}

pub(crate) fn save_remote_idempotency_result(
    project_root: &Path,
    command: &str,
    key: &str,
    payload: &Value,
    result: &Value,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = remote_idempotency_record_path(project_root, command, key);
    let record = json!({
        "version": 1,
        "command": command,
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": result,
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

pub(crate) fn load_remote_upload_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<UploadResult>, CliError> {
    load_remote_idempotency_result(project_root, "remote_upload", key, payload)?
        .map(|result| {
            Ok(UploadResult {
                head: required_string(&result, &["head"])?,
                plan: result_plan(&result, project_root, "remote_upload", key)?,
            })
        })
        .transpose()
}

pub(crate) fn save_remote_upload_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
    result: &UploadResult,
) -> Result<(), CliError> {
    save_remote_idempotency_result(
        project_root,
        "remote_upload",
        key,
        payload,
        &json!({"head": &result.head, "plan": &result.plan}),
    )
}

pub(crate) fn load_remote_merge_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<RequestMergeResult>, CliError> {
    load_remote_idempotency_result(project_root, "remote_request_merge", key, payload)?
        .map(|result| {
            Ok(RequestMergeResult {
                id: required_string(&result, &["id"])?,
                source_head: required_string(&result, &["source_head"])?,
                target_head: required_string(&result, &["target_head"])?,
                plan: result_plan(&result, project_root, "remote_request_merge", key)?,
            })
        })
        .transpose()
}

pub(crate) fn save_remote_merge_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
    result: &RequestMergeResult,
) -> Result<(), CliError> {
    save_remote_idempotency_result(
        project_root,
        "remote_request_merge",
        key,
        payload,
        &json!({
            "id": &result.id,
            "source_head": &result.source_head,
            "target_head": &result.target_head,
            "plan": &result.plan,
        }),
    )
}

pub(crate) fn load_remote_review_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<RequestReviewResult>, CliError> {
    load_remote_idempotency_result(project_root, "remote_request_review", key, payload)?
        .map(|result| {
            Ok(RequestReviewResult {
                reviewer: required_string(&result, &["reviewer"])?,
                decision: required_string(&result, &["decision"])?,
                plan: result_plan(&result, project_root, "remote_request_review", key)?,
            })
        })
        .transpose()
}

pub(crate) fn save_remote_review_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
    result: &RequestReviewResult,
) -> Result<(), CliError> {
    save_remote_idempotency_result(
        project_root,
        "remote_request_review",
        key,
        payload,
        &json!({
            "reviewer": &result.reviewer,
            "decision": &result.decision,
            "plan": &result.plan,
        }),
    )
}

pub(crate) fn load_remote_apply_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<RequestApplyResult>, CliError> {
    load_remote_idempotency_result(project_root, "remote_request_apply", key, payload)?
        .map(|result| {
            Ok(RequestApplyResult {
                target_branch: required_string(&result, &["target_branch"])?,
                head: required_string(&result, &["head"])?,
                plan: result_plan(&result, project_root, "remote_request_apply", key)?,
            })
        })
        .transpose()
}

pub(crate) fn save_remote_apply_idempotency(
    project_root: &Path,
    key: &str,
    payload: &Value,
    result: &RequestApplyResult,
) -> Result<(), CliError> {
    save_remote_idempotency_result(
        project_root,
        "remote_request_apply",
        key,
        payload,
        &json!({
            "target_branch": &result.target_branch,
            "head": &result.head,
            "plan": &result.plan,
        }),
    )
}

fn remote_idempotency_record_path(project_root: &Path, command: &str, key: &str) -> PathBuf {
    remote_dirs(project_root)
        .idempotency
        .join(command)
        .join(format!("{}.json", ref_file_name(key)))
}

fn result_plan(
    result: &Value,
    project_root: &Path,
    command: &str,
    key: &str,
) -> Result<Value, CliError> {
    result.get("plan").cloned().ok_or_else(|| {
        missing_result_error(&remote_idempotency_record_path(project_root, command, key))
    })
}

fn missing_result_error(path: &Path) -> CliError {
    CliError::InvalidRepository(format!(
        "remote idempotency record missing result: {}",
        path.display()
    ))
}
