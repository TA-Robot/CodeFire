use super::http::{
    http_json, http_remote_path, is_cf_http_url, parse_cf_http_project_url, parse_cf_http_url,
    CfHttpProjectUrl, CfHttpUrl,
};
use super::*;
use crate::compatibility::validate_record_version;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

mod doctor;
mod idempotency;
mod locks;
mod operations;
mod plans;
mod project;
mod publication;
pub(crate) use doctor::{
    diagnose_remote_project_with_work_and_poll, print_remote_doctor_report,
    remote_doctor_data_json, remote_doctor_diagnostics_json, run_remote_doctor,
};
pub(crate) use idempotency::remote_idempotency_key;
pub(crate) use locks::{
    ArmedRemoteProjectBootstrap, RemoteProjectExclusivePreparation, RemoteProjectGate,
    RemoteProjectGates,
};
pub(crate) use operations::{
    apply_publication, apply_result, apply_writes, request_merge_publication, request_merge_result,
    review_publication, review_result, review_write, upload_publication, upload_result,
    upload_writes, ApplyPostimages, ApplyPublicationIdentity, MergePublicationIdentity,
    ReviewPublicationIdentity, UploadPostimages, UploadPublicationIdentity,
};
use plans::{
    request_apply_operation_plan, request_merge_operation_plan, request_review_operation_plan,
    upload_operation_plan, UploadPlanRemote,
};
pub(crate) use project::{
    ensure_remote_project_layout, remote_project_stable_id, validate_remote_component,
    validated_remote_component, BootstrapStageFootprint, RemoteAuthority,
    REMOTE_INTERNAL_HASH_PREFIX,
};
#[cfg(test)]
pub(crate) use project::{
    inject_ledger_bootstrap_rename_failure_once, ledger_bootstrap_rename_attempts,
    reset_ledger_bootstrap_rename_attempts,
};
pub(crate) use publication::{
    execute_json_publication, missing_upload_publication_footprint,
    preflight_publication_signature_with_recovery_and_poll,
    prepare_json_publication_plan_lazy_with_nonce_and_provisional_roots, replay_json_publication,
    validate_publication_signature_with_recovery_and_poll,
    validate_publication_signature_with_recovery_evidence_and_poll, AuthorityWrite,
    PreparedJsonPublication, PublicationRequest,
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
    pub(crate) page: crate::pagination::PageRequest,
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
    let object_records = collect_object_records(&local_objects, &head)?;
    if !options.dry_run {
        let actor = "local";
        let target = super::signatures::remote_request_target(
            &remote.org,
            &remote.app,
            "upload",
            Some(&remote_branch),
            None,
        );
        let payload = json!({
            "branch": options.branch,
            "head": &head,
            "objects": &object_records,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        let request_signature = super::signatures::sign_remote_request(
            "upload",
            actor,
            &target,
            &payload,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &remote.project_root,
            "upload",
            actor,
            &target,
            &payload,
            request_signature.as_ref(),
        )?;
        ensure_remote_layout(&remote.project_root)?;
        super::signatures::enforce_remote_commit_signature_policy(
            &remote.project_root,
            &local_objects,
            &head,
        )?;
    }
    let _project_gate = if options.dry_run {
        None
    } else {
        Some(RemoteProjectGate::acquire_exclusive(
            &remote.project_root,
            &options.lock,
        )?)
    };
    let branch_before = read_optional_remote_branch(&remote.project_root, &remote_branch)?;
    if options.dry_run {
        if let Some(current) = branch_before.as_ref() {
            let current_head = required_string(current, &["head"])?;
            if !is_ancestor_in_objects(&local_objects, &current_head, &head)? {
                return Err(CliError::Usage(
                    "upload rejected: remote branch is not an ancestor of local branch head"
                        .to_string(),
                ));
            }
        }
    }
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
    let _branch_lock = FileLock::acquire_with_options(
        remote_dirs(&remote.project_root)
            .locks
            .join(format!("branch-{}.lock", ref_file_name(&remote_branch))),
        &options.lock,
    )?;
    copy_object_graph(
        &local_objects,
        &remote_dirs(&remote.project_root).objects,
        &head,
    )?;
    let stable_project_id = remote_project_stable_id(&remote.project_root)?;
    let object_set_digest = source_object_set_digest(&object_records)?;
    let publication_request = upload_publication(
        "local",
        options.idempotency_key.clone(),
        &UploadPublicationIdentity {
            stable_project_id: &stable_project_id,
            remote_branch: &remote_branch,
            local_branch: &options.branch,
            head: &head,
            object_set_digest: &object_set_digest,
        },
    );
    let gate = _project_gate.as_ref().ok_or_else(|| {
        CliError::InvalidRepository("remote upload project gate is missing".to_string())
    })?;
    if let Some(published) =
        replay_json_publication(gate, &remote.project_root, &publication_request)?
    {
        return Ok(UploadResult {
            head: required_string(&published, &["head"])?,
            plan,
        });
    }
    if let Some(current) = branch_before.as_ref() {
        let current_head = required_string(current, &["head"])?;
        if !is_ancestor_in_objects(&local_objects, &current_head, &head)? {
            return Err(CliError::Usage(
                "upload rejected: remote branch is not an ancestor of local branch head"
                    .to_string(),
            ));
        }
    }
    let generation_before = read_optional_json(&remote.project_root.join("gc_state.json"))?;
    let generation = generation_before
        .as_ref()
        .and_then(|value| value.get("current_generation"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let branch_after = json!({
        "version": 1,
        "name": remote_branch,
        "head": head,
        "generation": generation,
        "uploaded_from": options.branch,
        "uploaded_by": "local",
        "updated_at": now_iso_utc(),
    });
    let immutable_result = upload_result(&remote_branch, &head);
    let published = execute_json_publication(
        gate,
        &remote.project_root,
        &publication_request,
        &upload_writes(UploadPostimages {
            remote_branch: &remote_branch,
            generation_before,
            generation_after: json!({"current_generation": generation}),
            branch_before,
            branch_after,
        }),
        &immutable_result,
    )?;
    Ok(UploadResult {
        head: required_string(&published, &["head"])?,
        plan,
    })
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
    let _project_gate =
        RemoteProjectGate::acquire_shared(&project.project_root, &LockOptions::default())?;
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
        validate_remote_branch_record(&branch)?;
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
            validate_remote_branch_record(branch)?;
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

pub(crate) struct MergeRequestSubjectInput<'a> {
    pub(crate) mr_id: &'a str,
    pub(crate) source_project_root: &'a Path,
    pub(crate) target_project_root: &'a Path,
    pub(crate) source_branch: &'a str,
    pub(crate) target_branch: &'a str,
    pub(crate) source_head: &'a str,
    pub(crate) target_head: &'a str,
    pub(crate) source_object_set_digest: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) struct MergeRequestSubjectRoots<'a> {
    source_project_root: &'a Path,
    target_project_root: &'a Path,
    source_branch: &'a str,
    target_branch: &'a str,
}

impl<'a> MergeRequestSubjectRoots<'a> {
    pub(crate) fn new(
        source_project_root: &'a Path,
        target_project_root: &'a Path,
        source_branch: &'a str,
        target_branch: &'a str,
    ) -> Self {
        Self {
            source_project_root,
            target_project_root,
            source_branch,
            target_branch,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PlannedMergeRequestSubject {
    merge_request_id: String,
    source_branch: String,
    source_head: String,
    source_object_set_digest: String,
    source_project_id: String,
    target_branch: String,
    target_head_at_request: String,
    target_project_id: String,
    r#type: &'static str,
    version: u64,
}

#[derive(Serialize)]
pub(crate) struct PlannedLegacyReviewHistory<'a> {
    version: u64,
    reason: &'static str,
    reviews: &'a [Value],
}

pub(crate) struct MergeRequestSubjectMaterializationPlan {
    pub(crate) subject: PlannedMergeRequestSubject,
    pub(crate) subject_digest: String,
}

pub(crate) struct MergeRequestMaterializationOwnership {
    pub(crate) retained_bytes: u64,
    pub(crate) temporary_bytes: u64,
}

#[derive(Debug)]
pub(crate) struct AccountedMergeRequestSubjectMaterialization {
    subject_digest: String,
    retained_bytes: u64,
}

impl AccountedMergeRequestSubjectMaterialization {
    pub(crate) fn into_parts(self) -> (String, u64) {
        (self.subject_digest, self.retained_bytes)
    }
}

#[must_use = "planned materialization objects must be published, transferred, or released"]
pub(crate) struct PlannedMergeRequestSubjectMaterialization {
    accounted: AccountedMergeRequestSubjectMaterialization,
    object_records: Option<AccountedObjectRecords>,
}

impl PlannedMergeRequestSubjectMaterialization {
    pub(crate) fn into_parts(
        self,
    ) -> (
        AccountedMergeRequestSubjectMaterialization,
        Option<AccountedObjectRecords>,
    ) {
        (self.accounted, self.object_records)
    }
}

fn planned_merge_request_subject(
    input: &MergeRequestSubjectInput<'_>,
) -> Result<PlannedMergeRequestSubject, CliError> {
    Ok(PlannedMergeRequestSubject {
        merge_request_id: input.mr_id.to_string(),
        source_branch: input.source_branch.to_string(),
        source_head: input.source_head.to_string(),
        source_object_set_digest: input.source_object_set_digest.to_string(),
        source_project_id: project::remote_project_stable_id(input.source_project_root)?,
        target_branch: input.target_branch.to_string(),
        target_head_at_request: input.target_head.to_string(),
        target_project_id: project::remote_project_stable_id(input.target_project_root)?,
        r#type: "codefire_merge_request_subject",
        version: 1,
    })
}

pub(crate) fn merge_request_subject_for_roots(
    input: &MergeRequestSubjectInput<'_>,
) -> Result<Value, CliError> {
    serde_json::to_value(planned_merge_request_subject(input)?).map_err(CliError::from)
}

pub(crate) fn merge_request_subject_digest(subject: &Value) -> Result<String, CliError> {
    Ok(format!(
        "sha256:{}",
        sha256_hex(&codefire_store::canonical_json(subject)?)
    ))
}

fn merge_request_subject_digest_with_work_and_poll<T, F>(
    subject: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<(String, u64), CliError>
where
    T: Serialize + ?Sized,
    F: FnMut() -> Result<(), CliError>,
{
    struct SubjectDigestWriter<'a, F> {
        digest: &'a mut Sha256,
        ledger: &'a mut codefire_core::DecisionWorkLedger,
        poll: &'a mut F,
        accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
        poll_error: Option<CliError>,
    }

    impl<F> std::io::Write for SubjectDigestWriter<'_, F>
    where
        F: FnMut() -> Result<(), CliError>,
    {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if let Err(error) = (self.poll)() {
                self.poll_error = Some(error);
                return Err(std::io::Error::other(
                    "merge-request subject digest cancelled",
                ));
            }
            let amount = u64::try_from(bytes.len())
                .map_err(|_| std::io::Error::other("merge-request subject digest size overflow"))?;
            if let Err(error) = self.ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .count(codefire_core::DecisionWorkCounter::BytesHashed, amount),
            ) {
                self.accounting_error = Some(error);
                return Err(std::io::Error::other(
                    "merge-request subject digest budget exceeded",
                ));
            }
            self.digest.update(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut digest = Sha256::new();
    let mut writer = SubjectDigestWriter {
        digest: &mut digest,
        ledger,
        poll: &mut poll,
        accounting_error: None,
        poll_error: None,
    };
    let encoded = serde_json::to_writer(&mut writer, subject);
    if let Some(error) = writer.accounting_error.take() {
        return Err(decision_work_cli_error(error));
    }
    if let Some(error) = writer.poll_error.take() {
        return Err(error);
    }
    encoded?;
    drop(writer);
    const DIGEST_PLACEHOLDER: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    let retained_bytes = codefire_core::logical_retained_frame_bytes(DIGEST_PLACEHOLDER)
        .map_err(decision_work_cli_error)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )
        .map_err(decision_work_cli_error)?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let finalized = digest.finalize();
    let mut value = String::with_capacity(DIGEST_PLACEHOLDER.len());
    value.push_str("sha256:");
    for byte in finalized {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok((value, retained_bytes))
}

pub(crate) fn validate_merge_request_subject_for_roots(
    mr: &Value,
    _source_project_root: &Path,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
) -> Result<String, CliError> {
    validate_merge_request_subject_for_roots_internal(
        mr,
        target_project_root,
        source_branch,
        target_branch,
        None,
        || Ok(()),
    )
}

pub(crate) fn validate_merge_request_subject_for_roots_with_work_and_poll<F>(
    mr: &Value,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: F,
) -> Result<String, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    validate_merge_request_subject_for_roots_internal(
        mr,
        target_project_root,
        source_branch,
        target_branch,
        Some(ledger),
        poll,
    )
}

pub(crate) fn merge_request_object_set_digest_with_work_and_poll<F>(
    mr: &Value,
    target_project_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<AccountedSourceObjectSetDigest, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let source_head = required_string(mr, &["source_head"])?;
    let escrow_objects = remote_dirs(target_project_root).objects;
    let mut escrow_records =
        collect_object_records_internal(&escrow_objects, &source_head, Some(ledger), &mut poll)?;
    let digest = (|| -> Result<AccountedSourceObjectSetDigest, CliError> {
        poll()?;
        source_object_set_digest_ordered_with_poll(
            escrow_records.as_slice(),
            0..escrow_records.len(),
            ledger,
            &mut poll,
        )
    })();
    match digest {
        Ok(mut digest) => {
            if let Err(error) = escrow_records.release(ledger) {
                let mut first_error = Some(error);
                record_remote_cleanup_error(&mut first_error, digest.release(ledger));
                return Err(first_error.expect("record release error must be retained"));
            }
            Ok(digest)
        }
        Err(error) => {
            escrow_records.release(ledger)?;
            Err(error)
        }
    }
}

pub(crate) fn validate_merge_request_subject_with_object_set_digest(
    mr: &Value,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
    object_set_digest: &str,
) -> Result<String, CliError> {
    let subject = mr.get("subject").ok_or_else(|| {
        CliError::Usage(
            "legacy merge request requires a new subject-bound review before apply".to_string(),
        )
    })?;
    let subject_fields = subject
        .as_object()
        .map(|value| value.keys().map(String::as_str).collect::<BTreeSet<_>>());
    let expected_fields = BTreeSet::from([
        "type",
        "version",
        "merge_request_id",
        "source_project_id",
        "target_project_id",
        "source_branch",
        "target_branch",
        "source_head",
        "target_head_at_request",
        "source_object_set_digest",
    ]);
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    let source_project_id = subject
        .get("source_project_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let valid_source_project_id =
        source_project_id
            .strip_prefix("CF-PROJECT-")
            .is_some_and(|value| {
                value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            });
    let target_project_id = project::remote_project_stable_id(target_project_root)?;
    let valid = subject_fields.as_ref() == Some(&expected_fields)
        && subject.get("type").and_then(Value::as_str) == Some("codefire_merge_request_subject")
        && subject.get("version").and_then(Value::as_u64) == Some(1)
        && subject.get("merge_request_id") == mr.get("id")
        && valid_source_project_id
        && subject.get("target_project_id").and_then(Value::as_str)
            == Some(target_project_id.as_str())
        && subject.get("source_branch").and_then(Value::as_str) == Some(source_branch)
        && subject.get("target_branch").and_then(Value::as_str) == Some(target_branch)
        && subject.get("source_head").and_then(Value::as_str) == Some(source_head.as_str())
        && subject
            .get("target_head_at_request")
            .and_then(Value::as_str)
            == Some(target_head.as_str())
        && subject
            .get("source_object_set_digest")
            .and_then(Value::as_str)
            == Some(object_set_digest);
    if !valid {
        return Err(CliError::InvalidRepository(
            "merge request subject does not match the current request".to_string(),
        ));
    }
    let digest = merge_request_subject_digest(subject)?;
    if mr.get("subject_digest").and_then(Value::as_str) != Some(&digest) {
        return Err(CliError::InvalidRepository(
            "merge request subject digest diverged".to_string(),
        ));
    }
    Ok(digest)
}

fn validate_merge_request_subject_for_roots_internal<F>(
    mr: &Value,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
    mut ledger: Option<&mut codefire_core::DecisionWorkLedger>,
    mut poll: F,
) -> Result<String, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let source_head = required_string(mr, &["source_head"])?;
    let escrow_objects = remote_dirs(target_project_root).objects;
    let mut escrow_records = collect_object_records_internal(
        &escrow_objects,
        &source_head,
        ledger.as_deref_mut(),
        &mut poll,
    )?;
    if let Some(ledger) = ledger {
        let digest = (|| -> Result<AccountedSourceObjectSetDigest, CliError> {
            poll()?;
            source_object_set_digest_ordered_with_poll(
                escrow_records.as_slice(),
                0..escrow_records.len(),
                ledger,
                &mut poll,
            )
        })();
        let mut digest = match digest {
            Ok(digest) => digest,
            Err(error) => {
                escrow_records.release(ledger)?;
                return Err(error);
            }
        };
        if let Err(error) = escrow_records.release(ledger) {
            let mut first_error = Some(error);
            record_remote_cleanup_error(&mut first_error, digest.release(ledger));
            return Err(first_error.expect("record release error must be retained"));
        }
        let validation = validate_merge_request_subject_with_object_set_digest(
            mr,
            target_project_root,
            source_branch,
            target_branch,
            digest.as_str(),
        );
        let cleanup = digest.release(ledger);
        match (validation, cleanup) {
            (Ok(subject_digest), Ok(())) => Ok(subject_digest),
            (Err(error), Ok(())) => Err(error),
            (_, Err(error)) => Err(error),
        }
    } else {
        poll()?;
        let digest = source_object_set_digest(escrow_records.as_slice())?;
        let _ = escrow_records.into_unaccounted_records();
        validate_merge_request_subject_with_object_set_digest(
            mr,
            target_project_root,
            source_branch,
            target_branch,
            &digest,
        )
    }
}

fn validate_merge_request_subject(mr: &Value) -> Result<String, CliError> {
    let source = parse_cf_url(&required_string(mr, &["source_url"])?)?;
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    validate_merge_request_subject_for_roots(
        mr,
        &source.project_root,
        &target.project_root,
        &source.branch,
        &target.branch,
    )
}

fn materialize_merge_request_subject(
    mr: &mut Value,
    source: &CfUrl,
    target: &CfUrl,
) -> Result<String, CliError> {
    materialize_merge_request_subject_for_roots(
        mr,
        &source.project_root,
        &target.project_root,
        &source.branch,
        &target.branch,
    )
}

/// Compatibility materialization for callers that do not use decision-work
/// accounting. Accounted callers must use
/// [`materialize_merge_request_subject_for_roots_with_work_and_poll`].
pub(crate) fn materialize_merge_request_subject_for_roots(
    mr: &mut Value,
    source_project_root: &Path,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
) -> Result<String, CliError> {
    materialize_merge_request_subject_for_roots_without_work_and_poll(
        mr,
        source_project_root,
        target_project_root,
        source_branch,
        target_branch,
        || Ok(()),
    )
}

pub(crate) fn materialize_merge_request_subject_for_roots_with_work_and_poll<F, R>(
    mr: &mut Value,
    roots: MergeRequestSubjectRoots<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
    mut reserve: R,
) -> Result<AccountedMergeRequestSubjectMaterialization, CliError>
where
    F: FnMut() -> Result<(), CliError>,
    R: FnMut(
        &mut codefire_core::DecisionWorkLedger,
        &Value,
        &MergeRequestSubjectMaterializationPlan,
        Option<&PlannedLegacyReviewHistory<'_>>,
    ) -> Result<MergeRequestMaterializationOwnership, CliError>,
{
    let planned = plan_merge_request_subject_materialization_for_roots_with_work_and_poll(
        mr,
        roots,
        ledger,
        &mut poll,
        &mut reserve,
    )?;
    let (accounted, object_records) = planned.into_parts();
    if let Some(mut records) = object_records {
        if let Err(error) = records.publish_with_work_and_poll(
            &remote_dirs(roots.target_project_root).objects,
            ledger,
            &mut poll,
        ) {
            rollback_planned_merge_request_subject_materialization(mr);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    accounted.retained_bytes,
                )
                .map_err(decision_work_cli_error)?;
            return Err(error);
        }
    }
    Ok(accounted)
}

fn rollback_planned_merge_request_subject_materialization(mr: &mut Value) {
    let object = mr
        .as_object_mut()
        .expect("planned merge request materialization validated its object record");
    object.remove("subject");
    object.remove("subject_digest");
    if let Some(mut history) = object.remove("legacy_review_history") {
        let reviews = history
            .as_object_mut()
            .and_then(|history| history.get_mut("reviews"))
            .map(Value::take)
            .expect("planned legacy review history owns its original reviews");
        *object
            .get_mut("reviews")
            .expect("planned legacy materialization retained the reviews field") = reviews;
    }
}

/// Plans legacy merge-request subject materialization without publishing its
/// object closure. The returned object-record owner must be transferred to the
/// request-wide publication transaction, explicitly published by a legacy
/// compatibility caller, or released on cancellation.
pub(crate) fn plan_merge_request_subject_materialization_for_roots_with_work_and_poll<F, R>(
    mr: &mut Value,
    roots: MergeRequestSubjectRoots<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
    mut reserve: R,
) -> Result<PlannedMergeRequestSubjectMaterialization, CliError>
where
    F: FnMut() -> Result<(), CliError>,
    R: FnMut(
        &mut codefire_core::DecisionWorkLedger,
        &Value,
        &MergeRequestSubjectMaterializationPlan,
        Option<&PlannedLegacyReviewHistory<'_>>,
    ) -> Result<MergeRequestMaterializationOwnership, CliError>,
{
    let MergeRequestSubjectRoots {
        source_project_root,
        target_project_root,
        source_branch,
        target_branch,
    } = roots;
    if mr.get("subject").is_some() {
        let subject_digest = validate_merge_request_subject_for_roots_internal(
            mr,
            target_project_root,
            source_branch,
            target_branch,
            Some(ledger),
            poll,
        )?;
        return Ok(PlannedMergeRequestSubjectMaterialization {
            accounted: AccountedMergeRequestSubjectMaterialization {
                subject_digest,
                retained_bytes: 0,
            },
            object_records: None,
        });
    }
    let has_legacy_reviews = match mr.get("reviews") {
        Some(Value::Array(reviews)) if !reviews.is_empty() => {
            if mr.get("legacy_review_history").is_some() {
                return Err(CliError::InvalidRepository(
                    "legacy merge request review history is already materialized".to_string(),
                ));
            }
            true
        }
        Some(Value::Array(_)) | None => false,
        Some(_) => {
            return Err(CliError::InvalidRepository(
                "merge request reviews must be an array".to_string(),
            ));
        }
    };
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    let source_objects = remote_dirs(source_project_root).objects;
    let mut records =
        collect_object_records_internal(&source_objects, &source_head, Some(ledger), &mut poll)?;
    let preparation = (|| -> Result<_, CliError> {
        poll()?;
        let mr_id = required_string(mr, &["id"])?;
        let mut object_set_digest = source_object_set_digest_ordered_with_poll(
            records.as_slice(),
            0..records.len(),
            ledger,
            &mut poll,
        )?;
        let subject = planned_merge_request_subject(&MergeRequestSubjectInput {
            mr_id: &mr_id,
            source_project_root,
            target_project_root,
            source_branch,
            target_branch,
            source_head: &source_head,
            target_head: &target_head,
            source_object_set_digest: object_set_digest.as_str(),
        });
        let subject = match subject {
            Ok(subject) => subject,
            Err(error) => {
                let mut first_error = Some(error);
                record_remote_cleanup_error(&mut first_error, object_set_digest.release(ledger));
                return Err(first_error.expect("subject construction error must be retained"));
            }
        };
        let (subject_digest, digest_owner_bytes) =
            match merge_request_subject_digest_with_work_and_poll(&subject, ledger, &mut poll) {
                Ok(digest) => digest,
                Err(error) => {
                    let mut first_error = Some(error);
                    record_remote_cleanup_error(
                        &mut first_error,
                        object_set_digest.release(ledger),
                    );
                    return Err(
                        first_error.expect("subject digest construction error must be retained")
                    );
                }
            };
        let plan = MergeRequestSubjectMaterializationPlan {
            subject,
            subject_digest,
        };
        let legacy_review_history = has_legacy_reviews.then(|| PlannedLegacyReviewHistory {
            version: 1,
            reason: "unbound_before_subject_v1",
            reviews: mr
                .get("reviews")
                .and_then(Value::as_array)
                .expect("legacy reviews were validated before planning"),
        });
        let ownership = match reserve(ledger, mr, &plan, legacy_review_history.as_ref()) {
            Ok(ownership) => ownership,
            Err(error) => {
                let mut first_error = Some(error);
                record_remote_cleanup_error(
                    &mut first_error,
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            digest_owner_bytes,
                        )
                        .map_err(decision_work_cli_error),
                );
                record_remote_cleanup_error(&mut first_error, object_set_digest.release(ledger));
                return Err(first_error.expect("destination reservation error must be retained"));
            }
        };
        let mut cleanup_error = None;
        record_remote_cleanup_error(
            &mut cleanup_error,
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    digest_owner_bytes,
                )
                .map_err(decision_work_cli_error),
        );
        record_remote_cleanup_error(&mut cleanup_error, object_set_digest.release(ledger));
        if let Some(error) = cleanup_error {
            release_merge_request_materialization_ownership(ledger, ownership)?;
            return Err(error);
        }
        Ok((plan, ownership))
    })();
    let (plan, ownership) = match preparation {
        Ok(prepared) => prepared,
        Err(error) => {
            records.release(ledger)?;
            return Err(error);
        }
    };
    if let Err(error) = poll() {
        release_merge_request_materialization_ownership(ledger, ownership)?;
        records.release(ledger)?;
        return Err(error);
    }
    let mutation = (|| -> Result<(), CliError> {
        if has_legacy_reviews {
            let reviews = mr
                .get_mut("reviews")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "merge request reviews changed during materialization".to_string(),
                    )
                })?;
            let reviews = std::mem::take(reviews);
            mr["legacy_review_history"] = json!({
                "version": 1,
                "reason": "unbound_before_subject_v1",
                "reviews": reviews,
            });
        }
        mr["subject"] = serde_json::to_value(&plan.subject)?;
        mr["subject_digest"] = Value::String(plan.subject_digest.clone());
        Ok(())
    })();
    if let Err(error) = mutation {
        records.release(ledger)?;
        release_merge_request_materialization_ownership(ledger, ownership)?;
        return Err(error);
    }
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            ownership.temporary_bytes,
        )
        .map_err(decision_work_cli_error)?;
    Ok(PlannedMergeRequestSubjectMaterialization {
        accounted: AccountedMergeRequestSubjectMaterialization {
            subject_digest: plan.subject_digest,
            retained_bytes: ownership.retained_bytes,
        },
        object_records: Some(records),
    })
}

