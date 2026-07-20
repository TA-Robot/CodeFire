use super::remote::{
    ensure_remote_layout, read_optional_json, remote_dirs, AuthorityWrite, RemoteAuthority,
};
use super::*;
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use serde_json::{json, Map, Value};
use sha2::Sha256;
#[cfg(test)]
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn commit_signing_key(require_signature: bool) -> Result<Option<String>, CliError> {
    let signing_key = configured_commit_signing_key();
    if signing_key.is_none() && require_signature {
        return Err(CliError::Usage(
            "commit blocked: commit signature is required but CODEFIRE_SIGNING_KEY is not set"
                .to_string(),
        ));
    }
    Ok(signing_key)
}

pub(crate) fn maybe_sign_commit(
    commit: Value,
    signer: Option<&str>,
    key_id: Option<&str>,
    signing_key: Option<&str>,
) -> Result<Value, CliError> {
    let Some(signing_key) = signing_key else {
        return Ok(commit);
    };
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
    sign_commit_payload(commit, &signer, signing_key, &key_id)
}

/// Verifies a locally constructed candidate with the same trusted key that
/// produced it. Store sealing validation intentionally does not authenticate
/// signatures, so callers must cross this boundary before publishing any
/// mutable reference to the candidate.
pub(crate) fn verify_local_commit_signature(
    commit: &Value,
    signing_key: Option<&str>,
    required: bool,
) -> Result<(), CliError> {
    let mut keys = Map::new();
    if let Some(signing_key) = signing_key {
        keys.insert("*".to_string(), Value::String(signing_key.to_string()));
    }
    verify_commit_signature(commit, &keys, required)
}

fn configured_commit_signing_key() -> Option<String> {
    #[cfg(test)]
    if let Some(signing_key) = TEST_COMMIT_SIGNING_KEY_OVERRIDE.with(|value| value.borrow().clone())
    {
        return signing_key.filter(|value| !value.is_empty());
    }

    env::var("CODEFIRE_SIGNING_KEY")
        .ok()
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
thread_local! {
    static TEST_COMMIT_SIGNING_KEY_OVERRIDE: RefCell<Option<Option<String>>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) struct CommitSigningKeyOverride {
    previous: Option<Option<String>>,
}

#[cfg(test)]
impl Drop for CommitSigningKeyOverride {
    fn drop(&mut self) {
        TEST_COMMIT_SIGNING_KEY_OVERRIDE.with(|value| {
            value.replace(self.previous.take());
        });
    }
}

#[cfg(test)]
pub(crate) fn override_commit_signing_key_for_test(
    signing_key: Option<&str>,
) -> CommitSigningKeyOverride {
    let previous = TEST_COMMIT_SIGNING_KEY_OVERRIDE
        .with(|value| value.replace(Some(signing_key.map(str::to_string))));
    CommitSigningKeyOverride { previous }
}

pub(crate) fn sign_commit_payload(
    commit: Value,
    signer: &str,
    key: &str,
    key_id: &str,
) -> Result<Value, CliError> {
    sign_commit_payload_at(commit, signer, key, key_id, &now_iso_utc())
}

fn sign_commit_payload_at(
    mut commit: Value,
    signer: &str,
    key: &str,
    key_id: &str,
    signed_at: &str,
) -> Result<Value, CliError> {
    let mut signature = json!({
        "type": "commit_signature",
        "version": 1,
        "algorithm": "hmac-sha256",
        "signer": signer,
        "key_id": key_id,
        "signed_at": signed_at,
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
    verify_commit_policy_iterative(
        objects_root,
        commit_id,
        keys,
        require_history,
        true,
        &mut seen,
    )
}

pub(crate) fn enforce_remote_commit_signature_policy_in_provisional_subject_with_poll<F>(
    policy: Option<&Value>,
    subject: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    commit_id: &str,
    mut poll: F,
) -> Result<(), CliError>
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
            CliError::InvalidRepository("commit signature retained-byte overflow".to_string())
        })?;
        ledger
            .apply(reservation.acquire(codefire_core::DecisionWorkDimension::RetainedBytes, bytes))
            .map_err(|error| nonce_work_error(error, "retain commit-signature traversal"))?;
        *owned = next;
        Ok(())
    }

    fn release(
        ledger: &mut codefire_core::DecisionWorkLedger,
        owned: &mut u64,
        bytes: u64,
    ) -> Result<(), CliError> {
        let next = owned.checked_sub(bytes).ok_or_else(|| {
            CliError::InvalidRepository("commit signature retained-byte underflow".to_string())
        })?;
        ledger
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, bytes)
            .map_err(|error| nonce_work_error(error, "release commit-signature traversal"))?;
        *owned = next;
        Ok(())
    }

    let Some(signature_policy) = policy.and_then(|policy| policy.get("commit_signatures")) else {
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
    subject.ensure_ledger(ledger).map_err(CliError::from)?;

    let mut retained_owned = 0_u64;
    let result = (|| {
        let root_bytes = codefire_core::logical_retained_frame_bytes(commit_id)
            .map_err(|error| CliError::from(codefire_store::StoreError::from(error)))?;
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
        let mut pending = vec![(commit_id.to_string(), true, 0_u64)];
        let mut seen = BTreeSet::new();
        while let Some((commit_id, is_head, depth)) = pending.pop() {
            poll()?;
            let frame_bytes = codefire_core::logical_retained_frame_bytes(&commit_id)
                .map_err(|error| CliError::from(codefire_store::StoreError::from(error)))?;
            if seen.contains(&commit_id) {
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Query,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .count(codefire_core::DecisionWorkCounter::Revisits, 1),
                    )
                    .map_err(|error| nonce_work_error(error, "count commit-signature revisit"))?;
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
                .map_err(|error| nonce_work_error(error, "count commit-signature graph node"))?;
            let commit = subject
                .retained_commit_payload(&commit_id)
                .map_err(CliError::from)?;
            let parents = commit
                .get("parents")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    CliError::InvalidRepository(format!(
                        "validated commit parents are missing: {commit_id}"
                    ))
                })?;
            let verify_required =
                is_head || commit.get("signature").is_some() || !parents.is_empty();
            if verify_required {
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::SemanticValidation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1),
                    )
                    .map_err(|error| {
                        nonce_work_error(error, "count commit-signature validation")
                    })?;
                verify_commit_signature_with_ledger(commit, keys, true, ledger)?;
            }
            if require_history {
                let next_depth = depth.checked_add(1).ok_or_else(|| {
                    CliError::InvalidRepository(
                        "commit signature history depth overflow".to_string(),
                    )
                })?;
                for parent in parents.iter().rev() {
                    let Some(parent_id) = parent
                        .as_str()
                        .filter(|parent_id| parent_id.starts_with("CF-COMMIT-"))
                    else {
                        continue;
                    };
                    let retained_bytes = codefire_core::logical_retained_frame_bytes(parent_id)
                        .map_err(|error| CliError::from(codefire_store::StoreError::from(error)))?;
                    let next_frontier = u64::try_from(pending.len())
                        .ok()
                        .and_then(|frontier| frontier.checked_add(1))
                        .ok_or_else(|| {
                            CliError::InvalidRepository(
                                "commit signature history frontier overflow".to_string(),
                            )
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
                    pending.push((parent_id.to_string(), false, next_depth));
                }
            }
            drop(commit_id);
            release(ledger, &mut retained_owned, frame_bytes)?;
        }
        Ok(())
    })();
    let cleanup = if retained_owned == 0 {
        Ok(())
    } else {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_owned,
            )
            .map_err(|error| nonce_work_error(error, "release commit-signature query"))
    };
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
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
    payload: &Value,
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
    sign_remote_request_with_key(operation, actor, target, payload, &key_id, &key).map(Some)
}

