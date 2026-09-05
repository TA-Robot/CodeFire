use super::sha256_hex;
use crate::stable_lock::{LockDeadline, StableFileLock, StableLockMode};
use crate::{now_iso_utc, secure_worktree, unique_temp_path, CliError, LockOptions};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use serde_json::{json, Value};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Maximum UTF-8 byte length accepted by every Rust/Python remote component.
pub(crate) const REMOTE_COMPONENT_BYTE_LIMIT: usize = 255;
pub(crate) const REMOTE_INTERNAL_HASH_PREFIX: &str = "CF-INTERNAL-HASH-";
const PROJECT_DESCRIPTOR_FILE: &str = "remote_project.json";
const PROJECT_PROTOCOL_GENERATION: u64 = 1;
const BOOTSTRAP_RECOVERY_ENTRY_LIMIT: u64 = 4_096;
const BOOTSTRAP_RECOVERY_BYTE_LIMIT: u64 = 16 * 1024 * 1024;
const BOOTSTRAP_RECOVERY_ITERATOR_SCRATCH_BYTES: u64 = 4 * 1024;
const BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES: u64 =
    codefire_core::LOGICAL_FRAME_BASE_BYTES + (REMOTE_COMPONENT_BYTE_LIMIT as u64 * 4);
// Descriptor + eight top-level directories + twelve object-kind directories
// + the project gate and its stable owner record.
const BOOTSTRAP_FIXED_LAYOUT_ENTRIES: u64 = 23;
const BOOTSTRAP_FIXED_LAYOUT_PAYLOAD_BYTES: u64 = 16 * 1024;
const BOOTSTRAP_VISIBILITY_LOGICAL_BYTES: u64 =
    codefire_core::LOGICAL_FRAME_BASE_BYTES + (REMOTE_COMPONENT_BYTE_LIMIT as u64 * 2);
const BOOTSTRAP_CLEANUP_TEMPORARY_BYTES: u64 = 2 * 16 * 1024 + BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES;
const OBJECT_SUBDIRS: &[&str] = &[
    "blobs",
    "content_manifests",
    "atom_indexes",
    "trace_graphs",
    "fire_ledgers",
    "resolution_ledgers",
    "verifications",
    "policies",
    "artifact_refs",
    "evidence",
    "commits",
    "branches",
];

#[cfg(test)]
thread_local! {
    static INJECT_BOOTSTRAP_BUILD_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_BOOTSTRAP_BUILD_EXTRA_ENTRY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_BOOTSTRAP_BUILD_CLEANUP_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_LEDGER_BOOTSTRAP_RENAME_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static LEDGER_BOOTSTRAP_RENAME_ATTEMPTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_ledger_bootstrap_rename_attempts() {
    LEDGER_BOOTSTRAP_RENAME_ATTEMPTS.set(0);
}

#[cfg(test)]
pub(crate) fn ledger_bootstrap_rename_attempts() -> u64 {
    LEDGER_BOOTSTRAP_RENAME_ATTEMPTS.get()
}

#[cfg(test)]
pub(crate) fn inject_ledger_bootstrap_rename_failure_once() {
    INJECT_LEDGER_BOOTSTRAP_RENAME_FAILURE.set(true);
}

#[cfg(test)]
fn take_test_injection(flag: &'static std::thread::LocalKey<std::cell::Cell<bool>>) -> bool {
    flag.with(|value| value.replace(false))
}

#[derive(Debug)]
pub(crate) enum BootstrapRecoveryError {
    EntryLimitExceeded { limit: u64 },
    ByteLimitExceeded { limit: u64 },
    InvalidOwner { path: PathBuf, reason: String },
    WorkAccounting(codefire_core::DecisionWorkAccountingError),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl BootstrapRecoveryError {
    fn into_cli(self) -> CliError {
        match self {
            Self::Io(error) => CliError::Io(error),
            Self::Json(error) => CliError::Json(error),
            Self::EntryLimitExceeded { limit } => CliError::InvalidRepository(format!(
                "remote bootstrap recovery entry limit exceeded ({limit})"
            )),
            Self::ByteLimitExceeded { limit } => CliError::InvalidRepository(format!(
                "remote bootstrap recovery byte limit exceeded ({limit})"
            )),
            Self::InvalidOwner { path, reason } => CliError::InvalidRepository(format!(
                "remote bootstrap recovery rejected {}: {reason}",
                path.display()
            )),
            Self::WorkAccounting(error) => {
                bootstrap_work_error(error, "remote bootstrap recovery work accounting failed")
            }
        }
    }
}

impl From<std::io::Error> for BootstrapRecoveryError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for BootstrapRecoveryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub(crate) struct BootstrapRecoveryWork<'a> {
    ledger: Option<&'a mut codefire_core::DecisionWorkLedger>,
    permit: Option<codefire_core::DecisionWorkPermit>,
    entries: u64,
    bytes: u64,
    descriptor_reads: u64,
    file_bytes_read: u64,
    temporary_current: u64,
    temporary_peak: u64,
    ledger_temporary_owned: u64,
    entry_limit: u64,
    byte_limit: u64,
}

impl BootstrapRecoveryWork<'_> {
    fn bounded() -> Self {
        Self {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit: BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
            byte_limit: BOOTSTRAP_RECOVERY_BYTE_LIMIT,
        }
    }

    pub(crate) fn with_ledger(
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<BootstrapRecoveryWork<'_>, BootstrapRecoveryError> {
        let upper = bootstrap_recovery_reservation(
            BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
            BOOTSTRAP_RECOVERY_BYTE_LIMIT,
            BOOTSTRAP_CLEANUP_TEMPORARY_BYTES,
        );
        let permit = ledger
            .reserve(upper)
            .map_err(BootstrapRecoveryError::WorkAccounting)?;
        Ok(BootstrapRecoveryWork {
            ledger: Some(ledger),
            permit: Some(permit),
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit: BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
            byte_limit: BOOTSTRAP_RECOVERY_BYTE_LIMIT,
        })
    }

    fn charge_entry(&mut self, logical_bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.entries =
            self.entries
                .checked_add(1)
                .ok_or(BootstrapRecoveryError::EntryLimitExceeded {
                    limit: self.entry_limit,
                })?;
        if self.entries > self.entry_limit {
            return Err(BootstrapRecoveryError::EntryLimitExceeded {
                limit: self.entry_limit,
            });
        }
        self.charge_bytes(logical_bytes)?;
        Ok(())
    }

    fn charge_bytes(&mut self, bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.bytes =
            self.bytes
                .checked_add(bytes)
                .ok_or(BootstrapRecoveryError::ByteLimitExceeded {
                    limit: self.byte_limit,
                })?;
        if self.bytes > self.byte_limit {
            return Err(BootstrapRecoveryError::ByteLimitExceeded {
                limit: self.byte_limit,
            });
        }
        Ok(())
    }

    fn charge_payload_bytes(&mut self, bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.charge_bytes(bytes)?;
        self.file_bytes_read = self.file_bytes_read.saturating_add(bytes);
        Ok(())
    }

    fn charge_tree_entry_frame(&mut self, name: &OsStr) -> Result<(), BootstrapRecoveryError> {
        let logical_bytes =
            bootstrap_entry_frame_bytes(name).ok_or(BootstrapRecoveryError::ByteLimitExceeded {
                limit: self.byte_limit,
            })?;
        self.charge_entry(logical_bytes)
    }

    fn charge_tree_entry_payload(
        &mut self,
        is_directory: bool,
        physical_len: u64,
    ) -> Result<(), BootstrapRecoveryError> {
        self.charge_payload_bytes(bootstrap_tree_entry_payload_bytes(
            is_directory,
            physical_len,
        ))
    }

    fn charge_descriptor_read(&mut self, bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.charge_bytes(bytes)?;
        self.descriptor_reads = self.descriptor_reads.saturating_add(1);
        self.file_bytes_read = self.file_bytes_read.saturating_add(bytes);
        Ok(())
    }

    fn acquire_temporary(&mut self, bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.temporary_current = self.temporary_current.checked_add(bytes).ok_or(
            BootstrapRecoveryError::ByteLimitExceeded {
                limit: self.byte_limit,
            },
        )?;
        self.temporary_peak = self.temporary_peak.max(self.temporary_current);
        Ok(())
    }

    fn release_temporary(&mut self, bytes: u64) -> Result<(), BootstrapRecoveryError> {
        self.temporary_current = self.temporary_current.checked_sub(bytes).ok_or({
            BootstrapRecoveryError::WorkAccounting(
                codefire_core::DecisionWorkAccountingError::ReleaseUnderflow {
                    dimension: codefire_core::DecisionWorkDimension::TemporaryBytes,
                    current: self.temporary_current,
                    amount: bytes,
                },
            )
        })?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), BootstrapRecoveryError> {
        if self.temporary_current != 0 {
            return Err(BootstrapRecoveryError::WorkAccounting(
                codefire_core::DecisionWorkAccountingError::ReleaseUnderflow {
                    dimension: codefire_core::DecisionWorkDimension::TemporaryBytes,
                    current: 0,
                    amount: self.temporary_current,
                },
            ));
        }
        let (Some(ledger), Some(mut permit)) = (self.ledger.as_deref_mut(), self.permit.take())
        else {
            return Ok(());
        };
        let actual = bootstrap_observed_reservation(
            self.entries,
            self.bytes,
            self.descriptor_reads,
            self.file_bytes_read,
            self.temporary_peak,
        );
        if let Err(error) = ledger.consume_reserved(&mut permit, actual) {
            self.permit = Some(permit);
            return Err(BootstrapRecoveryError::WorkAccounting(error));
        }
        self.ledger_temporary_owned = self.temporary_peak;
        if self.temporary_peak > 0 {
            if let Err(error) = ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                self.temporary_peak,
            ) {
                self.permit = Some(permit);
                return Err(BootstrapRecoveryError::WorkAccounting(error));
            }
            self.ledger_temporary_owned = 0;
        }
        match ledger.cancel_reserved(&permit) {
            Ok(()) | Err(codefire_core::DecisionWorkAccountingError::InvalidPermit) => {}
            Err(error) => {
                self.permit = Some(permit);
                return Err(BootstrapRecoveryError::WorkAccounting(error));
            }
        }
        Ok(())
    }

    fn observation(&self) -> BootstrapCleanupObservation {
        BootstrapCleanupObservation {
            entries: self.entries,
            bytes: self.bytes,
            descriptor_reads: self.descriptor_reads,
            file_bytes_read: self.file_bytes_read,
            temporary_peak: self.temporary_peak,
        }
    }
}

impl Drop for BootstrapRecoveryWork<'_> {
    fn drop(&mut self) {
        let Some(ledger) = self.ledger.as_deref_mut() else {
            return;
        };
        if self.ledger_temporary_owned > 0 {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                self.ledger_temporary_owned,
            );
            self.ledger_temporary_owned = 0;
        }
        if let Some(permit) = self.permit.take() {
            let _ = ledger.cancel_reserved(&permit);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct BootstrapCleanupObservation {
    entries: u64,
    bytes: u64,
    descriptor_reads: u64,
    file_bytes_read: u64,
    temporary_peak: u64,
}

impl BootstrapCleanupObservation {
    fn reservation(self) -> codefire_core::DecisionWorkReservation {
        bootstrap_observed_reservation(
            self.entries,
            self.bytes,
            self.descriptor_reads,
            self.file_bytes_read,
            self.temporary_peak,
        )
    }

    fn combine(self, other: Self) -> Self {
        Self {
            entries: self.entries.saturating_add(other.entries),
            bytes: self.bytes.saturating_add(other.bytes),
            descriptor_reads: self.descriptor_reads.saturating_add(other.descriptor_reads),
            file_bytes_read: self.file_bytes_read.saturating_add(other.file_bytes_read),
            temporary_peak: self.temporary_peak.max(other.temporary_peak),
        }
    }

    fn creation_reservation(self) -> codefire_core::DecisionWorkReservation {
        bootstrap_creation_actual_reservation(
            self.entries.max(1),
            self.bytes.max(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES),
            u64::from(self.descriptor_reads > 0),
        )
    }
}

fn bootstrap_recovery_reservation(
    entries: u64,
    bytes: u64,
    temporary: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::GraphDiscovery,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(codefire_core::DecisionWorkDimension::GraphNodes, entries)
    .charge(codefire_core::DecisionWorkDimension::AggregateBytes, bytes)
    .acquire(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary,
    )
    .count(
        codefire_core::DecisionWorkCounter::UniqueNodesDiscovered,
        entries,
    )
    .count(codefire_core::DecisionWorkCounter::FilesRead, entries)
    .count(codefire_core::DecisionWorkCounter::FileBytesRead, bytes)
}

fn bootstrap_observed_reservation(
    entries: u64,
    bytes: u64,
    descriptor_reads: u64,
    file_bytes_read: u64,
    temporary: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::GraphDiscovery,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(codefire_core::DecisionWorkDimension::GraphNodes, entries)
    .charge(codefire_core::DecisionWorkDimension::AggregateBytes, bytes)
    .acquire(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary,
    )
    .count(
        codefire_core::DecisionWorkCounter::UniqueNodesDiscovered,
        entries,
    )
    .count(
        codefire_core::DecisionWorkCounter::FilesRead,
        descriptor_reads,
    )
    .count(
        codefire_core::DecisionWorkCounter::FileBytesRead,
        file_bytes_read,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemoteAuthority {
    Branch(String),
    MergeRequest(String),
    Generation,
    PendingTransaction,
    IdempotencyReceipt { operation: String, digest: String },
    GcAudit(String),
    RequestNonceCache,
}

impl RemoteAuthority {
    pub(crate) fn path(&self, project_root: &Path) -> Result<PathBuf, CliError> {
        match self {
            Self::Branch(branch) => {
                validate_remote_component(branch, "branch")?;
                Ok(super::remote_branch_path(project_root, branch))
            }
            Self::MergeRequest(mr_id) => {
                validate_remote_component(mr_id, "merge-request ID")?;
                Ok(super::remote_merge_request_path(project_root, mr_id))
            }
            Self::Generation => Ok(project_root.join("gc_state.json")),
            Self::PendingTransaction => Ok(project_root.join("pending_transaction.json")),
            Self::IdempotencyReceipt { operation, digest } => {
                validate_remote_component(operation, "operation")?;
                validate_hex_digest(digest, "idempotency receipt")?;
                Ok(project_root
                    .join("idempotency")
                    .join("v2")
                    .join(operation)
                    .join(format!("{digest}.json")))
            }
            Self::GcAudit(plan_id) => {
                validate_remote_component(plan_id, "GC plan ID")?;
                Ok(project_root
                    .join("gc")
                    .join(super::remote_gc_audit_file_name(plan_id)))
            }
            Self::RequestNonceCache => Ok(project_root.join("request_nonce_cache.json")),
        }
    }

    pub(crate) fn record(&self) -> Value {
        match self {
            Self::Branch(branch) => json!({"kind": "branch", "name": branch}),
            Self::MergeRequest(mr_id) => {
                json!({"kind": "merge_request", "id": mr_id})
            }
            Self::Generation => json!({"kind": "generation"}),
            Self::PendingTransaction => json!({"kind": "pending_transaction"}),
            Self::IdempotencyReceipt { operation, digest } => json!({
                "kind": "idempotency_receipt",
                "operation": operation,
                "digest": digest,
            }),
            Self::GcAudit(plan_id) => json!({"kind": "gc_audit", "plan_id": plan_id}),
            Self::RequestNonceCache => json!({"kind": "request_nonce_cache"}),
        }
    }

    pub(crate) fn from_record(value: &Value) -> Result<Self, CliError> {
        let object = value.as_object().ok_or_else(|| {
            CliError::InvalidRepository("remote authority record is not an object".to_string())
        })?;
        let kind = value.get("kind").and_then(Value::as_str).ok_or_else(|| {
            CliError::InvalidRepository("remote authority kind is missing".to_string())
        })?;
        let string = |field: &str| {
            value
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| {
                    CliError::InvalidRepository(format!(
                        "remote authority {kind} field {field} is missing"
                    ))
                })
        };
        let exact = |fields: &[&str]| {
            if object.len() != fields.len()
                || !fields.iter().all(|field| object.contains_key(*field))
            {
                return Err(CliError::InvalidRepository(format!(
                    "remote authority {kind} record has unknown or missing fields"
                )));
            }
            Ok(())
        };
        let authority = match kind {
            "branch" => {
                exact(&["kind", "name"])?;
                Self::Branch(string("name")?)
            }
            "merge_request" => {
                exact(&["kind", "id"])?;
                Self::MergeRequest(string("id")?)
            }
            "generation" => {
                exact(&["kind"])?;
                Self::Generation
            }
            "pending_transaction" => {
                exact(&["kind"])?;
                Self::PendingTransaction
            }
            "idempotency_receipt" => {
                exact(&["kind", "operation", "digest"])?;
                Self::IdempotencyReceipt {
                    operation: string("operation")?,
                    digest: string("digest")?,
                }
            }
            "gc_audit" => {
                exact(&["kind", "plan_id"])?;
                Self::GcAudit(string("plan_id")?)
            }
            "request_nonce_cache" => {
                exact(&["kind"])?;
                Self::RequestNonceCache
            }
            _ => {
                return Err(CliError::InvalidRepository(format!(
                    "unsupported remote authority kind: {kind}"
                )))
            }
        };
        let _ = authority.path(Path::new("/remote-project-validation"))?;
        Ok(authority)
    }
}

fn validate_hex_digest(value: &str, kind: &str) -> Result<(), CliError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::InvalidRepository(format!(
            "invalid remote {kind} digest"
        )));
    }
    Ok(())
}