fn release_merge_request_materialization_ownership(
    ledger: &mut codefire_core::DecisionWorkLedger,
    ownership: MergeRequestMaterializationOwnership,
) -> Result<(), CliError> {
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            ownership.temporary_bytes,
        )
        .map_err(decision_work_cli_error)?;
    ledger
        .release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            ownership.retained_bytes,
        )
        .map_err(decision_work_cli_error)
}

fn materialize_merge_request_subject_for_roots_without_work_and_poll<F>(
    mr: &mut Value,
    source_project_root: &Path,
    target_project_root: &Path,
    source_branch: &str,
    target_branch: &str,
    mut poll: F,
) -> Result<String, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    if mr.get("subject").is_some() {
        return validate_merge_request_subject_for_roots_internal(
            mr,
            target_project_root,
            source_branch,
            target_branch,
            None,
            poll,
        );
    }
    if let Some(reviews) = mr.get("reviews").and_then(Value::as_array) {
        if !reviews.is_empty() {
            if mr.get("legacy_review_history").is_some() {
                return Err(CliError::InvalidRepository(
                    "legacy merge request review history is already materialized".to_string(),
                ));
            }
            mr["legacy_review_history"] = json!({
                "version": 1,
                "reason": "unbound_before_subject_v1",
                "reviews": reviews,
            });
            mr["reviews"] = Value::Array(Vec::new());
        }
    }
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(source_project_root).objects,
        &source_head,
    )?;
    let source_objects = remote_dirs(source_project_root).objects;
    let records = collect_object_records_internal(&source_objects, &source_head, None, &mut poll)?;
    poll()?;
    let object_set_digest = source_object_set_digest(records.as_slice())?;
    let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
        mr_id: &required_string(mr, &["id"])?,
        source_project_root,
        target_project_root,
        source_branch,
        target_branch,
        source_head: &source_head,
        target_head: &target_head,
        source_object_set_digest: &object_set_digest,
    })?;
    copy_object_graph_with_poll(
        &remote_dirs(source_project_root).objects,
        &remote_dirs(target_project_root).objects,
        &source_head,
        &mut poll,
    )?;
    let _ = records.into_unaccounted_records();
    let digest = merge_request_subject_digest(&subject)?;
    mr["subject"] = subject;
    mr["subject_digest"] = Value::String(digest.clone());
    Ok(digest)
}

