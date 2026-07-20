use super::project::{RemoteAuthority, RemoteProject, RemoteTransaction};
use super::sha256_hex;
use super::RemoteProjectGate;
use crate::signatures::{
    preflight_remote_request_nonce_prelocked_with_work_and_poll, request_nonce_cache_key,
    validate_remote_request_signature_prelocked_with_work_and_poll, PendingRequestNonce,
};
use crate::{now_iso_utc, CliError};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::io::{ErrorKind, Write};
use std::path::Path;

const PENDING_BYTE_LIMIT: u64 = 1024 * 1024;
const POSTIMAGE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;
const MAX_AUTHORITY_WRITES: usize = 64;
const MAX_OBJECT_ROOTS: usize = 64;
const MAX_IDEMPOTENCY_KEY_BYTES: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct PublicationRequest {
    pub(crate) operation: String,
    pub(crate) actor: String,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) object_roots: Vec<String>,
    pub(crate) semantic: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct AuthorityWrite {
    pub(crate) authority: RemoteAuthority,
    pub(crate) before: Option<Value>,
    pub(crate) after: Value,
}

#[derive(Debug)]
#[must_use = "validated publication-root ownership must be transferred to a prepared publication"]
#[cfg(test)]
pub(crate) struct ValidatedPublicationRoots {
    stable_project_id: String,
    object_store_subject: codefire_store::ValidatedObjectStoreSubject,
    roots: Vec<String>,
    retained_bytes: u64,
}

#[cfg(test)]
#[expect(
    clippy::result_large_err,
    reason = "the Store callback contract returns StoreError; this finalized-root adapter is test-only"
)]
pub(crate) fn validated_publication_roots_from_session(
    gate: &RemoteProjectGate,
    request: &PublicationRequest,
    session: &mut codefire_store::SealedCommitValidationSession,
) -> Result<ValidatedPublicationRoots, CliError> {
    let object_store_subject = session.validated_object_store_subject()?;
    gate.project()
        .ensure_object_store_subject(&object_store_subject)?;
    for root in &request.object_roots {
        session.validated_commit_payload(root)?;
    }
    let stable_project_id = gate.project().stable_id();
    let token = session.with_finalized_decision_work_ledger(|_, ledger| {
        let retained_bytes = request.object_roots.iter().try_fold(
            codefire_core::logical_retained_frame_bytes(stable_project_id).map_err(|error| {
                codefire_store::StoreError::DecisionWorkAccounting(Box::new(error))
            })?,
            |total, root| {
                codefire_core::logical_retained_frame_bytes(root)
                    .and_then(|bytes| {
                        total
                            .checked_add(bytes)
                            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
                    })
                    .map_err(|error| {
                        codefire_store::StoreError::DecisionWorkAccounting(Box::new(error))
                    })
            },
        )?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )?;
        Ok(ValidatedPublicationRoots {
            stable_project_id: stable_project_id.to_string(),
            object_store_subject,
            roots: request.object_roots.clone(),
            retained_bytes,
        })
    })?;
    Ok(token)
}

#[cfg(test)]
fn ensure_validated_publication_roots(
    project: &RemoteProject,
    request: &PublicationRequest,
    validated: &ValidatedPublicationRoots,
) -> Result<(), CliError> {
    if validated.roots != request.object_roots || validated.stable_project_id != project.stable_id()
    {
        return Err(CliError::InvalidRepository(
            "publication roots do not match the finalized request validation subject".to_string(),
        ));
    }
    project.ensure_object_store_subject(&validated.object_store_subject)
}

fn ensure_provisional_publication_roots(
    project: &RemoteProject,
    request: &PublicationRequest,
    provisional: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ledger: &codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    provisional.ensure_ledger(ledger)?;
    if provisional.roots() != request.object_roots {
        return Err(CliError::InvalidRepository(
            "publication roots do not match the provisional validation subject".to_string(),
        ));
    }
    project.ensure_provisional_object_store_subject(provisional)?;
    for root in &request.object_roots {
        provisional.commit_payload(root)?;
    }
    Ok(())
}

#[cfg(test)]
fn empty_validated_publication_roots(
    project_root: &Path,
    request: &PublicationRequest,
) -> ValidatedPublicationRoots {
    assert!(request.object_roots.is_empty());
    let mut session =
        codefire_store::begin_sealed_commit_validation_session(&project_root.join("objects"))
            .unwrap();
    session.finalize().unwrap();
    ValidatedPublicationRoots {
        stable_project_id: read_remote_project_stable_id(project_root),
        object_store_subject: session.validated_object_store_subject().unwrap(),
        roots: Vec::new(),
        retained_bytes: 0,
    }
}

#[cfg(test)]
fn read_remote_project_stable_id(project_root: &Path) -> String {
    let descriptor: Value =
        serde_json::from_slice(&std::fs::read(project_root.join("remote_project.json")).unwrap())
            .unwrap();
    descriptor["stable_project_id"]
        .as_str()
        .unwrap()
        .to_string()
}

struct RecoveredWrite {
    authority: RemoteAuthority,
    authority_path: std::path::PathBuf,
    after_bytes: Vec<u8>,
    after_hash: String,
    is_before: bool,
    is_after: bool,
}

/// A fully planned publication.  Constructing this value performs every JSON
/// encoding, hash, bounded read, and pre/postimage check needed by the durable
/// reducer.  The only decision-work operations left to `commit` are consuming
/// the permit and releasing the retained byte ownership.
#[must_use = "a prepared publication must be revalidated and committed or explicitly cancelled"]
pub(crate) struct PreparedJsonPublication<'a> {
    project: &'a RemoteProject,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    plan: Option<PreparedPublicationPlan>,
    #[cfg(test)]
    validated_roots: Option<ValidatedPublicationRoots>,
    permit: Option<codefire_core::DecisionWorkPermit>,
    reserved_work: codefire_core::DecisionWorkReservation,
    retained_bytes: u64,
    revalidated: bool,
}

#[cfg(test)]
pub(crate) struct PreparedPublicationOwner<'project, 'ledger> {
    prepared: Option<PreparedJsonPublication<'project>>,
    ledger: &'ledger mut codefire_core::DecisionWorkLedger,
}

#[cfg(test)]
impl PreparedPublicationOwner<'_, '_> {
    pub(crate) fn is_replay(&self) -> bool {
        self.prepared
            .as_ref()
            .expect("prepared publication owner is live")
            .is_replay()
    }

    pub(crate) fn seal_and_commit(mut self) -> Result<Value, CliError> {
        let prepared = self
            .prepared
            .as_mut()
            .expect("prepared publication owner is live");
        prepared.revalidate_preseal(self.ledger)?;
        self.ledger
            .seal_prepublication()
            .map_err(|error| publication_work_error(error, "seal prepared publication"))?;
        let result = self
            .prepared
            .as_mut()
            .expect("prepared publication owner is live")
            .commit_after_seal(self.ledger);
        if result.is_ok() {
            drop(self.prepared.take());
        }
        result
    }
}

#[cfg(test)]
impl Drop for PreparedPublicationOwner<'_, '_> {
    fn drop(&mut self) {
        if let Some(prepared) = self.prepared.as_mut() {
            let cancelled = prepared.cancel(self.ledger);
            debug_assert!(cancelled.is_ok());
            if cancelled.is_ok() {
                drop(self.prepared.take());
            }
        }
    }
}

impl PreparedJsonPublication<'_> {
    #[cfg(test)]
    pub(crate) fn is_replay(&self) -> bool {
        matches!(
            self.plan.as_ref(),
            Some(
                PreparedPublicationPlan::Replay(_)
                    | PreparedPublicationPlan::ReplayWithNonce(_)
                    | PreparedPublicationPlan::Resume(_)
                    | PreparedPublicationPlan::Completed(_)
                    | PreparedPublicationPlan::Quarantined(_)
            )
        )
    }

    fn ensure_ledger_identity(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if self.matches_ledger(ledger) {
            Ok(())
        } else {
            Err(CliError::InvalidRepository(
                "prepared JSON publication belongs to a different decision-work ledger".to_string(),
            ))
        }
    }

    /// Side-effect-free constructor guard for the request-wide mutation owner.
    pub(crate) fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        ledger.matches_identity(self.ledger_identity)
    }

    pub(crate) fn revalidate_preseal(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger_identity(ledger)?;
        if ledger.is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "prepared publication cannot be revalidated after prepublication sealing"
                    .to_string(),
            ));
        }
        self.revalidated = false;
        revalidate_prepared_json_publication(self, ledger)
    }

    pub(crate) fn commit_after_seal(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Value, CliError> {
        self.ensure_ledger_identity(ledger)?;
        if !ledger.is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "prepared publication requires a sealed prepublication ledger".to_string(),
            ));
        }
        commit_prepared_json_publication_in_place(self, ledger)
    }

    pub(crate) fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger_identity(ledger)?;
        cancel_prepared_json_publication_in_place(self, ledger)
    }
}

impl Drop for PreparedJsonPublication<'_> {
    fn drop(&mut self) {
        debug_assert!(
            self.plan.is_none()
                && {
                    #[cfg(test)]
                    {
                        self.validated_roots.is_none()
                    }
                    #[cfg(not(test))]
                    {
                        true
                    }
                }
                && self.permit.is_none()
                && self.retained_bytes == 0,
            "raw prepared JSON publication dropped without commit or cancellation"
        );
    }
}

enum PreparedPublicationPlan {
    Replay(Value),
    ReplayWithNonce(PreparedNewPublication),
    New(PreparedNewPublication),
    Resume(PreparedRecoveryPublication),
    Completed(PreparedCompletedPublication),
    Quarantined(PreparedQuarantinedPublication),
}

struct PreparedNewPublication {
    transaction_id: String,
    stages: Vec<(String, Vec<u8>)>,
    pending_initial: Vec<u8>,
    preimages: Vec<PreparedAuthorityPreimage>,
    recovery: PreparedRecoveryPublication,
}

struct PreparedRecoveryPublication {
    transaction: Option<RemoteTransaction>,
    transaction_id: String,
    writes: Vec<PreparedRecoveredWrite>,
    progress_postimages: Vec<Vec<u8>>,
    receipt: Option<PreparedReceiptPublication>,
    completed_pending: Vec<u8>,
    expected_stages: BTreeMap<String, String>,
    cleanup_stage_bytes: u64,
    cleanup_stage_records: u64,
    preimages: Vec<PreparedAuthorityPreimage>,
    result: Value,
}

struct PreparedRecoveredWrite {
    authority: RemoteAuthority,
    after_bytes: Vec<u8>,
    publish: bool,
}

struct PreparedReceiptPublication {
    authority: RemoteAuthority,
    bytes: Vec<u8>,
}

struct PreparedCompletedPublication {
    transaction: Option<RemoteTransaction>,
    transaction_id: String,
    expected_stages: BTreeMap<String, String>,
    cleanup_stage_bytes: u64,
    cleanup_stage_records: u64,
    pending_release_bytes: u64,
    preimages: Vec<PreparedAuthorityPreimage>,
    result: Value,
}

#[derive(Clone)]
struct PreparedAuthorityPreimage {
    authority: RemoteAuthority,
    identity: Option<crate::secure_worktree::StableFsIdentity>,
    hash: Option<String>,
}

struct PreparedQuarantinedPublication {
    completed: PreparedCompletedPublication,
    name: std::ffi::OsString,
    identity: crate::secure_worktree::StableFsIdentity,
}

#[derive(Clone, Copy)]
struct PublicationRecoveryContext<'a> {
    project: &'a RemoteProject,
    request: &'a PublicationRequest,
    request_hash: &'a str,
    receipt: Option<&'a RemoteAuthority>,
}

struct ValidatedRecoveryParts {
    pending: Value,
    records: Vec<Value>,
    published: usize,
    transaction: Option<RemoteTransaction>,
    recovered: Vec<(Vec<u8>, bool, bool)>,
    preimages: Vec<PreparedAuthorityPreimage>,
}

enum PublicationProbe<'a> {
    Resolved(Box<PreparedJsonPublication<'a>>),
    Fresh {
        request_hash: String,
        receipt: Option<RemoteAuthority>,
        identity_temporary_bytes: u64,
    },
}

#[derive(Clone, Copy)]
enum PublicationRootEvidence<'evidence, 'session> {
    #[cfg(test)]
    Finalized(&'evidence ValidatedPublicationRoots),
    Provisional(&'evidence codefire_store::ProvisionalSealedValidationSubject<'session>),
}

/// Resolves every durable replay/recovery case before a caller has to
/// materialize fresh business authority postimages. A nonce write is already
/// validation-owned and may still need publication on an otherwise completed
/// replay, so it is the only write accepted by this phase.
#[cfg(test)]
fn probe_json_publication_with_validated_roots_and_ledger<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    result: &Value,
    nonce_write: Option<&AuthorityWrite>,
    validated_roots: &ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PublicationProbe<'a>, CliError> {
    probe_json_publication_with_root_evidence_and_ledger(
        gate,
        project_root,
        request,
        result,
        nonce_write,
        PublicationRootEvidence::Finalized(validated_roots),
        ledger,
    )
}

fn probe_json_publication_with_root_evidence_and_ledger<'project>(
    gate: &'project RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    result: &Value,
    nonce_write: Option<&AuthorityWrite>,
    root_evidence: PublicationRootEvidence<'_, '_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PublicationProbe<'project>, CliError> {
    gate.ensure_exclusive(project_root)?;
    validate_request(request)?;
    match root_evidence {
        #[cfg(test)]
        PublicationRootEvidence::Finalized(validated) => {
            ensure_validated_publication_roots(gate.project(), request, validated)?
        }
        PublicationRootEvidence::Provisional(provisional) => {
            ensure_provisional_publication_roots(gate.project(), request, provisional, ledger)?
        }
    }
    let identity_temporary_bytes = publication_identity_temporary_bytes(request)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                identity_temporary_bytes,
            ),
        )
        .map_err(|error| publication_work_error(error, "reserve publication identity outputs"))?;
    let probed =
        probe_json_publication_body(gate, project_root, request, result, nonce_write, ledger);
    match probed {
        Ok(PublicationProbe::Fresh {
            request_hash,
            receipt,
            identity_temporary_bytes: 0,
        }) => Ok(PublicationProbe::Fresh {
            request_hash,
            receipt,
            identity_temporary_bytes,
        }),
        Ok(PublicationProbe::Resolved(prepared)) => {
            let release = release_publication_temporary(ledger, identity_temporary_bytes);
            match release {
                Ok(()) => Ok(PublicationProbe::Resolved(prepared)),
                Err(error) => {
                    cancel_prepared_json_publication(*prepared, ledger)?;
                    Err(error)
                }
            }
        }
        Ok(PublicationProbe::Fresh { .. }) => unreachable!("probe body owns no identity scratch"),
        Err(error) => {
            release_publication_temporary(ledger, identity_temporary_bytes)?;
            Err(error)
        }
    }
}

fn probe_json_publication_body<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    result: &Value,
    nonce_write: Option<&AuthorityWrite>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PublicationProbe<'a>, CliError> {
    let request_hash = accounted_json_hash(&NormalizedPublicationRequest::from(request), ledger)?;
    let receipt = receipt_authority(request)?;

    if let Some(path) = pending_quarantine_path(project_root)? {
        let mut temporary_bytes = 0;
        let quarantine_bytes =
            acquire_publication_read(ledger, &path, PENDING_BYTE_LIMIT, &mut temporary_bytes)?;
        let plan = match prepare_completed_pending_quarantine_plan(
            gate.project(),
            request,
            &request_hash,
            receipt.as_ref(),
            ledger,
            &mut temporary_bytes,
            quarantine_bytes,
        ) {
            Ok(Some(plan)) => plan,
            Ok(None) => {
                release_publication_temporary(ledger, temporary_bytes)?;
                return Err(CliError::InvalidRepository(
                    "remote pending quarantine disappeared during preparation".to_string(),
                ));
            }
            Err(error) => {
                release_publication_temporary(ledger, temporary_bytes)?;
                return Err(error);
            }
        };
        return reserve_prepared_plan_with_temporary(
            gate.project(),
            PreparedPublicationPlan::Quarantined(plan),
            ledger,
            temporary_bytes,
        )
        .map(|prepared| PublicationProbe::Resolved(Box::new(prepared)));
    }
    let pending_path = RemoteAuthority::PendingTransaction.path(project_root)?;
    let mut pending_temporary = 0;
    let pending_physical_bytes = if pending_path.exists() {
        acquire_publication_read(
            ledger,
            &pending_path,
            PENDING_BYTE_LIMIT,
            &mut pending_temporary,
        )?
    } else {
        0
    };
    let pending = match read_optional_authority_bounded(
        gate.project(),
        &RemoteAuthority::PendingTransaction,
        PENDING_BYTE_LIMIT,
    ) {
        Ok(pending) => pending,
        Err(error) => {
            release_publication_temporary(ledger, pending_temporary)?;
            return Err(error);
        }
    };
    if let Some(pending) = pending {
        let recovery = PublicationRecoveryContext {
            project: gate.project(),
            request,
            request_hash: &request_hash,
            receipt: receipt.as_ref(),
        };
        let plan = match prepare_recovery_plan(
            gate,
            project_root,
            recovery,
            pending,
            ledger,
            &mut pending_temporary,
            pending_physical_bytes,
        ) {
            Ok(plan) => plan,
            Err(error) => {
                release_publication_temporary(ledger, pending_temporary)?;
                return Err(error);
            }
        };
        return reserve_prepared_plan_with_temporary(
            gate.project(),
            plan,
            ledger,
            pending_temporary,
        )
        .map(|prepared| PublicationProbe::Resolved(Box::new(prepared)));
    } else if pending_temporary > 0 {
        release_publication_temporary(ledger, pending_temporary)?;
    }
    if let Some(receipt_authority) = receipt.as_ref() {
        // A completed receipt is the immutable authority for an exact request
        // replay. Inspect it before materializing fresh postimages: review and
        // apply contain wall-clock fields, and retry construction must not make
        // regenerated values part of replay eligibility.
        if let Some(replayed) = load_completed_receipt_preserving_effects(
            gate.project(),
            receipt_authority,
            request,
            &request_hash,
            result,
        )? {
            if let Some(nonce_write) = nonce_write {
                // A fresh authenticated retry still publishes its nonce, but
                // no regenerated business postimage participates in the
                // nonce-only transaction.
                let (analyzed_nonce, analysis_temporary) = analyze_authority_writes(
                    project_root,
                    std::slice::from_ref(nonce_write),
                    ledger,
                )?;
                let prepared = prepare_new_with_accounted_preimages(
                    gate,
                    project_root,
                    request,
                    std::slice::from_ref(nonce_write),
                    &replayed,
                    &request_hash,
                    None,
                    &analyzed_nonce,
                    ledger,
                    true,
                );
                return release_analysis_after_prepare(prepared, analysis_temporary, ledger)
                    .map(|prepared| PublicationProbe::Resolved(Box::new(prepared)));
            }
            return prepared_replay(gate.project(), replayed, ledger)
                .map(|prepared| PublicationProbe::Resolved(Box::new(prepared)));
        }
    }

    Ok(PublicationProbe::Fresh {
        request_hash,
        receipt,
        identity_temporary_bytes: 0,
    })
}

fn publication_identity_temporary_bytes(request: &PublicationRequest) -> Result<u64, CliError> {
    let hash = codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(64)
        .ok_or_else(|| CliError::InvalidRepository("publication identity size overflow".into()))?;
    request.idempotency_key.as_ref().map_or(Ok(hash), |_| {
        codefire_core::logical_retained_frame_bytes(&request.operation)
            .and_then(|operation| {
                hash.checked_add(operation)
                    .and_then(|total| total.checked_add(hash))
                    .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
            })
            .map_err(|_| CliError::InvalidRepository("publication identity size overflow".into()))
    })
}

fn authority_write_builder_temporary_bytes() -> Result<u64, CliError> {
    let slots = u64::try_from(MAX_AUTHORITY_WRITES + 1)
        .map_err(|_| CliError::InvalidRepository("publication builder size overflow".into()))?;
    let slot_bytes = u64::try_from(std::mem::size_of::<AuthorityWrite>())
        .map_err(|_| CliError::InvalidRepository("publication builder size overflow".into()))?;
    slots
        .checked_mul(slot_bytes)
        .and_then(|bytes| bytes.checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES))
        .ok_or_else(|| CliError::InvalidRepository("publication builder size overflow".into()))
}

/// Prepares either a fresh publication, a replay, or recovery of the matching
/// durable transaction and reserves the exact encoded postimages that commit
/// can publish.  This function never publishes an authority.
#[cfg(test)]
fn prepare_json_publication_with_validated_roots_and_ledger<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    validated_roots: &ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let nonce_write = writes
        .iter()
        .find(|write| write.authority == RemoteAuthority::RequestNonceCache);
    let (request_hash, receipt, identity_temporary_bytes) =
        match probe_json_publication_with_validated_roots_and_ledger(
            gate,
            project_root,
            request,
            result,
            nonce_write,
            validated_roots,
            ledger,
        )? {
            PublicationProbe::Resolved(prepared) => return Ok(*prepared),
            PublicationProbe::Fresh {
                request_hash,
                receipt,
                identity_temporary_bytes,
            } => (request_hash, receipt, identity_temporary_bytes),
        };

    // Only a genuinely new publication is coupled to the caller's current
    // authority postimages.  Delaying this analysis preserves receipt replay
    // precedence even when the current authority was removed or a timestamped
    // postimage was regenerated.
    let (analyzed_writes, analysis_temporary) =
        match analyze_authority_writes(project_root, writes, ledger) {
            Ok(analyzed) => analyzed,
            Err(error) => {
                drop(request_hash);
                drop(receipt);
                release_publication_temporary(ledger, identity_temporary_bytes)?;
                return Err(error);
            }
        };
    let temporary_bytes = identity_temporary_bytes
        .checked_add(analysis_temporary)
        .ok_or_else(|| CliError::InvalidRepository("publication scratch size overflow".into()))?;

    let prepared = prepare_new_with_accounted_preimages(
        gate,
        project_root,
        request,
        writes,
        result,
        &request_hash,
        receipt.as_ref(),
        &analyzed_writes,
        ledger,
        false,
    );
    drop(request_hash);
    drop(receipt);
    release_analysis_after_prepare(prepared, temporary_bytes, ledger)
}

