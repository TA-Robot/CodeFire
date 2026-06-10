use super::http::{
    http_json, http_remote_path, is_cf_http_url, parse_cf_http_project_url, parse_cf_http_url,
    CfHttpProjectUrl, CfHttpUrl,
};
use super::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

mod idempotency;
mod plans;
pub(crate) use idempotency::{
    load_remote_apply_idempotency, load_remote_idempotency_result, load_remote_merge_idempotency,
    load_remote_review_idempotency, load_remote_upload_idempotency, remote_apply_payload,
    remote_idempotency_key, remote_merge_payload, remote_review_payload, remote_upload_payload,
    save_remote_apply_idempotency, save_remote_idempotency_result, save_remote_merge_idempotency,
    save_remote_review_idempotency, save_remote_upload_idempotency,
};
use plans::{
    request_apply_operation_plan, request_merge_operation_plan, request_review_operation_plan,
    upload_operation_plan, UploadPlanRemote,
};

#[derive(Debug)]
pub(crate) struct UploadOptions {
    pub(crate) branch: String,
    pub(crate) remote_url: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) request_key_id: Option<String>,
    pub(crate) lock: LockOptions,
}

#[derive(Debug)]
pub(crate) struct UploadResult {
    pub(crate) head: String,
    pub(crate) plan: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemoteBranch {
    pub(crate) name: String,
    pub(crate) head: String,
}

#[derive(Debug)]
pub(crate) struct RemoteProjectOptions {
    pub(crate) project_url: String,
    pub(crate) json_output: bool,
}

#[derive(Debug)]
pub(crate) struct RequestMergeOptions {
    pub(crate) source_url: String,
    pub(crate) target_url: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) request_key_id: Option<String>,
    pub(crate) lock: LockOptions,
}

#[derive(Debug)]
pub(crate) struct RequestMergeResult {
    pub(crate) id: String,
    pub(crate) source_head: String,
    pub(crate) target_head: String,
    pub(crate) plan: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MergeRequestListItem {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) source_url: String,
    pub(crate) target_url: String,
}

#[derive(Debug)]
pub(crate) struct RequestReviewOptions {
    pub(crate) project_url: String,
    pub(crate) mr_id: String,
    pub(crate) reviewer: String,
    pub(crate) decision: String,
    pub(crate) comment: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) request_key_id: Option<String>,
    pub(crate) lock: LockOptions,
}

#[derive(Debug)]
pub(crate) struct RequestReviewResult {
    pub(crate) reviewer: String,
    pub(crate) decision: String,
    pub(crate) plan: Value,
}

#[derive(Debug)]
pub(crate) struct RequestApplyOptions {
    pub(crate) project_url: String,
    pub(crate) mr_id: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) request_key_id: Option<String>,
    pub(crate) lock: LockOptions,
}

#[derive(Debug)]
pub(crate) struct RequestApplyResult {
    pub(crate) target_branch: String,
    pub(crate) head: String,
    pub(crate) plan: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct CfUrl {
    pub(crate) project_root: PathBuf,
    pub(crate) org: String,
    pub(crate) app: String,
    pub(crate) branch: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CfProjectUrl {
    pub(crate) project_root: PathBuf,
    pub(crate) org: String,
    pub(crate) app: String,
}

pub(crate) struct RemoteGenerationLock {
    project_root: PathBuf,
    _lock: FileLock,
}

impl RemoteGenerationLock {
    pub(crate) fn acquire(
        project_root: &Path,
        options: &LockOptions,
    ) -> Result<RemoteGenerationLock, CliError> {
        let lock = FileLock::acquire_with_options(
            remote_dirs(project_root).locks.join("generation.lock"),
            options,
        )?;
        Ok(RemoteGenerationLock {
            project_root: project_root.to_path_buf(),
            _lock: lock,
        })
    }
}

pub(crate) fn upload_branch(
    start: &Path,
    options: &UploadOptions,
) -> Result<UploadResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let branch = load_branch_record(&repo_root, &options.branch)?;
    let head = required_string(&branch, &["head"])?;
    let local_objects = repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&local_objects, &head)?;
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    if registry_path.exists() {
        let registry = read_json(&registry_path)?;
        let state = required_string(&registry, &["state", "last_known"])?;
        if state != "open-clean" {
            return Err(CliError::Usage(format!(
                "branch is {state}; commit or discard before upload"
            )));
        }
    }