pub(crate) fn merge_request_has_effective_approval(
    mr: &Value,
    subject_digest: &str,
) -> Result<bool, CliError> {
    merge_request_has_effective_approval_with_poll(mr, subject_digest, || Ok(()))
}

pub(crate) fn merge_request_has_effective_approval_with_poll<F>(
    mr: &Value,
    subject_digest: &str,
    mut poll: F,
) -> Result<bool, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let Some(reviews) = mr.get("reviews").and_then(Value::as_array) else {
        return Ok(false);
    };
    for review in reviews {
        poll()?;
        let object = review.as_object().ok_or_else(|| {
            CliError::InvalidRepository("merge request review is not an object".to_string())
        })?;
        if object.len() != 5
            || review.get("reviewer").and_then(Value::as_str).is_none()
            || review.get("comment").and_then(Value::as_str).is_none()
            || review.get("reviewed_at").and_then(Value::as_str).is_none()
            || review
                .get("subject_digest")
                .and_then(Value::as_str)
                .is_none()
            || !matches!(
                review.get("decision").and_then(Value::as_str),
                Some("approve" | "reject")
            )
        {
            return Err(CliError::InvalidRepository(
                "merge request review has an invalid or future schema".to_string(),
            ));
        }
    }
    poll()?;
    let Some(effective) = reviews.last() else {
        return Ok(false);
    };
    let decision = effective.get("decision").and_then(Value::as_str);
    let expected_status = if decision == Some("approve") {
        "approved"
    } else {
        "rejected"
    };
    if mr.get("status").and_then(Value::as_str) != Some(expected_status) {
        return Err(CliError::InvalidRepository(
            "merge request status diverges from its effective review".to_string(),
        ));
    }
    Ok(decision == Some("approve")
        && effective.get("subject_digest").and_then(Value::as_str) == Some(subject_digest))
}

pub(crate) fn canonical_source_object_records(records: &[Value]) -> Result<Vec<Value>, CliError> {
    let mut canonical = records.to_vec();
    canonical.sort_by(|left, right| {
        left.get("object_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(
                right
                    .get("object_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            )
    });
    let mut seen = BTreeSet::new();
    for record in &canonical {
        let object_id = required_string(record, &["object_id"])?;
        if !seen.insert(object_id) {
            return Err(CliError::Usage(
                "remote object set contains a duplicate object ID".to_string(),
            ));
        }
    }
    Ok(canonical)
}

/// Legacy no-ledger compatibility digest. Work-aware callers must use
/// `source_object_set_digest_ordered_with_poll` and retain its owner.
pub(crate) fn source_object_set_digest(records: &[Value]) -> Result<String, CliError> {
    let canonical = canonical_source_object_records(records)?;
    Ok(format!(
        "sha256:{}",
        sha256_hex(&codefire_store::canonical_json(&Value::Array(canonical))?)
    ))
}

const SOURCE_OBJECT_SET_DIGEST_TEMPLATE: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const SOURCE_OBJECT_SET_DIGEST_LEN: usize = SOURCE_OBJECT_SET_DIGEST_TEMPLATE.len();

/// Inline, ledger-bound ownership for a generic source object-set digest.
///
/// The retained reservation is acquired before this value is constructed.
/// Callers must explicitly release it after borrowed use or transfer its
/// contents into the next owned representation while the reservation is live.
#[must_use = "accounted object-set digests must be transferred or released"]
#[derive(Debug)]
pub(crate) struct AccountedSourceObjectSetDigest {
    bytes: [u8; SOURCE_OBJECT_SET_DIGEST_LEN],
    retained_bytes: u64,
    ledger_identity: Option<codefire_core::DecisionWorkLedgerIdentity>,
}

impl AccountedSourceObjectSetDigest {
    pub(crate) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes)
            .expect("source object-set digest is constructed from ASCII literals")
    }

    pub(crate) fn ensure_ledger(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        match self.ledger_identity {
            Some(identity) if ledger.matches_identity(identity) => Ok(()),
            Some(_) => Err(CliError::InvalidRepository(
                "source object-set digest belongs to a different decision-work ledger".to_string(),
            )),
            None => Err(CliError::InvalidRepository(
                "source object-set digest ownership was already released".to_string(),
            )),
        }
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    pub(crate) fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )
            .map_err(decision_work_cli_error)?;
        self.retained_bytes = 0;
        self.ledger_identity = None;
        Ok(())
    }
}

impl Drop for AccountedSourceObjectSetDigest {
    fn drop(&mut self) {
        debug_assert!(
            self.retained_bytes == 0 && self.ledger_identity.is_none(),
            "accounted source object-set digest was dropped without release or transfer"
        );
    }
}