struct PinnedRemoteDirectory {
    directory: Dir,
    identity: secure_worktree::StableFsIdentity,
    display_path: PathBuf,
}

pub(crate) struct RemoteProject {
    display_root: PathBuf,
    root: PinnedRemoteDirectory,
    objects: PinnedRemoteDirectory,
    branches: PinnedRemoteDirectory,
    merge_requests: PinnedRemoteDirectory,
    locks: PinnedRemoteDirectory,
    idempotency: PinnedRemoteDirectory,
    transactions: PinnedRemoteDirectory,
    gc: PinnedRemoteDirectory,
    descriptor_identity: secure_worktree::StableFsIdentity,
    descriptor_digest: String,
    stable_id: String,
}

struct PreparedRemoteProjectDisplayPaths {
    display_root: PathBuf,
    root: PathBuf,
    objects: PathBuf,
    branches: PathBuf,
    merge_requests: PathBuf,
    locks: PathBuf,
    idempotency: PathBuf,
    transactions: PathBuf,
    gc: PathBuf,
}

impl PreparedRemoteProjectDisplayPaths {
    fn new(display_root: PathBuf) -> Self {
        Self {
            root: display_root.clone(),
            objects: display_root.join("objects"),
            branches: display_root.join("branches"),
            merge_requests: display_root.join("merge_requests"),
            locks: display_root.join("locks"),
            idempotency: display_root.join("idempotency"),
            transactions: display_root.join("transactions"),
            gc: display_root.join("gc"),
            display_root,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteProjectVisibility {
    Existing,
    Unpublished,
    PublishedNow,
}

pub(crate) struct PrelockedRemoteProject {
    project: RemoteProject,
    gate_lock: StableFileLock,
    visibility: RemoteProjectVisibility,
}

pub(crate) struct UnpublishedRemoteProject {
    project: RemoteProject,
    org_root: PinnedRemoteDirectory,
    app: String,
    stage_name: OsString,
    owner: BootstrapPublicationOwner,
}

pub(crate) struct UnpublishedRemoteProjectPublication {
    org_root: PinnedRemoteDirectory,
    app: String,
    stage_name: OsString,
    owner: BootstrapVisibilityOwner,
}

/// Drop order is part of the safety contract: an unpublished stage is
/// quarantined and removed while both its hidden project gate and every
/// bootstrap namespace lock are still held.
struct BootstrapPublicationOwner {
    cleanup: BootstrapStageCleanup,
    gate_lock: StableFileLock,
    bootstrap_locks: Vec<StableFileLock>,
    decision_work: Option<BootstrapDecisionWorkOwner>,
}

/// The namespace half of an unpublished project. This value must be dropped
/// before the corresponding hidden project gate: its cleanup runs while both
/// the hidden gate and the bootstrap namespace locks are still held.
struct BootstrapVisibilityOwner {
    cleanup: BootstrapStageCleanup,
    #[allow(dead_code)] // Held to preserve namespace-lock lifetime through publication.
    bootstrap_locks: Vec<StableFileLock>,
    decision_work: Option<BootstrapDecisionWorkOwner>,
}

/// Upper bound for files and payload bytes which a caller will add to an
/// unpublished project before its sole namespace publication.  The fixed
/// descriptor and base layout are accounted separately by the bootstrap
/// owner; HTTP can add its object/authority postimage footprint without
/// exposing either the ledger or the bootstrap cleanup capability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct BootstrapStageFootprint {
    pub(crate) entries: u64,
    pub(crate) payload_bytes: u64,
}

impl BootstrapStageFootprint {
    #[allow(dead_code)] // Consumed by the HTTP bootstrap integration slice.
    pub(crate) const fn new(entries: u64, payload_bytes: u64) -> Self {
        Self {
            entries,
            payload_bytes,
        }
    }

    #[allow(dead_code)] // Used by HTTP to preflight a missing-project upload.
    pub(crate) fn maximum_payload_bytes(entries: u64) -> Option<u64> {
        let total_entries = BOOTSTRAP_FIXED_LAYOUT_ENTRIES.checked_add(entries)?;
        if total_entries > BOOTSTRAP_RECOVERY_ENTRY_LIMIT {
            return None;
        }
        let framing = total_entries.checked_mul(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
        BOOTSTRAP_RECOVERY_BYTE_LIMIT
            .checked_sub(framing)?
            .checked_sub(BOOTSTRAP_FIXED_LAYOUT_PAYLOAD_BYTES)
    }

    pub(crate) const fn maximum_entries() -> u64 {
        BOOTSTRAP_RECOVERY_ENTRY_LIMIT - BOOTSTRAP_FIXED_LAYOUT_ENTRIES
    }

    pub(crate) fn validate(self) -> Result<(), CliError> {
        validate_bootstrap_footprint(self)
    }
}

struct BootstrapDecisionWorkOwner {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    creation_permit: Option<codefire_core::DecisionWorkPermit>,
    rollback_permit: Option<codefire_core::DecisionWorkPermit>,
    visibility_permit: Option<codefire_core::DecisionWorkPermit>,
    footprint: BootstrapStageFootprint,
    creation_recorded: bool,
    armed: bool,
}

impl BootstrapDecisionWorkOwner {
    fn reserve(
        ledger: &mut codefire_core::DecisionWorkLedger,
        footprint: BootstrapStageFootprint,
    ) -> Result<Self, CliError> {
        validate_bootstrap_footprint(footprint)?;
        let creation = ledger
            .reserve(bootstrap_creation_reservation(
                BOOTSTRAP_FIXED_LAYOUT_PAYLOAD_BYTES,
            ))
            .map_err(|error| bootstrap_work_error(error, "reserve remote bootstrap creation"))?;
        let rollback = match ledger.reserve(bootstrap_rollback_reservation(footprint)) {
            Ok(permit) => permit,
            Err(error) => {
                ledger.cancel_reserved(&creation).map_err(|cancel| {
                    bootstrap_work_error(cancel, "cancel rejected bootstrap creation")
                })?;
                return Err(bootstrap_work_error(
                    error,
                    "reserve remote bootstrap rollback",
                ));
            }
        };
        let visibility = match ledger.reserve(bootstrap_visibility_reservation()) {
            Ok(permit) => permit,
            Err(error) => {
                ledger.cancel_reserved(&rollback).map_err(|cancel| {
                    bootstrap_work_error(cancel, "cancel rejected bootstrap rollback")
                })?;
                ledger.cancel_reserved(&creation).map_err(|cancel| {
                    bootstrap_work_error(cancel, "cancel rejected bootstrap creation")
                })?;
                return Err(bootstrap_work_error(
                    error,
                    "reserve remote bootstrap visibility",
                ));
            }
        };
        Ok(Self {
            ledger_identity: ledger.identity(),
            creation_permit: Some(creation),
            rollback_permit: Some(rollback),
            visibility_permit: Some(visibility),
            footprint,
            creation_recorded: false,
            armed: false,
        })
    }

    fn ensure_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(CliError::InvalidRepository(
                "remote bootstrap work belongs to a different decision-work ledger".to_string(),
            ))
        }
    }

    fn owns_prepublication_permits(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        self.ensure_ledger(ledger).is_ok()
            && self.creation_recorded
            && self.creation_permit.is_none()
            && self.rollback_permit.is_some()
            && self.visibility_permit.is_some()
            && !self.armed
    }

    fn record_creation(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        actual: codefire_core::DecisionWorkReservation,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        let mut permit = self.creation_permit.take().ok_or_else(|| {
            CliError::InvalidRepository("remote bootstrap creation permit is missing".to_string())
        })?;
        if let Err(error) = ledger.consume_reserved(&mut permit, actual) {
            self.creation_permit = Some(permit);
            return Err(bootstrap_work_error(
                error,
                "record remote bootstrap creation",
            ));
        }
        match ledger.cancel_reserved(&permit) {
            Ok(()) | Err(codefire_core::DecisionWorkAccountingError::InvalidPermit) => {}
            Err(error) => {
                self.creation_permit = Some(permit);
                return Err(bootstrap_work_error(
                    error,
                    "close remote bootstrap creation permit",
                ));
            }
        }
        self.creation_recorded = true;
        Ok(())
    }

    fn arm(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        if !self.creation_recorded || self.creation_permit.is_some() {
            return Err(CliError::InvalidRepository(
                "remote bootstrap creation accounting is incomplete".to_string(),
            ));
        }
        if !ledger.is_prepublication_sealed() {
            return Err(CliError::InvalidRepository(
                "remote bootstrap must be sealed before arming publication".to_string(),
            ));
        }
        if self.armed {
            return Ok(());
        }
        if self.rollback_permit.is_none() || self.visibility_permit.is_none() {
            return Err(CliError::InvalidRepository(
                "remote bootstrap arm permits are incomplete".to_string(),
            ));
        }
        let mut visibility = self.visibility_permit.take().ok_or_else(|| {
            CliError::InvalidRepository("remote bootstrap visibility permit is missing".to_string())
        })?;
        if let Err(error) =
            ledger.consume_reserved(&mut visibility, bootstrap_visibility_reservation())
        {
            self.visibility_permit = Some(visibility);
            return Err(bootstrap_work_error(
                error,
                "arm remote bootstrap visibility",
            ));
        }
        assert!(matches!(
            ledger.cancel_reserved(&visibility),
            Err(codefire_core::DecisionWorkAccountingError::InvalidPermit)
        ));
        let rollback = self
            .rollback_permit
            .take()
            .expect("prevalidated bootstrap rollback permit");
        ledger
            .cancel_reserved(&rollback)
            .expect("prevalidated bootstrap rollback permit must be cancellable");
        self.armed = true;
        Ok(())
    }

    fn cleanup_limits(&self) -> Result<(u64, u64), CliError> {
        bootstrap_cleanup_pass_limits(self.footprint)
    }

    fn cancel_without_effect(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        let mut first_error = None;
        for (permit, operation) in [
            (
                &mut self.visibility_permit,
                "cancel remote bootstrap visibility",
            ),
            (
                &mut self.rollback_permit,
                "cancel remote bootstrap rollback",
            ),
            (
                &mut self.creation_permit,
                "cancel remote bootstrap creation",
            ),
        ] {
            if let Some(permit) = permit.take() {
                if let Err(error) = ledger.cancel_reserved(&permit) {
                    first_error.get_or_insert_with(|| bootstrap_work_error(error, operation));
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn begin_rollback_attempt(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<codefire_core::DecisionWorkPermit, CliError> {
        self.ensure_ledger(ledger)?;
        if self.armed {
            return Err(CliError::InvalidRepository(
                "armed remote bootstrap cannot be cancelled with a terminal ledger".to_string(),
            ));
        }
        if let Some(permit) = self.rollback_permit.take() {
            Ok(permit)
        } else {
            ledger
                .reserve(bootstrap_rollback_reservation(self.footprint))
                .map_err(|error| {
                    bootstrap_work_error(error, "reserve remote bootstrap rollback retry")
                })
        }
    }

    fn finish_rollback_attempt(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut permit: codefire_core::DecisionWorkPermit,
        actual: codefire_core::DecisionWorkReservation,
        temporary: u64,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        if let Err(error) = ledger.consume_reserved(&mut permit, actual) {
            self.rollback_permit = Some(permit);
            return Err(bootstrap_work_error(
                error,
                "record remote bootstrap rollback",
            ));
        }
        if temporary > 0 {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    temporary,
                )
                .map_err(|error| {
                    bootstrap_work_error(error, "release remote bootstrap rollback scratch")
                })?;
        }
        match ledger.cancel_reserved(&permit) {
            Ok(()) | Err(codefire_core::DecisionWorkAccountingError::InvalidPermit) => {}
            Err(error) => {
                self.rollback_permit = Some(permit);
                return Err(bootstrap_work_error(
                    error,
                    "close remote bootstrap rollback",
                ));
            }
        }
        Ok(())
    }

    #[allow(dead_code)] // Reached by the HTTP ledger integration path.
    fn finish_publish_infallible(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) {
        assert!(ledger.matches_identity(self.ledger_identity));
        assert!(self.armed);
        assert!(self.visibility_permit.is_none());
        assert!(self.rollback_permit.is_none());
    }

    fn cancel_visibility(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.ensure_ledger(ledger)?;
        if let Some(permit) = self.visibility_permit.take() {
            ledger.cancel_reserved(&permit).map_err(|error| {
                bootstrap_work_error(error, "cancel unused bootstrap visibility")
            })?;
        }
        Ok(())
    }
}

fn bootstrap_work_error(
    error: codefire_core::DecisionWorkAccountingError,
    operation: &str,
) -> CliError {
    if let Some(exhaustion) = error.budget_exceeded() {
        CliError::from(exhaustion.clone())
    } else {
        CliError::InvalidRepository(format!("{operation}: {error}"))
    }
}

fn bootstrap_combined_error(primary: CliError, secondary: CliError, context: &str) -> CliError {
    CliError::InvalidRepository(format!(
        "{context}: {primary}; cleanup/accounting failed: {secondary}"
    ))
}

fn validate_bootstrap_footprint(footprint: BootstrapStageFootprint) -> Result<(), CliError> {
    let total_entries = BOOTSTRAP_FIXED_LAYOUT_ENTRIES
        .checked_add(footprint.entries)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote bootstrap entry overflow".to_string())
        })?;
    let total_bytes = total_entries
        .checked_mul(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)
        .and_then(|value| value.checked_add(BOOTSTRAP_FIXED_LAYOUT_PAYLOAD_BYTES))
        .and_then(|value| value.checked_add(footprint.payload_bytes))
        .ok_or_else(|| CliError::InvalidRepository("remote bootstrap byte overflow".to_string()))?;
    if total_entries > BOOTSTRAP_RECOVERY_ENTRY_LIMIT || total_bytes > BOOTSTRAP_RECOVERY_BYTE_LIMIT
    {
        return Err(CliError::InvalidRepository(format!(
            "remote bootstrap footprint exceeds bounded recovery limits (entries {total_entries}/{BOOTSTRAP_RECOVERY_ENTRY_LIMIT}, bytes {total_bytes}/{BOOTSTRAP_RECOVERY_BYTE_LIMIT})"
        )));
    }
    Ok(())
}

fn bootstrap_creation_reservation(payload_bytes: u64) -> codefire_core::DecisionWorkReservation {
    let aggregate = BOOTSTRAP_FIXED_LAYOUT_ENTRIES
        .saturating_mul(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)
        .saturating_add(payload_bytes);
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::Staging,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(
        codefire_core::DecisionWorkDimension::GraphNodes,
        BOOTSTRAP_FIXED_LAYOUT_ENTRIES,
    )
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        aggregate,
    )
    .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1)
}

fn bootstrap_creation_actual_reservation(
    entries: u64,
    aggregate_bytes: u64,
    object_records: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::Staging,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(codefire_core::DecisionWorkDimension::GraphNodes, entries)
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        aggregate_bytes,
    )
    .count(
        codefire_core::DecisionWorkCounter::ObjectRecords,
        object_records,
    )
}

fn bootstrap_rollback_reservation(
    footprint: BootstrapStageFootprint,
) -> codefire_core::DecisionWorkReservation {
    let (entries, bytes) = bootstrap_cleanup_pass_limits(footprint)
        .expect("validated bootstrap footprint has bounded cleanup limits");
    // Scan and remove are separate passes whose observations are summed.
    // TemporaryBytes is a peak, so it is reserved once.
    bootstrap_recovery_reservation(
        entries.saturating_mul(2),
        bytes.saturating_mul(2),
        BOOTSTRAP_CLEANUP_TEMPORARY_BYTES,
    )
}

