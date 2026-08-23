//! Durable, idempotent cleanup of an open workspace after commit publication.
//!
//! The pending commit marker is the authority: it records exact preimages and
//! an ordered progress prefix before destructive reset. Recovery first checks
//! every target without writing, accepts only the single apply-before-progress
//! gap that the state machine can produce, and rejects replacements or
//! post-progress reappearance before continuing.

use crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE;
use crate::{
    active_state, commit_transaction_path, maybe_fail_commit_transaction_for_test, now_iso_utc,
    read_commit_transaction, required_string, validate_commit_transaction_record,
    write_pending_commit_transaction, CliError, REPOSITORY_JSON_BYTE_LIMIT,
};
use serde_json::{json, Value};
use std::path::Path;

const REMOVALS: &[(&str, &str)] = &[
    ("fires.json", "after_active_reset_remove_fires"),
    ("resolutions.json", "after_active_reset_remove_resolutions"),
    ("scan.json", "after_active_reset_remove_scan"),
    (
        "verification.json",
        "after_active_reset_remove_verification",
    ),
    (
        ACTIVE_LEDGER_TRANSACTION_FILE,
        "after_active_reset_remove_active_ledger_transaction",
    ),
    (
        "state_transaction.json",
        "after_active_reset_remove_state_transaction",
    ),
];
const STATE_PATH: &str = "state.json";
const STATE_FAILPOINT: &str = "after_active_reset_state_write";

pub(crate) fn build_commit_active_reset_authority(
    active_state: &active_state::ActiveStateDir,
) -> Result<Value, CliError> {
    let removals = REMOVALS
        .iter()
        .map(|(path, _)| {
            let bytes = active_state.read_optional(Path::new(path), REPOSITORY_JSON_BYTE_LIMIT)?;
            Ok(json!({
                "path": path,
                "preimage": snapshot(bytes.as_deref()),
                "progress": "planned",
            }))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    let state_preimage =
        active_state.read_optional(Path::new(STATE_PATH), REPOSITORY_JSON_BYTE_LIMIT)?;
    let state_postimage = commit_active_reset_state_postimage_bytes()?;
    Ok(json!({
        "version": 1,
        "removals": removals,
        "state_file": {
            "path": STATE_PATH,
            "preimage": snapshot(state_preimage.as_deref()),
            "postimage": snapshot(Some(&state_postimage)),
            "progress": "planned",
        },
    }))
}

pub(crate) fn validate_commit_active_reset_authority(authority: &Value) -> Result<(), String> {
    let object = authority
        .as_object()
        .ok_or_else(|| "commit_transaction.json active_reset must be an object".to_string())?;
    if object.len() != 3
        || !object.contains_key("version")
        || !object.contains_key("removals")
        || !object.contains_key("state_file")
    {
        return Err(
            "commit_transaction.json active_reset must contain only version, removals, and state_file"
                .to_string(),
        );
    }
    if object.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("commit_transaction.json active_reset.version must be 1".to_string());
    }
    let removals = object
        .get("removals")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "commit_transaction.json active_reset.removals must be a list".to_string()
        })?;
    if removals.len() != REMOVALS.len() {
        return Err(format!(
            "commit_transaction.json active_reset.removals must contain {} entries",
            REMOVALS.len()
        ));
    }
    let mut saw_planned_removal = false;
    for ((expected_path, _), removal) in REMOVALS.iter().zip(removals) {
        let removal = removal.as_object().ok_or_else(|| {
            "commit_transaction.json active reset removal must be an object".to_string()
        })?;
        if removal.len() != 3
            || !removal.contains_key("path")
            || !removal.contains_key("preimage")
            || !removal.contains_key("progress")
        {
            return Err(
                "commit active reset removal must contain only path, preimage, and progress"
                    .to_string(),
            );
        }
        if removal.get("path").and_then(Value::as_str) != Some(*expected_path) {
            return Err(format!(
                "commit active reset removal path must be {expected_path}"
            ));
        }
        validate_snapshot(
            removal
                .get("preimage")
                .expect("active reset preimage presence was checked"),
        )?;
        match removal.get("progress").and_then(Value::as_str) {
            Some("planned") => saw_planned_removal = true,
            Some("removed") if !saw_planned_removal => {}
            Some("removed") => {
                return Err(
                    "commit active reset removal progress must be an ordered removed prefix"
                        .to_string(),
                );
            }
            _ => {
                return Err(format!(
                    "commit active reset removal progress for {expected_path} must be planned or removed"
                ));
            }
        }
    }
    let state_file = object
        .get("state_file")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "commit_transaction.json active_reset.state_file must be an object".to_string()
        })?;
    if state_file.len() != 4
        || !state_file.contains_key("path")
        || !state_file.contains_key("preimage")
        || !state_file.contains_key("postimage")
        || !state_file.contains_key("progress")
    {
        return Err(
            "commit active reset state_file must contain only path, preimage, postimage, and progress"
                .to_string(),
        );
    }
    if state_file.get("path").and_then(Value::as_str) != Some(STATE_PATH) {
        return Err(format!(
            "commit active reset state_file path must be {STATE_PATH}"
        ));
    }
    validate_snapshot(
        state_file
            .get("preimage")
            .expect("active reset state preimage presence was checked"),
    )?;
    let postimage = state_file
        .get("postimage")
        .expect("active reset state postimage presence was checked");
    validate_snapshot(postimage)?;
    let expected_postimage = canonical_state_postimage_snapshot()
        .map_err(|error| format!("failed to construct active reset postimage: {error}"))?;
    if postimage != &expected_postimage {
        return Err(
            "commit active reset state_file.postimage must be the open-clean authority".to_string(),
        );
    }
    match state_file.get("progress").and_then(Value::as_str) {
        Some("planned") => {}
        Some("written") if !saw_planned_removal => {}
        Some("written") => {
            return Err(
                "commit active reset state_file cannot be written before every removal completes"
                    .to_string(),
            );
        }
        _ => {
            return Err(
                "commit active reset state_file.progress must be planned or written".to_string(),
            );
        }
    }
    Ok(())
}