pub(crate) fn sign_remote_request_with_key(
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    key_id: &str,
    key: &str,
) -> Result<Value, CliError> {
    let timestamp = now_iso_utc();
    let nonce = random_nonce()?;
    sign_remote_request_with_key_at(
        operation,
        actor,
        target,
        payload,
        key_id,
        key,
        RequestFreshness {
            timestamp: &timestamp,
            nonce: &nonce,
        },
    )
}

struct RequestFreshness<'a> {
    timestamp: &'a str,
    nonce: &'a str,
}

fn sign_remote_request_with_key_at(
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    key_id: &str,
    key: &str,
    freshness: RequestFreshness<'_>,
) -> Result<Value, CliError> {
    let payload_digest = remote_request_payload_digest(payload)?;
    let idempotency_key = normalized_idempotency_key(payload);
    let mut signature = json!({
        "type": "remote_request_signature",
        "version": 2,
        "algorithm": "hmac-sha256",
        "subject_type": "codefire_remote_mutation_subject",
        "subject_version": 1,
        "actor": actor,
        "operation": operation,
        "target": target,
        "key_id": key_id,
        "timestamp": freshness.timestamp,
        "nonce": freshness.nonce,
        "payload_digest": payload_digest,
        "idempotency_key": idempotency_key,
    });
    signature["subject_digest"] = Value::String(remote_request_subject_digest(&signature)?);
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
    if let Some(signature) = sign_remote_request(operation, actor, target, payload, request_key_id)?
    {
        payload["request_signature"] = signature;
    }
    Ok(())
}

pub(crate) fn verify_remote_request_signature(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
) -> Result<(), CliError> {
    validate_remote_request_signature(project_root, operation, actor, target, payload, signature)?
        .publish(project_root)
}

#[derive(Debug)]
pub(crate) struct PendingRequestNonce {
    prepared: Option<PreparedRequestNonce>,
    retained_accounted: bool,
}

#[derive(Debug)]
struct PreparedRequestNonce {
    expected_preimage_hash: Option<String>,
    postimage_bytes: Vec<u8>,
    actor: String,
    key_id: String,
    nonce: String,
}

#[cfg(test)]
pub(crate) struct ReservedRequestNonce {
    prepared: PendingRequestNonce,
    permit: Option<codefire_core::DecisionWorkPermit>,
    reservation: codefire_core::DecisionWorkReservation,
    retained_bytes: u64,
}

impl PendingRequestNonce {
    fn disabled() -> Self {
        Self {
            prepared: None,
            retained_accounted: false,
        }
    }

    pub(crate) fn recovered_from_matching_publication() -> Self {
        Self::disabled()
    }

    pub(crate) fn publish(self, project_root: &Path) -> Result<(), CliError> {
        match self.prepared {
            Some(prepared) => publish_prepared_request_nonce(project_root, prepared),
            None => Ok(()),
        }
    }

    pub(crate) fn retained_bytes(&self) -> Result<u64, CliError> {
        self.prepared.as_ref().map_or(Ok(0), |prepared| {
            u64::try_from(prepared.postimage_bytes.len()).map_err(|_| {
                CliError::InvalidRepository("nonce authority write size overflow".to_string())
            })
        })
    }

    pub(crate) fn has_authority_write(&self) -> bool {
        self.prepared.is_some()
    }

    /// Converts a validated nonce into the ordinary remote authority write
    /// that must be included in the caller's main publication transaction.
    pub(crate) fn authority_write(
        &self,
        before: Option<Value>,
    ) -> Result<Option<AuthorityWrite>, CliError> {
        let Some(prepared) = self.prepared.as_ref() else {
            return Ok(None);
        };
        let actual_preimage_hash = before
            .as_ref()
            .map(codefire_store::canonical_json)
            .transpose()?
            .map(|bytes| codefire_util::sha256_hex(&bytes));
        if actual_preimage_hash != prepared.expected_preimage_hash {
            return signature_error("nonce cache changed after validation; retry the request");
        }
        let after = serde_json::from_slice(&prepared.postimage_bytes)?;
        Ok(Some(AuthorityWrite {
            authority: RemoteAuthority::RequestNonceCache,
            before,
            after,
        }))
    }

    /// Releases validation's exact encoded nonce ownership after a main
    /// publication has acquired ownership of its complete plan.
    pub(crate) fn release_retained(
        self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let retained_bytes = self.retained_bytes()?;
        if self.retained_accounted && retained_bytes > 0 {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(|error| nonce_work_error(error, "release merged nonce authority write"))?;
        }
        Ok(())
    }

    /// Reserves the exact canonical nonce-cache postimage so callers can join
    /// nonce publication to the same prepublication seal as another mutation.
    #[cfg(test)]
    pub(crate) fn reserve(
        self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<ReservedRequestNonce, CliError> {
        let retained_bytes = self.retained_bytes()?;
        let reservation = nonce_publication_reservation(retained_bytes);
        let permit = if self.prepared.is_some() {
            if !self.retained_accounted {
                ledger
                    .apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::PublicationPreflight,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained_bytes,
                        ),
                    )
                    .map_err(|error| {
                        nonce_work_error(error, "acquire nonce postimage ownership")
                    })?;
            }
            match ledger.reserve(reservation.clone()) {
                Ok(permit) => Some(permit),
                Err(error) => {
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained_bytes,
                        )
                        .map_err(|release| {
                            nonce_work_error(release, "release rejected nonce postimage")
                        })?;
                    return Err(nonce_work_error(error, "reserve nonce publication"));
                }
            }
        } else {
            None
        };
        Ok(ReservedRequestNonce {
            prepared: self,
            permit,
            reservation,
            retained_bytes,
        })
    }
}

#[cfg(test)]
impl ReservedRequestNonce {
    pub(crate) fn publish(
        mut self,
        project_root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if let Some(permit) = self.permit.as_mut() {
            ledger
                .consume_reserved(permit, self.reservation.clone())
                .map_err(|error| nonce_work_error(error, "consume nonce publication permit"))?;
        }
        let result = self.prepared.publish(project_root);
        if self.retained_bytes > 0 {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    self.retained_bytes,
                )
                .map_err(|error| nonce_work_error(error, "release nonce postimage"))?;
        }
        result
    }
}

