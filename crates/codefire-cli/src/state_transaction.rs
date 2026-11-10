//! Journaled publication of open-directory state for scan, verify, and the active ledger.

use crate::cli_error::CliError;
use crate::fire::active_ledger_transaction::{
    active_ledger_hash_is_valid, active_ledger_request_hash,
    ensure_completed_active_ledger_transition, release_completed_active_ledger_transition,
    validate_active_ledger_descriptor, ActiveLedgerPublicationStatus,
};
use crate::json_files::{
    read_active_json_optional, read_active_json_required, write_active_json_atomic,
    write_active_json_atomic_bounded, write_json_atomic,
};
use crate::scan_verify::verification_open_state;
use crate::test_hooks::maybe_fail_state_transaction_for_test;
use crate::{
    active_state, load_branch_record, now_iso_utc, read_json, required_string, save_branch_record,
    scan_branch_state, scan_change_count, OpenContext,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(crate) fn state_transaction_path(active_state: &active_state::ActiveStateDir) -> PathBuf {
    active_state.display_path(Path::new("state_transaction.json"))
}

pub(crate) const STATE_TRANSACTION_BYTE_LIMIT: u64 = 1024 * 1024;

pub(crate) fn begin_state_transaction(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
) -> Result<(), CliError> {
    begin_state_transaction_with_request_hash(context, operation, intended_state, None)
}

pub(crate) fn begin_state_transaction_with_request_hash(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
    request_hash: Option<&str>,
) -> Result<(), CliError> {
    let active_state = &context.active_state;
    if state_transaction_phases(operation).is_none() {
        return Err(CliError::InvalidRepository(format!(
            "unsupported state transaction operation: {operation}"
        )));
    }
    match (active_ledger_state_operation(operation), request_hash) {
        (true, None) => {
            return Err(CliError::InvalidRepository(format!(
            "active-ledger state transaction operation {operation} requires an exact request hash"
        )))
        }
        (false, Some(_)) => {
            return Err(CliError::InvalidRepository(format!(
                "state transaction operation {operation} cannot carry an active-ledger request hash"
            )))
        }
        (_, Some(hash)) if !active_ledger_hash_is_valid(hash) => {
            return Err(CliError::InvalidRepository(format!(
                "active-ledger state transaction operation {operation} has an invalid request hash"
            )))
        }
        _ => {}
    }
    if !matches!(
        intended_state,
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported state transaction target: {intended_state}"
        )));
    }
    let current_registry = read_json(&context.registry_path)?;
    let open_instance_id = required_string(&current_registry, &["open", "open_instance_id"])?;
    let current_base_commit = required_string(&current_registry, &["open", "current_base_commit"])?;
    if active_state.exists(Path::new("state_transaction.json"))? {
        validate_pending_state_transaction_for_recovery(active_state)?;
        let pending = read_state_transaction(active_state)?;
        if pending.get("version").and_then(Value::as_u64) != Some(2)
            || required_string(&pending, &["operation"])? != operation
            || required_string(&pending, &["intended_state"])? != intended_state
            || required_string(&pending, &["subject", "branch"])? != context.branch
            || required_string(&pending, &["subject", "open_instance_id"])? != open_instance_id
            || required_string(&pending, &["subject", "current_base_commit"])?
                != current_base_commit
            || pending.get("request_hash").and_then(Value::as_str) != request_hash
        {
            return Err(CliError::InvalidRepository(format!(
                "pending state transaction does not match the exact {operation} retry subject: {}",
                state_transaction_path(active_state).display()
            )));
        }
        return Ok(());
    }
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    let active = read_active_json_required(active_state, Path::new("state.json"))?;
    let branch_state = required_string(&branch, &["state"])?;
    let registry_state = required_string(&current_registry, &["state", "last_known"])?;
    let active_state_value = required_string(&active, &["state"])?;
    if branch_state != registry_state || branch_state != active_state_value {
        return Err(CliError::InvalidRepository(
            "cannot begin a state transaction from divergent stable authorities".to_string(),
        ));
    }
    let pre_state = branch_state;
    let now = now_iso_utc();
    let mut transaction = json!({
        "version": 2,
        "status": "pending",
        "operation": operation,
        "phase": "started",
        "intended_state": intended_state,
        "subject": {
            "branch": &context.branch,
            "open_instance_id": open_instance_id,
            "current_base_commit": current_base_commit,
        },
        "pre_state": {
            "branch": &pre_state,
            "registry": &pre_state,
            "active": &pre_state,
        },
        "started_at": now,
        "updated_at": now,
    });
    if let Some(request_hash) = request_hash {
        transaction["request_hash"] = Value::String(request_hash.to_string());
    }
    write_active_json_atomic_bounded(
        active_state,
        Path::new("state_transaction.json"),
        &transaction,
        STATE_TRANSACTION_BYTE_LIMIT,
        "state transaction marker",
    )
}

