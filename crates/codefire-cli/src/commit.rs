//! Sealing a commit: the commit transaction and its recovery.

use crate::automation::change_coverage_data_json;
use crate::cli_error::{CliError, OpenRecoveryPendingError};
use crate::commit_active_reset::{
    build_commit_active_reset_authority, commit_active_reset_complete,
    commit_active_reset_unstarted, preflight_commit_active_reset_authority,
    reset_commit_active_state, transition_commit_transaction_to_active_reset,
    validate_commit_active_reset_authority,
};
use crate::exit_code::{verification_exit_code, ExitCode};
use crate::fire::active_ledger_transaction::reject_pending_active_ledger_transition;
use crate::idempotency::{
    idempotency_payload_hash, idempotency_record_path, idempotency_result_plan,
    require_idempotency_key, verify_idempotency_record,
};
use crate::json_files::{
    read_active_json_optional, read_active_json_required, write_active_json_atomic_bounded,
    write_json_atomic,
};
use crate::manifest::{
    build_manifest_fingerprint, build_manifest_plan, manifest_blob_payload,
    safe_relative_manifest_path,
};
use crate::open_context::open_context;
use crate::remote::read_optional_json;
use crate::repo_lock::{acquire_locked_open_context_for_recovery, RepoLock};
use crate::scan_verify::{compute_verify_with_cache, ensure_verification_subject_matches_worktree};
use crate::secure_worktree::StagedWorktree;
use crate::test_hooks::{
    maybe_corrupt_commit_candidate_for_test, maybe_fail_commit_transaction_for_test,
    maybe_mutate_commit_worktree_for_test,
};
use crate::{
    active_state, commit_parents_for_open, decode_base64, load_branch_record, now_iso_utc,
    required_string, signatures, verification, write_branch_record_reference, CommitOptions,
    CommitResult, OpenContext,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn run_commit(options: &CommitOptions) -> Result<CommitResult, CliError> {
    let lock;
    let context;
    if options.dry_run {
        lock = None;
        context = open_context(&options.path)?;
    } else {
        let (acquired_lock, locked_context) =
            acquire_locked_open_context_for_recovery(&options.path, &options.lock, "commit")?;
        lock = Some(acquired_lock);
        context = locked_context;
    }
    reject_pending_active_ledger_transition(&context.active_state)?;
    let idempotency_payload = commit_idempotency_payload(options, &context);
    let active_state_path = context.active_state.path().to_path_buf();
    if options.dry_run {
        reject_pending_commit_transaction_for_preview(&context.active_state)?;
    } else if let Some(result) = recover_commit_transaction(
        &context,
        lock.as_ref()
            .expect("live commit must hold repository lock"),
        &context.active_state,
        options,
        &idempotency_payload,
    )? {
        return Ok(result);
    }
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_commit_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    let verify_execution =
        compute_verify_with_cache(&context.open_dir, lock.as_ref(), &mut validation_cache)?;
    let verification = verify_execution.verification;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "after_verify")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "after verification",
    )?;

    let objects = context.repo_root.join(".codefire").join("objects");
    let scan = verify_execution.scan;
    let fires: Vec<codefire_core::Fire> =
        read_active_json_optional(&context.active_state, Path::new("fires.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let extinguished_fires = fires
        .iter()
        .filter(|fire| fire.status == "extinguished")
        .map(|fire| fire.fire_uid.clone())
        .collect::<Vec<_>>();
    let resolutions: Vec<codefire_core::Resolution> =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let extinguished_fire_count = extinguished_fires.len();
    let active_resolution_count = resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
        .count();
    let evidence_ref_count = resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
        .flat_map(|resolution| resolution.evidence_refs.iter())
        .count();
    let summary = CommitOperationSummary {
        extinguished_fire_count,
        active_resolution_count,
        evidence_ref_count,
    };
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    let parents = commit_parents_for_open(&context, &context.active_state)?;
    let plan = commit_operation_plan(
        options,
        &context,
        &active_state_path,
        &scan,
        &verification,
        &parents,
        &summary,
    );
    let decision = codefire_core::committable_decision(&verification);
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "before commit decision publication",
    )?;
    if !decision.is_committable() {
        return Ok(CommitResult {
            repo_root: context.repo_root,
            open_dir: context.open_dir,
            commit_id: String::new(),
            branch: context.branch,
            dry_run: options.dry_run,
            blocked: true,
            exit_code: verification_exit_code(&verification),
            plan,
            blocked_report: Some(verification::render_verification(&verification, true, true)),
        });
    }
    if options.dry_run {
        return Ok(CommitResult {
            repo_root: context.repo_root,
            open_dir: context.open_dir,
            commit_id: String::new(),
            branch: context.branch,
            dry_run: true,
            blocked: false,
            exit_code: ExitCode::Success,
            plan,
            blocked_report: None,
        });
    }

    let signature_required = policy.require_commit_signature;
    let transaction_id = begin_commit_transaction(
        &context,
        &context.active_state,
        options,
        &parents,
        &idempotency_payload,
        &plan,
    )?;
    let signing_key = signatures::commit_signing_key(signature_required)?;
    let manifest_plan = build_manifest_plan(&context.open_dir)?;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "after_manifest_plan")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "after manifest planning",
    )?;
    let manifest_id = manifest_plan.manifest.object_id.clone();
    let atom_plan = planned_commit_object(
        "atom_index",
        None,
        "atom_index",
        serde_json::to_value(&scan.atom_index)?,
        CommitObjectStage::SealedRoot,
    )?;
    let atom_id = atom_plan.object_id.clone();
    let trace_plan = planned_commit_object(
        "trace_graph",
        None,
        "trace_graph",
        serde_json::to_value(&scan.trace_graph)?,
        CommitObjectStage::SealedRoot,
    )?;
    let trace_id = trace_plan.object_id.clone();
    let fire_plan = planned_commit_object(
        "fire_ledger",
        None,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": fires}),
        CommitObjectStage::SealedRoot,
    )?;
    let fire_id = fire_plan.object_id.clone();
    let resolution_plan = planned_commit_object(
        "resolution_ledger",
        None,
        "resolution_ledger",
        json!({"type": "resolution_ledger", "version": 1, "resolutions": resolutions}),
        CommitObjectStage::SealedRoot,
    )?;
    let resolution_id = resolution_plan.object_id.clone();
    let verification_plan = planned_commit_object(
        "verification",
        None,
        "verification",
        serde_json::to_value(&verification)?,
        CommitObjectStage::SealedRoot,
    )?;
    let verification_id = verification_plan.object_id.clone();
    let policy_plan = planned_commit_object(
        "policy",
        None,
        "policy",
        json!({
            "type": "policy",
            "version": 1,
            "policy": policy,
            "effective_policy": parsed_policy.effective,
        }),
        CommitObjectStage::SealedRoot,
    )?;
    let policy_id = policy_plan.object_id.clone();

    let commit = json!({
        "type": "commit",
        "version": 1,
        "root_schema_version": 2,
        "semantic_validation_profile": codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE,
        "parents": parents,
        "message": options.message,
        "roots": {
            "content_manifest": manifest_id,
            "atom_index": atom_id,
            "trace_graph": trace_id,
            "fire_delta": fire_id,
            "resolution_ledger": resolution_id,
            "verification": verification_id,
            "policy": policy_id,
        },
        "certificate": {
            "result": decision.outcome.as_str(),
            "subject": &verification.subject,
            "profile": "local_verification_v1",
            "phase": "pre_commit",
            "open_required_fires": verification.open_required_fires,
            "failed_checks": verification.failed_checks.len(),
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "missing_evidence_refs": verification.missing_evidence_refs.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
            "effective_policy_digest": verification.effective_policy.as_ref().map(|policy| &policy.digest),
            "change_coverage_digest": verification.change_coverage.as_ref().map(|coverage| &coverage.digest),
        },
        "changed_atoms": scan.changed_atoms,
        "extinguished_fires": extinguished_fires,
        "created_at": now_iso_utc(),
    });
    let commit = signatures::maybe_sign_commit(
        commit,
        options.signer.as_deref(),
        options.key_id.as_deref(),
        signing_key.as_deref(),
    )?;
    let commit = maybe_corrupt_commit_candidate_for_test(&active_state_path, commit);
    let candidate_plan = planned_commit_object(
        "candidate_commit",
        None,
        "commit",
        commit,
        CommitObjectStage::CandidateCommit,
    )?;
    let commit_id = candidate_plan.object_id.clone();
    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["head"] = Value::String(commit_id.clone());
    branch["state"] = Value::String("open-clean".to_string());
    let branch_snapshot_plan = planned_commit_object(
        "branch_snapshot",
        None,
        "branch",
        branch.clone(),
        CommitObjectStage::BranchSnapshot,
    )?;

    // Every immutable role is declared before the first store. Identical file
    // contents may intentionally produce duplicate object IDs; each path keeps
    // its own ordered role entry and therefore its own planned -> stored state.
    let mut planned_objects = manifest_plan.blobs;
    planned_objects.push(manifest_plan.manifest);
    planned_objects.push(atom_plan);
    planned_objects.push(trace_plan);
    planned_objects.push(fire_plan);
    planned_objects.push(resolution_plan);
    planned_objects.push(verification_plan);
    planned_objects.push(policy_plan);
    planned_objects.push(candidate_plan);
    planned_objects.push(branch_snapshot_plan);
    plan_commit_transaction_objects(&context.active_state, &transaction_id, &planned_objects)?;
    let candidate_index = planned_objects
        .iter()
        .position(|planned| planned.role == "candidate_commit")
        .expect("commit plan must contain a candidate");
    for (index, planned) in planned_objects
        .iter_mut()
        .enumerate()
        .take(candidate_index + 1)
    {
        store_planned_commit_object(
            &objects,
            &context.open_dir,
            &context.active_state,
            &transaction_id,
            index,
            planned,
        )?;
    }
    maybe_fail_commit_transaction_for_test(&active_state_path, "before_candidate_validation")?;
    let candidate_commit_path =
        codefire_store::object_record_path(&objects, &commit_id).unwrap_or_else(|| objects.clone());
    let validated_candidate = codefire_store::validate_sealed_commit_pinned_with_cache(
        &objects,
        &commit_id,
        &mut validation_cache,
    )
    .map_err(|error| CliError::StoreContext {
        error: Box::new(error),
        repo_root: context.repo_root.clone(),
        path: candidate_commit_path,
    })?;
    let authenticated_candidate = validated_candidate.read_commit()?;
    signatures::verify_local_commit_signature(
        &authenticated_candidate,
        signing_key.as_deref(),
        signature_required,
    )?;
    for (index, planned) in planned_objects
        .iter_mut()
        .enumerate()
        .skip(candidate_index + 1)
    {
        store_planned_commit_object(
            &objects,
            &context.open_dir,
            &context.active_state,
            &transaction_id,
            index,
            planned,
        )?;
    }
    lock.as_ref()
        .expect("live commit must hold repo lock")
        .ensure_repo_root(&context.repo_root)?;
    update_commit_transaction(&context.active_state, &transaction_id, "publishing_branch")?;
    validated_candidate
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: objects.clone(),
        })?;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "before_branch_publish")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "before branch publication",
    )?;
    write_branch_record_reference(&context.repo_root, &branch)?;
    validated_candidate
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: objects.clone(),
        })?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_branch_publish_before_registry",
    )?;
    let mut registry = context.registry.clone();
    registry["open"]["current_base_commit"] = Value::String(commit_id.clone());
    registry["state"]["last_known"] = Value::String("open-clean".to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_registry_publish_before_phase",
    )?;
    transition_commit_transaction_to_active_reset(&context.active_state, &transaction_id)?;
    reset_commit_active_state(&context.active_state, &transaction_id)?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_active_reset_before_marker_clear",
    )?;
    let result = CommitResult {
        repo_root: context.repo_root.clone(),
        open_dir: context.open_dir.clone(),
        commit_id,
        branch: context.branch,
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan,
        blocked_report: None,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        update_commit_transaction(
            &context.active_state,
            &transaction_id,
            "publishing_idempotency",
        )?;
        ensure_commit_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
        maybe_fail_commit_transaction_for_test(
            &active_state_path,
            "after_idempotency_store_before_marker_update",
        )?;
    }
    update_commit_transaction(
        &context.active_state,
        &transaction_id,
        "idempotency_published",
    )?;
    complete_commit_transaction(&context.active_state)?;
    Ok(result)
}