fn bootstrap_cleanup_pass_limits(
    footprint: BootstrapStageFootprint,
) -> Result<(u64, u64), CliError> {
    let entries = BOOTSTRAP_FIXED_LAYOUT_ENTRIES
        .checked_add(footprint.entries)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote bootstrap cleanup entry overflow".to_string())
        })?;
    let bytes = BOOTSTRAP_FIXED_LAYOUT_PAYLOAD_BYTES
        .checked_add(footprint.payload_bytes)
        .and_then(|value| {
            value.checked_add(entries.checked_mul(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?)
        })
        .ok_or_else(|| {
            CliError::InvalidRepository("remote bootstrap cleanup byte overflow".to_string())
        })?;
    Ok((entries, bytes))
}

fn bootstrap_entry_frame_bytes(name: &OsStr) -> Option<u64> {
    u64::try_from(name.as_encoded_bytes().len())
        .ok()
        .and_then(|len| codefire_core::LOGICAL_FRAME_BASE_BYTES.checked_add(len))
}

fn bootstrap_tree_entry_payload_bytes(is_directory: bool, physical_len: u64) -> u64 {
    // Directory st_size is host allocator data (for example 4096 on ext4),
    // not a reserved recovery payload. File payload is the bounded term.
    if is_directory {
        0
    } else {
        physical_len
    }
}

fn bootstrap_visibility_reservation() -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::Staging,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        BOOTSTRAP_VISIBILITY_LOGICAL_BYTES,
    )
    .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1)
}

impl PrelockedRemoteProject {
    pub(crate) fn into_parts(self) -> (RemoteProject, StableFileLock, RemoteProjectVisibility) {
        (self.project, self.gate_lock, self.visibility)
    }
}

impl UnpublishedRemoteProject {
    pub(crate) fn project(&self) -> &RemoteProject {
        &self.project
    }

    pub(crate) fn publish<F>(self, before_publish: F) -> Result<PrelockedRemoteProject, CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        let (mut prelocked, mut publication) = self.into_hidden_parts();
        publication.publish_with_legacy_owner(
            &mut prelocked.project,
            &mut prelocked.gate_lock,
            before_publish,
        )?;
        prelocked.visibility = RemoteProjectVisibility::PublishedNow;
        Ok(prelocked)
    }

    pub(crate) fn into_hidden_parts(
        self,
    ) -> (PrelockedRemoteProject, UnpublishedRemoteProjectPublication) {
        let BootstrapPublicationOwner {
            cleanup,
            gate_lock,
            bootstrap_locks,
            decision_work,
        } = self.owner;
        (
            PrelockedRemoteProject {
                project: self.project,
                gate_lock,
                visibility: RemoteProjectVisibility::Unpublished,
            },
            UnpublishedRemoteProjectPublication {
                org_root: self.org_root,
                app: self.app,
                stage_name: self.stage_name,
                owner: BootstrapVisibilityOwner {
                    cleanup,
                    bootstrap_locks,
                    decision_work,
                },
            },
        )
    }
}

impl UnpublishedRemoteProjectPublication {
    #[cfg(test)]
    pub(crate) fn inject_bounded_cleanup_failure_once(&mut self) {
        self.owner.cleanup.fail_bounded_cleanup_once = true;
    }

    #[cfg(test)]
    pub(crate) fn final_project_root_for_test(&self) -> PathBuf {
        self.org_root.display_path.join(&self.app)
    }

    pub(crate) fn owns_prepublication_permits(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> bool {
        self.owner
            .decision_work
            .as_ref()
            .is_some_and(|work| work.owns_prepublication_permits(ledger))
    }

    #[allow(dead_code)] // Consumed by the HTTP bootstrap integration slice.
    pub(crate) fn arm_with_ledger(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.owner
            .decision_work
            .as_mut()
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "legacy remote bootstrap has no request decision-work owner".to_string(),
                )
            })?
            .arm(ledger)?;
        // After arming, all request permits are closed and a callback may
        // terminalize the ledger. Drop therefore performs no filesystem I/O;
        // an unpublished owned descriptor is recovered by the next bounded
        // bootstrap pass.
        self.owner.cleanup.drop_quarantine = false;
        Ok(())
    }

    #[allow(dead_code)] // Consumed by the HTTP bootstrap integration slice.
    pub(crate) fn cancel_with_ledger(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        let (limits, permit) = {
            let work = self.owner.decision_work.as_mut().ok_or_else(|| {
                CliError::InvalidRepository(
                    "legacy remote bootstrap has no request decision-work owner".to_string(),
                )
            })?;
            work.ensure_ledger(ledger)?;
            let limits = work.cleanup_limits()?;
            let permit = work.begin_rollback_attempt(ledger)?;
            (limits, permit)
        };
        let scan = self.owner.cleanup.scan_bounded_attempt(limits.0, limits.1);
        let removal = if scan.result.is_ok() {
            self.owner
                .cleanup
                .remove_bounded_attempt(limits.0, limits.1)
        } else {
            BootstrapCleanupAttempt {
                observation: BootstrapCleanupObservation::default(),
                result: Ok(()),
            }
        };
        let observation = scan.observation.combine(removal.observation);
        let accounting = self
            .owner
            .decision_work
            .as_mut()
            .expect("validated bootstrap work owner")
            .finish_rollback_attempt(
                ledger,
                permit,
                observation.reservation(),
                observation.temporary_peak,
            );
        let visibility = self
            .owner
            .decision_work
            .as_mut()
            .expect("validated bootstrap work owner")
            .cancel_visibility(ledger);
        accounting?;
        visibility?;
        scan.result?;
        removal.result
    }

    /// Publish a ledger-bound stage without consuming its cleanup owner on
    /// any fallible precondition or callback/rename failure. The caller may
    /// correct the ledger, retry publication, or explicitly cancel using the
    /// same value.
    #[allow(dead_code)] // Reached by the HTTP ledger integration path.
    pub(crate) fn publish_with_ledger<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        project: &mut RemoteProject,
        gate_lock: &mut StableFileLock,
        before_publish: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        self.publish_with_ledger_core(ledger, project, gate_lock, |_| before_publish())
    }

    fn publish_with_ledger_core<F, T>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        project: &mut RemoteProject,
        gate_lock: &mut StableFileLock,
        before_publish: F,
    ) -> Result<T, CliError>
    where
        F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<T, CliError>,
    {
        let work = self.owner.decision_work.as_ref().ok_or_else(|| {
            CliError::InvalidRepository(
                "legacy remote bootstrap has no request decision-work owner".to_string(),
            )
        })?;
        work.ensure_ledger(ledger)?;
        if !work.armed {
            return Err(CliError::InvalidRepository(
                "ledger-bound remote bootstrap must be armed before publication".to_string(),
            ));
        }
        project.ensure_bound()?;
        project.ensure_gate_lock_subject(gate_lock)?;
        let rebound_stage = self
            .org_root
            .directory
            .open_dir_nofollow(&self.stage_name)?;
        project
            .root
            .identity
            .ensure_directory(&rebound_stage, "remote bootstrap stage")?;
        match self.org_root.directory.symlink_metadata(&self.app) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(CliError::Io(std::io::Error::from(
                    std::io::ErrorKind::AlreadyExists,
                )))
            }
            Err(error) => return Err(CliError::Io(error)),
        }
        let final_root = self.org_root.display_path.join(&self.app);
        let final_gate_path = final_root.join("locks/project-gate.lock");
        let gate_label = gate_lock.prepare_display_path_label(&final_gate_path)?;
        let project_paths = PreparedRemoteProjectDisplayPaths::new(final_root);

        let callback_output = before_publish(ledger)?;
        #[cfg(test)]
        LEDGER_BOOTSTRAP_RENAME_ATTEMPTS.set(
            LEDGER_BOOTSTRAP_RENAME_ATTEMPTS
                .get()
                .checked_add(1)
                .expect("bootstrap rename attempt counter overflow"),
        );
        #[cfg(test)]
        if INJECT_LEDGER_BOOTSTRAP_RENAME_FAILURE.replace(false) {
            return Err(CliError::Io(std::io::Error::from(
                std::io::ErrorKind::PermissionDenied,
            )));
        }
        secure_worktree::rename_entry_noreplace(
            &self.org_root.directory,
            &self.stage_name,
            OsStr::new(&self.app),
        )?;

        // Every operation after the namespace rename is infallible: the
        // permits and display labels were validated above and the stage is
        // now the final project directory.
        self.owner
            .decision_work
            .as_mut()
            .expect("validated decision-work owner")
            .finish_publish_infallible(ledger);
        self.owner.cleanup.published = true;
        gate_lock.adopt_prepared_display_path_label(gate_label);
        project.adopt_prepared_display_paths(project_paths);
        Ok(callback_output)
    }

    pub(crate) fn publish_with_terminal_ledger<F, T>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        project: &mut RemoteProject,
        gate_lock: &mut StableFileLock,
        before_publish: F,
    ) -> Result<T, CliError>
    where
        F: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<T, CliError>,
    {
        self.publish_with_ledger_core(ledger, project, gate_lock, before_publish)
    }

    pub(crate) fn publish_with_legacy_owner<F>(
        &mut self,
        project: &mut RemoteProject,
        gate_lock: &mut StableFileLock,
        before_publish: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        if self.owner.decision_work.is_some() {
            return Err(CliError::InvalidRepository(
                "ledger-bound remote bootstrap requires borrowed publish_with_ledger".to_string(),
            ));
        }
        project.ensure_bound()?;
        project.ensure_gate_lock_subject(gate_lock)?;
        let rebound_stage = self
            .org_root
            .directory
            .open_dir_nofollow(&self.stage_name)?;
        project
            .root
            .identity
            .ensure_directory(&rebound_stage, "remote bootstrap stage")?;
        match self.org_root.directory.symlink_metadata(&self.app) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(CliError::Io(std::io::Error::from(
                    std::io::ErrorKind::AlreadyExists,
                )))
            }
            Err(error) => return Err(CliError::Io(error)),
        }
        let final_root = self.org_root.display_path.join(&self.app);
        let final_gate_path = final_root.join("locks/project-gate.lock");
        let gate_label = gate_lock.prepare_display_path_label(&final_gate_path)?;
        let project_paths = PreparedRemoteProjectDisplayPaths::new(final_root);

        // This is the sole cancellable boundary. All identities, the gate
        // subject, and the final display labels have already been validated.
        if self
            .owner
            .decision_work
            .as_ref()
            .is_some_and(|work| !work.armed)
        {
            return Err(CliError::InvalidRepository(
                "ledger-bound remote bootstrap must be armed before publication".to_string(),
            ));
        }
        if let Err(error) = before_publish() {
            let cleanup = self.cleanup_bounded_without_ledger();
            return match cleanup {
                Ok(()) => Err(error),
                Err(cleanup) => Err(CliError::InvalidRepository(format!(
                    "remote bootstrap publication was cancelled ({error}); cleanup failed: {cleanup}"
                ))),
            };
        }
        if let Err(error) = secure_worktree::rename_entry_noreplace(
            &self.org_root.directory,
            &self.stage_name,
            OsStr::new(&self.app),
        ) {
            let cleanup = self.cleanup_bounded_without_ledger();
            return match cleanup {
                Ok(()) => Err(CliError::Io(error)),
                Err(cleanup) => Err(CliError::InvalidRepository(format!(
                    "remote bootstrap visibility rename failed ({error}); cleanup failed: {cleanup}"
                ))),
            };
        }

        // Infallible tail: after namespace visibility changes, only ownership
        // flags and already-prepared labels are adopted. No re-open, identity
        // check, allocation, lock acquisition, or fallible conversion remains.
        self.owner.cleanup.published = true;
        gate_lock.adopt_prepared_display_path_label(gate_label);
        project.adopt_prepared_display_paths(project_paths);
        Ok(())
    }

    fn cleanup_bounded_without_ledger(&mut self) -> Result<(), CliError> {
        let (entry_limit, byte_limit) = self
            .owner
            .decision_work
            .as_ref()
            .map(BootstrapDecisionWorkOwner::cleanup_limits)
            .transpose()?
            .unwrap_or((
                BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
                BOOTSTRAP_RECOVERY_BYTE_LIMIT,
            ));
        self.owner.cleanup.remove_bounded(entry_limit, byte_limit)
    }
}

pub(crate) struct RemoteTransaction {
    transaction_id: String,
    directory: PinnedRemoteDirectory,
}

struct RemoteAuthorityLocation {
    directory: Dir,
    file_name: OsString,
    display_path: PathBuf,
}

impl RemoteProject {
    pub(crate) fn open(project_root: &Path) -> Result<Self, CliError> {
        let root_dir = secure_worktree::open_absolute_dir_nofollow(project_root)?;
        let root_identity = secure_worktree::StableFsIdentity::for_directory(&root_dir)?;
        let canonical_root = fs::canonicalize(project_root)?;
        let rebound = secure_worktree::open_absolute_dir_nofollow(&canonical_root)?;
        root_identity.ensure_directory(&rebound, "remote project root")?;
        let root = PinnedRemoteDirectory {
            directory: root_dir,
            identity: root_identity,
            display_path: canonical_root.clone(),
        };
        let (projects_root, org, app) = project_coordinates(&canonical_root)?;
        let _ = projects_root;
        let project = Self::from_pinned_root(root, &org, &app)?;
        project.ensure_bound()?;
        Ok(project)
    }