pub(crate) fn validate_pending_state_transaction_for_recovery(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    let path = state_transaction_path(active_state);
    let transaction = read_state_transaction(active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))
}

pub(crate) fn read_state_transaction(
    active_state: &active_state::ActiveStateDir,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(
        Path::new("state_transaction.json"),
        STATE_TRANSACTION_BYTE_LIMIT,
    )?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) const SCAN_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "verification_invalidated",
    "fires_written",
    "scan_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];
pub(crate) const VERIFY_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "verification_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];
pub(crate) const ACTIVE_LEDGER_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "registry_written",
    "active_state_written",
    "branch_written",
];

pub(crate) fn active_ledger_state_operation(operation: &str) -> bool {
    matches!(
        operation,
        "fire" | "fire-batch" | "extinguish" | "extinguish-batch"
    )
}

pub(crate) fn state_transaction_phases(operation: &str) -> Option<&'static [&'static str]> {
    match operation {
        "scan" => Some(SCAN_STATE_TRANSACTION_PHASES),
        "verify" => Some(VERIFY_STATE_TRANSACTION_PHASES),
        operation if active_ledger_state_operation(operation) => {
            Some(ACTIVE_LEDGER_STATE_TRANSACTION_PHASES)
        }
        _ => None,
    }
}

pub(crate) fn state_transaction_phase_is_supported(operation: &str, phase: &str) -> bool {
    state_transaction_phases(operation).is_some_and(|phases| phases.contains(&phase))
}

pub(crate) fn state_transaction_transition_is_supported(
    operation: &str,
    current_phase: &str,
    next_phase: &str,
) -> bool {
    state_transaction_phases(operation).is_some_and(|phases| {
        phases
            .windows(2)
            .any(|transition| transition[0] == current_phase && transition[1] == next_phase)
    })
}

