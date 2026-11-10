//! Scan and verify computation, including verification command execution.

use crate::cli_error::CliError;
use crate::extinguish::collect_missing_evidence_refs;
use crate::json_files::{
    read_active_json_optional, read_active_json_required, write_active_json_atomic,
};
use crate::manifest::{
    build_manifest_fingerprint, manifest_file_changes_since_base,
    source_manifest_unchanged_validated, to_posix,
};
use crate::merge_resolution::unresolved_merge_conflicts_from_state;
use crate::open_context::{open_context, open_context_for_recovery};
use crate::repo_lock::RepoLock;
use crate::state_transaction::{
    begin_state_transaction, ensure_validated_store_subject, invalidate_active_verification,
    persist_transactional_open_state, update_state_transaction_phase, StateTransactionAuthority,
};
use crate::test_hooks::{
    maybe_fail_state_transaction_for_test, maybe_swap_active_state_after_validation_for_test,
    maybe_swap_validated_store_before_verify_persist_for_test,
};
use crate::{
    assessment_subject, command_execution, find_repo_root, now_iso_utc, open_snapshot,
    required_string, scan_branch_state, scan_change_count, secret_redaction, LockOptions,
    OpenContext, OpenContextAuthority, ScanExecution, VerifyExecution,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub(crate) fn run_scan(start: &Path) -> Result<codefire_core::ScanResult, CliError> {
    run_scan_with_lock_options(start, &LockOptions::default())
}

pub(crate) fn run_scan_with_lock_options(
    start: &Path,
    options: &LockOptions,
) -> Result<codefire_core::ScanResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    Ok(compute_scan_with_cache_for_recovery(
        start,
        Some(&lock),
        &mut validation_cache,
        Some("scan"),
    )?
    .scan)
}

pub(crate) fn compute_scan(start: &Path) -> Result<ScanExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_scan_with_cache(start, None, &mut validation_cache)
}

pub(crate) fn compute_scan_locked(
    start: &Path,
    lock: &RepoLock,
) -> Result<ScanExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_scan_with_cache(start, Some(lock), &mut validation_cache)
}

