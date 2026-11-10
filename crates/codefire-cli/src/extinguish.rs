//! Recording fire resolutions.

use crate::automation::ready_verify_details_action;
use crate::cli_error::CliError;
use crate::fire::active_ledger_transaction::{
    apply_active_ledger_transition, recover_active_ledger_transition,
    reject_pending_active_ledger_transition, ActiveLedgerWrite,
};
use crate::idempotency::{
    idempotency_payload_hash, idempotency_record_path, idempotency_result_plan,
    require_idempotency_key, verify_idempotency_record,
};
use crate::json_files::{read_active_json_optional, write_json_atomic};
use crate::open_context::open_context;
use crate::remote::read_optional_json;
use crate::repo_lock::acquire_locked_open_context_for_recovery;
use crate::scan_verify::{compute_scan, compute_scan_locked};
use crate::state_transaction::{
    ensure_active_ledger_state_transaction_request, set_open_state_transactional,
};
use crate::{
    now_iso_utc, required_string, scan_branch_state, stable_hash_48, ExtinguishOptions,
    ExtinguishResult, OpenContext,
};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn run_extinguish(options: &ExtinguishOptions) -> Result<ExtinguishResult, CliError> {
    let lock;
    let context;
    if options.dry_run {
        lock = None;
        context = open_context(&options.path)?;
    } else {
        let (acquired_lock, locked_context) =
            acquire_locked_open_context_for_recovery(&options.path, &options.lock, "extinguish")?;
        lock = Some(acquired_lock);
        context = locked_context;
    }
    let idempotency_payload = extinguish_idempotency_payload(options, &context);
    let ledger_request = (!options.dry_run)
        .then(|| extinguish_ledger_transition_request(&context, &idempotency_payload))
        .transpose()?;
    if let Some(request) = ledger_request.as_ref() {
        ensure_active_ledger_state_transaction_request(&context, "extinguish", request)?;
    }
    if options.dry_run {
        reject_pending_active_ledger_transition(&context.active_state)?;
    } else if let Some(durable) = recover_active_ledger_transition(
        &context.active_state,
        "extinguish",
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
    )? {
        let (result, target_state) = extinguish_result_from_ledger_transition(durable)?;
        set_open_state_transactional(
            &context,
            "extinguish",
            &target_state,
            ledger_request
                .as_ref()
                .expect("live extinguish must have a ledger request"),
        )?;
        if let Some(key) = options.idempotency_key.as_deref() {
            save_extinguish_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_extinguish_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let active_state_path = context.active_state.path().to_path_buf();
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    if options.resolution == "no-change-required"
        && policy.no_change_required_requires_rationale
        && options.rationale.is_empty()
    {
        return Err(CliError::Usage(
            "no-change-required requires --rationale".to_string(),
        ));
    }
    if options.rationale.is_empty()
        && options.evidence.is_empty()
        && options.evidence_refs.is_empty()
    {
        return Err(CliError::Usage(
            "extinguish requires --rationale, --evidence, or --evidence-ref".to_string(),
        ));
    }
    validate_evidence_refs(&context.repo_root, &options.evidence_refs)?;

    let scan_execution = if options.dry_run {
        compute_scan(&context.open_dir)?
    } else {
        compute_scan_locked(
            &context.open_dir,
            lock.as_ref().expect("live extinguish must hold repo lock"),
        )?
    };
    let scan = scan_execution.scan;
    let mut fires = scan_execution.fires;
    let fire_index = fires
        .iter()
        .position(|fire| fire.display_id == options.fire_id || fire.fire_uid == options.fire_id)
        .ok_or_else(|| CliError::Usage(format!("unknown fire: {}", options.fire_id)))?;
    if fires[fire_index].status != "open" && !options.refresh {
        return Err(CliError::Usage(format!(
            "fire is not open: {}",
            options.fire_id
        )));
    }

    let resolved_at = now_iso_utc();
    let resolution_uid = format!(
        "res_{:012x}",
        stable_hash_48(format!("{}:{resolved_at}", options.fire_id).as_bytes())
    );
    let resolution = codefire_core::build_resolution(
        &fires[fire_index],
        &scan.atom_index,
        &scan.trace_graph,
        policy,
        codefire_core::ResolutionRequest {
            resolution_uid: resolution_uid.clone(),
            resolution_type: options.resolution.clone(),
            rationale: options.rationale.clone(),
            evidence: options.evidence.clone(),
            evidence_refs: options.evidence_refs.clone(),
            resolved_at,
        },
    )?;
    let display_id = fires[fire_index].display_id.clone();
    let fire_uid = fires[fire_index].fire_uid.clone();
    let source_atom = fires[fire_index].source.atom_id.clone();
    let target_atom = fires[fire_index].target.atom_id.clone();
    let reason = fires[fire_index].reason.clone();
    let plan = extinguish_operation_plan(
        options,
        &context,
        &active_state_path,
        &fires[fire_index],
        &resolution_uid,
    );
    if options.dry_run {
        return Ok(ExtinguishResult {
            display_id,
            fire_uid,
            source_atom,
            target_atom,
            reason,
            remaining_open_fire_count: scan.open_fires.len(),
            plan,
        });
    }
    fires[fire_index].status = "extinguished".to_string();
    fires[fire_index].resolution_uid = Some(resolution_uid);

    let fires_before = read_active_json_optional(&context.active_state, Path::new("fires.json"))?;
    let resolutions_before =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?;
    let mut resolutions: Vec<codefire_core::Resolution> = resolutions_before
        .clone()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    if options.refresh {
        for existing in &mut resolutions {
            if existing.fire_uid == fires[fire_index].fire_uid && existing.status == "active" {
                existing.status = "superseded".to_string();
            }
        }
    }
    resolutions.push(resolution);
    let remaining_open_fires = fires.iter().filter(|fire| fire.status == "open").count();
    let target_state =
        scan_branch_state(scan.changed_atoms.len(), remaining_open_fires).to_string();
    let result = ExtinguishResult {
        display_id,
        fire_uid,
        source_atom,
        target_atom,
        reason,
        remaining_open_fire_count: remaining_open_fires,
        plan,
    };
    let durable_result = extinguish_ledger_transition_result(&result, &target_state);
    let durable_result = apply_active_ledger_transition(
        &context.active_state,
        "extinguish",
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
        vec![
            ActiveLedgerWrite::new("fires.json", fires_before, serde_json::to_value(&fires)?),
            ActiveLedgerWrite::new(
                "resolutions.json",
                resolutions_before,
                serde_json::to_value(&resolutions)?,
            ),
        ],
        &durable_result,
    )?;
    let (result, target_state) = extinguish_result_from_ledger_transition(durable_result)?;
    set_open_state_transactional(
        &context,
        "extinguish",
        &target_state,
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
    )?;
    if let Some(key) = options.idempotency_key.as_deref() {
        save_extinguish_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn extinguish_ledger_transition_request(
    context: &OpenContext,
    idempotency_payload: &Value,
) -> Result<Value, CliError> {
    Ok(json!({
        "type": "codefire_extinguish_ledger_transition_request",
        "version": 1,
        "branch": &context.branch,
        "open_instance_id": required_string(&context.registry, &["open", "open_instance_id"] )?,
        "current_base_commit": required_string(&context.registry, &["open", "current_base_commit"] )?,
        "request": idempotency_payload,
    }))
}

pub(crate) fn extinguish_ledger_transition_result(
    result: &ExtinguishResult,
    target_state: &str,
) -> Value {
    json!({
        "type": "codefire_extinguish_ledger_transition_result",
        "version": 1,
        "display_id": &result.display_id,
        "fire_uid": &result.fire_uid,
        "source_atom": &result.source_atom,
        "target_atom": &result.target_atom,
        "reason": &result.reason,
        "remaining_open_fire_count": result.remaining_open_fire_count,
        "target_state": target_state,
        "plan": &result.plan,
    })
}

pub(crate) fn extinguish_result_from_ledger_transition(
    value: Value,
) -> Result<(ExtinguishResult, String), CliError> {
    if value.get("type").and_then(Value::as_str)
        != Some("codefire_extinguish_ledger_transition_result")
        || value.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err(CliError::InvalidRepository(
            "active ledger extinguish transaction contains an invalid durable result".to_string(),
        ));
    }
    let required = |field: &str| -> Result<String, CliError> {
        value
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "active ledger extinguish durable result has invalid {field}"
                ))
            })
    };
    let remaining_open_fire_count = value
        .get("remaining_open_fire_count")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "active ledger extinguish durable result has invalid remaining_open_fire_count"
                    .to_string(),
            )
        })?;
    let target_state = required("target_state")?;
    if !matches!(
        target_state.as_str(),
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(CliError::InvalidRepository(format!(
            "active ledger extinguish durable result has unsupported target_state {target_state}"
        )));
    }
    let result = ExtinguishResult {
        display_id: required("display_id")?,
        fire_uid: required("fire_uid")?,
        source_atom: required("source_atom")?,
        target_atom: required("target_atom")?,
        reason: required("reason")?,
        remaining_open_fire_count,
        plan: value.get("plan").cloned().ok_or_else(|| {
            CliError::InvalidRepository(
                "active ledger extinguish durable result is missing plan".to_string(),
            )
        })?,
    };
    Ok((result, target_state))
}