pub(crate) fn validate_remote_request_signature(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
) -> Result<PendingRequestNonce, CliError> {
    validate_remote_request_signature_with_layout(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        true,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_remote_request_signature_with_layout(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ensure_layout: bool,
    prepare_nonce: bool,
) -> Result<PendingRequestNonce, CliError> {
    let policy = remote_policy(project_root)?;
    let policy = policy.as_object().ok_or_else(|| {
        CliError::AuthenticationOrSignature(
            "request signature validation failed: server policy must be an object".to_string(),
        )
    })?;
    let request_policy = match policy.get("request_signatures") {
        Some(policy) => policy.as_object().ok_or_else(|| {
            CliError::AuthenticationOrSignature(
                "request signature validation failed: request_signatures policy must be an object"
                    .to_string(),
            )
        })?,
        None if signature.is_none() => return Ok(PendingRequestNonce::disabled()),
        None => {
            return signature_error("request_signatures policy must be an object");
        }
    };
    let required = match request_policy.get("required") {
        Some(Value::Bool(required)) => *required,
        Some(_) => return signature_error("request_signatures.required must be a boolean"),
        None => false,
    };
    let Some(signature) = signature else {
        if required {
            return signature_error("signature is required");
        }
        return Ok(PendingRequestNonce::disabled());
    };
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
    if sig.get("version").and_then(Value::as_u64) != Some(2) {
        return signature_error("signature version must be 2 (payload-bound subject)");
    }
    require_sig_string(
        sig,
        "subject_type",
        "signature subject type is invalid",
        Some("codefire_remote_mutation_subject"),
    )?;
    if sig.get("subject_version").and_then(Value::as_u64) != Some(1) {
        return signature_error("signature subject version is invalid");
    }
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
    let max_skew = optional_nonnegative_number(request_policy, "max_skew_seconds", 300.0)?;
    let signed_at = parse_iso_utc_seconds(timestamp)?;
    let now = unix_now_seconds();
    if ((now as f64) - (signed_at as f64)).abs() > max_skew {
        return signature_error("timestamp is outside allowed skew");
    }
    let payload_digest = required_sig_string(sig, "payload_digest", "payload digest is required")?;
    let expected_payload_digest = remote_request_payload_digest(payload)?;
    if payload_digest != expected_payload_digest {
        return signature_error("payload digest mismatch");
    }
    if sig.get("idempotency_key").cloned().unwrap_or(Value::Null)
        != normalized_idempotency_key(payload)
    {
        return signature_error("idempotency key mismatch");
    }
    let subject_digest = required_sig_string(sig, "subject_digest", "subject digest is required")?;
    if !is_sha256_label(subject_digest)
        || subject_digest != remote_request_subject_digest(signature)?
    {
        return signature_error("subject digest is invalid");
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
    let key = hmac_policy_secret_at(
        key_entry,
        actor,
        signed_at,
        "request signature validation failed",
    )?;
    let expected = request_signature_digest(signature, &key)?;
    if !constant_time_eq(expected.as_bytes(), digest.as_bytes()) {
        return signature_error("invalid signature");
    }
    if request_policy
        .get("nonce_cache")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        let ttl = optional_nonnegative_number(request_policy, "nonce_ttl_seconds", max_skew)?;
        if ttl <= 0.0 {
            return signature_error("nonce_ttl_seconds must be positive");
        }
        let max_entries =
            optional_positive_usize(request_policy, "max_nonce_cache_entries", 10_000)?;
        if !prepare_nonce {
            return Ok(PendingRequestNonce::disabled());
        }
        return Ok(PendingRequestNonce {
            prepared: Some(prepare_request_nonce(
                project_root,
                signature,
                ttl,
                max_entries,
                ensure_layout,
            )?),
            retained_accounted: false,
        });
    }
    Ok(PendingRequestNonce::disabled())
}

/// Variant for callers that already hold the exclusive project gate. It must
/// not attempt the layout bootstrap path because that reacquires the gate.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_remote_request_signature_prelocked_with_work_and_poll<F>(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: F,
) -> Result<PendingRequestNonce, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    validate_remote_request_signature_with_layout_work_and_poll(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ledger,
        false,
        true,
        true,
        poll,
    )
}

/// Verifies request-signature policy, subject binding, freshness, and HMAC
/// without reading or preparing the nonce-cache authority. This lets callers
/// authenticate before observable request-specific validation while deferring
/// nonce ownership to their single atomic publication transaction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn preflight_remote_request_signature_with_work_and_poll<F>(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    validate_remote_request_signature_with_layout_work_and_poll(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ledger,
        false,
        false,
        false,
        poll,
    )
    .map(drop)
}

/// Validates nonce-cache semantics while returning no retained nonce owner.
/// Publication-aware callers use this before work that may expose an early
/// rejection, then perform the owning validation once their transaction owner
/// is ready.
#[allow(clippy::too_many_arguments)]
pub(crate) fn preflight_remote_request_nonce_prelocked_with_work_and_poll<F>(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    validate_remote_request_signature_with_layout_work_and_poll(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ledger,
        false,
        true,
        false,
        poll,
    )
    .map(drop)
}

#[allow(clippy::too_many_arguments)]
fn validate_remote_request_signature_with_layout_work_and_poll<F>(
    project_root: &Path,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    ensure_layout: bool,
    prepare_nonce: bool,
    retain_nonce: bool,
    mut poll: F,
) -> Result<PendingRequestNonce, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    poll()?;
    let policy_path = project_root.join("server_policy.json");
    let nonce_path = project_root.join("request_nonce_cache.json");
    let mut physical_bytes = 0_u64;
    let mut files = 0_u64;
    for path in
        std::iter::once(policy_path.as_path()).chain(prepare_nonce.then_some(nonce_path.as_path()))
    {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) => {
                files = files.checked_add(1).ok_or_else(|| {
                    CliError::InvalidRepository("signature file count overflow".to_string())
                })?;
                physical_bytes = physical_bytes.checked_add(metadata.len()).ok_or_else(|| {
                    CliError::InvalidRepository("signature input byte size overflow".to_string())
                })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    let scratch_bytes = physical_bytes
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(16 * 1024))
        .ok_or_else(|| {
            CliError::InvalidRepository("signature validation scratch size overflow".to_string())
        })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                physical_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch_bytes,
            )
            .count(codefire_core::DecisionWorkCounter::FilesRead, files)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                physical_bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::BytesHashed,
                physical_bytes,
            ),
        )
        .map_err(|error| nonce_work_error(error, "reserve signature validation reads"))?;
    let validated = validate_remote_request_signature_with_layout(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ensure_layout,
        prepare_nonce,
    );
    let mut pending = match validated {
        Ok(pending) => pending,
        Err(error) => {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    scratch_bytes,
                )
                .map_err(|release| {
                    nonce_work_error(release, "release failed signature validation")
                })?;
            return Err(error);
        }
    };
    if let Err(error) = poll() {
        drop(pending);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch_bytes,
            )
            .map_err(|release| {
                nonce_work_error(release, "release cancelled signature validation")
            })?;
        return Err(error);
    }
    let retained_bytes = if retain_nonce {
        match pending.retained_bytes() {
            Ok(bytes) => bytes,
            Err(error) => {
                drop(pending);
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        scratch_bytes,
                    )
                    .map_err(|release| {
                        nonce_work_error(release, "release invalid signature validation")
                    })?;
                return Err(error);
            }
        }
    } else {
        0
    };
    if retained_bytes > 0 {
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        ) {
            drop(pending);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    scratch_bytes,
                )
                .map_err(|release| {
                    nonce_work_error(release, "release rejected nonce validation")
                })?;
            return Err(nonce_work_error(error, "retain prepared nonce postimage"));
        }
        pending.retained_accounted = true;
    }
    if !retain_nonce {
        drop(pending);
        pending = PendingRequestNonce::disabled();
    }
    if let Err(error) = ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        scratch_bytes,
    ) {
        drop(pending);
        if retained_bytes > 0 {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(|release| {
                    nonce_work_error(release, "release failed retained nonce postimage")
                })?;
        }
        return Err(nonce_work_error(
            error,
            "release signature validation scratch",
        ));
    }
    Ok(pending)
}

