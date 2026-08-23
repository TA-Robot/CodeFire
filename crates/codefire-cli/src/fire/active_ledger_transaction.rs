use crate::{active_state::ActiveStateDir, now_iso_utc, CliError, REPOSITORY_JSON_BYTE_LIMIT};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) const ACTIVE_LEDGER_TRANSACTION_FILE: &str = "active_ledger_transaction.json";
const TRANSACTION_TYPE: &str = "codefire_active_ledger_transaction";
const TRANSACTION_VERSION: u64 = 1;
const MAX_WRITES: usize = 2;
pub(crate) const ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT: u64 = REPOSITORY_JSON_BYTE_LIMIT;

const SUPPORTED_OPERATIONS: &[&str] = &["fire", "fire-batch", "extinguish", "extinguish-batch"];
const SUPPORTED_LEDGER_FILES: &[&str] = &["fires.json", "resolutions.json"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveLedgerPublicationStatus {
    LedgerPending,
    StatePublicationPending,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedActiveLedgerDescriptor {
    pub(crate) operation: String,
    pub(crate) phase: String,
    pub(crate) request_hash: String,
    pub(crate) publication_status: ActiveLedgerPublicationStatus,
}

#[derive(Debug)]
pub(crate) struct ActiveLedgerWrite {
    path: &'static str,
    before: Option<Value>,
    after: Value,
}

impl ActiveLedgerWrite {
    pub(crate) fn new(path: &'static str, before: Option<Value>, after: Value) -> Self {
        Self {
            path,
            before,
            after,
        }
    }
}

pub(crate) fn active_ledger_request_hash(request: &Value) -> Result<String, CliError> {
    value_hash(request)
}

pub(crate) fn active_ledger_hash_is_valid(hash: &str) -> bool {
    valid_hash(hash)
}

pub(crate) fn validate_active_ledger_descriptor(
    active_state: &ActiveStateDir,
) -> Result<Option<ValidatedActiveLedgerDescriptor>, CliError> {
    let Some(record) = read_transaction(active_state)? else {
        return Ok(None);
    };
    let transaction = validate_transaction(record, active_state)?;
    let publication_status = match transaction.status {
        TransactionStatus::Pending => {
            validate_recovery_images(active_state, &transaction)?;
            ActiveLedgerPublicationStatus::LedgerPending
        }
        TransactionStatus::Complete => {
            if transaction.state_publication_pending {
                validate_complete_effects(active_state, &transaction)?;
                ActiveLedgerPublicationStatus::StatePublicationPending
            } else {
                ActiveLedgerPublicationStatus::Released
            }
        }
    };
    let phase = transaction
        .record
        .get("phase")
        .and_then(Value::as_str)
        .expect("validated active ledger transaction phase")
        .to_string();
    Ok(Some(ValidatedActiveLedgerDescriptor {
        operation: transaction.operation,
        phase,
        request_hash: transaction.request_hash,
        publication_status,
    }))
}

/// Proves that the ledger postimages consumed by the following state
/// publication belong to this exact request. This check is read-only: a
/// pending ledger transaction must be recovered by its owner before state
/// publication can begin.
pub(crate) fn ensure_completed_active_ledger_transition(
    active_state: &ActiveStateDir,
    operation: &str,
    request: &Value,
) -> Result<(), CliError> {
    validate_operation(operation)?;
    let request_hash = value_hash(request)?;
    let Some(record) = read_transaction(active_state)? else {
        return Err(CliError::InvalidRepository(format!(
            "active-ledger state publication for {operation}/{request_hash} requires a completed ledger transaction at {}",
            transaction_path(active_state).display()
        )));
    };
    let transaction = validate_transaction(record, active_state)?;
    if transaction.operation != operation || transaction.request_hash != request_hash {
        return Err(CliError::IdempotencyConflict(format!(
            "active ledger transaction {} ({}) does not own state publication for {operation}/{request_hash} at {}",
            transaction.request_hash,
            transaction.operation,
            transaction_path(active_state).display()
        )));
    }
    if transaction.status != TransactionStatus::Complete {
        return Err(CliError::IdempotencyConflict(format!(
            "active ledger transaction {request_hash} ({operation}) is not complete at {}",
            transaction_path(active_state).display()
        )));
    }
    validate_complete_effects(active_state, &transaction)
}

pub(crate) fn release_completed_active_ledger_transition(
    active_state: &ActiveStateDir,
    operation: &str,
    request_hash: &str,
) -> Result<(), CliError> {
    validate_operation(operation)?;
    if !valid_hash(request_hash) {
        return Err(CliError::InvalidRepository(format!(
            "cannot release active-ledger state publication for invalid request hash: {}",
            transaction_path(active_state).display()
        )));
    }
    let Some(record) = read_transaction(active_state)? else {
        return Err(CliError::InvalidRepository(format!(
            "cannot release missing active-ledger state publication for {operation}/{request_hash}: {}",
            transaction_path(active_state).display()
        )));
    };
    let mut transaction = validate_transaction(record, active_state)?;
    if transaction.status != TransactionStatus::Complete
        || transaction.operation != operation
        || transaction.request_hash != request_hash
    {
        return Err(CliError::IdempotencyConflict(format!(
            "active ledger transaction {} ({}) cannot release state publication for {operation}/{request_hash} at {}",
            transaction.request_hash,
            transaction.operation,
            transaction_path(active_state).display()
        )));
    }
    validate_complete_effects(active_state, &transaction)?;
    if !transaction.state_publication_pending {
        return Ok(());
    }
    transaction.record["state_publication"] = Value::String("released".to_string());
    transaction.record["state_publication_completed_at"] = Value::String(now_iso_utc());
    write_transaction(active_state, &transaction.record)
}

/// Completes a matching pending transition or replays its durable result.
///
/// A pending transition for another request is deliberately not adopted: the
/// caller must retry the request that owns the staged postimages first.
pub(crate) fn recover_active_ledger_transition(
    active_state: &ActiveStateDir,
    operation: &str,
    request: &Value,
) -> Result<Option<Value>, CliError> {
    validate_operation(operation)?;
    let request_hash = value_hash(request)?;
    let Some(record) = read_transaction(active_state)? else {
        return Ok(None);
    };
    let transaction = validate_transaction(record, active_state)?;
    if transaction.operation != operation || transaction.request_hash != request_hash {
        if transaction.status == TransactionStatus::Pending || transaction.state_publication_pending
        {
            return Err(CliError::IdempotencyConflict(format!(
                "active ledger transaction {} is pending ledger/state publication for a different request at {}",
                transaction.request_hash,
                transaction_path(active_state).display()
            )));
        }
        return Ok(None);
    }
    match transaction.status {
        TransactionStatus::Complete => {
            validate_complete_effects(active_state, &transaction)?;
            Ok(Some(transaction.result))
        }
        TransactionStatus::Pending => {
            let result = drive_pending_transaction(active_state, transaction)?;
            Ok(Some(result))
        }
    }
}

/// Read-only guard for previews and unrelated mutations.
///
/// A caller that cannot prove it owns the pending request must not publish or
/// auto-recover its postimages. A complete receipt remains blocking until its
/// owner publishes the corresponding branch/registry/active state.
pub(crate) fn reject_pending_active_ledger_transition(
    active_state: &ActiveStateDir,
) -> Result<(), CliError> {
    let Some(descriptor) = validate_active_ledger_descriptor(active_state)? else {
        return Ok(());
    };
    if descriptor.publication_status != ActiveLedgerPublicationStatus::Released {
        return Err(CliError::IdempotencyConflict(format!(
            "active ledger transaction {} ({}) has pending ledger/state publication at {}",
            descriptor.request_hash,
            descriptor.operation,
            transaction_path(active_state).display()
        )));
    }
    Ok(())
}

/// Stages and publishes a bounded Fire/Resolution ledger transition.
///
/// The returned value is always the authoritative durable result. It may come
/// from an exact replay when a prior attempt already prepared this request.
pub(crate) fn apply_active_ledger_transition(
    active_state: &ActiveStateDir,
    operation: &str,
    request: &Value,
    writes: Vec<ActiveLedgerWrite>,
    result: &Value,
) -> Result<Value, CliError> {
    if let Some(replayed) = recover_active_ledger_transition(active_state, operation, request)? {
        return Ok(replayed);
    }
    validate_operation(operation)?;
    validate_write_plan(active_state, &writes)?;
    validate_result_size(result, active_state)?;
    maybe_fail(active_state, "before_prepare")?;

    let request_hash = value_hash(request)?;
    let now = now_iso_utc();
    let write_records = writes
        .into_iter()
        .map(|write| {
            Ok(json!({
                "path": write.path,
                "before_present": write.before.is_some(),
                "before_hash": write.before.as_ref().map(value_hash).transpose()?,
                "after_hash": value_hash(&write.after)?,
                "after": write.after,
            }))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    let pending = json!({
        "type": TRANSACTION_TYPE,
        "version": TRANSACTION_VERSION,
        "status": "pending",
        "operation": operation,
        "phase": "prepared",
        "published_count": 0,
        "request_hash": request_hash,
        "result_hash": value_hash(result)?,
        "result": result,
        "writes": write_records,
        "started_at": now,
        "updated_at": now,
    });

    validate_expected_preimages(active_state, &pending)?;
    write_new_or_replace_completed(active_state, &pending)?;
    maybe_fail(active_state, "after_prepare")?;
    let transaction = validate_transaction(pending, active_state)?;
    drive_pending_transaction(active_state, transaction)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionStatus {
    Pending,
    Complete,
}

#[derive(Debug)]
struct ValidatedTransaction {
    record: Value,
    status: TransactionStatus,
    state_publication_pending: bool,
    operation: String,
    request_hash: String,
    result: Value,
    published_count: usize,
    writes: Vec<ValidatedWrite>,
    effects: Vec<ValidatedEffect>,
}

#[derive(Debug)]
struct ValidatedWrite {
    path: String,
    before_hash: Option<String>,
    after_hash: String,
    after: Value,
}

#[derive(Debug)]
struct ValidatedEffect {
    path: String,
    after_hash: String,
}

fn validate_operation(operation: &str) -> Result<(), CliError> {
    if SUPPORTED_OPERATIONS.contains(&operation) {
        Ok(())
    } else {
        Err(CliError::InvalidRepository(format!(
            "unsupported active ledger transaction operation: {operation}"
        )))
    }
}

fn validate_write_plan(
    active_state: &ActiveStateDir,
    writes: &[ActiveLedgerWrite],
) -> Result<(), CliError> {
    if writes.is_empty() || writes.len() > MAX_WRITES {
        return Err(CliError::InvalidRepository(format!(
            "active ledger transaction must contain between 1 and {MAX_WRITES} writes: {}",
            transaction_path(active_state).display()
        )));
    }
    let mut paths = BTreeSet::new();
    for write in writes {
        if !SUPPORTED_LEDGER_FILES.contains(&write.path) || !paths.insert(write.path) {
            return Err(CliError::InvalidRepository(format!(
                "active ledger transaction contains unsupported or duplicate path {}: {}",
                write.path,
                transaction_path(active_state).display()
            )));
        }
        render_ledger_value(&write.after, active_state, write.path)?;
        let actual = read_ledger_value(active_state, write.path)?;
        if actual != write.before {
            return Err(divergence_error(
                active_state,
                write.path,
                write.before.as_ref().map(value_hash).transpose()?,
                actual.as_ref().map(value_hash).transpose()?,
            ));
        }
        if write.before.as_ref().map(value_hash).transpose()? == Some(value_hash(&write.after)?) {
            return Err(CliError::InvalidRepository(format!(
                "active ledger transaction write has identical preimage and postimage for {}: {}",
                write.path,
                transaction_path(active_state).display()
            )));
        }
    }
    Ok(())
}

fn validate_result_size(result: &Value, active_state: &ActiveStateDir) -> Result<(), CliError> {
    let rendered = codefire_store::canonical_json(result)?;
    if rendered.len() as u64 > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "active ledger transaction result exceeds the {}-byte limit at {}: {} bytes",
            REPOSITORY_JSON_BYTE_LIMIT,
            transaction_path(active_state).display(),
            rendered.len()
        )));
    }
    Ok(())
}

fn write_new_or_replace_completed(
    active_state: &ActiveStateDir,
    pending: &Value,
) -> Result<(), CliError> {
    let rendered = render_transaction(pending, active_state)?;
    if active_state.exists(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE))? {
        let existing = read_transaction(active_state)?.ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "active ledger transaction disappeared during replacement: {}",
                transaction_path(active_state).display()
            ))
        })?;
        let existing = validate_transaction(existing, active_state)?;
        if existing.status != TransactionStatus::Complete || existing.state_publication_pending {
            return Err(CliError::IdempotencyConflict(format!(
                "active ledger transaction {} has pending ledger/state publication at {}",
                existing.request_hash,
                transaction_path(active_state).display()
            )));
        }
        active_state.write_atomic(
            Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
            &rendered,
            ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
        )?;
    } else {
        active_state.write_new(
            Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
            &rendered,
            ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
        )?;
    }
    Ok(())
}

