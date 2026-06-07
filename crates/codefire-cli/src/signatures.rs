use super::remote::{ensure_remote_layout, read_optional_json, remote_dirs};
use super::*;
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use serde_json::{json, Map, Value};
use sha2::Sha256;
use std::collections::BTreeSet;
use std::env;
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn maybe_sign_commit(
    commit: Value,
    signer: Option<&str>,
    key_id: Option<&str>,
    require_signature: bool,
) -> Result<Value, CliError> {
    let signing_key = env::var("CODEFIRE_SIGNING_KEY").unwrap_or_default();
    if signing_key.is_empty() {
        if require_signature {
            return Err(CliError::Usage(
                "commit blocked: commit signature is required but CODEFIRE_SIGNING_KEY is not set"
                    .to_string(),
            ));
        }
        return Ok(commit);
    }
    let signer = signer
        .map(str::to_string)
        .or_else(|| env::var("CODEFIRE_SIGNER").ok())
        .or_else(|| env::var("USER").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "anonymous".to_string());
    let key_id = key_id
        .map(str::to_string)
        .or_else(|| env::var("CODEFIRE_SIGNING_KEY_ID").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| signer.clone());
    sign_commit_payload(commit, &signer, &signing_key, &key_id)
}

pub(crate) fn sign_commit_payload(
    mut commit: Value,
    signer: &str,
    key: &str,
    key_id: &str,
) -> Result<Value, CliError> {
    let mut signature = json!({
        "type": "commit_signature",
        "version": 1,
        "algorithm": "hmac-sha256",
        "signer": signer,
        "key_id": key_id,
        "signed_at": now_iso_utc(),
    });
    let digest = commit_signature_digest(&commit, &signature, key)?;
    signature["signature"] = Value::String(digest);
    commit["signature"] = signature;
    Ok(commit)
}

pub(crate) fn enforce_remote_commit_signature_policy(
    project_root: &Path,
    objects_root: &Path,
    commit_id: &str,
) -> Result<(), CliError> {
    let policy = remote_policy(project_root)?;
    let Some(signature_policy) = policy.get("commit_signatures") else {
        return Ok(());
    };
    if !signature_policy
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let keys = signature_policy
        .get("keys")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(
                "commit signature validation failed: commit_signatures.keys must be an object"
                    .to_string(),
            )
        })?;
    let require_history = signature_policy
        .get("require_history")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut seen = BTreeSet::new();
    verify_commit_policy_recursive(
        objects_root,
        commit_id,
        keys,
        require_history,
        true,
        &mut seen,
    )
}

pub(crate) fn remote_request_target(
    org: &str,
    app: &str,
    operation: &str,
    branch: Option<&str>,
    mr_id: Option<&str>,
) -> String {
    match operation {
        "upload" => format!("{org}/{app}/branches/{}/upload", branch.unwrap_or_default()),
        "request_merge" => format!("{org}/{app}/merge-requests"),
        "review" => format!(
            "{org}/{app}/merge-requests/{}/review",
            mr_id.unwrap_or_default()
        ),
        "apply" => format!(
            "{org}/{app}/merge-requests/{}/apply",
            mr_id.unwrap_or_default()
        ),
        "gc" => format!("{org}/{app}/gc"),
        _ => format!("{org}/{app}/{operation}"),
    }
}

pub(crate) fn sign_remote_request(
    operation: &str,
    actor: &str,
    target: &str,
    request_key_id: Option<&str>,
) -> Result<Option<Value>, CliError> {
    let key = env::var("CODEFIRE_REQUEST_SIGNING_KEY").unwrap_or_default();
    if key.is_empty() {
        return Ok(None);
    }
    let key_id = request_key_id
        .map(str::to_string)
        .or_else(|| env::var("CODEFIRE_REQUEST_KEY_ID").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| actor.to_string());
    sign_remote_request_with_key(operation, actor, target, &key_id, &key).map(Some)
}

pub(crate) fn sign_remote_request_with_key(
    operation: &str,
    actor: &str,
    target: &str,
    key_id: &str,
    key: &str,
) -> Result<Value, CliError> {
    let mut signature = json!({
        "type": "remote_request_signature",
        "version": 1,
        "algorithm": "hmac-sha256",
        "actor": actor,
        "operation": operation,
        "target": target,
        "key_id": key_id,
        "timestamp": now_iso_utc(),
        "nonce": random_nonce()?,
    });
    let digest = request_signature_digest(&signature, key)?;
    signature["signature"] = Value::String(digest);
    Ok(signature)
}