pub(crate) fn validate_state_transaction_record(transaction: &Value) -> Result<(), String> {
    let version = transaction
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "state_transaction.json version must be 1 or 2".to_string())?;
    if !matches!(version, 1 | 2) {
        return Err("state_transaction.json version must be 1 or 2".to_string());
    }
    for field in [
        "status",
        "operation",
        "phase",
        "intended_state",
        "started_at",
        "updated_at",
    ] {
        if transaction.get(field).and_then(Value::as_str).is_none() {
            return Err(format!(
                "state_transaction.json must contain string field {field}"
            ));
        }
    }
    if transaction.get("status").and_then(Value::as_str) != Some("pending") {
        return Err("state_transaction.json status must be pending".to_string());
    }
    let operation = transaction["operation"]
        .as_str()
        .expect("operation string was checked above");
    if state_transaction_phases(operation).is_none() {
        return Err(format!(
            "state_transaction.json contains unsupported operation: {operation}"
        ));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("phase string was checked above");
    if !state_transaction_phase_is_supported(operation, phase) {
        return Err(format!(
            "state_transaction.json phase {phase} is unsupported for operation {operation}"
        ));
    }
    let intended_state = transaction["intended_state"]
        .as_str()
        .expect("intended_state string was checked above");
    if !matches!(
        intended_state,
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(format!(
            "state_transaction.json contains unsupported intended_state: {intended_state}"
        ));
    }
    if version == 2 {
        for field in [
            "/subject/branch",
            "/subject/open_instance_id",
            "/subject/current_base_commit",
            "/pre_state/branch",
            "/pre_state/registry",
            "/pre_state/active",
        ] {
            if transaction.pointer(field).and_then(Value::as_str).is_none() {
                return Err(format!(
                    "state_transaction.json version 2 must contain string field {field}"
                ));
            }
        }
        let pre_branch = transaction
            .pointer("/pre_state/branch")
            .and_then(Value::as_str)
            .expect("version 2 pre-state branch was checked");
        let pre_registry = transaction
            .pointer("/pre_state/registry")
            .and_then(Value::as_str)
            .expect("version 2 pre-state registry was checked");
        let pre_active = transaction
            .pointer("/pre_state/active")
            .and_then(Value::as_str)
            .expect("version 2 pre-state active was checked");
        if pre_branch != pre_registry || pre_branch != pre_active {
            return Err(
                "state_transaction.json version 2 pre-state authorities must agree".to_string(),
            );
        }
        if !matches!(
            pre_branch,
            "open-clean" | "open-burning" | "open-consistent"
        ) {
            return Err(format!(
                "state_transaction.json contains unsupported pre-state: {pre_branch}"
            ));
        }
        if active_ledger_state_operation(operation) {
            let request_hash = transaction
                .get("request_hash")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "state_transaction.json version 2 operation {operation} must contain string field request_hash"
                    )
                })?;
            if !active_ledger_hash_is_valid(request_hash) {
                return Err(format!(
                    "state_transaction.json version 2 operation {operation} contains invalid request_hash"
                ));
            }
        } else if transaction.get("request_hash").is_some() {
            return Err(format!(
                "state_transaction.json version 2 operation {operation} cannot contain request_hash"
            ));
        }
    }
    Ok(())
}

pub(crate) fn invalidate_active_verification(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    active_state.remove_file_if_exists(Path::new("verification.json"))?;
    Ok(())
}

pub(crate) fn update_state_transaction_phase(
    active_state: &active_state::ActiveStateDir,
    phase: &str,
) -> Result<(), CliError> {
    let path = state_transaction_path(active_state);
    let mut transaction = read_state_transaction(active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    let operation = transaction["operation"]
        .as_str()
        .expect("validated state transaction operation")
        .to_string();
    let current_phase = transaction["phase"]
        .as_str()
        .expect("validated state transaction phase")
        .to_string();
    let phases = state_transaction_phases(&operation)
        .expect("validated state transaction operation must have phases");
    let current_index = phases
        .iter()
        .position(|candidate| *candidate == current_phase)
        .expect("validated current state transaction phase must be supported");
    let target_index = phases
        .iter()
        .position(|candidate| *candidate == phase)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "state transaction operation {operation} cannot advance from {current_phase} to {phase}: {}",
                path.display()
            ))
        })?;
    if target_index <= current_index {
        return Ok(());
    }
    if target_index != current_index + 1
        || !state_transaction_transition_is_supported(&operation, &current_phase, phase)
    {
        return Err(CliError::InvalidRepository(format!(
            "state transaction operation {operation} cannot advance from {current_phase} to {phase}: {}",
            path.display()
        )));
    }
    transaction["phase"] = Value::String(phase.to_string());
    transaction["updated_at"] = Value::String(now_iso_utc());
    write_active_json_atomic_bounded(
        active_state,
        Path::new("state_transaction.json"),
        &transaction,
        STATE_TRANSACTION_BYTE_LIMIT,
        "state transaction marker",
    )
}