fn drive_pending_transaction(
    active_state: &ActiveStateDir,
    mut transaction: ValidatedTransaction,
) -> Result<Value, CliError> {
    validate_recovery_images(active_state, &transaction)?;
    while transaction.published_count < transaction.writes.len() {
        let index = transaction.published_count;
        maybe_fail(active_state, &format!("before_publish_{index}"))?;
        let write = &transaction.writes[index];
        let current = read_ledger_value(active_state, &write.path)?;
        let current_hash = current.as_ref().map(value_hash).transpose()?;
        if current_hash != Some(write.after_hash.clone()) {
            if current_hash != write.before_hash {
                return Err(divergence_error(
                    active_state,
                    &write.path,
                    write.before_hash.clone(),
                    current_hash,
                ));
            }
            let rendered = render_ledger_value(&write.after, active_state, &write.path)?;
            active_state.write_atomic(
                Path::new(&write.path),
                &rendered,
                REPOSITORY_JSON_BYTE_LIMIT,
            )?;
        }
        maybe_fail(active_state, &format!("after_publish_{index}"))?;
        transaction.published_count += 1;
        transaction.record["published_count"] =
            Value::Number((transaction.published_count as u64).into());
        transaction.record["phase"] = Value::String("publishing".to_string());
        transaction.record["updated_at"] = Value::String(now_iso_utc());
        write_transaction(active_state, &transaction.record)?;
        maybe_fail(
            active_state,
            &format!("after_progress_{}", transaction.published_count),
        )?;
    }
    maybe_fail(active_state, "before_complete")?;
    let effects = transaction
        .writes
        .iter()
        .map(|write| {
            json!({
                "path": &write.path,
                "after_hash": &write.after_hash,
            })
        })
        .collect::<Vec<_>>();
    let complete = json!({
        "type": TRANSACTION_TYPE,
        "version": TRANSACTION_VERSION,
        "status": "complete",
        "operation": transaction.operation,
        "phase": "complete",
        "request_hash": transaction.request_hash,
        "result_hash": value_hash(&transaction.result)?,
        "result": transaction.result,
        "effects": effects,
        "state_publication": "pending",
        "completed_at": now_iso_utc(),
    });
    write_transaction(active_state, &complete)?;
    maybe_fail(active_state, "after_complete")?;
    Ok(complete["result"].clone())
}