#[cfg(test)]
pub(crate) fn prepare_owned_json_publication_with_validated_roots<'project, 'ledger>(
    gate: &'project RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    validated_roots: ValidatedPublicationRoots,
    ledger: &'ledger mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedPublicationOwner<'project, 'ledger>, CliError> {
    let prepared = prepare_json_publication_with_validated_roots_and_ledger(
        gate,
        project_root,
        request,
        writes,
        result,
        &validated_roots,
        ledger,
    );
    let prepared = transfer_validated_root_ownership(prepared, validated_roots, ledger)?;
    Ok(PreparedPublicationOwner {
        prepared: Some(prepared),
        ledger,
    })
}

#[cfg(test)]
fn transfer_validated_root_ownership<'project>(
    prepared: Result<PreparedJsonPublication<'project>, CliError>,
    validated_roots: ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'project>, CliError> {
    let retained_bytes = validated_roots.retained_bytes;
    match prepared {
        Ok(mut prepared) => {
            let Some(total) = prepared.retained_bytes.checked_add(retained_bytes) else {
                cancel_prepared_json_publication(prepared, ledger)?;
                drop(validated_roots);
                if retained_bytes > 0 {
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained_bytes,
                        )
                        .map_err(|error| {
                            publication_work_error(
                                error,
                                "release overflowing publication-root token",
                            )
                        })?;
                }
                return Err(CliError::InvalidRepository(
                    "prepared publication ownership overflow".to_string(),
                ));
            };
            prepared.retained_bytes = total;
            prepared.validated_roots = Some(validated_roots);
            Ok(prepared)
        }
        Err(error) => {
            drop(validated_roots);
            if retained_bytes > 0 {
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )
                    .map_err(|release| {
                        publication_work_error(release, "release failed publication-root token")
                    })?;
            }
            Err(error)
        }
    }
}

#[cfg(test)]
fn prepare_json_publication_with_ledger<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let validated = empty_validated_publication_roots(project_root, request);
    prepare_json_publication_with_validated_roots_and_ledger(
        gate,
        project_root,
        request,
        writes,
        result,
        &validated,
        ledger,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_new_with_accounted_preimages<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    analyzed_writes: &[AnalyzedAuthorityWrite],
    ledger: &mut codefire_core::DecisionWorkLedger,
    replay_with_nonce: bool,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let mut preimage_temporary = 0;
    let mut preimages = Vec::with_capacity(writes.len());
    for (write, analyzed) in writes.iter().zip(analyzed_writes) {
        let path = match write.authority.path(project_root) {
            Ok(path) => path,
            Err(error) => {
                release_publication_temporary(ledger, preimage_temporary)?;
                return Err(error);
            }
        };
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                if let Err(error) = acquire_publication_read(
                    ledger,
                    &path,
                    POSTIMAGE_BYTE_LIMIT,
                    &mut preimage_temporary,
                ) {
                    release_publication_temporary(ledger, preimage_temporary)?;
                    return Err(error);
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                release_publication_temporary(ledger, preimage_temporary)?;
                return Err(CliError::Io(error));
            }
        }
        match capture_authority_preimage(
            gate.project(),
            &write.authority,
            write.before.as_ref(),
            analyzed.before_hash.as_deref(),
        ) {
            Ok(preimage) => preimages.push(preimage),
            Err(error) => {
                release_publication_temporary(ledger, preimage_temporary)?;
                return Err(error);
            }
        }
    }
    let result_hash = receipt
        .map(|_| accounted_json_hash(result, ledger))
        .transpose()?;
    let prepared = prepare_new_publication_preallocated(
        gate.project(),
        request,
        writes,
        result,
        request_hash,
        receipt,
        analyzed_writes,
        preimages,
        result_hash,
        ledger,
    );
    let release = release_publication_temporary(ledger, preimage_temporary);
    match (prepared, release) {
        (Ok(mut prepared), Ok(())) => {
            if replay_with_nonce {
                let plan = prepared
                    .plan
                    .take()
                    .expect("fresh nonce preparation must retain its plan");
                prepared.plan = Some(match plan {
                    PreparedPublicationPlan::New(plan) => {
                        PreparedPublicationPlan::ReplayWithNonce(plan)
                    }
                    _ => unreachable!("fresh nonce preparation must produce a new plan"),
                });
            }
            Ok(prepared)
        }
        (Err(error), Ok(())) => Err(error),
        (Ok(prepared), Err(error)) => {
            cancel_prepared_json_publication(prepared, ledger)?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

fn release_analysis_after_prepare<'a>(
    prepared: Result<PreparedJsonPublication<'a>, CliError>,
    temporary_bytes: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let release = release_publication_temporary(ledger, temporary_bytes);
    match (prepared, release) {
        (Ok(prepared), Ok(())) => Ok(prepared),
        (Err(error), Ok(())) => Err(error),
        (Ok(prepared), Err(error)) => {
            cancel_prepared_json_publication(prepared, ledger)?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

/// Prepares the request nonce as one authority in the same durable
/// publication as the operation's other writes. The nonce's validation-time
/// ownership is released only after the complete publication plan owns its
/// canonical postimages (or after preparation has failed).
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn prepare_json_publication_lazy_with_nonce_validated_roots_and_ledger<'a, F>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    nonce: PendingRequestNonce,
    result: &Value,
    validated_roots: &ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
    build_business_writes: F,
) -> Result<PreparedJsonPublication<'a>, CliError>
where
    F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<Vec<AuthorityWrite>, CliError>,
{
    prepare_json_publication_lazy_with_nonce_root_evidence_and_ledger(
        gate,
        project_root,
        request,
        nonce,
        result,
        PublicationRootEvidence::Finalized(validated_roots),
        ledger,
        build_business_writes,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_json_publication_lazy_with_nonce_root_evidence_and_ledger<'project, F>(
    gate: &'project RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    nonce: PendingRequestNonce,
    result: &Value,
    root_evidence: PublicationRootEvidence<'_, '_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    build_business_writes: F,
) -> Result<PreparedJsonPublication<'project>, CliError>
where
    F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<Vec<AuthorityWrite>, CliError>,
{
    let mut nonce_temporary = 0;
    let nonce_before = if nonce.has_authority_write() {
        let authority = RemoteAuthority::RequestNonceCache;
        let path = match authority.path(project_root) {
            Ok(path) => path,
            Err(error) => {
                release_nonce_preparation_owners(nonce, ledger, nonce_temporary)?;
                return Err(error);
            }
        };
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                if let Err(error) = acquire_publication_read(
                    ledger,
                    &path,
                    POSTIMAGE_BYTE_LIMIT,
                    &mut nonce_temporary,
                ) {
                    release_nonce_preparation_owners(nonce, ledger, nonce_temporary)?;
                    return Err(error);
                }
                match read_optional_authority_bounded(
                    gate.project(),
                    &authority,
                    POSTIMAGE_BYTE_LIMIT,
                ) {
                    Ok(before) => before,
                    Err(error) => {
                        release_nonce_preparation_owners(nonce, ledger, nonce_temporary)?;
                        return Err(error);
                    }
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => {
                release_nonce_preparation_owners(nonce, ledger, nonce_temporary)?;
                return Err(CliError::Io(error));
            }
        }
    } else {
        None
    };
    let nonce_write = match nonce.authority_write(nonce_before) {
        Ok(write) => write,
        Err(error) => {
            release_nonce_preparation_owners(nonce, ledger, nonce_temporary)?;
            return Err(error);
        }
    };
    let prepared = match probe_json_publication_with_root_evidence_and_ledger(
        gate,
        project_root,
        request,
        result,
        nonce_write.as_ref(),
        root_evidence,
        ledger,
    ) {
        Ok(PublicationProbe::Resolved(prepared)) => Ok(*prepared),
        Ok(PublicationProbe::Fresh {
            request_hash,
            receipt,
            identity_temporary_bytes,
        }) => (|| {
            let builder_temporary_bytes = authority_write_builder_temporary_bytes()?;
            let owned_temporary_bytes = identity_temporary_bytes
                .checked_add(builder_temporary_bytes)
                .ok_or_else(|| {
                    CliError::InvalidRepository("publication scratch size overflow".into())
                })?;
            if let Err(error) = ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    builder_temporary_bytes,
                ),
            ) {
                drop(request_hash);
                drop(receipt);
                release_publication_temporary(ledger, identity_temporary_bytes)?;
                return Err(publication_work_error(
                    error,
                    "reserve fresh authority-write builder",
                ));
            }
            let mut writes = match build_business_writes(ledger) {
                Ok(writes) => writes,
                Err(error) => {
                    drop(request_hash);
                    drop(receipt);
                    release_publication_temporary(ledger, owned_temporary_bytes)?;
                    return Err(error);
                }
            };
            if writes.is_empty() {
                drop(request_hash);
                drop(receipt);
                release_publication_temporary(ledger, owned_temporary_bytes)?;
                return Err(CliError::InvalidRepository(
                    "nonce publication requires at least one business authority write".to_string(),
                ));
            }
            if let Some(write) = nonce_write {
                writes.push(write);
            }
            let (analyzed_writes, analysis_temporary) =
                match analyze_authority_writes(project_root, &writes, ledger) {
                    Ok(analyzed) => analyzed,
                    Err(error) => {
                        drop(request_hash);
                        drop(receipt);
                        release_publication_temporary(ledger, owned_temporary_bytes)?;
                        return Err(error);
                    }
                };
            let temporary_bytes = owned_temporary_bytes
                .checked_add(analysis_temporary)
                .ok_or_else(|| {
                    CliError::InvalidRepository("publication scratch size overflow".into())
                })?;
            let prepared = prepare_new_with_accounted_preimages(
                gate,
                project_root,
                request,
                &writes,
                result,
                &request_hash,
                receipt.as_ref(),
                &analyzed_writes,
                ledger,
                false,
            );
            drop(request_hash);
            drop(receipt);
            release_analysis_after_prepare(prepared, temporary_bytes, ledger)
        })(),
        Err(error) => Err(error),
    };
    let release = release_nonce_preparation_owners(nonce, ledger, nonce_temporary);
    match (prepared, release) {
        (Ok(prepared), Ok(())) => Ok(prepared),
        (Err(error), Ok(())) => Err(error),
        (Ok(prepared), Err(error)) => {
            cancel_prepared_json_publication(prepared, ledger)?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn prepare_json_publication_with_nonce_validated_roots_and_ledger<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    nonce: PendingRequestNonce,
    result: &Value,
    validated_roots: &ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    prepare_json_publication_lazy_with_nonce_validated_roots_and_ledger(
        gate,
        project_root,
        request,
        nonce,
        result,
        validated_roots,
        ledger,
        |_| Ok(writes.to_vec()),
    )
}

/// Production planning path for an unfinalized Store session. The borrowed
/// subject proves the root/capability/ledger relationship without cloning a
/// second root token; the detached Store proof becomes the owning evidence in
/// the request-wide mutation owner immediately after this function returns.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_json_publication_plan_lazy_with_nonce_and_provisional_roots<'project, F>(
    gate: &'project RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    nonce: PendingRequestNonce,
    result: &Value,
    provisional: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    build_business_writes: F,
) -> Result<PreparedJsonPublication<'project>, CliError>
where
    F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<Vec<AuthorityWrite>, CliError>,
{
    prepare_json_publication_lazy_with_nonce_root_evidence_and_ledger(
        gate,
        project_root,
        request,
        nonce,
        result,
        PublicationRootEvidence::Provisional(provisional),
        ledger,
        build_business_writes,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn prepare_json_publication_plan_lazy_with_nonce_and_validated_roots<'project, F>(
    gate: &'project RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    nonce: PendingRequestNonce,
    result: &Value,
    validated_roots: ValidatedPublicationRoots,
    ledger: &mut codefire_core::DecisionWorkLedger,
    build_business_writes: F,
) -> Result<PreparedJsonPublication<'project>, CliError>
where
    F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<Vec<AuthorityWrite>, CliError>,
{
    let prepared = prepare_json_publication_lazy_with_nonce_validated_roots_and_ledger(
        gate,
        project_root,
        request,
        nonce,
        result,
        &validated_roots,
        ledger,
        build_business_writes,
    );
    transfer_validated_root_ownership(prepared, validated_roots, ledger)
}

#[cfg(test)]
fn prepare_json_publication_with_nonce_and_ledger<'a>(
    gate: &'a RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    nonce: PendingRequestNonce,
    result: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let validated = empty_validated_publication_roots(project_root, request);
    prepare_json_publication_with_nonce_validated_roots_and_ledger(
        gate,
        project_root,
        request,
        writes,
        nonce,
        result,
        &validated,
        ledger,
    )
}

/// Validates a signed mutation while permitting exactly one recovery case:
/// the same publication owns an applying pending transaction whose nonce-cache
/// authority is already at its recorded postimage. All other nonce reuse keeps
/// the normal rejection policy.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_publication_signature_with_recovery_and_poll<F>(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<PendingRequestNonce, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    match validate_remote_request_signature_prelocked_with_work_and_poll(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ledger,
        &mut poll,
    ) {
        Ok(nonce) => Ok(nonce),
        Err(CliError::AuthenticationOrSignature(message))
            if message.contains("nonce has already been used") =>
        {
            let error = CliError::AuthenticationOrSignature(message);
            poll()?;
            let expected_nonce_key = request_nonce_cache_key(signature.ok_or_else(|| {
                CliError::AuthenticationOrSignature(
                    "request signature validation failed: signature is missing".to_string(),
                )
            })?)?;
            if matching_pending_nonce_postimage(
                gate,
                project_root,
                request,
                &expected_nonce_key,
                ledger,
            )? {
                poll()?;
                Ok(PendingRequestNonce::recovered_from_matching_publication())
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

/// Performs publication-aware nonce validation without returning retained
/// nonce ownership. The matching in-flight publication exception is identical
/// to the owning validation path below.
#[allow(clippy::too_many_arguments)]
pub(crate) fn preflight_publication_signature_with_recovery_and_poll<F>(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    operation: &str,
    actor: &str,
    target: &str,
    payload: &Value,
    signature: Option<&Value>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut poll: F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    match preflight_remote_request_nonce_prelocked_with_work_and_poll(
        project_root,
        operation,
        actor,
        target,
        payload,
        signature,
        ledger,
        &mut poll,
    ) {
        Ok(()) => Ok(()),
        Err(CliError::AuthenticationOrSignature(message))
            if message.contains("nonce has already been used") =>
        {
            let error = CliError::AuthenticationOrSignature(message);
            poll()?;
            let expected_nonce_key = request_nonce_cache_key(signature.ok_or_else(|| {
                CliError::AuthenticationOrSignature(
                    "request signature validation failed: signature is missing".to_string(),
                )
            })?)?;
            if matching_pending_nonce_postimage(
                gate,
                project_root,
                request,
                &expected_nonce_key,
                ledger,
            )? {
                poll()?;
                Ok(())
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

fn matching_pending_nonce_postimage(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    expected_nonce_key: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<bool, CliError> {
    let pending_authority = RemoteAuthority::PendingTransaction;
    let nonce_authority = RemoteAuthority::RequestNonceCache;
    let active_pending_path = pending_authority.path(project_root)?;
    let (pending_path, quarantined) = if active_pending_path.exists() {
        (active_pending_path, false)
    } else if let Some(path) = pending_quarantine_path(project_root)? {
        (path, true)
    } else {
        return Ok(false);
    };
    let nonce_path = nonce_authority.path(project_root)?;
    if !nonce_path.exists() {
        return Ok(false);
    }
    let mut temporary_bytes = 0;
    acquire_publication_read(
        ledger,
        &pending_path,
        PENDING_BYTE_LIMIT,
        &mut temporary_bytes,
    )?;
    if let Err(error) = acquire_publication_read(
        ledger,
        &nonce_path,
        POSTIMAGE_BYTE_LIMIT,
        &mut temporary_bytes,
    ) {
        release_publication_temporary(ledger, temporary_bytes)?;
        return Err(error);
    }
    let checked = (|| {
        let pending = if quarantined {
            let (_, bytes, _) = gate
                .project()
                .pending_quarantine_bounded(PENDING_BYTE_LIMIT)?
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "pending publication quarantine disappeared during nonce recovery validation"
                            .to_string(),
                    )
                })?;
            serde_json::from_slice(&bytes)?
        } else {
            read_optional_authority_bounded(gate.project(), &pending_authority, PENDING_BYTE_LIMIT)?
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "pending publication disappeared during nonce recovery validation"
                            .to_string(),
                    )
                })?
        };
        let request_hash = accounted_json_hash(&normalized_request_value(request), ledger)?;
        match validate_pending_owner(&pending, request, &request_hash) {
            Ok(()) => {}
            Err(CliError::IdempotencyConflict(_)) => return Ok(false),
            Err(error) => return Err(error),
        }
        let status = pending.get("status").and_then(Value::as_str);
        let phase = pending.get("phase").and_then(Value::as_str);
        if !matches!(
            (status, phase),
            (Some("pending"), Some("applying")) | (Some("completed"), Some("completed"))
        ) {
            return Ok(false);
        }
        if quarantined && status != Some("completed") {
            return Ok(false);
        }
        let records = pending
            .get("writes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote pending writes are missing".to_string())
            })?;
        let published = pending
            .get("published_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote pending progress is missing".to_string())
            })? as usize;
        if published > records.len() {
            return Err(CliError::InvalidRepository(
                "remote pending progress exceeds its write set".to_string(),
            ));
        }
        let mut nonce_record = None;
        for (index, record) in records.iter().enumerate() {
            validate_pending_write_record_shape(record, index)?;
            let authority =
                RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                    CliError::InvalidRepository("remote pending authority is missing".to_string())
                })?)?;
            if authority == RemoteAuthority::RequestNonceCache {
                if nonce_record.is_some() {
                    return Err(CliError::InvalidRepository(
                        "remote pending transaction has duplicate nonce authorities".to_string(),
                    ));
                }
                if record.get("nonce_key").and_then(Value::as_str) != Some(expected_nonce_key) {
                    return Ok(false);
                }
                nonce_record = Some((index, required_pending_string(record, "after_hash")?));
            }
        }
        let Some((nonce_index, expected_after_hash)) = nonce_record else {
            return Ok(false);
        };
        validate_sha256_hex(&expected_after_hash, "remote pending nonce postimage")?;
        if nonce_index > published {
            return Ok(false);
        }
        let cache = read_optional_authority_bounded(
            gate.project(),
            &nonce_authority,
            POSTIMAGE_BYTE_LIMIT,
        )?
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "remote pending nonce postimage disappeared during recovery validation".to_string(),
            )
        })?;
        Ok(accounted_json_hash(&cache, ledger)? == expected_after_hash)
    })();
    let release = release_publication_temporary(ledger, temporary_bytes);
    match (checked, release) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn release_nonce_preparation_owners(
    nonce: PendingRequestNonce,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: u64,
) -> Result<(), CliError> {
    let retained_release = nonce.release_retained(ledger);
    let temporary_release = release_publication_temporary(ledger, temporary_bytes);
    retained_release.and(temporary_release)
}