pub(crate) fn commit_idempotency_payload(options: &CommitOptions, context: &OpenContext) -> Value {
    json!({
        "command": "commit",
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "message": &options.message,
        "signer": &options.signer,
        "key_id": &options.key_id,
    })
}

pub(crate) fn load_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<CommitResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "commit", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "commit", key, payload)?;
    Ok(Some(CommitResult {
        repo_root: repo_root.to_path_buf(),
        open_dir: record
            .get("result")
            .and_then(|result| result.get("open_dir"))
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_default(),
        commit_id: required_string(&record, &["result", "commit_id"])?,
        branch: required_string(&record, &["result", "branch"])?,
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan: idempotency_result_plan(&record, &path)?,
        blocked_report: None,
    }))
}

pub(crate) fn save_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &CommitResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "commit", key);
    let record = json!({
        "version": 1,
        "command": "commit",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "commit_id": &result.commit_id,
            "branch": &result.branch,
            "open_dir": &result.open_dir,
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

pub(crate) fn ensure_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &CommitResult,
) -> Result<(), CliError> {
    let path = idempotency_record_path(repo_root, "commit", key);
    if let Some(existing) = read_optional_json(&path)? {
        verify_idempotency_record(&existing, &path, "commit", key, payload)?;
        let existing_commit = required_string(&existing, &["result", "commit_id"])?;
        let existing_branch = required_string(&existing, &["result", "branch"])?;
        let existing_open_dir = existing
            .pointer("/result/open_dir")
            .and_then(Value::as_str)
            .map(Path::new)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "commit idempotency result is missing open_dir: {}",
                    path.display()
                ))
            })?;
        if existing_commit != result.commit_id
            || existing_branch != result.branch
            || existing_open_dir != result.open_dir
            || existing.pointer("/result/plan") != Some(&result.plan)
        {
            return Err(CliError::IdempotencyConflict(format!(
                "commit idempotency result conflicts with recovered publication: {}",
                path.display()
            )));
        }
        return Ok(());
    }
    save_commit_idempotency(repo_root, key, payload, result)
}