fn verify_commit_policy_iterative(
    objects_root: &Path,
    commit_id: &str,
    keys: &Map<String, Value>,
    require_history: bool,
    is_head: bool,
    seen: &mut BTreeSet<String>,
) -> Result<(), CliError> {
    verify_commit_policy_iterative_with(
        commit_id,
        require_history,
        is_head,
        seen,
        |object_id| codefire_store::read_object(objects_root, object_id).map_err(CliError::from),
        |commit| verify_commit_signature(commit, keys, true),
    )
}

fn verify_commit_policy_iterative_with<R, V>(
    commit_id: &str,
    require_history: bool,
    is_head: bool,
    seen: &mut BTreeSet<String>,
    mut read_commit: R,
    mut verify_signature: V,
) -> Result<(), CliError>
where
    R: FnMut(&str) -> Result<Value, CliError>,
    V: FnMut(&Value) -> Result<(), CliError>,
{
    let mut pending = vec![(commit_id.to_string(), is_head)];
    while let Some((commit_id, is_head)) = pending.pop() {
        // Preserve the recursive evaluator's visited semantics: an identity is
        // marked before its read, and duplicate/cyclic entries perform no I/O
        // or signature work.
        if !seen.insert(commit_id.clone()) {
            continue;
        }
        let commit = read_commit(&commit_id)?;
        let parents = commit
            .get("parents")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if is_head || commit.get("signature").is_some() || !parents.is_empty() {
            verify_signature(&commit)?;
        }
        if require_history {
            // Reverse push plus LIFO pop exactly reproduces recursive preorder
            // in the stored parent-array order.
            for parent in parents.iter().rev() {
                if let Some(parent_id) = parent.as_str().filter(|id| id.starts_with("CF-COMMIT-")) {
                    pending.push((parent_id.to_string(), false));
                }
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

struct SignatureJsonByteCounter<'a> {
    bytes: u64,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
}

impl std::io::Write for SignatureJsonByteCounter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("signature JSON size overflow"))?;
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("signature JSON size overflow"))?;
        if let Err(error) = self.ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SemanticValidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::AggregateBytes, amount),
        ) {
            self.accounting_error = Some(error);
            return Err(std::io::Error::other(
                "signature JSON measurement budget exceeded",
            ));
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn signature_json_bytes<T: serde::Serialize + ?Sized>(
    value: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<u64, CliError> {
    let mut counter = SignatureJsonByteCounter {
        bytes: 0,
        ledger,
        accounting_error: None,
    };
    let serialized = serde_json::to_writer(&mut counter, value);
    if let Some(error) = counter.accounting_error.take() {
        return Err(nonce_work_error(error, "measure commit-signature JSON"));
    }
    serialized?;
    Ok(counter.bytes)
}

fn verify_commit_signature_with_ledger(
    commit: &Value,
    keys: &Map<String, Value>,
    required: bool,
    ledger: &mut codefire_core::DecisionWorkLedger,
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
    let key_bytes = key_entry
        .map(|key| signature_json_bytes(key, ledger))
        .transpose()?
        .unwrap_or(0);
    let logical_input = signature_json_bytes(commit, ledger)?
        .checked_add(signature_json_bytes(signature, ledger)?)
        .and_then(|bytes| bytes.checked_add(key_bytes))
        .ok_or_else(|| {
            CliError::InvalidRepository("commit signature scratch size overflow".to_string())
        })?;
    let scratch_bytes = logical_input
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(16 * 1024))
        .ok_or_else(|| {
            CliError::InvalidRepository("commit signature scratch size overflow".to_string())
        })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SemanticValidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch_bytes,
            ),
        )
        .map_err(|error| nonce_work_error(error, "reserve commit-signature verification"))?;
    let verified = (|| {
        let key = commit_signature_key_secret(signature, key_entry, signer)?;
        let input = commit_signature_input(commit, signature)?;
        let input_bytes = u64::try_from(input.len()).map_err(|_| {
            CliError::InvalidRepository("commit signature input size overflow".to_string())
        })?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::SemanticValidation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    input_bytes,
                )
                .count(codefire_core::DecisionWorkCounter::BytesHashed, input_bytes),
            )
            .map_err(|error| nonce_work_error(error, "charge commit-signature HMAC input"))?;
        let expected = hmac_hex(&key, &input)?;
        if !constant_time_eq(expected.as_bytes(), digest.as_bytes()) {
            return commit_error("invalid signature");
        }
        Ok(())
    })();
    let release = ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )
        .map_err(|error| nonce_work_error(error, "release commit-signature verification"));
    match (verified, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
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

fn hmac_policy_secret_at(
    key_entry: Option<&Value>,
    signer: &str,
    signed_at: i64,
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
    if let Some(not_before) = key.get("not_before").or_else(|| key.get("valid_from")) {
        let not_before = parse_request_key_boundary(not_before, "not_before", prefix)?;
        if signed_at < not_before {
            return signature_error("key is not valid yet");
        }
    }
    if let Some(not_after) = key.get("not_after").or_else(|| key.get("valid_until")) {
        let not_after = parse_request_key_boundary(not_after, "not_after", prefix)?;
        if signed_at > not_after {
            return signature_error("key is expired");
        }
    }
    key_secret(key, prefix)
}

fn parse_request_key_boundary(value: &Value, field: &str, prefix: &str) -> Result<i64, CliError> {
    value
        .as_str()
        .and_then(codefire_util::parse_iso_utc_seconds)
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(format!(
                "{prefix}: {field} must be UTC seconds ending in Z"
            ))
        })
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

pub(crate) fn remote_request_payload_digest(payload: &Value) -> Result<String, CliError> {
    let mut unsigned = payload.clone();
    if let Some(object) = unsigned.as_object_mut() {
        object.remove("request_signature");
    }
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&unsigned)?)
    ))
}

fn normalized_idempotency_key(payload: &Value) -> Value {
    payload
        .get("idempotency_key")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null)
}

fn remote_request_subject_digest(signature: &Value) -> Result<String, CliError> {
    let identity = json!({
        "type": signature.get("subject_type").cloned().unwrap_or(Value::Null),
        "version": signature.get("subject_version").cloned().unwrap_or(Value::Null),
        "operation": signature.get("operation").cloned().unwrap_or(Value::Null),
        "actor": signature.get("actor").cloned().unwrap_or(Value::Null),
        "target": signature.get("target").cloned().unwrap_or(Value::Null),
        "payload_digest": signature.get("payload_digest").cloned().unwrap_or(Value::Null),
        "idempotency_key": signature.get("idempotency_key").cloned().unwrap_or(Value::Null),
        "key_id": signature.get("key_id").cloned().unwrap_or(Value::Null),
        "timestamp": signature.get("timestamp").cloned().unwrap_or(Value::Null),
        "nonce": signature.get("nonce").cloned().unwrap_or(Value::Null),
    });
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&identity)?)
    ))
}