#[allow(clippy::too_many_arguments)]
fn prepare_new_publication_preallocated<'a>(
    project: &'a RemoteProject,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    analyzed_writes: &[AnalyzedAuthorityWrite],
    preimages: Vec<PreparedAuthorityPreimage>,
    result_hash: Option<String>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let pending_copies = u64::try_from(writes.len())
        .ok()
        .and_then(|count| count.checked_add(4 + u64::from(receipt.is_some())))
        .ok_or_else(|| {
            CliError::InvalidRepository("publication planning size overflow".to_string())
        })?;
    let result_bytes = canonical_json_len(result)?;
    let planning_bytes = pending_copies
        .checked_mul(PENDING_BYTE_LIMIT)
        .and_then(|bytes| bytes.checked_add(result_bytes))
        .ok_or_else(|| {
            CliError::InvalidRepository("publication planning size overflow".to_string())
        })?;
    let mut temporary_bytes = 0;
    acquire_publication_materialization(ledger, planning_bytes, &mut temporary_bytes)?;
    let prepared = prepare_new_publication_preallocated_owned(
        project,
        request,
        writes,
        result,
        request_hash,
        receipt,
        analyzed_writes,
        preimages,
        result_hash,
        ledger,
    );
    let release = release_publication_temporary(ledger, temporary_bytes);
    match (prepared, release) {
        (Ok(prepared), Ok(())) => Ok(prepared),
        (Err(error), Ok(())) => Err(error),
        (Ok(prepared), Err(error)) => {
            cancel_prepared_json_publication(prepared, ledger)?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_new_publication_preallocated_owned<'a>(
    project: &'a RemoteProject,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    analyzed_writes: &[AnalyzedAuthorityWrite],
    preimages: Vec<PreparedAuthorityPreimage>,
    result_hash: Option<String>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let transaction_id = random_transaction_id()?;
    let records = writes
        .iter()
        .zip(analyzed_writes)
        .enumerate()
        .map(|(index, (write, analyzed))| {
            let mut record = json!({
                "authority": write.authority.record(),
                "before_absent": write.before.is_none(),
                "before_hash": analyzed.before_hash,
                "after_hash": analyzed.after_hash,
                "postimage_index": index,
            });
            if write.authority == RemoteAuthority::RequestNonceCache {
                record["nonce_key"] = Value::String(added_nonce_cache_key(write)?);
            }
            Ok(record)
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    let pending = json!({
        "type": "remote_publication_transaction",
        "version": 1,
        "status": "pending",
        "phase": "applying",
        "transaction_id": transaction_id,
        "operation": request.operation,
        "actor": request.actor,
        "idempotency_key": request.idempotency_key,
        "request_hash": request_hash,
        "object_roots": request.object_roots.iter().map(|root| json!({
            "kind": "sealed_commit",
            "id": root,
        })).collect::<Vec<_>>(),
        "writes": records,
        "published_count": 0,
        "result": result,
        "created_at": now_iso_utc(),
    });
    let mut pending_values = Vec::with_capacity(writes.len() + 1);
    let mut progress = pending.clone();
    for index in 0..writes.len() {
        progress["published_count"] = json!(index + 1);
        pending_values.push(progress.clone());
    }
    let result_owned = progress.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote pending result is missing".to_string())
    })?;
    let receipt_value = receipt
        .map(|_| {
            let effects = effects_from_pending_records(&records)?;
            Ok::<_, CliError>(json!({
                "type": "remote_idempotency_receipt",
                "version": 2,
                "status": "completed",
                "operation": request.operation,
                "actor": request.actor,
                "key": request.idempotency_key,
                "request_hash": request_hash,
                "effects": effects,
                "result": result,
                "result_hash": result_hash.as_deref().ok_or_else(|| {
                    CliError::InvalidRepository("publication result hash is missing".to_string())
                })?,
                "completed_at": now_iso_utc(),
            }))
        })
        .transpose()?;
    progress["status"] = Value::String("completed".to_string());
    progress["phase"] = Value::String("completed".to_string());
    progress["completed_at"] = Value::String(now_iso_utc());

    let pending_len = canonical_json_len(&pending)?;
    ensure_pending_len(pending_len, "exceeds its bounded limit")?;
    let mut retained_bytes = pending_len;
    for value in &pending_values {
        let value_len = canonical_json_len(value)?;
        ensure_pending_len(value_len, "grew beyond its bounded limit")?;
        retained_bytes = retained_bytes.checked_add(value_len).ok_or_else(|| {
            CliError::InvalidRepository("publication byte size overflow".to_string())
        })?;
    }
    let completed_len = canonical_json_len(&progress)?;
    ensure_pending_len(completed_len, "grew beyond its bounded limit")?;
    let result_len = canonical_json_len(&result_owned)?;
    retained_bytes = retained_bytes
        .checked_add(completed_len)
        .and_then(|total| total.checked_add(result_len))
        .ok_or_else(|| CliError::InvalidRepository("publication byte size overflow".to_string()))?;
    if let Some(value) = &receipt_value {
        let receipt_len = canonical_json_len(value)?;
        if receipt_len > PENDING_BYTE_LIMIT {
            return Err(CliError::InvalidRepository(
                "remote v2 idempotency receipt exceeds its bounded limit".to_string(),
            ));
        }
        retained_bytes = retained_bytes.checked_add(receipt_len).ok_or_else(|| {
            CliError::InvalidRepository("publication byte size overflow".to_string())
        })?;
    }
    for analyzed in analyzed_writes {
        retained_bytes = retained_bytes
            .checked_add(analyzed.after_len.checked_mul(2).ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })?)
            .ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })?;
    }
    acquire_prepared_retained(ledger, retained_bytes)?;
    let cleanup_stage_bytes = analyzed_writes.iter().fold(0_u64, |total, analyzed| {
        total.saturating_add(analyzed.after_len)
    });
    let reserved_work = publication_reservation(
        retained_bytes,
        writes.len() as u64,
        cleanup_stage_bytes,
        writes.len() as u64,
        completed_len,
    );
    let permit = match ledger.reserve(reserved_work.clone()) {
        Ok(permit) => permit,
        Err(error) => {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(|release| {
                    publication_work_error(release, "release rejected publication plan")
                })?;
            return Err(publication_work_error(error, "reserve publication reducer"));
        }
    };
    let materialized = (|| {
        let stages = writes
            .iter()
            .enumerate()
            .map(|(index, write)| {
                Ok((
                    format!("write-{index}.json"),
                    codefire_store::canonical_json(&write.after)?,
                ))
            })
            .collect::<Result<Vec<_>, CliError>>()?;
        let recovered = writes
            .iter()
            .map(|write| Ok((codefire_store::canonical_json(&write.after)?, true, false)))
            .collect::<Result<Vec<_>, CliError>>()?;
        let pending_initial = encode_pending(&pending, "exceeds its bounded limit")?;
        let progress_postimages = pending_values
            .iter()
            .map(|value| encode_pending(value, "grew beyond its bounded limit"))
            .collect::<Result<Vec<_>, _>>()?;
        let completed_pending = encode_pending(&progress, "grew beyond its bounded limit")?;
        let receipt_plan = match (receipt, receipt_value) {
            (Some(authority), Some(value)) => {
                let bytes = codefire_store::canonical_json(&value)?;
                if bytes.len() as u64 > PENDING_BYTE_LIMIT {
                    return Err(CliError::InvalidRepository(
                        "remote v2 idempotency receipt exceeds its bounded limit".to_string(),
                    ));
                }
                Some(PreparedReceiptPublication {
                    authority: authority.clone(),
                    bytes,
                })
            }
            _ => None,
        };
        let expected_stages = analyzed_writes
            .iter()
            .enumerate()
            .map(|(index, analyzed)| (format!("write-{index}.json"), analyzed.after_hash.clone()))
            .collect();
        let cleanup_stage_bytes = analyzed_writes.iter().try_fold(0_u64, |total, analyzed| {
            total.checked_add(analyzed.after_len).ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })
        })?;
        let planned_writes = writes
            .iter()
            .zip(recovered)
            .map(|(write, (after_bytes, _, _))| PreparedRecoveredWrite {
                authority: write.authority.clone(),
                after_bytes,
                publish: true,
            })
            .collect();
        Ok::<_, CliError>(PreparedPublicationPlan::New(PreparedNewPublication {
            transaction_id: transaction_id.clone(),
            stages,
            pending_initial,
            preimages,
            recovery: PreparedRecoveryPublication {
                transaction: None,
                transaction_id,
                writes: planned_writes,
                progress_postimages,
                receipt: receipt_plan,
                completed_pending,
                expected_stages,
                cleanup_stage_bytes,
                cleanup_stage_records: writes.len() as u64,
                preimages: Vec::new(),
                result: result_owned,
            },
        }))
    })();
    match materialized {
        Ok(plan) => {
            if prepared_plan_size(&plan)?.0 != retained_bytes {
                ledger.cancel_reserved(&permit).map_err(|cancel| {
                    publication_work_error(cancel, "cancel mismatched publication materialization")
                })?;
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )
                    .map_err(|release| {
                        publication_work_error(
                            release,
                            "release mismatched publication materialization",
                        )
                    })?;
                return Err(CliError::InvalidRepository(
                    "publication materialization diverged from its counting pass".to_string(),
                ));
            }
            Ok(PreparedJsonPublication {
                project,
                ledger_identity: ledger.identity(),
                plan: Some(plan),
                #[cfg(test)]
                validated_roots: None,
                permit: Some(permit),
                reserved_work,
                retained_bytes,
                revalidated: false,
            })
        }
        Err(error) => {
            ledger.cancel_reserved(&permit).map_err(|cancel| {
                publication_work_error(cancel, "cancel failed publication materialization")
            })?;
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(|release| {
                    publication_work_error(release, "release failed publication materialization")
                })?;
            Err(error)
        }
    }
}

fn ensure_pending_len(len: u64, reason: &str) -> Result<(), CliError> {
    if len > PENDING_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "remote publication pending marker {reason}"
        )));
    }
    Ok(())
}

/// Commits a prepared reducer.  Callers should seal the ledger immediately
/// before this function; no ordinary `apply` is performed here.
fn revalidate_prepared_json_publication(
    prepared: &mut PreparedJsonPublication<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    let preimages: &[PreparedAuthorityPreimage] =
        match prepared.plan.as_ref().ok_or_else(|| {
            CliError::InvalidRepository("prepared publication plan is missing".into())
        })? {
            PreparedPublicationPlan::Replay(_) => &[],
            PreparedPublicationPlan::ReplayWithNonce(plan) | PreparedPublicationPlan::New(plan) => {
                &plan.preimages
            }
            PreparedPublicationPlan::Resume(plan) => &plan.preimages,
            PreparedPublicationPlan::Completed(plan) => &plan.preimages,
            PreparedPublicationPlan::Quarantined(plan) => &plan.completed.preimages,
        };
    let mut temporary_bytes = 0_u64;
    for expected in preimages {
        let path = expected.authority.path(prepared.project.display_root())?;
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                if let Err(error) = acquire_publication_read(
                    ledger,
                    &path,
                    POSTIMAGE_BYTE_LIMIT,
                    &mut temporary_bytes,
                ) {
                    release_publication_temporary(ledger, temporary_bytes)?;
                    return Err(error);
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                release_publication_temporary(ledger, temporary_bytes)?;
                return Err(CliError::Io(error));
            }
        }
        let actual = match optional_authority_json_preimage(prepared.project, &expected.authority) {
            Ok(actual) => actual,
            Err(error) => {
                release_publication_temporary(ledger, temporary_bytes)?;
                return Err(error);
            }
        };
        if actual.identity != expected.identity || actual.hash != expected.hash {
            release_publication_temporary(ledger, temporary_bytes)?;
            return Err(authority_divergence(&path));
        }
    }
    release_publication_temporary(ledger, temporary_bytes)?;
    prepared.revalidated = true;
    Ok(())
}

fn commit_prepared_json_publication_in_place(
    prepared: &mut PreparedJsonPublication<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<Value, CliError> {
    if !prepared.revalidated {
        return Err(CliError::InvalidRepository(
            "prepared publication was not revalidated immediately before sealing".to_string(),
        ));
    }
    if let Some(permit) = prepared.permit.as_mut() {
        ledger
            .consume_reserved(permit, prepared.reserved_work.clone())
            .map_err(|error| publication_work_error(error, "consume publication permit"))?;
    }
    let _ = prepared.permit.take();
    let retained = prepared.retained_bytes;
    let plan = prepared.plan.take().ok_or_else(|| {
        CliError::InvalidRepository("prepared publication plan is missing".to_string())
    })?;
    let result = match plan {
        PreparedPublicationPlan::Replay(result) => Ok(result),
        PreparedPublicationPlan::ReplayWithNonce(plan) => commit_new_plan(prepared.project, plan),
        PreparedPublicationPlan::New(plan) => commit_new_plan(prepared.project, plan),
        PreparedPublicationPlan::Resume(plan) => commit_recovery_plan(prepared.project, plan),
        PreparedPublicationPlan::Completed(plan) => commit_completed_plan(prepared.project, plan),
        PreparedPublicationPlan::Quarantined(plan) => {
            commit_quarantined_plan(prepared.project, plan)
        }
    };
    // The concrete root strings/subjects are part of the retained owner, not
    // just a scalar charge.  Destroy them before returning their ledger bytes.
    #[cfg(test)]
    drop(prepared.validated_roots.take());
    if retained > 0 {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            )
            .map_err(|error| publication_work_error(error, "release publication postimages"))?;
        prepared.retained_bytes = 0;
    }
    prepared.revalidated = false;
    result
}

/// Compatibility helper for the existing single-publication tests and
/// callers. The raw request-wide owner uses the in-place variant so a failed
/// precondition never consumes its recoverable plan.
#[cfg(test)]
fn commit_prepared_json_publication(
    mut prepared: PreparedJsonPublication<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<Value, CliError> {
    let result = commit_prepared_json_publication_in_place(&mut prepared, ledger);
    if result.is_err() && prepared.plan.is_some() {
        cancel_prepared_json_publication_in_place(&mut prepared, ledger)?;
    }
    result
}

/// Cancels an uncommitted prepared publication without changing repository
/// authorities.
fn cancel_prepared_json_publication(
    mut prepared: PreparedJsonPublication<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    prepared.ensure_ledger_identity(ledger)?;
    cancel_prepared_json_publication_in_place(&mut prepared, ledger)
}

fn cancel_prepared_json_publication_in_place(
    prepared: &mut PreparedJsonPublication<'_>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    if let Some(permit) = prepared.permit.as_ref() {
        ledger
            .cancel_reserved(permit)
            .map_err(|error| publication_work_error(error, "cancel publication permit"))?;
        let _ = prepared.permit.take();
    }
    drop(prepared.plan.take());
    #[cfg(test)]
    drop(prepared.validated_roots.take());
    if prepared.retained_bytes > 0 {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                prepared.retained_bytes,
            )
            .map_err(|error| publication_work_error(error, "release cancelled publication plan"))?;
        prepared.retained_bytes = 0;
    }
    prepared.revalidated = false;
    Ok(())
}

#[derive(Debug)]
struct AnalyzedAuthorityWrite {
    after_len: u64,
    after_hash: String,
    before_hash: Option<String>,
}

fn analyze_authority_writes(
    _project_root: &Path,
    writes: &[AuthorityWrite],
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<AnalyzedAuthorityWrite>, u64), CliError> {
    if writes.is_empty() || writes.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(format!(
            "remote publication write count must be 1..={MAX_AUTHORITY_WRITES}"
        )));
    }
    let temporary_bytes = writes.iter().try_fold(0_u64, |total, write| {
        let hash_bytes = 64_u64
            .checked_mul(1 + u64::from(write.before.is_some()))
            .and_then(|bytes| bytes.checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES))
            .ok_or_else(|| {
                CliError::InvalidRepository("publication analysis size overflow".to_string())
            })?;
        total.checked_add(hash_bytes).ok_or_else(|| {
            CliError::InvalidRepository("publication analysis size overflow".to_string())
        })
    })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            ),
        )
        .map_err(|error| publication_work_error(error, "reserve publication analysis"))?;
    let mut analyzed = Vec::with_capacity(writes.len());
    let result = (|| {
        for (index, write) in writes.iter().enumerate() {
            if writes[..index]
                .iter()
                .any(|prior| prior.authority == write.authority)
            {
                return Err(CliError::InvalidRepository(
                    "remote publication contains duplicate authority writes".to_string(),
                ));
            }
            let (after_len, after_hash) = analyze_json_accounted(&write.after, ledger)?;
            if after_len > POSTIMAGE_BYTE_LIMIT {
                return Err(CliError::InvalidRepository(
                    "remote publication postimage exceeds the bounded record limit".to_string(),
                ));
            }
            analyzed.push(AnalyzedAuthorityWrite {
                after_len,
                after_hash,
                before_hash: write
                    .before
                    .as_ref()
                    .map(|before| analyze_json_accounted(before, ledger).map(|(_, hash)| hash))
                    .transpose()?,
            });
        }
        Ok(())
    })();
    if let Err(error) = result {
        release_publication_temporary(ledger, temporary_bytes)?;
        return Err(error);
    }
    Ok((analyzed, temporary_bytes))
}

fn analyze_json<T>(value: &T) -> Result<(u64, String), CliError>
where
    T: Serialize + ?Sized,
{
    struct Analyzer {
        len: u64,
        hash: Sha256,
    }
    impl Write for Analyzer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.len = self
                .len
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| std::io::Error::other("canonical JSON size overflow"))?;
            self.hash.update(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut analyzer = Analyzer {
        len: 0,
        hash: Sha256::new(),
    };
    serde_json::to_writer(&mut analyzer, value)?;
    Ok((
        analyzer.len,
        codefire_util::hex_lower(&analyzer.hash.finalize()),
    ))
}

fn analyze_json_accounted<T>(
    value: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(u64, String), CliError>
where
    T: Serialize + ?Sized,
{
    let len = canonical_json_len(value)?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::AggregateBytes, len)
            .count(codefire_core::DecisionWorkCounter::BytesHashed, len),
        )
        .map_err(|error| publication_work_error(error, "account publication JSON hash"))?;
    let (actual_len, hash) = analyze_json(value)?;
    if actual_len != len {
        return Err(CliError::InvalidRepository(
            "publication JSON changed during hash accounting".to_string(),
        ));
    }
    Ok((len, hash))
}

fn accounted_json_hash<T>(
    value: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<String, CliError>
where
    T: Serialize + ?Sized,
{
    analyze_json_accounted(value, ledger).map(|(_, hash)| hash)
}

fn added_nonce_cache_key(write: &AuthorityWrite) -> Result<String, CliError> {
    let before = write
        .before
        .as_ref()
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_object);
    let after = write
        .after
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "request nonce cache postimage entries are malformed".to_string(),
            )
        })?;
    let mut added = after
        .keys()
        .filter(|key| before.map_or(true, |entries| !entries.contains_key(*key)));
    let key = added.next().cloned().ok_or_else(|| {
        CliError::InvalidRepository(
            "request nonce cache publication has no new nonce identity".to_string(),
        )
    })?;
    if added.next().is_some() {
        return Err(CliError::InvalidRepository(
            "request nonce cache publication has multiple new nonce identities".to_string(),
        ));
    }
    Ok(key)
}

fn streaming_json_hash(value: &Value) -> Result<String, CliError> {
    analyze_json(value).map(|(_, hash)| hash)
}

fn prepared_replay<'a>(
    project: &'a RemoteProject,
    result: Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let retained_bytes = canonical_json_len(&result)?;
    acquire_prepared_retained(ledger, retained_bytes)?;
    Ok(PreparedJsonPublication {
        project,
        ledger_identity: ledger.identity(),
        plan: Some(PreparedPublicationPlan::Replay(result)),
        #[cfg(test)]
        validated_roots: None,
        permit: None,
        reserved_work: publication_reservation(0, 0, 0, 0, 0),
        retained_bytes,
        revalidated: false,
    })
}

fn prepare_recovery_plan(
    gate: &RemoteProjectGate,
    project_root: &Path,
    recovery: PublicationRecoveryContext<'_>,
    pending: Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
    pending_physical_bytes: u64,
) -> Result<PreparedPublicationPlan, CliError> {
    gate.ensure_exclusive(project_root)?;
    validate_pending_owner(&pending, recovery.request, recovery.request_hash)?;
    let transaction_id = required_pending_string(&pending, "transaction_id")?;
    super::validate_remote_component(&transaction_id, "transaction ID")?;
    let records = pending
        .get("writes")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending writes are missing".to_string())
        })?;
    if records.is_empty() || records.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(
            "remote pending write count is invalid".to_string(),
        ));
    }
    let published = pending
        .get("published_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending progress is missing".to_string())
        })? as usize;
    if published > records.len() {
        return Err(CliError::InvalidRepository(
            "remote pending progress exceeds its write set".to_string(),
        ));
    }
    let transaction = match gate.project().open_transaction(&transaction_id) {
        Ok(transaction) => Some(transaction),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if pending.get("status").and_then(Value::as_str) == Some("completed") {
        if published != records.len() {
            return Err(CliError::InvalidRepository(
                "completed remote pending transaction has incomplete progress".to_string(),
            ));
        }
        return prepare_completed_plan(
            recovery,
            &pending,
            &records,
            transaction,
            ledger,
            temporary_bytes,
            pending_physical_bytes,
        )
        .map(PreparedPublicationPlan::Completed);
    }
    let transaction = transaction.ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "pending remote transaction staging is missing: {transaction_id}"
        ))
    })?;
    let mut expected_stages = BTreeMap::new();
    let mut authority_paths = BTreeSet::new();
    let mut recovered = Vec::with_capacity(records.len());
    let mut preimages = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        validate_pending_write_record_shape(record, index)?;
        let authority =
            RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote pending authority is missing".to_string())
            })?)?;
        if authority == RemoteAuthority::PendingTransaction {
            return Err(CliError::InvalidRepository(
                "remote pending transaction cannot write itself".to_string(),
            ));
        }
        let stage_name = format!("write-{index}.json");
        acquire_publication_read(
            ledger,
            &project_root
                .join("transactions")
                .join(&transaction_id)
                .join(&stage_name),
            POSTIMAGE_BYTE_LIMIT,
            temporary_bytes,
        )?;
        let after_bytes =
            transaction.read_bounded(gate.project(), &stage_name, POSTIMAGE_BYTE_LIMIT)?;
        let after_hash = required_pending_string(record, "after_hash")?;
        validate_sha256_hex(&after_hash, "remote pending postimage")?;
        if sha256_hex(&after_bytes) != after_hash {
            return Err(CliError::InvalidRepository(
                "remote staged postimage hash diverged".to_string(),
            ));
        }
        let after: Value = serde_json::from_slice(&after_bytes)?;
        if !json_matches_canonical_bytes(&after, &after_bytes)? {
            return Err(CliError::InvalidRepository(
                "remote staged postimage is not canonical JSON".to_string(),
            ));
        }
        let authority_path = authority.path(project_root)?;
        if !authority_paths.insert(authority_path.clone()) {
            return Err(CliError::InvalidRepository(
                "remote pending transaction contains duplicate authorities".to_string(),
            ));
        }
        if authority_path.exists() {
            acquire_publication_read(
                ledger,
                &authority_path,
                POSTIMAGE_BYTE_LIMIT,
                temporary_bytes,
            )?;
        }
        let current_preimage = optional_authority_json_preimage(gate.project(), &authority)?;
        let current_hash = current_preimage.hash.clone();
        let before_absent = record
            .get("before_absent")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote pending preimage kind is missing".to_string())
            })?;
        let before_hash = match (before_absent, record.get("before_hash")) {
            (true, Some(Value::Null)) => None,
            (false, Some(Value::String(hash))) => {
                validate_sha256_hex(hash, "remote pending preimage")?;
                Some(hash.as_str())
            }
            _ => {
                return Err(CliError::InvalidRepository(
                    "remote pending before_absent/before_hash shape is invalid".to_string(),
                ))
            }
        };
        let is_before = if before_absent {
            current_hash.is_none()
        } else {
            current_hash.as_deref() == before_hash
        };
        let is_after = current_hash.as_deref() == Some(after_hash.as_str());
        if index < published && !is_after
            || index > published && !is_before
            || index == published && !is_before && !is_after
        {
            return Err(authority_divergence(&authority_path));
        }
        expected_stages.insert(stage_name, after_hash);
        recovered.push((after_bytes, is_before, is_after));
        preimages.push(current_preimage);
    }
    let actual_stage_names = transaction
        .entry_names_bounded(gate.project(), MAX_AUTHORITY_WRITES)?
        .into_iter()
        .map(|name| {
            name.into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "remote transaction staging contains a non-UTF-8 entry".to_string(),
                )
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual_stage_names != expected_stages.keys().cloned().collect() {
        return Err(CliError::InvalidRepository(
            "remote transaction staging set diverged before recovery".to_string(),
        ));
    }
    prepare_recovery_from_validated_parts(
        recovery,
        ValidatedRecoveryParts {
            pending,
            records,
            published,
            transaction: Some(transaction),
            recovered,
            preimages,
        },
        ledger,
        temporary_bytes,
    )
    .map(PreparedPublicationPlan::Resume)
}