    fn from_pinned_root(
        root: PinnedRemoteDirectory,
        org: &str,
        app: &str,
    ) -> Result<Self, CliError> {
        let display_root = root.display_path.clone();
        let (descriptor_bytes, descriptor_identity) =
            secure_worktree::read_file_in_directory_nofollow_bounded_with_identity(
                &root.directory,
                OsStr::new(PROJECT_DESCRIPTOR_FILE),
                &display_root.join(PROJECT_DESCRIPTOR_FILE),
                16 * 1024,
            )?;
        let descriptor: Value = serde_json::from_slice(&descriptor_bytes)?;
        validate_descriptor_value(&descriptor, &display_root, org, app)?;
        let stable_id = descriptor
            .get("stable_project_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CliError::InvalidRepository("remote project stable identity is missing".to_string())
            })?
            .to_string();
        Ok(Self {
            display_root,
            objects: open_pinned_child(&root, "objects")?,
            branches: open_pinned_child(&root, "branches")?,
            merge_requests: open_pinned_child(&root, "merge_requests")?,
            locks: open_pinned_child(&root, "locks")?,
            idempotency: open_pinned_child(&root, "idempotency")?,
            transactions: open_pinned_child(&root, "transactions")?,
            gc: open_pinned_child(&root, "gc")?,
            descriptor_identity,
            descriptor_digest: sha256_hex(&descriptor_bytes),
            root,
            stable_id,
        })
    }

    pub(crate) fn display_root(&self) -> &Path {
        &self.display_root
    }

    pub(crate) fn stable_id(&self) -> &str {
        &self.stable_id
    }

    #[cfg(test)]
    pub(crate) fn ensure_object_store_subject(
        &self,
        subject: &codefire_store::ValidatedObjectStoreSubject,
    ) -> Result<(), CliError> {
        self.objects.identity.ensure_directory(
            &self.objects.directory,
            "remote project object-store directory",
        )?;
        subject
            .ensure_directory(&self.objects.directory)
            .map_err(CliError::from)
    }

    pub(crate) fn ensure_provisional_object_store_subject(
        &self,
        subject: &codefire_store::ProvisionalSealedValidationSubject<'_>,
    ) -> Result<(), CliError> {
        self.objects.identity.ensure_directory(
            &self.objects.directory,
            "remote project object-store directory",
        )?;
        subject
            .ensure_object_store_directory(&self.objects.directory)
            .map_err(CliError::from)
    }

    pub(crate) fn acquire_gate_lock(
        &self,
        mode: StableLockMode,
        deadline: &LockDeadline,
    ) -> Result<StableFileLock, CliError> {
        self.ensure_bound()?;
        let lock_name = OsStr::new("project-gate.lock");
        let lock = StableFileLock::acquire_in_directory(
            &self.locks.directory,
            lock_name,
            &self.locks.display_path.join(lock_name),
            mode,
            deadline,
        )?;
        self.ensure_bound()?;
        Ok(lock)
    }

    fn acquire_hidden_gate_lock(
        &self,
        deadline: &LockDeadline,
    ) -> Result<StableFileLock, CliError> {
        self.ensure_bound()?;
        let lock_name = OsStr::new("project-gate.lock");
        StableFileLock::acquire_in_directory(
            &self.locks.directory,
            lock_name,
            &self.locks.display_path.join(lock_name),
            StableLockMode::Exclusive,
            deadline,
        )
    }

    pub(crate) fn ensure_gate_lock_subject(&self, lock: &StableFileLock) -> Result<(), CliError> {
        self.ensure_bound()?;
        let lock_name = OsStr::new("project-gate.lock");
        lock.ensure_subject(
            &self.locks.directory,
            lock_name,
            &self.locks.display_path.join(lock_name),
        )?;
        self.ensure_bound()
    }

    fn adopt_prepared_display_paths(&mut self, paths: PreparedRemoteProjectDisplayPaths) {
        self.display_root = paths.display_root;
        self.root.display_path = paths.root;
        self.objects.display_path = paths.objects;
        self.branches.display_path = paths.branches;
        self.merge_requests.display_path = paths.merge_requests;
        self.locks.display_path = paths.locks;
        self.idempotency.display_path = paths.idempotency;
        self.transactions.display_path = paths.transactions;
        self.gc.display_path = paths.gc;
    }

    pub(crate) fn ensure_bound(&self) -> Result<(), CliError> {
        self.root
            .identity
            .ensure_directory(&self.root.directory, "remote project root")?;
        let rebound = secure_worktree::open_absolute_dir_nofollow(&self.display_root)?;
        self.root
            .identity
            .ensure_directory(&rebound, "remote project rebound root")?;
        let (descriptor, descriptor_identity) =
            secure_worktree::read_file_in_directory_nofollow_bounded_with_identity(
                &self.root.directory,
                OsStr::new(PROJECT_DESCRIPTOR_FILE),
                &self.display_root.join(PROJECT_DESCRIPTOR_FILE),
                16 * 1024,
            )?;
        if descriptor_identity != self.descriptor_identity
            || sha256_hex(&descriptor) != self.descriptor_digest
        {
            return Err(CliError::InvalidRepository(format!(
                "remote project descriptor changed while in use: {}",
                self.display_root.join(PROJECT_DESCRIPTOR_FILE).display()
            )));
        }
        for (name, pinned) in [
            ("objects", &self.objects),
            ("branches", &self.branches),
            ("merge_requests", &self.merge_requests),
            ("locks", &self.locks),
            ("idempotency", &self.idempotency),
            ("transactions", &self.transactions),
            ("gc", &self.gc),
        ] {
            pinned.identity.ensure_directory(
                &pinned.directory,
                &format!("remote project {name} directory"),
            )?;
            let rebound = self.root.directory.open_dir_nofollow(name)?;
            pinned.identity.ensure_directory(
                &rebound,
                &format!("remote project rebound {name} directory"),
            )?;
        }
        Ok(())
    }

    pub(crate) fn read_authority_bounded(
        &self,
        authority: &RemoteAuthority,
        limit: u64,
    ) -> Result<Vec<u8>, CliError> {
        self.ensure_bound()?;
        let location = self.authority_location(authority, false)?;
        let bytes = secure_worktree::read_file_in_directory_nofollow_bounded(
            &location.directory,
            &location.file_name,
            &location.display_path,
            limit,
        )?;
        self.ensure_bound()?;
        Ok(bytes)
    }

    pub(crate) fn write_authority_atomic(
        &self,
        authority: &RemoteAuthority,
        contents: &[u8],
    ) -> Result<(), CliError> {
        self.ensure_bound()?;
        let location = self.authority_location(authority, true)?;
        let temp_path = unique_temp_path(&location.display_path);
        let temp_name = temp_path.file_name().ok_or_else(|| {
            CliError::InvalidRepository("remote authority temp name is missing".to_string())
        })?;
        secure_worktree::write_file_atomic_in_directory_nofollow(
            &location.directory,
            &location.file_name,
            temp_name,
            contents,
        )?;
        self.ensure_bound()?;
        Ok(())
    }

    pub(crate) fn write_authority_create_noreplace(
        &self,
        authority: &RemoteAuthority,
        contents: &[u8],
    ) -> Result<(), CliError> {
        self.ensure_bound()?;
        let location = self.authority_location(authority, true)?;
        let temp_path = unique_temp_path(&location.display_path);
        let temp_name = temp_path.file_name().ok_or_else(|| {
            CliError::InvalidRepository("remote authority temp name is missing".to_string())
        })?;
        secure_worktree::write_file_create_noreplace_in_directory_nofollow(
            &location.directory,
            &location.file_name,
            temp_name,
            contents,
        )?;
        self.ensure_bound()?;
        Ok(())
    }

    pub(crate) fn read_authority_bounded_with_identity(
        &self,
        authority: &RemoteAuthority,
        limit: u64,
    ) -> Result<(Vec<u8>, secure_worktree::StableFsIdentity), CliError> {
        self.ensure_bound()?;
        let location = self.authority_location(authority, false)?;
        let result = secure_worktree::read_file_in_directory_nofollow_bounded_with_identity(
            &location.directory,
            &location.file_name,
            &location.display_path,
            limit,
        )?;
        self.ensure_bound()?;
        Ok(result)
    }

    pub(crate) fn remove_authority_quarantined(
        &self,
        authority: &RemoteAuthority,
        quarantine_name: &OsStr,
        expected: secure_worktree::StableFsIdentity,
    ) -> Result<(), CliError> {
        self.ensure_bound()?;
        let location = self.authority_location(authority, false)?;
        secure_worktree::remove_file_quarantined_in_directory_nofollow(
            &location.directory,
            &location.file_name,
            quarantine_name,
            expected,
        )?;
        self.ensure_bound()?;
        Ok(())
    }

    pub(crate) fn create_transaction(
        &self,
        transaction_id: &str,
    ) -> Result<RemoteTransaction, CliError> {
        validate_remote_component(transaction_id, "transaction ID")?;
        self.ensure_bound()?;
        self.transactions.directory.create_dir(transaction_id)?;
        secure_worktree::sync_capability_directory_best_effort(&self.transactions.directory)?;
        let transaction = self.open_transaction(transaction_id)?;
        self.ensure_bound()?;
        Ok(transaction)
    }

    pub(crate) fn open_transaction(
        &self,
        transaction_id: &str,
    ) -> Result<RemoteTransaction, CliError> {
        validate_remote_component(transaction_id, "transaction ID")?;
        self.ensure_bound()?;
        let directory = open_pinned_child(&self.transactions, transaction_id)?;
        let transaction = RemoteTransaction {
            transaction_id: transaction_id.to_string(),
            directory,
        };
        transaction.ensure_bound(self)?;
        Ok(transaction)
    }

    pub(crate) fn remove_transaction_directory(
        &self,
        transaction: &RemoteTransaction,
    ) -> Result<(), CliError> {
        transaction.ensure_bound(self)?;
        let quarantine = format!(".removing-{}", transaction.transaction_id);
        secure_worktree::remove_directory_quarantined_in_directory_nofollow(
            &self.transactions.directory,
            OsStr::new(&transaction.transaction_id),
            OsStr::new(&quarantine),
            transaction.directory.identity,
        )?;
        self.ensure_bound()?;
        Ok(())
    }

    pub(crate) fn remove_transaction_quarantine_if_present(
        &self,
        transaction_id: &str,
    ) -> Result<bool, CliError> {
        validate_remote_component(transaction_id, "transaction ID")?;
        self.ensure_bound()?;
        let quarantine = format!(".removing-{transaction_id}");
        let directory = match self.transactions.directory.open_dir_nofollow(&quarantine) {
            Ok(directory) => directory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(CliError::Io(error)),
        };
        let identity = secure_worktree::StableFsIdentity::for_directory(&directory)?;
        secure_worktree::remove_directory_quarantined_in_directory_nofollow(
            &self.transactions.directory,
            OsStr::new(&quarantine),
            OsStr::new(&quarantine),
            identity,
        )?;
        self.ensure_bound()?;
        Ok(true)
    }

    pub(crate) fn pending_quarantine_bounded(
        &self,
        limit: u64,
    ) -> Result<Option<(OsString, Vec<u8>, secure_worktree::StableFsIdentity)>, CliError> {
        self.ensure_bound()?;
        let mut candidate = None;
        for entry in self.root.directory.entries()? {
            let name = entry?.file_name();
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
            let (bytes, identity) =
                secure_worktree::read_file_in_directory_nofollow_bounded_with_identity(
                    &self.root.directory,
                    &name,
                    &self.display_root.join(&name),
                    limit,
                )?;
            candidate = Some((name, bytes, identity));
        }
        self.ensure_bound()?;
        Ok(candidate)
    }

    pub(crate) fn remove_pending_quarantine_exact(
        &self,
        name: &OsStr,
        expected: secure_worktree::StableFsIdentity,
    ) -> Result<(), CliError> {
        let text = name.to_str().ok_or_else(|| {
            CliError::InvalidRepository(
                "remote pending transaction quarantine name is not UTF-8".to_string(),
            )
        })?;
        if !text.starts_with(".pending_transaction.removing-") || !text.ends_with(".json") {
            return Err(CliError::InvalidRepository(
                "remote pending transaction quarantine name is invalid".to_string(),
            ));
        }
        self.ensure_bound()?;
        secure_worktree::remove_file_quarantined_in_directory_nofollow(
            &self.root.directory,
            name,
            name,
            expected,
        )?;
        self.ensure_bound()?;
        Ok(())
    }

    fn authority_location(
        &self,
        authority: &RemoteAuthority,
        create_parents: bool,
    ) -> Result<RemoteAuthorityLocation, CliError> {
        let (directory, file_name, display_path) = match authority {
            RemoteAuthority::Branch(branch) => {
                validate_remote_component(branch, "branch")?;
                let name = super::remote_branch_file_name(branch);
                (
                    self.branches.directory.try_clone()?,
                    OsString::from(&name),
                    self.branches.display_path.join(name),
                )
            }
            RemoteAuthority::MergeRequest(mr_id) => {
                validate_remote_component(mr_id, "merge-request ID")?;
                let name = super::remote_merge_request_file_name(mr_id);
                (
                    self.merge_requests.directory.try_clone()?,
                    OsString::from(&name),
                    self.merge_requests.display_path.join(name),
                )
            }
            RemoteAuthority::Generation => (
                self.root.directory.try_clone()?,
                OsString::from("gc_state.json"),
                self.display_root.join("gc_state.json"),
            ),
            RemoteAuthority::PendingTransaction => (
                self.root.directory.try_clone()?,
                OsString::from("pending_transaction.json"),
                self.display_root.join("pending_transaction.json"),
            ),
            RemoteAuthority::RequestNonceCache => (
                self.root.directory.try_clone()?,
                OsString::from("request_nonce_cache.json"),
                self.display_root.join("request_nonce_cache.json"),
            ),
            RemoteAuthority::IdempotencyReceipt { operation, digest } => {
                validate_remote_component(operation, "operation")?;
                validate_hex_digest(digest, "idempotency receipt")?;
                let directory = open_or_create_child(&self.idempotency, "v2", create_parents)?;
                let operation_dir = open_or_create_child(&directory, operation, create_parents)?;
                let name = format!("{digest}.json");
                (
                    operation_dir.directory.try_clone()?,
                    OsString::from(&name),
                    operation_dir.display_path.join(name),
                )
            }
            RemoteAuthority::GcAudit(plan_id) => {
                validate_remote_component(plan_id, "GC plan ID")?;
                let name = super::remote_gc_audit_file_name(plan_id);
                (
                    self.gc.directory.try_clone()?,
                    OsString::from(&name),
                    self.gc.display_path.join(name),
                )
            }
        };
        Ok(RemoteAuthorityLocation {
            directory,
            file_name,
            display_path,
        })
    }
}

impl RemoteTransaction {
    pub(crate) fn id(&self) -> &str {
        &self.transaction_id
    }

    fn ensure_bound(&self, project: &RemoteProject) -> Result<(), CliError> {
        project.ensure_bound()?;
        self.directory
            .identity
            .ensure_directory(&self.directory.directory, "remote transaction directory")?;
        let rebound = project
            .transactions
            .directory
            .open_dir_nofollow(&self.transaction_id)?;
        self.directory
            .identity
            .ensure_directory(&rebound, "remote transaction rebound directory")?;
        Ok(())
    }

    pub(crate) fn write_create_noreplace(
        &self,
        project: &RemoteProject,
        file_name: &str,
        contents: &[u8],
    ) -> Result<(), CliError> {
        validate_remote_component(file_name, "transaction file")?;
        self.ensure_bound(project)?;
        let display_path = self.directory.display_path.join(file_name);
        let temp_path = unique_temp_path(&display_path);
        let temp_name = temp_path.file_name().ok_or_else(|| {
            CliError::InvalidRepository("remote transaction temp name is missing".to_string())
        })?;
        secure_worktree::write_file_create_noreplace_in_directory_nofollow(
            &self.directory.directory,
            OsStr::new(file_name),
            temp_name,
            contents,
        )?;
        self.ensure_bound(project)?;
        Ok(())
    }

    pub(crate) fn read_bounded(
        &self,
        project: &RemoteProject,
        file_name: &str,
        limit: u64,
    ) -> Result<Vec<u8>, CliError> {
        validate_remote_component(file_name, "transaction file")?;
        self.ensure_bound(project)?;
        let bytes = secure_worktree::read_file_in_directory_nofollow_bounded(
            &self.directory.directory,
            OsStr::new(file_name),
            &self.directory.display_path.join(file_name),
            limit,
        )?;
        self.ensure_bound(project)?;
        Ok(bytes)
    }

    pub(crate) fn entry_names_bounded(
        &self,
        project: &RemoteProject,
        limit: usize,
    ) -> Result<Vec<OsString>, CliError> {
        self.ensure_bound(project)?;
        let mut names = Vec::new();
        for entry in self.directory.directory.entries()? {
            if names.len() == limit {
                return Err(CliError::InvalidRepository(format!(
                    "remote transaction staging exceeds {limit} entries"
                )));
            }
            names.push(entry?.file_name());
        }
        self.ensure_bound(project)?;
        Ok(names)
    }

    pub(crate) fn remove_file_exact(
        &self,
        project: &RemoteProject,
        file_name: &OsStr,
        expected: secure_worktree::StableFsIdentity,
    ) -> Result<(), CliError> {
        let name = file_name.to_str().ok_or_else(|| {
            CliError::InvalidRepository("remote transaction filename is not UTF-8".to_string())
        })?;
        validate_remote_component(name, "transaction file")?;
        self.ensure_bound(project)?;
        let quarantine = OsString::from(format!(".{name}.removing-{}", self.transaction_id));
        secure_worktree::remove_file_quarantined_in_directory_nofollow(
            &self.directory.directory,
            file_name,
            &quarantine,
            expected,
        )?;
        self.ensure_bound(project)?;
        Ok(())
    }

    pub(crate) fn remove_quarantined_file_exact(
        &self,
        project: &RemoteProject,
        quarantine_name: &OsStr,
        expected: secure_worktree::StableFsIdentity,
    ) -> Result<(), CliError> {
        let name = quarantine_name.to_str().ok_or_else(|| {
            CliError::InvalidRepository("remote transaction filename is not UTF-8".to_string())
        })?;
        validate_remote_component(name, "transaction quarantine file")?;
        self.ensure_bound(project)?;
        secure_worktree::remove_file_quarantined_in_directory_nofollow(
            &self.directory.directory,
            quarantine_name,
            quarantine_name,
            expected,
        )?;
        self.ensure_bound(project)?;
        Ok(())
    }

    pub(crate) fn read_bounded_with_identity(
        &self,
        project: &RemoteProject,
        file_name: &str,
        limit: u64,
    ) -> Result<(Vec<u8>, secure_worktree::StableFsIdentity), CliError> {
        validate_remote_component(file_name, "transaction file")?;
        self.ensure_bound(project)?;
        let result = secure_worktree::read_file_in_directory_nofollow_bounded_with_identity(
            &self.directory.directory,
            OsStr::new(file_name),
            &self.directory.display_path.join(file_name),
            limit,
        )?;
        self.ensure_bound(project)?;
        Ok(result)
    }
}

fn open_pinned_child(
    parent: &PinnedRemoteDirectory,
    name: &str,
) -> Result<PinnedRemoteDirectory, CliError> {
    let directory = parent.directory.open_dir_nofollow(name)?;
    let identity = secure_worktree::StableFsIdentity::for_directory(&directory)?;
    Ok(PinnedRemoteDirectory {
        directory,
        identity,
        display_path: parent.display_path.join(name),
    })
}

fn open_pinned_absolute(path: &Path) -> Result<PinnedRemoteDirectory, CliError> {
    let directory = secure_worktree::open_absolute_dir_nofollow(path)?;
    let identity = secure_worktree::StableFsIdentity::for_directory(&directory)?;
    let canonical = fs::canonicalize(path)?;
    let rebound = secure_worktree::open_absolute_dir_nofollow(&canonical)?;
    identity.ensure_directory(&rebound, "remote bootstrap directory")?;
    Ok(PinnedRemoteDirectory {
        directory,
        identity,
        display_path: canonical,
    })
}