pub(crate) fn attach_remote_request_signature(
    payload: &mut Value,
    operation: &str,
    actor: &str,
    target: &str,
    request_key_id: Option<&str>,
) -> Result<(), CliError> {
    if let Some(signature) = sign_remote_request(operation, actor, target, request_key_id)? {
        payload["request_signature"] = signature;
    }
    Ok(())
}

pub(crate) fn verify_remote_request_signature(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    signature: Option<&Value>,
) -> Result<(), CliError> {
    let policy = remote_policy(project_root)?;
    let request_policy = policy.get("request_signatures");
    let required = request_policy
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("required"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Some(signature) = signature else {
        if required {
            return signature_error("signature is required");
        }
        return Ok(());
    };
    let request_policy = request_policy.and_then(Value::as_object).ok_or_else(|| {
        CliError::AuthenticationOrSignature(
            "request signature validation failed: request_signatures policy must be an object"
                .to_string(),
        )
    })?;
    let sig = signature.as_object().ok_or_else(|| {
        CliError::AuthenticationOrSignature(
            "request signature validation failed: signature must be an object".to_string(),
        )
    })?;
    require_sig_string(
        sig,
        "type",
        "signature type is invalid",
        Some("remote_request_signature"),
    )?;
    require_sig_string(
        sig,
        "algorithm",
        "unsupported signature algorithm",
        Some("hmac-sha256"),
    )?;
    require_sig_string(sig, "actor", "actor mismatch", Some(actor))?;
    require_sig_string(sig, "operation", "operation mismatch", Some(operation))?;
    require_sig_string(sig, "target", "target mismatch", Some(target))?;
    let timestamp = required_sig_string(sig, "timestamp", "timestamp is required")?;
    let nonce = required_sig_string(sig, "nonce", "nonce is required")?;
    if nonce.is_empty() {
        return signature_error("nonce is required");
    }
    let digest = required_sig_string(sig, "signature", "signature digest is invalid")?;
    if !is_lower_hex_64(digest) {
        return signature_error("signature digest is invalid");
    }
    let max_skew = request_policy
        .get("max_skew_seconds")
        .and_then(number_as_i64)
        .unwrap_or(300);
    if max_skew < 0 {
        return signature_error("max_skew_seconds must be a number");
    }
    let signed_at = parse_iso_utc_seconds(timestamp)?;
    let now = unix_now_seconds();
    if (now - signed_at).abs() > max_skew {
        return signature_error("timestamp is outside allowed skew");
    }
    let keys = request_policy
        .get("keys")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(
                "request signature validation failed: request_signatures.keys must be an object"
                    .to_string(),
            )
        })?;
    let key_id = sig
        .get("key_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(actor);
    let key_entry = keys
        .get(key_id)
        .or_else(|| keys.get(actor))
        .or_else(|| keys.get("*"));
    let key = hmac_policy_secret(key_entry, actor, "request signature validation failed")?;
    let expected = request_signature_digest(signature, &key)?;
    if !constant_time_eq(expected.as_bytes(), digest.as_bytes()) {
        return signature_error("invalid signature");
    }
    if request_policy
        .get("nonce_cache")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        let ttl = request_policy
            .get("nonce_ttl_seconds")
            .and_then(number_as_i64)
            .unwrap_or(max_skew);
        if ttl <= 0 {
            return signature_error("nonce_ttl_seconds must be positive");
        }
        let max_entries = request_policy
            .get("max_nonce_cache_entries")
            .and_then(number_as_i64)
            .unwrap_or(10_000);
        if max_entries <= 0 {
            return signature_error("max_nonce_cache_entries must be positive");
        }
        record_request_nonce(project_root, signature, ttl, max_entries as usize)?;
    }
    Ok(())
}