fn prepare_recovery_from_validated_parts(
    recovery: PublicationRecoveryContext<'_>,
    parts: ValidatedRecoveryParts,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
) -> Result<PreparedRecoveryPublication, CliError> {
    let ValidatedRecoveryParts {
        mut pending,
        records,
        published,
        transaction,
        recovered,
        preimages,
    } = parts;
    let cleanup_stage_bytes = recovered
        .iter()
        .try_fold(0_u64, |total, (after_bytes, _, _)| {
            total.checked_add(after_bytes.len() as u64).ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })
        })?;
    let mut expected_stages = BTreeMap::new();
    let mut planned_writes = Vec::new();
    let mut progress_postimages = Vec::new();
    for (index, (record, (after_bytes, is_before, is_after))) in
        records.iter().zip(recovered).enumerate()
    {
        let authority =
            RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote pending authority is missing".to_string())
            })?)?;
        let after_hash = required_pending_string(record, "after_hash")?;
        expected_stages.insert(format!("write-{index}.json"), after_hash);
        if index < published {
            continue;
        }
        planned_writes.push(PreparedRecoveredWrite {
            authority: authority.clone(),
            after_bytes,
            publish: is_before && !is_after,
        });
        pending["published_count"] = json!(index + 1);
        let progress_len = canonical_json_len(&pending)?;
        ensure_pending_len(progress_len, "grew beyond its bounded limit")?;
        acquire_publication_materialization(ledger, progress_len, temporary_bytes)?;
        progress_postimages.push(encode_pending(&pending, "grew beyond its bounded limit")?);
    }
    let result = pending.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote pending result is missing".to_string())
    })?;
    if let Some(authority) = recovery.receipt {
        let path = authority.path(recovery.project.display_root())?;
        if path.exists() {
            acquire_publication_read(ledger, &path, PENDING_BYTE_LIMIT, temporary_bytes)?;
        }
    }
    let receipt =
        prepare_receipt_publication(recovery, &records, &result, ledger, temporary_bytes)?;
    pending["status"] = Value::String("completed".to_string());
    pending["phase"] = Value::String("completed".to_string());
    pending["completed_at"] = Value::String(now_iso_utc());
    let completed_len = canonical_json_len(&pending)?;
    ensure_pending_len(completed_len, "grew beyond its bounded limit")?;
    acquire_publication_materialization(ledger, completed_len, temporary_bytes)?;
    let completed_pending = encode_pending(&pending, "grew beyond its bounded limit")?;
    Ok(PreparedRecoveryPublication {
        transaction,
        transaction_id: required_pending_string(&pending, "transaction_id")?,
        writes: planned_writes,
        progress_postimages,
        receipt,
        completed_pending,
        expected_stages,
        cleanup_stage_bytes,
        cleanup_stage_records: records.len() as u64,
        preimages,
        result,
    })
}

fn prepare_receipt_publication(
    recovery: PublicationRecoveryContext<'_>,
    records: &[Value],
    result: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
) -> Result<Option<PreparedReceiptPublication>, CliError> {
    let Some(authority) = recovery.receipt else {
        return Ok(None);
    };
    let effects = effects_from_pending_records(records)?;
    if effects.is_empty() {
        load_completed_receipt_preserving_effects(
            recovery.project,
            authority,
            recovery.request,
            recovery.request_hash,
            result,
        )?
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "replayed nonce transaction is missing its completed receipt".to_string(),
            )
        })?;
        return Ok(None);
    }
    if load_completed_receipt(
        recovery.project,
        authority,
        recovery.request,
        recovery.request_hash,
        &effects,
        result,
    )?
    .is_some()
    {
        return Ok(None);
    }
    let receipt = json!({
        "type": "remote_idempotency_receipt",
        "version": 2,
        "status": "completed",
        "operation": recovery.request.operation,
        "actor": recovery.request.actor,
        "key": recovery.request.idempotency_key,
        "request_hash": recovery.request_hash,
        "effects": effects,
        "result": result,
        "result_hash": accounted_json_hash(result, ledger)?,
        "completed_at": now_iso_utc(),
    });
    let receipt_len = canonical_json_len(&receipt)?;
    if receipt_len > PENDING_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(
            "remote v2 idempotency receipt exceeds its bounded limit".to_string(),
        ));
    }
    acquire_publication_materialization(ledger, receipt_len, temporary_bytes)?;
    let bytes = codefire_store::canonical_json(&receipt)?;
    Ok(Some(PreparedReceiptPublication {
        authority: authority.clone(),
        bytes,
    }))
}

fn prepare_completed_plan(
    recovery: PublicationRecoveryContext<'_>,
    pending: &Value,
    records: &[Value],
    transaction: Option<RemoteTransaction>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
    pending_release_bytes: u64,
) -> Result<PreparedCompletedPublication, CliError> {
    let transaction_id = required_pending_string(pending, "transaction_id")?;
    let effects = effects_from_pending_records(records)?;
    let mut expected_stages = BTreeMap::new();
    let mut preimages = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        validate_pending_write_record_shape(record, index)?;
        let authority =
            RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote pending authority is missing".to_string())
            })?)?;
        let after_hash = required_pending_string(record, "after_hash")?;
        validate_sha256_hex(&after_hash, "remote pending postimage")?;
        let authority_path = authority.path(recovery.project.display_root())?;
        acquire_publication_read(
            ledger,
            &authority_path,
            POSTIMAGE_BYTE_LIMIT,
            temporary_bytes,
        )?;
        let preimage = optional_authority_json_preimage(recovery.project, &authority)?;
        if preimage.hash.as_deref() != Some(after_hash.as_str()) {
            return Err(authority_divergence(
                &authority.path(recovery.project.display_root())?,
            ));
        }
        preimages.push(preimage);
        expected_stages.insert(format!("write-{index}.json"), after_hash);
    }
    let (cleanup_stage_bytes, cleanup_stage_records) = match transaction.as_ref() {
        Some(transaction) => preflight_cleanup_stage(
            recovery.project,
            transaction,
            &expected_stages,
            ledger,
            temporary_bytes,
        )?,
        None => (0, 0),
    };
    let result = pending.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote pending result is missing".to_string())
    })?;
    if let Some(receipt) = recovery.receipt {
        acquire_publication_read(
            ledger,
            &receipt.path(recovery.project.display_root())?,
            PENDING_BYTE_LIMIT,
            temporary_bytes,
        )?;
        let loaded = if effects.is_empty() {
            load_completed_receipt_preserving_effects(
                recovery.project,
                receipt,
                recovery.request,
                recovery.request_hash,
                &result,
            )?
        } else {
            load_completed_receipt(
                recovery.project,
                receipt,
                recovery.request,
                recovery.request_hash,
                &effects,
                &result,
            )?
        };
        loaded.ok_or_else(|| {
            CliError::InvalidRepository(
                "completed remote transaction is missing its v2 receipt".to_string(),
            )
        })?;
    }
    Ok(PreparedCompletedPublication {
        transaction,
        transaction_id,
        expected_stages,
        cleanup_stage_bytes,
        cleanup_stage_records,
        pending_release_bytes,
        preimages,
        result,
    })
}

fn encode_pending(pending: &Value, reason: &str) -> Result<Vec<u8>, CliError> {
    let bytes = codefire_store::canonical_json(pending)?;
    if bytes.len() as u64 > PENDING_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "remote publication pending marker {reason}"
        )));
    }
    Ok(bytes)
}

fn reserve_prepared_plan<'a>(
    project: &'a RemoteProject,
    plan: PreparedPublicationPlan,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let (retained_bytes, record_count) = prepared_plan_size(&plan)?;
    let (cleanup_stage_bytes, cleanup_stage_records, pending_release_bytes) =
        prepared_cleanup_work(&plan);
    let reserved_work = publication_reservation(
        retained_bytes,
        record_count,
        cleanup_stage_bytes,
        cleanup_stage_records,
        pending_release_bytes,
    );
    acquire_prepared_retained(ledger, retained_bytes)?;
    let permit = match ledger.reserve(reserved_work.clone()) {
        Ok(permit) => permit,
        Err(error) => {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(|release| {
                    publication_work_error(release, "release rejected publication plan")
                })?;
            return Err(publication_work_error(error, "reserve publication reducer"));
        }
    };
    Ok(PreparedJsonPublication {
        project,
        ledger_identity: ledger.identity(),
        plan: Some(plan),
        #[cfg(test)]
        validated_roots: None,
        permit: Some(permit),
        reserved_work,
        retained_bytes,
        revalidated: false,
    })
}

fn reserve_prepared_plan_with_temporary<'a>(
    project: &'a RemoteProject,
    plan: PreparedPublicationPlan,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: u64,
) -> Result<PreparedJsonPublication<'a>, CliError> {
    let prepared = reserve_prepared_plan(project, plan, ledger);
    let release = release_publication_temporary(ledger, temporary_bytes);
    match (prepared, release) {
        (Ok(prepared), Ok(())) => Ok(prepared),
        (Err(error), Ok(())) => Err(error),
        (Ok(prepared), Err(error)) => {
            cancel_prepared_json_publication(prepared, ledger)?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

fn pending_quarantine_path(project_root: &Path) -> Result<Option<std::path::PathBuf>, CliError> {
    let mut candidate = None;
    for entry in std::fs::read_dir(project_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(text) = name.to_str() else {
            continue;
        };
        if !text.starts_with(".pending_transaction.removing-") || !text.ends_with(".json") {
            continue;
        }
        if candidate.is_some() {
            return Err(CliError::InvalidRepository(
                "multiple remote pending transaction quarantines are preserved".to_string(),
            ));
        }
        candidate = Some(entry.path());
    }
    Ok(candidate)
}

fn acquire_publication_read(
    ledger: &mut codefire_core::DecisionWorkLedger,
    path: &Path,
    limit: u64,
    temporary_bytes: &mut u64,
) -> Result<u64, CliError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(CliError::InvalidRepository(format!(
            "remote publication input is not a regular file: {}",
            path.display()
        )));
    }
    let physical_bytes = metadata.len();
    if physical_bytes > limit {
        return Err(CliError::InvalidRepository(format!(
            "remote publication input exceeds its {limit}-byte limit: {}",
            path.display()
        )));
    }
    let scratch = physical_bytes.checked_mul(2).ok_or_else(|| {
        CliError::InvalidRepository("remote publication input size overflow".to_string())
    })?;
    let next_temporary = temporary_bytes.checked_add(scratch).ok_or_else(|| {
        CliError::InvalidRepository("remote publication temporary size overflow".to_string())
    })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                physical_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch,
            )
            .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                physical_bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::BytesHashed,
                physical_bytes,
            ),
        )
        .map_err(|error| publication_work_error(error, "reserve publication input read"))?;
    *temporary_bytes = next_temporary;
    Ok(physical_bytes)
}

fn acquire_publication_materialization(
    ledger: &mut codefire_core::DecisionWorkLedger,
    bytes: u64,
    temporary_bytes: &mut u64,
) -> Result<(), CliError> {
    let next_temporary = temporary_bytes.checked_add(bytes).ok_or_else(|| {
        CliError::InvalidRepository("remote publication temporary size overflow".to_string())
    })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, bytes),
        )
        .map_err(|error| publication_work_error(error, "reserve publication materialization"))?;
    *temporary_bytes = next_temporary;
    Ok(())
}

fn release_publication_temporary(
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: u64,
) -> Result<(), CliError> {
    if temporary_bytes == 0 {
        return Ok(());
    }
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )
        .map_err(|error| publication_work_error(error, "release publication input ownership"))
}

fn publication_reservation(
    retained_bytes: u64,
    record_count: u64,
    cleanup_stage_bytes: u64,
    cleanup_stage_records: u64,
    pending_release_bytes: u64,
) -> codefire_core::DecisionWorkReservation {
    let cleanup_bytes = cleanup_stage_bytes.saturating_add(pending_release_bytes);
    let cleanup_files = cleanup_stage_records.saturating_add(u64::from(pending_release_bytes > 0));
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::PublicationPreflight,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        retained_bytes.saturating_add(cleanup_bytes),
    )
    .charge(
        codefire_core::DecisionWorkDimension::ObjectRecords,
        record_count,
    )
    .count(codefire_core::DecisionWorkCounter::FilesRead, cleanup_files)
    .count(
        codefire_core::DecisionWorkCounter::FileBytesRead,
        cleanup_bytes,
    )
    .count(
        codefire_core::DecisionWorkCounter::BytesHashed,
        cleanup_stage_bytes,
    )
}

fn prepared_cleanup_work(plan: &PreparedPublicationPlan) -> (u64, u64, u64) {
    fn recovery(plan: &PreparedRecoveryPublication) -> (u64, u64, u64) {
        (
            plan.cleanup_stage_bytes,
            plan.cleanup_stage_records,
            plan.completed_pending.len() as u64,
        )
    }
    fn completed(plan: &PreparedCompletedPublication) -> (u64, u64, u64) {
        (
            plan.cleanup_stage_bytes,
            plan.cleanup_stage_records,
            plan.pending_release_bytes,
        )
    }
    match plan {
        PreparedPublicationPlan::Replay(_) => (0, 0, 0),
        PreparedPublicationPlan::ReplayWithNonce(plan) | PreparedPublicationPlan::New(plan) => {
            recovery(&plan.recovery)
        }
        PreparedPublicationPlan::Resume(plan) => recovery(plan),
        PreparedPublicationPlan::Completed(plan) => completed(plan),
        PreparedPublicationPlan::Quarantined(plan) => {
            let (stage_bytes, stage_records, _) = completed(&plan.completed);
            (stage_bytes, stage_records, 0)
        }
    }
}

fn prepared_plan_size(plan: &PreparedPublicationPlan) -> Result<(u64, u64), CliError> {
    fn add(total: &mut u64, bytes: &[u8]) -> Result<(), CliError> {
        *total = total.checked_add(bytes.len() as u64).ok_or_else(|| {
            CliError::InvalidRepository("publication byte size overflow".to_string())
        })?;
        Ok(())
    }
    fn recovery_size(plan: &PreparedRecoveryPublication) -> Result<(u64, u64), CliError> {
        let mut total = 0_u64;
        for write in &plan.writes {
            add(&mut total, &write.after_bytes)?;
        }
        for bytes in &plan.progress_postimages {
            add(&mut total, bytes)?;
        }
        if let Some(receipt) = &plan.receipt {
            add(&mut total, &receipt.bytes)?;
        }
        add(&mut total, &plan.completed_pending)?;
        total = total
            .checked_add(canonical_json_len(&plan.result)?)
            .ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })?;
        Ok((total, plan.writes.len() as u64))
    }
    match plan {
        PreparedPublicationPlan::Replay(result) => Ok((canonical_json_len(result)?, 0)),
        PreparedPublicationPlan::ReplayWithNonce(plan) => {
            let mut total = 0_u64;
            for (_, bytes) in &plan.stages {
                add(&mut total, bytes)?;
            }
            add(&mut total, &plan.pending_initial)?;
            let (recovery, records) = recovery_size(&plan.recovery)?;
            total = total.checked_add(recovery).ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })?;
            Ok((total, records))
        }
        PreparedPublicationPlan::Completed(plan) => Ok((canonical_json_len(&plan.result)?, 0)),
        PreparedPublicationPlan::Quarantined(plan) => {
            Ok((canonical_json_len(&plan.completed.result)?, 0))
        }
        PreparedPublicationPlan::Resume(plan) => recovery_size(plan),
        PreparedPublicationPlan::New(plan) => {
            let mut total = 0_u64;
            for (_, bytes) in &plan.stages {
                add(&mut total, bytes)?;
            }
            add(&mut total, &plan.pending_initial)?;
            let (recovery, records) = recovery_size(&plan.recovery)?;
            total = total.checked_add(recovery).ok_or_else(|| {
                CliError::InvalidRepository("publication byte size overflow".to_string())
            })?;
            Ok((total, records))
        }
    }
}

fn acquire_prepared_retained(
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: u64,
) -> Result<(), CliError> {
    if retained_bytes == 0 {
        return Ok(());
    }
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
        .map_err(|error| publication_work_error(error, "acquire publication plan ownership"))
}

fn canonical_json_len<T>(value: &T) -> Result<u64, CliError>
where
    T: Serialize + ?Sized,
{
    struct Counter(u64);
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| std::io::Error::other("canonical JSON size overflow"))?;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.0)
}

fn commit_new_plan(
    project: &RemoteProject,
    mut plan: PreparedNewPublication,
) -> Result<Value, CliError> {
    let transaction = project.create_transaction(&plan.transaction_id)?;
    let setup = (|| {
        for (name, bytes) in &plan.stages {
            transaction.write_create_noreplace(project, name, bytes)?;
        }
        project.write_authority_create_noreplace(
            &RemoteAuthority::PendingTransaction,
            &plan.pending_initial,
        )?;
        Ok::<_, CliError>(())
    })();
    if let Err(error) = setup {
        if let Err(cleanup) =
            cleanup_transaction_stage(project, &transaction, &plan.recovery.expected_stages)
        {
            return Err(CliError::InvalidRepository(format!(
                "remote publication setup failed ({error}); owned transaction cleanup also failed ({cleanup})"
            )));
        }
        return Err(error);
    }
    plan.recovery.transaction = Some(transaction);
    commit_recovery_plan(project, plan.recovery)
}

fn commit_recovery_plan(
    project: &RemoteProject,
    plan: PreparedRecoveryPublication,
) -> Result<Value, CliError> {
    let transaction = plan.transaction.ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "pending remote transaction staging is missing: {}",
            plan.transaction_id
        ))
    })?;
    for (write, progress) in plan.writes.iter().zip(&plan.progress_postimages) {
        if write.publish {
            project.write_authority_atomic(&write.authority, &write.after_bytes)?;
        }
        project.write_authority_atomic(&RemoteAuthority::PendingTransaction, progress)?;
    }
    if let Some(receipt) = &plan.receipt {
        project.write_authority_create_noreplace(&receipt.authority, &receipt.bytes)?;
    }
    project.write_authority_atomic(
        &RemoteAuthority::PendingTransaction,
        &plan.completed_pending,
    )?;
    cleanup_transaction_stage(project, &transaction, &plan.expected_stages)?;
    release_pending(project, &plan.transaction_id)?;
    Ok(plan.result)
}

fn commit_completed_plan(
    project: &RemoteProject,
    plan: PreparedCompletedPublication,
) -> Result<Value, CliError> {
    if let Some(transaction) = plan.transaction.as_ref() {
        cleanup_transaction_stage(project, transaction, &plan.expected_stages)?;
    } else {
        project.remove_transaction_quarantine_if_present(&plan.transaction_id)?;
    }
    release_pending(project, &plan.transaction_id)?;
    Ok(plan.result)
}

fn commit_quarantined_plan(
    project: &RemoteProject,
    plan: PreparedQuarantinedPublication,
) -> Result<Value, CliError> {
    let completed = plan.completed;
    if let Some(transaction) = completed.transaction.as_ref() {
        cleanup_transaction_stage(project, transaction, &completed.expected_stages)?;
    } else {
        project.remove_transaction_quarantine_if_present(&completed.transaction_id)?;
    }
    project.remove_pending_quarantine_exact(&plan.name, plan.identity)?;
    Ok(completed.result)
}