fn is_sha256_label(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

fn prepare_request_nonce(
    project_root: &Path,
    signature: &Value,
    ttl_seconds: f64,
    max_entries: usize,
    ensure_layout: bool,
) -> Result<PreparedRequestNonce, CliError> {
    if ensure_layout {
        ensure_remote_layout(project_root)?;
    }
    let dirs = remote_dirs(project_root);
    let _lock = FileLock::acquire_with_options(
        dirs.locks.join("request-nonce-cache.lock"),
        &LockOptions::default(),
    )?;
    let now = unix_now_seconds() as f64;
    let cache_path = project_root.join("request_nonce_cache.json");
    let existing = read_optional_json(&cache_path)?;
    let expected_preimage_hash = existing
        .as_ref()
        .map(codefire_store::canonical_json)
        .transpose()?
        .map(|bytes| codefire_util::sha256_hex(&bytes));
    let mut cache = match existing {
        Some(cache)
            if cache.get("version").and_then(Value::as_u64) == Some(1)
                && cache.get("entries").is_some_and(Value::is_object) =>
        {
            cache
        }
        Some(_) => return signature_error("nonce cache is malformed; replay memory was preserved"),
        None => json!({"version": 1, "entries": {}}),
    };
    let entries = cache
        .as_object_mut()
        .expect("cache shape validated")
        .remove("entries")
        .and_then(|entries| match entries {
            Value::Object(entries) => Some(entries),
            _ => None,
        })
        .expect("cache entries shape validated");
    let before_prune = entries.len();
    let cutoff = now - ttl_seconds;
    let mut live_entries = Map::new();
    for (cache_key, entry) in entries {
        let Value::Object(entry) = entry else {
            return signature_error("nonce cache entry is malformed; replay memory was preserved");
        };
        let Some(seen_at) = entry.get("seen_at").and_then(Value::as_f64) else {
            return signature_error("nonce cache entry is malformed; replay memory was preserved");
        };
        if !seen_at.is_finite()
            || entry.get("actor").and_then(Value::as_str).is_none()
            || entry.get("nonce").and_then(Value::as_str).is_none()
        {
            return signature_error("nonce cache entry is malformed; replay memory was preserved");
        }
        if seen_at >= cutoff {
            live_entries.insert(cache_key, Value::Object(entry));
        }
    }
    let mut entries = live_entries;
    let pruned_count = before_prune.saturating_sub(entries.len());
    let actor = signature
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let key_id = signature
        .get("key_id")
        .and_then(Value::as_str)
        .unwrap_or(actor);
    let nonce = signature
        .get("nonce")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if entries.values().any(|entry| {
        entry.get("actor").and_then(Value::as_str) == Some(actor)
            && entry.get("nonce").and_then(Value::as_str) == Some(nonce)
            && match entry.get("key_id").and_then(Value::as_str) {
                Some(existing) => existing == key_id,
                None => true,
            }
    }) {
        return signature_error("nonce has already been used");
    }
    let cache_key = request_nonce_cache_key(signature)?;
    if entries.contains_key(&cache_key) {
        return signature_error("nonce has already been used");
    }
    if entries.len() >= max_entries {
        return signature_error("nonce cache capacity reached; live entries were preserved");
    }
    entries.insert(
        cache_key,
        json!({
            "actor": signature.get("actor").cloned().unwrap_or(Value::Null),
            "key_id": signature.get("key_id").cloned().unwrap_or(Value::Null),
            "operation": signature.get("operation").cloned().unwrap_or(Value::Null),
            "target": signature.get("target").cloned().unwrap_or(Value::Null),
            "timestamp": signature.get("timestamp").cloned().unwrap_or(Value::Null),
            "nonce": signature.get("nonce").cloned().unwrap_or(Value::Null),
            "seen_at": now,
        }),
    );
    let entry_count = entries.len();
    cache["entries"] = Value::Object(entries);
    cache["updated_at"] = Value::String(now_iso_utc());
    cache["ttl_seconds"] = if ttl_seconds.fract() == 0.0 && ttl_seconds <= u64::MAX as f64 {
        Value::Number((ttl_seconds as u64).into())
    } else {
        Value::Number(
            serde_json::Number::from_f64(ttl_seconds).expect("validated finite nonce ttl"),
        )
    };
    cache["max_entries"] = Value::Number((max_entries as u64).into());
    cache["entry_count"] = Value::Number((entry_count as u64).into());
    cache["last_pruned_count"] = Value::Number((pruned_count as u64).into());
    let postimage_bytes = codefire_store::canonical_json(&cache)?;
    if postimage_bytes.len() as u64 > REPOSITORY_JSON_BYTE_LIMIT {
        return signature_error("nonce cache postimage exceeds the bounded repository JSON limit");
    }
    Ok(PreparedRequestNonce {
        expected_preimage_hash,
        postimage_bytes,
        actor: actor.to_string(),
        key_id: key_id.to_string(),
        nonce: nonce.to_string(),
    })
}

fn publish_prepared_request_nonce(
    project_root: &Path,
    prepared: PreparedRequestNonce,
) -> Result<(), CliError> {
    ensure_remote_layout(project_root)?;
    let dirs = remote_dirs(project_root);
    let _lock = FileLock::acquire_with_options(
        dirs.locks.join("request-nonce-cache.lock"),
        &LockOptions::default(),
    )?;
    let cache_path = project_root.join("request_nonce_cache.json");
    let current = read_optional_json(&cache_path)?;
    let current_hash = current
        .as_ref()
        .map(codefire_store::canonical_json)
        .transpose()?
        .map(|bytes| codefire_util::sha256_hex(&bytes));
    if current_hash != prepared.expected_preimage_hash {
        if current.as_ref().is_some_and(|cache| {
            cache
                .get("entries")
                .and_then(Value::as_object)
                .is_some_and(|entries| {
                    entries.values().any(|entry| {
                        entry.get("actor").and_then(Value::as_str) == Some(&prepared.actor)
                            && entry.get("nonce").and_then(Value::as_str) == Some(&prepared.nonce)
                            && entry
                                .get("key_id")
                                .and_then(Value::as_str)
                                .map_or(true, |key_id| key_id == prepared.key_id)
                    })
                })
        }) {
            return signature_error("nonce has already been used");
        }
        return signature_error("nonce cache changed after validation; retry the request");
    }
    write_bytes_atomic(&cache_path, &prepared.postimage_bytes)
}

#[cfg(test)]
fn nonce_publication_reservation(retained_bytes: u64) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::PublicationPreflight,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        retained_bytes,
    )
    .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
}

fn nonce_work_error(error: codefire_core::DecisionWorkAccountingError, context: &str) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while attempting to {context}: {error}"
        )),
    }
}