pub(crate) enum StateTransactionAuthority<'a> {
    Scan {
        scan: &'a codefire_core::ScanResult,
        fires: &'a [codefire_core::Fire],
    },
    Verify {
        scan: &'a codefire_core::ScanResult,
        fires: &'a [codefire_core::Fire],
        verification: &'a codefire_core::Verification,
    },
}

impl StateTransactionAuthority<'_> {
    pub(crate) fn operation(&self) -> &'static str {
        match self {
            Self::Scan { .. } => "scan",
            Self::Verify { .. } => "verify",
        }
    }

    pub(crate) fn scan(&self) -> &codefire_core::ScanResult {
        match self {
            Self::Scan { scan, .. } | Self::Verify { scan, .. } => scan,
        }
    }

    pub(crate) fn fires(&self) -> &[codefire_core::Fire] {
        match self {
            Self::Scan { fires, .. } | Self::Verify { fires, .. } => fires,
        }
    }
}

pub(crate) fn persist_transactional_open_state(
    context: &OpenContext,
    operation: &str,
    state: &str,
    validated_base: &codefire_store::ValidatedSealedCommit,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    if authority.operation() != operation {
        return Err(CliError::InvalidRepository(format!(
            "state transaction authority operation {} does not match {operation}",
            authority.operation()
        )));
    }
    publish_transactional_open_state_fields(context, operation, state)?;
    ensure_validated_store_subject(context, validated_base)?;
    complete_state_transaction(context, state, authority)?;
    ensure_validated_store_subject(context, validated_base)
}

pub(crate) fn publish_transactional_open_state_fields(
    context: &OpenContext,
    operation: &str,
    state: &str,
) -> Result<(), CliError> {
    let mut registry = read_json(&context.registry_path)?;
    registry["state"]["last_known"] = Value::String(state.to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "after_registry_write_before_phase",
    )?;
    update_state_transaction_phase(&context.active_state, "registry_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "registry_written",
    )?;

    let mut active_state_record =
        read_active_json_optional(&context.active_state, Path::new("state.json"))?
            .unwrap_or_else(|| json!({}));
    active_state_record["state"] = Value::String(state.to_string());
    write_active_json_atomic(
        &context.active_state,
        Path::new("state.json"),
        &active_state_record,
    )?;
    update_state_transaction_phase(&context.active_state, "active_state_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "active_state_written",
    )?;

    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["state"] = Value::String(state.to_string());
    save_branch_record(&context.repo_root, &branch)?;
    update_state_transaction_phase(&context.active_state, "branch_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "branch_written",
    )?;
    Ok(())
}

pub(crate) fn ensure_validated_store_subject(
    context: &OpenContext,
    validated: &codefire_store::ValidatedSealedCommit,
) -> Result<(), CliError> {
    validated
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: validated.objects_root().to_path_buf(),
        })
}

pub(crate) fn complete_state_transaction(
    context: &OpenContext,
    intended_state: &str,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    let transaction_path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction).map_err(|message| {
        CliError::InvalidRepository(format!("{message}: {}", transaction_path.display()))
    })?;
    let transaction_operation = required_string(&transaction, &["operation"])?;
    if transaction_operation != authority.operation() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction expected authority operation {}, but marker contains {transaction_operation}: {}",
            authority.operation(),
            transaction_path.display()
        )));
    }
    let transaction_phase = required_string(&transaction, &["phase"])?;
    if transaction_phase != "branch_written" {
        return Err(CliError::InvalidRepository(format!(
            "state transaction cannot complete from phase {transaction_phase}: {}",
            transaction_path.display()
        )));
    }
    let marker_state = required_string(&transaction, &["intended_state"])?;
    if marker_state != intended_state {
        return Err(CliError::InvalidRepository(format!(
            "state transaction expected {intended_state}, but marker contains {marker_state}: {}",
            transaction_path.display()
        )));
    }

    let registry = read_json(&context.registry_path)?;
    let active_state_record =
        read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    for (subject, actual) in [
        (
            "open registry",
            required_string(&registry, &["state", "last_known"])?,
        ),
        (
            "active state",
            required_string(&active_state_record, &["state"])?,
        ),
        ("branch", required_string(&branch, &["state"])?),
    ] {
        if actual != intended_state {
            return Err(CliError::InvalidRepository(format!(
                "state transaction expected {intended_state}, but {subject} contains {actual}"
            )));
        }
    }
    validate_state_transaction_authorities(context, intended_state, &registry, &branch, authority)?;
    context
        .active_state
        .remove_file_if_exists(Path::new("state_transaction.json"))?;
    Ok(())
}