fn prepare_completed_pending_quarantine_plan(
    project: &RemoteProject,
    request: &PublicationRequest,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
    quarantine_bytes: u64,
) -> Result<Option<PreparedQuarantinedPublication>, CliError> {
    let Some((name, bytes, identity)) = project.pending_quarantine_bounded(PENDING_BYTE_LIMIT)?
    else {
        return Ok(None);
    };
    let pending: Value = serde_json::from_slice(&bytes)?;
    validate_pending_owner(&pending, request, request_hash)?;
    if pending.get("status").and_then(Value::as_str) != Some("completed")
        || pending.get("phase").and_then(Value::as_str) != Some("completed")
    {
        return Err(CliError::InvalidRepository(
            "non-completed remote pending quarantine is preserved".to_string(),
        ));
    }
    let transaction_id = required_pending_string(&pending, "transaction_id")?;
    super::validate_remote_component(&transaction_id, "transaction ID")?;
    let expected_name = format!(".pending_transaction.removing-{transaction_id}.json");
    if name != OsStr::new(&expected_name) {
        return Err(CliError::InvalidRepository(
            "remote pending quarantine name does not match its transaction".to_string(),
        ));
    }
    let records = pending
        .get("writes")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending quarantine writes are missing".to_string())
        })?;
    if records.is_empty()
        || records.len() > MAX_AUTHORITY_WRITES
        || pending.get("published_count").and_then(Value::as_u64) != Some(records.len() as u64)
    {
        return Err(CliError::InvalidRepository(
            "remote pending quarantine progress is invalid".to_string(),
        ));
    }
    let transaction = match project.open_transaction(&transaction_id) {
        Ok(transaction) => Some(transaction),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let completed = prepare_completed_plan(
        PublicationRecoveryContext {
            project,
            request,
            request_hash,
            receipt,
        },
        &pending,
        &records,
        transaction,
        ledger,
        temporary_bytes,
        quarantine_bytes,
    )?;
    Ok(Some(PreparedQuarantinedPublication {
        completed,
        name,
        identity,
    }))
}

fn publication_work_error(
    error: codefire_core::DecisionWorkAccountingError,
    context: &str,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while attempting to {context}: {error}"
        )),
    }
}

pub(crate) fn execute_json_publication(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
) -> Result<Value, CliError> {
    gate.ensure_exclusive(project_root)?;
    validate_request(request)?;
    validate_writes(project_root, writes)?;
    validate_object_roots(project_root, request)?;
    let request_hash = normalized_request_hash(request)?;
    let receipt = receipt_authority(request)?;
    if let Some(result) = recover_completed_pending_quarantine(
        gate.project(),
        request,
        &request_hash,
        receipt.as_ref(),
    )? {
        return Ok(result);
    }
    let pending = read_optional_authority_bounded(
        gate.project(),
        &RemoteAuthority::PendingTransaction,
        PENDING_BYTE_LIMIT,
    )?;
    if let Some(pending) = pending {
        return recover_publication(
            gate,
            project_root,
            request,
            &request_hash,
            receipt.as_ref(),
            pending,
        );
    }
    if let Some(receipt) = receipt.as_ref() {
        let expected_effects = effects_from_authority_writes(writes)?;
        if let Some(replayed) = load_completed_receipt(
            gate.project(),
            receipt,
            request,
            &request_hash,
            &expected_effects,
            result,
        )? {
            return Ok(replayed);
        }
    }
    prepare_publication(gate.project(), request, writes, result, &request_hash)?;
    let pending = read_optional_authority_bounded(
        gate.project(),
        &RemoteAuthority::PendingTransaction,
        PENDING_BYTE_LIMIT,
    )?
    .ok_or_else(|| {
        CliError::InvalidRepository("remote publication pending marker disappeared".to_string())
    })?;
    recover_publication(
        gate,
        project_root,
        request,
        &request_hash,
        receipt.as_ref(),
        pending,
    )
}

pub(crate) fn replay_json_publication(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
) -> Result<Option<Value>, CliError> {
    gate.ensure_exclusive(project_root)?;
    validate_request(request)?;
    validate_object_roots(project_root, request)?;
    let request_hash = normalized_request_hash(request)?;
    let receipt = receipt_authority(request)?;
    if let Some(result) = recover_completed_pending_quarantine(
        gate.project(),
        request,
        &request_hash,
        receipt.as_ref(),
    )? {
        return Ok(Some(result));
    }
    if let Some(pending) = read_optional_authority_bounded(
        gate.project(),
        &RemoteAuthority::PendingTransaction,
        PENDING_BYTE_LIMIT,
    )? {
        return recover_publication(
            gate,
            project_root,
            request,
            &request_hash,
            receipt.as_ref(),
            pending,
        )
        .map(Some);
    }
    let Some(receipt_authority) = receipt.as_ref() else {
        return Ok(None);
    };
    let Some(receipt) =
        read_optional_authority_bounded(gate.project(), receipt_authority, PENDING_BYTE_LIMIT)?
    else {
        return Ok(None);
    };
    let object = receipt.as_object().ok_or_else(|| {
        CliError::InvalidRepository("remote v2 idempotency receipt is not an object".to_string())
    })?;
    let expected_fields = BTreeSet::from([
        "type",
        "version",
        "status",
        "operation",
        "actor",
        "key",
        "request_hash",
        "effects",
        "result",
        "result_hash",
        "completed_at",
    ]);
    if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_fields
        || receipt
            .get("completed_at")
            .and_then(Value::as_str)
            .map_or(true, str::is_empty)
    {
        return Err(CliError::InvalidRepository(
            "remote v2 idempotency receipt has unknown or missing fields".to_string(),
        ));
    }
    let effects = validate_receipt_effects(&receipt)?;
    let result = receipt.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote v2 receipt result is missing".to_string())
    })?;
    validate_completed_receipt_value(&receipt, request, &request_hash, &effects, &result).map(Some)
}

fn validate_request(request: &PublicationRequest) -> Result<(), CliError> {
    super::validate_remote_component(&request.operation, "operation")?;
    if request.actor.is_empty()
        || request.actor.len() > 255
        || request.actor.chars().any(char::is_control)
    {
        return Err(CliError::Usage(
            "remote publication actor is invalid".to_string(),
        ));
    }
    if request.idempotency_key.as_ref().is_some_and(|key| {
        key.is_empty() || key.len() > MAX_IDEMPOTENCY_KEY_BYTES || key.chars().any(char::is_control)
    }) {
        return Err(CliError::Usage(
            "remote idempotency key is invalid".to_string(),
        ));
    }
    let duplicate_root = request
        .object_roots
        .iter()
        .enumerate()
        .any(|(index, root)| {
            request.object_roots[..index]
                .iter()
                .any(|prior| prior == root)
        });
    if request.object_roots.len() > MAX_OBJECT_ROOTS || duplicate_root {
        return Err(CliError::Usage(
            "remote publication object roots are duplicated or exceed the bounded limit"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_object_roots(
    project_root: &Path,
    request: &PublicationRequest,
) -> Result<(), CliError> {
    let objects = super::remote_dirs(project_root).objects;
    for root in &request.object_roots {
        codefire_store::validate_sealed_commit(&objects, root).map_err(|error| {
            CliError::InvalidRepository(format!(
                "remote publication object root is not a complete target-local sealed commit ({root}): {error}"
            ))
        })?;
    }
    Ok(())
}

fn validate_writes(project_root: &Path, writes: &[AuthorityWrite]) -> Result<(), CliError> {
    if writes.is_empty() || writes.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(format!(
            "remote publication write count must be 1..={MAX_AUTHORITY_WRITES}"
        )));
    }
    let mut paths = BTreeSet::new();
    for write in writes {
        let path = write.authority.path(project_root)?;
        if !paths.insert(path) {
            return Err(CliError::InvalidRepository(
                "remote publication contains duplicate authority writes".to_string(),
            ));
        }
        let bytes = codefire_store::canonical_json(&write.after)?;
        if bytes.len() as u64 > POSTIMAGE_BYTE_LIMIT {
            return Err(CliError::InvalidRepository(
                "remote publication postimage exceeds the bounded record limit".to_string(),
            ));
        }
    }
    Ok(())
}

fn normalized_request_hash(request: &PublicationRequest) -> Result<String, CliError> {
    Ok(sha256_hex(&codefire_store::canonical_json(
        &normalized_request_value(request),
    )?))
}

#[derive(Serialize)]
struct NormalizedPublicationRequest<'a> {
    actor: &'a str,
    idempotency_key: &'a Option<String>,
    object_roots: &'a [String],
    operation: &'a str,
    semantic: &'a Value,
    #[serde(rename = "type")]
    kind: &'static str,
    version: u64,
}

impl<'a> From<&'a PublicationRequest> for NormalizedPublicationRequest<'a> {
    fn from(request: &'a PublicationRequest) -> Self {
        Self {
            actor: &request.actor,
            idempotency_key: &request.idempotency_key,
            object_roots: &request.object_roots,
            operation: &request.operation,
            semantic: &request.semantic,
            kind: "codefire_remote_publication_request",
            version: 1,
        }
    }
}

fn normalized_request_value(request: &PublicationRequest) -> Value {
    json!({
        "type": "codefire_remote_publication_request",
        "version": 1,
        "operation": request.operation,
        "actor": request.actor,
        "idempotency_key": request.idempotency_key,
        "object_roots": request.object_roots,
        "semantic": request.semantic,
    })
}

fn receipt_authority(request: &PublicationRequest) -> Result<Option<RemoteAuthority>, CliError> {
    let Some(key) = request.idempotency_key.as_deref() else {
        return Ok(None);
    };
    let mut digest = Sha256::new();
    digest.update(b"codefire-remote-idempotency-v2\0");
    digest.update(request.actor.as_bytes());
    digest.update(b"\0");
    digest.update(request.operation.as_bytes());
    digest.update(b"\0");
    digest.update(key.as_bytes());
    let digest = codefire_util::hex_lower(&digest.finalize());
    Ok(Some(RemoteAuthority::IdempotencyReceipt {
        operation: request.operation.clone(),
        digest,
    }))
}

fn prepare_publication(
    project: &RemoteProject,
    request: &PublicationRequest,
    writes: &[AuthorityWrite],
    result: &Value,
    request_hash: &str,
) -> Result<(), CliError> {
    for write in writes {
        ensure_preimage(project, &write.authority, write.before.as_ref())?;
    }
    let transaction_id = random_transaction_id()?;
    let transaction = project.create_transaction(&transaction_id)?;
    let mut records = Vec::with_capacity(writes.len());
    for (index, write) in writes.iter().enumerate() {
        let bytes = codefire_store::canonical_json(&write.after)?;
        transaction.write_create_noreplace(project, &format!("write-{index}.json"), &bytes)?;
        let mut record = json!({
            "authority": write.authority.record(),
            "before_absent": write.before.is_none(),
            "before_hash": write.before.as_ref().map(json_hash).transpose()?,
            "after_hash": sha256_hex(&bytes),
            "postimage_index": index,
        });
        if write.authority == RemoteAuthority::RequestNonceCache {
            record["nonce_key"] = Value::String(added_nonce_cache_key(write)?);
        }
        records.push(record);
    }
    let pending = json!({
        "type": "remote_publication_transaction",
        "version": 1,
        "status": "pending",
        "phase": "applying",
        "transaction_id": transaction_id,
        "operation": request.operation,
        "actor": request.actor,
        "idempotency_key": request.idempotency_key,
        "request_hash": request_hash,
        "object_roots": request.object_roots.iter().map(|root| json!({
            "kind": "sealed_commit",
            "id": root,
        })).collect::<Vec<_>>(),
        "writes": records,
        "published_count": 0,
        "result": result,
        "created_at": now_iso_utc(),
    });
    let bytes = codefire_store::canonical_json(&pending)?;
    if bytes.len() as u64 > PENDING_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(
            "remote publication pending marker exceeds its bounded limit".to_string(),
        ));
    }
    project.write_authority_create_noreplace(&RemoteAuthority::PendingTransaction, &bytes)?;
    Ok(())
}

fn recover_publication(
    gate: &RemoteProjectGate,
    project_root: &Path,
    request: &PublicationRequest,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    mut pending: Value,
) -> Result<Value, CliError> {
    gate.ensure_exclusive(project_root)?;
    let project = gate.project();
    validate_pending_owner(&pending, request, request_hash)?;
    validate_object_roots(project_root, request)?;
    let transaction_id = required_pending_string(&pending, "transaction_id")?;
    super::validate_remote_component(&transaction_id, "transaction ID")?;
    let records = pending
        .get("writes")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending writes are missing".to_string())
        })?;
    if records.is_empty() || records.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(
            "remote pending write count is invalid".to_string(),
        ));
    }
    let mut published = pending
        .get("published_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending progress is missing".to_string())
        })? as usize;
    if published > records.len() {
        return Err(CliError::InvalidRepository(
            "remote pending progress exceeds its write set".to_string(),
        ));
    }
    let transaction = match project.open_transaction(&transaction_id) {
        Ok(transaction) => Some(transaction),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if pending.get("status").and_then(Value::as_str) == Some("completed") {
        if published != records.len() {
            return Err(CliError::InvalidRepository(
                "completed remote pending transaction has incomplete progress".to_string(),
            ));
        }
        return finish_completed_pending(
            project,
            request,
            request_hash,
            receipt,
            &pending,
            &records,
            transaction.as_ref(),
        );
    }
    let transaction = transaction.ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "pending remote transaction staging is missing: {transaction_id}"
        ))
    })?;
    let mut expected_stages = BTreeMap::new();
    let mut authority_paths = BTreeSet::new();
    let mut recovered_writes = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        validate_pending_write_record_shape(record, index)?;
        let authority =
            RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote pending authority is missing".to_string())
            })?)?;
        if authority == RemoteAuthority::PendingTransaction {
            return Err(CliError::InvalidRepository(
                "remote pending transaction cannot write itself".to_string(),
            ));
        }
        let postimage_index = record
            .get("postimage_index")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote postimage index is missing".to_string())
            })? as usize;
        if postimage_index != index {
            return Err(CliError::InvalidRepository(
                "remote postimage index is not canonical".to_string(),
            ));
        }
        let stage_name = format!("write-{index}.json");
        let after_bytes = transaction.read_bounded(project, &stage_name, POSTIMAGE_BYTE_LIMIT)?;
        let after_hash = required_pending_string(record, "after_hash")?;
        expected_stages.insert(stage_name.clone(), after_hash.clone());
        validate_sha256_hex(&after_hash, "remote pending postimage")?;
        if sha256_hex(&after_bytes) != after_hash {
            return Err(CliError::InvalidRepository(
                "remote staged postimage hash diverged".to_string(),
            ));
        }
        let after: Value = serde_json::from_slice(&after_bytes)?;
        if codefire_store::canonical_json(&after)? != after_bytes {
            return Err(CliError::InvalidRepository(
                "remote staged postimage is not canonical JSON".to_string(),
            ));
        }
        let authority_path = authority.path(project_root)?;
        if !authority_paths.insert(authority_path.clone()) {
            return Err(CliError::InvalidRepository(
                "remote pending transaction contains duplicate authorities".to_string(),
            ));
        }
        let current_hash = optional_authority_json_hash(project, &authority)?;
        let before_absent = record
            .get("before_absent")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote pending preimage kind is missing".to_string())
            })?;
        let before_hash = match (before_absent, record.get("before_hash")) {
            (true, Some(Value::Null)) => None,
            (false, Some(Value::String(hash))) => {
                validate_sha256_hex(hash, "remote pending preimage")?;
                Some(hash.as_str())
            }
            _ => {
                return Err(CliError::InvalidRepository(
                    "remote pending before_absent/before_hash shape is invalid".to_string(),
                ))
            }
        };
        let is_before = if before_absent {
            current_hash.is_none()
        } else {
            current_hash.as_deref() == before_hash
        };
        let is_after = current_hash.as_deref() == Some(after_hash.as_str());
        if index < published && !is_after {
            return Err(authority_divergence(&authority_path));
        }
        if index > published && !is_before {
            return Err(authority_divergence(&authority_path));
        }
        if index == published && !is_before && !is_after {
            return Err(authority_divergence(&authority_path));
        }
        recovered_writes.push(RecoveredWrite {
            authority,
            authority_path,
            after_bytes,
            after_hash,
            is_before,
            is_after,
        });
    }
    let actual_stage_names = transaction
        .entry_names_bounded(project, MAX_AUTHORITY_WRITES)?
        .into_iter()
        .map(|name| {
            name.into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "remote transaction staging contains a non-UTF-8 entry".to_string(),
                )
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual_stage_names != expected_stages.keys().cloned().collect() {
        return Err(CliError::InvalidRepository(
            "remote transaction staging set diverged before recovery".to_string(),
        ));
    }
    for (index, recovered) in recovered_writes.iter().enumerate().skip(published) {
        if recovered.is_before {
            project.write_authority_atomic(&recovered.authority, &recovered.after_bytes)?;
            if optional_authority_json_hash(project, &recovered.authority)?.as_deref()
                != Some(recovered.after_hash.as_str())
            {
                return Err(authority_divergence(&recovered.authority_path));
            }
        } else if !recovered.is_after {
            return Err(authority_divergence(&recovered.authority_path));
        }
        published = index + 1;
        pending["published_count"] = json!(published);
        write_pending(project, &pending)?;
    }
    let result = pending.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote pending result is missing".to_string())
    })?;
    if let Some(receipt_authority) = receipt {
        publish_completed_receipt(
            project,
            receipt_authority,
            request,
            request_hash,
            &records,
            &result,
        )?;
    }
    pending["status"] = Value::String("completed".to_string());
    pending["phase"] = Value::String("completed".to_string());
    pending["completed_at"] = Value::String(now_iso_utc());
    write_pending(project, &pending)?;
    cleanup_transaction_stage(project, &transaction, &expected_stages)?;
    release_pending(project, &transaction_id)?;
    Ok(result)
}

fn finish_completed_pending(
    project: &RemoteProject,
    request: &PublicationRequest,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    pending: &Value,
    records: &[Value],
    transaction: Option<&RemoteTransaction>,
) -> Result<Value, CliError> {
    let result = finish_completed_pending_body(
        project,
        request,
        request_hash,
        receipt,
        pending,
        records,
        transaction,
    )?;
    let transaction_id = required_pending_string(pending, "transaction_id")?;
    release_pending(project, &transaction_id)?;
    Ok(result)
}

fn finish_completed_pending_body(
    project: &RemoteProject,
    request: &PublicationRequest,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
    pending: &Value,
    records: &[Value],
    transaction: Option<&RemoteTransaction>,
) -> Result<Value, CliError> {
    let transaction_id = required_pending_string(pending, "transaction_id")?;
    let effects = effects_from_pending_records(records)?;
    let mut expected_stages = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        validate_pending_write_record_shape(record, index)?;
        let authority =
            RemoteAuthority::from_record(record.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote pending authority is missing".to_string())
            })?)?;
        let after_hash = required_pending_string(record, "after_hash")?;
        let postimage_index = record
            .get("postimage_index")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote postimage index is missing".to_string())
            })? as usize;
        if postimage_index != index {
            return Err(CliError::InvalidRepository(
                "remote postimage index is not canonical".to_string(),
            ));
        }
        match (
            record.get("before_absent").and_then(Value::as_bool),
            record.get("before_hash"),
        ) {
            (Some(true), Some(Value::Null)) => {}
            (Some(false), Some(Value::String(hash))) => {
                validate_sha256_hex(hash, "remote pending preimage")?;
            }
            _ => {
                return Err(CliError::InvalidRepository(
                    "remote pending before_absent/before_hash shape is invalid".to_string(),
                ))
            }
        }
        if optional_authority_json_hash(project, &authority)?.as_deref()
            != Some(after_hash.as_str())
        {
            return Err(authority_divergence(
                &authority.path(project.display_root())?,
            ));
        }
        expected_stages.insert(format!("write-{index}.json"), after_hash);
    }
    let result = pending.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote pending result is missing".to_string())
    })?;
    if let Some(receipt) = receipt {
        let replayed =
            load_completed_receipt(project, receipt, request, request_hash, &effects, &result)?
                .ok_or_else(|| {
                    CliError::InvalidRepository(
                        "completed remote transaction is missing its v2 receipt".to_string(),
                    )
                })?;
        if replayed != result {
            return Err(CliError::InvalidRepository(
                "completed remote transaction receipt result diverged".to_string(),
            ));
        }
    }
    if let Some(transaction) = transaction {
        cleanup_transaction_stage(project, transaction, &expected_stages)?;
    } else {
        project.remove_transaction_quarantine_if_present(&transaction_id)?;
    }
    Ok(result)
}

fn recover_completed_pending_quarantine(
    project: &RemoteProject,
    request: &PublicationRequest,
    request_hash: &str,
    receipt: Option<&RemoteAuthority>,
) -> Result<Option<Value>, CliError> {
    let Some((name, bytes, identity)) = project.pending_quarantine_bounded(PENDING_BYTE_LIMIT)?
    else {
        return Ok(None);
    };
    let pending: Value = serde_json::from_slice(&bytes)?;
    validate_pending_owner(&pending, request, request_hash)?;
    if pending.get("status").and_then(Value::as_str) != Some("completed")
        || pending.get("phase").and_then(Value::as_str) != Some("completed")
    {
        return Err(CliError::InvalidRepository(
            "non-completed remote pending quarantine is preserved".to_string(),
        ));
    }
    let transaction_id = required_pending_string(&pending, "transaction_id")?;
    super::validate_remote_component(&transaction_id, "transaction ID")?;
    let expected_name = format!(".pending_transaction.removing-{transaction_id}.json");
    if name != OsStr::new(&expected_name) {
        return Err(CliError::InvalidRepository(
            "remote pending quarantine name does not match its transaction".to_string(),
        ));
    }
    let records = pending
        .get("writes")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote pending quarantine writes are missing".to_string())
        })?;
    if records.is_empty()
        || records.len() > MAX_AUTHORITY_WRITES
        || pending.get("published_count").and_then(Value::as_u64) != Some(records.len() as u64)
    {
        return Err(CliError::InvalidRepository(
            "remote pending quarantine progress is invalid".to_string(),
        ));
    }
    let transaction = match project.open_transaction(&transaction_id) {
        Ok(transaction) => Some(transaction),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let result = finish_completed_pending_body(
        project,
        request,
        request_hash,
        receipt,
        &pending,
        &records,
        transaction.as_ref(),
    )?;
    project.remove_pending_quarantine_exact(&name, identity)?;
    Ok(Some(result))
}