fn validate_expected_preimages(
    active_state: &ActiveStateDir,
    pending: &Value,
) -> Result<(), CliError> {
    let transaction = validate_transaction(pending.clone(), active_state)?;
    for write in &transaction.writes {
        let current = read_ledger_value(active_state, &write.path)?;
        let current_hash = current.as_ref().map(value_hash).transpose()?;
        if current_hash != write.before_hash {
            return Err(divergence_error(
                active_state,
                &write.path,
                write.before_hash.clone(),
                current_hash,
            ));
        }
    }
    Ok(())
}

fn validate_recovery_images(
    active_state: &ActiveStateDir,
    transaction: &ValidatedTransaction,
) -> Result<(), CliError> {
    for (index, write) in transaction.writes.iter().enumerate() {
        let current = read_ledger_value(active_state, &write.path)?;
        let current_hash = current.as_ref().map(value_hash).transpose()?;
        let valid = if index < transaction.published_count {
            current_hash == Some(write.after_hash.clone())
        } else if index == transaction.published_count {
            current_hash == write.before_hash || current_hash == Some(write.after_hash.clone())
        } else {
            current_hash == write.before_hash
        };
        if !valid {
            let expected = if index < transaction.published_count {
                Some(write.after_hash.clone())
            } else {
                write.before_hash.clone()
            };
            return Err(divergence_error(
                active_state,
                &write.path,
                expected,
                current_hash,
            ));
        }
    }
    Ok(())
}

fn validate_complete_effects(
    active_state: &ActiveStateDir,
    transaction: &ValidatedTransaction,
) -> Result<(), CliError> {
    for effect in &transaction.effects {
        let current = read_ledger_value(active_state, &effect.path)?;
        let current_hash = current.as_ref().map(value_hash).transpose()?;
        if current_hash != Some(effect.after_hash.clone()) {
            return Err(divergence_error(
                active_state,
                &effect.path,
                Some(effect.after_hash.clone()),
                current_hash,
            ));
        }
    }
    Ok(())
}