pub(crate) fn extinguish_idempotency_payload(
    options: &ExtinguishOptions,
    context: &OpenContext,
) -> Value {
    json!({
        "command": "extinguish",
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "fire_id": &options.fire_id,
        "resolution": &options.resolution,
        "rationale": &options.rationale,
        "evidence": &options.evidence,
        "evidence_refs": &options.evidence_refs,
        "refresh": options.refresh,
    })
}

pub(crate) fn validate_evidence_refs(repo_root: &Path, refs: &[String]) -> Result<(), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    for evidence_id in refs {
        if evidence_id.trim().is_empty() {
            return Err(CliError::Usage(
                "--evidence-ref requires a non-empty evidence id".to_string(),
            ));
        }
        let payload = codefire_store::read_object(&objects, evidence_id).map_err(|error| {
            if matches!(error, codefire_store::StoreError::ObjectNotFound(_)) {
                CliError::MissingEvidenceRef {
                    evidence_id: evidence_id.clone(),
                    repo_root: repo_root.to_path_buf(),
                }
            } else {
                CliError::Store(Box::new(error))
            }
        })?;
        if payload.get("type").and_then(Value::as_str) != Some("evidence") {
            return Err(CliError::Usage(format!(
                "--evidence-ref must point to an evidence object: {evidence_id}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn collect_missing_evidence_refs(
    repo_root: &Path,
    resolutions: &[codefire_core::Resolution],
) -> Vec<codefire_core::MissingEvidenceRef> {
    let objects = repo_root.join(".codefire").join("objects");
    let mut missing = Vec::new();
    for resolution in resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
    {
        for evidence_id in &resolution.evidence_refs {
            let valid = codefire_store::read_object(&objects, evidence_id)
                .map(|payload| payload.get("type").and_then(Value::as_str) == Some("evidence"))
                .unwrap_or(false);
            if !valid {
                missing.push(codefire_core::MissingEvidenceRef {
                    resolution_uid: resolution.resolution_uid.clone(),
                    evidence_id: evidence_id.clone(),
                });
            }
        }
    }
    missing
}

pub(crate) fn load_extinguish_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<ExtinguishResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "extinguish", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "extinguish", key, payload)?;
    let plan = idempotency_result_plan(&record, &path)?;
    let fire_uid = record
        .pointer("/result/fire_uid")
        .and_then(Value::as_str)
        .or_else(|| plan.pointer("/fire/fire_uid").and_then(Value::as_str))
        .unwrap_or("(unknown)");
    Ok(Some(ExtinguishResult {
        display_id: required_string(&record, &["result", "display_id"])?,
        fire_uid: fire_uid.to_string(),
        source_atom: record
            .pointer("/result/source_atom")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/source_atom").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        target_atom: record
            .pointer("/result/target_atom")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/target_atom").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        reason: record
            .pointer("/result/reason")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/reason").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        remaining_open_fire_count: record
            .pointer("/result/remaining_open_fire_count")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or_default(),
        plan,
    }))
}

pub(crate) fn save_extinguish_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &ExtinguishResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "extinguish", key);
    let record = json!({
        "version": 1,
        "command": "extinguish",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "display_id": &result.display_id,
            "fire_uid": &result.fire_uid,
            "source_atom": &result.source_atom,
            "target_atom": &result.target_atom,
            "reason": &result.reason,
            "remaining_open_fire_count": result.remaining_open_fire_count,
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

pub(crate) fn extinguish_operation_plan(
    options: &ExtinguishOptions,
    context: &OpenContext,
    active_state_path: &Path,
    fire: &codefire_core::Fire,
    resolution_uid: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "extinguish",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "fire": {
            "display_id": &fire.display_id,
            "fire_uid": &fire.fire_uid,
            "source_atom": &fire.source.atom_id,
            "target_atom": &fire.target.atom_id,
            "reason": &fire.reason,
            "status": &fire.status,
        },
        "resolution": {
            "resolution_uid": resolution_uid,
            "resolution_type": &options.resolution,
            "refresh": options.refresh,
            "has_rationale": !options.rationale.is_empty(),
            "has_evidence": !options.evidence.is_empty() || !options.evidence_refs.is_empty(),
            "evidence_refs": &options.evidence_refs,
        },
        "operations": [
            {"kind": "update_fire_status", "path": active_state_path.join("fires.json"), "status": "extinguished"},
            {"kind": "append_resolution", "path": active_state_path.join("resolutions.json"), "resolution_uid": resolution_uid},
        ],
        "next_actions": [
            ready_verify_details_action(
                &context.branch,
                &context.open_dir,
                Some(&context.repo_root),
            ),
        ],
    })
}