pub(crate) fn commit_active_reset_complete(authority: &Value) -> bool {
    authority
        .get("removals")
        .and_then(Value::as_array)
        .is_some_and(|removals| {
            removals
                .iter()
                .all(|removal| removal.get("progress").and_then(Value::as_str) == Some("removed"))
        })
        && authority
            .pointer("/state_file/progress")
            .and_then(Value::as_str)
            == Some("written")
}

pub(crate) fn commit_active_reset_unstarted(authority: &Value) -> bool {
    authority
        .get("removals")
        .and_then(Value::as_array)
        .is_some_and(|removals| {
            removals
                .iter()
                .all(|removal| removal.get("progress").and_then(Value::as_str) == Some("planned"))
        })
        && authority
            .pointer("/state_file/progress")
            .and_then(Value::as_str)
            == Some("planned")
}

pub(crate) fn transition_commit_transaction_to_active_reset(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    validate_commit_transaction_record(&record).map_err(|message| {
        CliError::InvalidRepository(format!(
            "{message}: {}",
            commit_transaction_path(active_state).display()
        ))
    })?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed before active-state reset".to_string(),
        ));
    }
    if required_string(&record, &["phase"])? != "publishing_branch" {
        return Err(CliError::InvalidRepository(
            "commit transaction phase changed before active-state reset".to_string(),
        ));
    }
    if record.get("active_reset").is_none() {
        record["active_reset"] = build_commit_active_reset_authority(active_state)?;
    }
    record["phase"] = Value::String("resetting_active_state".to_string());
    record["updated_at"] = Value::String(now_iso_utc());
    validate_commit_transaction_record(&record).map_err(|message| {
        CliError::InvalidRepository(format!(
            "cannot begin durable active-state reset: {message}: {}",
            commit_transaction_path(active_state).display()
        ))
    })?;
    write_pending_commit_transaction(active_state, &record)
}

pub(crate) fn preflight_commit_active_reset_authority(
    active_state: &active_state::ActiveStateDir,
    authority: &Value,
) -> Result<(), CliError> {
    validate_commit_active_reset_authority(authority).map_err(|message| {
        CliError::InvalidRepository(format!(
            "invalid durable commit active-state reset authority: {message}"
        ))
    })?;
    let removals = authority["removals"]
        .as_array()
        .expect("validated active reset removals");
    let first_planned_index = removals
        .iter()
        .position(|removal| removal["progress"].as_str() == Some("planned"));
    for (index, removal) in removals.iter().enumerate() {
        let path = removal["path"]
            .as_str()
            .expect("validated active reset removal path");
        let current = active_state.read_optional(Path::new(path), REPOSITORY_JSON_BYTE_LIMIT)?;
        let matches_preimage = snapshot_matches(&removal["preimage"], current.as_deref());
        let valid = match removal["progress"]
            .as_str()
            .expect("validated active reset removal progress")
        {
            "planned" => {
                matches_preimage || (Some(index) == first_planned_index && current.is_none())
            }
            "removed" => current.is_none(),
            _ => unreachable!("validated active reset removal progress"),
        };
        if !valid {
            return Err(divergence(
                active_state,
                Path::new(path),
                "file does not match its durable preimage/removal progress",
            ));
        }
    }

    let state_file = &authority["state_file"];
    let state_path = state_file["path"]
        .as_str()
        .expect("validated active reset state path");
    let current = active_state.read_optional(Path::new(state_path), REPOSITORY_JSON_BYTE_LIMIT)?;
    let matches_preimage = snapshot_matches(&state_file["preimage"], current.as_deref());
    let matches_postimage = snapshot_matches(&state_file["postimage"], current.as_deref());
    let valid = match state_file["progress"]
        .as_str()
        .expect("validated active reset state progress")
    {
        "planned" => matches_preimage || matches_postimage,
        "written" => matches_postimage,
        _ => unreachable!("validated active reset state progress"),
    };
    if !valid {
        return Err(divergence(
            active_state,
            Path::new(state_path),
            "file does not match its durable preimage/postimage progress",
        ));
    }
    Ok(())
}