fn verify_commit_policy_recursive(
    objects_root: &Path,
    commit_id: &str,
    keys: &Map<String, Value>,
    require_history: bool,
    is_head: bool,
    seen: &mut BTreeSet<String>,
) -> Result<(), CliError> {
    if !seen.insert(commit_id.to_string()) {
        return Ok(());
    }
    let commit = codefire_store::read_object(objects_root, commit_id)?;
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if is_head || commit.get("signature").is_some() || !parents.is_empty() {
        verify_commit_signature(&commit, keys, true)?;
    }
    if require_history {
        for parent in parents {
            if let Some(parent_id) = parent.as_str().filter(|id| id.starts_with("CF-COMMIT-")) {
                verify_commit_policy_recursive(objects_root, parent_id, keys, true, false, seen)?;
            }
        }
    }
    Ok(())
}

fn verify_commit_signature(
    commit: &Value,
    keys: &Map<String, Value>,
    required: bool,
) -> Result<(), CliError> {
    let Some(signature) = commit.get("signature") else {
        if required {
            return commit_error("signature is required");
        }
        return Ok(());
    };
    let sig = signature.as_object().ok_or_else(|| {
        CliError::AuthenticationOrSignature(
            "commit signature validation failed: signature must be an object".to_string(),
        )
    })?;
    require_commit_string(
        sig,
        "type",
        "signature type is invalid",
        Some("commit_signature"),
    )?;
    require_commit_string(
        sig,
        "algorithm",
        "unsupported signature algorithm",
        Some("hmac-sha256"),
    )?;
    let signer = required_commit_string(sig, "signer", "signer is required")?;
    let key_id = sig
        .get("key_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(signer);
    let digest = required_commit_string(sig, "signature", "signature digest is invalid")?;
    if !is_lower_hex_64(digest) {
        return commit_error("signature digest is invalid");
    }
    let key_entry = keys
        .get(key_id)
        .or_else(|| keys.get(signer))
        .or_else(|| keys.get("*"));
    let key = commit_signature_key_secret(signature, key_entry, signer)?;
    let expected = commit_signature_digest(commit, signature, &key)?;
    if !constant_time_eq(expected.as_bytes(), digest.as_bytes()) {
        return commit_error("invalid signature");
    }
    Ok(())
}

fn commit_signature_key_secret(
    signature: &Value,
    key_entry: Option<&Value>,
    signer: &str,
) -> Result<String, CliError> {
    if let Some(secret) = key_entry.and_then(Value::as_str) {
        return Ok(secret.to_string());
    }
    let key = key_entry.and_then(Value::as_object).ok_or_else(|| {
        CliError::AuthenticationOrSignature(format!(
            "commit signature validation failed: no key for signer '{signer}'"
        ))
    })?;
    check_key_status(key, "commit signature validation failed")?;
    check_allowed_signers(key, signer, "commit signature validation failed")?;
    let signed_at = signature
        .get("signed_at")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(
                "commit signature validation failed: signed_at is required".to_string(),
            )
        })?;
    let signed_at_seconds = parse_commit_signature_time(signed_at, "signed_at")?;
    if let Some(not_before) = key
        .get("not_before")
        .or_else(|| key.get("valid_from"))
        .and_then(Value::as_str)
    {
        let not_before = parse_commit_signature_time(not_before, "not_before")?;
        if signed_at_seconds < not_before {
            return commit_error("key is not valid yet");
        }
    }
    if let Some(not_after) = key
        .get("not_after")
        .or_else(|| key.get("valid_until"))
        .and_then(Value::as_str)
    {
        let not_after = parse_commit_signature_time(not_after, "not_after")?;
        if signed_at_seconds > not_after {
            return commit_error("key is expired");
        }
    }
    key_secret(key, "commit signature validation failed")
}

fn hmac_policy_secret(
    key_entry: Option<&Value>,
    signer: &str,
    prefix: &str,
) -> Result<String, CliError> {
    if let Some(secret) = key_entry.and_then(Value::as_str) {
        return Ok(secret.to_string());
    }
    let key = key_entry.and_then(Value::as_object).ok_or_else(|| {
        CliError::AuthenticationOrSignature(format!("{prefix}: no key for signer '{signer}'"))
    })?;
    check_key_status(key, prefix)?;
    check_allowed_signers(key, signer, prefix)?;
    key_secret(key, prefix)
}

fn check_key_status(key: &Map<String, Value>, prefix: &str) -> Result<(), CliError> {
    let status = key
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("active")
        .to_ascii_lowercase();
    if status == "revoked"
        || status == "disabled"
        || key.get("revoked").and_then(Value::as_bool) == Some(true)
    {
        return Err(CliError::AuthenticationOrSignature(format!(
            "{prefix}: key is revoked"
        )));
    }
    if key.get("active").and_then(Value::as_bool) == Some(false) {
        return Err(CliError::AuthenticationOrSignature(format!(
            "{prefix}: key is inactive"
        )));
    }
    Ok(())
}

