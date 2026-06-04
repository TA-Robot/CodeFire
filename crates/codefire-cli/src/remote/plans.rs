use super::{RequestApplyOptions, RequestMergeOptions, RequestReviewOptions, UploadOptions};
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn upload_operation_plan(
    options: &UploadOptions,
    repo_root: &Path,
    head: &str,
    transport: &str,
    remote_branch: &str,
    object_count: Option<usize>,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "upload",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "branch": &options.branch,
        "head": head,
        "remote_url": &options.remote_url,
        "remote_branch": remote_branch,
        "transport": transport,
        "object_count": object_count,
        "operations": [
            {"kind": "validate_local_branch", "branch": &options.branch, "head": head},
            {"kind": "copy_object_graph", "commit": head},
            {"kind": "write_remote_branch", "branch": remote_branch, "head": head},
        ],
        "next_actions": [
            {"kind": "list_remote", "command": "codefire-rs list <cf-project-url>", "target": {"remote_url": &options.remote_url}},
        ],
    })
}

pub(super) fn request_merge_operation_plan(
    options: &RequestMergeOptions,
    source_head: &str,
    target_head: &str,
    mr_id: &str,
    transport: &str,
    target_branch: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "request-merge",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "source_url": &options.source_url,
        "target_url": &options.target_url,
        "source_head": source_head,
        "target_head": target_head,
        "merge_request_id": mr_id,
        "target_branch": target_branch,
        "transport": transport,
        "operations": [
            {"kind": "validate_remote_branches", "source": &options.source_url, "target": &options.target_url},
            {"kind": "write_merge_request", "id": mr_id},
        ],
        "next_actions": [
            {"kind": "request_review", "command": format!("codefire-rs request-review <project-url> {mr_id} --decision approve"), "target": {"merge_request_id": mr_id}},
        ],
    })
}

pub(super) fn request_review_operation_plan(
    options: &RequestReviewOptions,
    transport: &str,
    current_status: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "request-review",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "project_url": &options.project_url,
        "merge_request_id": &options.mr_id,
        "reviewer": &options.reviewer,
        "decision": &options.decision,
        "current_status": current_status,
        "transport": transport,
        "operations": [
            {"kind": "validate_merge_request", "id": &options.mr_id},
            {"kind": "append_review", "reviewer": &options.reviewer, "decision": &options.decision},
            {"kind": "update_merge_request_status", "status": if options.decision == "approve" { "approved" } else { "rejected" }},
        ],
        "next_actions": [
            {"kind": "request_apply", "command": format!("codefire-rs request-apply {} {}", options.project_url, options.mr_id), "target": {"merge_request_id": &options.mr_id}},
        ],
    })
}

pub(super) fn request_apply_operation_plan(
    options: &RequestApplyOptions,
    transport: &str,
    target_branch: &str,
    apply_head: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "request-apply",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "project_url": &options.project_url,
        "merge_request_id": &options.mr_id,
        "target_branch": target_branch,
        "head": apply_head,
        "transport": transport,
        "operations": [
            {"kind": "validate_merge_request", "id": &options.mr_id, "required_status": "approved"},
            {"kind": "validate_fast_forward", "head": apply_head},
            {"kind": "copy_object_graph", "commit": apply_head},
            {"kind": "write_remote_branch", "branch": target_branch, "head": apply_head},
            {"kind": "update_merge_request_status", "status": "applied"},
        ],
        "next_actions": [
            {"kind": "clone", "command": format!("codefire-rs clone <cf-url> {}", target_branch), "target": {"branch": target_branch}},
        ],
    })
}