#[cfg(test)]
pub(crate) fn source_object_set_digest_ordered(
    records: &[Value],
    canonical_indices: &[usize],
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<AccountedSourceObjectSetDigest, CliError> {
    source_object_set_digest_ordered_with_poll(
        records,
        canonical_indices.iter().copied(),
        ledger,
        || Ok(()),
    )
}

pub(crate) fn source_object_set_digest_ordered_with_poll<I, F>(
    records: &[Value],
    canonical_indices: I,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<AccountedSourceObjectSetDigest, CliError>
where
    I: IntoIterator<Item = usize>,
    F: FnMut() -> Result<(), CliError>,
{
    let digest = source_object_set_digest_ordered_hash_with_poll(
        records,
        canonical_indices,
        ledger,
        &mut poll,
    )?;
    let retained_bytes =
        codefire_core::logical_retained_frame_bytes(SOURCE_OBJECT_SET_DIGEST_TEMPLATE)
            .map_err(decision_work_cli_error)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )
        .map_err(decision_work_cli_error)?;

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut bytes = [0_u8; SOURCE_OBJECT_SET_DIGEST_LEN];
    bytes[..7].copy_from_slice(b"sha256:");
    for (index, byte) in digest.into_iter().enumerate() {
        bytes[7 + index * 2] = HEX[usize::from(byte >> 4)];
        bytes[8 + index * 2] = HEX[usize::from(byte & 0x0f)];
    }
    Ok(AccountedSourceObjectSetDigest {
        bytes,
        retained_bytes,
        ledger_identity: Some(ledger.identity()),
    })
}

fn source_object_set_digest_ordered_hash_with_poll<I, F>(
    records: &[Value],
    canonical_indices: I,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<[u8; 32], CliError>
where
    I: IntoIterator<Item = usize>,
    F: FnMut() -> Result<(), CliError>,
{
    struct DigestWriter<'a, F> {
        digest: &'a mut Sha256,
        ledger: &'a mut codefire_core::DecisionWorkLedger,
        poll: &'a mut F,
        accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
        poll_error: Option<CliError>,
    }

    impl<F> std::io::Write for DigestWriter<'_, F>
    where
        F: FnMut() -> Result<(), CliError>,
    {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if let Err(error) = (self.poll)() {
                self.poll_error = Some(error);
                return Err(std::io::Error::other("object-set digest cancelled"));
            }
            let amount = u64::try_from(bytes.len())
                .map_err(|_| std::io::Error::other("object-set digest byte count overflow"))?;
            if let Err(error) = self.ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .count(codefire_core::DecisionWorkCounter::BytesHashed, amount),
            ) {
                self.accounting_error = Some(error);
                return Err(std::io::Error::other(
                    "object-set digest work budget exceeded",
                ));
            }
            self.digest.update(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut digest = Sha256::new();
    let mut writer = DigestWriter {
        digest: &mut digest,
        ledger,
        poll: &mut poll,
        accounting_error: None,
        poll_error: None,
    };
    let hashed = (|| -> Result<(), CliError> {
        std::io::Write::write_all(&mut writer, b"[")?;
        for (position, index) in canonical_indices.into_iter().enumerate() {
            let record = records.get(index).ok_or_else(|| {
                CliError::InvalidRepository("canonical object index is out of bounds".to_string())
            })?;
            if position > 0 {
                std::io::Write::write_all(&mut writer, b",")?;
            }
            serde_json::to_writer(&mut writer, record)?;
        }
        std::io::Write::write_all(&mut writer, b"]")?;
        Ok(())
    })();
    if let Some(error) = writer.accounting_error.take() {
        return Err(decision_work_cli_error(error));
    }
    if let Some(error) = writer.poll_error.take() {
        return Err(error);
    }
    hashed?;
    drop(writer);
    Ok(digest.finalize().into())
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
        let source_head = required_string(&response, &["source_head"])?;
        let target_head = required_string(&response, &["target_head"])?;
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
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &target.org,
            &target.app,
            "request_merge",
            None,
            None,
        );
        let payload = json!({
            "source_url": options.source_url,
            "target_url": options.target_url,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        let request_signature = super::signatures::sign_remote_request(
            "request_merge",
            actor,
            &request_target,
            &payload,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &target.project_root,
            "request_merge",
            actor,
            &request_target,
            &payload,
            request_signature.as_ref(),
        )?;
        ensure_remote_layout(&target.project_root)?;
    }
    let project_gates = if options.dry_run {
        None
    } else {
        Some(RemoteProjectGates::acquire_for_merge(
            &source.project_root,
            &target.project_root,
            &options.lock,
        )?)
    };
    let source_branch = load_remote_branch(&source)?;
    let target_branch = load_remote_branch(&target)?;
    let source_head = required_string(&source_branch, &["head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&source.project_root).objects,
        &source_head,
    )?;
    let source_records =
        collect_object_records(&remote_dirs(&source.project_root).objects, &source_head)?;
    let object_set_digest = source_object_set_digest(&source_records)?;
    codefire_store::validate_sealed_commit(
        &remote_dirs(&target.project_root).objects,
        &target_head,
    )?;
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
    let subject = merge_request_subject_for_roots(&MergeRequestSubjectInput {
        mr_id: &mr_id,
        source_project_root: &source.project_root,
        target_project_root: &target.project_root,
        source_branch: &source.branch,
        target_branch: &target.branch,
        source_head: &source_head,
        target_head: &target_head,
        source_object_set_digest: &object_set_digest,
    })?;
    mr_payload["subject_digest"] = Value::String(merge_request_subject_digest(&subject)?);
    mr_payload["subject"] = subject;
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
    let result = RequestMergeResult {
        id: mr_id.clone(),
        source_head: required_string(&mr_payload, &["source_head"])?,
        target_head: required_string(&mr_payload, &["target_head_at_request"])?,
        plan: plan.clone(),
    };
    let immutable_result =
        request_merge_result(&result.id, &result.source_head, &result.target_head);
    let source_project_id = project::remote_project_stable_id(&source.project_root)?;
    let target_project_id = project::remote_project_stable_id(&target.project_root)?;
    let publication_request = request_merge_publication(
        "local",
        options.idempotency_key.clone(),
        &MergePublicationIdentity {
            source_project_id: &source_project_id,
            target_project_id: &target_project_id,
            source_branch: &source.branch,
            target_branch: &target.branch,
            source_head: &source_head,
            target_head_at_request: &target_head,
            source_object_set_digest: &object_set_digest,
        },
    );
    let gate = project_gates
        .as_ref()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote merge project gates are missing".to_string())
        })?
        .exclusive_for(&target.project_root)?;
    copy_object_graph(
        &remote_dirs(&source.project_root).objects,
        &remote_dirs(&target.project_root).objects,
        &source_head,
    )?;
    if let Some(published) =
        replay_json_publication(gate, &target.project_root, &publication_request)?
    {
        return Ok(RequestMergeResult {
            id: required_string(&published, &["id"])?,
            source_head: required_string(&published, &["source_head"])?,
            target_head: required_string(&published, &["target_head"])?,
            plan,
        });
    }
    let published = execute_json_publication(
        gate,
        &target.project_root,
        &publication_request,
        &[AuthorityWrite {
            authority: project::RemoteAuthority::MergeRequest(mr_id),
            before: None,
            after: mr_payload,
        }],
        &immutable_result,
    )?;
    Ok(RequestMergeResult {
        id: required_string(&published, &["id"])?,
        source_head: required_string(&published, &["source_head"])?,
        target_head: required_string(&published, &["target_head"])?,
        plan,
    })
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
    let _project_gate =
        RemoteProjectGate::acquire_shared(&project.project_root, &LockOptions::default())?;
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
        validate_record_version(&mr, "remote_merge_request")?;
        let source = parse_cf_url(&required_string(&mr, &["source_url"])?)?;
        let target = parse_cf_url(&required_string(&mr, &["target_url"])?)?;
        if target.project_root != project.project_root {
            return Err(CliError::InvalidRepository(
                "merge request target does not belong to the listed project".to_string(),
            ));
        }
        reject_legacy_cross_project_merge_request_read(
            &mr,
            &source.project_root,
            &target.project_root,
        )?;
        validate_merge_request_record(&mr)?;
        let current_target = read_optional_remote_branch(&target.project_root, &target.branch)?;
        let persisted_status = mr.get("status").and_then(Value::as_str).unwrap_or("open");
        let mut status = persisted_status.to_string();
        if let Some(current_target) = current_target {
            let current_head = required_string(&current_target, &["head"])?;
            codefire_store::validate_sealed_commit(
                &remote_dirs(&target.project_root).objects,
                &current_head,
            )?;
            if !persisted_merge_request_status_is_terminal(persisted_status)
                && current_head != required_string(&mr, &["target_head_at_request"])?
            {
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

pub(crate) fn reject_legacy_cross_project_merge_request_read(
    mr: &Value,
    source_project_root: &Path,
    target_project_root: &Path,
) -> Result<(), CliError> {
    if mr.get("subject").is_none() && source_project_root != target_project_root {
        return Err(CliError::InvalidRepository(
            "legacy cross-project merge request cannot be read without holding both project gates; run request-review to materialize its target-local subject"
                .to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn review_merge_request(
    options: &RequestReviewOptions,
) -> Result<RequestReviewResult, CliError> {
    validate_remote_component(&options.mr_id, "merge-request ID")?;
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
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "review",
            None,
            Some(&options.mr_id),
        );
        let payload = json!({
            "reviewer": options.reviewer,
            "decision": options.decision,
            "comment": options.comment,
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        let request_signature = super::signatures::sign_remote_request(
            "review",
            actor,
            &request_target,
            &payload,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &project.project_root,
            "review",
            actor,
            &request_target,
            &payload,
            request_signature.as_ref(),
        )?;
        ensure_remote_layout(&project.project_root)?;
    }
    let mr_path = remote_merge_request_path(&project.project_root, &options.mr_id);
    let mut mr = read_optional_json(&mr_path)?
        .ok_or_else(|| CliError::Usage(format!("merge request not found: {}", options.mr_id)))?;
    validate_record_version(&mr, "remote_merge_request")?;
    if options.dry_run {
        let initial_source_url = required_string(&mr, &["source_url"])?;
        let initial_target_url = required_string(&mr, &["target_url"])?;
        let source = parse_cf_url(&initial_source_url)?;
        let target = parse_cf_url(&initial_target_url)?;
        if target.project_root != project.project_root {
            return Err(CliError::InvalidRepository(
                "merge request target does not belong to the reviewed project".to_string(),
            ));
        }
        reject_legacy_cross_project_merge_request_read(
            &mr,
            &source.project_root,
            &target.project_root,
        )?;
        let _project_gate = RemoteProjectGate::acquire_shared(&target.project_root, &options.lock)?;
        mr = read_optional_json(&mr_path)?.ok_or_else(|| {
            CliError::Usage(format!("merge request not found: {}", options.mr_id))
        })?;
        if mr.get("source_url").and_then(Value::as_str) != Some(initial_source_url.as_str())
            || mr.get("target_url").and_then(Value::as_str) != Some(initial_target_url.as_str())
        {
            return Err(CliError::InvalidRepository(
                "merge request endpoints changed while acquiring project gate".to_string(),
            ));
        }
        validate_merge_request_record(&mr)?;
        reject_stale_merge_request_read_only(&mr)?;
        let plan =
            request_review_operation_plan(options, "file", &required_string(&mr, &["status"])?);
        return Ok(RequestReviewResult {
            reviewer: options.reviewer.clone(),
            decision: options.decision.clone(),
            plan,
        });
    }
    let initial_source_url = required_string(&mr, &["source_url"])?;
    let initial_target_url = required_string(&mr, &["target_url"])?;
    let source = parse_cf_url(&initial_source_url)?;
    let target = parse_cf_url(&initial_target_url)?;
    if target.project_root != project.project_root {
        return Err(CliError::InvalidRepository(
            "merge request target does not belong to the reviewed project".to_string(),
        ));
    }
    let single_gate = if mr.get("subject").is_some() {
        Some(RemoteProjectGate::acquire_exclusive(
            &target.project_root,
            &options.lock,
        )?)
    } else {
        None
    };
    let project_gates = if single_gate.is_none() {
        Some(RemoteProjectGates::acquire_for_merge(
            &source.project_root,
            &target.project_root,
            &options.lock,
        )?)
    } else {
        None
    };
    let gate = if let Some(gate) = single_gate.as_ref() {
        gate
    } else {
        project_gates
            .as_ref()
            .ok_or_else(|| {
                CliError::InvalidRepository("remote review project gates are missing".to_string())
            })?
            .exclusive_for(&target.project_root)?
    };
    mr = read_optional_json(&mr_path)?
        .ok_or_else(|| CliError::Usage(format!("merge request not found: {}", options.mr_id)))?;
    if mr.get("source_url").and_then(Value::as_str) != Some(initial_source_url.as_str())
        || mr.get("target_url").and_then(Value::as_str) != Some(initial_target_url.as_str())
    {
        return Err(CliError::InvalidRepository(
            "merge request endpoints changed while acquiring project gates".to_string(),
        ));
    }
    validate_merge_request_record(&mr)?;
    let subject_is_materialized = mr.get("subject").is_some();
    if !subject_is_materialized {
        reject_stale_merge_request(&mr)?;
    }
    let authority_before = mr.clone();
    let subject_digest = materialize_merge_request_subject(&mut mr, &source, &target)?;
    let plan = request_review_operation_plan(options, "file", &required_string(&mr, &["status"])?);
    let immutable_result = review_result(&options.mr_id, &options.reviewer, &options.decision);
    let stable_project_id = project::remote_project_stable_id(&project.project_root)?;
    let source_head = required_string(&mr, &["source_head"])?;
    let publication_request = review_publication(
        "local",
        options.idempotency_key.clone(),
        &ReviewPublicationIdentity {
            stable_project_id: &stable_project_id,
            merge_request_id: &options.mr_id,
            reviewer: &options.reviewer,
            decision: &options.decision,
            comment: &options.comment,
            subject_digest: &subject_digest,
            source_head: &source_head,
        },
    );
    if let Some(published) =
        replay_json_publication(gate, &project.project_root, &publication_request)?
    {
        return Ok(RequestReviewResult {
            reviewer: required_string(&published, &["reviewer"])?,
            decision: required_string(&published, &["decision"])?,
            plan,
        });
    }
    if subject_is_materialized {
        reject_stale_merge_request(&mr)?;
    }
    let review = json!({
        "reviewer": options.reviewer,
        "decision": options.decision,
        "comment": options.comment,
        "subject_digest": subject_digest,
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
    let published = execute_json_publication(
        gate,
        &project.project_root,
        &publication_request,
        &[review_write(&options.mr_id, authority_before, mr)],
        &immutable_result,
    )?;
    Ok(RequestReviewResult {
        reviewer: required_string(&published, &["reviewer"])?,
        decision: required_string(&published, &["decision"])?,
        plan,
    })
}

pub(crate) fn apply_merge_request(
    options: &RequestApplyOptions,
) -> Result<RequestApplyResult, CliError> {
    validate_remote_component(&options.mr_id, "merge-request ID")?;
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
        let actor = "local";
        let request_target = super::signatures::remote_request_target(
            &project.org,
            &project.app,
            "apply",
            None,
            Some(&options.mr_id),
        );
        let payload = json!({
            "actor": actor,
            "idempotency_key": &options.idempotency_key,
            "lock": lock_options_json(&options.lock),
        });
        let request_signature = super::signatures::sign_remote_request(
            "apply",
            actor,
            &request_target,
            &payload,
            options.request_key_id.as_deref(),
        )?;
        super::signatures::verify_remote_request_signature(
            &project.project_root,
            "apply",
            actor,
            &request_target,
            &payload,
            request_signature.as_ref(),
        )?;
        ensure_remote_layout(&project.project_root)?;
    }
    let _project_gate = if options.dry_run {
        None
    } else {
        Some(RemoteProjectGate::acquire_exclusive(
            &project.project_root,
            &options.lock,
        )?)
    };
    let mr_path = remote_merge_request_path(&project.project_root, &options.mr_id);
    let mut mr = read_optional_json(&mr_path)?
        .ok_or_else(|| CliError::Usage(format!("merge request not found: {}", options.mr_id)))?;
    validate_record_version(&mr, "remote_merge_request")?;
    validate_merge_request_record(&mr)?;
    let target = parse_cf_url(&required_string(&mr, &["target_url"])?)?;
    let target_branch = load_remote_branch(&target)?;
    let source_head = required_string(&mr, &["source_head"])?;
    let target_head = required_string(&target_branch, &["head"])?;
    let target_head_at_request = required_string(&mr, &["target_head_at_request"])?;
    let subject_digest = validate_merge_request_subject(&mr)?;
    let plan = request_apply_operation_plan(options, "file", &target.branch, &source_head);
    let immutable_result = apply_result(&options.mr_id, &target.branch, &source_head);
    let stable_project_id = project::remote_project_stable_id(&target.project_root)?;
    let publication_request = apply_publication(
        "local",
        options.idempotency_key.clone(),
        &ApplyPublicationIdentity {
            stable_project_id: &stable_project_id,
            merge_request_id: &options.mr_id,
            subject_digest: &subject_digest,
            target_branch: &target.branch,
            source_head: &source_head,
            target_head_at_request: &target_head_at_request,
        },
    );
    if let Some(gate) = _project_gate.as_ref() {
        if let Some(published) =
            replay_json_publication(gate, &project.project_root, &publication_request)?
        {
            let target_branch = required_string(&published, &["target_branch"])?;
            let head = required_string(&published, &["head"])?;
            return Ok(RequestApplyResult {
                plan: request_apply_replay_plan(options, "file", &target_branch, &head),
                target_branch,
                head,
            });
        }
    }
    if mr.get("status").and_then(Value::as_str) != Some("approved") {
        return Err(CliError::Usage(format!(
            "merge request must be approved before apply: {}",
            options.mr_id
        )));
    }
    let subject_approved = merge_request_has_effective_approval(&mr, &subject_digest)?;
    if !subject_approved {
        return Err(CliError::Usage(format!(
            "merge request approval is not bound to its current subject: {}",
            options.mr_id
        )));
    }
    if target_head != target_head_at_request {
        if options.dry_run {
            return Err(CliError::Usage(format!(
                "merge request is stale: {}",
                options.mr_id
            )));
        }
        return Err(CliError::Usage(format!(
            "merge request is stale: {}",
            options.mr_id
        )));
    }
    let target_objects = remote_dirs(&target.project_root).objects;
    let source_objects = &target_objects;
    codefire_store::validate_sealed_commit(source_objects, &source_head)?;
    codefire_store::validate_sealed_commit(&target_objects, &target_head)?;
    super::signatures::enforce_remote_commit_signature_policy(
        &target.project_root,
        source_objects,
        &source_head,
    )?;
    if !is_ancestor_in_objects(source_objects, &target_head, &source_head)? {
        return Err(CliError::Usage(
            "request apply rejected: source is not a fast-forward of target".to_string(),
        ));
    }
    if options.dry_run {
        return Ok(RequestApplyResult {
            target_branch: target.branch,
            head: source_head,
            plan,
        });
    }
    let current_target = load_remote_branch(&target)?;
    if required_string(&current_target, &["head"])? != target_head_at_request {
        return Err(CliError::Usage(format!(
            "merge request is stale: {}",
            options.mr_id
        )));
    }
    let generation_before = read_optional_json(&target.project_root.join("gc_state.json"))?;
    let generation = generation_before
        .as_ref()
        .and_then(|value| value.get("current_generation"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let branch_before = read_optional_remote_branch(&target.project_root, &target.branch)?;
    let branch_after = json!({
        "version": 1,
        "name": target.branch,
        "head": source_head,
        "generation": generation,
        "applied_from": required_string(&mr, &["source_url"])?,
        "applied_by": "local",
        "updated_at": now_iso_utc(),
    });
    let before_mr = mr.clone();
    mr["status"] = Value::String("applied".to_string());
    mr["applied_at"] = Value::String(now_iso_utc());
    mr["applied_by"] = Value::String("local".to_string());
    mr["applied_head"] = Value::String(source_head.clone());
    let published = execute_json_publication(
        _project_gate.as_ref().ok_or_else(|| {
            CliError::InvalidRepository("remote apply project gate is missing".to_string())
        })?,
        &target.project_root,
        &publication_request,
        &apply_writes(ApplyPostimages {
            target_branch: &target.branch,
            merge_request_id: &options.mr_id,
            generation_before,
            generation_after: json!({"current_generation": generation}),
            branch_before: branch_before.ok_or_else(|| {
                CliError::InvalidRepository("remote target branch disappeared".to_string())
            })?,
            branch_after,
            merge_request_before: before_mr,
            merge_request_after: mr,
        }),
        &immutable_result,
    )?;
    Ok(RequestApplyResult {
        target_branch: required_string(&published, &["target_branch"])?,
        head: required_string(&published, &["head"])?,
        plan,
    })
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
    ensure_remote_project_layout(project_root)
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
    let org = validated_remote_component(percent_decode(org_part)?, "organization")?;
    let app = validated_remote_component(percent_decode(app_part)?, "application")?;
    let branch = validated_remote_component(percent_decode(branch_part)?, "branch")?;
    Ok(CfUrl {
        project_root: server_root
            .join(".codefire-server")
            .join("projects")
            .join(&org)
            .join(&app),
        org,
        app,
        branch,
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
    let org = validated_remote_component(percent_decode(org_part)?, "organization")?;
    let app = validated_remote_component(percent_decode(app_part)?, "application")?;
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
        .join(remote_branch_file_name(branch))
}

pub(crate) fn remote_branch_file_name(branch: &str) -> String {
    remote_authority_file_name("branch", branch, true)
}

pub(crate) fn remote_merge_request_path(project_root: &Path, mr_id: &str) -> PathBuf {
    remote_dirs(project_root)
        .merge_requests
        .join(remote_merge_request_file_name(mr_id))
}

pub(crate) fn remote_merge_request_file_name(mr_id: &str) -> String {
    remote_authority_file_name("merge-request", mr_id, false)
}

pub(crate) fn remote_gc_audit_file_name(plan_id: &str) -> String {
    remote_authority_file_name("gc-audit", plan_id, false)
}

fn remote_authority_file_name(domain: &str, value: &str, percent_encode: bool) -> String {
    const DIRECT_STEM_BYTE_LIMIT: usize = 160;
    let direct = if percent_encode {
        ref_file_name(value)
    } else {
        value.to_string()
    };
    if direct.len() <= DIRECT_STEM_BYTE_LIMIT {
        return format!("{direct}.json");
    }
    format!(
        "{}{}.json",
        project::REMOTE_INTERNAL_HASH_PREFIX,
        sha256_hex(format!("codefire-remote-authority-v1\0{domain}\0{value}").as_bytes())
    )
}

pub(crate) fn load_remote_branch(remote: &CfUrl) -> Result<Value, CliError> {
    read_optional_remote_branch(&remote.project_root, &remote.branch)?
        .ok_or_else(|| CliError::Usage(format!("remote branch not found: {}", remote.branch)))
}

pub(crate) fn read_optional_remote_branch(
    project_root: &Path,
    branch: &str,
) -> Result<Option<Value>, CliError> {
    let record = read_optional_json(&remote_branch_path(project_root, branch))?;
    if let Some(record) = record.as_ref() {
        validate_remote_branch_record(record)?;
    }
    Ok(record)
}

pub(crate) fn validate_remote_branch_record(branch: &Value) -> Result<(), CliError> {
    validate_record_version(branch, "remote_branch")?;
    let name = required_string(branch, &["name"])?;
    validate_remote_component(&name, "branch")?;
    let head = required_string(branch, &["head"])?;
    if !head.starts_with("CF-COMMIT-") {
        return Err(CliError::InvalidRepository(
            "remote branch head is not a commit object ID".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn read_optional_json(path: &Path) -> Result<Option<Value>, CliError> {
    match read_json(path) {
        Ok(value) => Ok(Some(value)),
        Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn copy_object_graph(
    src_objects: &Path,
    dest_objects: &Path,
    root_id: &str,
) -> Result<(), CliError> {
    copy_object_graph_with_poll(src_objects, dest_objects, root_id, || Ok(()))
}

pub(crate) fn copy_object_graph_with_poll<F>(
    src_objects: &Path,
    dest_objects: &Path,
    root_id: &str,
    mut poll: F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let mut seen = BTreeSet::new();
    let mut stack = vec![root_id.to_string()];
    while let Some(object_id) = stack.pop() {
        poll()?;
        if !seen.insert(object_id.clone()) {
            continue;
        }
        for reference in copy_object_record(src_objects, dest_objects, &object_id)? {
            stack.push(reference);
        }
    }
    Ok(())
}

pub(crate) fn copy_object_record(
    src_objects: &Path,
    dest_objects: &Path,
    object_id: &str,
) -> Result<Vec<String>, CliError> {
    let path = codefire_store::object_record_path(src_objects, object_id)
        .ok_or_else(|| codefire_store::StoreError::ObjectNotFound(object_id.to_string()))?;
    let document = codefire_store::read_object_record_document_path(&path)?;
    codefire_store::validate_object_record(&document.record, Some(object_id), Some(&path))?;
    let references = object_document_references(&document);
    let expected_id = document.record.object_id.clone();
    let stored_id = codefire_store::store_object_record_document(dest_objects, document)?;
    if stored_id != expected_id || stored_id != object_id {
        return Err(CliError::InvalidRepository(format!(
            "copied object identity changed: expected {object_id}, stored {stored_id}"
        )));
    }
    Ok(references)
}

pub(crate) fn collect_object_records(
    objects: &Path,
    root_id: &str,
) -> Result<Vec<Value>, CliError> {
    collect_object_records_internal(objects, root_id, None, || Ok(()))
        .map(AccountedObjectRecords::into_unaccounted_records)
}

#[derive(Debug)]
pub(crate) struct AccountedObjectRecords {
    records: Option<Vec<Value>>,
    retained_bytes: u64,
    ledger_identity: Option<codefire_core::DecisionWorkLedgerIdentity>,
}

#[derive(Debug)]
pub(crate) struct AccountedObjectRecordPayloadOwnership {
    retained_bytes: u64,
    ledger_identity: Option<codefire_core::DecisionWorkLedgerIdentity>,
}

impl AccountedObjectRecordPayloadOwnership {
    pub(crate) fn release_after_payload_drop(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let identity = self.ledger_identity.ok_or_else(|| {
            CliError::InvalidRepository(
                "object-record HTTP payload ownership was already released".to_string(),
            )
        })?;
        if !ledger.matches_identity(identity) {
            return Err(CliError::InvalidRepository(
                "object-record HTTP payload belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )
            .map_err(decision_work_cli_error)?;
        self.retained_bytes = 0;
        self.ledger_identity = None;
        Ok(())
    }
}

impl Drop for AccountedObjectRecordPayloadOwnership {
    fn drop(&mut self) {
        debug_assert!(
            self.retained_bytes == 0 && self.ledger_identity.is_none(),
            "object-record HTTP payload ownership was dropped before payload release"
        );
    }
}

impl AccountedObjectRecords {
    pub(crate) fn as_slice(&self) -> &[Value] {
        self.records
            .as_deref()
            .expect("accounted object records must still own their values")
    }

    pub(crate) fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub(crate) fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        let retained_bytes = self.retained_bytes;
        drop(self.records.take());
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .map_err(decision_work_cli_error)?;
        self.retained_bytes = 0;
        self.ledger_identity = None;
        Ok(())
    }

    /// Transfers the retained values to the HTTP route payload. The route's
    /// command ledger remains the owner and releases the bytes only after the
    /// payload has been encoded and dropped.
    pub(crate) fn transfer_to_http_payload(
        &mut self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(Vec<Value>, AccountedObjectRecordPayloadOwnership), CliError> {
        self.ensure_ledger(ledger)?;
        let records = self
            .records
            .take()
            .expect("accounted object records must still own their values");
        let ownership = AccountedObjectRecordPayloadOwnership {
            retained_bytes: self.retained_bytes,
            ledger_identity: self.ledger_identity,
        };
        self.retained_bytes = 0;
        self.ledger_identity = None;
        Ok((records, ownership))
    }

    fn into_unaccounted_records(mut self) -> Vec<Value> {
        debug_assert!(self.ledger_identity.is_none());
        debug_assert_eq!(self.retained_bytes, 0);
        self.records
            .take()
            .expect("unaccounted object records must still own their values")
    }

    fn ensure_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        match self.ledger_identity {
            Some(identity) if ledger.matches_identity(identity) => Ok(()),
            Some(_) => Err(CliError::InvalidRepository(
                "accounted object records belong to a different decision-work ledger".to_string(),
            )),
            None => Err(CliError::InvalidRepository(
                "object records do not have decision-work ownership".to_string(),
            )),
        }
    }

    pub(crate) fn publish_with_work_and_poll<F>(
        &mut self,
        objects: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut poll: F,
    ) -> Result<(), CliError>
    where
        F: FnMut() -> Result<(), CliError>,
    {
        self.ensure_ledger(ledger)?;
        let records = self
            .records
            .take()
            .expect("accounted object records must still own their values");
        let mut temporary_bytes = 0_u64;
        let publication = (|| -> Result<(), CliError> {
            ensure_object_dirs(objects)?;
            for record_value in records {
                poll()?;
                let document = codefire_store::object_record_document_from_value(record_value)?;
                let logical_bytes = codefire_store::compact_object_record_document_size(&document)?;
                let ownership_bytes = logical_bytes
                    .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES)
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "object publication retained byte count overflow".to_string(),
                        )
                    })?;
                let rendered_bytes =
                    codefire_store::rendered_object_record_document_size(&document)?;
                let scratch_bytes = rendered_bytes
                    .checked_mul(2)
                    .and_then(|bytes| bytes.checked_add(logical_bytes))
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "object publication scratch byte count overflow".to_string(),
                        )
                    })?;
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Staging,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .charge(
                            codefire_core::DecisionWorkDimension::AggregateBytes,
                            rendered_bytes,
                        )
                        .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
                        .acquire(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            scratch_bytes,
                        )
                        .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
                    )
                    .map_err(decision_work_cli_error)?;
                temporary_bytes = scratch_bytes;
                let expected_id = document.record.object_id.clone();
                let stored_id = codefire_store::store_object_record_document(objects, document)?;
                if stored_id != expected_id {
                    return Err(CliError::InvalidRepository(format!(
                        "received object identity changed: expected {expected_id}, stored {stored_id}"
                    )));
                }
                drop(expected_id);
                drop(stored_id);
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        scratch_bytes,
                    )
                    .map_err(decision_work_cli_error)?;
                temporary_bytes = 0;
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        ownership_bytes,
                    )
                    .map_err(decision_work_cli_error)?;
                self.retained_bytes = self
                    .retained_bytes
                    .checked_sub(ownership_bytes)
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "object publication retained byte count underflow".to_string(),
                        )
                    })?;
            }
            Ok(())
        })();
        if let Err(error) = publication {
            let mut first_error = Some(error);
            if temporary_bytes > 0 {
                record_remote_cleanup_error(
                    &mut first_error,
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            temporary_bytes,
                        )
                        .map_err(decision_work_cli_error),
                );
            }
            let retained_bytes = self.retained_bytes;
            if retained_bytes > 0 {
                record_remote_cleanup_error(
                    &mut first_error,
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained_bytes,
                        )
                        .map_err(decision_work_cli_error),
                );
            }
            self.retained_bytes = 0;
            self.ledger_identity = None;
            return Err(first_error.expect("object publication error must be retained"));
        }
        debug_assert_eq!(self.retained_bytes, 0);
        self.ledger_identity = None;
        Ok(())
    }
}

impl Drop for AccountedObjectRecords {
    fn drop(&mut self) {
        debug_assert!(
            self.ledger_identity.is_none() && self.retained_bytes == 0,
            "accounted object records were dropped without release or transfer"
        );
    }
}

#[cfg(test)]
pub(crate) fn collect_object_records_with_work(
    objects: &Path,
    root_id: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<AccountedObjectRecords, CliError> {
    collect_object_records_with_owned_work_and_poll(objects, root_id, ledger, || Ok(()))
}

pub(crate) fn collect_object_records_with_owned_work_and_poll<F>(
    objects: &Path,
    root_id: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: F,
) -> Result<AccountedObjectRecords, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    collect_object_records_internal(objects, root_id, Some(ledger), poll)
}

fn collect_object_records_internal<F>(
    objects: &Path,
    root_id: &str,
    mut ledger: Option<&mut codefire_core::DecisionWorkLedger>,
    mut poll: F,
) -> Result<AccountedObjectRecords, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let ledger_identity = ledger.as_deref().map(|ledger| ledger.identity());
    let mut retained_current = 0_u64;
    let mut temporary_current = 0_u64;
    let mut accounted_reader = None;
    let collected = (|| -> Result<Vec<Value>, CliError> {
        // One logical frontier frame remains owned while its ID is the key of
        // this map. Its fixed base covers the map node and the eventual output
        // Vec slot while both containers coexist; the variable ID bytes cover
        // the key itself. The key is dropped before that frame is released.
        let mut records = BTreeMap::new();
        if let Some(ledger) = ledger.as_deref_mut() {
            let retained_bytes = codefire_core::logical_retained_frame_bytes(root_id)
                .map_err(decision_work_cli_error)?;
            let next_retained = retained_current
                .checked_add(retained_bytes)
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "object collection retained bytes overflow".to_string(),
                    )
                })?;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::GraphDiscovery,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .observe_maximum(codefire_core::DecisionWorkDimension::LogicalDepth, 0)
                    .observe_maximum(codefire_core::DecisionWorkDimension::MaximumFrontier, 1)
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )
                    .count(
                        codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
                        1,
                    ),
                )
                .map_err(decision_work_cli_error)?;
            retained_current = next_retained;
        }
        let mut stack = vec![(root_id.to_string(), 0_u64)];
        if let Some(ledger) = ledger.as_deref_mut() {
            let mut cancellation_error = None;
            accounted_reader = Some(
                match codefire_store::AccountedObjectStoreReader::begin_with_work_and_cancellation(
                    objects,
                    ledger,
                    || match poll() {
                        Ok(()) => false,
                        Err(error) => {
                            cancellation_error = Some(error);
                            true
                        }
                    },
                ) {
                    Ok(reader) => reader,
                    Err(codefire_store::StoreError::ValidationCancelled)
                        if cancellation_error.is_some() =>
                    {
                        return Err(cancellation_error
                            .expect("cancelled object-store bind retains its poll error"));
                    }
                    Err(error) => return Err(CliError::from(error)),
                },
            );
        }
        while let Some((object_id, depth)) = stack.pop() {
            poll()?;
            if records.contains_key(&object_id) {
                if let Some(ledger) = ledger.as_deref_mut() {
                    ledger
                        .apply(
                            codefire_core::DecisionWorkReservation::new(
                                codefire_core::DecisionWorkPhase::GraphDiscovery,
                                codefire_core::DecisionWorkSurface::Atomic,
                            )
                            .count(codefire_core::DecisionWorkCounter::Revisits, 1),
                        )
                        .map_err(decision_work_cli_error)?;
                    let retained_bytes = codefire_core::logical_retained_frame_bytes(&object_id)
                        .map_err(decision_work_cli_error)?;
                    let next_retained =
                        retained_current
                            .checked_sub(retained_bytes)
                            .ok_or_else(|| {
                                CliError::InvalidRepository(
                                    "object collection retained bytes underflow".to_string(),
                                )
                            })?;
                    drop(object_id);
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained_bytes,
                        )
                        .map_err(decision_work_cli_error)?;
                    retained_current = next_retained;
                }
                continue;
            }
            if let Some(ledger) = ledger.as_deref_mut() {
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::GraphDiscovery,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
                        .count(codefire_core::DecisionWorkCounter::UniqueNodesDiscovered, 1),
                    )
                    .map_err(decision_work_cli_error)?;
            }
            let (document, document_retained_bytes) = if let Some(ledger) = ledger.as_deref_mut() {
                let mut cancellation_error = None;
                let mut accounted = match accounted_reader
                    .as_mut()
                    .expect("accounted object collection must retain its pinned reader")
                    .read_with_work_and_cancellation(&object_id, ledger, || match poll() {
                        Ok(()) => false,
                        Err(error) => {
                            cancellation_error = Some(error);
                            true
                        }
                    }) {
                    Ok(accounted) => accounted,
                    Err(codefire_store::StoreError::ValidationCancelled)
                        if cancellation_error.is_some() =>
                    {
                        return Err(cancellation_error
                            .expect("cancelled object read retains its poll error"));
                    }
                    Err(error) => return Err(CliError::from(error)),
                };
                let retained_bytes = accounted.retained_bytes();
                let next_retained = match retained_current.checked_add(retained_bytes) {
                    Some(next_retained) => next_retained,
                    None => {
                        accounted.release(ledger)?;
                        return Err(CliError::InvalidRepository(
                            "collected object retained byte count overflow".to_string(),
                        ));
                    }
                };
                let (document, transferred_bytes) = accounted.transfer_to_owner(ledger)?;
                debug_assert_eq!(retained_bytes, transferred_bytes);
                retained_current = next_retained;
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::RecordRead,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
                    )
                    .map_err(decision_work_cli_error)?;
                (document, retained_bytes)
            } else {
                let path = codefire_store::object_record_path(objects, &object_id)
                    .ok_or_else(|| codefire_store::StoreError::ObjectNotFound(object_id.clone()))?;
                let document = codefire_store::read_object_record_document_path(&path)?;
                codefire_store::validate_object_record(
                    &document.record,
                    Some(&object_id),
                    Some(&path),
                )?;
                (document, 0)
            };
            try_for_each_object_document_reference(&document, |reference| {
                let next_depth = depth.checked_add(1).ok_or_else(|| {
                    CliError::InvalidRepository("object graph depth overflow".to_string())
                })?;
                if let Some(ledger) = ledger.as_deref_mut() {
                    let next_frontier = u64::try_from(stack.len())
                        .ok()
                        .and_then(|size| size.checked_add(1))
                        .ok_or_else(|| {
                            CliError::InvalidRepository(
                                "object graph frontier overflow".to_string(),
                            )
                        })?;
                    let retained_bytes = codefire_core::logical_retained_frame_bytes(reference)
                        .map_err(decision_work_cli_error)?;
                    let next_retained =
                        retained_current
                            .checked_add(retained_bytes)
                            .ok_or_else(|| {
                                CliError::InvalidRepository(
                                    "object collection retained bytes overflow".to_string(),
                                )
                            })?;
                    ledger
                        .apply(
                            codefire_core::DecisionWorkReservation::new(
                                codefire_core::DecisionWorkPhase::GraphDiscovery,
                                codefire_core::DecisionWorkSurface::Atomic,
                            )
                            .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
                            .observe_maximum(
                                codefire_core::DecisionWorkDimension::LogicalDepth,
                                next_depth,
                            )
                            .observe_maximum(
                                codefire_core::DecisionWorkDimension::MaximumFrontier,
                                next_frontier,
                            )
                            .acquire(
                                codefire_core::DecisionWorkDimension::RetainedBytes,
                                retained_bytes,
                            )
                            .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1)
                            .count(
                                codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
                                1,
                            ),
                        )
                        .map_err(decision_work_cli_error)?;
                    retained_current = next_retained;
                }
                stack.push((reference.to_string(), next_depth));
                Ok::<(), CliError>(())
            })?;
            if let Some(ledger) = ledger.as_deref_mut() {
                let next_temporary = temporary_current
                    .checked_add(document_retained_bytes)
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "object collection temporary bytes overflow".to_string(),
                        )
                    })?;
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::RecordRead,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            document_retained_bytes,
                        ),
                    )
                    .map_err(decision_work_cli_error)?;
                temporary_current = next_temporary;
            }
            let record_value = codefire_store::object_record_document_value(&document)?;
            drop(document);
            if let Some(ledger) = ledger.as_deref_mut() {
                let next_temporary = temporary_current
                    .checked_sub(document_retained_bytes)
                    .ok_or_else(|| {
                        CliError::InvalidRepository(
                            "object collection temporary bytes underflow".to_string(),
                        )
                    })?;
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        document_retained_bytes,
                    )
                    .map_err(decision_work_cli_error)?;
                temporary_current = next_temporary;
            }
            records.insert(object_id, record_value);
        }
        // Allocate the output slots while every map entry still owns its
        // logical frame, then retire each key/frame only after the key drops.
        let mut ordered = Vec::with_capacity(records.len());
        for (object_id, record) in records {
            poll()?;
            if let Some(ledger) = ledger.as_deref_mut() {
                let retained_bytes = codefire_core::logical_retained_frame_bytes(&object_id)
                    .map_err(decision_work_cli_error)?;
                let next_retained =
                    retained_current
                        .checked_sub(retained_bytes)
                        .ok_or_else(|| {
                            CliError::InvalidRepository(
                                "object collection retained bytes underflow".to_string(),
                            )
                        })?;
                drop(object_id);
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )
                    .map_err(decision_work_cli_error)?;
                retained_current = next_retained;
            }
            ordered.push(record);
        }
        if let Some(ledger) = ledger.as_deref_mut() {
            let mut cancellation_error = None;
            match accounted_reader
                .as_mut()
                .expect("accounted object collection must close its pinned reader")
                .finish_with_cancellation(ledger, || match poll() {
                    Ok(()) => false,
                    Err(error) => {
                        cancellation_error = Some(error);
                        true
                    }
                }) {
                Ok(()) => {}
                Err(codefire_store::StoreError::ValidationCancelled)
                    if cancellation_error.is_some() =>
                {
                    return Err(cancellation_error
                        .expect("cancelled object-store closing fence retains its poll error"));
                }
                Err(error) => return Err(CliError::from(error)),
            }
        }
        Ok(ordered)
    })();

    match collected {
        Ok(records) => {
            debug_assert_eq!(temporary_current, 0);
            Ok(AccountedObjectRecords {
                records: Some(records),
                retained_bytes: retained_current,
                ledger_identity,
            })
        }
        Err(error) => {
            let mut first_error = Some(error);
            if let Some(ledger) = ledger {
                if let Some(reader) = accounted_reader.as_mut() {
                    record_remote_cleanup_error(
                        &mut first_error,
                        reader.cancel(ledger).map_err(CliError::from),
                    );
                }
                if temporary_current > 0 {
                    record_remote_cleanup_error(
                        &mut first_error,
                        ledger
                            .release(
                                codefire_core::DecisionWorkDimension::TemporaryBytes,
                                temporary_current,
                            )
                            .map_err(decision_work_cli_error),
                    );
                }
                if retained_current > 0 {
                    record_remote_cleanup_error(
                        &mut first_error,
                        ledger
                            .release(
                                codefire_core::DecisionWorkDimension::RetainedBytes,
                                retained_current,
                            )
                            .map_err(decision_work_cli_error),
                    );
                }
            }
            Err(first_error.expect("object collection error must be retained"))
        }
    }
}