fn validate_pending_owner(
    pending: &Value,
    request: &PublicationRequest,
    request_hash: &str,
) -> Result<(), CliError> {
    let object = pending.as_object().ok_or_else(|| {
        CliError::InvalidRepository("remote pending transaction is not an object".to_string())
    })?;
    let shape = pending.get("type").and_then(Value::as_str)
        == Some("remote_publication_transaction")
        && pending.get("version").and_then(Value::as_u64) == Some(1);
    if !shape {
        return Err(CliError::InvalidRepository(
            "foreign or future remote pending transaction is preserved".to_string(),
        ));
    }
    validate_sha256_hex(request_hash, "remote pending request")?;
    let expected_object_roots = Value::Array(
        request
            .object_roots
            .iter()
            .map(|root| json!({"kind": "sealed_commit", "id": root}))
            .collect(),
    );
    if pending.get("request_hash").and_then(Value::as_str) != Some(request_hash)
        || pending.get("operation").and_then(Value::as_str) != Some(&request.operation)
        || pending.get("actor").and_then(Value::as_str) != Some(&request.actor)
        || pending.get("idempotency_key")
            != Some(
                &request
                    .idempotency_key
                    .clone()
                    .map_or(Value::Null, Value::String),
            )
        || pending.get("object_roots") != Some(&expected_object_roots)
    {
        return Err(CliError::IdempotencyConflict(
            "a different remote publication transaction is pending".to_string(),
        ));
    }
    let status = pending.get("status").and_then(Value::as_str);
    if status != Some("pending") && status != Some("completed") {
        return Err(CliError::InvalidRepository(
            "remote pending transaction status is invalid".to_string(),
        ));
    }
    let phase = pending.get("phase").and_then(Value::as_str);
    if (status == Some("pending") && phase != Some("applying"))
        || (status == Some("completed") && phase != Some("completed"))
    {
        return Err(CliError::InvalidRepository(
            "remote pending transaction status/phase pair is invalid".to_string(),
        ));
    }
    if pending.get("result").is_none()
        || pending.get("created_at").and_then(Value::as_str).is_none()
        || (status == Some("completed")
            && pending
                .get("completed_at")
                .and_then(Value::as_str)
                .is_none())
    {
        return Err(CliError::InvalidRepository(
            "remote pending transaction immutable result or timestamp is missing".to_string(),
        ));
    }
    let mut expected_fields = BTreeSet::from([
        "type",
        "version",
        "status",
        "phase",
        "transaction_id",
        "operation",
        "actor",
        "idempotency_key",
        "request_hash",
        "object_roots",
        "writes",
        "published_count",
        "result",
        "created_at",
    ]);
    if status == Some("completed") {
        expected_fields.insert("completed_at");
    }
    let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual_fields != expected_fields {
        return Err(CliError::InvalidRepository(
            "remote pending transaction has unknown or missing fields".to_string(),
        ));
    }
    Ok(())
}

fn publish_completed_receipt(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    request: &PublicationRequest,
    request_hash: &str,
    writes: &[Value],
    result: &Value,
) -> Result<(), CliError> {
    let effects = effects_from_pending_records(writes)?;
    let result_hash = json_hash(result)?;
    let receipt = json!({
        "type": "remote_idempotency_receipt",
        "version": 2,
        "status": "completed",
        "operation": request.operation,
        "actor": request.actor,
        "key": request.idempotency_key,
        "request_hash": request_hash,
        "effects": effects,
        "result": result,
        "result_hash": result_hash,
        "completed_at": now_iso_utc(),
    });
    let bytes = codefire_store::canonical_json(&receipt)?;
    match project.write_authority_create_noreplace(authority, &bytes) {
        Ok(()) => Ok(()),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::AlreadyExists => {
            let existing = project.read_authority_bounded(authority, PENDING_BYTE_LIMIT)?;
            let value: Value = serde_json::from_slice(&existing)?;
            validate_completed_receipt_value(&value, request, request_hash, &effects, result)
                .map(drop)
        }
        Err(error) => Err(error),
    }
}

fn load_completed_receipt(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    request: &PublicationRequest,
    request_hash: &str,
    expected_effects: &[Value],
    expected_result: &Value,
) -> Result<Option<Value>, CliError> {
    let Some(receipt) = read_optional_authority_bounded(project, authority, PENDING_BYTE_LIMIT)?
    else {
        return Ok(None);
    };
    validate_completed_receipt_value(
        &receipt,
        request,
        request_hash,
        expected_effects,
        expected_result,
    )
    .map(Some)
}

fn load_completed_receipt_preserving_effects(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    request: &PublicationRequest,
    request_hash: &str,
    expected_result: &Value,
) -> Result<Option<Value>, CliError> {
    let Some(receipt) = read_optional_authority_bounded(project, authority, PENDING_BYTE_LIMIT)?
    else {
        return Ok(None);
    };
    let effects = validate_receipt_effects(&receipt)?;
    validate_completed_receipt_value(&receipt, request, request_hash, &effects, expected_result)
        .map(Some)
}

fn validate_completed_receipt_value(
    receipt: &Value,
    request: &PublicationRequest,
    request_hash: &str,
    expected_effects: &[Value],
    expected_result: &Value,
) -> Result<Value, CliError> {
    let actual_result_hash = receipt.get("result").map(json_hash).transpose()?;
    validate_completed_receipt_value_with_hash(
        receipt,
        request,
        request_hash,
        expected_effects,
        expected_result,
        actual_result_hash.as_deref(),
    )
}

fn validate_completed_receipt_value_with_hash(
    receipt: &Value,
    request: &PublicationRequest,
    request_hash: &str,
    expected_effects: &[Value],
    expected_result: &Value,
    actual_result_hash: Option<&str>,
) -> Result<Value, CliError> {
    let valid_shape = receipt.get("type").and_then(Value::as_str)
        == Some("remote_idempotency_receipt")
        && receipt.get("version").and_then(Value::as_u64) == Some(2)
        && receipt.get("status").and_then(Value::as_str) == Some("completed");
    if !valid_shape {
        return Err(CliError::InvalidRepository(
            "remote v2 idempotency receipt is malformed or future".to_string(),
        ));
    }
    if receipt.get("request_hash").and_then(Value::as_str) != Some(request_hash) {
        return Err(CliError::IdempotencyConflict(
            "remote v2 idempotency key was already used for another request".to_string(),
        ));
    }
    let expected_key = request
        .idempotency_key
        .clone()
        .map_or(Value::Null, Value::String);
    if receipt.get("operation").and_then(Value::as_str) != Some(&request.operation)
        || receipt.get("actor").and_then(Value::as_str) != Some(&request.actor)
        || receipt.get("key") != Some(&expected_key)
        || receipt.get("effects") != Some(&Value::Array(expected_effects.to_vec()))
    {
        return Err(CliError::InvalidRepository(
            "remote v2 receipt scope or effects diverged".to_string(),
        ));
    }
    let result = receipt.get("result").cloned().ok_or_else(|| {
        CliError::InvalidRepository("remote v2 receipt result is missing".to_string())
    })?;
    let result_hash = receipt
        .get("result_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote v2 receipt result hash is missing".to_string())
        })?;
    validate_sha256_hex(result_hash, "remote v2 receipt result")?;
    if &result != expected_result || actual_result_hash != Some(result_hash) {
        return Err(CliError::InvalidRepository(
            "remote v2 receipt result diverged".to_string(),
        ));
    }
    Ok(result)
}

fn effects_from_pending_records(records: &[Value]) -> Result<Vec<Value>, CliError> {
    if records.is_empty() || records.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(
            "remote receipt effect count is invalid".to_string(),
        ));
    }
    let mut paths = BTreeSet::new();
    let effects = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            validate_pending_write_record_shape(record, index)?;
            let authority_record = record.get("authority").cloned().ok_or_else(|| {
                CliError::InvalidRepository("receipt authority is missing".to_string())
            })?;
            let authority = RemoteAuthority::from_record(&authority_record)?;
            if authority == RemoteAuthority::RequestNonceCache {
                return Ok(None);
            }
            let path = authority.path(Path::new("/receipt-authority"))?;
            if !paths.insert(path) {
                return Err(CliError::InvalidRepository(
                    "remote receipt contains duplicate effects".to_string(),
                ));
            }
            let after_hash = required_pending_string(record, "after_hash")?;
            validate_sha256_hex(&after_hash, "remote receipt effect")?;
            Ok(Some(json!({
                "authority": authority_record,
                "after_hash": after_hash,
            })))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    Ok(effects.into_iter().flatten().collect())
}

fn validate_receipt_effects(receipt: &Value) -> Result<Vec<Value>, CliError> {
    let effects = receipt
        .get("effects")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidRepository("remote v2 receipt effects are missing".to_string())
        })?;
    if effects.is_empty() || effects.len() > MAX_AUTHORITY_WRITES {
        return Err(CliError::InvalidRepository(
            "remote v2 receipt effect count is invalid".to_string(),
        ));
    }
    let expected_fields = BTreeSet::from(["authority", "after_hash"]);
    let mut authorities = BTreeSet::new();
    for effect in &effects {
        let object = effect.as_object().ok_or_else(|| {
            CliError::InvalidRepository("remote v2 receipt effect is not an object".to_string())
        })?;
        if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_fields {
            return Err(CliError::InvalidRepository(
                "remote v2 receipt effect has unknown or missing fields".to_string(),
            ));
        }
        let authority =
            RemoteAuthority::from_record(effect.get("authority").ok_or_else(|| {
                CliError::InvalidRepository("remote v2 receipt authority is missing".to_string())
            })?)?;
        if !authorities.insert(authority.path(Path::new("/receipt-authority"))?) {
            return Err(CliError::InvalidRepository(
                "remote v2 receipt contains duplicate effects".to_string(),
            ));
        }
        validate_sha256_hex(
            &required_pending_string(effect, "after_hash")?,
            "remote v2 receipt effect",
        )?;
    }
    Ok(effects)
}

fn validate_pending_write_record_shape(record: &Value, index: usize) -> Result<(), CliError> {
    let object = record.as_object().ok_or_else(|| {
        CliError::InvalidRepository("remote pending write is not an object".to_string())
    })?;
    let mut expected = BTreeSet::from([
        "authority",
        "before_absent",
        "before_hash",
        "after_hash",
        "postimage_index",
    ]);
    let is_nonce = record
        .get("authority")
        .and_then(|authority| authority.get("kind"))
        .and_then(Value::as_str)
        == Some("request_nonce_cache");
    if is_nonce {
        expected.insert("nonce_key");
    }
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected
        || record.get("postimage_index").and_then(Value::as_u64) != Some(index as u64)
        || (is_nonce && record.get("nonce_key").and_then(Value::as_str).is_none())
    {
        return Err(CliError::InvalidRepository(
            "remote pending write has unknown, missing, or noncanonical fields".to_string(),
        ));
    }
    Ok(())
}

fn effects_from_authority_writes(writes: &[AuthorityWrite]) -> Result<Vec<Value>, CliError> {
    writes
        .iter()
        .filter(|write| write.authority != RemoteAuthority::RequestNonceCache)
        .map(|write| {
            Ok(json!({
                "authority": write.authority.record(),
                "after_hash": json_hash(&write.after)?,
            }))
        })
        .collect()
}

fn ensure_preimage(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    expected: Option<&Value>,
) -> Result<(), CliError> {
    let expected_hash = expected.map(streaming_json_hash).transpose()?;
    capture_authority_preimage(project, authority, expected, expected_hash.as_deref()).map(drop)
}

fn capture_authority_preimage(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    expected: Option<&Value>,
    expected_hash: Option<&str>,
) -> Result<PreparedAuthorityPreimage, CliError> {
    let preimage = optional_authority_json_preimage(project, authority)?;
    if expected.is_some() != expected_hash.is_some() || preimage.hash.as_deref() != expected_hash {
        return Err(authority_divergence(
            &authority.path(project.display_root())?,
        ));
    }
    Ok(preimage)
}

fn optional_authority_json_hash(
    project: &RemoteProject,
    authority: &RemoteAuthority,
) -> Result<Option<String>, CliError> {
    optional_authority_json_preimage(project, authority).map(|preimage| preimage.hash)
}

fn optional_authority_json_preimage(
    project: &RemoteProject,
    authority: &RemoteAuthority,
) -> Result<PreparedAuthorityPreimage, CliError> {
    let (bytes, identity) =
        match project.read_authority_bounded_with_identity(authority, POSTIMAGE_BYTE_LIMIT) {
            Ok(value) => value,
            Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                return Ok(PreparedAuthorityPreimage {
                    authority: authority.clone(),
                    identity: None,
                    hash: None,
                })
            }
            Err(error) => return Err(error),
        };
    let value: Value = serde_json::from_slice(&bytes)?;
    if !json_matches_canonical_bytes(&value, &bytes)? {
        return Err(CliError::InvalidRepository(format!(
            "remote authority is not canonical JSON: {}",
            authority.path(project.display_root())?.display()
        )));
    }
    Ok(PreparedAuthorityPreimage {
        authority: authority.clone(),
        identity: Some(identity),
        hash: Some(sha256_hex(&bytes)),
    })
}

fn json_matches_canonical_bytes(value: &Value, expected: &[u8]) -> Result<bool, CliError> {
    struct Comparator<'a> {
        expected: &'a [u8],
        offset: usize,
        matches: bool,
    }
    impl Write for Comparator<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let end = self
                .offset
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("canonical JSON comparison size overflow"))?;
            if end > self.expected.len() || &self.expected[self.offset..end] != bytes {
                self.matches = false;
            }
            self.offset = end;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut comparator = Comparator {
        expected,
        offset: 0,
        matches: true,
    };
    serde_json::to_writer(&mut comparator, value)?;
    Ok(comparator.matches && comparator.offset == expected.len())
}

fn json_hash(value: &Value) -> Result<String, CliError> {
    Ok(sha256_hex(&codefire_store::canonical_json(value)?))
}

fn read_optional_authority_bounded(
    project: &RemoteProject,
    authority: &RemoteAuthority,
    limit: u64,
) -> Result<Option<Value>, CliError> {
    match project.read_authority_bounded(authority, limit) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn write_pending(project: &RemoteProject, pending: &Value) -> Result<(), CliError> {
    let bytes = codefire_store::canonical_json(pending)?;
    if bytes.len() as u64 > PENDING_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(
            "remote pending transaction grew beyond its bounded limit".to_string(),
        ));
    }
    project.write_authority_atomic(&RemoteAuthority::PendingTransaction, &bytes)
}

fn preflight_cleanup_stage(
    project: &RemoteProject,
    transaction: &RemoteTransaction,
    expected_stages: &BTreeMap<String, String>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
) -> Result<(u64, u64), CliError> {
    let actual = transaction
        .entry_names_bounded(project, MAX_AUTHORITY_WRITES * 2)?
        .into_iter()
        .map(|name| {
            name.into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "remote transaction staging contains a non-UTF-8 entry".to_string(),
                )
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let quarantine_name = |name: &str| format!(".{name}.removing-{}", transaction.id());
    let mut allowed = expected_stages.keys().cloned().collect::<BTreeSet<_>>();
    allowed.extend(expected_stages.keys().map(|name| quarantine_name(name)));
    if !actual.is_subset(&allowed) {
        return Err(CliError::InvalidRepository(format!(
            "remote transaction staging contains foreign entries for {}",
            project.display_root().display()
        )));
    }
    let mut total = 0_u64;
    let mut records = 0_u64;
    for (name, expected_hash) in expected_stages {
        let quarantine = quarantine_name(name);
        let has_original = actual.contains(name);
        let has_quarantine = actual.contains(&quarantine);
        if has_original && has_quarantine {
            return Err(CliError::InvalidRepository(
                "remote transaction staging contains both original and quarantine entries"
                    .to_string(),
            ));
        }
        let present_name = if has_original {
            name.as_str()
        } else if has_quarantine {
            quarantine.as_str()
        } else {
            continue;
        };
        let display_path = project
            .display_root()
            .join("transactions")
            .join(transaction.id())
            .join(present_name);
        acquire_publication_read(ledger, &display_path, POSTIMAGE_BYTE_LIMIT, temporary_bytes)?;
        let bytes = transaction.read_bounded(project, present_name, POSTIMAGE_BYTE_LIMIT)?;
        if sha256_hex(&bytes) != *expected_hash {
            return Err(CliError::InvalidRepository(
                "remote transaction staging content changed before cleanup".to_string(),
            ));
        }
        total = total.checked_add(bytes.len() as u64).ok_or_else(|| {
            CliError::InvalidRepository("publication byte size overflow".to_string())
        })?;
        records = records.checked_add(1).ok_or_else(|| {
            CliError::InvalidRepository("publication record count overflow".to_string())
        })?;
    }
    Ok((total, records))
}

fn cleanup_transaction_stage(
    project: &RemoteProject,
    transaction: &RemoteTransaction,
    expected_stages: &BTreeMap<String, String>,
) -> Result<(), CliError> {
    let actual = transaction
        .entry_names_bounded(project, MAX_AUTHORITY_WRITES * 2)?
        .into_iter()
        .map(|name| {
            name.into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "remote transaction staging contains a non-UTF-8 entry".to_string(),
                )
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let quarantine_name = |name: &str| format!(".{name}.removing-{}", transaction.id());
    let mut allowed = expected_stages.keys().cloned().collect::<BTreeSet<_>>();
    allowed.extend(expected_stages.keys().map(|name| quarantine_name(name)));
    if !actual.is_subset(&allowed) {
        return Err(CliError::InvalidRepository(format!(
            "remote transaction staging contains foreign entries for {}",
            project.display_root().display()
        )));
    }
    for (name, expected_hash) in expected_stages {
        let quarantine = quarantine_name(name);
        let has_original = actual.contains(name);
        let has_quarantine = actual.contains(&quarantine);
        if has_original && has_quarantine {
            return Err(CliError::InvalidRepository(
                "remote transaction staging contains both original and quarantine entries"
                    .to_string(),
            ));
        }
        if has_original {
            let (bytes, identity) =
                transaction.read_bounded_with_identity(project, name, POSTIMAGE_BYTE_LIMIT)?;
            if sha256_hex(&bytes) != *expected_hash {
                return Err(CliError::InvalidRepository(
                    "remote transaction staging content changed before cleanup".to_string(),
                ));
            }
            transaction.remove_file_exact(project, OsStr::new(name), identity)?;
        } else if has_quarantine {
            let (bytes, identity) = transaction.read_bounded_with_identity(
                project,
                &quarantine,
                POSTIMAGE_BYTE_LIMIT,
            )?;
            if sha256_hex(&bytes) != *expected_hash {
                return Err(CliError::InvalidRepository(
                    "remote transaction quarantine content changed before cleanup".to_string(),
                ));
            }
            transaction.remove_quarantined_file_exact(
                project,
                OsStr::new(&quarantine),
                identity,
            )?;
        }
    }
    project.remove_transaction_directory(transaction)?;
    Ok(())
}

fn release_pending(project: &RemoteProject, transaction_id: &str) -> Result<(), CliError> {
    project.ensure_bound()?;
    let authority = RemoteAuthority::PendingTransaction;
    let (bytes, identity) =
        project.read_authority_bounded_with_identity(&authority, PENDING_BYTE_LIMIT)?;
    let pending: Value = serde_json::from_slice(&bytes)?;
    if pending.get("transaction_id").and_then(Value::as_str) != Some(transaction_id)
        || pending.get("status").and_then(Value::as_str) != Some("completed")
    {
        return Err(CliError::InvalidRepository(
            "remote pending marker changed before release".to_string(),
        ));
    }
    let quarantine = format!(".pending_transaction.removing-{transaction_id}.json");
    project.remove_authority_quarantined(&authority, quarantine.as_ref(), identity)?;
    Ok(())
}

fn required_pending_string(value: &Value, field: &str) -> Result<String, CliError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!("remote pending field {field} is missing"))
        })
}

fn validate_sha256_hex(value: &str, kind: &str) -> Result<(), CliError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::InvalidRepository(format!(
            "{kind} hash is invalid"
        )));
    }
    Ok(())
}

fn authority_divergence(path: &Path) -> CliError {
    CliError::InvalidRepository(format!(
        "remote publication authority diverged from its exact pre/postimage: {}",
        path.display()
    ))
}