    if is_cf_http_url(&options.remote_url) {
        let remote = parse_cf_http_url(&options.remote_url)?;
        let transport = remote.endpoint_scheme();
        let object_records = collect_object_records(&local_objects, &head)?;
        let diagnostics = upload_object_graph_diagnostics(&object_records);
        let plan = upload_operation_plan(
            options,
            &repo_root,
            &head,
            UploadPlanRemote {
                transport,
                branch: remote.branch.as_str(),
                object_count: Some(object_records.len()),
                diagnostics,
                validations: vec![
                    json!({"kind": "remote_validation", "mode": "server_apply_path"}),
                ],
            },
        );
        if options.dry_run {
            return Ok(UploadResult { head, plan });
        }
        let actor = "local";
        let target = super::signatures::remote_request_target(
            &remote.org,
            &remote.app,
            "upload",
            Some(&remote.branch),
            None,
        );
        let mut payload = json!({
            "branch": options.branch,
            "head": head,
            "objects": object_records,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        super::signatures::attach_remote_request_signature(
            &mut payload,
            "upload",
            actor,
            &target,
            options.request_key_id.as_deref(),
        )?;
        let response = http_json(
            "POST",
            &http_remote_path(
                &remote.endpoint,
                &remote.org,
                &remote.app,
                &["branches", &remote.branch, "upload"],
            ),
            Some(payload),
        )?;
        let response_head = required_string(&response, &["head"])?;
        return Ok(UploadResult {
            head: response_head,
            plan,
        });
    }

    let remote = parse_cf_url(&options.remote_url)?;
    let remote_branch = remote.branch.clone();
    let dry_run_validations = if options.dry_run {
        validate_remote_upload_dry_run_preconditions(&remote.project_root, &local_objects, &head)?
    } else {
        Vec::new()
    };
    if !options.dry_run {
        ensure_remote_layout(&remote.project_root)?;
        let actor = "local";
        let target = super::signatures::remote_request_target(
            &remote.org,
            &remote.app,
            "upload",
            Some(&remote_branch),
            None,
        );
        let request_signature = super::signatures::sign_remote_request(
            "upload",
            actor,
            &target,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &remote.project_root,
            "upload",
            actor,
            &target,
            request_signature.as_ref(),
        )?;
        super::signatures::enforce_remote_commit_signature_policy(
            &remote.project_root,
            &local_objects,
            &head,
        )?;
    }
    let idempotency_payload = remote_upload_payload(
        &remote.project_root,
        &remote_branch,
        &options.branch,
        &head,
        None,
    );
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_remote_upload_idempotency(&remote.project_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    if let Some(current) =
        read_optional_json(&remote_branch_path(&remote.project_root, &remote_branch))?
    {
        let current_head = required_string(&current, &["head"])?;
        if !is_ancestor_in_objects(&local_objects, &current_head, &head)? {
            return Err(CliError::Usage(
                "upload rejected: remote branch is not an ancestor of local branch head"
                    .to_string(),
            ));
        }
    }
    let object_records = collect_object_records(&local_objects, &head)?;
    let diagnostics = upload_object_graph_diagnostics(&object_records);
    let plan = upload_operation_plan(
        options,
        &repo_root,
        &head,
        UploadPlanRemote {
            transport: "file",
            branch: remote_branch.as_str(),
            object_count: Some(object_records.len()),
            diagnostics,
            validations: dry_run_validations,
        },
    );
    if options.dry_run {
        return Ok(UploadResult { head, plan });
    }
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&remote.project_root)
            .locks
            .join(format!("branch-{}.lock", ref_file_name(&remote_branch))),
        &options.lock,
    )?;
    let generation_lock = RemoteGenerationLock::acquire(&remote.project_root, &options.lock)?;
    let generation = next_remote_generation(&generation_lock)?;
    copy_object_graph(
        &local_objects,
        &remote_dirs(&remote.project_root).objects,
        &head,
    )?;
    write_json_atomic(
        &remote_branch_path(&remote.project_root, &remote_branch),
        &json!({
            "version": 1,
            "name": remote_branch,
            "head": head,
            "generation": generation,
            "uploaded_from": options.branch,
            "uploaded_by": "local",
            "updated_at": now_iso_utc(),
        }),
    )?;
    let result = UploadResult { head, plan };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_remote_upload_idempotency(&remote.project_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn list_remote_branches(project_url: &str) -> Result<Vec<RemoteBranch>, CliError> {
    if is_cf_http_url(project_url) {
        let project = parse_cf_http_project_url(project_url)?;
        return list_http_remote_branches(&project);
    }
    let project = parse_cf_project_url(project_url)?;
    let dirs = remote_dirs(&project.project_root);
    if !dirs.branches.exists() {
        return Err(CliError::Usage(format!(
            "remote project not found: {project_url}"
        )));
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dirs.branches)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut branches = Vec::with_capacity(paths.len());
    for path in paths {
        let branch = read_json(&path)?;
        let name = required_string(&branch, &["name"])?;
        let head = required_string(&branch, &["head"])?;
        codefire_store::validate_sealed_commit(&dirs.objects, &head)?;
        branches.push(RemoteBranch { name, head });
    }
    Ok(branches)
}

fn list_http_remote_branches(project: &CfHttpProjectUrl) -> Result<Vec<RemoteBranch>, CliError> {
    let response = http_json(
        "GET",
        &http_remote_path(&project.endpoint, &project.org, &project.app, &["branches"]),
        None,
    )?;
    let branches = response
        .get("branches")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("HTTP remote returned an invalid branch list".to_string())
        })?;
    branches
        .iter()
        .map(|branch| {
            Ok(RemoteBranch {
                name: required_string(branch, &["name"])?,
                head: required_string(branch, &["head"])?,
            })
        })
        .collect()
}

fn load_http_remote_branch(remote: &CfHttpUrl) -> Result<RemoteBranch, CliError> {
    let project = CfHttpProjectUrl {
        endpoint: remote.endpoint.clone(),
        org: remote.org.clone(),
        app: remote.app.clone(),
    };
    list_http_remote_branches(&project)?
        .into_iter()
        .find(|branch| branch.name == remote.branch)
        .ok_or_else(|| CliError::Usage(format!("remote branch not found: {}", remote.branch)))
}

pub(crate) fn request_merge(options: &RequestMergeOptions) -> Result<RequestMergeResult, CliError> {
    if is_cf_http_url(&options.source_url) || is_cf_http_url(&options.target_url) {
        if !is_cf_http_url(&options.source_url) || !is_cf_http_url(&options.target_url) {
            return Err(CliError::Usage(
                "request-merge requires both URLs to use the same remote transport".to_string(),
            ));
        }
        let source = parse_cf_http_url(&options.source_url)?;
        let target = parse_cf_http_url(&options.target_url)?;
        if options.dry_run {
            let source_branch = load_http_remote_branch(&source)?;
            let target_branch = load_http_remote_branch(&target)?;
            let plan = request_merge_operation_plan(
                options,
                &source_branch.head,
                &target_branch.head,
                "pending",
                target.endpoint_scheme(),
                &target.branch,
            );
            return Ok(RequestMergeResult {
                id: String::new(),
                source_head: source_branch.head,
                target_head: target_branch.head,
                plan,
            });
        }
        let actor = "local";
        let target_string = super::signatures::remote_request_target(
            &target.org,
            &target.app,
            "request_merge",
            None,
            None,
        );
        let mut payload = json!({
            "source_url": options.source_url,
            "target_url": options.target_url,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        super::signatures::attach_remote_request_signature(
            &mut payload,
            "request_merge",
            actor,
            &target_string,
            options.request_key_id.as_deref(),
        )?;
        let response = http_json(
            "POST",
            &http_remote_path(
                &target.endpoint,
                &target.org,
                &target.app,
                &["merge-requests"],
            ),
            Some(payload),
        )?;
        let source_head = required_string(&response, &["source"])?;
        let target_head = required_string(&response, &["target"])?;
        let mr_id = required_string(&response, &["id"])?;
        let plan = request_merge_operation_plan(
            options,
            &source_head,
            &target_head,
            &mr_id,
            target.endpoint_scheme(),
            &target.branch,
        );
        return Ok(RequestMergeResult {
            id: mr_id,
            source_head,
            target_head,
            plan,
        });
    }
    let source = parse_cf_url(&options.source_url)?;
    let target = parse_cf_url(&options.target_url)?;
    if !options.dry_run {
        ensure_remote_layout(&target.project_root)?;
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &target.org,
            &target.app,
            "request_merge",
            None,
            None,
        );
        let request_signature = super::signatures::sign_remote_request(
            "request_merge",
            actor,
            &request_target,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &target.project_root,
            "request_merge",
            actor,
            &request_target,
            request_signature.as_ref(),
        )?;
    }
    let source_branch = load_remote_branch(&source)?;
    let target_branch = load_remote_branch(&target)?;
    let source_head = required_string(&source_branch, &["head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&source.project_root).objects,
        &source_head,
    )?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&target.project_root).objects,
        &target_head,
    )?;
    let idempotency_payload = remote_merge_payload(
        &target.project_root,
        &options.source_url,
        &source_head,
        &options.target_url,
        &target_head,
    );
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_remote_merge_idempotency(&target.project_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let mut mr_payload = json!({
        "version": 1,
        "source_url": options.source_url,
        "source_head": source_head,
        "target_url": options.target_url,
        "target_head_at_request": target_head,
        "status": "open",
        "created_by": "local",
        "created_at": now_iso_utc(),
    });
    let mr_id = format!(
        "MR-{}",
        &sha256_hex(&codefire_store::canonical_json(&mr_payload)?)[..12]
    );
    mr_payload["id"] = Value::String(mr_id.clone());
    let plan = request_merge_operation_plan(
        options,
        &source_head,
        &target_head,
        &mr_id,
        "file",
        &target.branch,
    );
    if options.dry_run {
        return Ok(RequestMergeResult {
            id: mr_id,
            source_head,
            target_head,
            plan,
        });
    }
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&target.project_root)
            .locks
            .join("merge-requests.lock"),
        &options.lock,
    )?;
    write_json_atomic(
        &remote_dirs(&target.project_root)
            .merge_requests
            .join(format!("{mr_id}.json")),
        &mr_payload,
    )?;
    let result = RequestMergeResult {
        id: mr_id,
        source_head: required_string(&mr_payload, &["source_head"])?,
        target_head: required_string(&mr_payload, &["target_head_at_request"])?,
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_remote_merge_idempotency(&target.project_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn list_merge_requests(
    project_url: &str,
) -> Result<Vec<MergeRequestListItem>, CliError> {
    if is_cf_http_url(project_url) {
        let project = parse_cf_http_project_url(project_url)?;
        let response = http_json(
            "GET",
            &http_remote_path(
                &project.endpoint,
                &project.org,
                &project.app,
                &["merge-requests"],
            ),
            None,
        )?;
        let requests = response
            .get("merge_requests")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid merge request list".to_string(),
                )
            })?;
        return requests
            .iter()
            .map(|request| {
                Ok(MergeRequestListItem {
                    id: required_string(request, &["id"])?,
                    status: required_string(request, &["status"])?,
                    source_url: required_string(request, &["source_url"])?,
                    target_url: required_string(request, &["target_url"])?,
                })
            })
            .collect();
    }
    let project = parse_cf_project_url(project_url)?;
    let dirs = remote_dirs(&project.project_root);
    if !dirs.merge_requests.exists() {
        return Err(CliError::Usage(format!(
            "remote project not found: {project_url}"
        )));
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dirs.merge_requests)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut requests = Vec::with_capacity(paths.len());
    for path in paths {
        let mr = read_json(&path)?;
        validate_merge_request_record(&mr)?;
        let target = parse_cf_url(&required_string(&mr, &["target_url"])?)?;
        let current_target =
            read_optional_json(&remote_branch_path(&target.project_root, &target.branch))?;
        let mut status = mr
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("open")
            .to_string();
        if let Some(current_target) = current_target {
            let current_head = required_string(&current_target, &["head"])?;
            codefire_store::validate_sealed_commit(
                &remote_dirs(&target.project_root).objects,
                &current_head,
            )?;
            if current_head != required_string(&mr, &["target_head_at_request"])? {
                status = "stale".to_string();
            }
        }
        requests.push(MergeRequestListItem {
            id: required_string(&mr, &["id"])?,
            status,
            source_url: required_string(&mr, &["source_url"])?,
            target_url: required_string(&mr, &["target_url"])?,
        });
    }
    Ok(requests)
}