pub(crate) struct CommitOperationSummary {
    pub(crate) extinguished_fire_count: usize,
    pub(crate) active_resolution_count: usize,
    pub(crate) evidence_ref_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitObjectStage {
    ManifestBlob,
    SealedRoot,
    CandidateCommit,
    BranchSnapshot,
}

#[derive(Debug)]
pub(crate) struct PlannedCommitObject {
    pub(crate) role: &'static str,
    pub(crate) path: Option<String>,
    pub(crate) object_type: &'static str,
    pub(crate) object_id: String,
    pub(crate) payload: Option<Value>,
    pub(crate) worktree_source: Option<PathBuf>,
    pub(crate) stage: CommitObjectStage,
}

impl PlannedCommitObject {
    pub(crate) fn marker_value(&self) -> Value {
        json!({
            "role": self.role,
            "path": &self.path,
            "object_type": self.object_type,
            "object_id": &self.object_id,
            "state": "planned",
        })
    }

    pub(crate) fn storing_phase(&self) -> &'static str {
        match self.stage {
            CommitObjectStage::ManifestBlob => "storing_manifest_blobs",
            CommitObjectStage::SealedRoot => "storing_commit_roots",
            CommitObjectStage::CandidateCommit => "storing_candidate_commit",
            CommitObjectStage::BranchSnapshot => "storing_branch_snapshot",
        }
    }

    pub(crate) fn stored_phase(&self) -> &'static str {
        match self.role {
            "manifest_blob" => "stored_manifest_blob",
            "content_manifest" => "stored_content_manifest",
            "atom_index" => "stored_atom_index",
            "trace_graph" => "stored_trace_graph",
            "fire_ledger" => "stored_fire_ledger",
            "resolution_ledger" => "stored_resolution_ledger",
            "verification" => "stored_verification",
            "policy" => "stored_policy",
            "candidate_commit" => "validating_candidate_commit",
            "branch_snapshot" => "stored_branch_snapshot",
            _ => "stored_commit_object",
        }
    }

    pub(crate) fn store_gap_failpoint(&self) -> &'static str {
        match self.stage {
            CommitObjectStage::ManifestBlob => "after_manifest_blob_store_before_marker",
            CommitObjectStage::SealedRoot => "after_root_store_before_marker",
            CommitObjectStage::CandidateCommit => "after_candidate_store_before_marker",
            CommitObjectStage::BranchSnapshot => "after_branch_snapshot_store_before_marker",
        }
    }
}

pub(crate) struct ManifestCommitPlan {
    pub(crate) blobs: Vec<PlannedCommitObject>,
    pub(crate) manifest: PlannedCommitObject,
}