pub(crate) fn request_nonce_cache_key(signature: &Value) -> Result<String, CliError> {
    let payload = json!({
        "actor": signature.get("actor").cloned().unwrap_or(Value::Null),
        "key_id": signature.get("key_id").cloned().unwrap_or(Value::Null),
        "nonce": signature.get("nonce").cloned().unwrap_or(Value::Null),
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

fn optional_nonnegative_number(
    policy: &Map<String, Value>,
    field: &str,
    default: f64,
) -> Result<f64, CliError> {
    let Some(value) = policy.get(field) else {
        return Ok(default);
    };
    let Some(value) = value.as_f64().filter(|value| value.is_finite()) else {
        return signature_error(&format!("{field} must be a number"));
    };
    if value < 0.0 {
        return signature_error(&format!("{field} must be non-negative"));
    }
    Ok(value)
}

fn optional_positive_usize(
    policy: &Map<String, Value>,
    field: &str,
    default: usize,
) -> Result<usize, CliError> {
    let Some(value) = policy.get(field) else {
        return Ok(default);
    };
    let Some(value) = value.as_u64() else {
        return signature_error(&format!("{field} must be a positive integer"));
    };
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            CliError::AuthenticationOrSignature(format!(
                "request signature validation failed: {field} must be a positive integer"
            ))
        })
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
    use std::collections::BTreeMap;
    use std::sync::{Arc, Barrier};
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
        let signed = sign_commit_payload_at(
            commit,
            "alice",
            "alice-signing-key",
            "alice-2026-06",
            "2026-06-05T00:00:00Z",
        )
        .unwrap();
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
    fn iterative_commit_policy_handles_deep_history_in_preorder() {
        const DEPTH: usize = 20_000;

        let mut seen = BTreeSet::new();
        let mut read_order = Vec::with_capacity(DEPTH);
        let mut verification_order = Vec::with_capacity(DEPTH - 1);

        verify_commit_policy_iterative_with(
            "CF-COMMIT-00000",
            true,
            true,
            &mut seen,
            |commit_id| {
                let index = commit_id
                    .strip_prefix("CF-COMMIT-")
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|index| *index < DEPTH)
                    .ok_or_else(|| CliError::Usage(format!("unknown commit {commit_id}")))?;
                read_order.push(index);
                let parents = if index + 1 == DEPTH {
                    Vec::new()
                } else {
                    vec![Value::String(format!("CF-COMMIT-{:05}", index + 1))]
                };
                Ok(json!({
                    "type": "commit",
                    "version": 1,
                    "parents": parents,
                    "sequence": index
                }))
            },
            |commit| {
                verification_order.push(commit["sequence"].as_u64().unwrap() as usize);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(seen.len(), DEPTH);
        assert_eq!(read_order, (0..DEPTH).collect::<Vec<_>>());
        assert_eq!(verification_order, (0..DEPTH - 1).collect::<Vec<_>>());
    }

    #[test]
    fn iterative_commit_policy_preserves_diamond_repeated_and_cycle_visits() {
        let diamond = BTreeMap::from([
            (
                "CF-COMMIT-root".to_string(),
                json!({"parents": ["CF-COMMIT-left", "CF-COMMIT-right"], "signature": true}),
            ),
            (
                "CF-COMMIT-left".to_string(),
                json!({"parents": ["CF-COMMIT-base"], "signature": true}),
            ),
            (
                "CF-COMMIT-right".to_string(),
                json!({"parents": ["CF-COMMIT-base"], "signature": true}),
            ),
            (
                "CF-COMMIT-base".to_string(),
                json!({"parents": [], "signature": true}),
            ),
        ]);
        let mut diamond_seen = BTreeSet::new();
        let mut diamond_reads = Vec::new();
        let mut diamond_verifications = Vec::new();

        verify_commit_policy_iterative_with(
            "CF-COMMIT-root",
            true,
            true,
            &mut diamond_seen,
            |commit_id| {
                diamond_reads.push(commit_id.to_string());
                diamond
                    .get(commit_id)
                    .cloned()
                    .ok_or_else(|| CliError::Usage(format!("unknown commit {commit_id}")))
            },
            |commit| {
                diamond_verifications
                    .push(commit["parents"].as_array().map_or(0, std::vec::Vec::len));
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(
            diamond_reads,
            vec![
                "CF-COMMIT-root",
                "CF-COMMIT-left",
                "CF-COMMIT-base",
                "CF-COMMIT-right",
            ]
        );
        assert_eq!(diamond_verifications, vec![2, 1, 0, 1]);
        assert_eq!(diamond_seen.len(), 4);

        let repeated = BTreeMap::from([
            (
                "CF-COMMIT-repeated".to_string(),
                json!({
                    "parents": [
                        "CF-COMMIT-base",
                        "CF-COMMIT-base",
                        "CF-COMMIT-base"
                    ],
                    "signature": true
                }),
            ),
            (
                "CF-COMMIT-base".to_string(),
                json!({"parents": [], "signature": true}),
            ),
        ]);
        let mut repeated_seen = BTreeSet::new();
        let mut repeated_reads = Vec::new();
        verify_commit_policy_iterative_with(
            "CF-COMMIT-repeated",
            true,
            true,
            &mut repeated_seen,
            |commit_id| {
                repeated_reads.push(commit_id.to_string());
                repeated
                    .get(commit_id)
                    .cloned()
                    .ok_or_else(|| CliError::Usage(format!("unknown commit {commit_id}")))
            },
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(repeated_reads, vec!["CF-COMMIT-repeated", "CF-COMMIT-base"]);
        assert_eq!(repeated_seen.len(), 2);

        let cycle = BTreeMap::from([
            (
                "CF-COMMIT-cycle-a".to_string(),
                json!({"parents": ["CF-COMMIT-cycle-b"], "signature": true}),
            ),
            (
                "CF-COMMIT-cycle-b".to_string(),
                json!({"parents": ["CF-COMMIT-cycle-a"], "signature": true}),
            ),
        ]);
        let mut cycle_seen = BTreeSet::new();
        let mut cycle_reads = Vec::new();
        verify_commit_policy_iterative_with(
            "CF-COMMIT-cycle-a",
            true,
            true,
            &mut cycle_seen,
            |commit_id| {
                cycle_reads.push(commit_id.to_string());
                cycle
                    .get(commit_id)
                    .cloned()
                    .ok_or_else(|| CliError::Usage(format!("unknown commit {commit_id}")))
            },
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(cycle_reads, vec!["CF-COMMIT-cycle-a", "CF-COMMIT-cycle-b"]);
        assert_eq!(cycle_seen.len(), 2);
    }

    #[test]
    fn iterative_commit_policy_keeps_parent_order_first_error() {
        let commits = BTreeMap::from([
            (
                "CF-COMMIT-root".to_string(),
                json!({
                    "parents": [7, "CF-COMMIT-first", "CF-COMMIT-second"],
                    "signature": true
                }),
            ),
            (
                "CF-COMMIT-first".to_string(),
                json!({"parents": [], "signature": true, "error": "first"}),
            ),
            (
                "CF-COMMIT-second".to_string(),
                json!({"parents": [], "signature": true, "error": "second"}),
            ),
        ]);
        let mut seen = BTreeSet::new();
        let mut reads = Vec::new();

        let error = verify_commit_policy_iterative_with(
            "CF-COMMIT-root",
            true,
            true,
            &mut seen,
            |commit_id| {
                reads.push(commit_id.to_string());
                commits
                    .get(commit_id)
                    .cloned()
                    .ok_or_else(|| CliError::Usage(format!("unknown commit {commit_id}")))
            },
            |commit| match commit.get("error").and_then(Value::as_str) {
                Some(message) => Err(CliError::AuthenticationOrSignature(message.to_string())),
                None => Ok(()),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("first"));
        assert_eq!(reads, vec!["CF-COMMIT-root", "CF-COMMIT-first"]);
        assert_eq!(
            seen,
            BTreeSet::from(["CF-COMMIT-first".to_string(), "CF-COMMIT-root".to_string(),])
        );
    }

    #[test]
    fn iterative_commit_policy_keeps_signature_key_certificate_precedence() {
        let unsigned = json!({
            "type": "commit",
            "version": 1,
            "parents": ["CF-COMMIT-missing"],
            "certificate": {"result": "consistent"}
        });
        let mut signed_then_tampered = sign_commit_payload_at(
            unsigned.clone(),
            "alice",
            "alice-key",
            "alice-key-id",
            "2026-07-19T00:00:00Z",
        )
        .unwrap();
        signed_then_tampered["certificate"]["result"] = json!("failed");

        let mut unsigned_seen = BTreeSet::new();
        let mut unsigned_reads = Vec::new();
        let unsigned_error = verify_commit_policy_iterative_with(
            "CF-COMMIT-head",
            true,
            true,
            &mut unsigned_seen,
            |commit_id| {
                unsigned_reads.push(commit_id.to_string());
                Ok(unsigned.clone())
            },
            |commit| verify_commit_signature(commit, &Map::new(), true),
        )
        .unwrap_err();
        assert!(unsigned_error.to_string().contains("signature is required"));
        assert_eq!(unsigned_reads, vec!["CF-COMMIT-head"]);

        let mut missing_key_seen = BTreeSet::new();
        let mut missing_key_reads = Vec::new();
        let missing_key_error = verify_commit_policy_iterative_with(
            "CF-COMMIT-head",
            true,
            true,
            &mut missing_key_seen,
            |commit_id| {
                missing_key_reads.push(commit_id.to_string());
                Ok(signed_then_tampered.clone())
            },
            |commit| verify_commit_signature(commit, &Map::new(), true),
        )
        .unwrap_err();
        assert!(missing_key_error
            .to_string()
            .contains("no key for signer 'alice'"));
        assert_eq!(missing_key_reads, vec!["CF-COMMIT-head"]);

        let keys = json!({"alice-key-id": "alice-key"});
        let mut certificate_seen = BTreeSet::new();
        let mut certificate_reads = Vec::new();
        let certificate_error = verify_commit_policy_iterative_with(
            "CF-COMMIT-head",
            true,
            true,
            &mut certificate_seen,
            |commit_id| {
                certificate_reads.push(commit_id.to_string());
                Ok(signed_then_tampered.clone())
            },
            |commit| verify_commit_signature(commit, keys.as_object().unwrap(), true),
        )
        .unwrap_err();
        assert!(certificate_error.to_string().contains("invalid signature"));
        assert_eq!(certificate_reads, vec!["CF-COMMIT-head"]);
    }

    #[test]
    fn request_signature_rejects_old_timestamp_and_replayed_nonce() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
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
        let payload = json!({
            "branch": "main",
            "head": "CF-COMMIT-original",
            "objects": [],
            "idempotency_key": "request-1",
        });
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap();
        let replay = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap_err();
        assert!(replay.to_string().contains("nonce has already been used"));

        let mut old = signature.clone();
        old["timestamp"] = Value::String("1970-01-01T00:00:00Z".to_string());
        old["subject_digest"] = Value::String(remote_request_subject_digest(&old).unwrap());
        let digest = request_signature_digest(&old, "alice-request-key").unwrap();
        old["signature"] = Value::String(digest);
        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&old),
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("timestamp is outside allowed skew"));
    }

    #[test]
    fn deferred_nonce_publication_allows_exactly_one_racing_publisher() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
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
        let payload = json!({"branch": "main", "head": "CF-COMMIT-one"});
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let first = validate_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap();
        let second = validate_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap();
        let after_race = validate_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap();
        assert!(!project_root.join("request_nonce_cache.json").exists());

        let barrier = Arc::new(Barrier::new(3));
        let mut threads = Vec::new();
        for pending in [first, second] {
            let barrier = Arc::clone(&barrier);
            let project_root = project_root.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                pending
                    .publish(&project_root)
                    .map_err(|error| error.to_string())
            }));
        }
        barrier.wait();
        let results = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        let rejected = results
            .iter()
            .find_map(|result| result.as_ref().err())
            .unwrap();
        assert!(
            rejected.contains("nonce has already been used")
                || rejected.contains("CodeFire resource is locked"),
            "unexpected deferred nonce rejection: {rejected}"
        );
        let replay = after_race.publish(&project_root).unwrap_err();
        assert!(replay.to_string().contains("nonce has already been used"));
    }

    #[test]
    fn ledger_aware_nonce_validation_owns_exact_bounded_postimage() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 1024,
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
        let now = unix_now_seconds() as f64;
        let entries = (0..256)
            .map(|index| {
                (
                    format!("entry-{index:04}"),
                    json!({
                        "actor": format!("actor-{index}"),
                        "key_id": format!("key-{index}"),
                        "nonce": format!("nonce-{index}"),
                        "seen_at": now,
                    }),
                )
            })
            .collect::<Map<_, _>>();
        write_json_atomic(
            &project_root.join("request_nonce_cache.json"),
            &json!({"version": 1, "entries": entries}),
        )
        .unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        let payload = json!({"branch": "main", "head": "CF-COMMIT-one"});
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            "nonce-postimage",
        )
        .unwrap();
        let pending = validate_remote_request_signature_prelocked_with_work_and_poll(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
            &mut ledger,
            || Ok(()),
        )
        .unwrap();
        let postimage = pending.prepared.as_ref().unwrap().postimage_bytes.clone();
        let retained = pending.retained_bytes().unwrap();
        assert!(retained < REPOSITORY_JSON_BYTE_LIMIT);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(retained)
        );
        let reserved = pending.reserve(&mut ledger).unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(retained)
        );
        ledger.seal_prepublication().unwrap();
        assert_eq!(
            ledger.apply(codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )),
            Err(codefire_core::DecisionWorkAccountingError::PrepublicationSealed)
        );
        reserved.publish(&project_root, &mut ledger).unwrap();
        assert_eq!(
            std::fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
            postimage
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn request_nonce_capacity_fails_closed_and_preserves_live_replay_memory() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
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
        let payload = json!({"branch": "main", "head": "CF-COMMIT-one"});
        let first = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&first),
        )
        .unwrap();
        let same_nonce_payload = json!({"branch": "main", "head": "CF-COMMIT-two"});
        let reused_nonce = sign_remote_request_with_key_at(
            "upload",
            "alice",
            &target,
            &same_nonce_payload,
            "alice-request",
            "alice-request-key",
            RequestFreshness {
                timestamp: first["timestamp"].as_str().unwrap(),
                nonce: first["nonce"].as_str().unwrap(),
            },
        )
        .unwrap();
        let reused = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &same_nonce_payload,
            Some(&reused_nonce),
        )
        .unwrap_err();
        assert!(reused.to_string().contains("nonce has already been used"));
        let second = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let capacity = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&second),
        )
        .unwrap_err();
        assert!(capacity
            .to_string()
            .contains("nonce cache capacity reached"));
        let replay = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&first),
        )
        .unwrap_err();
        assert!(replay.to_string().contains("nonce has already been used"));

        let cache = read_json(&project_root.join("request_nonce_cache.json")).unwrap();
        assert_eq!(cache["entry_count"], 1);
        assert_eq!(cache["max_entries"], 1);
        assert_eq!(cache["ttl_seconds"].as_f64(), Some(300.0));
        assert_eq!(cache["last_pruned_count"], 0);
        assert_eq!(cache["entries"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn malformed_nonce_cache_fails_closed_without_rewriting_replay_memory() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
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
        let cache_path = project_root.join("request_nonce_cache.json");
        write_json_atomic(
            &cache_path,
            &json!({
                "version": 1,
                "entries": {
                    "legacy-entry": {
                        "actor": "alice",
                        "nonce": "legacy-nonce",
                        "seen_at": "invalid"
                    }
                }
            }),
        )
        .unwrap();
        let before = fs::read(&cache_path).unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        let payload = json!({"branch": "main", "head": "CF-COMMIT-one"});
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();

        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap_err();

        assert!(error.to_string().contains("nonce cache entry is malformed"));
        assert_eq!(fs::read(&cache_path).unwrap(), before);
    }

    #[test]
    fn request_signature_binds_payload_and_key_validity_before_nonce_write() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"],
                            "not_after": "1970-01-01T00:00:00Z"
                        }
                    }
                }
            }),
        )
        .unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        let payload = json!({"head": "CF-COMMIT-original", "objects": []});
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let tampered = json!({"head": "CF-COMMIT-tampered", "objects": []});
        let tamper_error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &tampered,
            Some(&signature),
        )
        .unwrap_err();
        assert!(tamper_error.to_string().contains("payload digest mismatch"));
        assert!(!project_root.join("request_nonce_cache.json").exists());

        let key_error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap_err();
        assert!(key_error.to_string().contains("key is expired"));
        assert!(!project_root.join("request_nonce_cache.json").exists());
    }

    #[test]
    fn request_signature_rejects_each_mutation_subject_field() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        write_json_atomic(
            &project_root.join("server_policy.json"),
            &json!({
                "request_signatures": {
                    "required": true,
                    "nonce_cache": false,
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
        let payload = json!({
            "actor": "alice",
            "branch": "main",
            "head": "CF-COMMIT-original",
            "idempotency_key": "request-1",
            "lock": {"wait": true, "timeout_ms": 1000},
            "objects": [{
                "id": "CF-BLOB-original",
                "payload": {"type": "blob", "content": "eA=="}
            }]
        });
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();

        let mut mutations = Vec::new();
        for (label, pointer, value) in [
            ("actor", "/actor", json!("mallory")),
            ("branch", "/branch", json!("other")),
            ("head", "/head", json!("CF-COMMIT-tampered")),
            ("idempotency", "/idempotency_key", json!("request-2")),
            ("lock", "/lock/timeout_ms", json!(2000)),
            ("object id", "/objects/0/id", json!("CF-BLOB-tampered")),
            (
                "object payload",
                "/objects/0/payload/content",
                json!("eQ=="),
            ),
        ] {
            let mut tampered = payload.clone();
            *tampered.pointer_mut(pointer).unwrap() = value;
            mutations.push((label, tampered));
        }

        for (label, tampered) in mutations {
            let error = verify_remote_request_signature(
                &project_root,
                "upload",
                "alice",
                &target,
                &tampered,
                Some(&signature),
            )
            .unwrap_err();
            assert!(
                error.to_string().contains("payload digest mismatch"),
                "{label}: {error}"
            );
        }
        assert!(!project_root.join("request_nonce_cache.json").exists());
    }

    #[test]
    fn request_key_validity_window_rejects_both_sides_before_nonce_write() {
        for (field, boundary, expected) in [
            ("not_before", "2100-01-01T00:00:00Z", "key is not valid yet"),
            ("not_after", "1970-01-01T00:00:00Z", "key is expired"),
        ] {
            let temp = tempdir().unwrap();
            let project_root = temp.path().join("projects/org/remote");
            ensure_remote_layout(&project_root).unwrap();
            let mut key = json!({
                "secret": "alice-request-key",
                "signers": ["alice"]
            });
            key[field] = Value::String(boundary.to_string());
            write_json_atomic(
                &project_root.join("server_policy.json"),
                &json!({
                    "request_signatures": {
                        "required": true,
                        "keys": {"alice-request": key}
                    }
                }),
            )
            .unwrap();
            let target = remote_request_target("org", "app", "upload", Some("main"), None);
            let payload = json!({"branch": "main", "head": "CF-COMMIT-original"});
            let signature = sign_remote_request_with_key(
                "upload",
                "alice",
                &target,
                &payload,
                "alice-request",
                "alice-request-key",
            )
            .unwrap();

            let error = verify_remote_request_signature(
                &project_root,
                "upload",
                "alice",
                &target,
                &payload,
                Some(&signature),
            )
            .unwrap_err();
            assert!(error.to_string().contains(expected), "{field}: {error}");
            assert!(!project_root.join("request_nonce_cache.json").exists());
        }
    }

    #[test]
    fn malformed_request_policy_and_key_window_fail_closed() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("projects/org/remote");
        ensure_remote_layout(&project_root).unwrap();
        let policy_path = project_root.join("server_policy.json");
        write_json_atomic(&policy_path, &json!([])).unwrap();
        let target = remote_request_target("org", "app", "upload", Some("main"), None);
        let payload = json!({"branch": "main", "head": "CF-COMMIT-original"});
        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            None,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("server policy must be an object"));

        write_json_atomic(
            &policy_path,
            &json!({"request_signatures": "not-an-object"}),
        )
        .unwrap();
        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("policy must be an object"));

        write_json_atomic(
            &policy_path,
            &json!({"request_signatures": {"required": "true"}}),
        )
        .unwrap();
        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("required must be a boolean"));

        write_json_atomic(
            &policy_path,
            &json!({
                "request_signatures": {
                    "required": true,
                    "keys": {
                        "alice-request": {
                            "secret": "alice-request-key",
                            "signers": ["alice"],
                            "not_before": 1
                        }
                    }
                }
            }),
        )
        .unwrap();
        let signature = sign_remote_request_with_key(
            "upload",
            "alice",
            &target,
            &payload,
            "alice-request",
            "alice-request-key",
        )
        .unwrap();
        let error = verify_remote_request_signature(
            &project_root,
            "upload",
            "alice",
            &target,
            &payload,
            Some(&signature),
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("not_before must be UTC seconds ending in Z"));
        assert!(!project_root.join("request_nonce_cache.json").exists());
    }

    #[test]
    fn remote_mutation_subject_matches_cross_runtime_golden_vector() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../schemas/fixtures/remote-mutation-subject-v1.json"
        ))
        .unwrap();
        let signature = sign_remote_request_with_key_at(
            fixture["operation"].as_str().unwrap(),
            fixture["actor"].as_str().unwrap(),
            fixture["target"].as_str().unwrap(),
            &fixture["payload"],
            fixture["key_id"].as_str().unwrap(),
            fixture["key"].as_str().unwrap(),
            RequestFreshness {
                timestamp: fixture["timestamp"].as_str().unwrap(),
                nonce: fixture["nonce"].as_str().unwrap(),
            },
        )
        .unwrap();
        assert_eq!(
            signature["payload_digest"],
            fixture["expected"]["payload_digest"]
        );
        assert_eq!(
            signature["subject_digest"],
            fixture["expected"]["subject_digest"]
        );
        assert_eq!(signature["signature"], fixture["expected"]["signature"]);
    }
}