pub(crate) fn compute_scan_with_cache(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Result<ScanExecution, CliError> {
    compute_scan_with_cache_for_recovery(start, lock, validation_cache, None)
}

pub(crate) fn compute_scan_with_cache_for_recovery(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
    recovery_operation: Option<&str>,
) -> Result<ScanExecution, CliError> {
    let context = match recovery_operation {
        Some(operation) => open_context_for_recovery(start, operation)?,
        None => open_context(start)?,
    };
    let recovering_verify_state = recovery_operation == Some("verify")
        && matches!(
            &context.authority,
            OpenContextAuthority::Verified(snapshot)
                if matches!(
                    &snapshot.disposition,
                    open_snapshot::OpenSnapshotDisposition::PendingState { operation, .. }
                        if operation == "verify"
                )
        );
    if let Some(lock) = lock {
        lock.ensure_repo_root(&context.repo_root)?;
    }
    maybe_swap_active_state_after_validation_for_test(&context)?;
    let objects = context.repo_root.join(".codefire").join("objects");
    let base_commit = required_string(&context.registry, &["open", "current_base_commit"])?;
    let base_commit_path = codefire_store::object_record_path(&objects, &base_commit)
        .unwrap_or_else(|| objects.clone());
    let validated_base = codefire_store::validate_sealed_commit_pinned_with_cache(
        &objects,
        &base_commit,
        validation_cache,
    )
    .map_err(|error| CliError::StoreContext {
        error: Box::new(error),
        repo_root: context.repo_root.clone(),
        path: base_commit_path,
    })?;
    let active_state = &context.active_state;
    let current = codefire_core::build_atom_index(&context.open_dir)?;
    let base_index = load_validated_base_atom_index(&validated_base)?;
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    codefire_core::validate_trace_graph_endpoints(&current, &trace_graph).map_err(|error| {
        CliError::CoreContext {
            error,
            repo_root: context.repo_root.clone(),
            path: context.open_dir.join("codefire.links.yaml"),
        }
    })?;
    let fires = read_active_json_optional(active_state, Path::new("fires.json"))?
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let state_data = read_active_json_optional(active_state, Path::new("state.json"))?
        .unwrap_or_else(|| json!({"state": "open-clean"}));
    let reason = if state_data.get("pending_merge_parent").is_some() {
        "merge_changed"
    } else {
        "atom_changed"
    };
    let now = now_iso_utc();
    let source_unchanged = source_manifest_unchanged_validated(&validated_base, &context.open_dir)?;
    let manifest_changes = manifest_file_changes_since_base(&validated_base, &context.open_dir)?;
    let (mut scan, fires) = codefire_core::build_scan_result_against_base(
        current,
        &base_index,
        trace_graph,
        fires,
        base_commit,
        codefire_core::ScanBuildOptions {
            reason,
            now: &now,
            source_unchanged,
        },
    )?;
    let coverage = codefire_core::build_change_coverage(
        manifest_changes,
        &scan.atom_index,
        &base_index,
        &scan.changed_atoms,
        scan.tool_migration.clone(),
    )?;
    scan.non_atom_changed_files = coverage
        .entries
        .iter()
        .filter(|entry| {
            entry.relation != codefire_core::ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans
        })
        .map(|entry| codefire_core::NonAtomFileChange {
            path: entry.path.clone(),
            status: entry.status.clone(),
            base_blob: entry.base_blob.clone(),
            current_blob: entry.current_blob.clone(),
        })
        .collect();
    scan.change_coverage = Some(coverage);
    if lock.is_some() && !recovering_verify_state {
        ensure_validated_store_subject(&context, &validated_base)?;
        let state = scan_branch_state(scan_change_count(&scan), scan.open_fires.len());
        begin_state_transaction(&context, "scan", state)?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "started")?;
        invalidate_active_verification(active_state)?;
        update_state_transaction_phase(active_state, "verification_invalidated")?;
        maybe_fail_state_transaction_for_test(
            active_state.path(),
            "scan",
            "verification_invalidated",
        )?;
        write_active_json_atomic(
            active_state,
            Path::new("fires.json"),
            &serde_json::to_value(&fires)?,
        )?;
        update_state_transaction_phase(active_state, "fires_written")?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "fires_written")?;
        write_active_json_atomic(
            active_state,
            Path::new("scan.json"),
            &serde_json::to_value(&scan)?,
        )?;
        update_state_transaction_phase(active_state, "scan_written")?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "scan_written")?;
        persist_transactional_open_state(
            &context,
            "scan",
            state,
            &validated_base,
            StateTransactionAuthority::Scan {
                scan: &scan,
                fires: &fires,
            },
        )?;
    }
    Ok(ScanExecution {
        context,
        scan,
        fires,
        validated_base,
    })
}

#[cfg(test)]
pub(crate) fn run_verify(start: &Path) -> Result<codefire_core::Verification, CliError> {
    Ok(run_verify_with_lock(start)?.verification)
}

#[cfg(test)]
pub(crate) fn run_verify_with_lock(start: &Path) -> Result<VerifyExecution, CliError> {
    run_verify_with_lock_options(start, &LockOptions::default())
}

pub(crate) fn run_verify_with_lock_options(
    start: &Path,
    options: &LockOptions,
) -> Result<VerifyExecution, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    compute_verify_locked(start, &lock)
}

pub(crate) fn compute_verify(start: &Path) -> Result<VerifyExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_verify_with_cache(start, None, &mut validation_cache)
}

pub(crate) fn compute_verify_locked(
    start: &Path,
    lock: &RepoLock,
) -> Result<VerifyExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_verify_with_cache(start, Some(lock), &mut validation_cache)
}