fn open_or_create_pinned_absolute(path: &Path) -> Result<PinnedRemoteDirectory, CliError> {
    match open_pinned_absolute(path) {
        Ok(directory) => Ok(directory),
        Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent_path = path.parent().ok_or_else(|| {
                CliError::InvalidRepository(
                    "remote bootstrap path has no existing ancestor".to_string(),
                )
            })?;
            let name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
                CliError::InvalidRepository(
                    "remote bootstrap path has no UTF-8 component".to_string(),
                )
            })?;
            let parent = open_or_create_pinned_absolute(parent_path)?;
            open_or_create_child(&parent, name, true)
        }
        Err(error) => Err(error),
    }
}

fn open_or_create_child(
    parent: &PinnedRemoteDirectory,
    name: &str,
    create: bool,
) -> Result<PinnedRemoteDirectory, CliError> {
    if create {
        match parent.directory.create_dir(name) {
            Ok(()) => secure_worktree::sync_capability_directory_best_effort(&parent.directory)?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    open_pinned_child(parent, name)
}

/// Validate one already-decoded external authority component.
///
/// This function intentionally operates before any path derivation or write.
/// The same contract is mirrored by the Python file-server adapter.
pub(crate) fn validate_remote_component(value: &str, kind: &str) -> Result<(), CliError> {
    let invalid_shape = value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > REMOTE_COMPONENT_BYTE_LIMIT
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains('/')
        || value.contains('\\')
        || value.contains(':')
        || value.starts_with(REMOTE_INTERNAL_HASH_PREFIX)
        || value.chars().any(char::is_control);
    let mut components = Path::new(value).components();
    let exactly_one_normal =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    if invalid_shape || !exactly_one_normal {
        return Err(CliError::Usage(format!(
            "invalid remote {kind} component: {value:?}"
        )));
    }
    Ok(())
}

pub(crate) fn validated_remote_component(value: String, kind: &str) -> Result<String, CliError> {
    validate_remote_component(&value, kind)?;
    Ok(value)
}

pub(crate) fn ensure_remote_project_layout(project_root: &Path) -> Result<(), CliError> {
    let deadline = LockDeadline::from_options(&LockOptions::default());
    ensure_remote_project_layout_prelocked(project_root, &deadline, || Ok(())).map(drop)
}

pub(crate) fn ensure_remote_project_layout_prelocked<F>(
    project_root: &Path,
    deadline: &LockDeadline,
    before_new_project_publish: F,
) -> Result<PrelockedRemoteProject, CliError>
where
    F: FnOnce() -> Result<(), CliError>,
{
    let (projects_root, org, app) = project_coordinates(project_root)?;
    let projects_parent_path = projects_root.parent().ok_or_else(|| {
        CliError::InvalidRepository("remote projects root has no parent".to_string())
    })?;
    let projects_name = projects_root
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote projects root has no UTF-8 name".to_string())
        })?;
    let projects_parent = open_or_create_pinned_absolute(projects_parent_path)?;
    let projects = open_or_create_child(&projects_parent, projects_name, true)?;
    let bootstrap_locks = open_or_create_child(&projects, ".bootstrap-locks", true)?;
    let bootstrap_guards =
        acquire_bootstrap_namespace_locks(&bootstrap_locks, &org, &app, deadline)?;

    let org_root = open_or_create_child(&projects, &org, true)?;
    recover_owned_bootstrap_stages(&org_root, &org, &app, &mut BootstrapRecoveryWork::bounded())
        .map_err(BootstrapRecoveryError::into_cli)?;
    match org_root.directory.symlink_metadata(&app) {
        Ok(_) => {
            let existing = open_pinned_child(&org_root, &app)?;
            initialize_or_validate_existing(&existing, &org, &app)?;
            let project = RemoteProject::open(project_root)?;
            let gate_lock = project.acquire_gate_lock(StableLockMode::Exclusive, deadline)?;
            project.ensure_gate_lock_subject(&gate_lock)?;
            Ok(PrelockedRemoteProject {
                project,
                gate_lock,
                visibility: RemoteProjectVisibility::Existing,
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_unpublished_remote_project(
                org_root,
                &org,
                app,
                bootstrap_guards,
                deadline,
                None,
            )?
            .publish(before_new_project_publish)
        }
        Err(error) => Err(CliError::Io(error)),
    }
}

pub(crate) fn prepare_unpublished_remote_project(
    project_root: &Path,
    deadline: &LockDeadline,
) -> Result<Option<UnpublishedRemoteProject>, CliError> {
    prepare_unpublished_remote_project_with_recovery_work(
        project_root,
        deadline,
        &mut BootstrapRecoveryWork::bounded(),
        None,
    )
}

#[allow(dead_code)] // Reached through the next HTTP bootstrap integration slice.
pub(crate) fn prepare_unpublished_remote_project_with_ledger(
    project_root: &Path,
    deadline: &LockDeadline,
    ledger: &mut codefire_core::DecisionWorkLedger,
    footprint: BootstrapStageFootprint,
) -> Result<Option<UnpublishedRemoteProject>, CliError> {
    let mut recovery =
        BootstrapRecoveryWork::with_ledger(ledger).map_err(BootstrapRecoveryError::into_cli)?;
    prepare_unpublished_remote_project_with_recovery_work(
        project_root,
        deadline,
        &mut recovery,
        Some(footprint),
    )
}

#[allow(dead_code)]
pub(crate) fn prepare_unpublished_remote_project_with_recovery_work(
    project_root: &Path,
    deadline: &LockDeadline,
    recovery_work: &mut BootstrapRecoveryWork<'_>,
    footprint: Option<BootstrapStageFootprint>,
) -> Result<Option<UnpublishedRemoteProject>, CliError> {
    let (projects_root, org, app) = project_coordinates(project_root)?;
    let projects_parent_path = projects_root.parent().ok_or_else(|| {
        CliError::InvalidRepository("remote projects root has no parent".to_string())
    })?;
    let projects_name = projects_root
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| {
            CliError::InvalidRepository("remote projects root has no UTF-8 name".to_string())
        })?;
    let projects_parent = open_or_create_pinned_absolute(projects_parent_path)?;
    let projects = open_or_create_child(&projects_parent, projects_name, true)?;
    let bootstrap_locks = open_or_create_child(&projects, ".bootstrap-locks", true)?;
    let bootstrap_guards =
        acquire_bootstrap_namespace_locks(&bootstrap_locks, &org, &app, deadline)?;
    let org_root = open_or_create_child(&projects, &org, true)?;
    recover_owned_bootstrap_stages(&org_root, &org, &app, recovery_work)
        .map_err(BootstrapRecoveryError::into_cli)?;
    match org_root.directory.symlink_metadata(&app) {
        Ok(_) => {
            RemoteProject::open(project_root)?;
            Ok(None)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let ledger = recovery_work.ledger.as_deref_mut();
            Ok(Some(create_unpublished_remote_project(
                org_root,
                &org,
                app,
                bootstrap_guards,
                deadline,
                ledger.zip(footprint),
            )?))
        }
        Err(error) => Err(CliError::Io(error)),
    }
}

fn acquire_bootstrap_namespace_locks(
    bootstrap_locks: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
    deadline: &LockDeadline,
) -> Result<Vec<StableFileLock>, CliError> {
    // Missing-target operations use one global namespace order before taking
    // any stable project gate. This serializes opposite bootstrap requests;
    // once a target exists, the ordinary stable-project-ID ordering applies.
    let namespace_name = OsStr::new("bootstrap-namespace.lock");
    let namespace = StableFileLock::acquire_in_directory(
        &bootstrap_locks.directory,
        namespace_name,
        &bootstrap_locks.display_path.join(namespace_name),
        StableLockMode::Exclusive,
        deadline,
    )?;
    let lock_digest = sha256_hex(format!("codefire-remote-project\0{org}\0{app}").as_bytes());
    let lock_name = format!("project-{lock_digest}.lock");
    let target = StableFileLock::acquire_in_directory(
        &bootstrap_locks.directory,
        OsStr::new(&lock_name),
        &bootstrap_locks.display_path.join(&lock_name),
        StableLockMode::Exclusive,
        deadline,
    )?;
    Ok(vec![namespace, target])
}

fn recover_owned_bootstrap_stages(
    org_root: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<(), BootstrapRecoveryError> {
    let result = recover_owned_bootstrap_stages_inner(org_root, org, app, work);
    let accounting = work.finish();
    match (result, accounting) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(primary), Err(_)) => Err(primary),
    }
}