pub(crate) fn reset_commit_active_state(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
) -> Result<(), CliError> {
    let transaction = read_commit_transaction(active_state)?;
    preflight_transaction(active_state, transaction_id, &transaction)?;

    for (index, (expected_path, failpoint)) in REMOVALS.iter().enumerate() {
        let transaction = read_commit_transaction(active_state)?;
        let authority =
            validated_resetting_transaction(active_state, transaction_id, &transaction)?;
        let removal = &authority["removals"][index];
        if removal["progress"].as_str() == Some("removed") {
            continue;
        }
        let current =
            active_state.read_optional(Path::new(expected_path), REPOSITORY_JSON_BYTE_LIMIT)?;
        if current.is_some() && !snapshot_matches(&removal["preimage"], current.as_deref()) {
            return Err(divergence(
                active_state,
                Path::new(expected_path),
                "file changed after reset preflight",
            ));
        }
        if current.is_some() {
            active_state.remove_file_if_exists(Path::new(expected_path))?;
        }
        maybe_fail_commit_transaction_for_test(active_state.path(), failpoint)?;
        mark_removal(active_state, transaction_id, index, expected_path)?;
    }

    let transaction = read_commit_transaction(active_state)?;
    let authority = validated_resetting_transaction(active_state, transaction_id, &transaction)?;
    if authority["state_file"]["progress"].as_str() == Some("planned") {
        let current =
            active_state.read_optional(Path::new(STATE_PATH), REPOSITORY_JSON_BYTE_LIMIT)?;
        let postimage = commit_active_reset_state_postimage_bytes()?;
        if !snapshot_matches(&authority["state_file"]["postimage"], current.as_deref()) {
            if !snapshot_matches(&authority["state_file"]["preimage"], current.as_deref()) {
                return Err(divergence(
                    active_state,
                    Path::new(STATE_PATH),
                    "file changed after reset preflight",
                ));
            }
            active_state.write_atomic(
                Path::new(STATE_PATH),
                &postimage,
                REPOSITORY_JSON_BYTE_LIMIT,
            )?;
        }
        maybe_fail_commit_transaction_for_test(active_state.path(), STATE_FAILPOINT)?;
        mark_state_written(active_state, transaction_id)?;
    }

    let transaction = read_commit_transaction(active_state)?;
    preflight_transaction(active_state, transaction_id, &transaction)?;
    let authority = transaction
        .get("active_reset")
        .expect("preflight requires active reset authority");
    if !commit_active_reset_complete(authority) {
        return Err(CliError::InvalidRepository(
            "durable commit active-state reset did not reach its complete progress state"
                .to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn commit_active_reset_state_postimage_bytes() -> Result<Vec<u8>, CliError> {
    let mut bytes = serde_json::to_vec_pretty(&json!({"state": "open-clean"}))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_state_postimage_snapshot() -> Result<Value, CliError> {
    Ok(snapshot(
        Some(&commit_active_reset_state_postimage_bytes()?),
    ))
}

fn snapshot(bytes: Option<&[u8]>) -> Value {
    match bytes {
        Some(bytes) => json!({
            "present": true,
            "byte_count": bytes.len(),
            "sha256": codefire_util::sha256_hex(bytes),
        }),
        None => json!({"present": false}),
    }
}

fn validate_snapshot(snapshot: &Value) -> Result<(), String> {
    let object = snapshot
        .as_object()
        .ok_or_else(|| "commit active reset snapshot must be an object".to_string())?;
    let present = object
        .get("present")
        .and_then(Value::as_bool)
        .ok_or_else(|| "commit active reset snapshot.present must be a boolean".to_string())?;
    if !present {
        if object.len() != 1 {
            return Err(
                "an absent commit active reset snapshot may contain only present".to_string(),
            );
        }
        return Ok(());
    }
    if object.len() != 3 || !object.contains_key("byte_count") || !object.contains_key("sha256") {
        return Err(
            "a present commit active reset snapshot requires present, byte_count, and sha256"
                .to_string(),
        );
    }
    let byte_count = object
        .get("byte_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "commit active reset snapshot.byte_count must be a non-negative integer".to_string()
        })?;
    if byte_count > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(format!(
            "commit active reset snapshot.byte_count must not exceed {REPOSITORY_JSON_BYTE_LIMIT}"
        ));
    }
    let hash = object
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "commit active reset snapshot.sha256 must be a string".to_string())?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "commit active reset snapshot.sha256 must be a lowercase 64-character hex digest"
                .to_string(),
        );
    }
    Ok(())
}