pub(crate) fn review_merge_request(
    options: &RequestReviewOptions,
) -> Result<RequestReviewResult, CliError> {
    if is_cf_http_url(&options.project_url) {
        let project = parse_cf_http_project_url(&options.project_url)?;
        let plan = request_review_operation_plan(options, project.endpoint_scheme(), "");
        if options.dry_run {
            return Ok(RequestReviewResult {
                reviewer: options.reviewer.clone(),
                decision: options.decision.clone(),
                plan,
            });
        }
        let actor = "local";
        let target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "review",
            None,
            Some(&options.mr_id),
        );
        let mut payload = json!({
            "reviewer": options.reviewer,
            "decision": options.decision,
            "comment": options.comment,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        super::signatures::attach_remote_request_signature(
            &mut payload,
            "review",
            actor,
            &target,
            options.request_key_id.as_deref(),
        )?;
        let response = http_json(
            "POST",
            &http_remote_path(
                &project.endpoint,
                &project.org,
                &project.app,
                &["merge-requests", &options.mr_id, "review"],
            ),
            Some(payload),
        )?;
        return Ok(RequestReviewResult {
            reviewer: required_string(&response, &["reviewer"])?,
            decision: required_string(&response, &["decision"])?,
            plan,
        });
    }
    let project = parse_cf_project_url(&options.project_url)?;
    if !options.dry_run {
        ensure_remote_layout(&project.project_root)?;
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "review",
            None,
            Some(&options.mr_id),
        );
        let request_signature = super::signatures::sign_remote_request(
            "review",
            actor,
            &request_target,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &project.project_root,
            "review",
            actor,
            &request_target,
            request_signature.as_ref(),
        )?;
    }
    let mr_path = remote_dirs(&project.project_root)
        .merge_requests
        .join(format!("{}.json", options.mr_id));
    let mut mr = read_optional_json(&mr_path)?
        .ok_or_else(|| CliError::Usage(format!("merge request not found: {}", options.mr_id)))?;
    validate_merge_request_record(&mr)?;
    let idempotency_payload = remote_review_payload(
        &project.project_root,
        &mr,
        &options.mr_id,
        &options.reviewer,
        &options.decision,
        &options.comment,
    )?;
    if options.dry_run {
        reject_stale_merge_request_read_only(&mr)?;
        let plan =
            request_review_operation_plan(options, "file", &required_string(&mr, &["status"])?);
        return Ok(RequestReviewResult {
            reviewer: options.reviewer.clone(),
            decision: options.decision.clone(),
            plan,
        });
    }
    if let Some(key) = options.idempotency_key.as_deref() {
        if let Some(result) =
            load_remote_review_idempotency(&project.project_root, key, &idempotency_payload)?
        {
            return Ok(result);
        }
    }
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&project.project_root).locks.join(format!(
            "merge-request-{}.lock",
            ref_file_name(&options.mr_id)
        )),
        &options.lock,
    )?;
    reject_stale_merge_request(&mut mr, &mr_path)?;
    let plan = request_review_operation_plan(options, "file", &required_string(&mr, &["status"])?);
    let review = json!({
        "reviewer": options.reviewer,
        "decision": options.decision,
        "comment": options.comment,
        "reviewed_at": now_iso_utc(),
    });
    let mut reviews = mr
        .get("reviews")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    reviews.push(review);
    mr["reviews"] = Value::Array(reviews);
    mr["status"] = Value::String(if options.decision == "approve" {
        "approved".to_string()
    } else {
        "rejected".to_string()
    });
    write_json_atomic(&mr_path, &mr)?;
    let result = RequestReviewResult {
        reviewer: options.reviewer.clone(),
        decision: options.decision.clone(),
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_remote_review_idempotency(&project.project_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn apply_merge_request(
    options: &RequestApplyOptions,
) -> Result<RequestApplyResult, CliError> {
    if is_cf_http_url(&options.project_url) {
        let project = parse_cf_http_project_url(&options.project_url)?;
        let plan = request_apply_operation_plan(options, project.endpoint_scheme(), "", "");
        if options.dry_run {
            return Ok(RequestApplyResult {
                target_branch: String::new(),
                head: String::new(),
                plan,
            });
        }
        let actor = "local";
        let target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "apply",
            None,
            Some(&options.mr_id),
        );
        let mut payload = json!({
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        super::signatures::attach_remote_request_signature(
            &mut payload,
            "apply",
            actor,
            &target,
            options.request_key_id.as_deref(),
        )?;
        let response = http_json(
            "POST",
            &http_remote_path(
                &project.endpoint,
                &project.org,
                &project.app,
                &["merge-requests", &options.mr_id, "apply"],
            ),
            Some(payload),
        )?;
        return Ok(RequestApplyResult {
            target_branch: required_string(&response, &["target_branch"])?,
            head: required_string(&response, &["head"])?,
            plan,
        });
    }
    let project = parse_cf_project_url(&options.project_url)?;
    if !options.dry_run {
        ensure_remote_layout(&project.project_root)?;
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "apply",
            None,
            Some(&options.mr_id),
        );
        let request_signature = super::signatures::sign_remote_request(
            "apply",
            actor,
            &request_target,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &project.project_root,
            "apply",
            actor,
            &request_target,
            request_signature.as_ref(),
        )?;
    }
    let dirs = remote_dirs(&project.project_root);
    let mr_path = dirs.merge_requests.join(format!("{}.json", options.mr_id));
    let mut mr = read_optional_json(&mr_path)?
        .ok_or_else(|| CliError::Usage(format!("merge request not found: {}", options.mr_id)))?;
    validate_merge_request_record(&mr)?;
    let idempotency_payload = remote_apply_payload(&project.project_root, &mr, &options.mr_id)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_remote_apply_idempotency(&project.project_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    if mr.get("status").and_then(Value::as_str) != Some("approved") {
        return Err(CliError::Usage(format!(
            "merge request must be approved before apply: {}",
            options.mr_id
        )));
    }
    let source = parse_cf_url(&required_string(&mr, &["source_url"])?)?;
    let target = parse_cf_url(&required_string(&mr, &["target_url"])?)?;
    let source_branch = load_remote_branch(&source)?;
    let target_branch = load_remote_branch(&target)?;
    let source_head = required_string(&source_branch, &["head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    let target_head_at_request = required_string(&mr, &["target_head_at_request"])?;
    if target_head != target_head_at_request {
        if options.dry_run {
            return Err(CliError::Usage(format!(
                "merge request is stale: {}",
                options.mr_id
            )));
        }
        mr["status"] = Value::String("stale".to_string());
        write_json_atomic(&mr_path, &mr)?;
        return Err(CliError::Usage(format!(
            "merge request is stale: {}",
            options.mr_id
        )));
    }
    let source_objects = remote_dirs(&source.project_root).objects;
    let target_objects = remote_dirs(&target.project_root).objects;
    codefire_store::validate_sealed_commit(&source_objects, &source_head)?;
    codefire_store::validate_sealed_commit(&target_objects, &target_head)?;
    super::signatures::enforce_remote_commit_signature_policy(
        &target.project_root,
        &source_objects,
        &source_head,
    )?;
    if !is_ancestor_in_objects(&source_objects, &target_head, &source_head)? {
        return Err(CliError::Usage(
            "request apply rejected: source is not a fast-forward of target".to_string(),
        ));
    }
    let plan = request_apply_operation_plan(options, "file", &target.branch, &source_head);
    if options.dry_run {
        return Ok(RequestApplyResult {
            target_branch: target.branch,
            head: source_head,
            plan,
        });
    }
    let _lock = FileLock::acquire_with_options(
        remote_dirs(&target.project_root)
            .locks
            .join(format!("branch-{}.lock", ref_file_name(&target.branch))),
        &options.lock,
    )?;
    let current_target = load_remote_branch(&target)?;
    if required_string(&current_target, &["head"])? != target_head_at_request {
        mr["status"] = Value::String("stale".to_string());
        write_json_atomic(&mr_path, &mr)?;
        return Err(CliError::Usage(format!(
            "merge request is stale: {}",
            options.mr_id
        )));
    }
    let generation_lock = RemoteGenerationLock::acquire(&target.project_root, &options.lock)?;
    let generation = next_remote_generation(&generation_lock)?;
    copy_object_graph(&source_objects, &target_objects, &source_head)?;
    write_json_atomic(
        &remote_branch_path(&target.project_root, &target.branch),
        &json!({
            "version": 1,
            "name": target.branch,
            "head": source_head,
            "generation": generation,
            "applied_from": required_string(&mr, &["source_url"])?,
            "applied_by": "local",
            "updated_at": now_iso_utc(),
        }),
    )?;
    mr["status"] = Value::String("applied".to_string());
    mr["applied_at"] = Value::String(now_iso_utc());
    mr["applied_by"] = Value::String("local".to_string());
    mr["applied_head"] = Value::String(source_head.clone());
    write_json_atomic(&mr_path, &mr)?;
    let result = RequestApplyResult {
        target_branch: target.branch,
        head: source_head,
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_remote_apply_idempotency(&project.project_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct RemoteDirs {
    pub(crate) objects: PathBuf,
    pub(crate) branches: PathBuf,
    pub(crate) merge_requests: PathBuf,
    pub(crate) locks: PathBuf,
    pub(crate) idempotency: PathBuf,
}

pub(crate) fn remote_dirs(project_root: &Path) -> RemoteDirs {
    RemoteDirs {
        objects: project_root.join("objects"),
        branches: project_root.join("branches"),
        merge_requests: project_root.join("merge_requests"),
        locks: project_root.join("locks"),
        idempotency: project_root.join("idempotency"),
    }
}

pub(crate) fn ensure_remote_layout(project_root: &Path) -> Result<(), CliError> {
    let dirs = remote_dirs(project_root);
    for path in [
        project_root.to_path_buf(),
        dirs.objects.clone(),
        dirs.branches,
        dirs.merge_requests,
        dirs.locks,
        dirs.idempotency,
        project_root.join("audit"),
    ] {
        fs::create_dir_all(path)?;
    }
    for subdir in [
        "blobs",
        "content_manifests",
        "atom_indexes",
        "trace_graphs",
        "fire_ledgers",
        "resolution_ledgers",
        "verifications",
        "policies",
        "artifact_refs",
        "evidence",
        "commits",
        "branches",
    ] {
        fs::create_dir_all(dirs.objects.join(subdir))?;
    }
    Ok(())
}

fn validate_remote_upload_dry_run_preconditions(
    project_root: &Path,
    local_objects: &Path,
    head: &str,
) -> Result<Vec<Value>, CliError> {
    if !project_root.exists() {
        return Ok(vec![
            json!({"kind": "remote_project_root", "status": "missing_would_create", "path": project_root}),
            json!({"kind": "remote_layout", "status": "planned_create"}),
        ]);
    }
    validate_existing_remote_layout(project_root)?;
    super::signatures::enforce_remote_commit_signature_policy(project_root, local_objects, head)?;
    Ok(vec![
        json!({"kind": "remote_project_root", "status": "exists", "path": project_root}),
        json!({"kind": "remote_layout", "status": "validated"}),
        json!({"kind": "remote_commit_signature_policy", "status": "validated"}),
    ])
}

fn validate_existing_remote_layout(project_root: &Path) -> Result<(), CliError> {
    if !project_root.is_dir() {
        return Err(CliError::InvalidRepository(format!(
            "remote layout invalid: project root is not a directory: {}",
            project_root.display()
        )));
    }
    let dirs = remote_dirs(project_root);
    for path in [
        dirs.objects.clone(),
        dirs.branches,
        dirs.merge_requests,
        dirs.locks,
        dirs.idempotency,
    ] {
        if !path.is_dir() {
            return Err(CliError::InvalidRepository(format!(
                "remote layout invalid: missing directory {}",
                path.display()
            )));
        }
    }
    for subdir in codefire_store::known_object_subdirs() {
        let path = dirs.objects.join(subdir);
        if !path.is_dir() {
            return Err(CliError::InvalidRepository(format!(
                "remote layout invalid: missing object directory {}",
                path.display()
            )));
        }
    }
    read_optional_json(&project_root.join("server_policy.json"))?;
    Ok(())
}

fn lock_options_json(options: &LockOptions) -> Value {
    json!({
        "wait": options.wait,
        "timeout_ms": options.timeout_ms,
    })
}

pub(crate) fn parse_cf_url(url: &str) -> Result<CfUrl, CliError> {
    if !url.starts_with("cf://") {
        return Err(CliError::Usage(format!("not a CodeFire remote URL: {url}")));
    }
    let raw = &url["cf://".len()..];
    if raw.is_empty() {
        return Err(CliError::Usage("remote URL is empty".to_string()));
    }
    let parts = raw
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 4 {
        return Err(CliError::Usage(
            "remote URL must be cf://<server-path>/<org>/<app>/<branch>".to_string(),
        ));
    }
    let branch_part = parts[parts.len() - 1];
    let app_part = parts[parts.len() - 2];
    let org_part = parts[parts.len() - 3];
    let suffix = format!("/{org_part}/{app_part}/{branch_part}");
    let mut server_raw = raw
        .strip_suffix(&suffix)
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();
    if server_raw.is_empty() {
        server_raw = ".".to_string();
    }
    let server_root = absolute_path(&PathBuf::from(server_raw))?;
    let org = percent_decode(org_part)?;
    let app = percent_decode(app_part)?;
    Ok(CfUrl {
        project_root: server_root
            .join(".codefire-server")
            .join("projects")
            .join(&org)
            .join(&app),
        org,
        app,
        branch: percent_decode(branch_part)?,
    })
}

pub(crate) fn parse_cf_project_url(url: &str) -> Result<CfProjectUrl, CliError> {
    if !url.starts_with("cf://") {
        return Err(CliError::Usage(format!("not a CodeFire remote URL: {url}")));
    }
    let raw = &url["cf://".len()..];
    let parts = raw
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(CliError::Usage(
            "remote project URL must be cf://<server-path>/<org>/<app>".to_string(),
        ));
    }
    let app_part = parts[parts.len() - 1];
    let org_part = parts[parts.len() - 2];
    let suffix = format!("/{org_part}/{app_part}");
    let mut server_raw = raw
        .strip_suffix(&suffix)
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();
    if server_raw.is_empty() {
        server_raw = ".".to_string();
    }
    let server_root = absolute_path(&PathBuf::from(server_raw))?;
    let org = percent_decode(org_part)?;
    let app = percent_decode(app_part)?;
    Ok(CfProjectUrl {
        project_root: server_root
            .join(".codefire-server")
            .join("projects")
            .join(&org)
            .join(&app),
        org,
        app,
    })
}

pub(crate) fn percent_decode(value: &str) -> Result<String, CliError> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hi = bytes.get(index + 1).copied().ok_or_else(|| {
                CliError::Usage(format!("invalid percent escape in remote URL: {value}"))
            })?;
            let lo = bytes.get(index + 2).copied().ok_or_else(|| {
                CliError::Usage(format!("invalid percent escape in remote URL: {value}"))
            })?;
            output.push(hex_value(hi)? << 4 | hex_value(lo)?);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output)
        .map_err(|_| CliError::Usage(format!("remote URL component is not UTF-8: {value}")))
}

fn hex_value(byte: u8) -> Result<u8, CliError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(CliError::Usage(
            "invalid percent escape in remote URL".to_string(),
        )),
    }
}

pub(crate) fn remote_branch_path(project_root: &Path, branch: &str) -> PathBuf {
    remote_dirs(project_root)
        .branches
        .join(format!("{}.json", ref_file_name(branch)))
}

pub(crate) fn load_remote_branch(remote: &CfUrl) -> Result<Value, CliError> {
    read_optional_json(&remote_branch_path(&remote.project_root, &remote.branch))?
        .ok_or_else(|| CliError::Usage(format!("remote branch not found: {}", remote.branch)))
}

pub(crate) fn read_optional_json(path: &Path) -> Result<Option<Value>, CliError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::Io(error)),
    }
}