pub(crate) fn compute_verify_with_cache(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Result<VerifyExecution, CliError> {
    // Verification must reject an invalid/unsafe policy before its scan phase publishes any
    // active-state mutation. Parse again from the post-scan context below so the decision uses the
    // same final path resolution as the scan authority.
    let policy_preflight = if lock.is_some() {
        open_context_for_recovery(start, "verify")?
    } else {
        open_context(start)?
    };
    if let Some(lock) = lock {
        lock.ensure_repo_root(&policy_preflight.repo_root)?;
    }
    let preflight_policy =
        codefire_core::parse_effective_verification_policy(&policy_preflight.open_dir)?;
    validate_verification_command_plans(&policy_preflight.open_dir, &preflight_policy.policy)?;
    let merge_state_preflight =
        read_active_json_required(&policy_preflight.active_state, Path::new("state.json"))?;
    unresolved_merge_conflicts_from_state(&merge_state_preflight)?;
    let ScanExecution {
        context,
        scan,
        fires,
        validated_base,
    } = compute_scan_with_cache_for_recovery(
        start,
        lock,
        validation_cache,
        lock.map(|_| "verify"),
    )?;
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    validate_verification_command_plans(&context.open_dir, policy)?;
    let trace_policy = codefire_core::TracePolicy {
        required_links: policy.required_links.clone(),
    };
    let missing_required_links =
        codefire_core::required_link_missing(&scan.atom_index, &scan.trace_graph, &trace_policy);
    let subject = assessment_subject::live_open_subject(&context, &scan)?;
    let command_results = run_verification_commands(&context.open_dir, policy, &subject)?;
    let resolutions: Vec<codefire_core::Resolution> =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let stale_resolutions = codefire_core::stale_resolutions(
        &scan.atom_index,
        &scan.trace_graph,
        policy,
        &resolutions,
    )?;
    let missing_evidence_refs = collect_missing_evidence_refs(&context.repo_root, &resolutions);
    let state = read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let unresolved_merge_conflicts = unresolved_merge_conflicts_from_state(&state)?;
    let mut verification = codefire_core::build_verification(
        &scan,
        codefire_core::VerificationInputs {
            unresolved_merge_conflicts,
            missing_required_links,
            failed_checks: command_results.failures,
            stale_resolutions,
            missing_evidence_refs,
            subject: Some(subject),
            execution: command_results.records,
        },
        policy,
        &now_iso_utc(),
    );
    verification.effective_policy = Some(parsed_policy.effective);
    if lock.is_some() {
        maybe_swap_validated_store_before_verify_persist_for_test(&context.repo_root)?;
        persist_verification_result_state(&context, &scan, &fires, &verification, &validated_base)?;
    }
    Ok(VerifyExecution { scan, verification })
}

pub(crate) fn persist_verification_result_state(
    context: &OpenContext,
    scan: &codefire_core::ScanResult,
    fires: &[codefire_core::Fire],
    verification: &codefire_core::Verification,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<(), CliError> {
    ensure_validated_store_subject(context, validated_base)?;
    let state = verification_open_state(verification, scan);
    begin_state_transaction(context, "verify", state)?;
    maybe_fail_state_transaction_for_test(context.active_state.path(), "verify", "started")?;
    write_active_json_atomic(
        &context.active_state,
        Path::new("verification.json"),
        &serde_json::to_value(verification)?,
    )?;
    update_state_transaction_phase(&context.active_state, "verification_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        "verify",
        "verification_written",
    )?;
    persist_transactional_open_state(
        context,
        "verify",
        state,
        validated_base,
        StateTransactionAuthority::Verify {
            scan,
            fires,
            verification,
        },
    )
}

pub(crate) fn verification_open_state(
    verification: &codefire_core::Verification,
    scan: &codefire_core::ScanResult,
) -> &'static str {
    if !codefire_core::committable_decision(verification).is_committable() {
        return "open-burning";
    }
    if scan_change_count(scan) == 0 && scan.open_fires.is_empty() {
        "open-clean"
    } else {
        "open-consistent"
    }
}

pub(crate) struct VerificationCommandBatch {
    pub(crate) failures: Vec<codefire_core::FailedCheck>,
    pub(crate) records: Vec<codefire_core::VerificationExecutionRecord>,
}

pub(crate) fn validate_verification_command_plans(
    open_dir: &Path,
    policy: &codefire_core::VerificationPolicy,
) -> Result<(), CliError> {
    for check in &policy.verification {
        resolve_verification_cwd(open_dir, &check.cwd)?;
        if secret_redaction::redact_text(&check.command).count > 0 {
            return Err(CliError::Usage(format!(
                "verification command '{}' contains secret-like inline metadata; use a secret-free helper and an explicitly inherited environment instead",
                check.id
            )));
        }
        if check
            .env
            .iter()
            .any(|(key, value)| secret_redaction::redact_text(&format!("{key}={value}")).count > 0)
        {
            return Err(CliError::Usage(format!(
                "verification command '{}' contains a secret-like explicit environment entry; policy environment values are persisted and cannot contain secrets",
                check.id
            )));
        }
    }
    Ok(())
}

pub(crate) fn run_verification_commands(
    open_dir: &Path,
    policy: &codefire_core::VerificationPolicy,
    subject: &codefire_core::AssessmentSubject,
) -> Result<VerificationCommandBatch, CliError> {
    let mut failures = Vec::new();
    let mut records = Vec::with_capacity(policy.verification.len());
    ensure_live_subject_matches_worktree(open_dir, subject, "before verification execution")?;
    for check in &policy.verification {
        let worktree_before = ensure_live_subject_matches_worktree(
            open_dir,
            subject,
            "before verification command execution",
        )?;
        let cwd = resolve_verification_cwd(open_dir, &check.cwd)?;
        let mut environment = if check.inherit_env {
            std::env::vars().collect::<BTreeMap<_, _>>()
        } else {
            command_execution::minimal_command_environment()
        };
        environment.extend(check.env.clone());
        let environment_policy = if check.inherit_env {
            "inherited_snapshot_with_explicit_overrides"
        } else {
            "minimal_allowlist_with_explicit_overrides"
        };
        let environment_digest = command_execution::canonical_digest(&json!(&environment))?;
        let input_digest = command_execution::canonical_digest(&json!({
            "type": "codefire_verification_execution_input",
            "version": 1,
            "profile": "trusted_local_shell_v1",
            "phase": "verification_command",
            "subject_digest": &subject.subject_digest,
            "command": &check.command,
            "cwd": &cwd.repo_relative,
            "environment_policy": environment_policy,
            "environment_digest": &environment_digest,
            "timeout_ms": check.timeout_ms,
            "max_output_bytes": check.max_output_bytes,
            "allow_failure": check.allow_failure,
        }))?;
        // Checks can run for a long time; say which one is running. This goes
        // to stderr so text and JSON results on stdout stay unchanged.
        eprintln!("codefire: running check '{}' ...", check.id);
        let output =
            command_execution::run_bounded_command(&command_execution::CommandExecutionSpec {
                program: "sh".to_string(),
                args: vec!["-c".to_string(), check.command.clone()],
                cwd: cwd.resolved.clone(),
                clear_env: true,
                env: environment,
                timeout_ms: check.timeout_ms,
                stdout_limit: check.max_output_bytes,
                stderr_limit: check.max_output_bytes,
            })?;
        let worktree_after = worktree_manifest_digest(open_dir)?;
        let subject_unchanged = worktree_before == worktree_after
            && subject.worktree_digest.as_deref() == Some(worktree_after.as_str());
        let stdout = secret_redaction::redact_text(&output.stdout.text);
        let stderr = secret_redaction::redact_text(&output.stderr.text);
        let command = secret_redaction::redact_text(&check.command);
        let redacted_value_count = command.count + stdout.count + stderr.count;
        let mut combined = String::with_capacity(stdout.value.len() + stderr.value.len() + 160);
        combined.push_str(&stdout.value);
        combined.push_str(&stderr.value);
        if output.stdout.truncated || output.stderr.truncated {
            combined.push_str(&format!(
                "\n[verification output truncated at {} bytes per stream]",
                check.max_output_bytes
            ));
        }
        if output.timed_out {
            combined.push_str(&format!(
                "\nverification command timed out after {}ms",
                check.timeout_ms
            ));
        }
        if !subject_unchanged {
            combined.push_str("\nverification subject changed while the command was running");
        }
        let success = output.success && subject_unchanged;
        let must_block =
            output.timed_out || !subject_unchanged || (!success && !check.allow_failure);
        if must_block {
            failures.push(codefire_core::FailedCheck {
                id: check.id.clone(),
                command: command.value,
                output: combined,
            });
        }
        let explicit_env_keys = check.env.keys().cloned().collect::<Vec<_>>();
        records.push(codefire_core::VerificationExecutionRecord {
            id: check.id.clone(),
            profile: "trusted_local_shell_v1".to_string(),
            phase: "verification_command".to_string(),
            trust_kind: "local_unisolated".to_string(),
            subject_digest: subject.subject_digest.clone(),
            input_digest,
            environment_digest,
            worktree_before,
            worktree_after,
            subject_unchanged,
            cwd: cwd.repo_relative,
            cwd_kind: "repo_relative".to_string(),
            environment_policy: environment_policy.to_string(),
            explicit_env_keys,
            timeout_ms: check.timeout_ms,
            max_output_bytes: check.max_output_bytes,
            stdout_bytes_seen: output.stdout.bytes_seen,
            stdout_bytes_stored: output.stdout.bytes_stored,
            stdout_truncated: output.stdout.truncated,
            stderr_bytes_seen: output.stderr.bytes_seen,
            stderr_bytes_stored: output.stderr.bytes_stored,
            stderr_truncated: output.stderr.truncated,
            exit_code: output.exit_code,
            success,
            timed_out: output.timed_out,
            termination_scope: output.termination_scope.to_string(),
            redacted_value_count,
        });
        if !subject_unchanged {
            break;
        }
    }
    Ok(VerificationCommandBatch { failures, records })
}

pub(crate) struct VerificationCwd {
    pub(crate) resolved: PathBuf,
    pub(crate) repo_relative: String,
}

pub(crate) fn resolve_verification_cwd(
    open_dir: &Path,
    requested: &str,
) -> Result<VerificationCwd, CliError> {
    let root = open_dir.canonicalize()?;
    let candidate = root.join(requested).canonicalize().map_err(|error| {
        CliError::Usage(format!(
            "verification cwd must exist below the open directory: {requested} ({error})"
        ))
    })?;
    if !candidate.is_dir() || !candidate.starts_with(&root) {
        return Err(CliError::Usage(format!(
            "verification cwd escapes the open directory: {requested}"
        )));
    }
    let relative = candidate
        .strip_prefix(&root)
        .expect("candidate checked below root");
    let repo_relative = if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        to_posix(relative)
    };
    Ok(VerificationCwd {
        resolved: candidate,
        repo_relative,
    })
}