fn random_transaction_id() -> Result<String, CliError> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate remote publication transaction ID: {error}"
        ))
    })?;
    let mut encoded = String::with_capacity(32);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    Ok(format!("tx_{encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::project::ensure_remote_project_layout;
    use crate::LockOptions;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn fixture(label: &str) -> (PathBuf, PathBuf, RemoteProjectGate) {
        let root = std::env::temp_dir().join(format!(
            "codefire-publication-{label}-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        ensure_remote_project_layout(&project).unwrap();
        let gate = RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap();
        (root, project, gate)
    }

    fn request(key: Option<&str>, semantic: &str) -> PublicationRequest {
        PublicationRequest {
            operation: "fixture_publish".to_string(),
            actor: "fixture-actor".to_string(),
            idempotency_key: key.map(str::to_string),
            object_roots: Vec::new(),
            semantic: json!({"semantic": semantic}),
        }
    }

    fn branch_write(name: &str) -> AuthorityWrite {
        AuthorityWrite {
            authority: RemoteAuthority::Branch(name.to_string()),
            before: None,
            after: json!({"version": 1, "name": name, "head": "CF-COMMIT-fixture"}),
        }
    }

    fn http_ledger(command: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            command,
        )
        .unwrap()
    }

    #[test]
    fn prepared_publication_commits_after_seal_without_ordinary_apply() {
        let (root, project_root, gate) = fixture("prepared-sealed");
        let request = request(Some("prepared-sealed"), "sealed");
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});
        let mut ledger = http_ledger("prepared-publication");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut ledger,
        )
        .unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(prepared.retained_bytes)
        );
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        let (cleanup_stage_bytes, cleanup_stage_records, pending_release_bytes) =
            prepared_cleanup_work(
                prepared
                    .plan
                    .as_ref()
                    .expect("prepared publication must retain its plan"),
            );
        revalidate_prepared_json_publication(&mut prepared, &mut ledger).unwrap();
        let hashed_before_commit = ledger
            .snapshot()
            .counters
            .get(&codefire_core::DecisionWorkCounter::BytesHashed)
            .copied()
            .unwrap_or_default();
        ledger.seal_prepublication().unwrap();
        assert_eq!(
            ledger.apply(codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )),
            Err(codefire_core::DecisionWorkAccountingError::PrepublicationSealed)
        );
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut ledger).unwrap(),
            result
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
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
            cleanup_stage_records + 1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FileBytesRead],
            cleanup_stage_bytes + pending_release_bytes
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            hashed_before_commit + cleanup_stage_bytes
        );
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publication_budget_failure_has_no_repository_effect() {
        let (root, project_root, gate) = fixture("prepared-budget");
        let request = request(None, "budget");
        let writes = [branch_write("main")];
        let mut ledger = http_ledger("publication-budget");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    1024 * 1024 * 1024,
                ),
            )
            .unwrap();
        let error = match prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &json!({"status": "never-published"}),
            &mut ledger,
        ) {
            Ok(_) => panic!("publication reservation unexpectedly succeeded"),
            Err(error) => error,
        };
        assert!(matches!(error, CliError::ResourceBudget(_)));
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        assert!(!RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publication_hash_accounting_accepts_limit_and_rejects_limit_plus_one_atomically() {
        let value = json!({"subject": "publication-hash-boundary"});
        let len = canonical_json_len(&value).unwrap();
        let mut exact = http_ledger("publication-hash-exact");
        let limit = exact
            .profile()
            .limit(codefire_core::DecisionWorkDimension::AggregateBytes)
            .unwrap()
            .limit;
        exact
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    limit - len,
                ),
            )
            .unwrap();
        accounted_json_hash(&value, &mut exact).unwrap();
        assert_eq!(
            exact.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            len
        );

        let mut exceeded = http_ledger("publication-hash-exceeded");
        exceeded
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    limit - len + 1,
                ),
            )
            .unwrap();
        assert!(matches!(
            accounted_json_hash(&value, &mut exceeded),
            Err(CliError::ResourceBudget(_))
        ));
        assert_eq!(
            exceeded
                .snapshot()
                .counters
                .get(&codefire_core::DecisionWorkCounter::BytesHashed)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn publication_rejects_unbound_root_token_before_work_or_write() {
        let (root, project_root, gate) = fixture("publication-root-token");
        let mut request = request(None, "root-token");
        request.object_roots = vec!["CF-COMMIT-unvalidated".to_string()];
        let mut session =
            codefire_store::begin_sealed_commit_validation_session(&project_root.join("objects"))
                .unwrap();
        session.finalize().unwrap();
        let validated = ValidatedPublicationRoots {
            stable_project_id: gate.project().stable_id().to_string(),
            object_store_subject: session.validated_object_store_subject().unwrap(),
            roots: Vec::new(),
            retained_bytes: 0,
        };
        let mut ledger = http_ledger("publication-root-token");
        let error = match prepare_owned_json_publication_with_validated_roots(
            &gate,
            &project_root,
            &request,
            &[branch_write("main")],
            &json!({"status": "never-published"}),
            validated,
            &mut ledger,
        ) {
            Ok(_) => panic!("unbound publication root token was accepted"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("do not match the finalized request validation subject"));
        assert!(ledger.snapshot().cumulative.is_empty());
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validated_root_token_exact_accounting_and_owner_drop_release_current_memory() {
        let (root, project_root, gate) = fixture("publication-root-token-owner");
        let request = request(None, "accounted-root-token");
        let token_bytes =
            codefire_core::logical_retained_frame_bytes(gate.project().stable_id()).unwrap();
        let mut session =
            codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
                &project_root.join("objects"),
                http_ledger("accounted publication-root token"),
            )
            .unwrap();
        session.finalize().unwrap();
        let retained_before = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or(0);

        let token =
            validated_publication_roots_from_session(&gate, &request, &mut session).unwrap();
        let retained_after = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or(0);
        assert_eq!(retained_after - retained_before, token_bytes);

        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            token_bytes
        );
        let owner = prepare_owned_json_publication_with_validated_roots(
            &gate,
            &project_root,
            &request,
            &[branch_write("main")],
            &json!({"status": "cancelled"}),
            token,
            &mut ledger,
        )
        .unwrap();
        assert!(
            owner
                .prepared
                .as_ref()
                .and_then(|prepared| prepared.validated_roots.as_ref())
                .is_some(),
            "the concrete validated-root token must remain owned by the publication"
        );
        drop(owner);
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publication_identity_outputs_reserve_limit_minus_one_exact_and_plus_one() {
        for offset in [-1_i64, 0, 1] {
            let (root, project_root, gate) = fixture(&format!("identity-scratch-{offset}"));
            let request = request(Some("identity-scratch-key"), "identity-scratch");
            let validated = empty_validated_publication_roots(&project_root, &request);
            let identity_bytes = publication_identity_temporary_bytes(&request).unwrap();
            let limit = codefire_core::DecisionWorkProfile::http_v1()
                .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
                .unwrap()
                .limit;
            let filler = if offset < 0 {
                limit - identity_bytes - offset.unsigned_abs()
            } else {
                limit - identity_bytes + offset as u64
            };
            let mut ledger = http_ledger("publication identity scratch boundary");
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, filler),
                )
                .unwrap();
            {
                let probed = probe_json_publication_with_validated_roots_and_ledger(
                    &gate,
                    &project_root,
                    &request,
                    &json!({"status": "fresh"}),
                    None,
                    &validated,
                    &mut ledger,
                );
                if offset == 1 {
                    assert!(matches!(probed, Err(CliError::ResourceBudget(_))));
                } else {
                    let PublicationProbe::Fresh {
                        request_hash,
                        receipt,
                        identity_temporary_bytes,
                    } = probed.unwrap()
                    else {
                        panic!("new request unexpectedly resolved as a replay")
                    };
                    assert_eq!(identity_temporary_bytes, identity_bytes);
                    assert_eq!(
                        ledger
                            .snapshot()
                            .current
                            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                            .copied(),
                        Some(filler + identity_bytes)
                    );
                    drop(request_hash);
                    drop(receipt);
                    release_publication_temporary(&mut ledger, identity_temporary_bytes).unwrap();
                }
            }
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied(),
                Some(filler)
            );
            ledger
                .release(codefire_core::DecisionWorkDimension::TemporaryBytes, filler)
                .unwrap();
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied()
                    .unwrap_or(0),
                0
            );
            drop(gate);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn fresh_builder_reservation_precedes_callback_at_memory_boundary() {
        for offset in [-1_i64, 0, 1] {
            let (root, project_root, gate) = fixture(&format!("builder-scratch-{offset}"));
            let request = request(None, "builder-scratch");
            let validated = empty_validated_publication_roots(&project_root, &request);
            let reserved = publication_identity_temporary_bytes(&request)
                .unwrap()
                .checked_add(authority_write_builder_temporary_bytes().unwrap())
                .unwrap();
            let limit = codefire_core::DecisionWorkProfile::http_v1()
                .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
                .unwrap()
                .limit;
            let filler = if offset < 0 {
                limit - reserved - offset.unsigned_abs()
            } else {
                limit - reserved + offset as u64
            };
            let mut ledger = http_ledger("fresh builder scratch boundary");
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, filler),
                )
                .unwrap();
            {
                let mut callback_called = false;
                let prepared = prepare_json_publication_lazy_with_nonce_validated_roots_and_ledger(
                    &gate,
                    &project_root,
                    &request,
                    PendingRequestNonce::recovered_from_matching_publication(),
                    &json!({"status": "fresh"}),
                    &validated,
                    &mut ledger,
                    |_| {
                        callback_called = true;
                        Ok(Vec::new())
                    },
                );
                if offset == 1 {
                    assert!(matches!(prepared, Err(CliError::ResourceBudget(_))));
                    assert!(!callback_called);
                } else {
                    assert!(matches!(prepared, Err(CliError::InvalidRepository(_))));
                    assert!(callback_called);
                }
            }
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied(),
                Some(filler)
            );
            ledger
                .release(codefire_core::DecisionWorkDimension::TemporaryBytes, filler)
                .unwrap();
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied()
                    .unwrap_or(0),
                0
            );
            drop(gate);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn cancelling_prepared_publication_releases_exact_owner_without_effect() {
        let (root, project_root, gate) = fixture("prepared-cancel");
        let request = request(None, "cancel");
        let writes = [branch_write("main")];
        let mut ledger = http_ledger("publication-cancel");
        let prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &json!({"status": "cancelled"}),
            &mut ledger,
        )
        .unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(prepared.retained_bytes)
        );
        cancel_prepared_json_publication(prepared, &mut ledger).unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        assert!(!RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn commit_without_preseal_revalidation_releases_all_owners_without_effect() {
        let (root, project_root, gate) = fixture("prepared-revalidation-required");
        let request = request(None, "revalidation-required");
        let writes = [branch_write("main")];
        let mut ledger = http_ledger("publication-revalidation-required");
        let prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &json!({"status": "never-published"}),
            &mut ledger,
        )
        .unwrap();

        ledger.seal_prepublication().unwrap();
        let error = commit_prepared_json_publication(prepared, &mut ledger).unwrap_err();
        assert!(error
            .to_string()
            .contains("was not revalidated immediately before sealing"));
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        assert!(!RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dropping_public_owner_cancels_permit_and_releases_retained_bytes() {
        let (root, project_root, gate) = fixture("prepared-owner-drop");
        let request = request(None, "owner-drop");
        let writes = [branch_write("main")];
        let result = json!({"status": "never-published"});
        let validated = empty_validated_publication_roots(&project_root, &request);
        let mut ledger = http_ledger("publication-owner-drop");
        {
            let owner = prepare_owned_json_publication_with_validated_roots(
                &gate,
                &project_root,
                &request,
                &writes,
                &result,
                validated,
                &mut ledger,
            )
            .unwrap();
            assert!(!owner.is_replay());
        }
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn public_owner_revalidates_seals_and_commits_as_one_terminal_operation() {
        let (root, project_root, gate) = fixture("prepared-owner-commit");
        let request = request(None, "owner-commit");
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});
        let validated = empty_validated_publication_roots(&project_root, &request);
        let mut ledger = http_ledger("publication-owner-commit");
        let owner = prepare_owned_json_publication_with_validated_roots(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            validated,
            &mut ledger,
        )
        .unwrap();
        assert_eq!(owner.seal_and_commit().unwrap(), result);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .is_file());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn raw_publication_plan_coexists_with_separate_permit_under_one_seal() {
        let (root, project_root, gate) = fixture("raw-plan-combined-owner");
        let request = request(None, "raw-plan-combined-owner");
        let ledger = http_ledger("raw publication combined owner");
        let mut session =
            codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
                &project_root.join("objects"),
                ledger,
            )
            .unwrap();
        session.finalize().unwrap();
        let validated =
            validated_publication_roots_from_session(&gate, &request, &mut session).unwrap();
        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();

        let separate_work = codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::AggregateBytes, 1);
        let mut separate_permit = ledger.reserve(separate_work.clone()).unwrap();
        let result = json!({"status": "published"});
        let mut prepared = prepare_json_publication_plan_lazy_with_nonce_and_validated_roots(
            &gate,
            &project_root,
            &request,
            PendingRequestNonce::recovered_from_matching_publication(),
            &result,
            validated,
            &mut ledger,
            |_| Ok(vec![branch_write("main")]),
        )
        .unwrap();
        assert!(!prepared.is_replay());

        prepared.revalidate_preseal(&mut ledger).unwrap();
        ledger.seal_prepublication().unwrap();
        ledger
            .consume_reserved(&mut separate_permit, separate_work)
            .unwrap();
        assert_eq!(prepared.commit_after_seal(&mut ledger).unwrap(), result);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .is_file());
        drop(prepared);
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn raw_publication_rejects_foreign_ledger_before_work_and_preserves_owner() {
        let (root, project_root, gate) = fixture("raw-plan-foreign-ledger");
        let request = request(None, "raw-plan-foreign-ledger");
        let result = json!({"status": "never-published"});
        let validated = empty_validated_publication_roots(&project_root, &request);
        let mut owner_ledger = http_ledger("raw publication owner ledger");
        let mut prepared = prepare_json_publication_plan_lazy_with_nonce_and_validated_roots(
            &gate,
            &project_root,
            &request,
            PendingRequestNonce::recovered_from_matching_publication(),
            &result,
            validated,
            &mut owner_ledger,
            |_| Ok(vec![branch_write("main")]),
        )
        .unwrap();

        let mut foreign_ledger = http_ledger("raw publication foreign ledger");
        let foreign_before = foreign_ledger.snapshot().clone();
        assert!(matches!(
            prepared.revalidate_preseal(&mut foreign_ledger),
            Err(CliError::InvalidRepository(message))
                if message.contains("different decision-work ledger")
        ));
        assert!(matches!(
            prepared.cancel(&mut foreign_ledger),
            Err(CliError::InvalidRepository(message))
                if message.contains("different decision-work ledger")
        ));
        foreign_ledger.seal_prepublication().unwrap();
        assert!(matches!(
            prepared.commit_after_seal(&mut foreign_ledger),
            Err(CliError::InvalidRepository(message))
                if message.contains("different decision-work ledger")
        ));
        assert_eq!(foreign_ledger.snapshot().current, foreign_before.current);
        assert_eq!(foreign_ledger.snapshot().counters, foreign_before.counters);
        assert!(prepared.plan.is_some());

        prepared.cancel(&mut owner_ledger).unwrap();
        assert!(prepared.plan.is_none());
        assert_eq!(
            owner_ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        owner_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        drop(prepared);
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preseal_revalidation_rejects_same_hash_identity_replacement_without_effect() {
        let (root, project_root, gate) = fixture("prepared-preimage-aba");
        let authority = RemoteAuthority::Branch("main".to_string());
        let before = json!({"head": "CF-COMMIT-before"});
        gate.project()
            .write_authority_atomic(
                &authority,
                &codefire_store::canonical_json(&before).unwrap(),
            )
            .unwrap();
        let writes = [AuthorityWrite {
            authority: authority.clone(),
            before: Some(before.clone()),
            after: json!({"head": "CF-COMMIT-after"}),
        }];
        let request = request(None, "preimage-aba");
        let mut ledger = http_ledger("publication-preimage-aba");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &json!({"status": "never-published"}),
            &mut ledger,
        )
        .unwrap();

        prepared.revalidate_preseal(&mut ledger).unwrap();

        gate.project()
            .write_authority_atomic(
                &authority,
                &codefire_store::canonical_json(&before).unwrap(),
            )
            .unwrap();
        assert!(prepared.revalidate_preseal(&mut ledger).is_err());
        assert!(!prepared.revalidated);
        cancel_prepared_json_publication(prepared, &mut ledger).unwrap();
        assert_eq!(
            read_optional_authority_bounded(gate.project(), &authority, POSTIMAGE_BYTE_LIMIT)
                .unwrap(),
            Some(before)
        );
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pending_create_collision_cleans_owned_stage_and_exact_retry_succeeds() {
        let (root, project_root, gate) = fixture("prepared-pending-collision");
        let request = request(None, "pending-collision");
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});
        let mut ledger = http_ledger("publication-pending-collision");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut ledger,
        )
        .unwrap();
        revalidate_prepared_json_publication(&mut prepared, &mut ledger).unwrap();
        let pending_path = RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap();
        fs::write(&pending_path, b"foreign\n").unwrap();
        ledger.seal_prepublication().unwrap();
        assert!(commit_prepared_json_publication(prepared, &mut ledger).is_err());
        assert_eq!(fs::read(&pending_path).unwrap(), b"foreign\n");
        assert_eq!(
            fs::read_dir(project_root.join("transactions"))
                .unwrap()
                .count(),
            0
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

        fs::remove_file(&pending_path).unwrap();
        let mut retry_ledger = http_ledger("publication-pending-collision-retry");
        let mut retry = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut retry_ledger,
        )
        .unwrap();
        revalidate_prepared_json_publication(&mut retry, &mut retry_ledger).unwrap();
        retry_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(retry, &mut retry_ledger).unwrap(),
            result
        );
        assert_eq!(
            fs::read_dir(project_root.join("transactions"))
                .unwrap()
                .count(),
            0
        );
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn nonce_is_last_authority_in_main_publication_and_cancel_does_not_consume_it() {
        let (root, project_root, gate) = fixture("merged-nonce");
        let policy = json!({
            "request_signatures": {
                "required": true,
                "nonce_ttl_seconds": 300,
                "max_nonce_cache_entries": 32,
                "keys": {
                    "fixture-request": {
                        "secret": "fixture-request-key",
                        "signers": ["fixture-actor"]
                    }
                }
            }
        });
        fs::write(
            project_root.join("server_policy.json"),
            codefire_store::canonical_json(&policy).unwrap(),
        )
        .unwrap();
        let request = request(None, "merged-nonce");
        let target = crate::signatures::remote_request_target(
            "org",
            "app",
            &request.operation,
            Some("main"),
            None,
        );
        let payload = json!({"branch": "main", "head": "CF-COMMIT-fixture"});
        let signature = crate::signatures::sign_remote_request_with_key(
            &request.operation,
            &request.actor,
            &target,
            &payload,
            "fixture-request",
            "fixture-request-key",
        )
        .unwrap();
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});

        let mut cancelled_ledger = http_ledger("merged-nonce-cancel");
        let nonce = validate_publication_signature_with_recovery_and_poll(
            &gate,
            &project_root,
            &request,
            &request.operation,
            &request.actor,
            &target,
            &payload,
            Some(&signature),
            &mut cancelled_ledger,
            || Ok(()),
        )
        .unwrap();
        let prepared = prepare_json_publication_with_nonce_and_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            nonce,
            &result,
            &mut cancelled_ledger,
        )
        .unwrap();
        let records = match prepared
            .plan
            .as_ref()
            .expect("prepared publication must retain its plan")
        {
            PreparedPublicationPlan::New(plan) => plan.recovery.writes.as_slice(),
            _ => panic!("expected a new merged publication"),
        };
        assert_eq!(
            records.last().map(|record| &record.authority),
            Some(&RemoteAuthority::RequestNonceCache)
        );
        cancel_prepared_json_publication(prepared, &mut cancelled_ledger).unwrap();
        assert!(!project_root.join("request_nonce_cache.json").exists());
        assert!(!RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .exists());

        let mut committed_ledger = http_ledger("merged-nonce-commit");
        let nonce = validate_publication_signature_with_recovery_and_poll(
            &gate,
            &project_root,
            &request,
            &request.operation,
            &request.actor,
            &target,
            &payload,
            Some(&signature),
            &mut committed_ledger,
            || Ok(()),
        )
        .unwrap();
        let mut prepared = prepare_json_publication_with_nonce_and_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            nonce,
            &result,
            &mut committed_ledger,
        )
        .unwrap();
        assert_eq!(
            committed_ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(prepared.retained_bytes)
        );
        revalidate_prepared_json_publication(&mut prepared, &mut committed_ledger).unwrap();
        committed_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut committed_ledger).unwrap(),
            result
        );
        assert!(project_root.join("request_nonce_cache.json").is_file());
        assert!(RemoteAuthority::Branch("main".to_string())
            .path(&project_root)
            .unwrap()
            .is_file());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn idempotency_replay_publishes_new_nonce_without_changing_business_receipt() {
        let (root, project_root, gate) = fixture("replay-new-nonce");
        fs::write(
            project_root.join("server_policy.json"),
            codefire_store::canonical_json(&json!({
                "request_signatures": {
                    "required": true,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 32,
                    "keys": {"fixture-request": {
                        "secret": "fixture-request-key",
                        "signers": ["fixture-actor"]
                    }}
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let request = request(Some("stable-business-key"), "same-business-request");
        let target = crate::signatures::remote_request_target(
            "org",
            "app",
            &request.operation,
            Some("main"),
            None,
        );
        let payload = json!({"branch": "main", "head": "CF-COMMIT-fixture"});
        let result = json!({"status": "original-result"});
        let receipt_path = receipt_authority(&request)
            .unwrap()
            .unwrap()
            .path(&project_root)
            .unwrap();
        let mut original_receipt = None;
        let original_business_postimage = json!({
            "version": 1,
            "name": "main",
            "head": "CF-COMMIT-fixture",
            "reviewed_at": "2026-07-20T00:00:00Z"
        });

        for attempt in 0..2 {
            let signature = crate::signatures::sign_remote_request_with_key(
                &request.operation,
                &request.actor,
                &target,
                &payload,
                "fixture-request",
                "fixture-request-key",
            )
            .unwrap();
            let mut ledger = http_ledger(&format!("replay-new-nonce-{attempt}"));
            let nonce = validate_publication_signature_with_recovery_and_poll(
                &gate,
                &project_root,
                &request,
                &request.operation,
                &request.actor,
                &target,
                &payload,
                Some(&signature),
                &mut ledger,
                || Ok(()),
            )
            .unwrap();
            let writes = [AuthorityWrite {
                authority: RemoteAuthority::Branch("main".to_string()),
                before: (attempt == 1).then(|| original_business_postimage.clone()),
                after: if attempt == 0 {
                    original_business_postimage.clone()
                } else {
                    json!({
                        "version": 1,
                        "name": "main",
                        "head": "CF-COMMIT-fixture",
                        "reviewed_at": "2026-07-20T00:00:01Z"
                    })
                },
            }];
            let validated = empty_validated_publication_roots(&project_root, &request);
            let mut business_writes_materialized = false;
            let mut prepared = prepare_json_publication_lazy_with_nonce_validated_roots_and_ledger(
                &gate,
                &project_root,
                &request,
                nonce,
                &result,
                &validated,
                &mut ledger,
                |_| {
                    business_writes_materialized = true;
                    Ok(writes.to_vec())
                },
            )
            .unwrap();
            assert_eq!(business_writes_materialized, attempt == 0);
            assert_eq!(prepared.is_replay(), attempt == 1);
            revalidate_prepared_json_publication(&mut prepared, &mut ledger).unwrap();
            ledger.seal_prepublication().unwrap();
            assert_eq!(
                commit_prepared_json_publication(prepared, &mut ledger).unwrap(),
                result
            );
            let receipt_bytes = fs::read(&receipt_path).unwrap();
            if let Some(original) = original_receipt.as_ref() {
                assert_eq!(&receipt_bytes, original);
            } else {
                original_receipt = Some(receipt_bytes);
            }
        }
        let receipt = fs::read(&receipt_path).unwrap();
        let receipt: Value = serde_json::from_slice(&receipt).unwrap();
        assert_eq!(receipt["effects"].as_array().unwrap().len(), 1);
        assert_eq!(
            receipt["effects"][0]["authority"]["kind"],
            Value::String("branch".to_string())
        );
        let cache: Value = serde_json::from_slice(
            &fs::read(project_root.join("request_nonce_cache.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cache["entries"].as_object().unwrap().len(), 2);
        assert_eq!(
            read_optional_authority_bounded(
                gate.project(),
                &RemoteAuthority::Branch("main".to_string()),
                POSTIMAGE_BYTE_LIMIT,
            )
            .unwrap()
            .unwrap(),
            original_business_postimage,
            "the authenticated replay must publish only the new nonce"
        );
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn matching_pending_allows_exact_nonce_write_progress_gap_recovery_only() {
        let (root, project_root, gate) = fixture("nonce-progress-gap");
        fs::write(
            project_root.join("server_policy.json"),
            codefire_store::canonical_json(&json!({
                "request_signatures": {
                    "required": true,
                    "nonce_ttl_seconds": 300,
                    "max_nonce_cache_entries": 32,
                    "keys": {"fixture-request": {
                        "secret": "fixture-request-key",
                        "signers": ["fixture-actor"]
                    }}
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let request = request(Some("nonce-gap-key"), "nonce-gap");
        let target = crate::signatures::remote_request_target(
            "org",
            "app",
            &request.operation,
            Some("main"),
            None,
        );
        let payload = json!({"branch": "main", "head": "CF-COMMIT-fixture"});
        let signature = crate::signatures::sign_remote_request_with_key(
            &request.operation,
            &request.actor,
            &target,
            &payload,
            "fixture-request",
            "fixture-request-key",
        )
        .unwrap();
        let mut initial_nonce_ledger = http_ledger("nonce-gap-initial");
        let nonce = validate_publication_signature_with_recovery_and_poll(
            &gate,
            &project_root,
            &request,
            &request.operation,
            &request.actor,
            &target,
            &payload,
            Some(&signature),
            &mut initial_nonce_ledger,
            || Ok(()),
        )
        .unwrap();
        let nonce_write = nonce.authority_write(None).unwrap().unwrap();
        nonce.release_retained(&mut initial_nonce_ledger).unwrap();
        let writes = [branch_write("main"), nonce_write];
        let result = json!({"status": "recovered-after-nonce-gap"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        let mut pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        pending["published_count"] = json!(1);
        write_pending(gate.project(), &pending).unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[1].authority,
                &codefire_store::canonical_json(&writes[1].after).unwrap(),
            )
            .unwrap();

        let mut ledger = http_ledger("nonce-gap-recovery");
        let recovered_nonce = validate_publication_signature_with_recovery_and_poll(
            &gate,
            &project_root,
            &request,
            &request.operation,
            &request.actor,
            &target,
            &payload,
            Some(&signature),
            &mut ledger,
            || Ok(()),
        )
        .unwrap();
        assert!(!recovered_nonce.has_authority_write());

        let mut completed_pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        completed_pending["published_count"] = json!(2);
        completed_pending["status"] = json!("completed");
        completed_pending["phase"] = json!("completed");
        completed_pending["completed_at"] = json!(now_iso_utc());
        let completed_records = completed_pending["writes"].as_array().unwrap().clone();
        publish_completed_receipt(
            gate.project(),
            &receipt_authority(&request).unwrap().unwrap(),
            &request,
            &request_hash,
            &completed_records,
            &result,
        )
        .unwrap();
        write_pending(gate.project(), &completed_pending).unwrap();
        let transaction_id = required_pending_string(&completed_pending, "transaction_id").unwrap();
        fs::rename(
            RemoteAuthority::PendingTransaction
                .path(&project_root)
                .unwrap(),
            project_root.join(format!(
                ".pending_transaction.removing-{transaction_id}.json"
            )),
        )
        .unwrap();
        let mut completed_gap_ledger = http_ledger("completed-nonce-gap-recovery");
        let recovered_nonce = validate_publication_signature_with_recovery_and_poll(
            &gate,
            &project_root,
            &request,
            &request.operation,
            &request.actor,
            &target,
            &payload,
            Some(&signature),
            &mut completed_gap_ledger,
            || Ok(()),
        )
        .unwrap();
        assert!(!recovered_nonce.has_authority_write());
        let mut prepared = prepare_json_publication_with_nonce_and_ledger(
            &gate,
            &project_root,
            &request,
            std::slice::from_ref(&writes[0]),
            recovered_nonce,
            &result,
            &mut completed_gap_ledger,
        )
        .unwrap();
        assert!(prepared.is_replay());
        revalidate_prepared_json_publication(&mut prepared, &mut completed_gap_ledger).unwrap();
        completed_gap_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut completed_gap_ledger).unwrap(),
            result
        );
        let mut completed_retry_ledger = http_ledger("completed-nonce-retry");
        assert!(matches!(
            validate_publication_signature_with_recovery_and_poll(
                &gate,
                &project_root,
                &request,
                &request.operation,
                &request.actor,
                &target,
                &payload,
                Some(&signature),
                &mut completed_retry_ledger,
                || Ok(()),
            ),
            Err(CliError::AuthenticationOrSignature(_))
        ));
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_recovery_replays_write_before_progress_gap() {
        let (root, project_root, gate) = fixture("prepared-recovery");
        let request = request(Some("prepared-recovery"), "recovery");
        let writes = [branch_write("main")];
        let result = json!({"status": "recovered"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        let mut ledger = http_ledger("prepared-recovery");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut ledger,
        )
        .unwrap();
        assert!(prepared.is_replay());
        let retained_bytes = prepared.retained_bytes;
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(retained_bytes)
        );
        assert!(
            ledger
                .snapshot()
                .peak
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default()
                > 0
        );
        revalidate_prepared_json_publication(&mut prepared, &mut ledger).unwrap();
        ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut ledger).unwrap(),
            result
        );
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancelling_prepared_recovery_releases_owners_without_advancing_progress() {
        let (root, project_root, gate) = fixture("prepared-recovery-cancel");
        let request = request(None, "recovery-cancel");
        let writes = [branch_write("first"), branch_write("second")];
        let result = json!({"status": "not-committed"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        let pending_before = fs::read(
            RemoteAuthority::PendingTransaction
                .path(&project_root)
                .unwrap(),
        )
        .unwrap();
        let mut ledger = http_ledger("prepared-recovery-cancel");
        let prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut ledger,
        )
        .unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(
            ledger
                .snapshot()
                .peak
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default()
                > 0
        );
        cancel_prepared_json_publication(prepared, &mut ledger).unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(
            fs::read(
                RemoteAuthority::PendingTransaction
                    .path(&project_root)
                    .unwrap()
            )
            .unwrap(),
            pending_before
        );
        assert!(RemoteAuthority::Branch("first".to_string())
            .path(&project_root)
            .unwrap()
            .is_file());
        assert!(!RemoteAuthority::Branch("second".to_string())
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_receipt_replays_and_conflicting_key_fails_before_write() {
        let (root, project_root, gate) = fixture("replay");
        let original = request(Some("same-key"), "first");
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});
        assert_eq!(
            execute_json_publication(&gate, &project_root, &original, &writes, &result).unwrap(),
            result
        );
        assert_eq!(
            execute_json_publication(&gate, &project_root, &original, &writes, &result).unwrap(),
            result
        );
        let conflict = request(Some("same-key"), "different");
        assert!(matches!(
            execute_json_publication(
                &gate,
                &project_root,
                &conflict,
                &[branch_write("other")],
                &json!({"status": "other"}),
            ),
            Err(CliError::IdempotencyConflict(_))
        ));
        assert!(read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::Branch("other".to_string()),
            1024,
        )
        .unwrap()
        .is_none());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_receipt_replay_precedes_regenerated_postimages_and_missing_authority() {
        let (root, project_root, gate) = fixture("replay-regenerated-postimage");
        let request = request(Some("stable-review-key"), "stable-review-request");
        let result = json!({"status": "published"});
        let original_write = AuthorityWrite {
            authority: RemoteAuthority::Branch("main".to_string()),
            before: None,
            after: json!({
                "version": 1,
                "name": "main",
                "head": "CF-COMMIT-fixture",
                "reviewed_at": "2026-07-20T00:00:00Z"
            }),
        };
        let mut initial_ledger = http_ledger("replay regenerated initial");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            std::slice::from_ref(&original_write),
            &result,
            &mut initial_ledger,
        )
        .unwrap();
        assert!(!prepared.is_replay());
        revalidate_prepared_json_publication(&mut prepared, &mut initial_ledger).unwrap();
        initial_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut initial_ledger).unwrap(),
            result
        );

        let regenerated_write = AuthorityWrite {
            authority: RemoteAuthority::Branch("main".to_string()),
            before: Some(original_write.after.clone()),
            after: json!({
                "version": 1,
                "name": "main",
                "head": "CF-COMMIT-fixture",
                "reviewed_at": "2026-07-20T00:00:01Z"
            }),
        };
        let mut timestamp_ledger = http_ledger("replay regenerated timestamp");
        let mut timestamp_replay = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            std::slice::from_ref(&regenerated_write),
            &result,
            &mut timestamp_ledger,
        )
        .unwrap();
        assert!(timestamp_replay.is_replay());
        revalidate_prepared_json_publication(&mut timestamp_replay, &mut timestamp_ledger).unwrap();
        timestamp_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(timestamp_replay, &mut timestamp_ledger).unwrap(),
            result
        );
        assert_eq!(
            read_optional_authority_bounded(
                gate.project(),
                &original_write.authority,
                POSTIMAGE_BYTE_LIMIT,
            )
            .unwrap()
            .unwrap(),
            original_write.after.clone()
        );

        fs::remove_file(original_write.authority.path(&project_root).unwrap()).unwrap();
        let mut missing_ledger = http_ledger("replay missing current authority");
        let mut missing_replay = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            std::slice::from_ref(&regenerated_write),
            &result,
            &mut missing_ledger,
        )
        .unwrap();
        assert!(missing_replay.is_replay());
        revalidate_prepared_json_publication(&mut missing_replay, &mut missing_ledger).unwrap();
        missing_ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(missing_replay, &mut missing_ledger).unwrap(),
            result
        );
        assert!(!original_write
            .authority
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn write_before_progress_crash_gap_converges_on_exact_retry() {
        let (root, project_root, gate) = fixture("write-gap");
        let request = request(None, "gap");
        let writes = [branch_write("main")];
        let result = json!({"status": "recovered"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        assert_eq!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result).unwrap(),
            result
        );
        assert!(read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .is_none());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_pending_is_released_before_receipt_replay() {
        let (root, project_root, gate) = fixture("completed-gap");
        let request = request(Some("completed-gap"), "gap");
        let writes = [branch_write("main")];
        let result = json!({"status": "completed"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        let mut pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        pending["published_count"] = json!(1);
        pending["status"] = json!("completed");
        pending["phase"] = json!("completed");
        pending["completed_at"] = json!(now_iso_utc());
        let records = pending["writes"].as_array().unwrap().clone();
        let receipt = receipt_authority(&request).unwrap().unwrap();
        publish_completed_receipt(
            gate.project(),
            &receipt,
            &request,
            &request_hash,
            &records,
            &result,
        )
        .unwrap();
        write_pending(gate.project(), &pending).unwrap();
        let transaction_id = required_pending_string(&pending, "transaction_id").unwrap();
        let transaction = gate.project().open_transaction(&transaction_id).unwrap();
        cleanup_transaction_stage(
            gate.project(),
            &transaction,
            &BTreeMap::from([(
                "write-0.json".to_string(),
                required_pending_string(&records[0], "after_hash").unwrap(),
            )]),
        )
        .unwrap();

        assert_eq!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result).unwrap(),
            result
        );
        assert!(read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .is_none());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn foreign_pending_transaction_is_preserved() {
        let (root, project_root, gate) = fixture("foreign");
        let owner = request(None, "owner");
        let writes = [branch_write("main")];
        let owner_hash = normalized_request_hash(&owner).unwrap();
        prepare_publication(
            gate.project(),
            &owner,
            &writes,
            &json!({"status": "owner"}),
            &owner_hash,
        )
        .unwrap();
        let foreign = request(None, "foreign");
        assert!(matches!(
            execute_json_publication(
                &gate,
                &project_root,
                &foreign,
                &writes,
                &json!({"status": "foreign"}),
            ),
            Err(CliError::IdempotencyConflict(_))
        ));
        assert!(RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .is_file());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_preflights_all_staging_and_schema_before_first_write() {
        for mode in [
            "corrupt",
            "foreign",
            "malformed-preimage",
            "missing-result",
            "future-phase",
            "extra-authority-field",
        ] {
            let (root, project_root, gate) = fixture(mode);
            let owner = request(None, mode);
            let writes = [branch_write("first"), branch_write("second")];
            let owner_hash = normalized_request_hash(&owner).unwrap();
            let result = json!({"status": mode});
            prepare_publication(gate.project(), &owner, &writes, &result, &owner_hash).unwrap();
            let mut pending = read_optional_authority_bounded(
                gate.project(),
                &RemoteAuthority::PendingTransaction,
                PENDING_BYTE_LIMIT,
            )
            .unwrap()
            .unwrap();
            let transaction_id = required_pending_string(&pending, "transaction_id").unwrap();
            let transaction_dir = project_root.join("transactions").join(transaction_id);
            match mode {
                "corrupt" => fs::write(transaction_dir.join("write-1.json"), b"{}\n").unwrap(),
                "foreign" => fs::write(transaction_dir.join("foreign.json"), b"{}\n").unwrap(),
                "malformed-preimage" => {
                    pending["writes"][1]["before_absent"] = json!(false);
                    pending["writes"][1]["before_hash"] = Value::Null;
                    write_pending(gate.project(), &pending).unwrap();
                }
                "missing-result" => {
                    pending.as_object_mut().unwrap().remove("result");
                    write_pending(gate.project(), &pending).unwrap();
                }
                "future-phase" => {
                    pending["phase"] = json!("future");
                    write_pending(gate.project(), &pending).unwrap();
                }
                "extra-authority-field" => {
                    pending["writes"][1]["authority"]["extra"] = json!(true);
                    write_pending(gate.project(), &pending).unwrap();
                }
                _ => unreachable!(),
            }
            assert!(
                execute_json_publication(&gate, &project_root, &owner, &writes, &result,).is_err()
            );
            assert!(read_optional_authority_bounded(
                gate.project(),
                &RemoteAuthority::Branch("first".to_string()),
                1024,
            )
            .unwrap()
            .is_none());
            drop(gate);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn malformed_existing_receipt_is_never_accepted_for_replay() {
        let (root, project_root, gate) = fixture("receipt-shape");
        let request = request(Some("receipt-shape"), "receipt");
        let writes = [branch_write("main")];
        let result = json!({"status": "published"});
        execute_json_publication(&gate, &project_root, &request, &writes, &result).unwrap();
        let authority = receipt_authority(&request).unwrap().unwrap();
        let path = authority.path(&project_root).unwrap();
        let mut receipt: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        receipt["operation"] = json!("tampered-operation");
        fs::write(&path, codefire_store::canonical_json(&receipt).unwrap()).unwrap();
        assert!(matches!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result),
            Err(CliError::InvalidRepository(_))
        ));
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_cleanup_resumes_from_quarantined_stage_file() {
        let (root, project_root, gate) = fixture("cleanup-gap");
        let request = request(Some("cleanup-gap"), "cleanup");
        let writes = [branch_write("main")];
        let result = json!({"status": "completed"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        let mut pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        pending["published_count"] = json!(1);
        pending["status"] = json!("completed");
        pending["phase"] = json!("completed");
        pending["completed_at"] = json!(now_iso_utc());
        let records = pending["writes"].as_array().unwrap().clone();
        publish_completed_receipt(
            gate.project(),
            &receipt_authority(&request).unwrap().unwrap(),
            &request,
            &request_hash,
            &records,
            &result,
        )
        .unwrap();
        write_pending(gate.project(), &pending).unwrap();
        let transaction_id = required_pending_string(&pending, "transaction_id").unwrap();
        let transaction_dir = project_root.join("transactions").join(&transaction_id);
        let quarantine = transaction_dir.join(format!(".write-0.json.removing-{transaction_id}"));
        fs::rename(transaction_dir.join("write-0.json"), &quarantine).unwrap();
        fs::write(&quarantine, b"{}\n").unwrap();
        assert!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result).is_err()
        );
        assert!(quarantine.is_file());
        assert!(RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .is_file());
        fs::write(
            &quarantine,
            codefire_store::canonical_json(&writes[0].after).unwrap(),
        )
        .unwrap();
        assert_eq!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result).unwrap(),
            result
        );
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_cleanup_removes_quarantined_transaction_directory() {
        let (root, project_root, gate) = fixture("transaction-dir-gap");
        let request = request(Some("transaction-dir-gap"), "cleanup");
        let writes = [branch_write("main")];
        let result = json!({"status": "completed"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        let mut pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        pending["published_count"] = json!(1);
        pending["status"] = json!("completed");
        pending["phase"] = json!("completed");
        pending["completed_at"] = json!(now_iso_utc());
        let records = pending["writes"].as_array().unwrap().clone();
        publish_completed_receipt(
            gate.project(),
            &receipt_authority(&request).unwrap().unwrap(),
            &request,
            &request_hash,
            &records,
            &result,
        )
        .unwrap();
        write_pending(gate.project(), &pending).unwrap();
        let transaction_id = required_pending_string(&pending, "transaction_id").unwrap();
        let transaction_dir = project_root.join("transactions").join(&transaction_id);
        fs::remove_file(transaction_dir.join("write-0.json")).unwrap();
        let quarantine = project_root
            .join("transactions")
            .join(format!(".removing-{transaction_id}"));
        fs::rename(&transaction_dir, &quarantine).unwrap();

        assert_eq!(
            execute_json_publication(&gate, &project_root, &request, &writes, &result).unwrap(),
            result
        );
        assert!(!quarantine.exists());
        assert!(!RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap()
            .exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completed_cleanup_removes_quarantined_pending_marker() {
        let (root, project_root, gate) = fixture("pending-marker-gap");
        let request = request(Some("pending-marker-gap"), "cleanup");
        let writes = [branch_write("main")];
        let result = json!({"status": "completed"});
        let request_hash = normalized_request_hash(&request).unwrap();
        prepare_publication(gate.project(), &request, &writes, &result, &request_hash).unwrap();
        let mut pending = read_optional_authority_bounded(
            gate.project(),
            &RemoteAuthority::PendingTransaction,
            PENDING_BYTE_LIMIT,
        )
        .unwrap()
        .unwrap();
        gate.project()
            .write_authority_atomic(
                &writes[0].authority,
                &codefire_store::canonical_json(&writes[0].after).unwrap(),
            )
            .unwrap();
        pending["published_count"] = json!(1);
        pending["status"] = json!("completed");
        pending["phase"] = json!("completed");
        pending["completed_at"] = json!(now_iso_utc());
        let records = pending["writes"].as_array().unwrap().clone();
        publish_completed_receipt(
            gate.project(),
            &receipt_authority(&request).unwrap().unwrap(),
            &request,
            &request_hash,
            &records,
            &result,
        )
        .unwrap();
        write_pending(gate.project(), &pending).unwrap();
        let transaction_id = required_pending_string(&pending, "transaction_id").unwrap();
        let transaction = gate.project().open_transaction(&transaction_id).unwrap();
        cleanup_transaction_stage(
            gate.project(),
            &transaction,
            &BTreeMap::from([(
                "write-0.json".to_string(),
                required_pending_string(&records[0], "after_hash").unwrap(),
            )]),
        )
        .unwrap();
        let pending_path = RemoteAuthority::PendingTransaction
            .path(&project_root)
            .unwrap();
        let quarantine = project_root.join(format!(
            ".pending_transaction.removing-{transaction_id}.json"
        ));
        fs::rename(&pending_path, &quarantine).unwrap();

        let mut ledger = http_ledger("prepared-quarantine-recovery");
        let mut prepared = prepare_json_publication_with_ledger(
            &gate,
            &project_root,
            &request,
            &writes,
            &result,
            &mut ledger,
        )
        .unwrap();
        revalidate_prepared_json_publication(&mut prepared, &mut ledger).unwrap();
        ledger.seal_prepublication().unwrap();
        assert_eq!(
            commit_prepared_json_publication(prepared, &mut ledger).unwrap(),
            result
        );
        assert!(!quarantine.exists());
        assert!(!pending_path.exists());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }
}