fn validate_transaction(
    record: Value,
    active_state: &ActiveStateDir,
) -> Result<ValidatedTransaction, CliError> {
    let invalid = |message: String| {
        CliError::InvalidRepository(format!(
            "invalid active ledger transaction at {}: {message}",
            transaction_path(active_state).display()
        ))
    };
    if record.get("type").and_then(Value::as_str) != Some(TRANSACTION_TYPE) {
        return Err(invalid(format!("type must be {TRANSACTION_TYPE}")));
    }
    if record.get("version").and_then(Value::as_u64) != Some(TRANSACTION_VERSION) {
        return Err(invalid(format!("version must be {TRANSACTION_VERSION}")));
    }
    let status = match record.get("status").and_then(Value::as_str) {
        Some("pending") => TransactionStatus::Pending,
        Some("complete") => TransactionStatus::Complete,
        _ => return Err(invalid("status must be pending or complete".to_string())),
    };
    let expected_phase = match status {
        TransactionStatus::Pending => ["prepared", "publishing"].as_slice(),
        TransactionStatus::Complete => ["complete"].as_slice(),
    };
    let phase = record
        .get("phase")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("phase must be a string".to_string()))?;
    if !expected_phase.contains(&phase) {
        return Err(invalid(format!("phase {phase} does not match status")));
    }
    let operation = record
        .get("operation")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("operation must be a string".to_string()))?
        .to_string();
    validate_operation(&operation).map_err(|error| invalid(error.to_string()))?;
    let request_hash = record
        .get("request_hash")
        .and_then(Value::as_str)
        .filter(|hash| valid_hash(hash))
        .ok_or_else(|| invalid("request_hash must be a string".to_string()))?
        .to_string();
    let result = record
        .get("result")
        .ok_or_else(|| invalid("result is required".to_string()))?
        .clone();
    validate_result_size(&result, active_state)?;
    let result_hash = record
        .get("result_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("result_hash must be a string".to_string()))?;
    if result_hash != value_hash(&result)? {
        return Err(invalid("result_hash does not match result".to_string()));
    }
    for timestamp in match status {
        TransactionStatus::Pending => ["started_at", "updated_at"].as_slice(),
        TransactionStatus::Complete => ["completed_at"].as_slice(),
    } {
        if record.get(*timestamp).and_then(Value::as_str).is_none() {
            return Err(invalid(format!("{timestamp} must be a string")));
        }
    }
    let state_publication_pending = match status {
        TransactionStatus::Pending => {
            if record.get("state_publication").is_some()
                || record.get("state_publication_completed_at").is_some()
            {
                return Err(invalid(
                    "pending ledger transaction cannot contain state publication completion"
                        .to_string(),
                ));
            }
            false
        }
        TransactionStatus::Complete => {
            match record.get("state_publication").and_then(Value::as_str) {
                None | Some("pending") => {
                    if record.get("state_publication_completed_at").is_some() {
                        return Err(invalid(
                            "pending state publication cannot have a completion timestamp"
                                .to_string(),
                        ));
                    }
                    true
                }
                Some("released") => {
                    if record
                        .get("state_publication_completed_at")
                        .and_then(Value::as_str)
                        .is_none()
                    {
                        return Err(invalid(
                            "released state publication requires state_publication_completed_at"
                                .to_string(),
                        ));
                    }
                    false
                }
                Some(other) => {
                    return Err(invalid(format!(
                        "unsupported state_publication status: {other}"
                    )))
                }
            }
        }
    };

    let (published_count, writes, effects) = if status == TransactionStatus::Pending {
        let published_count = record
            .get("published_count")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| invalid("published_count must be an integer".to_string()))?;
        let raw_writes = record
            .get("writes")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("writes must be an array".to_string()))?;
        if raw_writes.is_empty() || raw_writes.len() > MAX_WRITES {
            return Err(invalid(format!(
                "writes must contain between 1 and {MAX_WRITES} entries"
            )));
        }
        if published_count > raw_writes.len() {
            return Err(invalid(
                "published_count exceeds the write count".to_string(),
            ));
        }
        if (published_count == 0 && phase != "prepared")
            || (published_count > 0 && phase != "publishing")
        {
            return Err(invalid(format!(
                "phase {phase} disagrees with published_count {published_count}"
            )));
        }
        let mut paths = BTreeSet::new();
        let mut writes = Vec::with_capacity(raw_writes.len());
        for raw in raw_writes {
            let path = raw
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("write path must be a string".to_string()))?
                .to_string();
            if !SUPPORTED_LEDGER_FILES.contains(&path.as_str()) || !paths.insert(path.clone()) {
                return Err(invalid(format!(
                    "write path is unsupported or duplicated: {path}"
                )));
            }
            let before_present = raw
                .get("before_present")
                .and_then(Value::as_bool)
                .ok_or_else(|| invalid("before_present must be a boolean".to_string()))?;
            let before_hash = raw
                .get("before_hash")
                .and_then(Value::as_str)
                .map(str::to_string);
            if before_present != before_hash.is_some() {
                return Err(invalid(
                    "before_present and before_hash disagree".to_string(),
                ));
            }
            if before_hash.as_deref().is_some_and(|hash| !valid_hash(hash)) {
                return Err(invalid("before_hash is invalid".to_string()));
            }
            let after = raw
                .get("after")
                .ok_or_else(|| invalid("write after value is required".to_string()))?
                .clone();
            render_ledger_value(&after, active_state, &path)?;
            let after_hash = raw
                .get("after_hash")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("after_hash must be a string".to_string()))?
                .to_string();
            if after_hash != value_hash(&after)? {
                return Err(invalid(format!(
                    "after_hash does not match postimage for {path}"
                )));
            }
            if before_hash == Some(after_hash.clone()) {
                return Err(invalid(format!(
                    "preimage and postimage are identical for {path}"
                )));
            }
            writes.push(ValidatedWrite {
                path,
                before_hash,
                after_hash,
                after,
            });
        }
        (published_count, writes, Vec::new())
    } else {
        let raw_effects = record
            .get("effects")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("effects must be an array".to_string()))?;
        if raw_effects.is_empty() || raw_effects.len() > MAX_WRITES {
            return Err(invalid(format!(
                "effects must contain between 1 and {MAX_WRITES} entries"
            )));
        }
        let mut paths = BTreeSet::new();
        let mut effects = Vec::with_capacity(raw_effects.len());
        for raw in raw_effects {
            let path = raw
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("effect path must be a string".to_string()))?
                .to_string();
            if !SUPPORTED_LEDGER_FILES.contains(&path.as_str()) || !paths.insert(path.clone()) {
                return Err(invalid(format!(
                    "effect path is unsupported or duplicated: {path}"
                )));
            }
            let after_hash = raw
                .get("after_hash")
                .and_then(Value::as_str)
                .filter(|hash| valid_hash(hash))
                .ok_or_else(|| invalid("effect after_hash is invalid".to_string()))?
                .to_string();
            effects.push(ValidatedEffect { path, after_hash });
        }
        (0, Vec::new(), effects)
    };

    Ok(ValidatedTransaction {
        record,
        status,
        state_publication_pending,
        operation,
        request_hash,
        result,
        published_count,
        writes,
        effects,
    })
}

fn read_transaction(active_state: &ActiveStateDir) -> Result<Option<Value>, CliError> {
    active_state
        .read_optional(
            Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
            ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
        )?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(CliError::from))
        .transpose()
}

fn write_transaction(active_state: &ActiveStateDir, value: &Value) -> Result<(), CliError> {
    let rendered = render_transaction(value, active_state)?;
    active_state.write_atomic(
        Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
        &rendered,
        ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
    )?;
    Ok(())
}

fn render_transaction(value: &Value, active_state: &ActiveStateDir) -> Result<Vec<u8>, CliError> {
    let mut rendered = serde_json::to_vec(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "active ledger transaction exceeds the {ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT}-byte limit at {}: {} bytes",
            transaction_path(active_state).display(),
            rendered.len()
        )));
    }
    Ok(rendered)
}

fn read_ledger_value(active_state: &ActiveStateDir, path: &str) -> Result<Option<Value>, CliError> {
    active_state
        .read_optional(Path::new(path), REPOSITORY_JSON_BYTE_LIMIT)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(CliError::from))
        .transpose()
}

fn render_ledger_value(
    value: &Value,
    active_state: &ActiveStateDir,
    path: &str,
) -> Result<Vec<u8>, CliError> {
    let mut rendered = serde_json::to_vec_pretty(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "active ledger postimage exceeds the {}-byte limit at {}: {} bytes",
            REPOSITORY_JSON_BYTE_LIMIT,
            active_state.display_path(Path::new(path)).display(),
            rendered.len()
        )));
    }
    Ok(rendered)
}

fn value_hash(value: &Value) -> Result<String, CliError> {
    Ok(codefire_util::sha256_prefixed(
        &codefire_store::canonical_json(value)?,
    ))
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == "sha256:".len() + 64
        && hash.starts_with("sha256:")
        && hash["sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn divergence_error(
    active_state: &ActiveStateDir,
    path: &str,
    expected: Option<String>,
    actual: Option<String>,
) -> CliError {
    CliError::InvalidRepository(format!(
        "active ledger transaction divergence for {} at {}: expected {}, actual {}",
        path,
        transaction_path(active_state).display(),
        expected.as_deref().unwrap_or("missing"),
        actual.as_deref().unwrap_or("missing")
    ))
}

fn transaction_path(active_state: &ActiveStateDir) -> std::path::PathBuf {
    active_state.display_path(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE))
}