fn record_remote_cleanup_error(first_error: &mut Option<CliError>, result: Result<(), CliError>) {
    if first_error.is_none() {
        if let Err(error) = result {
            *first_error = Some(error);
        }
    }
}

fn decision_work_cli_error(error: codefire_core::DecisionWorkAccountingError) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while collecting object graph: {error}"
        )),
    }
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
        let document = codefire_store::object_record_document_from_value(record_value)?;
        let expected_id = document.record.object_id.clone();
        let stored_id = codefire_store::store_object_record_document(objects, document)?;
        if stored_id != expected_id {
            return Err(CliError::InvalidRepository(format!(
                "received object identity changed: expected {}, stored {stored_id}",
                expected_id
            )));
        }
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

fn try_for_each_payload_reference<E>(
    payload: &Value,
    visit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), E> {
    match payload.get("type").and_then(Value::as_str) {
        Some("commit") => {
            if let Some(parents) = payload.get("parents").and_then(Value::as_array) {
                for reference in parents
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|value| value.starts_with("CF-"))
                {
                    visit(reference)?;
                }
            }
            if let Some(roots) = payload.get("roots").and_then(Value::as_object) {
                for reference in roots
                    .values()
                    .filter_map(Value::as_str)
                    .filter(|value| value.starts_with("CF-"))
                {
                    visit(reference)?;
                }
            }
        }
        Some("content_manifest") => {
            for reference in payload
                .get("entries")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.get("blob").and_then(Value::as_str))
            {
                visit(reference)?;
            }
        }
        Some("resolution_ledger") => {
            for resolution in payload
                .get("resolutions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                for reference in resolution
                    .get("evidence_refs")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter(|value| value.starts_with("CF-"))
                {
                    visit(reference)?;
                }
            }
        }
        Some("evidence") => {
            if let Some(reference) = payload
                .get("artifact_ref")
                .and_then(Value::as_str)
                .filter(|value| value.starts_with("CF-"))
            {
                visit(reference)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn try_for_each_object_document_reference<E>(
    document: &codefire_store::ObjectRecordDocument,
    mut visit: impl FnMut(&str) -> Result<(), E>,
) -> Result<(), E> {
    try_for_each_payload_reference(&document.record.payload, &mut visit)?;
    if let Some(descriptors) = document
        .extensions
        .get("extensions")
        .and_then(Value::as_array)
    {
        for descriptor in descriptors {
            for reference in descriptor
                .get("references")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|reference| reference.starts_with("CF-"))
            {
                visit(reference)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn object_document_references(
    document: &codefire_store::ObjectRecordDocument,
) -> Vec<String> {
    let mut references = Vec::new();
    try_for_each_object_document_reference(document, |reference| {
        references.push(reference.to_string());
        Ok::<(), std::convert::Infallible>(())
    })
    .expect("infallible object reference collection");
    references
}

pub(crate) fn try_for_each_object_value_reference<E>(
    record: &Value,
    mut visit: impl FnMut(&str) -> Result<(), E>,
) -> Result<(), E> {
    if let Some(payload) = record.get("payload") {
        try_for_each_payload_reference(payload, &mut visit)?;
    }
    if let Some(descriptors) = record.get("extensions").and_then(Value::as_array) {
        for descriptor in descriptors {
            for reference in descriptor
                .get("references")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|reference| reference.starts_with("CF-"))
            {
                visit(reference)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_object_record_value(
    record: &Value,
    expected_id: &str,
) -> Result<(), CliError> {
    let document = codefire_store::object_record_document_from_value(record.clone())?;
    codefire_store::validate_object_record(&document.record, Some(expected_id), None)?;
    Ok(())
}

pub(crate) fn validate_merge_request_record(mr: &Value) -> Result<(), CliError> {
    validate_record_version(mr, "remote_merge_request")?;
    validate_remote_component(&required_string(mr, &["id"])?, "merge-request ID")?;
    let source = parse_cf_url(&required_string(mr, &["source_url"])?)?;
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    let source_head = required_string(mr, &["source_head"])?;
    let target_head = required_string(mr, &["target_head_at_request"])?;
    let source_objects = if mr.get("subject").is_some() {
        remote_dirs(&target.project_root).objects
    } else {
        remote_dirs(&source.project_root).objects
    };
    codefire_store::validate_sealed_commit(&source_objects, &source_head)?;
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
    if mr.get("subject").is_some() {
        validate_merge_request_subject_for_roots(
            mr,
            &source.project_root,
            &target.project_root,
            &source.branch,
            &target.branch,
        )?;
    }
    Ok(())
}

fn persisted_merge_request_status_is_terminal(status: &str) -> bool {
    matches!(status, "applied" | "rejected")
}

fn request_apply_replay_plan(
    options: &RequestApplyOptions,
    transport: &str,
    target_branch: &str,
    apply_head: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "request-apply",
        "dry_run": false,
        "would_apply": false,
        "replayed": true,
        "project_url": &options.project_url,
        "merge_request_id": &options.mr_id,
        "target_branch": target_branch,
        "head": apply_head,
        "transport": transport,
        "operations": [],
        "next_actions": [],
    })
}

fn reject_stale_merge_request(mr: &Value) -> Result<(), CliError> {
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    if let Some(current_target) = read_optional_remote_branch(&target.project_root, &target.branch)?
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

fn reject_stale_merge_request_read_only(mr: &Value) -> Result<(), CliError> {
    let target = parse_cf_url(&required_string(mr, &["target_url"])?)?;
    if let Some(current_target) = read_optional_remote_branch(&target.project_root, &target.branch)?
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

pub(crate) fn is_ancestor_in_provisional_subject_with_poll<F>(
    subject: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    ancestor: &str,
    commit_id: &str,
    mut poll: F,
) -> Result<bool, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    fn acquire(
        ledger: &mut codefire_core::DecisionWorkLedger,
        owned: &mut u64,
        reservation: codefire_core::DecisionWorkReservation,
        bytes: u64,
    ) -> Result<(), CliError> {
        let next = owned.checked_add(bytes).ok_or_else(|| {
            CliError::InvalidRepository("commit ancestry retained-byte overflow".to_string())
        })?;
        ledger
            .apply(reservation.acquire(codefire_core::DecisionWorkDimension::RetainedBytes, bytes))
            .map_err(decision_work_cli_error)?;
        *owned = next;
        Ok(())
    }

    fn release(
        ledger: &mut codefire_core::DecisionWorkLedger,
        owned: &mut u64,
        bytes: u64,
    ) -> Result<(), CliError> {
        let next = owned.checked_sub(bytes).ok_or_else(|| {
            CliError::InvalidRepository("commit ancestry retained-byte underflow".to_string())
        })?;
        ledger
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, bytes)
            .map_err(decision_work_cli_error)?;
        *owned = next;
        Ok(())
    }

    subject.ensure_ledger(ledger).map_err(CliError::from)?;
    let mut retained_owned = 0_u64;
    let result = (|| {
        let root_bytes = codefire_core::logical_retained_frame_bytes(commit_id)
            .map_err(decision_work_cli_error)?;
        acquire(
            ledger,
            &mut retained_owned,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Query,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .observe_maximum(codefire_core::DecisionWorkDimension::LogicalDepth, 0)
            .observe_maximum(codefire_core::DecisionWorkDimension::MaximumFrontier, 1)
            .count(
                codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
                1,
            ),
            root_bytes,
        )?;
        let mut pending = vec![(commit_id.to_string(), 0_u64)];
        let mut seen = BTreeSet::new();
        while let Some((commit_id, depth)) = pending.pop() {
            poll()?;
            let frame_bytes = codefire_core::logical_retained_frame_bytes(&commit_id)
                .map_err(decision_work_cli_error)?;
            if seen.contains(&commit_id) {
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Query,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .count(codefire_core::DecisionWorkCounter::Revisits, 1),
                    )
                    .map_err(decision_work_cli_error)?;
                drop(commit_id);
                release(ledger, &mut retained_owned, frame_bytes)?;
                continue;
            }
            acquire(
                ledger,
                &mut retained_owned,
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Query,
                    codefire_core::DecisionWorkSurface::Atomic,
                ),
                frame_bytes,
            )?;
            seen.insert(commit_id.clone());
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Query,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
                    .count(codefire_core::DecisionWorkCounter::UniqueNodesDiscovered, 1)
                    .count(codefire_core::DecisionWorkCounter::MemoHits, 1),
                )
                .map_err(decision_work_cli_error)?;
            let parents = subject
                .retained_commit_payload(&commit_id)
                .map_err(CliError::from)?
                .get("parents")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    CliError::InvalidRepository(format!(
                        "validated commit parents are missing: {commit_id}"
                    ))
                })?;
            if commit_id == ancestor {
                let pending_bytes =
                    pending
                        .iter()
                        .try_fold(frame_bytes, |total, (pending_id, _)| {
                            let bytes = codefire_core::logical_retained_frame_bytes(pending_id)
                                .map_err(decision_work_cli_error)?;
                            total.checked_add(bytes).ok_or_else(|| {
                                CliError::InvalidRepository(
                                    "commit ancestry retained-byte cleanup overflow".to_string(),
                                )
                            })
                        })?;
                drop(commit_id);
                drop(pending);
                release(ledger, &mut retained_owned, pending_bytes)?;
                return Ok(true);
            }
            let next_depth = depth.checked_add(1).ok_or_else(|| {
                CliError::InvalidRepository("commit ancestry depth overflow".to_string())
            })?;
            for parent in parents.iter().rev() {
                let Some(parent_id) = parent
                    .as_str()
                    .filter(|parent_id| parent_id.starts_with("CF-COMMIT-"))
                else {
                    continue;
                };
                let retained_bytes = codefire_core::logical_retained_frame_bytes(parent_id)
                    .map_err(decision_work_cli_error)?;
                let next_frontier = u64::try_from(pending.len())
                    .ok()
                    .and_then(|frontier| frontier.checked_add(1))
                    .ok_or_else(|| {
                        CliError::InvalidRepository("commit ancestry frontier overflow".to_string())
                    })?;
                acquire(
                    ledger,
                    &mut retained_owned,
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Query,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
                    .observe_maximum(
                        codefire_core::DecisionWorkDimension::LogicalDepth,
                        next_depth,
                    )
                    .observe_maximum(
                        codefire_core::DecisionWorkDimension::MaximumFrontier,
                        next_frontier,
                    )
                    .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1)
                    .count(
                        codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
                        1,
                    ),
                    retained_bytes,
                )?;
                pending.push((parent_id.to_string(), next_depth));
            }
            drop(commit_id);
            release(ledger, &mut retained_owned, frame_bytes)?;
        }
        Ok(false)
    })();
    let cleanup = if retained_owned == 0 {
        Ok(())
    } else {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_owned,
            )
            .map_err(decision_work_cli_error)
    };
    match (result, cleanup) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    codefire_util::sha256_hex(bytes)
}