fn recover_owned_bootstrap_stages_inner(
    org_root: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<(), BootstrapRecoveryError> {
    // Quarantines are removed first so a crashed rename cannot permanently
    // block the corresponding live-stage quarantine name.
    for remove_quarantines in [true, false] {
        let mut iterator = org_root.directory.entries()?;
        loop {
            let Some(entry) = next_bootstrap_recovery_entry(&mut iterator, work)? else {
                break;
            };
            work.acquire_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
            let entry_result = (|| -> Result<(), BootstrapRecoveryError> {
                let name = entry.file_name();
                let logical_bytes = bootstrap_entry_frame_bytes(&name).ok_or(
                    BootstrapRecoveryError::ByteLimitExceeded {
                        limit: work.byte_limit,
                    },
                )?;
                work.charge_entry(logical_bytes)?;
                let Some(name_text) = name.to_str() else {
                    return Ok(());
                };
                let is_quarantine =
                    name_text.starts_with(".bootstrap-") && name_text.ends_with(".removing");
                let is_stage = name_text.starts_with(".bootstrap-") && !is_quarantine;
                if (remove_quarantines && !is_quarantine) || (!remove_quarantines && !is_stage) {
                    return Ok(());
                }
                recover_owned_bootstrap_stage(org_root, &name, org, app, is_quarantine, work)
            })();
            work.release_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
            entry_result?;
        }
    }
    Ok(())
}

fn next_bootstrap_recovery_entry(
    iterator: &mut cap_std::fs::ReadDir,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<Option<cap_std::fs::DirEntry>, BootstrapRecoveryError> {
    work.acquire_temporary(BOOTSTRAP_RECOVERY_ITERATOR_SCRATCH_BYTES)?;
    let next = iterator.next().transpose();
    work.release_temporary(BOOTSTRAP_RECOVERY_ITERATOR_SCRATCH_BYTES)?;
    Ok(next?)
}

fn recover_owned_bootstrap_stage(
    org_root: &PinnedRemoteDirectory,
    name: &OsStr,
    org: &str,
    app: &str,
    is_quarantine: bool,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<(), BootstrapRecoveryError> {
    let stage = org_root.directory.open_dir_nofollow(name)?;
    let identity = secure_worktree::StableFsIdentity::for_directory(&stage)?;
    let display_path = org_root.display_path.join(name);
    let ownership = bootstrap_stage_owner(&stage, &display_path, org, app, is_quarantine, work)?;
    let stable_id = match ownership {
        BootstrapStageOwnership::Foreign => return Ok(()),
        BootstrapStageOwnership::EmptyQuarantine => {
            let rebound = org_root.directory.open_dir_nofollow(name)?;
            identity.ensure_directory(&rebound, "empty remote bootstrap quarantine")?;
            org_root.directory.remove_dir(name)?;
            secure_worktree::sync_capability_directory_best_effort(&org_root.directory)?;
            return Ok(());
        }
        BootstrapStageOwnership::Owned(stable_id) => stable_id,
    };
    let rebound = org_root.directory.open_dir_nofollow(name)?;
    identity.ensure_directory(&rebound, "owned remote bootstrap recovery stage")?;

    let removal_name = if is_quarantine {
        name.to_os_string()
    } else {
        OsString::from(format!("{}.removing", name.to_string_lossy()))
    };
    if !is_quarantine {
        secure_worktree::rename_entry_noreplace(&org_root.directory, name, &removal_name).map_err(
            |error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    BootstrapRecoveryError::InvalidOwner {
                        path: org_root.display_path.join(&removal_name),
                        reason: format!("quarantine collision while recovering owner {stable_id}"),
                    }
                } else {
                    BootstrapRecoveryError::Io(error)
                }
            },
        )?;
    }
    let quarantined = org_root.directory.open_dir_nofollow(&removal_name)?;
    identity.ensure_directory(
        &quarantined,
        "quarantined owned remote bootstrap recovery stage",
    )?;
    remove_bootstrap_tree_bounded(&org_root.directory, &removal_name, identity, 0, work)?;
    secure_worktree::sync_capability_directory_best_effort(&org_root.directory)?;
    Ok(())
}

enum BootstrapStageOwnership {
    Owned(String),
    EmptyQuarantine,
    Foreign,
}

fn bootstrap_stage_owner(
    stage: &Dir,
    display_path: &Path,
    org: &str,
    app: &str,
    is_quarantine: bool,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<BootstrapStageOwnership, BootstrapRecoveryError> {
    const DESCRIPTOR_LIMIT: u64 = 16 * 1024;
    // The bounded byte buffer and serde's parsed `Value` coexist during
    // ownership validation, so reserve the full 2P physical peak before the
    // read allocates either representation.
    const DESCRIPTOR_PHYSICAL_PEAK: u64 = DESCRIPTOR_LIMIT * 2;
    work.acquire_temporary(DESCRIPTOR_PHYSICAL_PEAK)?;
    let read = secure_worktree::read_file_in_directory_nofollow_bounded(
        stage,
        OsStr::new(PROJECT_DESCRIPTOR_FILE),
        &display_path.join(PROJECT_DESCRIPTOR_FILE),
        DESCRIPTOR_LIMIT,
    );
    let bytes = match read {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && is_quarantine => {
            work.release_temporary(DESCRIPTOR_PHYSICAL_PEAK)?;
            let mut entries = stage.entries()?;
            let next = next_bootstrap_recovery_entry(&mut entries, work)?;
            return if next.is_none() {
                Ok(BootstrapStageOwnership::EmptyQuarantine)
            } else {
                Err(BootstrapRecoveryError::InvalidOwner {
                    path: display_path.to_path_buf(),
                    reason: "descriptorless bootstrap quarantine is not empty".to_string(),
                })
            };
        }
        Err(error) => {
            work.release_temporary(DESCRIPTOR_PHYSICAL_PEAK)?;
            return Err(BootstrapRecoveryError::InvalidOwner {
                path: display_path.to_path_buf(),
                reason: error.to_string(),
            });
        }
    };
    let owner = (|| -> Result<BootstrapStageOwnership, BootstrapRecoveryError> {
        let byte_count =
            u64::try_from(bytes.len()).map_err(|_| BootstrapRecoveryError::ByteLimitExceeded {
                limit: work.byte_limit,
            })?;
        work.charge_descriptor_read(byte_count)?;
        let descriptor: Value = serde_json::from_slice(&bytes)?;
        let object =
            descriptor
                .as_object()
                .ok_or_else(|| BootstrapRecoveryError::InvalidOwner {
                    path: display_path.to_path_buf(),
                    reason: "owner descriptor is not an object".to_string(),
                })?;
        let descriptor_org = object.get("organization").and_then(Value::as_str);
        let descriptor_app = object.get("application").and_then(Value::as_str);
        if descriptor_org != Some(org) || descriptor_app != Some(app) {
            return Ok(BootstrapStageOwnership::Foreign);
        }
        let stable_id = object
            .get("stable_project_id")
            .and_then(Value::as_str)
            .filter(|value| value.starts_with("CF-PROJECT-") && value.len() == 43)
            .ok_or_else(|| BootstrapRecoveryError::InvalidOwner {
                path: display_path.to_path_buf(),
                reason: "owner descriptor has no valid stable project identity".to_string(),
            })?;
        let valid = object.get("type").and_then(Value::as_str) == Some("codefire_remote_project")
            && object.get("version").and_then(Value::as_u64) == Some(1)
            && object.get("protocol_generation").and_then(Value::as_u64)
                == Some(PROJECT_PROTOCOL_GENERATION);
        if !valid {
            return Err(BootstrapRecoveryError::InvalidOwner {
                path: display_path.to_path_buf(),
                reason: "owner descriptor protocol fields are invalid".to_string(),
            });
        }
        Ok(BootstrapStageOwnership::Owned(stable_id.to_string()))
    })();
    drop(bytes);
    work.release_temporary(DESCRIPTOR_PHYSICAL_PEAK)?;
    owner
}

fn remove_bootstrap_tree_bounded(
    parent: &Dir,
    name: &OsStr,
    expected: secure_worktree::StableFsIdentity,
    depth: u64,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<(), BootstrapRecoveryError> {
    if depth > 64 {
        return Err(BootstrapRecoveryError::InvalidOwner {
            path: PathBuf::from(name),
            reason: "bootstrap recovery tree exceeds depth 64".to_string(),
        });
    }
    let directory = parent.open_dir_nofollow(name)?;
    expected.ensure_directory(&directory, "remote bootstrap bounded removal root")?;
    let mut iterator = directory.entries()?;
    let mut descriptor_present = false;
    loop {
        let Some(entry) = next_bootstrap_recovery_entry(&mut iterator, work)? else {
            break;
        };
        work.acquire_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
        let entry_result = (|| -> Result<(), BootstrapRecoveryError> {
            let child_name = entry.file_name();
            work.charge_tree_entry_frame(&child_name)?;
            let metadata = directory.symlink_metadata(&child_name)?;
            work.charge_tree_entry_payload(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                metadata.len(),
            )?;
            if depth == 0 && child_name == OsStr::new(PROJECT_DESCRIPTOR_FILE) {
                descriptor_present = true;
                return Ok(());
            }
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                let child = directory.open_dir_nofollow(&child_name)?;
                let child_identity = secure_worktree::StableFsIdentity::for_directory(&child)?;
                remove_bootstrap_tree_bounded(
                    &directory,
                    &child_name,
                    child_identity,
                    depth + 1,
                    work,
                )?;
            } else {
                directory.remove_file(&child_name)?;
            }
            Ok(())
        })();
        work.release_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
        entry_result?;
    }
    if depth == 0 && descriptor_present {
        // The ownership descriptor is the recovery proof. Keep it until every
        // other child has been removed successfully; only an immediately
        // subsequent root rmdir may observe the descriptorless empty state.
        directory.remove_file(PROJECT_DESCRIPTOR_FILE)?;
    }
    let rebound = parent.open_dir_nofollow(name)?;
    expected.ensure_directory(&rebound, "remote bootstrap bounded removal rebound")?;
    parent.remove_dir(name)?;
    Ok(())
}

fn scan_bootstrap_tree_bounded(
    parent: &Dir,
    name: &OsStr,
    expected: secure_worktree::StableFsIdentity,
    depth: u64,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<(), BootstrapRecoveryError> {
    if depth > 64 {
        return Err(BootstrapRecoveryError::InvalidOwner {
            path: PathBuf::from(name),
            reason: "bootstrap recovery tree exceeds depth 64".to_string(),
        });
    }
    let directory = parent.open_dir_nofollow(name)?;
    expected.ensure_directory(&directory, "remote bootstrap bounded scan root")?;
    let mut iterator = directory.entries()?;
    loop {
        let Some(entry) = next_bootstrap_recovery_entry(&mut iterator, work)? else {
            break;
        };
        work.acquire_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
        let entry_result = (|| -> Result<(), BootstrapRecoveryError> {
            let child_name = entry.file_name();
            work.charge_tree_entry_frame(&child_name)?;
            let metadata = directory.symlink_metadata(&child_name)?;
            work.charge_tree_entry_payload(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                metadata.len(),
            )?;
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                let child = directory.open_dir_nofollow(&child_name)?;
                let child_identity = secure_worktree::StableFsIdentity::for_directory(&child)?;
                scan_bootstrap_tree_bounded(
                    &directory,
                    &child_name,
                    child_identity,
                    depth + 1,
                    work,
                )?;
            }
            Ok(())
        })();
        work.release_temporary(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)?;
        entry_result?;
    }
    let rebound = parent.open_dir_nofollow(name)?;
    expected.ensure_directory(&rebound, "remote bootstrap bounded scan rebound")?;
    Ok(())
}

#[cfg(test)]
fn remove_one_bootstrap_child_for_test(
    parent: &Dir,
    name: &OsStr,
    expected: secure_worktree::StableFsIdentity,
    work: &mut BootstrapRecoveryWork<'_>,
) -> Result<bool, BootstrapRecoveryError> {
    let directory = parent.open_dir_nofollow(name)?;
    expected.ensure_directory(&directory, "injected bootstrap partial cleanup")?;
    let mut iterator = directory.entries()?;
    loop {
        let Some(entry) = next_bootstrap_recovery_entry(&mut iterator, work)? else {
            return Ok(false);
        };
        let child_name = entry.file_name();
        work.charge_tree_entry_frame(&child_name)?;
        let metadata = directory.symlink_metadata(&child_name)?;
        work.charge_tree_entry_payload(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            metadata.len(),
        )?;
        if child_name != OsStr::new(PROJECT_DESCRIPTOR_FILE)
            && (!metadata.is_dir() || metadata.file_type().is_symlink())
        {
            directory.remove_file(&child_name)?;
            return Ok(true);
        }
    }
}

fn create_unpublished_remote_project(
    org_root: PinnedRemoteDirectory,
    org: &str,
    app: String,
    bootstrap_locks: Vec<StableFileLock>,
    deadline: &LockDeadline,
    mut ledger: Option<(
        &mut codefire_core::DecisionWorkLedger,
        BootstrapStageFootprint,
    )>,
) -> Result<UnpublishedRemoteProject, CliError> {
    let stage_name_text = format!(".bootstrap-{}", random_project_id()?);
    let stage_name = OsString::from(&stage_name_text);
    let cleanup_parent = org_root.directory.try_clone()?;
    let mut decision_work = match ledger.as_mut() {
        Some((ledger, footprint)) => Some(BootstrapDecisionWorkOwner::reserve(ledger, *footprint)?),
        None => None,
    };
    if let Err(error) = org_root.directory.create_dir(&stage_name) {
        if let (Some(work), Some((ledger, _))) = (decision_work.as_mut(), ledger.as_mut()) {
            if let Err(accounting) = work.cancel_without_effect(ledger) {
                return Err(bootstrap_combined_error(
                    CliError::Io(error),
                    accounting,
                    "remote bootstrap stage creation failed",
                ));
            }
        }
        return Err(CliError::Io(error));
    }
    let stage = match open_pinned_child(&org_root, &stage_name_text) {
        Ok(stage) => stage,
        Err(error) => {
            let mut rollback_permit = None;
            let mut settlement_error = None;
            if let (Some(work), Some((ledger, _))) = (decision_work.as_mut(), ledger.as_mut()) {
                if let Err(accounting) = work.record_creation(
                    ledger,
                    bootstrap_creation_actual_reservation(
                        1,
                        BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES,
                        0,
                    ),
                ) {
                    settlement_error = Some(accounting);
                }
                match work.begin_rollback_attempt(ledger) {
                    Ok(permit) => rollback_permit = Some(permit),
                    Err(accounting) => {
                        settlement_error.get_or_insert(accounting);
                    }
                };
            }
            let cleanup_result = if let Err(cleanup_error) =
                org_root.directory.remove_dir(&stage_name)
            {
                let quarantine = OsString::from(format!("{stage_name_text}.removing"));
                if let Err(rename_error) = secure_worktree::rename_entry_noreplace(
                    &org_root.directory,
                    &stage_name,
                    &quarantine,
                ) {
                    Err(CliError::InvalidRepository(format!(
                        "remote bootstrap stage open failed ({error}); empty-stage cleanup failed ({cleanup_error}); quarantine failed: {rename_error}"
                    )))
                } else {
                    let _ =
                        secure_worktree::sync_capability_directory_best_effort(&org_root.directory);
                    Ok(())
                }
            } else {
                Ok(())
            };
            if let (Some(work), Some((ledger, _))) = (decision_work.as_mut(), ledger.as_mut()) {
                if let Some(permit) = rollback_permit {
                    if let Err(accounting) = work.finish_rollback_attempt(
                        ledger,
                        permit,
                        bootstrap_observed_reservation(
                            1,
                            BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES,
                            0,
                            0,
                            0,
                        ),
                        0,
                    ) {
                        settlement_error.get_or_insert(accounting);
                    }
                }
                if let Err(accounting) = work.cancel_visibility(ledger) {
                    settlement_error.get_or_insert(accounting);
                }
                if let Err(accounting) = work.cancel_without_effect(ledger) {
                    settlement_error.get_or_insert(accounting);
                }
            }
            if let Err(cleanup) = cleanup_result {
                return Err(bootstrap_combined_error(
                    error,
                    cleanup,
                    "remote bootstrap stage open failed",
                ));
            }
            return match settlement_error {
                Some(accounting) => Err(bootstrap_combined_error(
                    error,
                    accounting,
                    "remote bootstrap stage open failed",
                )),
                None => Err(error),
            };
        }
    };
    let mut cleanup = BootstrapStageCleanup {
        parent: cleanup_parent,
        stage_name: stage_name.clone(),
        stage_identity: stage.identity,
        published: false,
        drop_quarantine: true,
        #[cfg(test)]
        fail_bounded_cleanup_once: take_test_injection(&INJECT_BOOTSTRAP_BUILD_CLEANUP_FAILURE),
    };
    let built = (|| {
        write_project_descriptor(&stage, org, &app, "authenticated_bootstrap")?;
        create_base_layout(&stage)?;
        #[cfg(test)]
        if take_test_injection(&INJECT_BOOTSTRAP_BUILD_EXTRA_ENTRY) {
            stage.directory.write("injected-extra", b"x")?;
        }
        #[cfg(test)]
        if take_test_injection(&INJECT_BOOTSTRAP_BUILD_FAILURE) {
            return Err(CliError::InvalidRepository(
                "injected remote bootstrap build failure".to_string(),
            ));
        }
        sync_layout_best_effort(&stage)?;
        let descriptor_bytes = stage
            .directory
            .symlink_metadata(PROJECT_DESCRIPTOR_FILE)?
            .len();
        let project = RemoteProject::from_pinned_root(stage, org, &app)?;
        project.ensure_bound()?;
        let gate_lock = project.acquire_hidden_gate_lock(deadline)?;
        project.ensure_gate_lock_subject(&gate_lock)?;
        Ok::<_, CliError>((project, gate_lock, descriptor_bytes))
    })();
    let (project, gate_lock, descriptor_bytes) = match built {
        Ok(built) => built,
        Err(error) => {
            let limits = decision_work
                .as_ref()
                .map(BootstrapDecisionWorkOwner::cleanup_limits)
                .transpose()
                .expect("validated bootstrap footprint has bounded cleanup limits")
                .unwrap_or((
                    BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
                    BOOTSTRAP_RECOVERY_BYTE_LIMIT,
                ));
            if let (Some(work), Some((ledger, _))) = (decision_work.as_mut(), ledger.as_mut()) {
                if let Err(settlement) =
                    settle_failed_bootstrap_stage(&mut cleanup, work, ledger, limits.0, limits.1)
                {
                    return Err(bootstrap_combined_error(
                        error,
                        settlement,
                        "remote bootstrap build failed",
                    ));
                }
            } else if let Err(cleanup_error) = cleanup.remove_bounded(limits.0, limits.1) {
                return Err(bootstrap_combined_error(
                    error,
                    cleanup_error,
                    "remote bootstrap build failed",
                ));
            }
            return Err(error);
        }
    };
    if let (Some(work), Some((ledger, _))) = (decision_work.as_mut(), ledger.as_mut()) {
        let aggregate_bytes = BOOTSTRAP_FIXED_LAYOUT_ENTRIES
            .saturating_mul(BOOTSTRAP_RECOVERY_ENTRY_FRAME_BYTES)
            .saturating_add(descriptor_bytes);
        if let Err(error) = work.record_creation(
            ledger,
            bootstrap_creation_actual_reservation(
                BOOTSTRAP_FIXED_LAYOUT_ENTRIES,
                aggregate_bytes,
                1,
            ),
        ) {
            let limits = work
                .cleanup_limits()
                .expect("validated bootstrap footprint has bounded cleanup limits");
            let settlement =
                settle_failed_bootstrap_stage(&mut cleanup, work, ledger, limits.0, limits.1);
            return match settlement {
                Ok(()) => Err(error),
                Err(settlement) => Err(bootstrap_combined_error(
                    error,
                    settlement,
                    "remote bootstrap creation accounting failed",
                )),
            };
        }
    }
    let owner = BootstrapPublicationOwner {
        cleanup,
        gate_lock,
        bootstrap_locks,
        decision_work,
    };
    Ok(UnpublishedRemoteProject {
        project,
        org_root,
        app,
        stage_name,
        owner,
    })
}

struct BootstrapStageCleanup {
    parent: Dir,
    stage_name: OsString,
    stage_identity: secure_worktree::StableFsIdentity,
    published: bool,
    drop_quarantine: bool,
    #[cfg(test)]
    fail_bounded_cleanup_once: bool,
}

struct BootstrapCleanupAttempt {
    observation: BootstrapCleanupObservation,
    result: Result<(), CliError>,
}

impl BootstrapStageCleanup {
    fn quarantine(&mut self) -> Result<OsString, CliError> {
        if self.published {
            return Err(CliError::InvalidRepository(
                "published remote bootstrap cannot be quarantined".to_string(),
            ));
        }
        let live = self.parent.open_dir_nofollow(&self.stage_name);
        match live {
            Ok(stage) => {
                self.stage_identity
                    .ensure_directory(&stage, "remote bootstrap cleanup stage")?;
                if self.stage_name.to_string_lossy().ends_with(".removing") {
                    return Ok(self.stage_name.clone());
                }
                let quarantine_name =
                    OsString::from(format!("{}.removing", self.stage_name.to_string_lossy()));
                secure_worktree::rename_entry_noreplace(
                    &self.parent,
                    &self.stage_name,
                    &quarantine_name,
                )?;
                let quarantined = self.parent.open_dir_nofollow(&quarantine_name)?;
                self.stage_identity
                    .ensure_directory(&quarantined, "quarantined remote bootstrap cleanup stage")?;
                self.stage_name = quarantine_name.clone();
                Ok(quarantine_name)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let quarantine_name = if self.stage_name.to_string_lossy().ends_with(".removing") {
                    self.stage_name.clone()
                } else {
                    OsString::from(format!("{}.removing", self.stage_name.to_string_lossy()))
                };
                let quarantined = self.parent.open_dir_nofollow(&quarantine_name)?;
                self.stage_identity.ensure_directory(
                    &quarantined,
                    "existing quarantined remote bootstrap cleanup stage",
                )?;
                self.stage_name = quarantine_name.clone();
                Ok(quarantine_name)
            }
            Err(error) => Err(CliError::Io(error)),
        }
    }

    fn remove_bounded(&mut self, entry_limit: u64, byte_limit: u64) -> Result<(), CliError> {
        self.remove_bounded_attempt(entry_limit, byte_limit).result
    }

    fn remove_bounded_attempt(
        &mut self,
        entry_limit: u64,
        byte_limit: u64,
    ) -> BootstrapCleanupAttempt {
        let quarantine_name = match self.quarantine() {
            Ok(name) => name,
            Err(error) => {
                return BootstrapCleanupAttempt {
                    observation: BootstrapCleanupObservation::default(),
                    result: Err(error),
                };
            }
        };
        let mut work = BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit,
            byte_limit,
        };
        #[cfg(test)]
        if std::mem::take(&mut self.fail_bounded_cleanup_once) {
            let result = remove_one_bootstrap_child_for_test(
                &self.parent,
                &quarantine_name,
                self.stage_identity,
                &mut work,
            )
            .and_then(|removed| {
                if removed {
                    Err(BootstrapRecoveryError::InvalidOwner {
                        path: PathBuf::from(&quarantine_name),
                        reason:
                            "injected remote bootstrap bounded cleanup failure after child deletion"
                                .to_string(),
                    })
                } else {
                    Err(BootstrapRecoveryError::InvalidOwner {
                        path: PathBuf::from(&quarantine_name),
                        reason: "injected remote bootstrap bounded cleanup failure".to_string(),
                    })
                }
            });
            return BootstrapCleanupAttempt {
                observation: work.observation(),
                result: result.map_err(BootstrapRecoveryError::into_cli),
            };
        }
        let removal = remove_bootstrap_tree_bounded(
            &self.parent,
            &quarantine_name,
            self.stage_identity,
            0,
            &mut work,
        )
        .map_err(BootstrapRecoveryError::into_cli);
        let observation = work.observation();
        let result = match removal {
            Ok(()) => {
                self.published = true;
                secure_worktree::sync_capability_directory_best_effort(&self.parent)
                    .map_err(CliError::Io)
            }
            Err(error) => Err(error),
        };
        BootstrapCleanupAttempt {
            observation,
            result,
        }
    }

    fn scan_bounded_attempt(
        &mut self,
        entry_limit: u64,
        byte_limit: u64,
    ) -> BootstrapCleanupAttempt {
        let quarantine_name = match self.quarantine() {
            Ok(name) => name,
            Err(error) => {
                return BootstrapCleanupAttempt {
                    observation: BootstrapCleanupObservation::default(),
                    result: Err(error),
                };
            }
        };
        let mut work = BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit,
            byte_limit,
        };
        let result = scan_bootstrap_tree_bounded(
            &self.parent,
            &quarantine_name,
            self.stage_identity,
            0,
            &mut work,
        )
        .map_err(BootstrapRecoveryError::into_cli);
        BootstrapCleanupAttempt {
            observation: work.observation(),
            result,
        }
    }
}