fn check_allowed_signers(
    key: &Map<String, Value>,
    signer: &str,
    prefix: &str,
) -> Result<(), CliError> {
    let Some(allowed) = key.get("signers") else {
        return Ok(());
    };
    let allowed = if let Some(value) = allowed.as_str() {
        vec![value]
    } else if let Some(values) = allowed.as_array() {
        values.iter().filter_map(Value::as_str).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if allowed
        .iter()
        .any(|value| *value == "*" || *value == signer)
    {
        return Ok(());
    }
    Err(CliError::AuthenticationOrSignature(format!(
        "{prefix}: key is not allowed for signer '{signer}'"
    )))
}

fn key_secret(key: &Map<String, Value>, prefix: &str) -> Result<String, CliError> {
    key.get("secret")
        .or_else(|| key.get("key"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(format!("{prefix}: key secret is missing"))
        })
}

fn commit_signature_digest(
    commit: &Value,
    signature: &Value,
    key: &str,
) -> Result<String, CliError> {
    hmac_hex(key, &commit_signature_input(commit, signature)?)
}

fn request_signature_digest(signature: &Value, key: &str) -> Result<String, CliError> {
    hmac_hex(
        key,
        &codefire_store::canonical_json(&signature_without_digest(signature))?,
    )
}

fn commit_signature_input(commit: &Value, signature: &Value) -> Result<Vec<u8>, CliError> {
    Ok(codefire_store::canonical_json(&json!({
        "commit": unsigned_commit_payload(commit),
        "signature": signature_without_digest(signature),
    }))?)
}

fn unsigned_commit_payload(commit: &Value) -> Value {
    let mut unsigned = commit.clone();
    if let Some(object) = unsigned.as_object_mut() {
        object.remove("signature");
    }
    unsigned
}

fn signature_without_digest(signature: &Value) -> Value {
    let mut metadata = signature.clone();
    if let Some(object) = metadata.as_object_mut() {
        object.remove("signature");
    }
    metadata
}

fn hmac_hex(key: &str, input: &[u8]) -> Result<String, CliError> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| {
        CliError::AuthenticationOrSignature("signature validation failed: invalid key".to_string())
    })?;
    mac.update(input);
    Ok(codefire_util::hex_lower(&mac.finalize().into_bytes()))
}

fn record_request_nonce(
    project_root: &Path,
    signature: &Value,
    ttl_seconds: i64,
    max_entries: usize,
) -> Result<(), CliError> {
    ensure_remote_layout(project_root)?;
    let dirs = remote_dirs(project_root);
    let _lock = FileLock::acquire_with_options(
        dirs.locks.join("request-nonce-cache.lock"),
        &LockOptions::default(),
    )?;
    let now = unix_now_seconds();
    let cache_path = project_root.join("request_nonce_cache.json");
    let mut cache =
        read_optional_json(&cache_path)?.unwrap_or_else(|| json!({"version": 1, "entries": {}}));
    let mut entries = cache
        .get("entries")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let before_prune = entries.len();
    let cutoff = now - ttl_seconds;
    entries.retain(|_, value| {
        value
            .get("seen_at")
            .and_then(Value::as_i64)
            .unwrap_or_default()
            >= cutoff
    });
    let mut pruned_count = before_prune.saturating_sub(entries.len());
    let cache_key = request_nonce_cache_key(signature)?;
    if entries.contains_key(&cache_key) {
        return signature_error("nonce has already been used");
    }
    entries.insert(
        cache_key,
        json!({
            "actor": signature.get("actor").cloned().unwrap_or(Value::Null),
            "operation": signature.get("operation").cloned().unwrap_or(Value::Null),
            "target": signature.get("target").cloned().unwrap_or(Value::Null),
            "timestamp": signature.get("timestamp").cloned().unwrap_or(Value::Null),
            "nonce": signature.get("nonce").cloned().unwrap_or(Value::Null),
            "seen_at": now,
        }),
    );
    while entries.len() > max_entries {
        let oldest_key = entries
            .iter()
            .min_by_key(|(_, value)| {
                value
                    .get("seen_at")
                    .and_then(Value::as_i64)
                    .unwrap_or_default()
            })
            .map(|(key, _)| key.clone());
        if let Some(oldest_key) = oldest_key {
            entries.remove(&oldest_key);
            pruned_count += 1;
        } else {
            break;
        }
    }
    let entry_count = entries.len();
    cache["entries"] = Value::Object(entries);
    cache["updated_at"] = Value::String(now_iso_utc());
    cache["ttl_seconds"] = Value::Number(ttl_seconds.into());
    cache["max_entries"] = Value::Number((max_entries as u64).into());
    cache["entry_count"] = Value::Number((entry_count as u64).into());
    cache["last_pruned_count"] = Value::Number((pruned_count as u64).into());
    write_json_atomic(&cache_path, &cache)
}