pub(crate) fn validate_state_transaction_authorities(
    context: &OpenContext,
    intended_state: &str,
    registry: &Value,
    branch: &Value,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    let persisted_fires: Vec<codefire_core::Fire> = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("fires.json"))?,
    )?;
    if persisted_fires != authority.fires() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction persisted fires differ from the computed authority: {}",
            context
                .active_state
                .display_path(Path::new("fires.json"))
                .display()
        )));
    }

    let persisted_scan: codefire_core::ScanResult = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("scan.json"))?,
    )?;
    if &persisted_scan != authority.scan() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction persisted scan differs from the computed authority: {}",
            context
                .active_state
                .display_path(Path::new("scan.json"))
                .display()
        )));
    }
    let open_fires = persisted_fires
        .iter()
        .filter(|fire| fire.status == "open")
        .cloned()
        .collect::<Vec<_>>();
    if persisted_scan.open_fires != open_fires {
        return Err(CliError::InvalidRepository(
            "state transaction scan.open_fires differs from persisted fires".to_string(),
        ));
    }

    let registry_base = required_string(registry, &["open", "current_base_commit"])?;
    let branch_head = required_string(branch, &["head"])?;
    if persisted_scan.base_commit != registry_base || persisted_scan.base_commit != branch_head {
        return Err(CliError::InvalidRepository(format!(
            "state transaction scan base {}, registry base {registry_base}, and branch head {branch_head} disagree",
            persisted_scan.base_commit
        )));
    }

    let derived_state = match authority {
        StateTransactionAuthority::Scan { .. } => {
            if read_active_json_optional(&context.active_state, Path::new("verification.json"))?
                .is_some()
            {
                return Err(CliError::InvalidRepository(
                    "state transaction scan completed with a stale verification authority"
                        .to_string(),
                ));
            }
            scan_branch_state(
                scan_change_count(&persisted_scan),
                persisted_scan.open_fires.len(),
            )
        }
        StateTransactionAuthority::Verify { verification, .. } => {
            let persisted_verification: codefire_core::Verification = serde_json::from_value(
                read_active_json_required(&context.active_state, Path::new("verification.json"))?,
            )?;
            if &persisted_verification != verification {
                return Err(CliError::InvalidRepository(format!(
                    "state transaction persisted verification differs from the computed authority: {}",
                    context
                        .active_state
                        .display_path(Path::new("verification.json"))
                        .display()
                )));
            }
            let open_required_fires = persisted_scan
                .open_fires
                .iter()
                .filter(|fire| fire.status == "open" && fire.severity == "required")
                .count();
            if persisted_verification.open_required_fires != open_required_fires
                || persisted_verification.duplicate_atom_ids
                    != persisted_scan.atom_index.duplicate_atom_ids
            {
                return Err(CliError::InvalidRepository(
                    "state transaction verification summary differs from the persisted scan"
                        .to_string(),
                ));
            }
            verification_open_state(&persisted_verification, &persisted_scan)
        }
    };
    if derived_state != intended_state {
        return Err(CliError::InvalidRepository(format!(
            "state transaction derived {derived_state} from persisted authorities, expected {intended_state}"
        )));
    }
    Ok(())
}