fn snapshot_matches(snapshot: &Value, current: Option<&[u8]>) -> bool {
    match snapshot.get("present").and_then(Value::as_bool) {
        Some(false) => current.is_none(),
        Some(true) => current.is_some_and(|bytes| {
            let byte_count = u64::try_from(bytes.len()).ok();
            let digest = codefire_util::sha256_hex(bytes);
            snapshot.get("byte_count").and_then(Value::as_u64) == byte_count
                && snapshot.get("sha256").and_then(Value::as_str) == Some(digest.as_str())
        }),
        None => false,
    }
}

fn divergence(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
    detail: &str,
) -> CliError {
    CliError::InvalidRepository(format!(
        "durable commit active-state reset found divergent content at {} ({detail}); the pending transaction was preserved without further reset mutations",
        active_state.display_path(relative).display()
    ))
}

fn validated_resetting_transaction<'a>(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    transaction: &'a Value,
) -> Result<&'a Value, CliError> {
    validate_commit_transaction_record(transaction).map_err(|message| {
        CliError::InvalidRepository(format!(
            "{message}: {}",
            commit_transaction_path(active_state).display()
        ))
    })?;
    if required_string(transaction, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed during active-state reset".to_string(),
        ));
    }
    if required_string(transaction, &["phase"])? != "resetting_active_state" {
        return Err(CliError::InvalidRepository(
            "commit transaction phase changed during active-state reset".to_string(),
        ));
    }
    transaction.get("active_reset").ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "pending commit transaction lacks the durable active-state reset authority required for automatic recovery: {}",
            commit_transaction_path(active_state).display()
        ))
    })
}

fn preflight_transaction(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    transaction: &Value,
) -> Result<(), CliError> {
    let authority = validated_resetting_transaction(active_state, transaction_id, transaction)?;
    preflight_commit_active_reset_authority(active_state, authority)
}

fn mark_removal(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    removal_index: usize,
    expected_path: &str,
) -> Result<(), CliError> {
    let mut transaction = read_commit_transaction(active_state)?;
    validated_resetting_transaction(active_state, transaction_id, &transaction)?;
    if active_state.exists(Path::new(expected_path))? {
        return Err(divergence(
            active_state,
            Path::new(expected_path),
            "file reappeared before its removal progress could be recorded",
        ));
    }
    let removal = transaction
        .pointer_mut(&format!("/active_reset/removals/{removal_index}"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "commit active-state reset removal entry {removal_index} disappeared"
            ))
        })?;
    if removal.get("path").and_then(Value::as_str) != Some(expected_path)
        || removal.get("progress").and_then(Value::as_str) != Some("planned")
    {
        return Err(CliError::InvalidRepository(format!(
            "commit active-state reset progress changed before recording removal of {expected_path}"
        )));
    }
    removal.insert("progress".to_string(), Value::String("removed".to_string()));
    transaction["updated_at"] = Value::String(now_iso_utc());
    validate_commit_transaction_record(&transaction).map_err(|message| {
        CliError::InvalidRepository(format!(
            "cannot record durable active-state removal progress: {message}"
        ))
    })?;
    write_pending_commit_transaction(active_state, &transaction)
}

fn mark_state_written(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
) -> Result<(), CliError> {
    let mut transaction = read_commit_transaction(active_state)?;
    let authority = validated_resetting_transaction(active_state, transaction_id, &transaction)?;
    let state_file = &authority["state_file"];
    let current = active_state.read_optional(Path::new(STATE_PATH), REPOSITORY_JSON_BYTE_LIMIT)?;
    if !snapshot_matches(&state_file["postimage"], current.as_deref()) {
        return Err(divergence(
            active_state,
            Path::new(STATE_PATH),
            "postimage changed before its write progress could be recorded",
        ));
    }
    let progress = transaction
        .pointer_mut("/active_reset/state_file/progress")
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "commit active-state reset state progress disappeared".to_string(),
            )
        })?;
    if progress.as_str() != Some("planned") {
        return Err(CliError::InvalidRepository(
            "commit active-state reset state progress changed before recording its write"
                .to_string(),
        ));
    }
    *progress = Value::String("written".to_string());
    transaction["updated_at"] = Value::String(now_iso_utc());
    validate_commit_transaction_record(&transaction).map_err(|message| {
        CliError::InvalidRepository(format!(
            "cannot record durable active-state state-file progress: {message}"
        ))
    })?;
    write_pending_commit_transaction(active_state, &transaction)
}