pub(crate) fn commit_operation_plan(
    options: &CommitOptions,
    context: &OpenContext,
    active_state_path: &Path,
    scan: &codefire_core::ScanResult,
    verification: &codefire_core::Verification,
    parents: &[String],
    summary: &CommitOperationSummary,
) -> Value {
    const DEFAULT_COMMIT_CHANGED_ATOM_LIMIT: usize = 50;
    let changed_atom_limit = if options.full_output {
        scan.changed_atoms.len()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let changed_atoms = scan
        .changed_atoms
        .iter()
        .take(changed_atom_limit)
        .cloned()
        .collect::<Vec<_>>();
    let changed_atoms_sample = changed_atoms.clone();
    let changed_atoms_omitted = scan.changed_atoms.len().saturating_sub(changed_atoms.len());
    let non_atom_changed_file_limit = if options.full_output {
        scan.non_atom_changed_files.len()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let non_atom_changed_files = scan
        .non_atom_changed_files
        .iter()
        .take(non_atom_changed_file_limit)
        .cloned()
        .collect::<Vec<_>>();
    let non_atom_changed_files_omitted = scan
        .non_atom_changed_files
        .len()
        .saturating_sub(non_atom_changed_files.len());
    let change_coverage_limit = if options.full_output {
        scan.change_coverage
            .as_ref()
            .map(|coverage| coverage.entries.len())
            .unwrap_or_default()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let change_coverage =
        change_coverage_data_json(scan.change_coverage.as_ref(), change_coverage_limit);
    let decision = codefire_core::committable_decision(verification);
    let next_actions = if !decision.is_committable() {
        vec![json!({
            "kind": "verify",
            "command": format!("codefire verify --path {} --details --json", context.open_dir.display()),
            "target": {"branch": &context.branch, "open_dir": &context.open_dir, "repo_root": &context.repo_root},
        })]
    } else if options.dry_run {
        vec![json!({
            "kind": "apply_commit",
            "command": format!("codefire commit --path {} -m <message>", context.open_dir.display()),
            "target": {"branch": &context.branch, "open_dir": &context.open_dir, "repo_root": &context.repo_root},
        })]
    } else {
        Vec::new()
    };
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "commit",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "message": &options.message,
        "parents": parents,
        "changed_atom_count": scan.changed_atoms.len(),
        "changed_atoms": changed_atoms,
        "changed_atoms_sample": changed_atoms_sample,
        "changed_atoms_sample_limit": changed_atom_limit,
        "changed_atoms_omitted": changed_atoms_omitted,
        "changed_atoms_truncated": changed_atoms_omitted > 0,
        "non_atom_changed_file_count": scan.non_atom_changed_files.len(),
        "non_atom_changed_files": non_atom_changed_files,
        "non_atom_changed_files_omitted": non_atom_changed_files_omitted,
        "non_atom_changed_files_truncated": non_atom_changed_files_omitted > 0,
        "change_coverage": change_coverage,
        "extinguished_fire_count": summary.extinguished_fire_count,
        "active_resolution_count": summary.active_resolution_count,
        "evidence_ref_count": summary.evidence_ref_count,
        "verification": {
            "result": decision.outcome.as_str(),
            "primary_reason": decision.primary_reason,
            "open_required_fires": verification.open_required_fires,
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
            "failed_checks": verification.failed_checks.len(),
            "effective_policy": &verification.effective_policy,
            "change_coverage": &verification.change_coverage,
        },
        "objects": [
            "blob",
            "content_manifest",
            "atom_index",
            "trace_graph",
            "fire_ledger",
            "resolution_ledger",
            "verification",
            "policy",
            "commit",
            "branch",
        ],
        "transaction": {
            "marker": active_state_path.join("commit_transaction.json"),
            "cleanup": "removed_after_success",
        },
        "operations": [
            {"kind": "begin_commit_transaction", "path": active_state_path.join("commit_transaction.json")},
            {"kind": "store_objects", "path": context.repo_root.join(".codefire").join("objects")},
            {"kind": "update_branch_head", "branch": &context.branch},
            {"kind": "update_open_registry", "path": &context.registry_path},
            {"kind": "reset_active_state", "path": active_state_path},
            {"kind": "clear_commit_transaction", "path": active_state_path.join("commit_transaction.json")},
        ],
        "next_actions": next_actions,
    })
}

pub(crate) fn begin_commit_transaction(
    context: &OpenContext,
    active_state: &active_state::ActiveStateDir,
    options: &CommitOptions,
    parents: &[String],
    request_payload: &Value,
    operation_plan: &Value,
) -> Result<String, CliError> {
    let transaction_path = commit_transaction_path(active_state);
    if active_state.exists(Path::new("commit_transaction.json"))? {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction must be inspected before another commit can start: {}",
            transaction_path.display()
        )));
    }
    let started_at = now_iso_utc();
    let mut nonce = [0u8; 16];
    getrandom::getrandom(&mut nonce).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate commit transaction identity: {error}"
        ))
    })?;
    let transaction_id = format!(
        "commit_tx_{}",
        nonce
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    let active_reset = build_commit_active_reset_authority(active_state)?;
    write_pending_commit_transaction(
        active_state,
        &json!({
            "version": 1,
            "transaction_id": transaction_id,
            "status": "pending",
            "phase": "started",
            "branch": &context.branch,
            "open_dir": &context.open_dir,
            "parents": parents,
            "message": &options.message,
            "request_payload": request_payload,
            "idempotency_key": &options.idempotency_key,
            "operation_plan": operation_plan,
            "active_reset": active_reset,
            "objects": [],
            "started_at": started_at,
            "updated_at": started_at,
        }),
    )?;
    Ok(transaction_id)
}

pub(crate) fn update_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    phase: &str,
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed during commit".to_string(),
        ));
    }
    record["phase"] = Value::String(phase.to_string());
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

pub(crate) fn plan_commit_transaction_objects(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    planned_objects: &[PlannedCommitObject],
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed while planning immutable objects".to_string(),
        ));
    }
    let existing_objects = record
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("commit transaction objects must be a list".to_string())
        })?;
    if !existing_objects.is_empty() {
        return Err(CliError::InvalidRepository(
            "commit transaction objects were already planned".to_string(),
        ));
    }
    let mut candidate_objects = planned_objects
        .iter()
        .filter(|planned| planned.role == "candidate_commit");
    let candidate = candidate_objects.next().ok_or_else(|| {
        CliError::InvalidRepository(
            "commit transaction plan is missing the candidate commit".to_string(),
        )
    })?;
    if candidate_objects.next().is_some() {
        return Err(CliError::InvalidRepository(
            "commit transaction plan contains multiple candidate commits".to_string(),
        ));
    }
    record["phase"] = Value::String("objects_planned".to_string());
    record["objects"] = Value::Array(
        planned_objects
            .iter()
            .map(PlannedCommitObject::marker_value)
            .collect(),
    );
    record["candidate_commit"] = json!({
        "object_id": &candidate.object_id,
        "state": "planned",
    });
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

pub(crate) fn store_planned_commit_object(
    objects_root: &Path,
    open_dir: &Path,
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    object_index: usize,
    planned: &mut PlannedCommitObject,
) -> Result<(), CliError> {
    update_commit_transaction(active_state, transaction_id, planned.storing_phase())?;
    let payload = match planned.payload.take() {
        Some(payload) => payload,
        None => {
            let source = planned.worktree_source.as_ref().ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "planned {} has neither inline payload nor worktree source",
                    planned.role
                ))
            })?;
            manifest_blob_payload(&fs::read(open_dir.join(source))?)
        }
    };
    let materialized_id = codefire_store::object_id(planned.object_type, &payload)?;
    if materialized_id != planned.object_id {
        return Err(CliError::InvalidRepository(format!(
            "planned {} source changed before storage: planned {}, materialized {materialized_id}",
            planned.role, planned.object_id
        )));
    }
    let stored_id = codefire_store::store_object(objects_root, planned.object_type, payload)?;
    if stored_id != planned.object_id {
        return Err(CliError::InvalidRepository(format!(
            "stored {} id differs from planned object: planned {}, stored {stored_id}",
            planned.role, planned.object_id
        )));
    }
    maybe_fail_commit_transaction_for_test(active_state.path(), planned.store_gap_failpoint())?;
    mark_commit_object_stored(active_state, transaction_id, object_index, planned)?;
    if planned.role == "content_manifest" {
        maybe_fail_commit_transaction_for_test(active_state.path(), "after_content_manifest")?;
    }
    Ok(())
}