fn request_nonce_cache_key(signature: &Value) -> Result<String, CliError> {
    let payload = json!({
        "actor": signature.get("actor").cloned().unwrap_or(Value::Null),
        "operation": signature.get("operation").cloned().unwrap_or(Value::Null),
        "target": signature.get("target").cloned().unwrap_or(Value::Null),
        "timestamp": signature.get("timestamp").cloned().unwrap_or(Value::Null),
        "nonce": signature.get("nonce").cloned().unwrap_or(Value::Null),
        "signature": signature.get("signature").cloned().unwrap_or(Value::Null),
    });
    Ok(codefire_util::sha256_hex(&codefire_store::canonical_json(
        &payload,
    )?))
}

fn remote_policy(project_root: &Path) -> Result<Value, CliError> {
    read_optional_json(&project_root.join("server_policy.json"))
        .map(|value| value.unwrap_or_else(|| json!({})))
}

fn require_sig_string(
    sig: &Map<String, Value>,
    field: &str,
    error: &str,
    expected: Option<&str>,
) -> Result<(), CliError> {
    let value = required_sig_string(sig, field, error)?;
    if expected.is_some_and(|expected| value != expected) {
        return signature_error(error);
    }
    Ok(())
}

fn required_sig_string<'a>(
    sig: &'a Map<String, Value>,
    field: &str,
    error: &str,
) -> Result<&'a str, CliError> {
    sig.get(field).and_then(Value::as_str).ok_or_else(|| {
        CliError::AuthenticationOrSignature(format!("request signature validation failed: {error}"))
    })
}

fn require_commit_string(
    sig: &Map<String, Value>,
    field: &str,
    error: &str,
    expected: Option<&str>,
) -> Result<(), CliError> {
    let value = required_commit_string(sig, field, error)?;
    if expected.is_some_and(|expected| value != expected) {
        return commit_error(error);
    }
    Ok(())
}

fn required_commit_string<'a>(
    sig: &'a Map<String, Value>,
    field: &str,
    error: &str,
) -> Result<&'a str, CliError> {
    sig.get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(format!(
                "commit signature validation failed: {error}"
            ))
        })
}

fn commit_error<T>(message: &str) -> Result<T, CliError> {
    Err(CliError::AuthenticationOrSignature(format!(
        "commit signature validation failed: {message}"
    )))
}

fn signature_error<T>(message: &str) -> Result<T, CliError> {
    Err(CliError::AuthenticationOrSignature(format!(
        "request signature validation failed: {message}"
    )))
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

fn random_nonce() -> Result<String, CliError> {
    let mut bytes = [0u8; 12];
    getrandom(&mut bytes).map_err(|error| {
        CliError::AuthenticationOrSignature(format!("request signature generation failed: {error}"))
    })?;
    Ok(base64_url_no_pad(&bytes))
}

fn base64_url_no_pad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let value = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[(value & 0x3f) as usize] as char);
        }
    }
    out
}

fn number_as_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_f64().map(|value| value as i64))
}

fn unix_now_seconds() -> i64 {
    codefire_util::unix_now_seconds()
}

fn parse_iso_utc_seconds(value: &str) -> Result<i64, CliError> {
    codefire_util::parse_iso_utc_seconds(value).ok_or_else(|| {
        CliError::AuthenticationOrSignature(
            "request signature validation failed: timestamp is required".to_string(),
        )
    })
}

