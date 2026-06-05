use super::{ref_file_name, required_string, CliError};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(super) fn require_idempotency_key(key: &str) -> Result<(), CliError> {
    if key.is_empty() {
        return Err(CliError::Usage(
            "--idempotency-key must not be empty".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn idempotency_record_path(repo_root: &Path, command: &str, key: &str) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("idempotency")
        .join(command)
        .join(format!("{}.json", ref_file_name(key)))
}

pub(super) fn verify_idempotency_record(
    record: &Value,
    path: &Path,
    command: &str,
    key: &str,
    payload: &Value,
) -> Result<(), CliError> {
    let payload_hash = idempotency_payload_hash(payload)?;
    let recorded_hash = required_string(record, &["payload_hash"])?;
    if recorded_hash != payload_hash {
        return Err(CliError::IdempotencyConflict(format!(
            "idempotency key conflict for {command}: {key}"
        )));
    }
    if required_string(record, &["key"])? != key {
        return Err(CliError::InvalidRepository(format!(
            "idempotency record key mismatch: {}",
            path.display()
        )));
    }
    Ok(())
}

pub(super) fn idempotency_result_plan(record: &Value, path: &Path) -> Result<Value, CliError> {
    record
        .get("result")
        .and_then(|result| result.get("plan"))
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "idempotency record missing result.plan: {}",
                path.display()
            ))
        })
}

pub(super) fn idempotency_payload_hash(payload: &Value) -> Result<String, CliError> {
    let mut hasher = Sha256::new();
    hasher.update(codefire_store::canonical_json(payload)?);
    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