pub(crate) fn mark_commit_object_stored(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    object_index: usize,
    planned: &PlannedCommitObject,
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed while recording an immutable object".to_string(),
        ));
    }
    let object = record
        .get_mut("objects")
        .and_then(Value::as_array_mut)
        .and_then(|objects| objects.get_mut(object_index))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "commit transaction object entry {object_index} is missing or invalid"
            ))
        })?;
    let marker_string = |field: &str| {
        object.get(field).and_then(Value::as_str).ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "commit transaction object entry {object_index} must contain string field {field}"
            ))
        })
    };
    if marker_string("role")? != planned.role
        || marker_string("object_type")? != planned.object_type
        || marker_string("object_id")? != planned.object_id
        || marker_string("state")? != "planned"
        || object.get("path").and_then(Value::as_str) != planned.path.as_deref()
    {
        return Err(CliError::InvalidRepository(
            "planned commit object changed before storage completed".to_string(),
        ));
    }
    object.insert("state".to_string(), Value::String("stored".to_string()));
    if planned.role == "candidate_commit" {
        if required_string(&record, &["candidate_commit", "object_id"])? != planned.object_id {
            return Err(CliError::InvalidRepository(
                "planned candidate commit id changed before storage completed".to_string(),
            ));
        }
        record["candidate_commit"]["state"] = Value::String("stored".to_string());
    }
    record["phase"] = Value::String(planned.stored_phase().to_string());
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

pub(crate) fn complete_commit_transaction(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    active_state.remove_file_if_exists(Path::new("commit_transaction.json"))?;
    Ok(())
}

pub(crate) fn commit_transaction_path(active_state: &active_state::ActiveStateDir) -> PathBuf {
    active_state.display_path(Path::new("commit_transaction.json"))
}

pub(crate) const COMMIT_TRANSACTION_BYTE_LIMIT: u64 = 64 * 1024 * 1024;
pub(crate) const COMMIT_TRANSACTION_RECOVERY_RESERVE_BYTES: u64 = 4096;
pub(crate) const COMMIT_TRANSACTION_PENDING_BYTE_LIMIT: u64 =
    COMMIT_TRANSACTION_BYTE_LIMIT - COMMIT_TRANSACTION_RECOVERY_RESERVE_BYTES;

pub(crate) const COMMIT_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "objects_planned",
    "storing_manifest_blobs",
    "stored_manifest_blob",
    "storing_commit_roots",
    "stored_content_manifest",
    "stored_atom_index",
    "stored_trace_graph",
    "stored_fire_ledger",
    "stored_resolution_ledger",
    "stored_verification",
    "stored_policy",
    "storing_candidate_commit",
    "validating_candidate_commit",
    "storing_branch_snapshot",
    "stored_branch_snapshot",
    "publishing_branch",
    "resetting_active_state",
    "publishing_idempotency",
    "idempotency_published",
];

pub(crate) const COMMIT_PREPUBLICATION_PHASES: &[&str] = &[
    "started",
    "objects_planned",
    "storing_manifest_blobs",
    "stored_manifest_blob",
    "storing_commit_roots",
    "stored_content_manifest",
    "stored_atom_index",
    "stored_trace_graph",
    "stored_fire_ledger",
    "stored_resolution_ledger",
    "stored_verification",
    "stored_policy",
    "storing_candidate_commit",
    "validating_candidate_commit",
    "storing_branch_snapshot",
    "stored_branch_snapshot",
    "publishing_branch",
];

pub(crate) fn validate_commit_transaction_record(transaction: &Value) -> Result<(), String> {
    if transaction.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("commit_transaction.json version must be 1".to_string());
    }
    for field in [
        "transaction_id",
        "status",
        "phase",
        "branch",
        "open_dir",
        "message",
        "started_at",
        "updated_at",
    ] {
        if transaction.get(field).and_then(Value::as_str).is_none() {
            return Err(format!(
                "commit_transaction.json must contain string field {field}"
            ));
        }
    }
    let status = transaction["status"]
        .as_str()
        .expect("commit transaction status was checked");
    if status != "pending" {
        return Err(format!(
            "commit_transaction.json status must be pending, found {status}"
        ));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("commit transaction phase was checked");
    if !COMMIT_TRANSACTION_PHASES.contains(&phase) {
        return Err(format!(
            "commit_transaction.json contains unsupported phase: {phase}"
        ));
    }
    if transaction
        .get("idempotency_key")
        .is_some_and(|key| !key.is_null() && !key.is_string())
    {
        return Err("commit_transaction.json idempotency_key must be a string or null".to_string());
    }
    for field in ["request_payload", "operation_plan"] {
        if transaction
            .get(field)
            .is_some_and(|value| !value.is_object())
        {
            return Err(format!(
                "commit_transaction.json {field} must be an object when present"
            ));
        }
    }
    if matches!(
        phase,
        "resetting_active_state" | "publishing_idempotency" | "idempotency_published"
    ) && ["request_payload", "idempotency_key", "operation_plan"]
        .iter()
        .any(|field| transaction.get(*field).is_none())
    {
        return Err(format!(
            "commit_transaction.json phase {phase} requires request_payload, idempotency_key, and operation_plan"
        ));
    }
    if let Some(authority) = transaction.get("active_reset") {
        validate_commit_active_reset_authority(authority)?;
        if matches!(phase, "publishing_idempotency" | "idempotency_published")
            && !commit_active_reset_complete(authority)
        {
            return Err(format!(
                "commit_transaction.json phase {phase} requires a completed active_reset authority"
            ));
        }
        if phase != "resetting_active_state"
            && !matches!(phase, "publishing_idempotency" | "idempotency_published")
            && !commit_active_reset_unstarted(authority)
        {
            return Err(format!(
                "commit_transaction.json phase {phase} cannot contain active_reset progress"
            ));
        }
    }
    let parents = transaction
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| "commit_transaction.json must contain list field parents".to_string())?;
    if parents.iter().any(|parent| !parent.is_string()) {
        return Err("commit_transaction.json parents entries must be strings".to_string());
    }
    let objects = transaction
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| "commit_transaction.json must contain list field objects".to_string())?;
    let mut candidate_entry = None;
    let mut has_legacy_entry = false;
    for (index, entry) in objects.iter().enumerate() {
        if entry.is_string() {
            has_legacy_entry = true;
            continue;
        }
        let object = entry.as_object().ok_or_else(|| {
            format!(
                "commit_transaction.json object entry {index} must be a legacy string or object"
            )
        })?;
        for field in ["role", "object_type", "object_id", "state"] {
            if !object.get(field).is_some_and(Value::is_string) {
                return Err(format!(
                    "commit_transaction.json object entry {index} must contain string field {field}"
                ));
            }
        }
        let state = object["state"]
            .as_str()
            .expect("commit object state was checked");
        if !matches!(state, "planned" | "stored") {
            return Err(format!(
                "commit_transaction.json object entry {index} contains unsupported state: {state}"
            ));
        }
        if object
            .get("path")
            .is_some_and(|path| !path.is_null() && !path.is_string())
        {
            return Err(format!(
                "commit_transaction.json object entry {index} path must be a string or null"
            ));
        }
        if object.get("role").and_then(Value::as_str) == Some("candidate_commit")
            && candidate_entry.replace(entry).is_some()
        {
            return Err(
                "commit_transaction.json contains multiple candidate commit entries".to_string(),
            );
        }
    }
    if objects.is_empty() {
        if phase != "started" || transaction.get("candidate_commit").is_some() {
            return Err(
                "an empty commit transaction plan is valid only at phase started without a candidate"
                    .to_string(),
            );
        }
    } else if !has_legacy_entry {
        let candidate = candidate_entry.ok_or_else(|| {
            "commit_transaction.json object plan is missing candidate_commit".to_string()
        })?;
        let alias = transaction
            .get("candidate_commit")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                "commit_transaction.json object plan is missing candidate alias".to_string()
            })?;
        for field in ["object_id", "state"] {
            if !alias.get(field).is_some_and(Value::is_string) {
                return Err(format!(
                    "commit_transaction.json candidate alias must contain string field {field}"
                ));
            }
        }
        if alias.get("object_id") != candidate.get("object_id")
            || alias.get("state") != candidate.get("state")
        {
            return Err(
                "commit_transaction.json candidate alias does not match its object entry"
                    .to_string(),
            );
        }
    }
    Ok(())
}