pub(crate) fn ensure_active_ledger_state_transaction_request(
    context: &OpenContext,
    operation: &str,
    request: &Value,
) -> Result<(), CliError> {
    if !active_ledger_state_operation(operation) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported active-ledger state transaction operation: {operation}"
        )));
    }
    if !context
        .active_state
        .exists(Path::new("state_transaction.json"))?
    {
        return Ok(());
    }
    let path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    let request_hash = active_ledger_request_hash(request)?;
    if transaction.get("version").and_then(Value::as_u64) != Some(2)
        || required_string(&transaction, &["operation"])? != operation
        || required_string(&transaction, &["request_hash"])? != request_hash
    {
        return Err(CliError::IdempotencyConflict(format!(
            "pending state transaction does not belong to the exact {operation}/{request_hash} request: {}",
            path.display()
        )));
    }
    Ok(())
}

pub(crate) fn set_open_state_transactional(
    context: &OpenContext,
    operation: &str,
    state: &str,
    request: &Value,
) -> Result<(), CliError> {
    if !active_ledger_state_operation(operation) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported active-ledger state transaction operation: {operation}"
        )));
    }
    ensure_active_ledger_state_transaction_request(context, operation, request)?;
    ensure_completed_active_ledger_transition(&context.active_state, operation, request)?;
    let request_hash = active_ledger_request_hash(request)?;
    let has_state_transaction = context
        .active_state
        .exists(Path::new("state_transaction.json"))?;
    let descriptor = validate_active_ledger_descriptor(&context.active_state)?.ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "active-ledger state publication lost its completed receipt for {operation}/{request_hash}"
        ))
    })?;
    if !has_state_transaction
        && descriptor.publication_status == ActiveLedgerPublicationStatus::Released
    {
        let registry = read_json(&context.registry_path)?;
        let active = read_active_json_required(&context.active_state, Path::new("state.json"))?;
        let branch = load_branch_record(&context.repo_root, &context.branch)?;
        for (authority, actual) in [
            (
                "registry",
                required_string(&registry, &["state", "last_known"])?,
            ),
            ("active", required_string(&active, &["state"])?),
            ("branch", required_string(&branch, &["state"])?),
        ] {
            if actual != state {
                return Err(CliError::InvalidRepository(format!(
                    "released active-ledger receipt {operation}/{request_hash} expected {state}, but {authority} contains {actual}"
                )));
            }
        }
        return Ok(());
    }
    begin_state_transaction_with_request_hash(context, operation, state, Some(&request_hash))?;
    maybe_fail_state_transaction_for_test(context.active_state.path(), operation, "started")?;
    publish_transactional_open_state_fields(context, operation, state)?;
    complete_active_ledger_state_transaction(context, operation, state, &request_hash)
}

pub(crate) fn complete_active_ledger_state_transaction(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
    request_hash: &str,
) -> Result<(), CliError> {
    let path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if required_string(&transaction, &["operation"])? != operation
        || required_string(&transaction, &["phase"])? != "branch_written"
        || required_string(&transaction, &["intended_state"])? != intended_state
        || required_string(&transaction, &["request_hash"])? != request_hash
    {
        return Err(CliError::InvalidRepository(format!(
            "active-ledger state transaction does not match completed {operation}/{intended_state}/{request_hash} publication: {}",
            path.display()
        )));
    }
    let registry = read_json(&context.registry_path)?;
    let active = read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    for (authority, actual) in [
        (
            "registry",
            required_string(&registry, &["state", "last_known"])?,
        ),
        ("active", required_string(&active, &["state"])?),
        ("branch", required_string(&branch, &["state"])?),
    ] {
        if actual != intended_state {
            return Err(CliError::InvalidRepository(format!(
                "active-ledger state transaction expected {intended_state}, but {authority} contains {actual}"
            )));
        }
    }
    release_completed_active_ledger_transition(&context.active_state, operation, request_hash)?;
    context
        .active_state
        .remove_file_if_exists(Path::new("state_transaction.json"))?;
    Ok(())
}