fn parse_commit_signature_time(value: &str, field: &str) -> Result<i64, CliError> {
    codefire_util::parse_iso_utc_seconds(value).ok_or_else(|| {
        CliError::AuthenticationOrSignature(format!(
            "commit signature validation failed: {field} must be UTC seconds ending in Z"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn base64_url_nonce_has_no_padding() {
        assert_eq!(base64_url_no_pad(&[0, 1, 2]), "AAEC");
        assert_eq!(base64_url_no_pad(&[0, 1]), "AAE");
    }

    #[test]
    fn iso_utc_parser_matches_unix_epoch() {
        assert_eq!(parse_iso_utc_seconds("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            parse_iso_utc_seconds("1970-01-02T00:00:01Z").unwrap(),
            86_401
        );
        assert!(parse_iso_utc_seconds("1970-01-01T00:00:00+00:00").is_err());
    }

    #[test]
    fn commit_signature_verifies_key_policy_and_rotation_bounds() {
        let commit = json!({
            "type": "commit",
            "version": 1,
            "parents": [],
            "message": "Signed",
            "roots": {},
            "certificate": {"result": "consistent"},
            "changed_atoms": [],
            "extinguished_fires": [],
            "created_at": "2026-06-05T00:00:00Z",
        });
        let signed =
            sign_commit_payload(commit, "alice", "alice-signing-key", "alice-2026-06").unwrap();
        let keys = json!({
            "alice-2026-06": {
                "secret": "alice-signing-key",
                "signers": ["alice"],
                "not_before": "2026-06-01T00:00:00Z",
                "not_after": "2026-07-01T00:00:00Z"
            }
        });
        verify_commit_signature(&signed, keys.as_object().unwrap(), true).unwrap();

        let revoked = json!({
            "alice-2026-06": {
                "secret": "alice-signing-key",
                "signers": ["alice"],
                "status": "revoked"
            }
        });
        let error =
            verify_commit_signature(&signed, revoked.as_object().unwrap(), true).unwrap_err();
        assert_eq!(
            error.exit_code(),
            ExitCode::AuthenticationOrSignatureFailure.code()
        );
        assert!(error.to_string().contains("key is revoked"));

        let invalid_window = json!({
            "alice-2026-06": {
                "secret": "alice-signing-key",
                "signers": ["alice"],
                "not_before": "2026-06-01T00:00:00+00:00"
            }
        });
        let invalid_window_error =
            verify_commit_signature(&signed, invalid_window.as_object().unwrap(), true)
                .unwrap_err();
        assert!(invalid_window_error
            .to_string()
            .contains("not_before must be UTC seconds ending in Z"));
    }

    #[test]
    fn request_signature_rejects_old_timestamp_and_replayed_nonce() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "max_skew_seconds": 300,
                    "nonce_ttl_seconds": 300,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"]
                        }
                    }
                }
            }),
        )
        .unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            Some(&signature),
        )
        .unwrap();
        let replay = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            Some(&signature),
        )
        .unwrap_err();
        assert!(replay.to_string().contains("nonce has already been used"));

        let mut old = signature.clone();
        old["timestamp"] = Value::String("1970-01-01T00:00:00Z".to_string());
        let digest = request_signature_digest(&old, "alice-request-key").unwrap();
        old["signature"] = Value::String(digest);
        let error =
            verify_remote_request_signature(&project_root, "upload", "alice", &target, Some(&old))
                .unwrap_err();
        assert!(error
            .to_string()
            .contains("timestamp is outside allowed skew"));
    }

    #[test]
    fn request_nonce_cache_prunes_to_policy_limit_and_reports_metadata() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "max_skew_seconds": 300,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 1,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"]
                        }
                    }
                }
            }),
        )
        .unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        for _ in 0..2 {
            let signature = sign_remote_request_with_key(
                "upload",
                "alice",
                &target,
                "alice-request",
                "alice-request-key",
            )
            .unwrap();
            verify_remote_request_signature(
                &project_root,
                "upload",
                "alice",
                &target,
                Some(&signature),
            )
            .unwrap();
        }

        let cache = read_json(&project_root.join("request_nonce_cache.json")).unwrap();
        assert_eq!(cache["entry_count"], 1);
        assert_eq!(cache["max_entries"], 1);
        assert_eq!(cache["ttl_seconds"], 300);
        assert_eq!(cache["last_pruned_count"], 1);
        assert_eq!(cache["entries"].as_object().unwrap().len(), 1);
    }
}