pub(crate) fn worktree_manifest_digest(open_dir: &Path) -> Result<String, CliError> {
    let manifest = build_manifest_fingerprint(open_dir)?;
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&manifest)?)
    ))
}

pub(crate) fn ensure_verification_subject_matches_worktree(
    open_dir: &Path,
    verification: &codefire_core::Verification,
    boundary: &str,
) -> Result<String, CliError> {
    let subject = verification.subject.as_ref().ok_or_else(|| {
        CliError::InvalidRepository(
            "current verification is missing its assessment subject".to_string(),
        )
    })?;
    ensure_live_subject_matches_worktree(open_dir, subject, boundary)
}

pub(crate) fn ensure_live_subject_matches_worktree(
    open_dir: &Path,
    subject: &codefire_core::AssessmentSubject,
    boundary: &str,
) -> Result<String, CliError> {
    if subject.kind != "live_open" || subject.trust_kind != "local_unisolated" || subject.immutable
    {
        return Err(CliError::InvalidRepository(
            "local verification carries an incompatible assessment subject".to_string(),
        ));
    }
    let expected = subject.worktree_digest.as_deref().ok_or_else(|| {
        CliError::InvalidRepository(
            "local verification assessment subject is missing its worktree digest".to_string(),
        )
    })?;
    let actual = worktree_manifest_digest(open_dir)?;
    if actual != expected {
        return Err(CliError::InvalidRepository(format!(
            "assessment subject no longer matches the worktree {boundary}; rerun verification"
        )));
    }
    Ok(actual)
}

pub(crate) fn load_base_atom_index(
    objects: &Path,
    base_commit: &str,
) -> Result<codefire_core::AtomIndex, CliError> {
    let validated = codefire_store::validate_sealed_commit_pinned(objects, base_commit)?;
    load_validated_base_atom_index(&validated)
}

pub(crate) fn load_validated_base_atom_index(
    validated: &codefire_store::ValidatedSealedCommit,
) -> Result<codefire_core::AtomIndex, CliError> {
    let atom_index = validated.read_root("atom_index")?;
    Ok(serde_json::from_value(atom_index)?)
}