fn settle_failed_bootstrap_stage(
    cleanup: &mut BootstrapStageCleanup,
    work: &mut BootstrapDecisionWorkOwner,
    ledger: &mut codefire_core::DecisionWorkLedger,
    entry_limit: u64,
    byte_limit: u64,
) -> Result<(), CliError> {
    let permit = match work.begin_rollback_attempt(ledger) {
        Ok(permit) => permit,
        Err(error) => {
            let close = work.cancel_without_effect(ledger);
            return match close {
                Ok(()) => Err(error),
                Err(close) => Err(bootstrap_combined_error(
                    error,
                    close,
                    "remote bootstrap failure settlement",
                )),
            };
        }
    };
    let scan = cleanup.scan_bounded_attempt(entry_limit, byte_limit);
    let removal = if scan.result.is_ok() {
        cleanup.remove_bounded_attempt(entry_limit, byte_limit)
    } else {
        BootstrapCleanupAttempt {
            observation: BootstrapCleanupObservation::default(),
            result: Ok(()),
        }
    };
    let combined = scan.observation.combine(removal.observation);
    let creation = work.record_creation(ledger, scan.observation.creation_reservation());
    let rollback = work.finish_rollback_attempt(
        ledger,
        permit,
        combined.reservation(),
        combined.temporary_peak,
    );
    let visibility = work.cancel_visibility(ledger);
    let close = work.cancel_without_effect(ledger);
    creation?;
    rollback?;
    visibility?;
    close?;
    scan.result?;
    removal.result
}

impl Drop for BootstrapStageCleanup {
    fn drop(&mut self) {
        if self.published {
            return;
        }
        if !self.drop_quarantine {
            return;
        }
        // Drop is deliberately quarantine-only. Recursive deletion belongs to
        // explicit ledger-aware cancellation or the next bounded recovery
        // pass. Identity fencing prevents an ambient replacement from being
        // moved or removed by this fail-safe.
        let _ = self.quarantine();
    }
}

pub(crate) fn remote_project_stable_id(project_root: &Path) -> Result<String, CliError> {
    Ok(RemoteProject::open(project_root)?.stable_id().to_string())
}

fn project_coordinates(project_root: &Path) -> Result<(PathBuf, String, String), CliError> {
    let app = project_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| CliError::InvalidRepository("remote project has no UTF-8 app".to_string()))?
        .to_string();
    let org_root = project_root.parent().ok_or_else(|| {
        CliError::InvalidRepository("remote project has no organization parent".to_string())
    })?;
    let org = org_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| CliError::InvalidRepository("remote project has no UTF-8 org".to_string()))?
        .to_string();
    let projects_root = org_root.parent().ok_or_else(|| {
        CliError::InvalidRepository("remote project has no projects parent".to_string())
    })?;
    validate_remote_component(&org, "organization")?;
    validate_remote_component(&app, "application")?;
    Ok((projects_root.to_path_buf(), org, app))
}

fn initialize_or_validate_existing(
    project_root: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
) -> Result<(), CliError> {
    match project_root
        .directory
        .symlink_metadata(PROJECT_DESCRIPTOR_FILE)
    {
        Ok(_) => validate_project_descriptor(project_root, org, app)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if contains_legacy_authority(project_root)? {
                return Err(CliError::InvalidRepository(format!(
                    "legacy remote project requires administrative quiescence and explicit migration before mutation: {}",
                    project_root.display_path.display()
                )));
            }
            write_project_descriptor(project_root, org, app, "configured_empty_root")?;
        }
        Err(error) => return Err(CliError::Io(error)),
    }
    create_base_layout(project_root)?;
    sync_layout_best_effort(project_root)?;
    validate_project_descriptor(project_root, org, app)
}

fn contains_legacy_authority(project_root: &PinnedRemoteDirectory) -> Result<bool, CliError> {
    for name in [
        "gc_state.json",
        "request_nonce_cache.json",
        "pending_transaction.json",
    ] {
        match project_root.directory.symlink_metadata(name) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    if directory_tree_contains_authority(&project_root.directory, "objects")? {
        return Ok(true);
    }
    for name in [
        "branches",
        "merge_requests",
        "idempotency",
        "transactions",
        "gc",
    ] {
        match project_root.directory.open_dir_nofollow(name) {
            Ok(directory) if directory.entries()?.next().transpose()?.is_some() => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Ok(true),
        }
    }
    Ok(false)
}

fn directory_tree_contains_authority(parent: &Dir, name: &str) -> Result<bool, CliError> {
    let directory = match parent.open_dir_nofollow(name) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Ok(true),
    };
    for entry in directory.entries()? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() || file_type.is_symlink() {
            return Ok(true);
        }
        let Some(child) = entry.file_name().to_str().map(str::to_string) else {
            return Ok(true);
        };
        if directory_tree_contains_authority(&directory, &child)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn create_base_layout(project_root: &PinnedRemoteDirectory) -> Result<(), CliError> {
    let mut objects = None;
    for name in [
        "objects",
        "branches",
        "merge_requests",
        "locks",
        "idempotency",
        "audit",
        "transactions",
        "gc",
    ] {
        let child = open_or_create_child(project_root, name, true)?;
        if name == "objects" {
            objects = Some(child);
        }
    }
    let objects = objects.ok_or_else(|| {
        CliError::InvalidRepository("remote objects directory was not created".to_string())
    })?;
    for subdir in OBJECT_SUBDIRS {
        open_or_create_child(&objects, subdir, true)?;
    }
    Ok(())
}

fn write_project_descriptor(
    project_root: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
    provenance: &str,
) -> Result<(), CliError> {
    let path = project_root.display_path.join(PROJECT_DESCRIPTOR_FILE);
    let descriptor = json!({
        "type": "codefire_remote_project",
        "version": 1,
        "stable_project_id": random_project_id()?,
        "protocol_generation": PROJECT_PROTOCOL_GENERATION,
        "organization": org,
        "application": app,
        "creation_provenance": provenance,
        "created_at": now_iso_utc(),
    });
    let bytes = codefire_store::canonical_json(&descriptor)?;
    let temp_path = unique_temp_path(&path);
    let temp_name = temp_path.file_name().ok_or_else(|| {
        CliError::InvalidRepository("remote descriptor temp name is missing".to_string())
    })?;
    match secure_worktree::write_file_create_noreplace_in_directory_nofollow(
        &project_root.directory,
        OsStr::new(PROJECT_DESCRIPTOR_FILE),
        temp_name,
        &bytes,
    ) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_project_descriptor(project_root, org, app)
        }
        Err(error) => Err(CliError::Io(error)),
    }
}

fn validate_project_descriptor(
    project_root: &PinnedRemoteDirectory,
    org: &str,
    app: &str,
) -> Result<(), CliError> {
    let path = project_root.display_path.join(PROJECT_DESCRIPTOR_FILE);
    let bytes = secure_worktree::read_file_in_directory_nofollow_bounded(
        &project_root.directory,
        OsStr::new(PROJECT_DESCRIPTOR_FILE),
        &path,
        16 * 1024,
    )?;
    let descriptor: Value = serde_json::from_slice(&bytes)?;
    validate_descriptor_value(&descriptor, &project_root.display_path, org, app)
}

fn validate_descriptor_value(
    descriptor: &Value,
    project_root: &Path,
    org: &str,
    app: &str,
) -> Result<(), CliError> {
    let valid = descriptor.get("type").and_then(Value::as_str) == Some("codefire_remote_project")
        && descriptor.get("version").and_then(Value::as_u64) == Some(1)
        && descriptor
            .get("protocol_generation")
            .and_then(Value::as_u64)
            == Some(PROJECT_PROTOCOL_GENERATION)
        && descriptor.get("organization").and_then(Value::as_str) == Some(org)
        && descriptor.get("application").and_then(Value::as_str) == Some(app)
        && descriptor
            .get("stable_project_id")
            .and_then(Value::as_str)
            .is_some_and(|value| value.starts_with("CF-PROJECT-") && value.len() == 43);
    if !valid {
        return Err(CliError::InvalidRepository(format!(
            "remote project descriptor is invalid: {}",
            project_root.join(PROJECT_DESCRIPTOR_FILE).display()
        )));
    }
    Ok(())
}

fn random_project_id() -> Result<String, CliError> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate remote project identity: {error}"
        ))
    })?;
    let mut encoded = String::with_capacity(32);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    Ok(format!("CF-PROJECT-{encoded}"))
}