#[cfg(test)]
mod object_set_digest_owner_tests {
    use super::*;
    use serde_json::json;

    const RETAINED: codefire_core::DecisionWorkDimension =
        codefire_core::DecisionWorkDimension::RetainedBytes;

    fn ledger(label: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            label,
        )
        .expect("object-transfer test ledger")
    }

    fn records() -> Vec<Value> {
        vec![
            json!({"object_id": "CF-B", "payload": {"value": 2}}),
            json!({"object_id": "CF-A", "payload": {"value": 1}}),
        ]
    }

    fn retained_current(ledger: &codefire_core::DecisionWorkLedger) -> u64 {
        ledger
            .snapshot()
            .current
            .get(&RETAINED)
            .copied()
            .unwrap_or_default()
    }

    fn precharge_retained(ledger: &mut codefire_core::DecisionWorkLedger, remaining: u64) -> u64 {
        let limit = ledger
            .profile()
            .limit(RETAINED)
            .expect("retained limit")
            .limit;
        let precharged = limit
            .checked_sub(remaining)
            .expect("valid remaining budget");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(RETAINED, precharged),
            )
            .expect("retained precharge");
        precharged
    }

    fn release_precharge(ledger: &mut codefire_core::DecisionWorkLedger, precharged: u64) {
        ledger
            .release(RETAINED, precharged)
            .expect("retained precharge release");
        assert_eq!(retained_current(ledger), 0);
    }

    #[test]
    fn object_set_digest_owner_preserves_legacy_identity() {
        let records = records();
        let expected = source_object_set_digest(&records).expect("legacy digest");
        let mut ledger = ledger("object-set digest identity");
        let mut digest =
            source_object_set_digest_ordered_with_poll(&records, [1, 0], &mut ledger, || Ok(()))
                .expect("accounted digest");

        assert_eq!(digest.as_str(), expected);
        digest.release(&mut ledger).expect("digest release");
        assert_eq!(retained_current(&ledger), 0);
        assert_eq!(ledger.open_permit_count(), 0);
    }

    #[test]
    fn object_set_digest_owner_enforces_limit_minus_one_exact_and_plus_one() {
        let records = records();
        let owner_bytes =
            codefire_core::logical_retained_frame_bytes(SOURCE_OBJECT_SET_DIGEST_TEMPLATE)
                .expect("digest retained bytes");

        let mut below = ledger("object-set digest limit minus one");
        let below_precharge = precharge_retained(&mut below, owner_bytes - 1);
        assert!(source_object_set_digest_ordered_with_poll(
            &records,
            [1, 0],
            &mut below,
            || Ok(())
        )
        .is_err());
        release_precharge(&mut below, below_precharge);

        for (label, remaining) in [
            ("object-set digest exact limit", owner_bytes),
            ("object-set digest limit plus one", owner_bytes + 1),
        ] {
            let mut ledger = ledger(label);
            let precharged = precharge_retained(&mut ledger, remaining);
            let mut digest = source_object_set_digest_ordered_with_poll(
                &records,
                [1, 0],
                &mut ledger,
                || Ok(()),
            )
            .expect("digest reservation should fit");
            digest.release(&mut ledger).expect("digest release");
            release_precharge(&mut ledger, precharged);
            assert_eq!(ledger.open_permit_count(), 0);
        }
    }

    #[test]
    fn object_set_digest_owner_cancellation_and_error_leave_no_current_bytes() {
        let records = records();
        let mut cancelled = ledger("object-set digest cancellation");
        let cancellation =
            source_object_set_digest_ordered_with_poll(&records, [1, 0], &mut cancelled, || {
                Err(CliError::Usage("cancelled".to_string()))
            });
        assert!(cancellation.is_err());
        assert_eq!(retained_current(&cancelled), 0);
        assert_eq!(cancelled.open_permit_count(), 0);

        let mut invalid = ledger("object-set digest invalid index");
        let error = source_object_set_digest_ordered_with_poll(
            &records,
            [records.len()],
            &mut invalid,
            || Ok(()),
        );
        assert!(error.is_err());
        assert_eq!(retained_current(&invalid), 0);
        assert_eq!(invalid.open_permit_count(), 0);
    }

    #[test]
    fn object_set_digest_owner_keeps_bytes_until_explicit_release() {
        let records = records();
        let expected = source_object_set_digest(&records).expect("legacy digest");
        let mut ledger = ledger("object-set digest transfer");
        let mut digest =
            source_object_set_digest_ordered_with_poll(&records, [1, 0], &mut ledger, || Ok(()))
                .expect("accounted digest");

        assert!(retained_current(&ledger) > 0);
        assert_eq!(digest.as_str(), expected);
        assert_eq!(retained_current(&ledger), digest.retained_bytes());
        digest.release(&mut ledger).expect("digest release");
        assert_eq!(retained_current(&ledger), 0);
        assert_eq!(ledger.open_permit_count(), 0);
    }

    #[test]
    fn object_set_digest_owner_rejects_foreign_ledger_without_losing_ownership() {
        let records = records();
        let mut owner_ledger = ledger("object-set digest owner ledger");
        let mut foreign_ledger = ledger("object-set digest foreign ledger");
        let mut digest =
            source_object_set_digest_ordered_with_poll(&records, [1, 0], &mut owner_ledger, || {
                Ok(())
            })
            .expect("accounted digest");

        assert!(digest.release(&mut foreign_ledger).is_err());
        assert_eq!(retained_current(&foreign_ledger), 0);
        assert!(retained_current(&owner_ledger) > 0);
        digest.release(&mut owner_ledger).expect("owner release");
        assert_eq!(retained_current(&owner_ledger), 0);
        assert_eq!(owner_ledger.open_permit_count(), 0);
        assert_eq!(foreign_ledger.open_permit_count(), 0);
    }
}