#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
static FAILURE_INJECTIONS: Mutex<Vec<(std::path::PathBuf, String)>> = Mutex::new(Vec::new());

#[cfg(test)]
pub(crate) fn set_failure(active_state: &ActiveStateDir, point: &str) {
    FAILURE_INJECTIONS
        .lock()
        .expect("active ledger failure injection lock")
        .push((active_state.path().to_path_buf(), point.to_string()));
}

#[cfg(test)]
fn maybe_fail(active_state: &ActiveStateDir, point: &str) -> Result<(), CliError> {
    let mut failures = FAILURE_INJECTIONS
        .lock()
        .expect("active ledger failure injection lock");
    if let Some(index) = failures
        .iter()
        .position(|(path, candidate)| path == active_state.path() && candidate == point)
    {
        failures.remove(index);
        return Err(CliError::Io(std::io::Error::other(format!(
            "injected active ledger transaction failure at {point}"
        ))));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail(_active_state: &ActiveStateDir, _point: &str) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    fn active_fixture() -> (tempfile::TempDir, ActiveStateDir) {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("active");
        fs::create_dir(&parent).unwrap();
        let active = ActiveStateDir::create(&parent.join("open_test")).unwrap();
        (temp, active)
    }

    fn request(name: &str) -> Value {
        json!({"type": "test_active_ledger_request", "version": 1, "name": name})
    }

    fn transition(active: &ActiveStateDir, request: &Value) -> Result<Value, CliError> {
        apply_active_ledger_transition(
            active,
            "extinguish-batch",
            request,
            vec![
                ActiveLedgerWrite::new(
                    "fires.json",
                    Some(json!([{"id": "F1", "status": "open"}])),
                    json!([{"id": "F1", "status": "extinguished"}]),
                ),
                ActiveLedgerWrite::new(
                    "resolutions.json",
                    None,
                    json!([{"id": "R1", "fire": "F1"}]),
                ),
            ],
            &json!({"type": "test_result", "version": 1, "ids": ["F1"]}),
        )
    }

    fn seed_fire(active: &ActiveStateDir) {
        let bytes = b"[\n  {\n    \"id\": \"F1\",\n    \"status\": \"open\"\n  }\n]\n";
        active
            .write_atomic(Path::new("fires.json"), bytes, REPOSITORY_JSON_BYTE_LIMIT)
            .unwrap();
    }

    fn workflow_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let open_dir = temp.path().join("main-open");
        crate::init_repo(&repo_root, false).unwrap();
        crate::open_branch_from(
            &repo_root,
            &crate::OpenOptions {
                branch: "main".to_string(),
                path: open_dir.clone(),
                dry_run: false,
                json_output: false,
                lock: crate::LockOptions::default(),
                idempotency_key: None,
            },
        )
        .unwrap();
        fs::create_dir_all(open_dir.join("docs").join("requirements")).unwrap();
        fs::create_dir_all(open_dir.join("docs").join("design")).unwrap();
        fs::write(
            open_dir.join("docs").join("requirements").join("tx.md"),
            "## REQ-ledger-tx: Requirement\nDurable transition\n",
        )
        .unwrap();
        fs::write(
            open_dir.join("docs").join("design").join("tx.md"),
            "## DES-ledger-tx: Design\nDurable transition\n",
        )
        .unwrap();
        (temp, repo_root, open_dir)
    }

    fn manual_fire_options(open_dir: &Path, dry_run: bool) -> crate::fire::FireOptions {
        crate::fire::FireOptions {
            path: open_dir.to_path_buf(),
            source_atom: "REQ-ledger-tx".to_string(),
            target_atom: "DES-ledger-tx".to_string(),
            reason: "durable transition smoke".to_string(),
            severity: "required".to_string(),
            dry_run,
            full_output: false,
            json_output: true,
            lock: crate::LockOptions::default(),
        }
    }

    fn write_fire_batch(path: &Path, reason: &str) {
        fs::write(
            path,
            serde_json::to_vec(&json!({
                "version": 1,
                "defaults": {"reason": reason, "severity": "required"},
                "fires": [{"from": "REQ-ledger-tx", "to": "DES-ledger-tx"}],
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_extinguish_batch(path: &Path, fire_id: &str, rationale: &str) {
        fs::write(
            path,
            serde_json::to_vec(&json!({
                "version": 1,
                "defaults": {"resolution": "addressed", "rationale": rationale},
                "fires": [{"id": fire_id}],
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn run_after_complete_command(
        operation: &str,
        open_dir: &Path,
        exact_batch: &Path,
        foreign_batch: &Path,
        fire_id: Option<&str>,
        foreign: bool,
    ) -> Result<(), CliError> {
        match operation {
            "fire" => {
                let mut options = manual_fire_options(open_dir, false);
                options.reason = if foreign {
                    "foreign durable transition".to_string()
                } else {
                    "durable transition smoke".to_string()
                };
                crate::fire::run_fire(&options).map(|_| ())
            }
            "fire-batch" => crate::fire::run_fire_batch(&crate::fire::FireBatchOptions {
                path: open_dir.to_path_buf(),
                batch_path: if foreign {
                    foreign_batch.to_path_buf()
                } else {
                    exact_batch.to_path_buf()
                },
                dry_run: false,
                full_output: false,
                json_output: true,
                lock: crate::LockOptions::default(),
            })
            .map(|_| ()),
            "extinguish" => crate::run_extinguish(&crate::ExtinguishOptions {
                path: open_dir.to_path_buf(),
                fire_id: fire_id
                    .expect("extinguish fixture must have a fire")
                    .to_string(),
                resolution: "addressed".to_string(),
                rationale: if foreign {
                    "foreign resolution".to_string()
                } else {
                    "exact resolution".to_string()
                },
                evidence: String::new(),
                evidence_refs: Vec::new(),
                refresh: false,
                dry_run: false,
                json_output: true,
                lock: crate::LockOptions::default(),
                idempotency_key: None,
                edit_rationale: false,
            })
            .map(|_| ()),
            "extinguish-batch" => {
                crate::batch::run_extinguish_batch(&crate::batch::BatchExtinguishOptions {
                    path: open_dir.to_path_buf(),
                    batch_path: if foreign {
                        foreign_batch.to_path_buf()
                    } else {
                        exact_batch.to_path_buf()
                    },
                    dry_run: false,
                    full_output: false,
                    json_output: true,
                    lock: crate::LockOptions::default(),
                })
                .map(|_| ())
            }
            other => panic!("unsupported after-complete test operation: {other}"),
        }
    }

    fn workflow_authority_snapshot(
        repo_root: &Path,
        active: &ActiveStateDir,
    ) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut snapshot = BTreeMap::new();
        for path in [
            crate::branch_record_path(repo_root, "main"),
            crate::opened_registry_path(repo_root, "main"),
        ] {
            snapshot.insert(path.clone(), fs::read(path).unwrap());
        }
        let mut entries = fs::read_dir(active.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_file() {
                snapshot.insert(path.clone(), fs::read(path).unwrap());
            }
        }
        snapshot
    }

    fn assert_recovery_pending(error: &CliError, operation: &str, phase: &str) {
        match error {
            CliError::OpenRecoveryPending(pending) => {
                assert_eq!(pending.operation, operation);
                assert_eq!(pending.phase, phase);
            }
            other => panic!(
                "expected open_recovery_pending for {operation}/{phase}, got {other}: {:?}",
                other.diagnostic()
            ),
        }
        assert_eq!(error.diagnostic()["kind"], "open_recovery_pending");
    }

    fn assert_unrelated_open_consumers_blocked(
        repo_root: &Path,
        open_dir: &Path,
        active: &ActiveStateDir,
        operation: &str,
        phase: &str,
    ) {
        let before = workflow_authority_snapshot(repo_root, active);
        let errors = [
            crate::read_status(open_dir).map(|_| ()).unwrap_err(),
            crate::run_scan(open_dir).map(|_| ()).unwrap_err(),
            crate::run_verify(open_dir).map(|_| ()).unwrap_err(),
            crate::run_commit(&crate::CommitOptions {
                path: open_dir.to_path_buf(),
                message: "must not cross active-ledger recovery".to_string(),
                dry_run: false,
                full_output: false,
                json_output: true,
                lock: crate::LockOptions::default(),
                idempotency_key: None,
                signer: None,
                key_id: None,
            })
            .map(|_| ())
            .unwrap_err(),
        ];
        for error in &errors {
            assert_recovery_pending(error, operation, phase);
            assert_eq!(
                workflow_authority_snapshot(repo_root, active),
                before,
                "{operation}/{phase}: unrelated consumer changed authority"
            );
        }
    }

    #[test]
    fn every_publication_gap_recovers_to_one_postimage() {
        for point in [
            "before_prepare",
            "after_prepare",
            "before_publish_0",
            "after_publish_0",
            "after_progress_1",
            "before_publish_1",
            "after_publish_1",
            "after_progress_2",
            "before_complete",
            "after_complete",
        ] {
            let (_temp, active) = active_fixture();
            seed_fire(&active);
            let request = request(point);
            set_failure(&active, point);
            assert!(transition(&active, &request).is_err(), "{point}");

            let result = if point == "before_prepare" {
                transition(&active, &request).unwrap()
            } else {
                recover_active_ledger_transition(&active, "extinguish-batch", &request)
                    .unwrap()
                    .expect("matching transition must recover")
            };
            assert_eq!(result["ids"], json!(["F1"]), "{point}");
            assert_eq!(
                read_ledger_value(&active, "fires.json").unwrap(),
                Some(json!([{"id": "F1", "status": "extinguished"}])),
                "{point}"
            );
            assert_eq!(
                read_ledger_value(&active, "resolutions.json").unwrap(),
                Some(json!([{"id": "R1", "fire": "F1"}])),
                "{point}"
            );
            let marker = read_transaction(&active).unwrap().unwrap();
            assert_eq!(marker["status"], "complete", "{point}");
            assert!(marker.get("writes").is_none(), "{point}");
        }
    }

    #[test]
    fn manual_fire_dry_run_is_write_free_and_live_retry_is_exact() {
        let (_temp, _repo_root, open_dir) = workflow_fixture();
        let active_path = crate::open_context(&open_dir)
            .unwrap()
            .active_state
            .path()
            .to_path_buf();
        let active = crate::active_state::ActiveStateDir::bind(&active_path).unwrap();

        let dry_run = crate::fire::run_fire(&manual_fire_options(&open_dir, true)).unwrap();
        assert!(dry_run.dry_run);
        assert!(!active
            .exists(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE))
            .unwrap());
        assert!(!active.exists(Path::new("fires.json")).unwrap());

        let first = crate::fire::run_fire(&manual_fire_options(&open_dir, false)).unwrap();
        let replay = crate::fire::run_fire(&manual_fire_options(&open_dir, false)).unwrap();
        assert_eq!(
            crate::fire::fire_data_json(&replay),
            crate::fire::fire_data_json(&first)
        );
        assert_eq!(
            serde_json::to_value(&replay.fires).unwrap(),
            serde_json::to_value(&first.fires).unwrap()
        );
        let fires = read_ledger_value(&active, "fires.json").unwrap().unwrap();
        assert_eq!(fires.as_array().unwrap().len(), 1);
        let marker = read_transaction(&active).unwrap().unwrap();
        assert_eq!(marker["status"], "complete");
        assert_eq!(marker["operation"], "fire");
    }

    #[test]
    fn batch_extinguish_dry_run_preserves_receipt_and_live_retry_is_exact() {
        let (temp, _repo_root, open_dir) = workflow_fixture();
        let opened = crate::fire::run_fire(&manual_fire_options(&open_dir, false)).unwrap();
        let batch_path = temp.path().join("extinguish.json");
        fs::write(
            &batch_path,
            serde_json::to_vec(&json!({
                "version": 1,
                "defaults": {"resolution": "addressed", "rationale": "verified"},
                "fires": [{"id": &opened.fires[0].display_id}],
            }))
            .unwrap(),
        )
        .unwrap();
        let active_path = crate::open_context(&open_dir)
            .unwrap()
            .active_state
            .path()
            .to_path_buf();
        let active = crate::active_state::ActiveStateDir::bind(&active_path).unwrap();
        let receipt_before = active
            .read_required(
                Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
                ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
            )
            .unwrap();

        let run = |dry_run| {
            crate::batch::run_extinguish_batch(&crate::batch::BatchExtinguishOptions {
                path: open_dir.clone(),
                batch_path: batch_path.clone(),
                dry_run,
                full_output: false,
                json_output: true,
                lock: crate::LockOptions::default(),
            })
        };
        let dry_run = run(true).unwrap();
        assert!(dry_run.dry_run);
        assert_eq!(
            active
                .read_required(
                    Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
                    ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
                )
                .unwrap(),
            receipt_before
        );
        assert!(!active.exists(Path::new("resolutions.json")).unwrap());

        let first = run(false).unwrap();
        let replay = run(false).unwrap();
        assert_eq!(
            crate::batch::batch_extinguish_data_json(&replay),
            crate::batch::batch_extinguish_data_json(&first)
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json")
                .unwrap()
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let fires = read_ledger_value(&active, "fires.json").unwrap().unwrap();
        assert_eq!(fires[0]["status"], "extinguished");
        let marker = read_transaction(&active).unwrap().unwrap();
        assert_eq!(marker["status"], "complete");
        assert_eq!(marker["operation"], "extinguish-batch");
    }

    #[test]
    fn after_complete_real_commands_hold_receipt_until_exact_state_publication() {
        for operation in ["fire", "fire-batch", "extinguish", "extinguish-batch"] {
            let (temp, repo_root, open_dir) = workflow_fixture();
            let seeded = if matches!(operation, "extinguish" | "extinguish-batch") {
                Some(
                    crate::fire::run_fire(&manual_fire_options(&open_dir, false))
                        .unwrap()
                        .fires[0]
                        .display_id
                        .clone(),
                )
            } else {
                None
            };
            let exact_batch = temp.path().join(format!("{operation}-exact.json"));
            let foreign_batch = temp.path().join(format!("{operation}-foreign.json"));
            match operation {
                "fire-batch" => {
                    write_fire_batch(&exact_batch, "exact batch request");
                    write_fire_batch(&foreign_batch, "foreign batch request");
                }
                "extinguish-batch" => {
                    let fire_id = seeded.as_deref().unwrap();
                    write_extinguish_batch(&exact_batch, fire_id, "exact batch resolution");
                    write_extinguish_batch(&foreign_batch, fire_id, "foreign batch resolution");
                }
                _ => {}
            }
            let active_path = crate::open_context(&open_dir)
                .unwrap()
                .active_state
                .path()
                .to_path_buf();
            let active = ActiveStateDir::bind(&active_path).unwrap();
            set_failure(&active, "after_complete");

            let interrupted = run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                false,
            )
            .unwrap_err();
            assert!(
                interrupted.to_string().contains("after_complete"),
                "{operation}: {interrupted}"
            );
            assert!(!active.exists(Path::new("state_transaction.json")).unwrap());
            let receipt = read_transaction(&active).unwrap().unwrap();
            assert_eq!(receipt["status"], "complete", "{operation}");
            assert_eq!(receipt["operation"], operation, "{operation}");
            assert_eq!(receipt["state_publication"], "pending", "{operation}");
            assert_unrelated_open_consumers_blocked(
                &repo_root,
                &open_dir,
                &active,
                operation,
                "state_publication_pending",
            );

            let before_foreign = workflow_authority_snapshot(&repo_root, &active);
            let foreign = run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                true,
            )
            .unwrap_err();
            assert!(
                matches!(foreign, CliError::IdempotencyConflict(_)),
                "{operation}: {foreign}"
            );
            assert_eq!(
                workflow_authority_snapshot(&repo_root, &active),
                before_foreign,
                "{operation}: foreign request changed authority after ledger completion"
            );

            run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                false,
            )
            .unwrap();
            assert!(!active.exists(Path::new("state_transaction.json")).unwrap());
            let released = read_transaction(&active).unwrap().unwrap();
            assert_eq!(released["status"], "complete", "{operation}");
            assert_eq!(released["operation"], operation, "{operation}");
            assert_eq!(released["state_publication"], "released", "{operation}");
            assert!(released["state_publication_completed_at"].is_string());
            crate::read_status(&open_dir).unwrap();
            let registry =
                crate::read_json(&crate::opened_registry_path(&repo_root, "main")).unwrap();
            let branch = crate::load_branch_record(&repo_root, "main").unwrap();
            let state = crate::read_active_json_required(&active, Path::new("state.json")).unwrap();
            assert_eq!(
                registry["state"]["last_known"], branch["state"],
                "{operation}"
            );
            assert_eq!(branch["state"], state["state"], "{operation}");

            let released_before = workflow_authority_snapshot(&repo_root, &active);
            crate::set_state_transaction_failure_for_test(active.path(), operation, "started");
            run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                false,
            )
            .unwrap();
            assert!(!active.exists(Path::new("state_transaction.json")).unwrap());
            assert_eq!(
                workflow_authority_snapshot(&repo_root, &active),
                released_before,
                "{operation}: released exact replay performed a write"
            );
        }
    }

    #[test]
    fn pending_ledger_real_commands_block_nonowners_and_exact_owner_converges() {
        for operation in ["fire", "fire-batch", "extinguish", "extinguish-batch"] {
            let (temp, repo_root, open_dir) = workflow_fixture();
            let seeded = if matches!(operation, "extinguish" | "extinguish-batch") {
                Some(
                    crate::fire::run_fire(&manual_fire_options(&open_dir, false))
                        .unwrap()
                        .fires[0]
                        .display_id
                        .clone(),
                )
            } else {
                None
            };
            let exact_batch = temp.path().join(format!("{operation}-pending-exact.json"));
            let foreign_batch = temp
                .path()
                .join(format!("{operation}-pending-foreign.json"));
            match operation {
                "fire-batch" => {
                    write_fire_batch(&exact_batch, "exact pending batch request");
                    write_fire_batch(&foreign_batch, "foreign pending batch request");
                }
                "extinguish-batch" => {
                    let fire_id = seeded.as_deref().unwrap();
                    write_extinguish_batch(&exact_batch, fire_id, "exact pending batch resolution");
                    write_extinguish_batch(
                        &foreign_batch,
                        fire_id,
                        "foreign pending batch resolution",
                    );
                }
                _ => {}
            }
            let active_path = crate::open_context(&open_dir)
                .unwrap()
                .active_state
                .path()
                .to_path_buf();
            let active = ActiveStateDir::bind(&active_path).unwrap();
            set_failure(&active, "after_prepare");

            let interrupted = run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                false,
            )
            .unwrap_err();
            assert!(
                interrupted.to_string().contains("after_prepare"),
                "{operation}: {interrupted}"
            );
            assert!(!active.exists(Path::new("state_transaction.json")).unwrap());
            let descriptor = validate_active_ledger_descriptor(&active)
                .unwrap()
                .expect("pending ledger descriptor");
            assert_eq!(descriptor.operation, operation);
            assert_eq!(descriptor.phase, "prepared");
            assert_eq!(
                descriptor.publication_status,
                ActiveLedgerPublicationStatus::LedgerPending
            );
            assert_unrelated_open_consumers_blocked(
                &repo_root, &open_dir, &active, operation, "prepared",
            );

            let before_foreign = workflow_authority_snapshot(&repo_root, &active);
            let foreign = run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                true,
            )
            .unwrap_err();
            assert!(
                matches!(foreign, CliError::IdempotencyConflict(_)),
                "{operation}: {foreign}"
            );
            assert_eq!(
                workflow_authority_snapshot(&repo_root, &active),
                before_foreign,
                "{operation}: foreign request changed a pending ledger"
            );

            run_after_complete_command(
                operation,
                &open_dir,
                &exact_batch,
                &foreign_batch,
                seeded.as_deref(),
                false,
            )
            .unwrap();
            assert!(!active.exists(Path::new("state_transaction.json")).unwrap());
            let released = validate_active_ledger_descriptor(&active)
                .unwrap()
                .expect("released ledger descriptor");
            assert_eq!(released.operation, operation);
            assert_eq!(
                released.publication_status,
                ActiveLedgerPublicationStatus::Released
            );
            let registry =
                crate::read_json(&crate::opened_registry_path(&repo_root, "main")).unwrap();
            let branch = crate::load_branch_record(&repo_root, "main").unwrap();
            let state = crate::read_active_json_required(&active, Path::new("state.json")).unwrap();
            assert_eq!(registry["state"]["last_known"], branch["state"]);
            assert_eq!(branch["state"], state["state"]);
        }
    }

    #[test]
    fn complete_request_replays_the_exact_recorded_result_without_writes() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        let request = request("exact");
        let first = transition(&active, &request).unwrap();
        let fires_before = active
            .read_required(Path::new("fires.json"), REPOSITORY_JSON_BYTE_LIMIT)
            .unwrap();
        let replay = recover_active_ledger_transition(&active, "extinguish-batch", &request)
            .unwrap()
            .unwrap();
        assert_eq!(replay, first);
        assert_eq!(
            active
                .read_required(Path::new("fires.json"), REPOSITORY_JSON_BYTE_LIMIT)
                .unwrap(),
            fires_before
        );
    }

    #[test]
    fn pending_request_collision_is_rejected_without_publication() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        let first = request("first");
        set_failure(&active, "after_prepare");
        assert!(transition(&active, &first).is_err());

        let error =
            recover_active_ledger_transition(&active, "extinguish-batch", &request("second"))
                .unwrap_err();
        assert!(matches!(error, CliError::IdempotencyConflict(_)));
        assert_eq!(
            read_ledger_value(&active, "fires.json").unwrap(),
            Some(json!([{"id": "F1", "status": "open"}]))
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            None
        );
    }

    #[test]
    fn recovery_rejects_divergent_ledger_instead_of_overwriting_it() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        let request = request("divergence");
        set_failure(&active, "after_progress_1");
        assert!(transition(&active, &request).is_err());
        let divergent = serde_json::to_vec(&json!([{"id": "F1", "status": "corrupt"}])).unwrap();
        active
            .write_atomic(
                Path::new("fires.json"),
                &divergent,
                REPOSITORY_JSON_BYTE_LIMIT,
            )
            .unwrap();

        let error =
            recover_active_ledger_transition(&active, "extinguish-batch", &request).unwrap_err();
        assert!(error.to_string().contains("divergence"));
        assert_eq!(
            read_ledger_value(&active, "fires.json").unwrap(),
            Some(json!([{"id": "F1", "status": "corrupt"}]))
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            None
        );
    }

    #[test]
    fn complete_replay_rejects_a_postimage_that_changed_after_publication() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        let request = request("complete-divergence");
        transition(&active, &request).unwrap();
        let divergent = serde_json::to_vec(&json!([{"id": "F1", "status": "corrupt"}])).unwrap();
        active
            .write_atomic(
                Path::new("fires.json"),
                &divergent,
                REPOSITORY_JSON_BYTE_LIMIT,
            )
            .unwrap();

        let error =
            recover_active_ledger_transition(&active, "extinguish-batch", &request).unwrap_err();
        assert!(error.to_string().contains("divergence"));
        assert_eq!(
            read_ledger_value(&active, "fires.json").unwrap(),
            Some(json!([{"id": "F1", "status": "corrupt"}]))
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            Some(json!([{"id": "R1", "fire": "F1"}]))
        );
    }

    #[cfg(unix)]
    #[test]
    fn ledger_and_journal_symlinks_are_never_followed() {
        use std::os::unix::fs::symlink;

        let (temp, active) = active_fixture();
        let outside = temp.path().join("outside.json");
        fs::write(&outside, b"outside\n").unwrap();
        symlink(&outside, active.display_path(Path::new("fires.json"))).unwrap();
        let error = transition(&active, &request("ledger-symlink")).unwrap_err();
        assert!(matches!(error, CliError::ActiveStateCapability(_)));
        assert_eq!(fs::read(&outside).unwrap(), b"outside\n");
        assert!(!active
            .exists(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE))
            .unwrap());

        fs::remove_file(active.display_path(Path::new("fires.json"))).unwrap();
        seed_fire(&active);
        symlink(
            &outside,
            active.display_path(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE)),
        )
        .unwrap();
        let error = transition(&active, &request("journal-symlink")).unwrap_err();
        assert!(matches!(error, CliError::ActiveStateCapability(_)));
        assert_eq!(fs::read(&outside).unwrap(), b"outside\n");
    }

    #[test]
    fn oversized_journal_is_rejected_before_allocation_or_ledger_write() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        let marker = active.display_path(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE));
        let file = fs::File::create(&marker).unwrap();
        file.set_len(ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT + 1)
            .unwrap();

        let error = transition(&active, &request("oversized")).unwrap_err();
        assert!(matches!(error, CliError::ActiveStateCapability(_)));
        assert_eq!(
            read_ledger_value(&active, "fires.json").unwrap(),
            Some(json!([{"id": "F1", "status": "open"}]))
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            None
        );
    }

    #[test]
    fn preview_guard_is_read_only_for_clean_and_pending_state() {
        let (_temp, active) = active_fixture();
        seed_fire(&active);
        reject_pending_active_ledger_transition(&active).unwrap();
        let _plan = [ActiveLedgerWrite::new(
            "fires.json",
            Some(json!([{"id": "F1", "status": "open"}])),
            json!([{"id": "F1", "status": "extinguished"}]),
        )];
        assert!(!active
            .exists(Path::new(ACTIVE_LEDGER_TRANSACTION_FILE))
            .unwrap());
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            None
        );

        let request = request("preview-pending");
        set_failure(&active, "after_prepare");
        assert!(transition(&active, &request).is_err());
        let marker_before = active
            .read_required(
                Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
                ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
            )
            .unwrap();
        let fires_before = active
            .read_required(Path::new("fires.json"), REPOSITORY_JSON_BYTE_LIMIT)
            .unwrap();
        let error = reject_pending_active_ledger_transition(&active).unwrap_err();
        assert!(matches!(error, CliError::IdempotencyConflict(_)));
        assert_eq!(
            active
                .read_required(
                    Path::new(ACTIVE_LEDGER_TRANSACTION_FILE),
                    ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT,
                )
                .unwrap(),
            marker_before
        );
        assert_eq!(
            active
                .read_required(Path::new("fires.json"), REPOSITORY_JSON_BYTE_LIMIT)
                .unwrap(),
            fires_before
        );
        assert_eq!(
            read_ledger_value(&active, "resolutions.json").unwrap(),
            None
        );
    }
}