fn sync_layout_best_effort(project_root: &PinnedRemoteDirectory) -> Result<(), CliError> {
    for name in [
        "objects",
        "branches",
        "merge_requests",
        "locks",
        "idempotency",
        "audit",
        "transactions",
        "gc",
    ] {
        let child = open_pinned_child(project_root, name)?;
        secure_worktree::sync_capability_directory_best_effort(&child.directory)?;
    }
    secure_worktree::sync_capability_directory_best_effort(&project_root.directory)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn assert_ledger_has_no_live_owners(ledger: &codefire_core::DecisionWorkLedger) {
        assert_eq!(ledger.open_permit_count(), 0);
        assert!(ledger.snapshot().current.values().all(|value| *value == 0));
    }

    fn recovery_work(entry_limit: u64, byte_limit: u64) -> BootstrapRecoveryWork<'static> {
        BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit,
            byte_limit,
        }
    }

    fn logical_bootstrap_tree_cost(root: &Path) -> (u64, u64) {
        fn visit(path: &Path, entries: &mut u64, bytes: &mut u64) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name();
                let meta = fs::symlink_metadata(entry.path()).unwrap();
                *entries += 1;
                *bytes += codefire_core::LOGICAL_FRAME_BASE_BYTES
                    + u64::try_from(name.as_encoded_bytes().len()).unwrap();
                if meta.is_dir() && !meta.file_type().is_symlink() {
                    visit(&entry.path(), entries, bytes);
                } else {
                    *bytes += meta.len();
                }
            }
        }
        let mut entries = 0;
        let mut bytes = 0;
        visit(root, &mut entries, &mut bytes);
        (entries, bytes)
    }

    #[test]
    fn strict_component_contract_accepts_single_portable_names() {
        for value in ["org", "app-name", "feature.alpha", "日本語", ".hidden"] {
            validate_remote_component(value, "test").unwrap();
        }
    }

    #[test]
    fn strict_component_contract_rejects_authority_escape_shapes() {
        for value in [
            "",
            ".",
            "..",
            "/absolute",
            "\\absolute",
            "a/b",
            "a\\b",
            "C:prefix",
            "line\nbreak",
        ] {
            let error = validate_remote_component(value, "test").unwrap_err();
            assert!(matches!(error, CliError::Usage(_)), "accepted {value:?}");
        }
        let oversized = "a".repeat(REMOTE_COMPONENT_BYTE_LIMIT + 1);
        validate_remote_component(&oversized, "test").unwrap_err();
    }

    #[test]
    fn strict_component_contract_matches_cross_runtime_fixture() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/fixtures/remote-components-v1.json");
        let fixture: Value = serde_json::from_slice(&fs::read(fixture_path).unwrap()).unwrap();
        let expand = |case: &Value| {
            case.get("value")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| {
                    case.get("repeat")
                        .and_then(Value::as_str)
                        .unwrap()
                        .repeat(case.get("count").and_then(Value::as_u64).unwrap() as usize)
                })
        };
        for case in fixture["valid"].as_array().unwrap() {
            let value = expand(case);
            validate_remote_component(&value, "fixture").unwrap();
        }
        for case in fixture["invalid"].as_array().unwrap() {
            let value = expand(case);
            assert!(validate_remote_component(&value, "fixture").is_err());
        }
    }

    #[test]
    fn file_url_rejects_decoded_escape_before_layout_creation() {
        let server = std::env::temp_dir().join(format!(
            "codefire-remote-component-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let encoded_separator = format!("cf://{}/org/app/feature%2Fescape", server.display());
        let encoded_parent = format!("cf://{}/%2E%2E/app/main", server.display());
        assert!(crate::remote::parse_cf_url(&encoded_separator).is_err());
        assert!(crate::remote::parse_cf_url(&encoded_parent).is_err());
        assert!(!server.join(".codefire-server").exists());
    }

    #[test]
    fn bootstrap_publishes_one_stable_descriptor_and_persistent_lock() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        ensure_remote_project_layout(&project).unwrap();
        let first: Value = serde_json::from_slice(
            &secure_worktree::read_file_nofollow_bounded(
                &project.join(PROJECT_DESCRIPTOR_FILE),
                16 * 1024,
            )
            .unwrap(),
        )
        .unwrap();
        ensure_remote_project_layout(&project).unwrap();
        let second: Value = serde_json::from_slice(
            &secure_worktree::read_file_nofollow_bounded(
                &project.join(PROJECT_DESCRIPTOR_FILE),
                16 * 1024,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(first["stable_project_id"], second["stable_project_id"]);
        let locks = root.join("projects/.bootstrap-locks");
        assert!(fs::read_dir(locks)
            .unwrap()
            .filter_map(Result::ok)
            .any(
                |entry| entry.path().extension().and_then(|value| value.to_str()) == Some("lock")
            ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_accepts_maximum_length_application_component() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-boundary-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org").join("a".repeat(255));
        ensure_remote_project_layout(&project).unwrap();
        assert!(project.join(PROJECT_DESCRIPTOR_FILE).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    fn create_crashed_bootstrap_stage(org_path: &Path, stage_name: &str, owner_app: &str) {
        fs::create_dir_all(org_path).unwrap();
        let org_root = open_pinned_absolute(org_path).unwrap();
        org_root.directory.create_dir(stage_name).unwrap();
        let stage = open_pinned_child(&org_root, stage_name).unwrap();
        write_project_descriptor(&stage, "org", owner_app, "crash-recovery-test").unwrap();
        create_base_layout(&stage).unwrap();
    }

    #[test]
    fn bootstrap_recovers_owned_crashed_stage_before_republication() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-recovery-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let project = org_path.join("app");
        let stale = ".bootstrap-CF-PROJECT-stale";
        let quarantined = ".bootstrap-CF-PROJECT-quarantined.removing";
        create_crashed_bootstrap_stage(&org_path, stale, "app");
        create_crashed_bootstrap_stage(&org_path, quarantined, "app");

        ensure_remote_project_layout(&project).unwrap();

        assert!(project.join(PROJECT_DESCRIPTOR_FILE).is_file());
        assert!(!org_path.join(stale).exists());
        assert!(!org_path.join(format!("{stale}.removing")).exists());
        assert!(!org_path.join(quarantined).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_recovery_preserves_stage_owned_by_another_application() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-foreign-recovery-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let project = org_path.join("app");
        let foreign = ".bootstrap-CF-PROJECT-foreign";
        create_crashed_bootstrap_stage(&org_path, foreign, "other-app");

        ensure_remote_project_layout(&project).unwrap();

        assert!(project.is_dir());
        assert!(org_path.join(foreign).is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_removal_keeps_descriptor_until_retry_can_finish() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-descriptor-last-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let name = ".bootstrap-CF-PROJECT-descriptor-last.removing";
        create_crashed_bootstrap_stage(&org_path, name, "app");
        fs::write(org_path.join(name).join("child"), b"payload").unwrap();
        let org_root = open_pinned_absolute(&org_path).unwrap();
        let stage = org_root.directory.open_dir_nofollow(name).unwrap();
        let identity = secure_worktree::StableFsIdentity::for_directory(&stage).unwrap();
        let mut limited = BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit: 1,
            byte_limit: BOOTSTRAP_RECOVERY_BYTE_LIMIT,
        };

        assert!(remove_bootstrap_tree_bounded(
            &org_root.directory,
            OsStr::new(name),
            identity,
            0,
            &mut limited,
        )
        .is_err());
        assert!(org_path.join(name).join(PROJECT_DESCRIPTOR_FILE).is_file());

        let mut retry = BootstrapRecoveryWork::bounded();
        remove_bootstrap_tree_bounded(
            &org_root.directory,
            OsStr::new(name),
            identity,
            0,
            &mut retry,
        )
        .unwrap();
        assert!(!org_path.join(name).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_recovery_removes_only_empty_descriptorless_quarantine() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-empty-quarantine-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let empty = org_path.join(".bootstrap-empty.removing");
        fs::create_dir_all(&empty).unwrap();
        let project = org_path.join("app");
        ensure_remote_project_layout(&project).unwrap();
        assert!(!empty.exists());

        fs::remove_dir_all(&project).unwrap();
        let foreign = org_path.join(".bootstrap-foreign.removing");
        fs::create_dir_all(&foreign).unwrap();
        fs::write(foreign.join("foreign"), b"preserve").unwrap();
        assert!(ensure_remote_project_layout(&project).is_err());
        assert_eq!(fs::read(foreign.join("foreign")).unwrap(), b"preserve");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_recovery_reports_typed_entry_bound() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-recovery-bound-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        fs::create_dir_all(org_path.join("one")).unwrap();
        fs::create_dir_all(org_path.join("two")).unwrap();
        let org_root = open_pinned_absolute(&org_path).unwrap();
        let mut work = BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit: 1,
            byte_limit: BOOTSTRAP_RECOVERY_BYTE_LIMIT,
        };

        let error = recover_owned_bootstrap_stages(&org_root, "org", "app", &mut work).unwrap_err();

        assert!(matches!(
            error,
            BootstrapRecoveryError::EntryLimitExceeded { limit: 1 }
        ));
        let mut byte_work = BootstrapRecoveryWork {
            ledger: None,
            permit: None,
            entries: 0,
            bytes: 0,
            descriptor_reads: 0,
            file_bytes_read: 0,
            temporary_current: 0,
            temporary_peak: 0,
            ledger_temporary_owned: 0,
            entry_limit: BOOTSTRAP_RECOVERY_ENTRY_LIMIT,
            byte_limit: 1,
        };
        let error =
            recover_owned_bootstrap_stages(&org_root, "org", "app", &mut byte_work).unwrap_err();
        assert!(matches!(
            error,
            BootstrapRecoveryError::ByteLimitExceeded { limit: 1 }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_cleanup_pass_limits_are_the_reproduced_single_pass_budgets() {
        assert_eq!(
            bootstrap_cleanup_pass_limits(BootstrapStageFootprint::default()).unwrap(),
            (23, 45_732)
        );
        assert_eq!(
            bootstrap_cleanup_pass_limits(BootstrapStageFootprint::new(1, 128)).unwrap(),
            (24, 47_136)
        );
    }

    #[test]
    fn bootstrap_cleanup_byte_budget_counts_frames_and_file_payload_not_directory_blocks() {
        let root = std::env::temp_dir().join(format!(
            "codefire-bootstrap-cleanup-logical-bytes-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let name = ".bootstrap-CF-PROJECT-logical-bytes.removing";
        create_crashed_bootstrap_stage(&org_path, name, "app");
        let stage_path = org_path.join(name);
        let (entries, bytes) = logical_bootstrap_tree_cost(&stage_path);
        assert!(entries > 0);
        assert!(bytes > 0);
        let reserved = bootstrap_cleanup_pass_limits(BootstrapStageFootprint::default()).unwrap();
        assert!(
            bytes <= reserved.1,
            "logical cleanup {bytes} exceeds reserved single-pass bytes {}",
            reserved.1
        );

        let org_root = open_pinned_absolute(&org_path).unwrap();
        let stage = org_root.directory.open_dir_nofollow(name).unwrap();
        let identity = secure_worktree::StableFsIdentity::for_directory(&stage).unwrap();
        let mut exact = recovery_work(entries, bytes);
        scan_bootstrap_tree_bounded(
            &org_root.directory,
            OsStr::new(name),
            identity,
            0,
            &mut exact,
        )
        .unwrap();
        assert_eq!(exact.entries, entries);
        assert_eq!(exact.bytes, bytes);

        let mut short = recovery_work(entries, bytes - 1);
        let error = scan_bootstrap_tree_bounded(
            &org_root.directory,
            OsStr::new(name),
            identity,
            0,
            &mut short,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            BootstrapRecoveryError::ByteLimitExceeded { limit } if limit == bytes - 1
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_cleanup_rejects_realized_file_payload_beyond_reserved_pass_limit() {
        let root = std::env::temp_dir().join(format!(
            "codefire-bootstrap-cleanup-payload-overflow-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        let name = ".bootstrap-CF-PROJECT-payload-overflow.removing";
        create_crashed_bootstrap_stage(&org_path, name, "app");
        fs::write(
            org_path.join(name).join("overflow.bin"),
            vec![0_u8; 64 * 1024],
        )
        .unwrap();
        let (entries, bytes) = logical_bootstrap_tree_cost(&org_path.join(name));
        let (entry_limit, byte_limit) =
            bootstrap_cleanup_pass_limits(BootstrapStageFootprint::new(1, 0)).unwrap();
        assert!(entries <= entry_limit);
        assert!(
            bytes > byte_limit,
            "realized file payload {bytes} should exceed reserved {byte_limit}"
        );

        let org_root = open_pinned_absolute(&org_path).unwrap();
        let stage = org_root.directory.open_dir_nofollow(name).unwrap();
        let identity = secure_worktree::StableFsIdentity::for_directory(&stage).unwrap();
        let mut work = recovery_work(entry_limit, byte_limit);
        let error = scan_bootstrap_tree_bounded(
            &org_root.directory,
            OsStr::new(name),
            identity,
            0,
            &mut work,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            BootstrapRecoveryError::ByteLimitExceeded { limit } if limit == byte_limit
        ));
        assert!(org_path.join(name).join("overflow.bin").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_ledger_cancel_fits_reserved_rollback_after_logical_payload_accounting() {
        let root = std::env::temp_dir().join(format!(
            "codefire-bootstrap-cleanup-ledger-cancel-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let deadline = LockDeadline::from_options(&LockOptions::default());
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "bootstrap-logical-cancel",
        )
        .unwrap();
        let unpublished = prepare_unpublished_remote_project_with_ledger(
            &project,
            &deadline,
            &mut ledger,
            BootstrapStageFootprint::new(1, 128),
        )
        .unwrap()
        .expect("missing project should bootstrap");
        let stage = unpublished.project().display_root().to_path_buf();
        let (entries, bytes) = logical_bootstrap_tree_cost(&stage);
        let reserved = bootstrap_cleanup_pass_limits(BootstrapStageFootprint::new(1, 128)).unwrap();
        assert_eq!(entries, BOOTSTRAP_FIXED_LAYOUT_ENTRIES);
        assert!(entries <= reserved.0);
        assert!(
            bytes <= reserved.1,
            "prepared layout logical bytes {bytes} exceed reserved {}",
            reserved.1
        );
        let (_prelocked, mut publication) = unpublished.into_hidden_parts();
        publication.cancel_with_ledger(&mut ledger).unwrap();
        assert_ledger_has_no_live_owners(&ledger);
        assert!(!project.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_recovery_work_ledger_releases_iterator_scratch() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-recovery-ledger-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let org_path = root.join("projects/org");
        fs::create_dir_all(org_path.join("ordinary")).unwrap();
        create_crashed_bootstrap_stage(&org_path, ".bootstrap-CF-PROJECT-ledger-accounting", "app");
        let org_root = open_pinned_absolute(&org_path).unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "bootstrap-recovery-test",
        )
        .unwrap();

        let mut recovery = BootstrapRecoveryWork::with_ledger(&mut ledger).unwrap();
        recover_owned_bootstrap_stages(&org_root, "org", "app", &mut recovery).unwrap();
        drop(recovery);

        assert!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::UniqueNodesDiscovered]
                >= 2
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or(0),
            0
        );
        assert!(
            ledger.snapshot().peak[&codefire_core::DecisionWorkDimension::TemporaryBytes]
                >= 2 * 16 * 1024
        );

        let mut exhausted = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "bootstrap-recovery-exhausted-test",
        )
        .unwrap();
        let graph_limit = exhausted
            .profile()
            .limit(codefire_core::DecisionWorkDimension::GraphNodes)
            .unwrap()
            .limit;
        exhausted
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::GraphDiscovery,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::GraphNodes,
                    graph_limit,
                ),
            )
            .unwrap();
        let error = match BootstrapRecoveryWork::with_ledger(&mut exhausted) {
            Ok(_) => panic!("exhausted ledger unexpectedly reserved recovery work"),
            Err(error) => error,
        };
        let cli_error = error.into_cli();
        let CliError::ResourceBudget(exhaustion) = cli_error else {
            panic!("recovery budget exhaustion lost its typed CLI error");
        };
        assert_eq!(
            exhaustion.dimension,
            codefire_core::DecisionWorkDimension::GraphNodes
        );
        assert_eq!(exhaustion.observed, graph_limit + 1);
        assert_eq!(exhaustion.limit, graph_limit);
        assert_eq!(
            exhaustion.phase,
            codefire_core::DecisionWorkPhase::GraphDiscovery
        );
        assert_eq!(
            exhausted
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or(0),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_recovery_init_errors_release_reserved_owner() {
        let mut invalid_root_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "bootstrap-invalid-root",
        )
        .unwrap();
        let deadline = LockDeadline::from_options(&LockOptions::default());
        assert!(prepare_unpublished_remote_project_with_ledger(
            Path::new("/"),
            &deadline,
            &mut invalid_root_ledger,
            BootstrapStageFootprint::default(),
        )
        .is_err());
        assert_ledger_has_no_live_owners(&invalid_root_ledger);

        let root = std::env::temp_dir().join(format!(
            "codefire-bootstrap-recovery-lock-failure-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let locks_path = root.join("projects/.bootstrap-locks");
        fs::create_dir_all(&locks_path).unwrap();
        let locks = open_pinned_absolute(&locks_path).unwrap();
        let lock_name = OsStr::new("bootstrap-namespace.lock");
        let held = StableFileLock::acquire_in_directory(
            &locks.directory,
            lock_name,
            &locks_path.join(lock_name),
            StableLockMode::Exclusive,
            &deadline,
        )
        .unwrap();
        let mut lock_ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "bootstrap-lock-failure",
        )
        .unwrap();
        assert!(prepare_unpublished_remote_project_with_ledger(
            &root.join("projects/org/app"),
            &deadline,
            &mut lock_ledger,
            BootstrapStageFootprint::default(),
        )
        .is_err());
        assert_ledger_has_no_live_owners(&lock_ledger);
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_build_cleanup_failures_close_ledger_and_remain_recoverable() {
        for (label, extra_entry, cleanup_failure) in
            [("scan-bound", true, false), ("partial-remove", false, true)]
        {
            let root = std::env::temp_dir().join(format!(
                "codefire-bootstrap-build-failure-{label}-{}-{}",
                std::process::id(),
                TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let project = root.join("projects/org/app");
            let deadline = LockDeadline::from_options(&LockOptions::default());
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::local_v1(),
                codefire_core::DecisionWorkCommandKind::ObjectTransfer,
                format!("bootstrap-build-failure-{label}"),
            )
            .unwrap();
            INJECT_BOOTSTRAP_BUILD_FAILURE.with(|flag| flag.set(true));
            INJECT_BOOTSTRAP_BUILD_EXTRA_ENTRY.with(|flag| flag.set(extra_entry));
            INJECT_BOOTSTRAP_BUILD_CLEANUP_FAILURE.with(|flag| flag.set(cleanup_failure));

            assert!(prepare_unpublished_remote_project_with_ledger(
                &project,
                &deadline,
                &mut ledger,
                BootstrapStageFootprint::default(),
            )
            .is_err());
            assert_ledger_has_no_live_owners(&ledger);

            let unpublished = prepare_unpublished_remote_project_with_ledger(
                &project,
                &deadline,
                &mut ledger,
                BootstrapStageFootprint::default(),
            )
            .unwrap()
            .expect("failed bootstrap stage must be recoverable");
            let (_prelocked, mut publication) = unpublished.into_hidden_parts();
            publication.cancel_with_ledger(&mut ledger).unwrap();
            assert_ledger_has_no_live_owners(&ledger);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn hardlinked_project_descriptor_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-descriptor-hardlink-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        ensure_remote_project_layout(&project).unwrap();
        fs::hard_link(
            project.join(PROJECT_DESCRIPTOR_FILE),
            root.join("descriptor-copy.json"),
        )
        .unwrap();
        assert!(RemoteProject::open(&project).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transaction_cleanup_preserves_replacement_directory() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-transaction-swap-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project_path = root.join("projects/org/app");
        ensure_remote_project_layout(&project_path).unwrap();
        let project = RemoteProject::open(&project_path).unwrap();
        let transaction = project.create_transaction("tx_fixture").unwrap();
        let transaction_path = project_path.join("transactions/tx_fixture");
        let moved = project_path.join("transactions/tx_fixture-moved");
        fs::rename(&transaction_path, &moved).unwrap();
        fs::create_dir(&transaction_path).unwrap();
        assert!(project.remove_transaction_directory(&transaction).is_err());
        assert!(transaction_path.is_dir());
        assert!(moved.is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn maximum_length_authorities_round_trip_through_bounded_filenames() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-authority-boundary-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project_path = root.join("projects/org/app");
        ensure_remote_project_layout(&project_path).unwrap();
        let project = RemoteProject::open(&project_path).unwrap();
        let maximum = "a".repeat(255);
        let unicode = "界".repeat(80);
        for authority in [
            RemoteAuthority::Branch(maximum.clone()),
            RemoteAuthority::Branch(unicode),
            RemoteAuthority::MergeRequest(maximum.clone()),
            RemoteAuthority::GcAudit(maximum),
        ] {
            let path = authority.path(&project_path).unwrap();
            assert!(path.file_name().unwrap().as_encoded_bytes().len() <= 255);
            project
                .write_authority_atomic(&authority, b"{\"version\":1}\n")
                .unwrap();
            assert_eq!(
                project.read_authority_bounded(&authority, 1024).unwrap(),
                b"{\"version\":1}\n"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hashed_and_direct_authority_names_cannot_alias() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-authority-collision-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project_path = root.join("projects/org/app");
        ensure_remote_project_layout(&project_path).unwrap();
        let project = RemoteProject::open(&project_path).unwrap();
        let maximum = "a".repeat(255);
        for long in [
            RemoteAuthority::Branch(maximum.clone()),
            RemoteAuthority::MergeRequest(maximum.clone()),
            RemoteAuthority::GcAudit(maximum.clone()),
        ] {
            let long_path = long.path(&project_path).unwrap();
            let digest = long_path
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap()
                .strip_prefix(REMOTE_INTERNAL_HASH_PREFIX)
                .unwrap();
            let direct_name = format!("h-{digest}");
            let direct = match &long {
                RemoteAuthority::Branch(_) => RemoteAuthority::Branch(direct_name),
                RemoteAuthority::MergeRequest(_) => RemoteAuthority::MergeRequest(direct_name),
                RemoteAuthority::GcAudit(_) => RemoteAuthority::GcAudit(direct_name),
                _ => unreachable!(),
            };
            let direct_path = direct.path(&project_path).unwrap();
            assert_ne!(long_path, direct_path);
            project
                .write_authority_atomic(&long, b"{\"authority\":\"long\"}\n")
                .unwrap();
            project
                .write_authority_atomic(&direct, b"{\"authority\":\"direct\"}\n")
                .unwrap();
            assert_eq!(
                project.read_authority_bounded(&long, 1024).unwrap(),
                b"{\"authority\":\"long\"}\n"
            );
            assert_eq!(
                project.read_authority_bounded(&direct, 1024).unwrap(),
                b"{\"authority\":\"direct\"}\n"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn bootstrap_rejects_intermediate_symlink_without_outside_write() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "codefire-remote-bootstrap-link-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let outside = root.with_extension("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("redirect")).unwrap();
        let project = root.join("redirect/projects/org/app");
        assert!(ensure_remote_project_layout(&project).is_err());
        assert!(!outside.join("projects").exists());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn legacy_authority_is_preserved_and_requires_explicit_migration() {
        let root = std::env::temp_dir().join(format!(
            "codefire-remote-legacy-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        fs::create_dir_all(project.join("branches")).unwrap();
        let legacy = project.join("branches/main.json");
        fs::write(&legacy, b"{\"head\":\"legacy\"}\n").unwrap();
        let error = ensure_remote_project_layout(&project).unwrap_err();
        assert!(error.to_string().contains("administrative quiescence"));
        assert_eq!(fs::read(&legacy).unwrap(), b"{\"head\":\"legacy\"}\n");
        assert!(!project.join(PROJECT_DESCRIPTOR_FILE).exists());
        fs::remove_dir_all(root).unwrap();
    }
}