pub(crate) fn read_commit_transaction(
    active_state: &active_state::ActiveStateDir,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(
        Path::new("commit_transaction.json"),
        COMMIT_TRANSACTION_BYTE_LIMIT,
    )?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn write_pending_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    transaction: &Value,
) -> Result<(), CliError> {
    write_active_json_atomic_bounded(
        active_state,
        Path::new("commit_transaction.json"),
        transaction,
        COMMIT_TRANSACTION_PENDING_BYTE_LIMIT,
        "commit transaction marker",
    )
}

pub(crate) fn recover_commit_transaction(
    context: &OpenContext,
    lock: &RepoLock,
    active_state: &active_state::ActiveStateDir,
    options: &CommitOptions,
    requested_payload: &Value,
) -> Result<Option<CommitResult>, CliError> {
    let path = commit_transaction_path(active_state);
    if !active_state.exists(Path::new("commit_transaction.json"))? {
        return Ok(None);
    }
    let transaction = read_commit_transaction(active_state)?;
    validate_commit_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if transaction.get("status").and_then(Value::as_str) != Some("pending") {
        return Err(CliError::InvalidRepository(format!(
            "commit transaction is not pending and cannot be recovered automatically: {}",
            path.display()
        )));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("validated commit transaction phase")
        .to_string();
    let marker_payload = transaction.get("request_payload").cloned();
    let marker_key = match transaction.get("idempotency_key") {
        Some(Value::String(key)) => Some(key.clone()),
        Some(Value::Null) => None,
        _ => {
            return Err(CliError::OpenRecoveryPending(Box::new(
                OpenRecoveryPendingError {
                    operation: "commit".to_string(),
                    phase,
                    repo_root: context.repo_root.clone(),
                    branch: context.branch.clone(),
                    open_dir: context.open_dir.clone(),
                },
            )))
        }
    };
    if marker_payload.as_ref() != Some(requested_payload)
        || marker_key.as_deref() != options.idempotency_key.as_deref()
    {
        return Err(CliError::OpenRecoveryPending(Box::new(
            OpenRecoveryPendingError {
                operation: "commit".to_string(),
                phase,
                repo_root: context.repo_root.clone(),
                branch: context.branch.clone(),
                open_dir: context.open_dir.clone(),
            },
        )));
    }
    let marker_payload = marker_payload.expect("exact commit retry has a request payload");
    let pending_transaction_id = transaction["transaction_id"]
        .as_str()
        .expect("validated commit transaction id")
        .to_string();
    let current_format_entries = transaction["objects"]
        .as_array()
        .expect("validated commit transaction objects")
        .iter()
        .all(Value::is_object);
    let marker_branch = transaction["branch"]
        .as_str()
        .expect("validated commit transaction branch");
    let marker_open_dir = Path::new(
        transaction["open_dir"]
            .as_str()
            .expect("validated commit transaction open_dir"),
    );
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    let branch_head = required_string(&branch, &["head"])?;
    let mut registry = context.registry.clone();
    let mut registry_base = required_string(&registry, &["open", "current_base_commit"])?;
    let marker_parents = transaction["parents"]
        .as_array()
        .expect("validated commit transaction parents")
        .iter()
        .map(|parent| {
            parent
                .as_str()
                .expect("validated commit transaction parent")
                .to_string()
        })
        .collect::<Vec<_>>();
    let current_parents = commit_parents_for_open(context, active_state)?;
    let can_abort_before_retry = COMMIT_PREPUBLICATION_PHASES.contains(&phase.as_str())
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && marker_parents == current_parents
        && branch_head == registry_base
        && current_parents.first() == Some(&branch_head);
    if can_abort_before_retry {
        archive_commit_transaction(
            active_state,
            transaction,
            "aborted",
            "aborted_before_publication",
            &phase,
        )?;
        return Ok(None);
    }

    let postpublication_phase = matches!(
        phase.as_str(),
        "publishing_branch"
            | "resetting_active_state"
            | "publishing_idempotency"
            | "idempotency_published"
    );
    let candidate_id = transaction
        .pointer("/candidate_commit/object_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let candidate_stored = transaction
        .pointer("/candidate_commit/state")
        .and_then(Value::as_str)
        == Some("stored");
    let every_object_stored = transaction["objects"]
        .as_array()
        .expect("validated commit transaction objects")
        .iter()
        .all(|entry| entry.get("state").and_then(Value::as_str) == Some("stored"));
    let objects_root = context.repo_root.join(".codefire").join("objects");
    let candidate_parents_match = match candidate_id.as_deref() {
        Some(candidate) if candidate_stored => {
            commit_object_parents(&objects_root, candidate)? == marker_parents
        }
        _ => false,
    };
    if phase == "publishing_branch"
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && candidate_stored
        && every_object_stored
    {
        let candidate = candidate_id.as_deref().expect("stored candidate has an id");
        let branch_is_candidate = branch_head == candidate;
        let registry_is_parent = marker_parents.first() == Some(&registry_base);
        let registry_is_candidate = registry_base == candidate;
        if branch_is_candidate
            && (registry_is_parent || registry_is_candidate)
            && branch.get("state").and_then(Value::as_str) == Some("open-clean")
            && (!registry_is_candidate
                || registry
                    .pointer("/state/last_known")
                    .and_then(Value::as_str)
                    == Some("open-clean"))
            && candidate_parents_match
            && commit_candidate_matches_open_subject(
                &objects_root,
                candidate,
                context,
                active_state,
                &marker_parents,
            )?
        {
            codefire_store::validate_sealed_commit(&objects_root, candidate).map_err(|error| {
                CliError::StoreContext {
                    error: Box::new(error),
                    repo_root: context.repo_root.clone(),
                    path: codefire_store::object_record_path(&objects_root, candidate)
                        .unwrap_or_else(|| objects_root.clone()),
                }
            })?;
            lock.ensure_repo_root(&context.repo_root)?;
            if registry_is_parent {
                registry["open"]["current_base_commit"] = Value::String(candidate.to_string());
                registry["state"]["last_known"] = Value::String("open-clean".to_string());
                write_json_atomic(&context.registry_path, &registry)?;
                registry_base = candidate.to_string();
            }
            transition_commit_transaction_to_active_reset(active_state, &pending_transaction_id)?;
            reset_commit_active_state(active_state, &pending_transaction_id)?;
        }
    }
    if phase == "resetting_active_state"
        && transaction.get("active_reset").is_some()
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && candidate_stored
        && every_object_stored
    {
        let candidate = candidate_id.as_deref().expect("stored candidate has an id");
        if branch_head == candidate
            && registry_base == candidate
            && branch.get("state").and_then(Value::as_str) == Some("open-clean")
            && registry
                .pointer("/state/last_known")
                .and_then(Value::as_str)
                == Some("open-clean")
            && candidate_parents_match
        {
            codefire_store::validate_sealed_commit(&objects_root, candidate).map_err(|error| {
                CliError::StoreContext {
                    error: Box::new(error),
                    repo_root: context.repo_root.clone(),
                    path: codefire_store::object_record_path(&objects_root, candidate)
                        .unwrap_or_else(|| objects_root.clone()),
                }
            })?;
            lock.ensure_repo_root(&context.repo_root)?;
            reset_commit_active_state(active_state, &pending_transaction_id)?;
        }
    }
    let transaction_after_reset = read_commit_transaction(active_state)?;
    validate_commit_transaction_record(&transaction_after_reset)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if matches!(
        transaction_after_reset.get("phase").and_then(Value::as_str),
        Some("publishing_idempotency" | "idempotency_published")
    ) {
        if let Some(authority) = transaction_after_reset.get("active_reset") {
            preflight_commit_active_reset_authority(active_state, authority)?;
        }
    }
    let durable_reset_progress_complete = transaction_after_reset
        .get("active_reset")
        .map(commit_active_reset_complete)
        .unwrap_or(true);
    let active_state_record = read_active_state_for_commit_recovery(active_state)?;
    let mut reset_files_absent = true;
    for name in [
        "fires.json",
        "resolutions.json",
        "scan.json",
        "verification.json",
        "state_transaction.json",
    ] {
        if active_state.exists(Path::new(name))? {
            reset_files_absent = false;
            break;
        }
    }
    let active_reset_complete = active_state_record.get("state").and_then(Value::as_str)
        == Some("open-clean")
        && reset_files_absent;
    let fully_published = candidate_id
        .as_deref()
        .is_some_and(|candidate| branch_head == candidate && registry_base == candidate)
        && branch.get("state").and_then(Value::as_str) == Some("open-clean")
        && registry
            .pointer("/state/last_known")
            .and_then(Value::as_str)
            == Some("open-clean");
    if !postpublication_phase
        || !current_format_entries
        || marker_branch != context.branch
        || marker_open_dir != context.open_dir
        || !candidate_stored
        || !every_object_stored
        || !candidate_parents_match
        || !durable_reset_progress_complete
        || !active_reset_complete
        || !fully_published
    {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction at phase {phase} requires explicit inspection; branch/open state or transaction subject may be partially published: {}",
            path.display()
        )));
    }
    let candidate_id = candidate_id.expect("fully published transaction has a candidate");
    codefire_store::validate_sealed_commit(&objects_root, &candidate_id).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: codefire_store::object_record_path(&objects_root, &candidate_id)
                .unwrap_or(objects_root),
        }
    })?;
    let operation_plan = transaction.get("operation_plan").cloned().ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "published commit transaction is missing operation_plan: {}",
            path.display()
        ))
    })?;
    let result = CommitResult {
        repo_root: context.repo_root.clone(),
        open_dir: context.open_dir.clone(),
        commit_id: candidate_id,
        branch: context.branch.clone(),
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan: operation_plan,
        blocked_report: None,
    };
    if let Some(key) = marker_key.as_deref() {
        ensure_commit_idempotency(&context.repo_root, key, &marker_payload, &result)?;
    }
    let completed_transaction = read_commit_transaction(active_state)?;
    archive_commit_transaction(
        active_state,
        completed_transaction,
        "recovered",
        "completed_published_commit",
        &phase,
    )?;
    Ok(Some(result))
}