pub(crate) fn copy_object_graph(
    src_objects: &Path,
    dest_objects: &Path,
    root_id: &str,
) -> Result<(), CliError> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![root_id.to_string()];
    while let Some(object_id) = stack.pop() {
        if !seen.insert(object_id.clone()) {
            continue;
        }
        let payload = codefire_store::read_object(src_objects, &object_id)?;
        for reference in object_references(&payload) {
            stack.push(reference);
        }
        copy_object_record(src_objects, dest_objects, &object_id)?;
    }
    Ok(())
}

fn copy_object_record(
    src_objects: &Path,
    dest_objects: &Path,
    object_id: &str,
) -> Result<(), CliError> {
    let record = codefire_store::read_object_record(src_objects, object_id)?;
    let subdir = codefire_store::object_subdir(&record.type_tag).ok_or_else(|| {
        CliError::InvalidRepository(format!("unknown object type: {}", record.type_tag))
    })?;
    let dest = dest_objects.join(subdir).join(format!("{object_id}.json"));
    if dest.exists() {
        let existing = codefire_store::read_object_record(dest_objects, object_id)?;
        if existing != record {
            return Err(CliError::InvalidRepository(format!(
                "remote object collision: {object_id}"
            )));
        }
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&dest)?;
    file.write_all(serde_json::to_string_pretty(&record)?.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub(crate) fn collect_object_records(
    objects: &Path,
    root_id: &str,
) -> Result<Vec<Value>, CliError> {
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack = vec![root_id.to_string()];
    while let Some(object_id) = stack.pop() {
        if !seen.insert(object_id.clone()) {
            continue;
        }
        let record = codefire_store::read_object_record(objects, &object_id)?;
        for reference in object_references(&record.payload) {
            stack.push(reference);
        }
        records.push(serde_json::to_value(record)?);
    }
    records.sort_by_key(|record| {
        record
            .get("object_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    });
    Ok(records)
}

pub(crate) fn upload_object_graph_diagnostics(records: &[Value]) -> Vec<Value> {
    records
        .iter()
        .filter_map(|record| {
            let payload = record.get("payload")?;
            if payload.get("type").and_then(Value::as_str) != Some("artifact_ref") {
                return None;
            }
            let path = payload.get("path").and_then(Value::as_str)?;
            let path_kind = payload.get("path_kind").and_then(Value::as_str);
            if path_kind == Some("repo_relative") || path_kind == Some("redacted") {
                return None;
            }
            if !looks_like_absolute_path(path) {
                return None;
            }
            Some(json!({
                "kind": "artifact_path_sensitive",
                "severity": "warning",
                "message": "artifact_ref contains an absolute local path; recreate the evidence to store a repo-relative or redacted artifact path before remote upload",
                "object_id": record.get("object_id").and_then(Value::as_str),
            }))
        })
        .collect()
}

fn looks_like_absolute_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('\\') || path.as_bytes().get(1) == Some(&b':')
}

pub(crate) fn write_object_records<I>(objects: &Path, records: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = Value>,
{
    ensure_object_dirs(objects)?;
    for record_value in records {
        let record: codefire_store::ObjectRecord = serde_json::from_value(record_value)?;
        codefire_store::validate_object_record(&record, Some(&record.object_id), None)?;
        let subdir = codefire_store::object_subdir(&record.type_tag).ok_or_else(|| {
            CliError::InvalidRepository(format!("unknown object type: {}", record.type_tag))
        })?;
        let path = objects
            .join(subdir)
            .join(format!("{}.json", record.object_id));
        if path.exists() {
            let existing = codefire_store::read_object_record(objects, &record.object_id)?;
            if existing != record {
                return Err(CliError::InvalidRepository(format!(
                    "object collision: {}",
                    record.object_id
                )));
            }
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(serde_json::to_string_pretty(&record)?.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

pub(crate) fn ensure_object_dirs(objects: &Path) -> Result<(), CliError> {
    for subdir in [
        "blobs",
        "content_manifests",
        "atom_indexes",
        "trace_graphs",
        "fire_ledgers",
        "resolution_ledgers",
        "verifications",
        "policies",
        "commits",
        "branches",
    ] {
        fs::create_dir_all(objects.join(subdir))?;
    }
    Ok(())
}

pub(crate) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), CliError> {
    if !src.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&src_path, &dest_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src_path, dest_path)?;
        }
    }
    Ok(())
}

fn object_references(payload: &Value) -> Vec<String> {
    match payload.get("type").and_then(Value::as_str) {
        Some("commit") => {
            let mut refs = Vec::new();
            if let Some(parents) = payload.get("parents").and_then(Value::as_array) {
                refs.extend(
                    parents
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|value| value.starts_with("CF-"))
                        .map(str::to_string),
                );
            }
            if let Some(roots) = payload.get("roots").and_then(Value::as_object) {
                refs.extend(
                    roots
                        .values()
                        .filter_map(Value::as_str)
                        .filter(|value| value.starts_with("CF-"))
                        .map(str::to_string),
                );
            }
            refs
        }
        Some("content_manifest") => payload
            .get("entries")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.get("blob").and_then(Value::as_str))
            .map(str::to_string)
            .collect(),
        Some("resolution_ledger") => payload
            .get("resolutions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|resolution| {
                resolution
                    .get("evidence_refs")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter(|value| value.starts_with("CF-"))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect(),
        Some("evidence") => payload
            .get("artifact_ref")
            .and_then(Value::as_str)
            .filter(|value| value.starts_with("CF-"))
            .map(|value| vec![value.to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub(crate) fn next_remote_generation(lock: &RemoteGenerationLock) -> Result<u64, CliError> {
    let path = lock.project_root.join("gc_state.json");
    let generation = read_optional_json(&path)?
        .and_then(|value| value.get("current_generation").and_then(Value::as_u64))
        .unwrap_or(0)
        + 1;
    write_json_atomic(&path, &json!({"current_generation": generation}))?;
    Ok(generation)
}

pub(crate) fn validate_merge_request_record(mr: &Value) -> Result<(), CliError> {
    let source = parse_cf_url(&required_string(mr, &["source_url"])?)?;
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&source.project_root).objects,
        &source_head,
    )?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&target.project_root).objects,
        &target_head,
    )?;
    if let Some(applied_head) = mr.get("applied_head").and_then(Value::as_str) {
        codefire_store::validate_sealed_commit(
            &remote_dirs(&target.project_root).objects,
            applied_head,
        )?;
    }
    Ok(())
}

fn reject_stale_merge_request(mr: &mut Value, mr_path: &Path) -> Result<(), CliError> {
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    if let Some(current_target) =
        read_optional_json(&remote_branch_path(&target.project_root, &target.branch))?
    {
        let current_head = required_string(&current_target, &["head"])?;
        if current_head != required_string(mr, &["target_head_at_request"])? {
            mr["status"] = Value::String("stale".to_string());
            write_json_atomic(mr_path, mr)?;
            return Err(CliError::Usage(format!(
                "merge request is stale: {}",
                required_string(mr, &["id"])?
            )));
        }
    }
    Ok(())
}

fn reject_stale_merge_request_read_only(mr: &Value) -> Result<(), CliError> {
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    if let Some(current_target) =
        read_optional_json(&remote_branch_path(&target.project_root, &target.branch))?
    {
        let current_head = required_string(&current_target, &["head"])?;
        if current_head != required_string(mr, &["target_head_at_request"])? {
            return Err(CliError::Usage(format!(
                "merge request is stale: {}",
                required_string(mr, &["id"])?
            )));
        }
    }
    Ok(())
}

pub(crate) fn is_ancestor_in_objects(
    objects: &Path,
    ancestor: &str,
    commit_id: &str,
) -> Result<bool, CliError> {
    Ok(ancestor_distances(objects, commit_id)?.contains_key(ancestor))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    codefire_util::sha256_hex(bytes)
}