pub(crate) fn read_active_state_for_commit_recovery(
    active_state: &active_state::ActiveStateDir,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(Path::new("state.json"), 1024 * 1024)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn commit_object_parents(
    objects: &Path,
    commit_id: &str,
) -> Result<Vec<String>, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    commit
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!("commit {commit_id} is missing its parents list"))
        })?
        .iter()
        .map(|parent| {
            parent.as_str().map(str::to_string).ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "commit {commit_id} contains a non-string parent"
                ))
            })
        })
        .collect()
}

pub(crate) fn commit_candidate_matches_open_subject(
    objects: &Path,
    commit_id: &str,
    context: &OpenContext,
    active_state: &active_state::ActiveStateDir,
    expected_parents: &[String],
) -> Result<bool, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let root_matches = |name: &str, object_type: &str, payload: &Value| {
        let expected = required_string(&commit, &["roots", name])?;
        Ok::<bool, CliError>(codefire_store::object_id(object_type, payload)? == expected)
    };

    let active_state_record = read_active_state_for_commit_recovery(active_state)?;
    if active_state_record
        .get("pending_merge_parent")
        .and_then(Value::as_str)
        != expected_parents.get(1).map(String::as_str)
    {
        return Ok(false);
    }

    let manifest = build_manifest_fingerprint(&context.open_dir)?;
    if !root_matches("content_manifest", "content_manifest", &manifest)? {
        return Ok(false);
    }
    let atom_index = codefire_core::build_atom_index(&context.open_dir)?;
    let atom_value = serde_json::to_value(&atom_index)?;
    if !root_matches("atom_index", "atom_index", &atom_value)? {
        return Ok(false);
    }
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    codefire_core::validate_trace_graph_endpoints(&atom_index, &trace_graph)?;
    let trace_value = serde_json::to_value(&trace_graph)?;
    if !root_matches("trace_graph", "trace_graph", &trace_value)? {
        return Ok(false);
    }

    let fires = read_active_json_optional(active_state, Path::new("fires.json"))?
        .unwrap_or_else(|| json!([]));
    if !root_matches(
        "fire_delta",
        "fire_ledger",
        &json!({"type": "fire_ledger", "version": 1, "fires": fires}),
    )? {
        return Ok(false);
    }
    let resolutions = read_active_json_optional(active_state, Path::new("resolutions.json"))?
        .unwrap_or_else(|| json!([]));
    if !root_matches(
        "resolution_ledger",
        "resolution_ledger",
        &json!({"type": "resolution_ledger", "version": 1, "resolutions": resolutions}),
    )? {
        return Ok(false);
    }
    let verification = read_active_json_required(active_state, Path::new("verification.json"))?;
    if !root_matches("verification", "verification", &verification)? {
        return Ok(false);
    }
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    if !root_matches(
        "policy",
        "policy",
        &json!({
            "type": "policy",
            "version": 1,
            "policy": parsed_policy.policy,
            "effective_policy": parsed_policy.effective,
        }),
    )? {
        return Ok(false);
    }
    Ok(true)
}

pub(crate) fn archive_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    mut transaction: Value,
    status: &str,
    action: &str,
    phase: &str,
) -> Result<(), CliError> {
    let transaction_id = transaction["transaction_id"]
        .as_str()
        .expect("validated commit transaction id");
    let suffix = transaction_id.strip_prefix("commit_tx_").filter(|suffix| {
        matches!(suffix.len(), 12 | 32) && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    let Some(suffix) = suffix else {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction uses an unsupported recovery identity: {}",
            commit_transaction_path(active_state).display()
        )));
    };
    let archive_name = format!("commit_transaction.{status}.commit_tx_{suffix}.json");
    let archive_relative = Path::new(&archive_name);
    let archive_path = active_state.display_path(archive_relative);
    transaction["status"] = Value::String(status.to_string());
    transaction["recovery"] = json!({
        "action": action,
        "reason": "safe_retry",
        "phase": phase,
    });
    if active_state.exists(archive_relative)? {
        let bytes = active_state.read_required(archive_relative, COMMIT_TRANSACTION_BYTE_LIMIT)?;
        let existing: Value = serde_json::from_slice(&bytes)?;
        if existing != transaction {
            return Err(CliError::InvalidRepository(format!(
                "commit transaction recovery archive conflicts with the pending marker: {}",
                archive_path.display()
            )));
        }
    } else {
        write_active_json_atomic_bounded(
            active_state,
            archive_relative,
            &transaction,
            COMMIT_TRANSACTION_BYTE_LIMIT,
            "commit transaction recovery archive",
        )?;
    }
    active_state.remove_file_if_exists(Path::new("commit_transaction.json"))?;
    Ok(())
}

pub(crate) fn reject_pending_commit_transaction_for_preview(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    let path = commit_transaction_path(active_state);
    if !active_state.exists(Path::new("commit_transaction.json"))? {
        Ok(())
    } else {
        let transaction = read_commit_transaction(active_state)?;
        validate_commit_transaction_record(&transaction).map_err(|message| {
            CliError::InvalidRepository(format!("{message}: {}", path.display()))
        })?;
        Err(CliError::InvalidRepository(format!(
            "pending commit transaction must be inspected before commit preview; dry-run never recovers or mutates it: {}",
            path.display()
        )))
    }
}

pub(crate) fn materialize_commit(
    validated: &codefire_store::ValidatedSealedCommit,
    target: &StagedWorktree,
) -> Result<(), CliError> {
    let manifest = validated.read_root("content_manifest")?;
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let rel_path = required_string(entry, &["path"])?;
        let blob_id = required_string(entry, &["blob"])?;
        let blob = validated.read_object(&blob_id)?;
        let encoding = required_string(&blob, &["encoding"])?;
        if encoding != "base64" {
            return Err(CliError::InvalidRepository(format!(
                "unsupported blob encoding: {encoding}"
            )));
        }
        let content = required_string(&blob, &["content"])?;
        let output_path = safe_relative_manifest_path(&rel_path)?;
        target
            .write_file(&output_path, &decode_base64(&content)?)
            .map_err(CliError::Io)?;
    }
    Ok(())
}

pub(crate) fn planned_commit_object(
    role: &'static str,
    path: Option<String>,
    object_type: &'static str,
    payload: Value,
    stage: CommitObjectStage,
) -> Result<PlannedCommitObject, CliError> {
    let object_id = codefire_store::object_id(object_type, &payload)?;
    Ok(PlannedCommitObject {
        role,
        path,
        object_type,
        object_id,
        payload: Some(payload),
        worktree_source: None,
        stage,
    })
}
