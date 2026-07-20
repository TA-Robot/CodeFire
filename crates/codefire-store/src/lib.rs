use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
#[cfg(unix)]
use cap_std::fs::OpenOptionsExt as _;
use cap_std::fs::{Dir, OpenOptions as CapabilityOpenOptions};
use fs2::FileExt as Fs2FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
#[cfg(test)]
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const OBJECT_ID_DIGEST_HEX_LENGTH: usize = 24;
pub const MAX_OBJECT_RECORD_BYTES: u64 = 64 * 1024 * 1024;
pub const CURRENT_SEMANTIC_VALIDATION_PROFILE: &str = "sealed-snapshot-v1";
const LEGACY_OBJECT_ID_DIGEST_HEX_LENGTH: usize = 12;
const SEALED_REFERENCE_DIAGNOSTIC_SAMPLE_BYTE_LIMIT: usize = 256;
const MAX_PINNED_EXTENSION_OBJECT_SUBDIRS: usize = 128;
const PREPARED_OBJECT_STATE_PREALLOCATION_BYTES: u64 =
    codefire_core::LOGICAL_FRAME_BASE_BYTES + 256;
const PREPARED_STAGE_LOCK_NAME: &str = ".codefire-stage-v1.lock";
const PREPARED_STAGE_PREFIX: &str = ".codefire-stage-";
const PREPARED_STAGE_V1_PREFIX: &str = ".codefire-stage-v1.";
const PREPARED_STAGE_SUFFIX: &str = ".tmp";
const PREPARED_STAGE_LOCK_RETRY: Duration = Duration::from_millis(10);

const OBJECT_KINDS: &[ObjectKind] = &[
    ObjectKind::new("blob", "CF-BLOB", "blobs"),
    ObjectKind::new("content_manifest", "CF-MANIFEST", "content_manifests"),
    ObjectKind::new("atom_index", "CF-ATOMINDEX", "atom_indexes"),
    ObjectKind::new("trace_graph", "CF-TRACE", "trace_graphs"),
    ObjectKind::new("fire_ledger", "CF-FIRELEDGER", "fire_ledgers"),
    ObjectKind::new("resolution_ledger", "CF-RESOLUTION", "resolution_ledgers"),
    ObjectKind::new("verification", "CF-VERIFY", "verifications"),
    ObjectKind::new("policy", "CF-POLICY", "policies"),
    ObjectKind::new("artifact_ref", "CF-ARTIFACT", "artifact_refs"),
    ObjectKind::new("evidence", "CF-EVIDENCE", "evidence"),
    ObjectKind::new("commit", "CF-COMMIT", "commits"),
    ObjectKind::new("branch", "CF-BRANCH", "branches"),
];

#[derive(Debug, Clone, Copy)]
struct ObjectKind {
    type_tag: &'static str,
    prefix: &'static str,
    subdir: &'static str,
}

impl ObjectKind {
    const fn new(type_tag: &'static str, prefix: &'static str, subdir: &'static str) -> Self {
        Self {
            type_tag,
            prefix,
            subdir,
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Json(serde_json::Error),
    Io(std::io::Error),
    UnknownObjectType(String),
    IncompleteRecord(String),
    ObjectRecordTooLarge {
        path: PathBuf,
        size: u64,
        limit: u64,
    },
    ObjectIdMismatch {
        expected: String,
        actual: String,
    },
    ObjectHashMismatch {
        expected: String,
        actual: String,
    },
    ObjectCollision {
        object_id: String,
        requested_type: String,
        existing_type: String,
        requested_hash: String,
        existing_hash: String,
        canonical_payload_matches: bool,
    },
    FilenameMismatch {
        expected: String,
        actual: String,
    },
    ObjectNotFound(String),
    ObjectPublicationRetryRequired,
    SealedValidationSubjectChanged {
        object_id: String,
        path: PathBuf,
        kind: SealedValidationSubjectChangeKind,
        expected_size: u64,
        actual_size: Option<u64>,
    },
    DecisionWorkAccounting(Box<codefire_core::DecisionWorkAccountingError>),
    ValidationCancelled,
    InvalidSealedCommit(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Json(error) => write!(f, "json error: {error}"),
            StoreError::Io(error) => write!(f, "io error: {error}"),
            StoreError::UnknownObjectType(type_tag) => write!(f, "unknown object type: {type_tag}"),
            StoreError::IncompleteRecord(context) => {
                write!(f, "object record is incomplete: {context}")
            }
            StoreError::ObjectRecordTooLarge { path, size, limit } => write!(
                f,
                "object record exceeds the {limit}-byte limit at {}: {size} bytes",
                path.display()
            ),
            StoreError::ObjectIdMismatch { expected, actual } => {
                write!(f, "object id mismatch: expected {expected}, got {actual}")
            }
            StoreError::ObjectHashMismatch { expected, actual } => {
                write!(f, "object hash mismatch: expected {expected}, got {actual}")
            }
            StoreError::ObjectCollision {
                object_id,
                requested_type,
                existing_type,
                requested_hash,
                existing_hash,
                canonical_payload_matches,
            } => {
                write!(
                    f,
                    "object id collision for {object_id}: requested {requested_type} with hash {requested_hash}, existing {existing_type} with hash {existing_hash}, canonical payloads match: {canonical_payload_matches}"
                )
            }
            StoreError::FilenameMismatch { expected, actual } => {
                write!(
                    f,
                    "object filename/id mismatch: expected {expected}, got {actual}"
                )
            }
            StoreError::ObjectNotFound(object_id) => write!(f, "object not found: {object_id}"),
            StoreError::ObjectPublicationRetryRequired => {
                write!(f, "object publication raced after seal; retry the same request")
            }
            StoreError::SealedValidationSubjectChanged {
                object_id,
                path,
                kind,
                expected_size,
                actual_size,
            } => write!(
                f,
                "sealed validation subject changed: object {object_id} at {} ({kind}; expected {expected_size} bytes, actual {})",
                path.display(),
                actual_size
                    .map(|size| size.to_string())
                    .unwrap_or_else(|| "unavailable".to_string())
            ),
            StoreError::DecisionWorkAccounting(error) => {
                write!(f, "sealed validation work accounting failed: {error}")
            }
            StoreError::ValidationCancelled => {
                write!(f, "sealed validation was cancelled")
            }
            StoreError::InvalidSealedCommit(message) => {
                write!(f, "sealed commit validation failed: {message}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SealedValidationSubjectChangeKind {
    Unreadable,
    SizeChanged,
    DigestChanged,
}

impl fmt::Display for SealedValidationSubjectChangeKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable => write!(formatter, "unreadable"),
            Self::SizeChanged => write!(formatter, "size_changed"),
            Self::DigestChanged => write!(formatter, "digest_changed"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        StoreError::Json(error)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        StoreError::Io(error)
    }
}

impl From<codefire_core::DecisionWorkAccountingError> for StoreError {
    fn from(error: codefire_core::DecisionWorkAccountingError) -> Self {
        Self::DecisionWorkAccounting(Box::new(error))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRecord {
    pub object_id: String,
    #[serde(rename = "type")]
    pub type_tag: String,
    pub hash: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectRecordDocument {
    pub record: ObjectRecord,
    pub extensions: Map<String, Value>,
}

#[must_use = "accounted object document ownership must be transferred or released"]
pub struct AccountedObjectRecordDocument {
    document: Option<ObjectRecordDocument>,
    retained_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    active: bool,
}

impl AccountedObjectRecordDocument {
    pub fn document(&self) -> &ObjectRecordDocument {
        self.document
            .as_ref()
            .expect("active accounted document retains its payload")
    }

    pub fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    /// Transfers the document and its unchanged RetainedBytes scalar to a
    /// caller-owned aggregate on the same ledger.
    pub fn transfer_to_owner(
        &mut self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(ObjectRecordDocument, u64), StoreError> {
        self.ensure_ledger(ledger)?;
        let document = self.document.take().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "accounted object document was already transferred".to_string(),
            )
        })?;
        self.active = false;
        Ok((document, self.retained_bytes))
    }

    pub fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger(ledger)?;
        if !self.active {
            return Ok(());
        }
        drop(self.document.take());
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        )?;
        self.retained_bytes = 0;
        self.active = false;
        Ok(())
    }

    fn ensure_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> Result<(), StoreError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "accounted object document belongs to a different decision-work ledger".to_string(),
            ))
        }
    }
}

impl Drop for AccountedObjectRecordDocument {
    fn drop(&mut self) {
        debug_assert!(
            !self.active,
            "accounted object document dropped without its ledger owner releasing it"
        );
    }
}

/// Request-scoped reader that pins one object-store root and namespace set for
/// every record read. Call `finish_with_cancellation` to close the ambient
/// identity fence, or `cancel` on an earlier failure.
#[must_use = "an accounted object-store reader must be finished or cancelled"]
pub struct AccountedObjectStoreReader {
    bound_store: Option<BoundObjectStore>,
    cache: SealedCommitValidationCache,
    root_bytes: u64,
    binding_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    active: bool,
}

impl Drop for AccountedObjectStoreReader {
    fn drop(&mut self) {
        debug_assert!(
            !self.active,
            "accounted object-store reader dropped without releasing its ledger ownership"
        );
    }
}

#[derive(Debug)]
pub struct ObjectStoreRecordScanEntry {
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
    pub document: Result<ObjectRecordDocument, StoreError>,
}

pub fn canonical_json(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(value)
}

pub fn object_digest(type_tag: &str, payload: &Value) -> Result<String, serde_json::Error> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(type_tag.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical_json(payload)?);
    Ok(codefire_util::sha256_hex(&bytes))
}

pub fn object_id(type_tag: &str, payload: &Value) -> Result<String, serde_json::Error> {
    let digest = object_digest(type_tag, payload)?;
    Ok(object_id_from_digest(
        type_tag,
        &digest,
        OBJECT_ID_DIGEST_HEX_LENGTH,
    ))
}

fn legacy_object_id(type_tag: &str, digest: &str) -> String {
    object_id_from_digest(type_tag, digest, LEGACY_OBJECT_ID_DIGEST_HEX_LENGTH)
}

fn object_id_from_digest(type_tag: &str, digest: &str, digest_hex_length: usize) -> String {
    format!(
        "{}-{}",
        object_prefix(type_tag),
        &digest[..digest_hex_length]
    )
}

pub fn object_prefix(type_tag: &str) -> &'static str {
    object_kind_by_type(type_tag)
        .map(|kind| kind.prefix)
        .unwrap_or("CF-OBJECT")
}

pub fn object_subdir(type_tag: &str) -> Option<&'static str> {
    object_kind_by_type(type_tag).map(|kind| kind.subdir)
}

pub fn known_object_subdirs() -> impl Iterator<Item = &'static str> {
    OBJECT_KINDS.iter().map(|kind| kind.subdir)
}

fn object_kind_by_type(type_tag: &str) -> Option<ObjectKind> {
    OBJECT_KINDS
        .iter()
        .copied()
        .find(|kind| kind.type_tag == type_tag)
}

fn object_subdir_by_id(object_id: &str) -> Option<&'static str> {
    OBJECT_KINDS
        .iter()
        .filter(|kind| object_id.starts_with(kind.prefix))
        .max_by_key(|kind| kind.prefix.len())
        .map(|kind| kind.subdir)
}

fn allows_unknown_subdir_lookup(object_id: &str) -> bool {
    !object_id.starts_with("CF-") || object_id.starts_with("CF-OBJECT-")
}

pub fn object_record(type_tag: &str, payload: Value) -> Result<ObjectRecord, StoreError> {
    object_record_with_digest(type_tag, payload, &object_digest)
}

fn object_record_with_digest<D>(
    type_tag: &str,
    payload: Value,
    digest: &D,
) -> Result<ObjectRecord, StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
{
    let hash = digest(type_tag, &payload)?;
    Ok(ObjectRecord {
        object_id: object_id_from_digest(type_tag, &hash, OBJECT_ID_DIGEST_HEX_LENGTH),
        type_tag: type_tag.to_string(),
        hash,
        payload,
    })
}

pub fn validate_object_record(
    record: &ObjectRecord,
    expected_object_id: Option<&str>,
    path: Option<&Path>,
) -> Result<(), StoreError> {
    validate_object_record_with_digest(record, expected_object_id, path, &object_digest)
}

fn validate_object_record_with_digest<D>(
    record: &ObjectRecord,
    expected_object_id: Option<&str>,
    path: Option<&Path>,
    digest: &D,
) -> Result<(), StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
{
    if record.type_tag.is_empty() || record.payload.is_null() {
        return Err(StoreError::IncompleteRecord(
            path.map(|value| value.display().to_string())
                .or_else(|| expected_object_id.map(str::to_string))
                .unwrap_or_else(|| "(unknown)".to_string()),
        ));
    }

    let computed_hash = digest(&record.type_tag, &record.payload)?;
    let computed_id = object_id_from_digest(
        &record.type_tag,
        &computed_hash,
        OBJECT_ID_DIGEST_HEX_LENGTH,
    );
    let legacy_id = legacy_object_id(&record.type_tag, &computed_hash);
    if record.object_id != computed_id && record.object_id != legacy_id {
        return Err(StoreError::ObjectIdMismatch {
            expected: computed_id,
            actual: record.object_id.clone(),
        });
    }
    if record.hash != computed_hash {
        return Err(StoreError::ObjectHashMismatch {
            expected: computed_hash,
            actual: record.hash.clone(),
        });
    }
    if let Some(expected) = expected_object_id {
        if expected != record.object_id {
            return Err(StoreError::ObjectIdMismatch {
                expected: expected.to_string(),
                actual: record.object_id.clone(),
            });
        }
    }
    if let Some(path) = path {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if stem != record.object_id {
            return Err(StoreError::FilenameMismatch {
                expected: record.object_id.clone(),
                actual: stem.to_string(),
            });
        }
    }
    Ok(())
}

pub fn object_record_path(objects_root: &Path, object_id: &str) -> Option<PathBuf> {
    if let Some(subdir) = object_subdir_by_id(object_id) {
        let candidate = objects_root.join(subdir).join(format!("{object_id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
        return None;
    }
    if !allows_unknown_subdir_lookup(object_id) {
        return None;
    }
    for entry in fs::read_dir(objects_root).ok()? {
        let entry = entry.ok()?;
        let file_type = entry.file_type().ok()?;
        if !file_type.is_dir() {
            continue;
        }
        let candidate = entry.path().join(format!("{object_id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn store_object(
    objects_root: &Path,
    type_tag: &str,
    payload: Value,
) -> Result<String, StoreError> {
    store_object_with_identity_and_link(
        objects_root,
        type_tag,
        payload,
        &object_digest,
        |directory, source, destination, _| directory.hard_link(source, directory, destination),
    )
}

/// Publishes an already authenticated object record without changing its supported public ID.
///
/// New objects should use [`store_object`], which always assigns the current public-ID width.
/// Transfer and migration boundaries may use this function after receiving a complete record; the
/// record is revalidated and then published under its exact current or legacy ID through the same
/// no-follow, collision, link-race, and capability-identity checks as an ordinary store write.
pub fn store_object_record(
    objects_root: &Path,
    record: ObjectRecord,
) -> Result<String, StoreError> {
    store_object_record_with_identity_and_link(
        objects_root,
        record,
        &object_digest,
        |directory, source, destination, _| directory.hard_link(source, directory, destination),
    )
}

/// Publishes an authenticated object record while preserving supported
/// non-identity top-level extension fields byte-semantically as JSON values.
/// Extension fields never participate in the object digest, so an existing
/// immutable record must carry the exact same extension map.
pub fn store_object_record_document(
    objects_root: &Path,
    document: ObjectRecordDocument,
) -> Result<String, StoreError> {
    store_object_record_document_with_identity_and_link(
        objects_root,
        document,
        &object_digest,
        |directory, source, destination, _| directory.hard_link(source, directory, destination),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedObjectRecordDisposition {
    Missing,
    Exact,
}

#[derive(Debug, Clone)]
struct PreparedExistingObjectRecord {
    size_bytes: u64,
    document_digest: String,
}

#[derive(Debug)]
enum PreparedObjectRecordState {
    Missing {
        stage_name: PathBuf,
        stage_owned: bool,
    },
    Exact(PreparedExistingObjectRecord),
    Published,
}

struct PreparedObjectRecordEntry {
    document: ObjectRecordDocument,
    rendered: Vec<u8>,
    directory: Dir,
    directory_identity: ObjectStoreIdentity,
    subdir: &'static str,
    file_name: PathBuf,
    display_path: PathBuf,
    state: PreparedObjectRecordState,
    publication_work: Option<codefire_core::DecisionWorkReservation>,
}

struct PreparedObjectStoreRoot {
    directory: Dir,
    identity: ObjectStoreIdentity,
}

struct PreparedObjectStageLock {
    file: std::fs::File,
    identity: ObjectStoreIdentity,
}

impl Drop for PreparedObjectStageLock {
    fn drop(&mut self) {
        let _ = Fs2FileExt::unlock(&self.file);
    }
}

impl PreparedObjectStageLock {
    fn ensure_current(&self, root: &Dir) -> Result<(), StoreError> {
        let metadata = root.symlink_metadata(PREPARED_STAGE_LOCK_NAME)?;
        if metadata.is_file()
            && prepared_stage_has_one_link(&metadata)
            && stable_capability_object_store_identity(&metadata)? == self.identity
        {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "prepared object stage lock identity changed".to_string(),
            ))
        }
    }
}

/// A no-follow, ledger-owned batch of immutable object records prepared for publication.
///
/// Preparation requires an existing object-store root and affected subdirectories; it never
/// creates public layout. It validates and encodes every requested document, pins those existing
/// directories, records exact existing files, and stages missing files under create-new `.tmp`
/// names (mode 0600 on Unix, excluded by the `.json` object namespace). `revalidate_preseal`
/// rechecks all pinned names and stage bytes immediately before the
/// combined owner seals its ledger. After that seal, `commit_after_seal` performs only reserved-work
/// consumption and no-replace hard links; an `AlreadyExists` race is a durable retry instead of a
/// post-seal parse/hash operation.
///
/// The batch does not borrow the ledger, so it can coexist with other prepared owners under one
/// combined transaction. The combined owner must pass the same ledger to commit or cancellation;
/// raw drop removes unpublished stages but deliberately debug-asserts unresolved accounting.
#[must_use = "a prepared object batch must be committed or explicitly cancelled"]
pub struct PreparedObjectRecordBatch {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    objects_root: PathBuf,
    root: Option<PreparedObjectStoreRoot>,
    stage_lock: Option<PreparedObjectStageLock>,
    pending_documents: Option<Vec<ObjectRecordDocument>>,
    entries: Vec<PreparedObjectRecordEntry>,
    permit: Option<codefire_core::DecisionWorkPermit>,
    reserved_records: u64,
    reserved_bytes: u64,
    retained_bytes: u64,
    revalidated: bool,
    active: bool,
}

impl PreparedObjectRecordBatch {
    pub fn prepare_with_ledger_and_poll<C>(
        objects_root: &Path,
        documents: Vec<ObjectRecordDocument>,
        ledger: &mut codefire_core::DecisionWorkLedger,
        should_cancel: C,
    ) -> Result<Self, StoreError>
    where
        C: FnMut() -> bool,
    {
        Self::prepare_with_ledger_poll_and_stage_name(
            objects_root,
            documents,
            ledger,
            should_cancel,
            unique_prepared_object_stage_name,
        )
    }

    fn prepare_with_ledger_poll_and_stage_name<C, N>(
        objects_root: &Path,
        documents: Vec<ObjectRecordDocument>,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
        mut stage_name: N,
    ) -> Result<Self, StoreError>
    where
        C: FnMut() -> bool,
        N: FnMut(&Path) -> Result<PathBuf, StoreError>,
    {
        if !objects_root.is_absolute() {
            return Err(StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "prepared object store path must be absolute: {}",
                    objects_root.display()
                ),
            )));
        }

        let input_record_count =
            u64::try_from(documents.len()).map_err(|_| prepared_object_batch_size_overflow())?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::ObjectRecords,
                input_record_count,
            )
            .count(
                codefire_core::DecisionWorkCounter::ObjectRecords,
                input_record_count,
            ),
        )?;

        let mut documents = documents;
        for document in &documents {
            poll_prepared_object_batch(&mut should_cancel)?;
            if object_subdir(&document.record.type_tag).is_none() {
                return Err(StoreError::UnknownObjectType(
                    document.record.type_tag.clone(),
                ));
            }
        }
        documents
            .sort_unstable_by(|left, right| left.record.object_id.cmp(&right.record.object_id));
        for pair in documents.windows(2) {
            if pair[0].record.object_id == pair[1].record.object_id && pair[0] != pair[1] {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "prepared object batch contains divergent documents for {}",
                    pair[0].record.object_id
                )));
            }
        }
        documents.dedup_by(|left, right| left == right);

        let (retained_bytes, transition_bytes) =
            prepared_object_batch_retained_upper(objects_root, &documents, &mut should_cancel)?;
        for document in &documents {
            poll_prepared_object_batch(&mut should_cancel)?;
            let compact_bytes = compact_object_record_document_size_with_cancellation(
                document,
                &mut should_cancel,
            )?;
            let rendered_bytes = rendered_object_record_document_size_with_cancellation(
                document,
                &mut should_cancel,
            )?;
            if rendered_bytes > MAX_OBJECT_RECORD_BYTES {
                return Err(StoreError::ObjectRecordTooLarge {
                    path: prepared_object_record_path(objects_root, &document.record)?,
                    size: rendered_bytes,
                    limit: MAX_OBJECT_RECORD_BYTES,
                });
            }
            let aggregate_bytes = compact_bytes
                .checked_add(rendered_bytes)
                .ok_or_else(prepared_object_batch_size_overflow)?;
            ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    aggregate_bytes,
                )
                .count(
                    codefire_core::DecisionWorkCounter::BytesHashed,
                    compact_bytes,
                ),
            )?;
        }
        let record_count =
            u64::try_from(documents.len()).map_err(|_| prepared_object_batch_size_overflow())?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Validation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::SemanticValidations,
                record_count,
            ),
        )?;

        let mut prepared = Self {
            ledger_identity: ledger.identity(),
            objects_root: objects_root.to_path_buf(),
            root: None,
            stage_lock: None,
            pending_documents: Some(documents),
            entries: Vec::new(),
            permit: None,
            reserved_records: 0,
            reserved_bytes: 0,
            retained_bytes,
            revalidated: false,
            active: true,
        };
        let initialized = (|| {
            for document in prepared
                .pending_documents
                .as_ref()
                .expect("prepared object documents are owned")
            {
                validate_object_extension_support(&document.extensions)?;
                validate_object_record(&document.record, Some(&document.record.object_id), None)?;
            }
            prepared.initialize(
                ledger,
                transition_bytes,
                &mut should_cancel,
                &mut stage_name,
            )
        })();
        match initialized {
            Ok(()) => Ok(prepared),
            Err(error) => match prepared.cancel_inner(ledger) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(cleanup),
            },
        }
    }

    fn initialize<C, N>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        transition_bytes: u64,
        should_cancel: &mut C,
        stage_name: &mut N,
    ) -> Result<(), StoreError>
    where
        C: FnMut() -> bool,
        N: FnMut(&Path) -> Result<PathBuf, StoreError>,
    {
        poll_prepared_object_batch(should_cancel)?;
        let root_directory = open_prepared_object_root_accounted(
            &self.objects_root,
            ledger,
            PreparedObjectReadKind::Initial,
        )?;
        let root_identity =
            stable_capability_object_store_identity(&root_directory.dir_metadata()?)?;
        self.root = Some(PreparedObjectStoreRoot {
            directory: root_directory,
            identity: root_identity,
        });
        self.stage_lock = Some(acquire_prepared_object_stage_lock(
            &self
                .root
                .as_ref()
                .expect("prepared object root is bound")
                .directory,
            should_cancel,
        )?);

        let documents = self
            .pending_documents
            .take()
            .expect("prepared object documents are owned");
        self.entries = Vec::with_capacity(documents.len());
        for document in documents {
            poll_prepared_object_batch(should_cancel)?;
            let subdir = object_subdir(&document.record.type_tag)
                .ok_or_else(|| StoreError::UnknownObjectType(document.record.type_tag.clone()))?;
            let root = self.root.as_ref().expect("prepared object root is bound");
            let directory = root.directory.open_dir_nofollow(subdir)?;
            let directory_identity =
                stable_capability_object_store_identity(&directory.dir_metadata()?)?;
            let mut file_name = std::ffi::OsString::with_capacity(
                document
                    .record
                    .object_id
                    .len()
                    .checked_add(5)
                    .ok_or_else(prepared_object_batch_size_overflow)?,
            );
            file_name.push(&document.record.object_id);
            file_name.push(".json");
            let file_name = PathBuf::from(file_name);
            let display_path = build_prepared_object_path(&self.objects_root, subdir, &file_name)?;
            let rendered_size =
                rendered_object_record_document_size_with_cancellation(&document, should_cancel)?;
            let rendered = serialize_prepared_object_document(
                &document,
                &display_path,
                rendered_size,
                should_cancel,
            )?;

            let state = match directory.symlink_metadata(&file_name) {
                Ok(_) => PreparedObjectRecordState::Exact(read_and_validate_prepared_existing(
                    &directory,
                    &file_name,
                    &display_path,
                    &document,
                    ledger,
                    PreparedObjectReadKind::Initial,
                    should_cancel,
                )?),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    PreparedObjectRecordState::Missing {
                        stage_name: stage_name(&file_name)?,
                        stage_owned: false,
                    }
                }
                Err(error) => return Err(StoreError::Io(error)),
            };
            let publication_work = matches!(state, PreparedObjectRecordState::Missing { .. })
                .then(|| prepared_object_publication_work(1, rendered.len() as u64));
            self.entries.push(PreparedObjectRecordEntry {
                document,
                rendered,
                directory,
                directory_identity,
                subdir,
                file_name,
                display_path,
                state,
                publication_work,
            });
        }

        self.reserved_records = self
            .entries
            .iter()
            .filter(|entry| matches!(entry.state, PreparedObjectRecordState::Missing { .. }))
            .count()
            .try_into()
            .map_err(|_| prepared_object_batch_size_overflow())?;
        self.reserved_bytes = self
            .entries
            .iter()
            .filter(|entry| matches!(entry.state, PreparedObjectRecordState::Missing { .. }))
            .try_fold(0_u64, |total, entry| {
                total
                    .checked_add(entry.rendered.len() as u64)
                    .ok_or_else(prepared_object_batch_size_overflow)
            })?;
        for entry in &self.entries {
            let state_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
                .checked_add(match &entry.state {
                    PreparedObjectRecordState::Missing { stage_name, .. } => {
                        stage_name.as_os_str().as_encoded_bytes().len() as u64
                    }
                    PreparedObjectRecordState::Exact(existing) => {
                        existing.document_digest.len() as u64
                    }
                    PreparedObjectRecordState::Published => 0,
                })
                .ok_or_else(prepared_object_batch_size_overflow)?;
            if state_bytes > PREPARED_OBJECT_STATE_PREALLOCATION_BYTES {
                return Err(StoreError::InvalidSealedCommit(
                    "prepared object state exceeded its retained frame".to_string(),
                ));
            }
        }
        recover_prepared_object_stages(&self.entries, ledger, should_cancel, || Ok(()))?;
        self.stage_lock
            .as_ref()
            .expect("prepared object stage lock is held")
            .ensure_current(
                &self
                    .root
                    .as_ref()
                    .expect("prepared object root is bound")
                    .directory,
            )?;
        if self.reserved_records > 0 {
            self.permit = Some(ledger.reserve(prepared_object_publication_work(
                self.reserved_records,
                self.reserved_bytes,
            ))?);
        }

        let release_bytes = transition_bytes;
        if release_bytes > 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                release_bytes,
            )?;
            self.retained_bytes = self
                .retained_bytes
                .checked_sub(release_bytes)
                .ok_or_else(prepared_object_batch_size_overflow)?;
        }

        for entry in &mut self.entries {
            if !matches!(entry.state, PreparedObjectRecordState::Missing { .. }) {
                continue;
            }
            create_and_validate_prepared_stage(entry, ledger, should_cancel)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn disposition(&self, object_id: &str) -> Option<PreparedObjectRecordDisposition> {
        self.entries
            .iter()
            .find(|entry| entry.document.record.object_id == object_id)
            .map(|entry| match entry.state {
                PreparedObjectRecordState::Missing { .. } => {
                    PreparedObjectRecordDisposition::Missing
                }
                PreparedObjectRecordState::Exact(_) => PreparedObjectRecordDisposition::Exact,
                PreparedObjectRecordState::Published => PreparedObjectRecordDisposition::Exact,
            })
    }

    pub fn revalidate_preseal<C>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        should_cancel: C,
    ) -> Result<(), StoreError>
    where
        C: FnMut() -> bool,
    {
        self.revalidate_preseal_with_closing_hook(ledger, should_cancel, || Ok(()))
    }

    fn revalidate_preseal_with_closing_hook<C, H>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
        before_closing_fence: H,
    ) -> Result<(), StoreError>
    where
        C: FnMut() -> bool,
        H: FnOnce() -> Result<(), StoreError>,
    {
        self.ensure_ledger_identity(ledger)?;
        self.revalidated = false;
        poll_prepared_object_batch(&mut should_cancel)?;
        let root = self.root.as_ref().ok_or_else(|| {
            StoreError::InvalidSealedCommit("prepared object root is unavailable".to_string())
        })?;
        self.stage_lock
            .as_ref()
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "prepared object stage lock is unavailable".to_string(),
                )
            })?
            .ensure_current(&root.directory)?;
        let ambient_root = open_prepared_object_root_accounted(
            &self.objects_root,
            ledger,
            PreparedObjectReadKind::Revalidation,
        )
        .map_err(prepared_object_revalidation_error)?;
        if stable_capability_object_store_identity(&ambient_root.dir_metadata()?)? != root.identity
        {
            return Err(prepared_object_subject_changed_error());
        }

        for entry in &mut self.entries {
            poll_prepared_object_batch(&mut should_cancel)?;
            let ambient_subdir = ambient_root
                .open_dir_nofollow(entry.subdir)
                .map_err(|_| prepared_object_subject_changed_error())?;
            if stable_capability_object_store_identity(&ambient_subdir.dir_metadata()?)?
                != entry.directory_identity
            {
                return Err(prepared_object_subject_changed_error());
            }
            match &entry.state {
                PreparedObjectRecordState::Exact(expected) => {
                    let current = read_and_validate_prepared_existing(
                        &entry.directory,
                        &entry.file_name,
                        &entry.display_path,
                        &entry.document,
                        ledger,
                        PreparedObjectReadKind::Revalidation,
                        &mut should_cancel,
                    )
                    .map_err(|error| prepared_existing_drift_error(entry, expected, error))?;
                    if current.size_bytes != expected.size_bytes {
                        return Err(prepared_object_subject_changed_error());
                    }
                    if current.document_digest != expected.document_digest {
                        return Err(prepared_object_subject_changed_error());
                    }
                }
                PreparedObjectRecordState::Missing { .. } => {
                    match entry.directory.symlink_metadata(&entry.file_name) {
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            validate_prepared_stage(
                                entry,
                                ledger,
                                PreparedObjectReadKind::Revalidation,
                                &mut should_cancel,
                            )?;
                        }
                        Ok(_) => {
                            let exact = read_and_validate_prepared_existing(
                                &entry.directory,
                                &entry.file_name,
                                &entry.display_path,
                                &entry.document,
                                ledger,
                                PreparedObjectReadKind::Revalidation,
                                &mut should_cancel,
                            )?;
                            remove_prepared_stage(entry)?;
                            entry.state = PreparedObjectRecordState::Exact(exact);
                        }
                        Err(error) => return Err(StoreError::Io(error)),
                    }
                }
                PreparedObjectRecordState::Published => {
                    return Err(StoreError::InvalidSealedCommit(
                        "prepared object batch was already published".to_string(),
                    ));
                }
            }
        }
        before_closing_fence()?;
        revalidate_prepared_object_closing_identity_fence(
            &self.objects_root,
            self.root
                .as_ref()
                .expect("prepared object root remains bound"),
            &self.entries,
            ledger,
            &mut should_cancel,
        )?;
        self.stage_lock
            .as_ref()
            .expect("prepared object stage lock remains held")
            .ensure_current(&root.directory)?;
        self.revalidated = true;
        Ok(())
    }

    pub fn commit_after_seal(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger_identity(ledger)?;
        if !ledger.is_prepublication_sealed() || !self.revalidated {
            let error = StoreError::InvalidSealedCommit(
                "prepared object batch must be revalidated and sealed before commit".to_string(),
            );
            return match self.cancel_inner(ledger) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(cleanup),
            };
        }
        let committed = self.commit_inner(ledger);
        match committed {
            Ok(()) => Ok(()),
            Err(error) => match self.cancel_inner(ledger) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(cleanup),
            },
        }
    }

    fn commit_inner(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        let mut permit = self.permit.take();
        let mut remaining_records = self.reserved_records;
        for entry in &mut self.entries {
            {
                let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                    continue;
                };
                let mut current_permit = permit
                    .take()
                    .expect("missing publication entries own a prepared permit");
                let actual = entry
                    .publication_work
                    .take()
                    .expect("missing publication entries own prepared work");
                if let Err(error) = ledger.consume_reserved(&mut current_permit, actual) {
                    self.permit = Some(current_permit);
                    return Err(error.into());
                }
                debug_assert!(remaining_records > 0);
                remaining_records -= 1;
                if remaining_records > 0 {
                    permit = Some(current_permit);
                }
                let effect = (|| {
                    match entry
                        .directory
                        .hard_link(stage_name, &entry.directory, &entry.file_name)
                    {
                        Ok(()) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                            return Err(StoreError::ObjectPublicationRetryRequired);
                        }
                        Err(error) => return Err(StoreError::Io(error)),
                    }
                    entry.directory.remove_file(stage_name)?;
                    sync_capability_directory_best_effort(&entry.directory)
                })();
                if let Err(error) = effect {
                    self.permit = permit;
                    return Err(error);
                }
            }
            entry.state = PreparedObjectRecordState::Published;
        }
        if let Some(unused) = permit {
            if let Err(error) = ledger.cancel_reserved(&unused) {
                self.permit = Some(unused);
                return Err(error.into());
            }
        }
        self.finish_ownership(ledger)?;
        Ok(())
    }

    pub fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger_identity(ledger)?;
        self.cancel_inner(ledger)
    }

    /// Releases a sealed durable-retry owner's accounting without requiring
    /// private stage cleanup to succeed. Unpublished `.tmp` stages are left in
    /// their non-authoritative namespace for a later bounded recovery pass;
    /// canonical `.json` records are never removed.
    ///
    /// This is deliberately narrower than [`Self::cancel`]. It may only move
    /// an active batch after the combined ledger is sealed. Once its permit is
    /// cancelled, payload and capability ownership is dropped before retained
    /// memory is released. If that final release fails, another call may retry
    /// it using the retained scalar without recovering the discarded payloads.
    pub fn abandon_for_durable_retry(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.abandon_for_durable_retry_with_release(ledger, |ledger, retained_bytes| {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )
                .map_err(StoreError::from)
        })
    }

    pub fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        ledger.matches_identity(self.ledger_identity)
    }

    fn ensure_ledger_identity(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "prepared object batch belongs to a different decision-work ledger".to_string(),
            ))
        }
    }

    fn cancel_inner(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.cancel_inner_with_remove(ledger, remove_prepared_stage)
    }

    fn cancel_inner_with_remove<R>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut remove_stage: R,
    ) -> Result<(), StoreError>
    where
        R: FnMut(&mut PreparedObjectRecordEntry) -> Result<(), StoreError>,
    {
        if !self.active {
            return Ok(());
        }
        let mut first_error = None;
        for entry in &mut self.entries {
            if let Err(error) = remove_stage(entry) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        if let Some(permit) = self.permit.take() {
            if let Err(error) = ledger.cancel_reserved(&permit).map_err(StoreError::from) {
                self.permit = Some(permit);
                return Err(error);
            }
        }
        self.drop_retained_payloads();
        if self.retained_bytes > 0 {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    self.retained_bytes,
                )
                .map_err(StoreError::from)?;
        }
        self.retained_bytes = 0;
        self.active = false;
        Ok(())
    }

    fn abandon_for_durable_retry_with_release<R>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        release_retained: R,
    ) -> Result<(), StoreError>
    where
        R: FnOnce(&mut codefire_core::DecisionWorkLedger, u64) -> Result<(), StoreError>,
    {
        self.ensure_ledger_identity(ledger)?;
        if !self.active {
            return Ok(());
        }
        if !ledger.is_prepublication_sealed() {
            return Err(StoreError::InvalidSealedCommit(
                "prepared object batch can only be abandoned for retry after prepublication sealing"
                    .to_string(),
            ));
        }

        if let Some(permit) = self.permit.take() {
            if let Err(error) = ledger.cancel_reserved(&permit).map_err(StoreError::from) {
                self.permit = Some(permit);
                return Err(error);
            }
        }

        // Dropping entries intentionally does not unlink their private stage
        // names. It destroys every payload and directory capability before
        // releasing the memory accounting that represents them.
        self.drop_retained_payloads();
        self.reserved_records = 0;
        self.reserved_bytes = 0;
        self.revalidated = false;
        if self.retained_bytes > 0 {
            release_retained(ledger, self.retained_bytes)?;
        }
        self.retained_bytes = 0;
        self.active = false;
        Ok(())
    }

    fn finish_ownership(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.drop_retained_payloads();
        if self.retained_bytes > 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )?;
        }
        self.retained_bytes = 0;
        self.active = false;
        Ok(())
    }

    fn drop_retained_payloads(&mut self) {
        drop(self.pending_documents.take());
        drop(std::mem::take(&mut self.entries));
        drop(self.root.take());
        drop(self.stage_lock.take());
        drop(std::mem::take(&mut self.objects_root));
    }
}

impl Drop for PreparedObjectRecordBatch {
    fn drop(&mut self) {
        for entry in &mut self.entries {
            let removed = remove_prepared_stage(entry);
            debug_assert!(removed.is_ok());
        }
        debug_assert!(
            !self.active,
            "prepared object batch dropped without its ledger owner cancelling it"
        );
    }
}

fn prepared_object_record_path(
    objects_root: &Path,
    record: &ObjectRecord,
) -> Result<PathBuf, StoreError> {
    let subdir = object_subdir(&record.type_tag)
        .ok_or_else(|| StoreError::UnknownObjectType(record.type_tag.clone()))?;
    let file_name = PathBuf::from(format!("{}.json", record.object_id));
    build_prepared_object_path(objects_root, subdir, &file_name)
}

fn unique_prepared_object_stage_name(file_name: &Path) -> Result<PathBuf, StoreError> {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = file_name.to_str().ok_or_else(|| {
        StoreError::IncompleteRecord("prepared object filename is not UTF-8".to_string())
    })?;
    let object_id = file_name.strip_suffix(".json").ok_or_else(|| {
        StoreError::InvalidSealedCommit(
            "prepared object stage destination must use the .json namespace".to_string(),
        )
    })?;
    Ok(PathBuf::from(format!(
        "{PREPARED_STAGE_V1_PREFIX}{object_id}.{}.{counter}{PREPARED_STAGE_SUFFIX}",
        std::process::id()
    )))
}

fn acquire_prepared_object_stage_lock<C>(
    root: &Dir,
    should_cancel: &mut C,
) -> Result<PreparedObjectStageLock, StoreError>
where
    C: FnMut() -> bool,
{
    let mut options = CapabilityOpenOptions::new();
    options.read(true).write(true).create(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    #[cfg(unix)]
    options.mode(0o600);
    let capability_file = root.open_with(PREPARED_STAGE_LOCK_NAME, &options)?;
    let metadata = capability_file.metadata()?;
    if !metadata.is_file() || !prepared_stage_has_one_link(&metadata) {
        return Err(StoreError::InvalidSealedCommit(
            "prepared object stage lock must be a single-link regular file".to_string(),
        ));
    }
    let identity = stable_capability_object_store_identity(&metadata)?;
    let file = capability_file.into_std();
    loop {
        poll_prepared_object_batch(should_cancel)?;
        match Fs2FileExt::try_lock_exclusive(&file) {
            Ok(()) => break,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(PREPARED_STAGE_LOCK_RETRY);
            }
            Err(error) => return Err(StoreError::Io(error)),
        }
    }
    let current = root.symlink_metadata(PREPARED_STAGE_LOCK_NAME)?;
    if !current.is_file()
        || !prepared_stage_has_one_link(&current)
        || stable_capability_object_store_identity(&current)? != identity
    {
        let _ = Fs2FileExt::unlock(&file);
        return Err(StoreError::InvalidSealedCommit(
            "prepared object stage lock identity changed while acquiring it".to_string(),
        ));
    }
    Ok(PreparedObjectStageLock { file, identity })
}

#[cfg(unix)]
fn prepared_stage_has_one_link(metadata: &cap_std::fs::Metadata) -> bool {
    use cap_std::fs::MetadataExt;
    metadata.nlink() == 1
}

#[cfg(windows)]
fn prepared_stage_has_one_link(metadata: &cap_std::fs::Metadata) -> bool {
    use cap_std::fs::MetadataExt;
    metadata.number_of_links() == Some(1)
}

#[cfg(not(any(unix, windows)))]
fn prepared_stage_has_one_link(_metadata: &cap_std::fs::Metadata) -> bool {
    false
}

fn prepared_stage_v1_object_id(name: &Path) -> Option<&str> {
    let name = name.to_str()?;
    let body = name
        .strip_prefix(PREPARED_STAGE_V1_PREFIX)?
        .strip_suffix(PREPARED_STAGE_SUFFIX)?;
    let (object_and_pid, counter) = body.rsplit_once('.')?;
    let (object_id, pid) = object_and_pid.rsplit_once('.')?;
    if object_id.is_empty() || pid.parse::<u32>().is_err() || counter.parse::<u64>().is_err() {
        return None;
    }
    Some(object_id)
}

fn recover_prepared_object_stages<C, H>(
    entries: &[PreparedObjectRecordEntry],
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
    before_unlink: H,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
    H: FnOnce() -> Result<(), StoreError>,
{
    let mut candidates = BTreeMap::<(usize, PathBuf), (ObjectStoreIdentity, u64)>::new();
    let mut candidate_retained_bytes = 0_u64;
    let recovery = (|| -> Result<(), StoreError> {
        let mut visited = [false; OBJECT_KINDS.len()];
        for (entry_index, entry) in entries.iter().enumerate() {
            let kind_index = OBJECT_KINDS
                .iter()
                .position(|kind| kind.subdir == entry.subdir)
                .ok_or_else(|| StoreError::UnknownObjectType(entry.subdir.to_string()))?;
            if std::mem::replace(&mut visited[kind_index], true) {
                continue;
            }
            let (names, mut names_retained_bytes) = collect_object_store_scan_names(
                &entry.directory,
                ledger,
                should_cancel,
                codefire_core::DecisionWorkPhase::Staging,
            )?;
            let mut names = names.into_iter();
            while let Some((name, frame_bytes)) = names.next() {
                names_retained_bytes = names_retained_bytes
                    .checked_sub(frame_bytes)
                    .ok_or_else(object_store_scan_accounting_underflow)?;
                let inspected =
                    inspect_prepared_stage_recovery_candidate(entry, &name, ledger, should_cancel);
                let inspected = match inspected {
                    Ok(inspected) => inspected,
                    Err(error) => {
                        drop(name);
                        release_object_store_scan_retained(ledger, frame_bytes)?;
                        drop(names);
                        release_object_store_scan_retained(ledger, names_retained_bytes)?;
                        return Err(error);
                    }
                };
                let Some(identity) = inspected else {
                    drop(name);
                    release_object_store_scan_retained(ledger, frame_bytes)?;
                    continue;
                };
                let Some(next_candidate_bytes) = candidate_retained_bytes.checked_add(frame_bytes)
                else {
                    drop(name);
                    release_object_store_scan_retained(ledger, frame_bytes)?;
                    drop(names);
                    release_object_store_scan_retained(ledger, names_retained_bytes)?;
                    return Err(prepared_object_batch_size_overflow());
                };
                candidate_retained_bytes = next_candidate_bytes;
                candidates.insert((entry_index, name), (identity, frame_bytes));
            }
            release_object_store_scan_retained(ledger, names_retained_bytes)?;
        }

        poll_prepared_object_batch(should_cancel)?;
        before_unlink()?;
        poll_prepared_object_batch(should_cancel)?;
        for ((entry_index, name), (expected_identity, _)) in &candidates {
            poll_prepared_object_batch(should_cancel)?;
            let entry = &entries[*entry_index];
            let metadata = match entry.directory.symlink_metadata(name) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(StoreError::Io(error)),
            };
            if !metadata.is_file()
                || !prepared_stage_has_one_link(&metadata)
                || stable_capability_object_store_identity(&metadata)? != *expected_identity
            {
                continue;
            }
            match entry.directory.remove_file(name) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(StoreError::Io(error)),
            }
        }
        Ok(())
    })();
    drop(candidates);
    let release = release_object_store_scan_retained(ledger, candidate_retained_bytes);
    match (recovery, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn inspect_prepared_stage_recovery_candidate<C>(
    entry: &PreparedObjectRecordEntry,
    name: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<Option<ObjectStoreIdentity>, StoreError>
where
    C: FnMut() -> bool,
{
    let Some(name_text) = name.to_str() else {
        return Ok(None);
    };
    if !name_text.starts_with(PREPARED_STAGE_PREFIX) {
        return Ok(None);
    }
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Staging,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
        .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
    )?;
    let Some(object_id) = prepared_stage_v1_object_id(name) else {
        return Ok(None);
    };
    if object_subdir_by_id(object_id) != Some(entry.subdir) {
        return Ok(None);
    }
    let metadata = match entry.directory.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    if !metadata.is_file()
        || !prepared_stage_has_one_link(&metadata)
        || metadata.len() > MAX_OBJECT_RECORD_BYTES
    {
        return Ok(None);
    }
    let identity = stable_capability_object_store_identity(&metadata)?;
    let validation = with_accounted_prepared_object_read(
        &entry.directory,
        name,
        &entry.display_path,
        ledger,
        PreparedObjectReadKind::Initial,
        should_cancel,
        |bytes| {
            let document = parse_object_record_document(bytes)?;
            validate_object_extension_support(&document.extensions)?;
            validate_object_record(&document.record, Some(object_id), None)?;
            Ok(())
        },
    );
    match validation {
        Ok(()) => Ok(Some(identity)),
        Err(error)
            if matches!(
                error,
                StoreError::DecisionWorkAccounting(_) | StoreError::ValidationCancelled
            ) =>
        {
            Err(error)
        }
        Err(_) => Ok(None),
    }
}

fn build_prepared_object_path(
    objects_root: &Path,
    subdir: &str,
    file_name: &Path,
) -> Result<PathBuf, StoreError> {
    let capacity = [
        objects_root.as_os_str(),
        subdir.as_ref(),
        file_name.as_os_str(),
    ]
    .into_iter()
    .try_fold(2_usize, |total, component: &std::ffi::OsStr| {
        total
            .checked_add(component.as_encoded_bytes().len())
            .ok_or_else(prepared_object_batch_size_overflow)
    })?;
    let mut path = PathBuf::with_capacity(capacity);
    path.push(objects_root);
    path.push(subdir);
    path.push(file_name);
    Ok(path)
}

fn prepared_object_batch_retained_upper<C>(
    objects_root: &Path,
    documents: &[ObjectRecordDocument],
    should_cancel: &mut C,
) -> Result<(u64, u64), StoreError>
where
    C: FnMut() -> bool,
{
    let root_bytes = object_store_scan_name_frame_bytes(objects_root)?;
    let transition_bytes = u64::try_from(documents.len())
        .ok()
        .and_then(|count| count.checked_mul(codefire_core::LOGICAL_FRAME_BASE_BYTES))
        .ok_or_else(prepared_object_batch_size_overflow)?;
    let mut total = root_bytes
        .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES)
        .ok_or_else(prepared_object_batch_size_overflow)?
        .checked_add(transition_bytes)
        .ok_or_else(prepared_object_batch_size_overflow)?;
    for document in documents {
        poll_prepared_object_batch(should_cancel)?;
        let subdir = object_subdir(&document.record.type_tag)
            .ok_or_else(|| StoreError::UnknownObjectType(document.record.type_tag.clone()))?;
        let compact =
            compact_object_record_document_size_with_cancellation(document, should_cancel)?;
        let rendered =
            rendered_object_record_document_size_with_cancellation(document, should_cancel)?;
        let file_name_bytes = u64::try_from(document.record.object_id.len())
            .ok()
            .and_then(|bytes| bytes.checked_add(5))
            .ok_or_else(prepared_object_batch_size_overflow)?;
        let display_path_bytes = u64::try_from(objects_root.as_os_str().as_encoded_bytes().len())
            .ok()
            .and_then(|bytes| bytes.checked_add(subdir.len() as u64))
            .and_then(|bytes| bytes.checked_add(file_name_bytes))
            .and_then(|bytes| bytes.checked_add(2))
            .ok_or_else(prepared_object_batch_size_overflow)?;
        let entry_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
            .checked_add(compact)
            .and_then(|bytes| bytes.checked_add(rendered))
            .and_then(|bytes| bytes.checked_add(file_name_bytes))
            .and_then(|bytes| bytes.checked_add(display_path_bytes))
            .and_then(|bytes| bytes.checked_add(PREPARED_OBJECT_STATE_PREALLOCATION_BYTES))
            .ok_or_else(prepared_object_batch_size_overflow)?;
        total = total
            .checked_add(entry_bytes)
            .ok_or_else(prepared_object_batch_size_overflow)?;
    }
    Ok((total, transition_bytes))
}

fn prepared_object_batch_size_overflow() -> StoreError {
    StoreError::InvalidSealedCommit("prepared object batch size overflow".to_string())
}

fn prepared_object_publication_work(
    records: u64,
    bytes: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::PublicationPreflight,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(codefire_core::DecisionWorkDimension::ObjectRecords, records)
    .charge(codefire_core::DecisionWorkDimension::AggregateBytes, bytes)
    .count(codefire_core::DecisionWorkCounter::ObjectRecords, records)
}

fn poll_prepared_object_batch<C>(should_cancel: &mut C) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    if should_cancel() {
        Err(StoreError::ValidationCancelled)
    } else {
        Ok(())
    }
}

fn rendered_object_record_document_size_with_cancellation<C>(
    document: &ObjectRecordDocument,
    should_cancel: &mut C,
) -> Result<u64, StoreError>
where
    C: FnMut() -> bool,
{
    let mut counter = CancellableObjectRecordJsonByteCounter::new(should_cancel);
    let result = serde_json::to_writer_pretty(
        &mut counter,
        &SerializableObjectRecordDocument {
            record: &document.record,
            extensions: &document.extensions,
        },
    );
    if counter.cancelled {
        return Err(StoreError::ValidationCancelled);
    }
    result?;
    poll_prepared_object_batch(counter.should_cancel)?;
    counter
        .bytes
        .checked_add(1)
        .ok_or_else(prepared_object_batch_size_overflow)
}

struct CancellableObjectRecordJsonBuffer<'a, C> {
    bytes: Vec<u8>,
    next_poll_at: u64,
    cancelled: bool,
    should_cancel: &'a mut C,
}

impl<C> Write for CancellableObjectRecordJsonBuffer<'_, C>
where
    C: FnMut() -> bool,
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len() as u64 >= self.next_poll_at {
            if (self.should_cancel)() {
                self.cancelled = true;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "prepared object serialization cancelled",
                ));
            }
            self.next_poll_at =
                (self.bytes.len() as u64).saturating_add(OBJECT_STORE_TRAVERSAL_POLL_BYTES as u64);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serialize_prepared_object_document<C>(
    document: &ObjectRecordDocument,
    path: &Path,
    expected_size: u64,
    should_cancel: &mut C,
) -> Result<Vec<u8>, StoreError>
where
    C: FnMut() -> bool,
{
    let capacity =
        usize::try_from(expected_size).map_err(|_| StoreError::ObjectRecordTooLarge {
            path: path.to_path_buf(),
            size: expected_size,
            limit: MAX_OBJECT_RECORD_BYTES,
        })?;
    let mut writer = CancellableObjectRecordJsonBuffer {
        bytes: Vec::with_capacity(capacity),
        next_poll_at: 0,
        cancelled: false,
        should_cancel,
    };
    let result = serde_json::to_writer_pretty(
        &mut writer,
        &SerializableObjectRecordDocument {
            record: &document.record,
            extensions: &document.extensions,
        },
    );
    if writer.cancelled {
        return Err(StoreError::ValidationCancelled);
    }
    result?;
    poll_prepared_object_batch(writer.should_cancel)?;
    writer.bytes.push(b'\n');
    if writer.bytes.len() as u64 != expected_size {
        return Err(StoreError::InvalidSealedCommit(format!(
            "prepared object encoding changed between count and materialization: {}",
            path.display()
        )));
    }
    Ok(writer.bytes)
}

#[derive(Clone, Copy)]
enum PreparedObjectReadKind {
    Initial,
    Revalidation,
}

fn revalidate_prepared_object_closing_identity_fence<C>(
    objects_root: &Path,
    pinned_root: &PreparedObjectStoreRoot,
    entries: &[PreparedObjectRecordEntry],
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    poll_prepared_object_batch(should_cancel)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1),
    )?;
    let ambient_root = open_prepared_object_root_accounted(
        objects_root,
        ledger,
        PreparedObjectReadKind::Revalidation,
    )
    .map_err(prepared_object_revalidation_error)?;
    if stable_capability_object_store_identity(&pinned_root.directory.dir_metadata()?)?
        != pinned_root.identity
        || stable_capability_object_store_identity(&ambient_root.dir_metadata()?)?
            != pinned_root.identity
    {
        return Err(prepared_object_subject_changed_error());
    }

    for entry in entries {
        poll_prepared_object_batch(should_cancel)?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
            .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1),
        )?;
        let ambient_subdir = ambient_root
            .open_dir_nofollow(entry.subdir)
            .map_err(|_| prepared_object_subject_changed_error())?;
        if stable_capability_object_store_identity(&entry.directory.dir_metadata()?)?
            != entry.directory_identity
            || stable_capability_object_store_identity(&ambient_subdir.dir_metadata()?)?
                != entry.directory_identity
        {
            return Err(prepared_object_subject_changed_error());
        }
    }
    poll_prepared_object_batch(should_cancel)
}

fn open_prepared_object_root_accounted(
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    kind: PreparedObjectReadKind,
) -> Result<Dir, StoreError> {
    let scratch_bytes = path.components().try_fold(
        codefire_core::LOGICAL_FRAME_BASE_BYTES,
        |total, component| {
            let component_bytes = u64::try_from(component.as_os_str().as_encoded_bytes().len())
                .map_err(|_| prepared_object_batch_size_overflow())?;
            total
                .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES)
                .and_then(|bytes| bytes.checked_add(component_bytes))
                .ok_or_else(prepared_object_batch_size_overflow)
        },
    )?;
    let phase = match kind {
        PreparedObjectReadKind::Initial => codefire_core::DecisionWorkPhase::Staging,
        PreparedObjectReadKind::Revalidation => {
            codefire_core::DecisionWorkPhase::SnapshotRevalidation
        }
    };
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        ),
    )?;
    let opened = open_absolute_directory_nofollow(path);
    let release = ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )
        .map_err(StoreError::from);
    match (opened, release) {
        (Ok(directory), Ok(())) => Ok(directory),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn prepared_object_read_reservation(
    kind: PreparedObjectReadKind,
    size_bytes: u64,
    scratch_bytes: u64,
) -> codefire_core::DecisionWorkReservation {
    let phase = match kind {
        PreparedObjectReadKind::Initial => codefire_core::DecisionWorkPhase::RecordRead,
        PreparedObjectReadKind::Revalidation => {
            codefire_core::DecisionWorkPhase::SnapshotRevalidation
        }
    };
    let mut reservation = codefire_core::DecisionWorkReservation::new(
        phase,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .charge(
        codefire_core::DecisionWorkDimension::AggregateBytes,
        size_bytes,
    )
    .acquire(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        scratch_bytes,
    )
    .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
    .count(
        codefire_core::DecisionWorkCounter::FileBytesRead,
        size_bytes,
    )
    .count(codefire_core::DecisionWorkCounter::BytesHashed, size_bytes)
    .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1);
    if matches!(kind, PreparedObjectReadKind::Revalidation) {
        reservation = reservation
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                1,
            )
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                size_bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
                1,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
                size_bytes,
            );
    }
    reservation
}

fn with_accounted_prepared_object_read<T, C, F>(
    directory: &Dir,
    file_name: &Path,
    display_path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    kind: PreparedObjectReadKind,
    should_cancel: &mut C,
    consume: F,
) -> Result<T, StoreError>
where
    C: FnMut() -> bool,
    F: FnOnce(&[u8]) -> Result<T, StoreError>,
{
    let mut scratch_bytes = 0_u64;
    let read = read_object_record_bytes_from_directory_with_limit_and_hooks(
        directory,
        file_name,
        display_path,
        MAX_OBJECT_RECORD_BYTES,
        |size_bytes| {
            scratch_bytes = size_bytes
                .checked_mul(2)
                .ok_or_else(prepared_object_batch_size_overflow)?;
            ledger.apply(prepared_object_read_reservation(
                kind,
                size_bytes,
                scratch_bytes,
            ))?;
            Ok(())
        },
        || poll_prepared_object_batch(should_cancel),
    );
    let result = match read {
        Ok(bytes) => consume(&bytes),
        Err(error) => Err(error),
    };
    let release = if scratch_bytes > 0 {
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch_bytes,
            )
            .map_err(StoreError::from)
    } else {
        Ok(())
    };
    match (result, release) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn read_and_validate_prepared_existing<C>(
    directory: &Dir,
    file_name: &Path,
    display_path: &Path,
    requested: &ObjectRecordDocument,
    ledger: &mut codefire_core::DecisionWorkLedger,
    kind: PreparedObjectReadKind,
    should_cancel: &mut C,
) -> Result<PreparedExistingObjectRecord, StoreError>
where
    C: FnMut() -> bool,
{
    with_accounted_prepared_object_read(
        directory,
        file_name,
        display_path,
        ledger,
        kind,
        should_cancel,
        |bytes| {
            let existing = parse_object_record_document(bytes)?;
            validate_existing_document_for_reuse(
                requested,
                &existing,
                display_path,
                &object_digest,
            )?;
            let size_bytes = bytes
                .len()
                .try_into()
                .map_err(|_| prepared_object_batch_size_overflow())?;
            Ok(PreparedExistingObjectRecord {
                size_bytes,
                document_digest: codefire_util::sha256_hex(bytes),
            })
        },
    )
}

fn create_and_validate_prepared_stage<C>(
    entry: &mut PreparedObjectRecordEntry,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    let PreparedObjectRecordState::Missing { .. } = &entry.state else {
        return Ok(());
    };
    let size_bytes =
        u64::try_from(entry.rendered.len()).map_err(|_| prepared_object_batch_size_overflow())?;
    let scratch_bytes = size_bytes
        .checked_mul(2)
        .ok_or_else(prepared_object_batch_size_overflow)?;
    ledger.apply(prepared_object_read_reservation(
        PreparedObjectReadKind::Initial,
        size_bytes,
        scratch_bytes,
    ))?;

    let result = (|| {
        let mut options = CapabilityOpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = {
            let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                unreachable!("stage state was checked")
            };
            entry.directory.open_with(stage_name, &options)?
        };
        let PreparedObjectRecordState::Missing { stage_owned, .. } = &mut entry.state else {
            unreachable!("stage state was checked")
        };
        *stage_owned = true;
        for chunk in entry.rendered.chunks(OBJECT_STORE_TRAVERSAL_POLL_BYTES) {
            poll_prepared_object_batch(should_cancel)?;
            file.write_all(chunk)?;
        }
        file.sync_all()?;
        drop(file);

        let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
            unreachable!("stage state was checked")
        };
        let bytes = read_object_record_bytes_from_directory_with_limit_and_hooks(
            &entry.directory,
            stage_name,
            &entry.display_path,
            MAX_OBJECT_RECORD_BYTES,
            |actual_size| {
                if actual_size == size_bytes {
                    Ok(())
                } else {
                    Err(prepared_object_subject_changed_error())
                }
            },
            || poll_prepared_object_batch(should_cancel),
        )?;
        if bytes != entry.rendered {
            return Err(prepared_object_subject_changed_error());
        }
        let staged = parse_object_record_document(&bytes)?;
        if staged != entry.document {
            return Err(prepared_object_subject_changed_error());
        }
        Ok(())
    })();
    let release = ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )
        .map_err(StoreError::from);
    match (result, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn validate_prepared_stage<C>(
    entry: &PreparedObjectRecordEntry,
    ledger: &mut codefire_core::DecisionWorkLedger,
    kind: PreparedObjectReadKind,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    let PreparedObjectRecordState::Missing {
        stage_name,
        stage_owned: true,
    } = &entry.state
    else {
        return Ok(());
    };
    with_accounted_prepared_object_read(
        &entry.directory,
        stage_name,
        &entry.display_path,
        ledger,
        kind,
        should_cancel,
        |bytes| {
            if bytes != entry.rendered {
                return Err(prepared_object_subject_changed_error());
            }
            let staged = parse_object_record_document(bytes)?;
            if staged != entry.document {
                return Err(prepared_object_subject_changed_error());
            }
            Ok(())
        },
    )
}

fn remove_prepared_stage(entry: &mut PreparedObjectRecordEntry) -> Result<(), StoreError> {
    let PreparedObjectRecordState::Missing {
        stage_name,
        stage_owned: true,
    } = &entry.state
    else {
        return Ok(());
    };
    match entry.directory.remove_file(stage_name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(StoreError::Io(error)),
    }
    let PreparedObjectRecordState::Missing { stage_owned, .. } = &mut entry.state else {
        unreachable!("stage state was checked")
    };
    *stage_owned = false;
    Ok(())
}

fn prepared_existing_drift_error(
    _entry: &PreparedObjectRecordEntry,
    expected: &PreparedExistingObjectRecord,
    error: StoreError,
) -> StoreError {
    match error {
        StoreError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let _ = expected;
            prepared_object_subject_changed_error()
        }
        other => other,
    }
}

fn prepared_object_subject_changed_error() -> StoreError {
    StoreError::Io(std::io::ErrorKind::InvalidData.into())
}

fn prepared_object_revalidation_error(error: StoreError) -> StoreError {
    match error {
        StoreError::DecisionWorkAccounting(_) | StoreError::ValidationCancelled => error,
        _ => prepared_object_subject_changed_error(),
    }
}

fn store_object_with_identity_and_link<D, L>(
    objects_root: &Path,
    type_tag: &str,
    payload: Value,
    digest: &D,
    link: L,
) -> Result<String, StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
    L: FnOnce(&Dir, &Path, &Path, &Path) -> std::io::Result<()>,
{
    let record = object_record_with_digest(type_tag, payload, digest)?;
    store_object_record_with_identity_and_link(objects_root, record, digest, link)
}

fn store_object_record_with_identity_and_link<D, L>(
    objects_root: &Path,
    record: ObjectRecord,
    digest: &D,
    link: L,
) -> Result<String, StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
    L: FnOnce(&Dir, &Path, &Path, &Path) -> std::io::Result<()>,
{
    store_object_record_document_with_identity_and_link(
        objects_root,
        ObjectRecordDocument {
            record,
            extensions: Map::new(),
        },
        digest,
        link,
    )
}

fn store_object_record_document_with_identity_and_link<D, L>(
    objects_root: &Path,
    document: ObjectRecordDocument,
    digest: &D,
    link: L,
) -> Result<String, StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
    L: FnOnce(&Dir, &Path, &Path, &Path) -> std::io::Result<()>,
{
    validate_object_extension_support(&document.extensions)?;
    let record = &document.record;
    let subdir = object_subdir(&record.type_tag)
        .ok_or_else(|| StoreError::UnknownObjectType(record.type_tag.clone()))?;
    validate_object_record_with_digest(record, Some(&record.object_id), None, digest)?;
    let path = objects_root
        .join(subdir)
        .join(format!("{}.json", record.object_id));
    let objects_directory = open_or_create_absolute_directory_nofollow(objects_root)?;
    let objects_identity =
        stable_capability_object_store_identity(&objects_directory.dir_metadata()?)?;
    let directory = open_or_create_directory_nofollow(&objects_directory, Path::new(subdir))?;
    let directory_identity = stable_capability_object_store_identity(&directory.dir_metadata()?)?;
    let file_name = PathBuf::from(format!("{}.json", record.object_id));

    match directory.symlink_metadata(&file_name) {
        Ok(_) => {
            let existing = read_object_record_document_from_directory_with_limit_and_hook(
                &directory,
                &file_name,
                &path,
                MAX_OBJECT_RECORD_BYTES,
                |_| Ok(()),
            )?;
            validate_existing_document_for_reuse(&document, &existing, &path, digest)?;
            ensure_object_publication_subject(
                objects_root,
                subdir,
                objects_identity,
                directory_identity,
            )?;
            return Ok(record.object_id.clone());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(StoreError::Io(error)),
    }

    let rendered =
        serialize_object_record_document_with_limit(&document, &path, MAX_OBJECT_RECORD_BYTES)?;
    let temp_path = unique_temp_path(&path);
    let temp_name = temp_path
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| StoreError::IncompleteRecord("object temp path has no name".to_string()))?;
    let mut options = CapabilityOpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = directory.open_with(&temp_name, &options)?;
    file.write_all(&rendered)?;
    file.sync_all()?;
    drop(file);
    let temp_document = read_object_record_document_from_directory_with_limit_and_hook(
        &directory,
        &temp_name,
        &temp_path,
        MAX_OBJECT_RECORD_BYTES,
        |_| Ok(()),
    )?;
    validate_object_record_with_digest(
        &temp_document.record,
        Some(&record.object_id),
        None,
        digest,
    )?;
    if temp_document.extensions != document.extensions {
        return Err(StoreError::InvalidSealedCommit(format!(
            "object extension fields changed during publication: {}",
            path.display()
        )));
    }
    match link(&directory, &temp_name, &file_name, &path) {
        Ok(()) => {
            directory.remove_file(&temp_name)?;
            sync_capability_directory_best_effort(&directory)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            directory.remove_file(&temp_name)?;
            let existing = read_object_record_document_from_directory_with_limit_and_hook(
                &directory,
                &file_name,
                &path,
                MAX_OBJECT_RECORD_BYTES,
                |_| Ok(()),
            )?;
            validate_existing_document_for_reuse(&document, &existing, &path, digest)?;
        }
        Err(error) => {
            let _ = directory.remove_file(&temp_name);
            return Err(StoreError::Io(error));
        }
    }
    ensure_object_publication_subject(objects_root, subdir, objects_identity, directory_identity)?;
    Ok(record.object_id.clone())
}

fn open_or_create_absolute_directory_nofollow(path: &Path) -> Result<Dir, StoreError> {
    let (root, components) = absolute_directory_components(path)?;
    let mut current = Dir::open_ambient_dir(&root, ambient_authority())?;
    for component in components {
        current = open_or_create_directory_nofollow(&current, &component)?;
    }
    Ok(current)
}

fn open_absolute_directory_nofollow(path: &Path) -> Result<Dir, StoreError> {
    let (root, components) = absolute_directory_components(path)?;
    let mut current = Dir::open_ambient_dir(&root, ambient_authority())?;
    for component in components {
        current = current.open_dir_nofollow(component)?;
    }
    Ok(current)
}

fn absolute_directory_components(path: &Path) -> Result<(PathBuf, Vec<PathBuf>), StoreError> {
    if !path.is_absolute() {
        return Err(StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("object store path must be absolute: {}", path.display()),
        )));
    }
    let mut root = PathBuf::new();
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => root.push(component.as_os_str()),
            Component::Normal(value) => components.push(PathBuf::from(value)),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "object store path contains parent traversal",
                )))
            }
        }
    }
    Ok((root, components))
}

fn open_or_create_directory_nofollow(parent: &Dir, name: &Path) -> Result<Dir, StoreError> {
    match parent.open_dir_nofollow(name) {
        Ok(directory) => Ok(directory),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match parent.create_dir(name) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(StoreError::Io(error)),
            }
            Ok(parent.open_dir_nofollow(name)?)
        }
        Err(error) => Err(StoreError::Io(error)),
    }
}

fn ensure_object_publication_subject(
    objects_root: &Path,
    subdir: &str,
    expected_objects: ObjectStoreIdentity,
    expected_subdir: ObjectStoreIdentity,
) -> Result<(), StoreError> {
    let current_objects = open_or_create_absolute_directory_nofollow(objects_root)
        .map_err(|_| object_store_publication_identity_changed(objects_root))?;
    let current_objects_identity =
        stable_capability_object_store_identity(&current_objects.dir_metadata()?)?;
    if current_objects_identity != expected_objects {
        return Err(object_store_publication_identity_changed(objects_root));
    }
    let subdir_path = objects_root.join(subdir);
    let current_subdir = current_objects
        .open_dir_nofollow(subdir)
        .map_err(|_| object_store_publication_identity_changed(&subdir_path))?;
    let current_subdir_identity =
        stable_capability_object_store_identity(&current_subdir.dir_metadata()?)?;
    if current_subdir_identity != expected_subdir {
        return Err(object_store_publication_identity_changed(&subdir_path));
    }
    Ok(())
}

fn object_store_publication_identity_changed(path: &Path) -> StoreError {
    StoreError::InvalidSealedCommit(format!(
        "object store identity changed during object publication: {}",
        path.display()
    ))
}

fn sync_capability_directory_best_effort(directory: &Dir) -> Result<(), StoreError> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(StoreError::Io(error)),
    }
}

#[cfg(test)]
fn serialize_object_record_with_limit(
    record: &ObjectRecord,
    path: &Path,
    limit: u64,
) -> Result<Vec<u8>, StoreError> {
    serialize_object_record_document_with_limit(
        &ObjectRecordDocument {
            record: record.clone(),
            extensions: Map::new(),
        },
        path,
        limit,
    )
}

fn serialize_object_record_document_with_limit(
    document: &ObjectRecordDocument,
    path: &Path,
    limit: u64,
) -> Result<Vec<u8>, StoreError> {
    let mut rendered = serde_json::to_vec_pretty(&object_record_document_value(document)?)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > limit {
        return Err(StoreError::ObjectRecordTooLarge {
            path: path.to_path_buf(),
            size: rendered.len() as u64,
            limit,
        });
    }
    Ok(rendered)
}

fn validate_existing_document_for_reuse<D>(
    requested: &ObjectRecordDocument,
    existing: &ObjectRecordDocument,
    path: &Path,
    digest: &D,
) -> Result<(), StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
{
    validate_existing_record_for_reuse(&requested.record, &existing.record, path, digest)?;
    if requested.extensions != existing.extensions {
        return Err(StoreError::InvalidSealedCommit(format!(
            "immutable object extension fields diverge at {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_existing_record_for_reuse<D>(
    requested: &ObjectRecord,
    existing: &ObjectRecord,
    path: &Path,
    digest: &D,
) -> Result<(), StoreError>
where
    D: Fn(&str, &Value) -> Result<String, serde_json::Error>,
{
    // A malformed immutable record is repository corruption, not a prefix collision. Establish
    // the existing record's own payload/hash/id/filename agreement before comparing identities.
    validate_object_record_with_digest(existing, None, Some(path), digest)?;
    require_same_object_identity(requested, existing)
}

fn require_same_object_identity(
    requested: &ObjectRecord,
    existing: &ObjectRecord,
) -> Result<(), StoreError> {
    let canonical_payload_matches =
        canonical_json(&requested.payload)? == canonical_json(&existing.payload)?;
    if requested.type_tag == existing.type_tag
        && requested.hash == existing.hash
        && canonical_payload_matches
    {
        return Ok(());
    }
    Err(StoreError::ObjectCollision {
        object_id: requested.object_id.clone(),
        requested_type: requested.type_tag.clone(),
        existing_type: existing.type_tag.clone(),
        requested_hash: requested.hash.clone(),
        existing_hash: existing.hash.clone(),
        canonical_payload_matches,
    })
}

pub fn read_object(objects_root: &Path, object_id: &str) -> Result<Value, StoreError> {
    Ok(read_object_record(objects_root, object_id)?.payload)
}

pub fn read_object_record(
    objects_root: &Path,
    object_id: &str,
) -> Result<ObjectRecord, StoreError> {
    let path = object_record_path(objects_root, object_id)
        .ok_or_else(|| StoreError::ObjectNotFound(object_id.to_string()))?;
    let record = read_object_record_path(&path)?;
    validate_object_record(&record, Some(object_id), Some(&path))?;
    Ok(record)
}

/// Opens and parses one record with the store's no-follow, regular-file, and
/// byte-limit policy. Callers that bypass `read_object_record` must still call
/// `validate_object_record` with their expected identity/path context.
pub fn read_object_record_path(path: &Path) -> Result<ObjectRecord, StoreError> {
    Ok(read_object_record_document_path(path)?.record)
}

/// Safely reads one record while retaining unknown top-level extension fields.
/// Known record fields are moved into `record`, so large payloads are not
/// cloned merely to inspect a small extension such as `remote` generation data.
pub fn read_object_record_document_path(path: &Path) -> Result<ObjectRecordDocument, StoreError> {
    let parent_path = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        StoreError::IncompleteRecord(format!(
            "object record path has no filename: {}",
            path.display()
        ))
    })?;
    let parent = open_absolute_directory_nofollow(parent_path)?;
    read_object_record_document_from_directory_with_limit_and_hook(
        &parent,
        Path::new(file_name),
        path,
        MAX_OBJECT_RECORD_BYTES,
        |_| Ok(()),
    )
}

/// Reads one exact path through a no-follow parent capability while charging
/// the opened file's fstat size before allocating its byte buffer. The
/// returned owner has already transferred the parsed document from Temporary
/// to RetainedBytes on the same ledger.
fn read_accounted_object_record_from_directory<C>(
    directory: &Dir,
    file_name: &Path,
    display_path: &Path,
    expected_path: Option<(ObjectStoreIdentity, u64)>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<AccountedObjectRecordDocument, StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(should_cancel)?;
    let initial_metadata = directory.symlink_metadata(file_name)?;
    if !initial_metadata.is_file() {
        return Err(StoreError::IncompleteRecord(format!(
            "object record must be a regular no-follow file: {}",
            display_path.display()
        )));
    }
    let initial_identity = stable_capability_object_store_identity(&initial_metadata)?;
    if expected_path.is_some_and(|expected| expected != (initial_identity, initial_metadata.len()))
    {
        return Err(StoreError::IncompleteRecord(format!(
            "object record path identity changed before read: {}",
            display_path.display()
        )));
    }
    poll_object_store_scan(should_cancel)?;
    let mut scratch_bytes = 0_u64;
    let bytes = (|| {
        let mut options = CapabilityOpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let file = directory.open_with(file_name, &options)?;
        let opened_metadata = file.metadata()?;
        poll_object_store_scan(should_cancel)?;
        if !opened_metadata.is_file() {
            return Err(StoreError::IncompleteRecord(format!(
                "object record must be a regular no-follow file: {}",
                display_path.display()
            )));
        }
        let opened_identity = stable_capability_object_store_identity(&opened_metadata)?;
        if opened_identity != initial_identity || opened_metadata.len() != initial_metadata.len() {
            return Err(StoreError::IncompleteRecord(format!(
                "object record path identity changed before open: {}",
                display_path.display()
            )));
        }
        if opened_metadata.len() > MAX_OBJECT_RECORD_BYTES {
            return Err(StoreError::ObjectRecordTooLarge {
                path: display_path.to_path_buf(),
                size: opened_metadata.len(),
                limit: MAX_OBJECT_RECORD_BYTES,
            });
        }
        let reserved_scratch = accounted_object_parse_scratch_bytes(opened_metadata.len())?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                opened_metadata.len(),
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                reserved_scratch,
            )
            .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
            .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                opened_metadata.len(),
            ),
        )?;
        scratch_bytes = reserved_scratch;
        let expected_size = opened_metadata.len();
        let initial_capacity = usize::try_from(expected_size)
            .unwrap_or(usize::MAX)
            .min(MAX_OBJECT_RECORD_BYTES as usize);
        let mut bytes = Vec::with_capacity(initial_capacity);
        let mut chunk = [0_u8; OBJECT_STORE_TRAVERSAL_POLL_BYTES];
        {
            let mut reader = (&file).take(expected_size);
            loop {
                poll_object_store_scan(should_cancel)?;
                let read = reader.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..read]);
            }
        }
        poll_object_store_scan(should_cancel)?;
        let mut growth_fence = [0_u8; 1];
        let grew_past_fstat = (&file).read(&mut growth_fence)? != 0;
        let final_opened_metadata = file.metadata()?;
        if grew_past_fstat
            || bytes.len() as u64 != expected_size
            || final_opened_metadata.len() != expected_size
            || stable_capability_object_store_identity(&final_opened_metadata)? != opened_identity
        {
            return Err(StoreError::IncompleteRecord(format!(
                "object record changed during read: {}",
                display_path.display()
            )));
        }
        Ok(bytes)
    })();
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(error) => {
            if scratch_bytes > 0 {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    scratch_bytes,
                )?;
            }
            return Err(error);
        }
    };
    let read_result = (|| {
        poll_object_store_scan(should_cancel)?;
        let final_metadata = directory.symlink_metadata(file_name)?;
        if !final_metadata.is_file()
            || final_metadata.len() != initial_metadata.len()
            || stable_capability_object_store_identity(&final_metadata)? != initial_identity
        {
            return Err(StoreError::IncompleteRecord(format!(
                "object record path identity changed during read: {}",
                display_path.display()
            )));
        }
        let document =
            parse_object_record_document_with_work_and_cancellation(&bytes, ledger, should_cancel)?;
        let retained_bytes = compact_object_record_document_size(&document)?
            .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "accounted object document ownership overflow".to_string(),
                )
            })?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )?;
        Ok((document, retained_bytes))
    })();
    drop(bytes);
    let release = ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )
        .map_err(StoreError::from);
    match (read_result, release) {
        (Ok((document, retained_bytes)), Ok(())) => Ok(AccountedObjectRecordDocument {
            document: Some(document),
            retained_bytes,
            ledger_identity: ledger.identity(),
            active: true,
        }),
        (Err(error), Ok(())) => Err(error),
        (Ok((document, retained_bytes)), Err(error)) => {
            drop(document);
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )?;
            Err(error)
        }
        (Err(_), Err(error)) => Err(error),
    }
}

pub fn read_object_record_document_path_with_work_and_cancellation<C>(
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut should_cancel: C,
) -> Result<AccountedObjectRecordDocument, StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(&mut should_cancel)?;
    let parent_path = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        StoreError::IncompleteRecord(format!(
            "object record path has no filename: {}",
            path.display()
        ))
    })?;
    let parent = open_absolute_directory_nofollow(parent_path)?;
    let initial_metadata = parent.symlink_metadata(file_name)?;
    if !initial_metadata.is_file() {
        return Err(StoreError::IncompleteRecord(format!(
            "object record must be a regular no-follow file: {}",
            path.display()
        )));
    }
    let initial_identity = stable_capability_object_store_identity(&initial_metadata)?;
    read_accounted_object_record_from_directory(
        &parent,
        Path::new(file_name),
        path,
        Some((initial_identity, initial_metadata.len())),
        ledger,
        &mut should_cancel,
    )
}

impl AccountedObjectStoreReader {
    pub fn begin_with_work_and_cancellation<C>(
        objects_root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
    ) -> Result<Self, StoreError>
    where
        C: FnMut() -> bool,
    {
        let root_bytes = object_store_scan_name_frame_bytes(objects_root)?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::GraphDiscovery,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                root_bytes,
            ),
        )?;
        let mut cache = SealedCommitValidationCache::default();
        let binding = bind_object_store_with_work_and_cancellation(
            objects_root.to_path_buf(),
            &mut cache,
            ledger,
            &mut should_cancel,
        );
        let (bound_store, binding_bytes) = match binding {
            Ok(binding) => binding,
            Err(error) => {
                drop(cache);
                ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    root_bytes,
                )?;
                return Err(error);
            }
        };
        Ok(Self {
            bound_store: Some(bound_store),
            cache,
            root_bytes,
            binding_bytes,
            ledger_identity: ledger.identity(),
            active: true,
        })
    }

    pub fn read_with_work_and_cancellation<C>(
        &mut self,
        object_id: &str,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
    ) -> Result<AccountedObjectRecordDocument, StoreError>
    where
        C: FnMut() -> bool,
    {
        self.ensure_ledger(ledger)?;
        if object_id.len() > SEALED_REFERENCE_DIAGNOSTIC_SAMPLE_BYTE_LIMIT {
            return Err(StoreError::InvalidSealedCommit(
                "object id exceeds the bounded diagnostic limit".to_string(),
            ));
        }
        let bound_store = self.bound_store.as_ref().ok_or_else(|| {
            StoreError::InvalidSealedCommit("accounted object-store reader is inactive".to_string())
        })?;
        let file_name_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
            .checked_add(
                object_id
                    .len()
                    .checked_add(5)
                    .ok_or_else(prepared_object_batch_size_overflow)? as u64,
            )
            .ok_or_else(prepared_object_batch_size_overflow)?;
        let diagnostic_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
            .checked_add(object_id.len() as u64)
            .ok_or_else(prepared_object_batch_size_overflow)?;
        let name_scratch_bytes = file_name_bytes
            .checked_add(diagnostic_bytes)
            .ok_or_else(prepared_object_batch_size_overflow)?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                name_scratch_bytes,
            ),
        )?;
        let result = (|| {
            let mut file_name = std::ffi::OsString::with_capacity(object_id.len() + 5);
            file_name.push(object_id);
            file_name.push(".json");
            let file_name = PathBuf::from(file_name);
            let direct = object_subdir_by_id(object_id).map(Path::new);
            let mut selected = None;
            for subject in &bound_store.subject.subdirs {
                if direct.is_some_and(|direct| direct != subject.name) {
                    continue;
                }
                poll_object_store_scan(&mut should_cancel)?;
                ledger.apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::GraphDiscovery,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1),
                )?;
                let Some(directory) = bound_store
                    .subdirs
                    .get(&subject.name)
                    .and_then(Option::as_ref)
                else {
                    if direct.is_some() {
                        break;
                    }
                    continue;
                };
                match directory.symlink_metadata(&file_name) {
                    Ok(metadata) if metadata.is_file() => {
                        selected = Some((
                            subject.name.as_path(),
                            directory,
                            stable_capability_object_store_identity(&metadata)?,
                            metadata.len(),
                        ));
                        break;
                    }
                    Ok(_) if direct.is_some() => break,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(StoreError::Io(error)),
                    Ok(_) => {}
                }
                if direct.is_some() {
                    break;
                }
            }
            let Some((subdir, directory, selected_identity, selected_size)) = selected else {
                return Err(StoreError::ObjectNotFound(object_id.to_string()));
            };
            let path_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
                .checked_add(
                    bound_store
                        .subject
                        .canonical_root
                        .as_os_str()
                        .as_encoded_bytes()
                        .len() as u64,
                )
                .and_then(|bytes| {
                    bytes.checked_add(subdir.as_os_str().as_encoded_bytes().len() as u64)
                })
                .and_then(|bytes| {
                    bytes.checked_add(file_name.as_os_str().as_encoded_bytes().len() as u64)
                })
                .and_then(|bytes| bytes.checked_add(2))
                .ok_or_else(prepared_object_batch_size_overflow)?;
            ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    path_bytes,
                ),
            )?;
            let path = build_prepared_object_path(
                &bound_store.subject.canonical_root,
                subdir.to_str().ok_or_else(|| {
                    StoreError::IncompleteRecord("object subdirectory is not UTF-8".to_string())
                })?,
                &file_name,
            )?;
            let read = read_accounted_object_record_from_directory(
                directory,
                &file_name,
                &path,
                Some((selected_identity, selected_size)),
                ledger,
                &mut should_cancel,
            );
            let read = match read {
                Ok(mut accounted) => match validate_object_record(
                    &accounted.document().record,
                    Some(object_id),
                    Some(&path),
                ) {
                    Ok(()) => Ok(accounted),
                    Err(error) => {
                        accounted.release(ledger)?;
                        Err(error)
                    }
                },
                Err(error) => Err(error),
            };
            drop(path);
            let release = ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    path_bytes,
                )
                .map_err(StoreError::from);
            match (read, release) {
                (Ok(accounted), Ok(())) => Ok(accounted),
                (Err(error), Ok(())) => Err(error),
                (Ok(mut accounted), Err(error)) => {
                    accounted.release(ledger)?;
                    Err(error)
                }
                (Err(_), Err(error)) => Err(error),
            }
        })();
        let release = ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                name_scratch_bytes,
            )
            .map_err(StoreError::from);
        match (result, release) {
            (Ok(accounted), Ok(())) => Ok(accounted),
            (Err(error), Ok(())) => Err(error),
            (Ok(mut accounted), Err(error)) => {
                accounted.release(ledger)?;
                Err(error)
            }
            (Err(_), Err(error)) => Err(error),
        }
    }

    pub fn finish_with_cancellation<C>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
    ) -> Result<(), StoreError>
    where
        C: FnMut() -> bool,
    {
        self.ensure_ledger(ledger)?;
        let closing = ensure_bound_object_store_identity_with_work_and_cancellation(
            self.bound_store.as_ref().ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "accounted object-store reader is inactive".to_string(),
                )
            })?,
            &self.cache,
            ledger,
            &mut should_cancel,
        );
        let release = self.release_ownership(ledger);
        match (closing, release) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) => Err(error),
            (_, Err(error)) => Err(error),
        }
    }

    pub fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger(ledger)?;
        if !self.active {
            return Ok(());
        }
        self.release_ownership(ledger)
    }

    pub fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        ledger.matches_identity(self.ledger_identity)
    }

    fn ensure_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> Result<(), StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(StoreError::InvalidSealedCommit(
                "accounted object-store reader belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn release_ownership(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        let payloads = (
            self.bound_store.take(),
            std::mem::take(&mut self.cache.pinned_object_stores),
        );
        drop(payloads);
        if self.binding_bytes != 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.binding_bytes,
            )?;
            self.binding_bytes = 0;
        }
        if self.root_bytes != 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.root_bytes,
            )?;
            self.root_bytes = 0;
        }
        self.active = false;
        Ok(())
    }
}

/// One-shot convenience wrapper. Multi-record callers must use
/// [`AccountedObjectStoreReader`] so every record shares one pinned subject and
/// one closing identity fence.
pub fn read_object_record_document_with_work_and_cancellation<C>(
    objects_root: &Path,
    object_id: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut should_cancel: C,
) -> Result<AccountedObjectRecordDocument, StoreError>
where
    C: FnMut() -> bool,
{
    let mut reader = AccountedObjectStoreReader::begin_with_work_and_cancellation(
        objects_root,
        ledger,
        &mut should_cancel,
    )?;
    let mut document =
        match reader.read_with_work_and_cancellation(object_id, ledger, &mut should_cancel) {
            Ok(document) => document,
            Err(error) => {
                reader.cancel(ledger)?;
                return Err(error);
            }
        };
    if let Err(error) = reader.finish_with_cancellation(ledger, &mut should_cancel) {
        document.release(ledger)?;
        return Err(error);
    }
    Ok(document)
}

/// Scans flat object-record namespaces through pinned root/subdirectory
/// capabilities. Entries are visited in deterministic path order and dropped
/// between callbacks unless the caller retains them. A root/subdirectory
/// identity change fails the overall snapshot after the last visit.
pub fn scan_object_store_record_documents<F>(
    objects_root: &Path,
    visit: F,
) -> Result<(), StoreError>
where
    F: FnMut(ObjectStoreRecordScanEntry),
{
    scan_object_store_record_documents_with_hook(objects_root, visit, |_| Ok(()))
}

const OBJECT_STORE_DIRECTORY_ITERATOR_SCRATCH_BYTES: u64 = 4 * 1024;
const OBJECT_STORE_TRAVERSAL_POLL_BYTES: usize = 64 * 1024;
// A directory entry may materialize its platform filename while advancing the
// iterator. Reserve a portable UTF-8 worst-case NAME_MAX frame before that
// allocation, then shrink to the exact encoded name length.
const OBJECT_STORE_DIRECTORY_ENTRY_PREALLOCATION_BYTES: u64 =
    codefire_core::LOGICAL_FRAME_BASE_BYTES + 4 * 255;

struct AccountedObjectStoreDirectoryEntry {
    entry: cap_std::fs::DirEntry,
    name: PathBuf,
    frame_bytes: u64,
}

/// Scans object-record namespaces through one pinned store subject while
/// charging all directory, read, parse, and callback-scoped document memory to
/// the caller's ledger. The visitor may add its own retained result accounting
/// to the same ledger; this function releases only scanner-owned memory.
pub fn scan_object_store_record_documents_with_work_and_cancellation<C, F>(
    objects_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    mut should_cancel: C,
    mut visit: F,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
    F: FnMut(
        ObjectStoreRecordScanEntry,
        &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError>,
{
    poll_object_store_scan(&mut should_cancel)?;
    // Both the scanner and its pinned subject retain a full root PathBuf. The
    // encoded length is known from the borrowed input, so reserve both exact
    // owners before either allocation occurs.
    let retained_root_bytes = object_store_scan_name_frame_bytes(objects_root)?
        .checked_mul(2)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store scan retained root ownership overflow".to_string(),
            )
        })?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_root_bytes,
        ),
    )?;
    let canonical_root = objects_root.to_path_buf();
    let mut cache = SealedCommitValidationCache::default();
    let binding = bind_object_store_with_work_and_cancellation(
        canonical_root.clone(),
        &mut cache,
        ledger,
        &mut should_cancel,
    );
    let (bound_store, binding_retained_bytes) = match binding {
        Ok(binding) => binding,
        Err(error) => {
            drop(cache);
            drop(canonical_root);
            let release = release_object_store_scan_retained(ledger, retained_root_bytes);
            return match release {
                Ok(()) => Err(error),
                Err(release_error) => Err(release_error),
            };
        }
    };
    let mut memberships = Vec::<(usize, BTreeMap<PathBuf, u64>, u64)>::new();
    let mut retained_membership_bytes =
        match binding_retained_bytes.checked_add(retained_root_bytes) {
            Some(bytes) => bytes,
            None => {
                drop(bound_store);
                drop(cache);
                drop(canonical_root);
                release_object_store_scan_retained(ledger, binding_retained_bytes)?;
                release_object_store_scan_retained(ledger, retained_root_bytes)?;
                return Err(StoreError::InvalidSealedCommit(
                    "object-store scan retained ownership overflow".to_string(),
                ));
            }
        };
    let scan_result = (|| -> Result<(), StoreError> {
        poll_object_store_scan(&mut should_cancel)?;

        // Binding produced subjects in deterministic BTreeMap order. Retain
        // every observed directory member until the final membership and
        // identity fence; otherwise an add/remove race could look complete.
        for (subdir_index, subdir_subject) in bound_store.subject.subdirs.iter().enumerate() {
            poll_object_store_scan(&mut should_cancel)?;
            let directory = bound_store
                .subdirs
                .get(&subdir_subject.name)
                .ok_or_else(|| object_store_identity_changed(&canonical_root))?;
            let Some(directory) = directory else {
                continue;
            };
            let (names, retained_name_bytes) = collect_object_store_scan_names(
                directory,
                ledger,
                &mut should_cancel,
                codefire_core::DecisionWorkPhase::RecordRead,
            )?;
            let next_membership_bytes =
                match retained_membership_bytes.checked_add(retained_name_bytes) {
                    Some(bytes) => bytes,
                    None => {
                        drop(names);
                        release_object_store_scan_retained(ledger, retained_name_bytes)?;
                        return Err(StoreError::InvalidSealedCommit(
                            "object-store retained membership overflow".to_string(),
                        ));
                    }
                };

            let directory_scan = (|| -> Result<(), StoreError> {
                for name in names.keys() {
                    poll_object_store_scan(&mut should_cancel)?;
                    if name.extension() != Some(std::ffi::OsStr::new("json")) {
                        continue;
                    }

                    let (path, retained_path_bytes) = build_object_store_scan_path_accounted(
                        &canonical_root,
                        &subdir_subject.name,
                        name,
                        ledger,
                    )?;
                    let record_scan = (|| -> Result<(), StoreError> {
                        poll_object_store_scan(&mut should_cancel)?;
                        let metadata = directory.symlink_metadata(name);
                        let size_bytes = metadata.as_ref().ok().map(cap_std::fs::Metadata::len);
                        let (document, retained_document_bytes) = match metadata {
                            Ok(_) => read_object_store_scan_document_accounted(
                                directory,
                                name,
                                &path,
                                ledger,
                                &mut should_cancel,
                            )?,
                            Err(error) => (Err(StoreError::Io(error)), 0),
                        };

                        if let Err(error) = poll_object_store_scan(&mut should_cancel) {
                            drop(document);
                            release_object_store_scan_retained(ledger, retained_document_bytes)?;
                            return Err(error);
                        }
                        let visit_result = visit(
                            ObjectStoreRecordScanEntry {
                                path,
                                size_bytes,
                                document,
                            },
                            ledger,
                        );
                        release_object_store_scan_retained(ledger, retained_document_bytes)?;
                        visit_result?;
                        poll_object_store_scan(&mut should_cancel)
                    })();
                    let release_path =
                        release_object_store_scan_retained(ledger, retained_path_bytes);
                    match (record_scan, release_path) {
                        (Ok(()), Ok(())) => {}
                        (Err(error), Ok(())) => return Err(error),
                        (_, Err(error)) => return Err(error),
                    }
                }
                Ok(())
            })();
            if let Err(error) = directory_scan {
                drop(names);
                release_object_store_scan_retained(ledger, retained_name_bytes)?;
                return Err(error);
            }
            retained_membership_bytes = next_membership_bytes;
            memberships.push((subdir_index, names, retained_name_bytes));
        }

        for (subdir_index, expected_names, _) in &memberships {
            poll_object_store_scan(&mut should_cancel)?;
            let subject = &bound_store.subject.subdirs[*subdir_index];
            let directory = bound_store
                .subdirs
                .get(&subject.name)
                .and_then(Option::as_ref)
                .ok_or_else(|| object_store_identity_changed(&canonical_root))?;
            let (current_names, current_retained_bytes) = collect_object_store_scan_names(
                directory,
                ledger,
                &mut should_cancel,
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            )?;
            let unchanged = current_names.keys().eq(expected_names.keys());
            drop(current_names);
            release_object_store_scan_retained(ledger, current_retained_bytes)?;
            if !unchanged {
                return Err(object_store_identity_changed(
                    &canonical_root.join(&subject.name),
                ));
            }
        }

        poll_object_store_scan(&mut should_cancel)?;
        ensure_bound_object_store_identity_with_work_and_cancellation(
            &bound_store,
            &cache,
            ledger,
            &mut should_cancel,
        )?;
        poll_object_store_scan(&mut should_cancel)
    })();
    drop(memberships);
    drop(bound_store);
    drop(cache);
    drop(canonical_root);
    let release = release_object_store_scan_retained(ledger, retained_membership_bytes);
    match (scan_result, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn collect_object_store_scan_names<C>(
    directory: &Dir,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<(BTreeMap<PathBuf, u64>, u64), StoreError>
where
    C: FnMut() -> bool,
{
    let mut iterator = directory.entries()?;
    let mut names = BTreeMap::new();
    let mut retained_bytes = 0_u64;
    let collection_result = (|| -> Result<(), StoreError> {
        loop {
            poll_object_store_scan(should_cancel)?;
            let next = next_object_store_directory_entry_accounted(
                &mut iterator,
                ledger,
                should_cancel,
                phase,
            )?;
            poll_object_store_scan(should_cancel)?;
            let Some(entry) = next else {
                break;
            };
            let AccountedObjectStoreDirectoryEntry {
                entry: _,
                name,
                frame_bytes,
            } = entry;
            let next_retained_bytes = match retained_bytes.checked_add(frame_bytes) {
                Some(bytes) => bytes,
                None => {
                    release_object_store_scan_temporary(ledger, frame_bytes)?;
                    return Err(StoreError::InvalidSealedCommit(
                        "object-store scan directory membership overflow".to_string(),
                    ));
                }
            };
            retain_object_store_directory_entry_frame(ledger, frame_bytes, phase)?;
            retained_bytes = next_retained_bytes;
            if let Some(replaced) = names.insert(name, frame_bytes) {
                ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    replaced,
                )?;
                retained_bytes = retained_bytes
                    .checked_sub(replaced)
                    .ok_or_else(object_store_scan_accounting_underflow)?;
            }
        }
        Ok(())
    })();

    if let Err(error) = collection_result {
        drop(names);
        release_object_store_scan_retained(ledger, retained_bytes)?;
        return Err(error);
    }
    Ok((names, retained_bytes))
}

fn read_object_store_scan_document_accounted<C>(
    directory: &Dir,
    name: &Path,
    display_path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(Result<ObjectRecordDocument, StoreError>, u64), StoreError>
where
    C: FnMut() -> bool,
{
    let mut scratch_bytes = 0_u64;
    let bytes = read_object_record_bytes_from_directory_with_limit_and_hooks(
        directory,
        name,
        display_path,
        MAX_OBJECT_RECORD_BYTES,
        |physical_bytes| {
            let reserved_scratch = accounted_object_parse_scratch_bytes(physical_bytes)?;
            ledger.apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
                .charge(
                    codefire_core::DecisionWorkDimension::AggregateBytes,
                    physical_bytes,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    reserved_scratch,
                )
                .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
                .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
                .count(
                    codefire_core::DecisionWorkCounter::FileBytesRead,
                    physical_bytes,
                )
                .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
            )?;
            scratch_bytes = reserved_scratch;
            Ok(())
        },
        || poll_object_store_scan(should_cancel),
    );

    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(error) => {
            release_object_store_scan_temporary(ledger, scratch_bytes)?;
            return if matches!(
                error,
                StoreError::DecisionWorkAccounting(_) | StoreError::ValidationCancelled
            ) {
                Err(error)
            } else {
                Ok((Err(error), 0))
            };
        }
    };
    if let Err(error) = poll_object_store_scan(should_cancel) {
        drop(bytes);
        release_object_store_scan_temporary(ledger, scratch_bytes)?;
        return Err(error);
    }
    let document = match parse_object_record_document_with_work_and_cancellation(
        &bytes,
        ledger,
        should_cancel,
    ) {
        Ok(document) => document,
        Err(error) => {
            drop(bytes);
            release_object_store_scan_temporary(ledger, scratch_bytes)?;
            return if matches!(
                error,
                StoreError::DecisionWorkAccounting(_) | StoreError::ValidationCancelled
            ) {
                Err(error)
            } else {
                Ok((Err(error), 0))
            };
        }
    };
    let logical_bytes =
        match compact_object_record_document_size_with_cancellation(&document, should_cancel) {
            Ok(logical_bytes) => logical_bytes,
            Err(error) => {
                drop(document);
                drop(bytes);
                release_object_store_scan_temporary(ledger, scratch_bytes)?;
                return if matches!(error, StoreError::ValidationCancelled) {
                    Err(error)
                } else {
                    Ok((Err(error), 0))
                };
            }
        };
    if let Err(error) = validate_object_record_document_with_work_and_cancellation(
        &document,
        display_path,
        ledger,
        should_cancel,
    ) {
        drop(document);
        drop(bytes);
        release_object_store_scan_temporary(ledger, scratch_bytes)?;
        return if matches!(
            error,
            StoreError::DecisionWorkAccounting(_) | StoreError::ValidationCancelled
        ) {
            Err(error)
        } else {
            Ok((Err(error), 0))
        };
    }
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            logical_bytes,
        ),
    ) {
        drop(document);
        drop(bytes);
        release_object_store_scan_temporary(ledger, scratch_bytes)?;
        return Err(error.into());
    }
    drop(bytes);
    if let Err(error) = ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        scratch_bytes,
    ) {
        drop(document);
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            logical_bytes,
        )?;
        return Err(error.into());
    }
    Ok((Ok(document), logical_bytes))
}

fn object_store_scan_name_frame_bytes(name: &Path) -> Result<u64, StoreError> {
    let name_bytes = u64::try_from(name.as_os_str().as_encoded_bytes().len()).map_err(|_| {
        StoreError::InvalidSealedCommit(
            "object-store scan directory member name length overflow".to_string(),
        )
    })?;
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(name_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store scan directory member frame overflow".to_string(),
            )
        })
}

fn build_object_store_scan_path_accounted(
    canonical_root: &Path,
    subdir: &Path,
    name: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(PathBuf, u64), StoreError> {
    let encoded_component_bytes =
        [canonical_root, subdir, name]
            .into_iter()
            .try_fold(2_u64, |total, component| {
                let bytes = u64::try_from(component.as_os_str().as_encoded_bytes().len()).map_err(
                    |_| {
                        StoreError::InvalidSealedCommit(
                            "object-store scan callback path length overflow".to_string(),
                        )
                    },
                )?;
                total.checked_add(bytes).ok_or_else(|| {
                    StoreError::InvalidSealedCommit(
                        "object-store scan callback path length overflow".to_string(),
                    )
                })
            })?;
    let preallocated_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(encoded_component_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store scan callback path frame overflow".to_string(),
            )
        })?;
    let path_capacity = usize::try_from(encoded_component_bytes).map_err(|_| {
        StoreError::InvalidSealedCommit(
            "object-store scan callback path capacity overflow".to_string(),
        )
    })?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            preallocated_bytes,
        ),
    )?;
    // Construct the callback path in one owned buffer. Chained `join` calls
    // transiently retain both the intermediate and final PathBuf, which would
    // exceed the single preaccounted callback-path owner at this boundary.
    let mut path = PathBuf::with_capacity(path_capacity);
    path.push(canonical_root);
    path.push(subdir);
    path.push(name);
    let frame_bytes = match object_store_scan_name_frame_bytes(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            release_object_store_scan_retained(ledger, preallocated_bytes)?;
            return Err(error);
        }
    };
    let Some(release_bytes) = preallocated_bytes.checked_sub(frame_bytes) else {
        release_object_store_scan_retained(ledger, preallocated_bytes)?;
        return Err(StoreError::InvalidSealedCommit(
            "object-store scan callback path exceeded its preallocation".to_string(),
        ));
    };
    release_object_store_scan_retained(ledger, release_bytes)?;
    Ok((path, frame_bytes))
}

fn next_object_store_directory_entry_accounted<C>(
    iterator: &mut cap_std::fs::ReadDir,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<Option<AccountedObjectStoreDirectoryEntry>, StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(should_cancel)?;
    let preallocated_bytes = OBJECT_STORE_DIRECTORY_ITERATOR_SCRATCH_BYTES
        .checked_add(OBJECT_STORE_DIRECTORY_ENTRY_PREALLOCATION_BYTES)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store directory entry preallocation overflow".to_string(),
            )
        })?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            preallocated_bytes,
        ),
    )?;
    let next = iterator.next().transpose();
    let entry = match next {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            release_object_store_scan_temporary(ledger, preallocated_bytes)?;
            poll_object_store_scan(should_cancel)?;
            return Ok(None);
        }
        Err(error) => {
            release_object_store_scan_temporary(ledger, preallocated_bytes)?;
            return Err(error.into());
        }
    };
    let name = PathBuf::from(entry.file_name());
    let frame_bytes = match object_store_scan_name_frame_bytes(&name) {
        Ok(bytes) => bytes,
        Err(error) => {
            release_object_store_scan_temporary(ledger, preallocated_bytes)?;
            return Err(error);
        }
    };
    if frame_bytes > OBJECT_STORE_DIRECTORY_ENTRY_PREALLOCATION_BYTES {
        release_object_store_scan_temporary(ledger, preallocated_bytes)?;
        return Err(StoreError::InvalidSealedCommit(format!(
            "object-store directory member exceeds the preallocated name frame: {} bytes",
            name.as_os_str().as_encoded_bytes().len()
        )));
    }
    let release_bytes = OBJECT_STORE_DIRECTORY_ITERATOR_SCRATCH_BYTES
        .checked_add(OBJECT_STORE_DIRECTORY_ENTRY_PREALLOCATION_BYTES - frame_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store directory entry release overflow".to_string(),
            )
        })?;
    release_object_store_scan_temporary(ledger, release_bytes)?;
    if let Err(error) = poll_object_store_scan(should_cancel) {
        release_object_store_scan_temporary(ledger, frame_bytes)?;
        return Err(error);
    }
    Ok(Some(AccountedObjectStoreDirectoryEntry {
        entry,
        name,
        frame_bytes,
    }))
}

fn retain_object_store_directory_entry_frame(
    ledger: &mut codefire_core::DecisionWorkLedger,
    frame_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<(), StoreError> {
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .count(codefire_core::DecisionWorkCounter::UniqueNodesDiscovered, 1),
    ) {
        release_object_store_scan_temporary(ledger, frame_bytes)?;
        return Err(error.into());
    }
    if let Err(error) = release_object_store_scan_temporary(ledger, frame_bytes) {
        release_object_store_scan_retained(ledger, frame_bytes)?;
        return Err(error);
    }
    Ok(())
}

fn discard_object_store_directory_entry_frame(
    ledger: &mut codefire_core::DecisionWorkLedger,
    frame_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<(), StoreError> {
    let charge = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .count(codefire_core::DecisionWorkCounter::UniqueNodesDiscovered, 1),
    );
    let release = release_object_store_scan_temporary(ledger, frame_bytes);
    match (charge, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error.into()),
        (_, Err(error)) => Err(error),
    }
}

fn poll_object_store_scan<C>(should_cancel: &mut C) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    if should_cancel() {
        Err(StoreError::ValidationCancelled)
    } else {
        Ok(())
    }
}

fn release_object_store_scan_retained(
    ledger: &mut codefire_core::DecisionWorkLedger,
    bytes: u64,
) -> Result<(), StoreError> {
    if bytes > 0 {
        ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, bytes)?;
    }
    Ok(())
}

fn release_object_store_scan_temporary(
    ledger: &mut codefire_core::DecisionWorkLedger,
    bytes: u64,
) -> Result<(), StoreError> {
    if bytes > 0 {
        ledger.release(codefire_core::DecisionWorkDimension::TemporaryBytes, bytes)?;
    }
    Ok(())
}

fn object_store_scan_accounting_underflow() -> StoreError {
    StoreError::InvalidSealedCommit("object-store scan retained accounting underflow".to_string())
}

fn scan_object_store_record_documents_with_hook<F, H>(
    objects_root: &Path,
    mut visit: F,
    before_final_identity_check: H,
) -> Result<(), StoreError>
where
    F: FnMut(ObjectStoreRecordScanEntry),
    H: FnOnce(&Path) -> Result<(), StoreError>,
{
    let canonical_root = objects_root.to_path_buf();
    let mut cache = SealedCommitValidationCache::default();
    let bound_store = bind_object_store(canonical_root.clone(), &mut cache)?;
    let mut subdirs = bound_store.subdirs.iter().collect::<Vec<_>>();
    subdirs.sort_by(|left, right| left.0.cmp(right.0));
    for (relative_directory, directory) in subdirs {
        let Some(directory) = directory else {
            continue;
        };
        let mut names = directory
            .entries()?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        names.retain(|name| Path::new(name).extension() == Some(std::ffi::OsStr::new("json")));
        names.sort();
        for name in names {
            let path = canonical_root.join(relative_directory).join(&name);
            let metadata = directory.symlink_metadata(&name);
            let size_bytes = metadata.as_ref().ok().map(cap_std::fs::Metadata::len);
            let document = match metadata {
                Ok(_) => read_object_record_document_from_directory_with_limit_and_hook(
                    directory,
                    Path::new(&name),
                    &path,
                    MAX_OBJECT_RECORD_BYTES,
                    |_| Ok(()),
                ),
                Err(error) => Err(StoreError::Io(error)),
            };
            visit(ObjectStoreRecordScanEntry {
                path,
                size_bytes,
                document,
            });
        }
    }
    before_final_identity_check(&canonical_root)?;
    ensure_bound_object_store_identity(&bound_store, &cache)
}

fn read_object_record_document_from_directory_with_limit_and_hook<F>(
    directory: &Dir,
    file_name: impl AsRef<Path>,
    display_path: &Path,
    limit: u64,
    after_metadata_check: F,
) -> Result<ObjectRecordDocument, StoreError>
where
    F: FnOnce(u64) -> Result<(), StoreError>,
{
    let bytes = read_object_record_bytes_from_directory_with_limit_and_hook(
        directory,
        file_name,
        display_path,
        limit,
        after_metadata_check,
    )?;
    parse_object_record_document(&bytes)
}

fn read_object_record_bytes_from_directory_with_limit_and_hook<F>(
    directory: &Dir,
    file_name: impl AsRef<Path>,
    display_path: &Path,
    limit: u64,
    after_metadata_check: F,
) -> Result<Vec<u8>, StoreError>
where
    F: FnOnce(u64) -> Result<(), StoreError>,
{
    read_object_record_bytes_from_directory_with_limit_and_hooks(
        directory,
        file_name,
        display_path,
        limit,
        after_metadata_check,
        || Ok(()),
    )
}

fn read_object_record_bytes_from_directory_with_limit_and_hooks<F, C>(
    directory: &Dir,
    file_name: impl AsRef<Path>,
    display_path: &Path,
    limit: u64,
    after_metadata_check: F,
    mut before_read_chunk: C,
) -> Result<Vec<u8>, StoreError>
where
    F: FnOnce(u64) -> Result<(), StoreError>,
    C: FnMut() -> Result<(), StoreError>,
{
    let mut options = CapabilityOpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let file = directory.open_with(file_name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(StoreError::IncompleteRecord(format!(
            "object record must be a regular no-follow file: {}",
            display_path.display()
        )));
    }
    if metadata.len() > limit {
        return Err(StoreError::ObjectRecordTooLarge {
            path: display_path.to_path_buf(),
            size: metadata.len(),
            limit,
        });
    }
    after_metadata_check(metadata.len())?;

    let expected_size = metadata.len();
    let initial_capacity = usize::try_from(expected_size)
        .unwrap_or(usize::MAX)
        .min(limit as usize);
    let mut bytes = Vec::with_capacity(initial_capacity);
    let mut chunk = [0_u8; OBJECT_STORE_TRAVERSAL_POLL_BYTES];
    {
        let mut reader = (&file).take(expected_size);
        loop {
            before_read_chunk()?;
            let read = reader.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..read]);
        }
    }
    before_read_chunk()?;
    let mut growth_fence = [0_u8; 1];
    let grew_past_fstat = (&file).read(&mut growth_fence)? != 0;
    let final_size = file.metadata()?.len();
    if final_size > limit {
        return Err(StoreError::ObjectRecordTooLarge {
            path: display_path.to_path_buf(),
            size: final_size,
            limit,
        });
    }
    let bytes_read = u64::try_from(bytes.len()).map_err(|_| {
        StoreError::IncompleteRecord(format!(
            "object record byte count does not fit u64: {}",
            display_path.display()
        ))
    })?;
    if grew_past_fstat || final_size != expected_size || bytes_read != expected_size {
        return Err(StoreError::IncompleteRecord(format!(
            "object record size changed during read: {} (expected {expected_size} bytes, read {bytes_read}, final size {final_size})",
            display_path.display()
        )));
    }
    Ok(bytes)
}

fn parse_object_record_document(bytes: &[u8]) -> Result<ObjectRecordDocument, StoreError> {
    object_record_document_from_value(serde_json::from_slice(bytes)?)
}

fn accounted_object_parse_scratch_bytes(physical_bytes: u64) -> Result<u64, StoreError> {
    let parser_buffer = physical_bytes.min(OBJECT_STORE_TRAVERSAL_POLL_BYTES as u64);
    physical_bytes
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(parser_buffer))
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "accounted object parse scratch size overflow".to_string(),
            )
        })
}

fn parse_object_record_document_with_work_and_cancellation<C>(
    bytes: &[u8],
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<ObjectRecordDocument, StoreError>
where
    C: FnMut() -> bool,
{
    let parsed = {
        let reader = CancellableObjectStoreSliceReader::new(bytes, should_cancel);
        let mut reader = std::io::BufReader::with_capacity(
            bytes.len().min(OBJECT_STORE_TRAVERSAL_POLL_BYTES),
            reader,
        );
        let parsed = serde_json::from_reader(&mut reader);
        let reader = reader.into_inner();
        if reader.cancelled {
            return Err(StoreError::ValidationCancelled);
        }
        parsed
    };
    let mut document: Value = parsed?;
    poll_object_store_scan(should_cancel)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::SemanticValidation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1),
    )?;
    poll_object_json_value_with_cancellation(&document, should_cancel)?;

    let mut extensions = Map::new();
    if let Some(object) = document.as_object_mut() {
        let extension_names = object
            .keys()
            .filter(|name| !matches!(name.as_str(), "object_id" | "type" | "hash" | "payload"))
            .cloned()
            .collect::<Vec<_>>();
        for name in extension_names {
            poll_object_store_scan(should_cancel)?;
            if let Some(value) = object.remove(&name) {
                extensions.insert(name, value);
            }
        }
    }
    validate_object_extension_support_with_cancellation(&extensions, should_cancel)?;
    let record = serde_json::from_value(document)?;
    Ok(ObjectRecordDocument { record, extensions })
}

struct CancellableObjectStoreSliceReader<'a, C> {
    remaining: &'a [u8],
    should_cancel: &'a mut C,
    cancelled: bool,
}

impl<'a, C> CancellableObjectStoreSliceReader<'a, C>
where
    C: FnMut() -> bool,
{
    fn new(remaining: &'a [u8], should_cancel: &'a mut C) -> Self {
        Self {
            remaining,
            should_cancel,
            cancelled: false,
        }
    }
}

impl<C> Read for CancellableObjectStoreSliceReader<'_, C>
where
    C: FnMut() -> bool,
{
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.cancelled {
            return Err(std::io::Error::other("object record parsing cancelled"));
        }
        if (self.should_cancel)() {
            self.cancelled = true;
            return Err(std::io::Error::other("object record parsing cancelled"));
        }
        if self.remaining.is_empty() || output.is_empty() {
            return Ok(0);
        }
        let amount = output
            .len()
            .min(self.remaining.len())
            .min(OBJECT_STORE_TRAVERSAL_POLL_BYTES);
        output[..amount].copy_from_slice(&self.remaining[..amount]);
        self.remaining = &self.remaining[amount..];
        Ok(amount)
    }
}

fn poll_object_json_value_with_cancellation<C>(
    value: &Value,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(should_cancel)?;
    match value {
        Value::Array(items) => {
            for item in items {
                poll_object_json_value_with_cancellation(item, should_cancel)?;
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                poll_object_json_value_with_cancellation(value, should_cancel)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_object_extension_support_with_cancellation<C>(
    extensions: &Map<String, Value>,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    let Some(descriptors) = extensions.get("extensions") else {
        return Ok(());
    };
    let descriptors = descriptors.as_array().ok_or_else(|| {
        StoreError::IncompleteRecord(
            "object extensions must be an array of extension descriptors".to_string(),
        )
    })?;
    if descriptors.len() > 128 {
        return Err(StoreError::IncompleteRecord(
            "object extension descriptor count exceeds 128".to_string(),
        ));
    }
    for descriptor in descriptors {
        poll_object_store_scan(should_cancel)?;
        let object = descriptor.as_object().ok_or_else(|| {
            StoreError::IncompleteRecord(
                "object extension descriptor must be an object".to_string(),
            )
        })?;
        let namespace = object
            .get("namespace")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                StoreError::IncompleteRecord(
                    "object extension namespace must be a non-empty string".to_string(),
                )
            })?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                StoreError::IncompleteRecord(
                    "object extension name must be a non-empty string".to_string(),
                )
            })?;
        if object.get("version").and_then(Value::as_u64).is_none()
            || object.get("critical").and_then(Value::as_bool).is_none()
        {
            return Err(StoreError::IncompleteRecord(format!(
                "object extension {namespace}/{name} requires unsigned version and boolean critical fields"
            )));
        }
        if object.get("critical").and_then(Value::as_bool) == Some(true) {
            return Err(StoreError::IncompleteRecord(format!(
                "unsupported critical object extension: {namespace}/{name}"
            )));
        }
        if let Some(references) = object.get("references") {
            let references = references.as_array().ok_or_else(|| {
                StoreError::IncompleteRecord(format!(
                    "object extension {namespace}/{name} references must be a string array"
                ))
            })?;
            for reference in references {
                poll_object_store_scan(should_cancel)?;
                if reference.as_str().is_none() {
                    return Err(StoreError::IncompleteRecord(format!(
                        "object extension {namespace}/{name} references must be a string array"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn validate_object_record_document_with_work_and_cancellation<C>(
    document: &ObjectRecordDocument,
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    validate_object_extension_support_with_cancellation(&document.extensions, should_cancel)?;
    let computed_hash = object_digest_with_work_and_cancellation(
        &document.record.type_tag,
        &document.record.payload,
        ledger,
        should_cancel,
    )?;
    validate_object_record_with_digest(&document.record, None, Some(path), &|_, _| {
        Ok(computed_hash.clone())
    })
}

fn object_digest_with_work_and_cancellation<C>(
    type_tag: &str,
    payload: &Value,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<String, StoreError>
where
    C: FnMut() -> bool,
{
    let mut writer = AccountedObjectDigestWriter::new(ledger, should_cancel);
    let serialization = (|| -> Result<(), serde_json::Error> {
        writer
            .write_all(type_tag.as_bytes())
            .map_err(serde_json::Error::io)?;
        writer.write_all(&[0]).map_err(serde_json::Error::io)?;
        serde_json::to_writer(&mut writer, payload)
    })();
    let (bytes, reserved_bytes, work_error) = writer.finish();
    if let Some(error) = work_error {
        release_object_store_scan_temporary(ledger, reserved_bytes)?;
        return Err(error);
    }
    if let Err(error) = serialization {
        release_object_store_scan_temporary(ledger, reserved_bytes)?;
        return Err(error.into());
    }
    let hashed_bytes = match u64::try_from(bytes.len()) {
        Ok(bytes) => bytes,
        Err(_) => {
            release_object_store_scan_temporary(ledger, reserved_bytes)?;
            return Err(StoreError::InvalidSealedCommit(
                "object digest byte count overflow".to_string(),
            ));
        }
    };
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::SemanticValidation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .count(
            codefire_core::DecisionWorkCounter::BytesHashed,
            hashed_bytes,
        ),
    ) {
        release_object_store_scan_temporary(ledger, reserved_bytes)?;
        return Err(error.into());
    }
    if let Err(error) = poll_object_store_scan(should_cancel) {
        release_object_store_scan_temporary(ledger, reserved_bytes)?;
        return Err(error);
    }
    let digest = codefire_util::sha256_hex(&bytes);
    drop(bytes);
    release_object_store_scan_temporary(ledger, reserved_bytes)?;
    poll_object_store_scan(should_cancel)?;
    Ok(digest)
}

struct AccountedObjectDigestWriter<'a, C> {
    bytes: Vec<u8>,
    reserved_bytes: u64,
    next_poll_at: u64,
    work_error: Option<StoreError>,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    should_cancel: &'a mut C,
}

impl<'a, C> AccountedObjectDigestWriter<'a, C>
where
    C: FnMut() -> bool,
{
    fn new(ledger: &'a mut codefire_core::DecisionWorkLedger, should_cancel: &'a mut C) -> Self {
        Self {
            bytes: Vec::new(),
            reserved_bytes: 0,
            next_poll_at: 0,
            work_error: None,
            ledger,
            should_cancel,
        }
    }

    fn finish(self) -> (Vec<u8>, u64, Option<StoreError>) {
        (self.bytes, self.reserved_bytes, self.work_error)
    }

    fn fail(&mut self, error: StoreError) -> std::io::Error {
        self.work_error = Some(error);
        std::io::Error::other("object digest work interrupted")
    }
}

impl<C> Write for AccountedObjectDigestWriter<'_, C>
where
    C: FnMut() -> bool,
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.work_error.is_some() {
            return Err(std::io::Error::other("object digest work interrupted"));
        }
        for bytes in bytes.chunks(OBJECT_STORE_TRAVERSAL_POLL_BYTES) {
            let current_bytes = u64::try_from(self.bytes.len())
                .map_err(|_| std::io::Error::other("object digest size overflow"))?;
            if current_bytes >= self.next_poll_at {
                if (self.should_cancel)() {
                    return Err(self.fail(StoreError::ValidationCancelled));
                }
                self.next_poll_at =
                    current_bytes.saturating_add(OBJECT_STORE_TRAVERSAL_POLL_BYTES as u64);
            }
            let amount = u64::try_from(bytes.len())
                .map_err(|_| std::io::Error::other("object digest size overflow"))?;
            let needed = current_bytes
                .checked_add(amount)
                .ok_or_else(|| std::io::Error::other("object digest size overflow"))?;
            if needed > self.reserved_bytes {
                let chunk = OBJECT_STORE_TRAVERSAL_POLL_BYTES as u64;
                let target = needed
                    .checked_add(chunk - 1)
                    .and_then(|value| value.checked_div(chunk))
                    .and_then(|value| value.checked_mul(chunk))
                    .ok_or_else(|| std::io::Error::other("object digest size overflow"))?;
                let additional = target - self.reserved_bytes;
                let reservation = codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::SemanticValidation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    additional,
                );
                if let Err(error) = self.ledger.apply(reservation) {
                    return Err(self.fail(error.into()));
                }
                self.reserved_bytes = target;
            }
            self.bytes.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn object_record_document_from_value(
    mut document: Value,
) -> Result<ObjectRecordDocument, StoreError> {
    let mut extensions = Map::new();
    if let Some(object) = document.as_object_mut() {
        let extension_names = object
            .keys()
            .filter(|name| !matches!(name.as_str(), "object_id" | "type" | "hash" | "payload"))
            .cloned()
            .collect::<Vec<_>>();
        for name in extension_names {
            if let Some(value) = object.remove(&name) {
                extensions.insert(name, value);
            }
        }
    }
    validate_object_extension_support(&extensions)?;
    Ok(ObjectRecordDocument {
        record: serde_json::from_value(document)?,
        extensions,
    })
}

pub fn object_record_document_value(document: &ObjectRecordDocument) -> Result<Value, StoreError> {
    validate_object_extension_support(&document.extensions)?;
    let mut value = serde_json::to_value(&document.record)?;
    let object = value.as_object_mut().ok_or_else(|| {
        StoreError::IncompleteRecord("object record serialization is not an object".to_string())
    })?;
    for (name, extension) in &document.extensions {
        if object.contains_key(name) {
            return Err(StoreError::IncompleteRecord(format!(
                "object extension collides with a required record field: {name}"
            )));
        }
        object.insert(name.clone(), extension.clone());
    }
    Ok(value)
}

#[derive(Serialize)]
struct SerializableObjectRecordDocument<'a> {
    #[serde(flatten)]
    record: &'a ObjectRecord,
    #[serde(flatten)]
    extensions: &'a Map<String, Value>,
}

pub fn compact_object_record_document_size(
    document: &ObjectRecordDocument,
) -> Result<u64, StoreError> {
    let mut counter = ObjectRecordJsonByteCounter::default();
    serde_json::to_writer(
        &mut counter,
        &SerializableObjectRecordDocument {
            record: &document.record,
            extensions: &document.extensions,
        },
    )?;
    Ok(counter.bytes)
}

fn compact_object_record_document_size_with_cancellation<C>(
    document: &ObjectRecordDocument,
    should_cancel: &mut C,
) -> Result<u64, StoreError>
where
    C: FnMut() -> bool,
{
    let mut counter = CancellableObjectRecordJsonByteCounter::new(should_cancel);
    let result = serde_json::to_writer(
        &mut counter,
        &SerializableObjectRecordDocument {
            record: &document.record,
            extensions: &document.extensions,
        },
    );
    if counter.cancelled {
        return Err(StoreError::ValidationCancelled);
    }
    result?;
    poll_object_store_scan(counter.should_cancel)?;
    Ok(counter.bytes)
}

struct CancellableObjectRecordJsonByteCounter<'a, C> {
    bytes: u64,
    next_poll_at: u64,
    cancelled: bool,
    should_cancel: &'a mut C,
}

impl<'a, C> CancellableObjectRecordJsonByteCounter<'a, C>
where
    C: FnMut() -> bool,
{
    fn new(should_cancel: &'a mut C) -> Self {
        Self {
            bytes: 0,
            next_poll_at: 0,
            cancelled: false,
            should_cancel,
        }
    }
}

impl<C> Write for CancellableObjectRecordJsonByteCounter<'_, C>
where
    C: FnMut() -> bool,
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes >= self.next_poll_at {
            if (self.should_cancel)() {
                self.cancelled = true;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "object document traversal cancelled",
                ));
            }
            self.next_poll_at = self
                .bytes
                .saturating_add(OBJECT_STORE_TRAVERSAL_POLL_BYTES as u64);
        }
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("object document size overflow"))?;
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("object document size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn rendered_object_record_document_size(
    document: &ObjectRecordDocument,
) -> Result<u64, StoreError> {
    let mut counter = ObjectRecordJsonByteCounter::default();
    serde_json::to_writer_pretty(
        &mut counter,
        &SerializableObjectRecordDocument {
            record: &document.record,
            extensions: &document.extensions,
        },
    )?;
    counter
        .bytes
        .checked_add(1)
        .ok_or_else(|| StoreError::IncompleteRecord("object document size overflow".to_string()))
}

#[derive(Default)]
struct ObjectRecordJsonByteCounter {
    bytes: u64,
}

impl Write for ObjectRecordJsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("object document size overflow"))?;
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("object document size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn validate_object_extension_support(extensions: &Map<String, Value>) -> Result<(), StoreError> {
    let Some(descriptors) = extensions.get("extensions") else {
        // Historical top-level metadata fields are noncritical and are preserved.
        return Ok(());
    };
    let descriptors = descriptors.as_array().ok_or_else(|| {
        StoreError::IncompleteRecord(
            "object extensions must be an array of extension descriptors".to_string(),
        )
    })?;
    if descriptors.len() > 128 {
        return Err(StoreError::IncompleteRecord(
            "object extension descriptor count exceeds 128".to_string(),
        ));
    }
    for descriptor in descriptors {
        let object = descriptor.as_object().ok_or_else(|| {
            StoreError::IncompleteRecord(
                "object extension descriptor must be an object".to_string(),
            )
        })?;
        let namespace = object
            .get("namespace")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                StoreError::IncompleteRecord(
                    "object extension namespace must be a non-empty string".to_string(),
                )
            })?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                StoreError::IncompleteRecord(
                    "object extension name must be a non-empty string".to_string(),
                )
            })?;
        if object.get("version").and_then(Value::as_u64).is_none()
            || object.get("critical").and_then(Value::as_bool).is_none()
        {
            return Err(StoreError::IncompleteRecord(format!(
                "object extension {namespace}/{name} requires unsigned version and boolean critical fields"
            )));
        }
        if object.get("critical").and_then(Value::as_bool) == Some(true) {
            return Err(StoreError::IncompleteRecord(format!(
                "unsupported critical object extension: {namespace}/{name}"
            )));
        }
        if let Some(references) = object.get("references") {
            let valid = references
                .as_array()
                .is_some_and(|items| items.iter().all(|item| item.as_str().is_some()));
            if !valid {
                return Err(StoreError::IncompleteRecord(format!(
                    "object extension {namespace}/{name} references must be a string array"
                )));
            }
        }
    }
    Ok(())
}

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_temp_path(path: &Path) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("object.json");
    path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        counter
    ))
}

pub fn validate_sealed_commit(objects_root: &Path, commit_id: &str) -> Result<(), StoreError> {
    let mut cache = SealedCommitValidationCache::default();
    validate_sealed_commit_with_cache(objects_root, commit_id, &mut cache)
}

/// A sealed commit bound to the exact object-store root and subdirectory
/// capabilities used by validation.
///
/// Consumers must keep this token alive across validated reads and mutable
/// publication. `ensure_subject_identity` proves that the ambient store still
/// names the pinned subject; `read_commit`, `read_root`, and `read_object`
/// perform that proof both before and after descriptor-relative consumption.
pub struct ValidatedSealedCommit {
    commit_id: String,
    bound_store: BoundObjectStore,
}

/// Opaque, allocation-free identity of the object-store directory capability
/// used by a finalized sealed-validation session. Consumers can bind a later
/// publication capability to the same directory without retaining or hashing
/// an ambient pathname (which may legitimately change during bootstrap
/// publication).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedObjectStoreSubject {
    identity: ObjectStoreIdentity,
}

/// Borrowed planning view over one provisional validation subject and the
/// exact requested commit roots. It borrows only non-ledger fields from an
/// active publication proof and cannot be mistaken for final publication
/// evidence.
pub struct ProvisionalSealedValidationSubject<'a> {
    objects_root: &'a Path,
    subject: &'a Arc<ObjectStoreSubject>,
    retained_documents: &'a BTreeMap<String, RetainedObjectDocument>,
    validated_subjects: &'a HashSet<(Arc<ObjectStoreSubject>, String)>,
    staged_subjects: &'a HashSet<(Arc<ObjectStoreSubject>, String)>,
    roots: &'a [String],
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

#[must_use = "a prepublication binding must be attached to its proof or cancelled"]
struct PrepublicationValidationBinding {
    subject: Option<Arc<ObjectStoreSubject>>,
    roots: Vec<String>,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    retained_bytes: u64,
    active: bool,
}

impl PrepublicationValidationBinding {
    fn roots(&self) -> &[String] {
        &self.roots
    }
}

#[must_use = "a bound prepublication session must be detached or cancelled"]
pub struct BoundPrepublicationValidationSession {
    session: Option<Box<SealedCommitValidationSession>>,
    binding: Option<Box<PrepublicationValidationBinding>>,
    active: bool,
}

pub struct BoundPrepublicationValidationSessionBuildError {
    error: Option<Box<StoreError>>,
    session: Option<Box<SealedCommitValidationSession>>,
}

pub struct BoundPrepublicationValidationProofDetachError {
    pub error: Box<StoreError>,
    owner: Option<Box<BoundPrepublicationValidationSession>>,
}

/// Recoverable failure while cancelling a bound prepublication session.
///
/// The error retains the complete cleanup owner, including its decision-work
/// ledger. Call [`Self::retry`] or [`Self::into_parts`] rather than discarding
/// it when the underlying accounting failure is transient.
pub struct BoundPrepublicationValidationSessionCancellationError {
    pub error: Box<StoreError>,
    owner: Option<Box<BoundPrepublicationValidationSession>>,
}

impl fmt::Debug for BoundPrepublicationValidationSessionBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundPrepublicationValidationSessionBuildError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl BoundPrepublicationValidationSessionBuildError {
    pub fn into_parts(mut self) -> (StoreError, SealedCommitValidationSession) {
        let session = *self
            .session
            .take()
            .expect("prepublication binding error retains its session");
        let error = *self
            .error
            .take()
            .expect("prepublication binding error retains its cause");
        (error, session)
    }
}

impl Drop for BoundPrepublicationValidationSessionBuildError {
    fn drop(&mut self) {
        let Some(mut session) = self.session.take() else {
            return;
        };
        session.finalized = true;
        session.validation_failed = true;
        session.drop_retained_validation_payloads();
        let cleanup = session.release_decision_work_memory().and_then(|()| {
            let Some(ledger) = session.cache.decision_work_ledger.as_mut() else {
                return Ok(());
            };
            ensure_cleanup_ledger_current_zero(ledger)?;
            if ledger.snapshot().terminal_reason.is_none() {
                ledger.finish(codefire_core::DecisionWorkTerminalReason::CorruptSubject)?;
            }
            Ok(())
        });
        if cleanup.is_err() {
            std::mem::forget(session);
            fail_closed_cleanup("bound prepublication session build-error cleanup failed");
        }
        drop(session.cache.decision_work_ledger.take());
    }
}

impl fmt::Debug for BoundPrepublicationValidationProofDetachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundPrepublicationValidationProofDetachError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl BoundPrepublicationValidationProofDetachError {
    pub fn into_parts(mut self) -> (StoreError, BoundPrepublicationValidationSession) {
        let owner = *self
            .owner
            .take()
            .expect("prepublication detach error retains its bound owner");
        (*self.error, owner)
    }
}

impl fmt::Debug for BoundPrepublicationValidationSessionCancellationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundPrepublicationValidationSessionCancellationError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl BoundPrepublicationValidationSessionCancellationError {
    pub fn into_parts(mut self) -> (StoreError, BoundPrepublicationValidationSession) {
        let owner = *self
            .owner
            .take()
            .expect("prepublication cancellation error retains its bound owner");
        (*self.error, owner)
    }

    pub fn retry(
        mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<
        codefire_core::DecisionWorkLedger,
        BoundPrepublicationValidationSessionCancellationError,
    > {
        let owner = *self
            .owner
            .take()
            .expect("prepublication cancellation error retains its bound owner");
        owner.cancel_and_take_decision_work_ledger(reason)
    }
}

impl Drop for BoundPrepublicationValidationSession {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let cleanup = self.cancel_and_take_decision_work_ledger_inner(
            codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            &mut |ledger, dimension, amount| ledger.release(dimension, amount).map_err(Into::into),
        );
        if let Ok(ledger) = cleanup {
            drop(ledger);
            return;
        }

        // A raw drop has no caller to return a recoverable owner to. Never
        // silently destroy an unfinished ledger after an accounting failure:
        // retain the complete owner as a fail-closed leak instead.
        let session = self.session.take();
        let binding = self.binding.take();
        self.active = false;
        std::mem::forget(session);
        std::mem::forget(binding);
        fail_closed_cleanup("bound prepublication session cleanup failed");
    }
}

fn fail_closed_cleanup(context: &str) -> ! {
    if std::thread::panicking() {
        std::process::abort();
    }
    panic!("{context}");
}

fn ensure_cleanup_ledger_current_zero(
    ledger: &codefire_core::DecisionWorkLedger,
) -> Result<(), StoreError> {
    if ledger
        .snapshot()
        .current
        .values()
        .all(|amount| *amount == 0)
    {
        Ok(())
    } else {
        Err(StoreError::InvalidSealedCommit(
            "cancelled validation ledger retained concurrent decision work".to_string(),
        ))
    }
}

impl Drop for PrepublicationValidationBinding {
    fn drop(&mut self) {
        debug_assert!(
            !self.active,
            "prepublication binding dropped without its session owner"
        );
    }
}

impl ProvisionalSealedValidationSubject<'_> {
    pub fn objects_root(&self) -> &Path {
        self.objects_root
    }

    pub fn roots(&self) -> &[String] {
        self.roots
    }

    pub fn commit_payload(&self, commit_id: &str) -> Result<&Value, StoreError> {
        if !self.roots.iter().any(|root| root == commit_id) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not bound to this provisional root set: {commit_id}"
            )));
        }
        let is_validated = self
            .staged_subjects
            .iter()
            .chain(self.validated_subjects.iter())
            .any(|(subject, id)| subject == self.subject && id == commit_id);
        if !is_validated {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the provisional validation subject: {commit_id}"
            )));
        }
        self.retained_documents
            .get(commit_id)
            .map(|document| &document.document.record.payload)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "provisional commit document is not retained: {commit_id}"
                ))
            })
    }

    /// Returns any retained commit that was validated under this subject, not
    /// only a bound publication root. This read-only closure view supports
    /// ancestry and signature-policy queries before the final raw-document
    /// revalidation; it never exposes documents cached for another object
    /// store subject.
    pub fn retained_commit_payload(&self, commit_id: &str) -> Result<&Value, StoreError> {
        let is_validated = self
            .staged_subjects
            .iter()
            .chain(self.validated_subjects.iter())
            .any(|(subject, id)| subject == self.subject && id == commit_id);
        if !is_validated {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the provisional validation subject: {commit_id}"
            )));
        }
        self.retained_documents
            .get(commit_id)
            .map(|document| &document.document.record.payload)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "provisional commit document is not retained: {commit_id}"
                ))
            })
    }

    pub fn ensure_object_store_directory(&self, directory: &Dir) -> Result<(), StoreError> {
        let identity = stable_capability_object_store_identity(&directory.dir_metadata()?)?;
        if identity != self.subject.identity {
            return Err(StoreError::InvalidSealedCommit(
                "object store capability does not match the provisional validation subject"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub fn ensure_ledger(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "provisional root subject belongs to a different decision-work ledger".to_string(),
            ))
        }
    }
}

impl ValidatedObjectStoreSubject {
    pub fn ensure_directory(&self, directory: &Dir) -> Result<(), StoreError> {
        let identity = stable_capability_object_store_identity(&directory.dir_metadata()?)?;
        if identity != self.identity {
            return Err(StoreError::InvalidSealedCommit(
                "object store capability does not match the finalized validation subject"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

impl fmt::Debug for ValidatedSealedCommit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedSealedCommit")
            .field("commit_id", &self.commit_id)
            .field("objects_root", &self.bound_store.subject.canonical_root)
            .finish_non_exhaustive()
    }
}

impl ValidatedSealedCommit {
    pub fn commit_id(&self) -> &str {
        &self.commit_id
    }

    pub fn objects_root(&self) -> &Path {
        &self.bound_store.subject.canonical_root
    }

    /// Rechecks the retained root/subdirectory handles and every ambient name
    /// against the validated subject. Call immediately before and after a
    /// mutable visibility point.
    pub fn ensure_subject_identity(&self) -> Result<(), StoreError> {
        ensure_validated_subject_identity(&self.bound_store)
    }

    pub fn read_commit(&self) -> Result<Value, StoreError> {
        self.read_pinned_object(&self.commit_id)
    }

    pub fn read_root(&self, root: &str) -> Result<Value, StoreError> {
        self.consume(|bound_store| {
            let commit = read_bound_object(bound_store, &self.commit_id)?;
            let root_schema_version = commit_root_schema_version(&commit)?;
            let roots = commit
                .get("roots")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    StoreError::InvalidSealedCommit("roots must be an object".to_string())
                })?;
            let object_id = roots
                .get(root)
                .and_then(Value::as_str)
                .ok_or_else(|| StoreError::InvalidSealedCommit(format!("missing root {root}")))?;
            let payload = read_bound_object(bound_store, object_id)?;
            if let Some(expected_type) = expected_root_type(root) {
                let actual_type = payload
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("(missing)");
                if actual_type != expected_type {
                    return Err(StoreError::InvalidSealedCommit(format!(
                        "root {root} has type {actual_type}, expected {expected_type}"
                    )));
                }
            }
            validate_root_payload(root, &payload, root_schema_version)?;
            Ok(payload)
        })
    }

    /// Reads an exact object ID through the validated capability subject. This
    /// is intended for references reached from a value returned by `read_root`.
    pub fn read_object(&self, object_id: &str) -> Result<Value, StoreError> {
        self.read_pinned_object(object_id)
    }

    fn read_pinned_object(&self, object_id: &str) -> Result<Value, StoreError> {
        self.consume(|bound_store| read_bound_object(bound_store, object_id))
    }

    fn consume<T>(
        &self,
        operation: impl FnOnce(&BoundObjectStore) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.ensure_subject_identity()?;
        let result = operation(&self.bound_store);
        // An identity change is stronger evidence than an incidental read
        // error: never let the latter hide a validation/use subject swap.
        self.ensure_subject_identity()?;
        result
    }
}

pub fn validate_sealed_commit_pinned(
    objects_root: &Path,
    commit_id: &str,
) -> Result<ValidatedSealedCommit, StoreError> {
    let mut cache = SealedCommitValidationCache::default();
    validate_sealed_commit_pinned_with_cache(objects_root, commit_id, &mut cache)
}

pub fn validate_sealed_commit_pinned_with_cache(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
) -> Result<ValidatedSealedCommit, StoreError> {
    let bound_store = validate_sealed_commit_bound_with_cache_and_hooks(
        objects_root,
        commit_id,
        cache,
        |_| Ok(()),
        |_| Ok(()),
    )?;
    Ok(ValidatedSealedCommit {
        commit_id: commit_id.to_string(),
        bound_store,
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedValidationWork {
    pub semantic_commit_validations: u64,
    pub validated_commit_hits: u64,
    pub document_reads: u64,
    pub document_read_bytes: u64,
    pub document_memo_hits: u64,
    pub revalidated_documents: u64,
    pub revalidated_bytes: u64,
}

impl SealedValidationWork {
    fn difference(self, earlier: Self) -> Self {
        Self {
            semantic_commit_validations: self
                .semantic_commit_validations
                .saturating_sub(earlier.semantic_commit_validations),
            validated_commit_hits: self
                .validated_commit_hits
                .saturating_sub(earlier.validated_commit_hits),
            document_reads: self.document_reads.saturating_sub(earlier.document_reads),
            document_read_bytes: self
                .document_read_bytes
                .saturating_sub(earlier.document_read_bytes),
            document_memo_hits: self
                .document_memo_hits
                .saturating_sub(earlier.document_memo_hits),
            revalidated_documents: self
                .revalidated_documents
                .saturating_sub(earlier.revalidated_documents),
            revalidated_bytes: self
                .revalidated_bytes
                .saturating_sub(earlier.revalidated_bytes),
        }
    }
}

#[derive(Debug, Clone)]
struct RetainedObjectDocument {
    document: Arc<ObjectRecordDocument>,
    relative_directory: PathBuf,
    file_name: PathBuf,
    display_path: PathBuf,
    size_bytes: u64,
    logical_bytes: u64,
    document_digest: String,
}

#[derive(Clone)]
struct RetainedObjectPayload(Arc<ObjectRecordDocument>);

impl AsRef<Value> for RetainedObjectPayload {
    fn as_ref(&self) -> &Value {
        &self.0.record.payload
    }
}

impl std::ops::Deref for RetainedObjectPayload {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

trait ObjectPayloadView {
    fn payload(&self) -> &Value;
}

impl ObjectPayloadView for RetainedObjectPayload {
    fn payload(&self) -> &Value {
        self.as_ref()
    }
}

impl ObjectPayloadView for Value {
    fn payload(&self) -> &Value {
        self
    }
}

#[derive(Debug, Default)]
pub struct SealedCommitValidationCache {
    validated_subjects: HashSet<(Arc<ObjectStoreSubject>, String)>,
    retained_documents: BTreeMap<String, RetainedObjectDocument>,
    pinned_object_stores: HashMap<Arc<ObjectStoreSubject>, PinnedObjectStore>,
    work: SealedValidationWork,
    decision_work_ledger: Option<codefire_core::DecisionWorkLedger>,
    overlay_documents: BTreeMap<String, (Arc<ObjectRecordDocument>, u64)>,
    overlay_document_ids: HashSet<String>,
    retained_logical_bytes: u64,
    post_validation_retained_bytes: u64,
    prepublication_binding_bytes: u64,
    temporary_scratch_bytes: u64,
    #[cfg(not(any(unix, windows)))]
    next_invocation_identity: u64,
    #[cfg(test)]
    cache_hits: usize,
}

impl SealedCommitValidationCache {
    /// Returns the cumulative work performed through this command-scoped cache.
    pub fn work(&self) -> SealedValidationWork {
        self.work
    }
}

/// A command-scoped sealed-validation session for validating several heads
/// against one stable object-store snapshot.
///
/// `validate_commit` results are provisional. Call `finalize` before publishing
/// any result derived from them; finalization re-reads every retained object
/// record exactly once and rejects any raw-document change.
#[must_use = "sealed validation sessions must be finalized before publishing success"]
pub struct SealedCommitValidationSession {
    bound_store: BoundObjectStore,
    subject: Arc<ObjectStoreSubject>,
    has_existing_subject: bool,
    cache: SealedCommitValidationCache,
    staged_subjects: HashSet<(Arc<ObjectStoreSubject>, String)>,
    finalized: bool,
    validation_failed: bool,
    terminal_failure: Option<codefire_core::DecisionWorkBudgetExceeded>,
}

/// Detached proof that keeps a provisionally validated subject, its retained
/// document fingerprints, and its staged semantic results alive until the
/// combined publication owner performs the one final pre-seal revalidation.
///
/// The proof is bound to the exact decision-work ledger returned alongside it.
/// A foreign ledger is rejected before either the proof or either ledger is
/// mutated. Call [`Self::cancel`] on every failure path, or
/// [`Self::finish_after_seal`] after the combined owner seals and publishes.
#[must_use = "a prepublication validation proof must be finished or explicitly cancelled"]
pub struct PrepublicationSealedValidationProof {
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    bound_store: Option<BoundObjectStore>,
    subject: Option<Arc<ObjectStoreSubject>>,
    cache: SealedCommitValidationCache,
    staged_subjects: HashSet<(Arc<ObjectStoreSubject>, String)>,
    bound_roots: Vec<String>,
    preseal_revalidated: bool,
    validation_failed: bool,
    active: bool,
}

#[derive(Debug)]
pub struct PrepublicationValidationProofRevalidationError {
    pub error: Box<StoreError>,
    ledger: Box<codefire_core::DecisionWorkLedger>,
}

impl PrepublicationValidationProofRevalidationError {
    pub fn into_parts(self) -> (StoreError, codefire_core::DecisionWorkLedger) {
        (*self.error, *self.ledger)
    }
}

#[cfg(test)]
pub struct PrepublicationValidationProofDetachError {
    pub error: Box<StoreError>,
    session: Box<SealedCommitValidationSession>,
}

struct PrepublicationValidationProofBindingDetachError {
    error: Option<Box<StoreError>>,
    session: Option<Box<SealedCommitValidationSession>>,
    binding: Option<Box<PrepublicationValidationBinding>>,
}

#[cfg(test)]
impl fmt::Debug for PrepublicationValidationProofDetachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PrepublicationValidationProofDetachError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl PrepublicationValidationProofDetachError {
    pub fn into_parts(self) -> (StoreError, SealedCommitValidationSession) {
        (*self.error, *self.session)
    }
}

impl fmt::Debug for PrepublicationValidationProofBindingDetachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PrepublicationValidationProofBindingDetachError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl PrepublicationValidationProofBindingDetachError {
    fn into_parts(
        mut self,
    ) -> (
        StoreError,
        SealedCommitValidationSession,
        PrepublicationValidationBinding,
    ) {
        let session = *self
            .session
            .take()
            .expect("binding detach error retains its session");
        let binding = *self
            .binding
            .take()
            .expect("binding detach error retains its binding");
        let error = *self
            .error
            .take()
            .expect("binding detach error retains its cause");
        (error, session, binding)
    }
}

impl Drop for PrepublicationValidationProofBindingDetachError {
    fn drop(&mut self) {
        let (Some(session), Some(binding)) = (self.session.take(), self.binding.take()) else {
            return;
        };
        drop(BoundPrepublicationValidationSession {
            session: Some(session),
            binding: Some(binding),
            active: true,
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SealedValidationFailurePolicy {
    AbortSession,
    RecoverableCandidate,
}

fn recoverable_validation_error_is_terminal(error: &StoreError) -> bool {
    matches!(
        error,
        StoreError::DecisionWorkAccounting(_)
            | StoreError::ValidationCancelled
            | StoreError::SealedValidationSubjectChanged { .. }
    )
}

fn rollback_recoverable_handler_concurrent_growth(
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_before: u64,
    temporary_before: u64,
) -> Result<(), StoreError> {
    let retained_after = ledger
        .snapshot()
        .current
        .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
        .copied()
        .unwrap_or_default();
    let temporary_after = ledger
        .snapshot()
        .current
        .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
        .copied()
        .unwrap_or_default();
    if temporary_after > temporary_before {
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_after - temporary_before,
        )?;
    }
    if retained_after > retained_before {
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_after - retained_before,
        )?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct SealedValidationSessionBeginError {
    pub error: Box<StoreError>,
    ledger: Box<codefire_core::DecisionWorkLedger>,
}

impl SealedValidationSessionBeginError {
    pub fn into_parts(self) -> (StoreError, codefire_core::DecisionWorkLedger) {
        (*self.error, *self.ledger)
    }
}

#[derive(Debug)]
pub struct SealedCommitOverlayValidationInput {
    documents: Vec<(ObjectRecordDocument, u64)>,
    logical_bytes: u64,
    ledger: codefire_core::DecisionWorkLedger,
}

#[derive(Debug)]
pub struct SealedCommitOverlayValidationError {
    pub error: Box<StoreError>,
    documents: Vec<ObjectRecordDocument>,
    ledger: Box<codefire_core::DecisionWorkLedger>,
    retained_bytes: u64,
}

impl SealedCommitOverlayValidationError {
    pub fn into_parts(
        self,
    ) -> (
        StoreError,
        Vec<ObjectRecordDocument>,
        codefire_core::DecisionWorkLedger,
    ) {
        (*self.error, self.documents, *self.ledger)
    }

    pub fn discard_documents_and_recover_ledger(
        self,
    ) -> (StoreError, codefire_core::DecisionWorkLedger) {
        let Self {
            error,
            documents,
            ledger,
            retained_bytes,
        } = self;
        drop(documents);
        let mut ledger = *ledger;
        let error = if retained_bytes == 0 {
            *error
        } else {
            match ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ) {
                Ok(()) => *error,
                Err(release_error) => StoreError::DecisionWorkAccounting(Box::new(release_error)),
            }
        };
        (error, ledger)
    }
}

pub fn prepare_sealed_commit_overlay_validation_input(
    documents: Vec<ObjectRecordDocument>,
    ledger: codefire_core::DecisionWorkLedger,
) -> Result<SealedCommitOverlayValidationInput, SealedCommitOverlayValidationError> {
    prepare_sealed_commit_overlay_validation_input_with_cancellation(documents, ledger, || false)
}

pub fn prepare_sealed_commit_overlay_validation_input_with_cancellation<F>(
    documents: Vec<ObjectRecordDocument>,
    ledger: codefire_core::DecisionWorkLedger,
    should_cancel: F,
) -> Result<SealedCommitOverlayValidationInput, SealedCommitOverlayValidationError>
where
    F: FnMut() -> bool,
{
    prepare_sealed_commit_overlay_validation_input_with_cancellation_and_temporary_transfer(
        documents,
        ledger,
        0,
        should_cancel,
    )
}

pub fn prepare_sealed_commit_overlay_validation_input_with_cancellation_and_temporary_transfer<F>(
    documents: Vec<ObjectRecordDocument>,
    mut ledger: codefire_core::DecisionWorkLedger,
    temporary_bytes_to_release: u64,
    mut should_cancel: F,
) -> Result<SealedCommitOverlayValidationInput, SealedCommitOverlayValidationError>
where
    F: FnMut() -> bool,
{
    fn measure_documents<F>(
        documents: &[ObjectRecordDocument],
        should_cancel: &mut F,
    ) -> Result<(Vec<u64>, u64), StoreError>
    where
        F: FnMut() -> bool,
    {
        let mut object_ids = HashSet::new();
        let mut logical_bytes = 0_u64;
        let mut measured = Vec::with_capacity(documents.len());
        for document in documents {
            if should_cancel() {
                return Err(StoreError::ValidationCancelled);
            }
            let object_id = document.record.object_id.clone();
            validate_object_record(&document.record, Some(&object_id), None)?;
            if !object_ids.insert(object_id.clone()) {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "duplicate overlay object document: {object_id}"
                )));
            }
            let document_bytes = compact_object_record_document_size(document)?;
            logical_bytes = logical_bytes.checked_add(document_bytes).ok_or_else(|| {
                StoreError::InvalidSealedCommit("overlay retained byte count overflow".to_string())
            })?;
            measured.push(document_bytes);
        }
        Ok((measured, logical_bytes))
    }

    let (measured, logical_bytes) = match measure_documents(&documents, &mut should_cancel) {
        Ok(prepared) => prepared,
        Err(error) => {
            return Err(SealedCommitOverlayValidationError {
                error: Box::new(error),
                documents,
                ledger: Box::new(ledger),
                retained_bytes: 0,
            });
        }
    };
    let documents = documents.into_iter().zip(measured).collect::<Vec<_>>();
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Validation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            logical_bytes,
        ),
    ) {
        return Err(SealedCommitOverlayValidationError {
            error: Box::new(StoreError::DecisionWorkAccounting(Box::new(error))),
            documents: documents
                .into_iter()
                .map(|(document, _)| document)
                .collect(),
            ledger: Box::new(ledger),
            retained_bytes: 0,
        });
    }
    if let Err(error) = ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary_bytes_to_release,
    ) {
        return Err(SealedCommitOverlayValidationError {
            error: Box::new(StoreError::DecisionWorkAccounting(Box::new(error))),
            documents: documents
                .into_iter()
                .map(|(document, _)| document)
                .collect(),
            ledger: Box::new(ledger),
            retained_bytes: logical_bytes,
        });
    }
    Ok(SealedCommitOverlayValidationInput {
        documents,
        logical_bytes,
        ledger,
    })
}

impl SealedCommitValidationSession {
    /// Canonical object-store root bound by this validation session.
    pub fn objects_root(&self) -> &Path {
        &self.subject.canonical_root
    }

    /// Returns an opaque capability identity suitable for binding a later
    /// publication token. This deliberately excludes the ambient root path.
    pub fn validated_object_store_subject(
        &self,
    ) -> Result<ValidatedObjectStoreSubject, StoreError> {
        if !self.finalized || self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "object store subject requested before successful finalization".to_string(),
            ));
        }
        if !self.subject.identity.is_stable()
            || matches!(self.subject.identity, ObjectStoreIdentity::Overlay)
        {
            return Err(StoreError::InvalidSealedCommit(
                "stable object store capability identity is unavailable".to_string(),
            ));
        }
        Ok(ValidatedObjectStoreSubject {
            identity: self.subject.identity,
        })
    }

    /// Provisionally validates a head while reusing semantic results and parsed
    /// object documents retained by this session.
    pub fn validate_commit(&mut self, commit_id: &str) -> Result<(), StoreError> {
        self.validate_commit_with_cancellation(commit_id, || false)
    }

    /// Provisionally validates a head and polls a caller-owned cancellation
    /// predicate between traversal frames. Cancellation never publishes a
    /// partial validation result; the session becomes abort-only.
    pub fn validate_commit_with_cancellation<F>(
        &mut self,
        commit_id: &str,
        should_cancel: F,
    ) -> Result<(), StoreError>
    where
        F: FnMut() -> bool,
    {
        self.validate_commit_with_failure_policy(
            commit_id,
            should_cancel,
            SealedValidationFailurePolicy::AbortSession,
        )
    }

    /// Provisionally validates one diagnostic candidate without poisoning the
    /// session when that candidate is missing, malformed, or semantically
    /// invalid. Successfully retained documents and semantic results remain
    /// available to later candidates in the same pinned snapshot.
    ///
    /// Decision-work accounting failures and cancellation remain terminal and
    /// make the session abort-only. Call [`Self::finalize`] before publishing
    /// any success obtained from this session.
    pub fn validate_commit_recoverable(&mut self, commit_id: &str) -> Result<(), StoreError> {
        self.validate_commit_recoverable_with_cancellation(commit_id, || false)
    }

    /// Cancellation-aware variant of [`Self::validate_commit_recoverable`].
    pub fn validate_commit_recoverable_with_cancellation<F>(
        &mut self,
        commit_id: &str,
        should_cancel: F,
    ) -> Result<(), StoreError>
    where
        F: FnMut() -> bool,
    {
        self.validate_commit_with_failure_policy(
            commit_id,
            should_cancel,
            SealedValidationFailurePolicy::RecoverableCandidate,
        )
    }

    /// Validates one diagnostic candidate and accounts an ordinary candidate
    /// error through the command ledger without detaching or restarting this
    /// session.
    ///
    /// The handler receives the error only by reference, so any diagnostic it
    /// keeps must be copied into caller-owned, ledger-accounted storage before
    /// returning. `Ok(false)` means the handler accepted an ordinary invalid
    /// candidate and the session remains usable. Cancellation, decision-work
    /// accounting errors, and any handler error are terminal and make the
    /// session abort-only.
    ///
    /// This API requires a session created with a decision-work ledger. The
    /// handler owns every concurrent-memory reservation it adds and must leave
    /// temporary reservations balanced before returning. Successful retained
    /// growth remains caller-owned; terminal handler paths roll concurrent
    /// growth back to the handler-entry baseline.
    pub fn validate_commit_recoverable_with_cancellation_and_error_handler<C, H>(
        &mut self,
        commit_id: &str,
        should_cancel: C,
        on_candidate_error: H,
    ) -> Result<bool, StoreError>
    where
        C: FnMut() -> bool,
        H: FnOnce(&StoreError, &mut codefire_core::DecisionWorkLedger) -> Result<(), StoreError>,
    {
        if self.cache.decision_work_ledger.is_none() {
            return Err(StoreError::InvalidSealedCommit(
                "accounted recoverable validation requires a decision-work ledger".to_string(),
            ));
        }
        match self.validate_commit_with_failure_policy(
            commit_id,
            should_cancel,
            SealedValidationFailurePolicy::RecoverableCandidate,
        ) {
            Ok(()) => Ok(true),
            Err(error) if recoverable_validation_error_is_terminal(&error) => Err(error),
            Err(error) => {
                let ledger = self
                    .cache
                    .decision_work_ledger
                    .as_mut()
                    .expect("accounted recoverable validation checked its ledger");
                let retained_before =
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
                let temporary_before = ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied()
                    .unwrap_or_default();
                let terminal_before = ledger.snapshot().terminal_reason;
                let handled = on_candidate_error(&error, ledger);
                if let Err(handler_error) = handled {
                    let terminal_error = match rollback_recoverable_handler_concurrent_growth(
                        ledger,
                        retained_before,
                        temporary_before,
                    ) {
                        Ok(()) => handler_error,
                        Err(rollback_error) => rollback_error,
                    };
                    self.record_terminal_validation_failure(&terminal_error);
                    return Err(terminal_error);
                }
                let retained_after =
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
                let temporary_after = ledger
                    .snapshot()
                    .current
                    .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                    .copied()
                    .unwrap_or_default();
                if retained_after < retained_before
                    || temporary_after != temporary_before
                    || ledger.snapshot().terminal_reason != terminal_before
                {
                    let invariant_error = StoreError::InvalidSealedCommit(
                        "accounted recoverable validation handler changed ledger ownership outside its retained diagnostic growth".to_string(),
                    );
                    let terminal_error = match rollback_recoverable_handler_concurrent_growth(
                        ledger,
                        retained_before,
                        temporary_before,
                    ) {
                        Ok(()) => invariant_error,
                        Err(rollback_error) => rollback_error,
                    };
                    self.record_terminal_validation_failure(&terminal_error);
                    return Err(terminal_error);
                }
                Ok(false)
            }
        }
    }

    /// Temporarily lends the command ledger to success-path diagnostic work
    /// while this validation session remains provisional and attached.
    ///
    /// The callback may retain caller-owned report data, but must balance all
    /// temporary memory and must not terminate the ledger. On callback failure
    /// or an ownership invariant violation, concurrent growth is rolled back
    /// to the entry baseline and the validation session becomes abort-only.
    pub fn with_provisional_decision_work_ledger<H>(
        &mut self,
        on_valid_candidate: H,
    ) -> Result<(), StoreError>
    where
        H: FnOnce(&mut codefire_core::DecisionWorkLedger) -> Result<(), StoreError>,
    {
        if self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session is already finalized".to_string(),
            ));
        }
        if self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session must be aborted after a validation failure".to_string(),
            ));
        }
        let Some(ledger) = self.cache.decision_work_ledger.as_mut() else {
            return Err(StoreError::InvalidSealedCommit(
                "provisional decision work requires a validation-session ledger".to_string(),
            ));
        };
        let retained_before = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let temporary_before = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();
        let terminal_before = ledger.snapshot().terminal_reason;

        if let Err(callback_error) = on_valid_candidate(ledger) {
            let terminal_error = match rollback_recoverable_handler_concurrent_growth(
                ledger,
                retained_before,
                temporary_before,
            ) {
                Ok(()) => callback_error,
                Err(rollback_error) => rollback_error,
            };
            self.record_terminal_validation_failure(&terminal_error);
            return Err(terminal_error);
        }

        let retained_after = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let temporary_after = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();
        if retained_after < retained_before
            || temporary_after != temporary_before
            || ledger.snapshot().terminal_reason != terminal_before
        {
            let invariant_error = StoreError::InvalidSealedCommit(
                "provisional decision-work callback changed ledger ownership outside its retained report growth".to_string(),
            );
            let terminal_error = match rollback_recoverable_handler_concurrent_growth(
                ledger,
                retained_before,
                temporary_before,
            ) {
                Ok(()) => invariant_error,
                Err(rollback_error) => rollback_error,
            };
            self.record_terminal_validation_failure(&terminal_error);
            return Err(terminal_error);
        }
        Ok(())
    }

    fn validate_commit_with_failure_policy<F>(
        &mut self,
        commit_id: &str,
        mut should_cancel: F,
        failure_policy: SealedValidationFailurePolicy,
    ) -> Result<(), StoreError>
    where
        F: FnMut() -> bool,
    {
        if self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session is already finalized".to_string(),
            ));
        }
        if self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session must be aborted after a validation failure".to_string(),
            ));
        }
        let mut visiting = HashSet::new();
        let result = validate_sealed_commit_inner_with_cancellation(
            &self.bound_store,
            &self.subject,
            commit_id,
            &mut self.cache,
            &mut self.staged_subjects,
            &mut visiting,
            &mut should_cancel,
        );
        if result.as_ref().is_err_and(|error| {
            failure_policy == SealedValidationFailurePolicy::AbortSession
                || recoverable_validation_error_is_terminal(error)
        }) {
            self.record_terminal_validation_failure(
                result.as_ref().expect_err("validation result was checked"),
            );
        }
        result
    }

    fn record_terminal_validation_failure(&mut self, error: &StoreError) {
        if let StoreError::DecisionWorkAccounting(accounting) = error {
            if let Some(exhaustion) = accounting.budget_exceeded() {
                self.terminal_failure = Some(exhaustion.clone());
            }
        }
        self.validation_failed = true;
    }

    /// Finalizes the snapshot and returns the total validation/revalidation
    /// work for this session.
    pub fn finalize(&mut self) -> Result<SealedValidationWork, StoreError> {
        self.finalize_with_cancellation(|| false)
    }

    /// Finalizes the retained subject while polling between document
    /// revalidation steps. A cancellation is terminal and returns no payload.
    pub fn finalize_with_cancellation<F>(
        &mut self,
        mut should_cancel: F,
    ) -> Result<SealedValidationWork, StoreError>
    where
        F: FnMut() -> bool,
    {
        if self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session is already finalized".to_string(),
            ));
        }
        if let Some(exhaustion) = self.terminal_failure.clone() {
            return Err(StoreError::DecisionWorkAccounting(Box::new(
                codefire_core::DecisionWorkAccountingError::BudgetExceeded(exhaustion),
            )));
        }
        if self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session must be aborted after a validation failure".to_string(),
            ));
        }
        self.finalized = true;
        let finalized = if self.has_existing_subject {
            finalize_sealed_validation_subject_with_cancellation(
                &self.bound_store,
                &mut self.cache,
                &mut self.staged_subjects,
                &mut should_cancel,
            )
        } else {
            let mut result = Ok(());
            for staged_subject in self.staged_subjects.drain() {
                if should_cancel() {
                    result = Err(StoreError::ValidationCancelled);
                    break;
                }
                self.cache.validated_subjects.insert(staged_subject);
            }
            if result.is_err() {
                invalidate_sealed_validation_subject(&mut self.cache, &self.subject);
                self.staged_subjects.clear();
            }
            result
        };
        if let Err(error) = finalized {
            self.validation_failed = true;
            if let Some(ledger) = self.cache.decision_work_ledger.as_mut() {
                if ledger.snapshot().terminal_reason.is_none() {
                    let reason = match &error {
                        StoreError::SealedValidationSubjectChanged { .. } => {
                            codefire_core::DecisionWorkTerminalReason::SubjectChanged
                        }
                        StoreError::ValidationCancelled => {
                            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded
                        }
                        _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    };
                    ledger.finish(reason)?;
                }
            }
            return Err(error);
        }
        Ok(self.cache.work)
    }

    /// Returns work accumulated so far, including a failed finalization.
    pub fn work(&self) -> SealedValidationWork {
        self.cache.work
    }

    /// Returns the command ledger snapshot when this session was created with
    /// `begin_sealed_commit_validation_session_with_ledger`.
    pub fn decision_work_snapshot(&self) -> Option<&codefire_core::DecisionWorkSnapshot> {
        self.cache
            .decision_work_ledger
            .as_ref()
            .map(codefire_core::DecisionWorkLedger::snapshot)
    }

    /// Lends the attached ledger to caller-owned success-path materialization
    /// after final snapshot revalidation. Retained growth is left owned by the
    /// caller and survives ledger detachment; temporary growth and terminal
    /// state changes are rejected and rolled back.
    pub fn with_finalized_decision_work_ledger<T, H>(
        &mut self,
        operation: H,
    ) -> Result<T, StoreError>
    where
        H: FnOnce(&Path, &mut codefire_core::DecisionWorkLedger) -> Result<T, StoreError>,
    {
        if !self.finalized || self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "finalized decision work requires a successful finalized validation session"
                    .to_string(),
            ));
        }
        let Some(ledger) = self.cache.decision_work_ledger.as_mut() else {
            return Err(StoreError::InvalidSealedCommit(
                "finalized decision work requires a validation-session ledger".to_string(),
            ));
        };
        let retained_before = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let temporary_before = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();
        let terminal_before = ledger.snapshot().terminal_reason;
        let result = operation(&self.subject.canonical_root, ledger);
        let retained_after = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let temporary_after = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();
        if result.is_err()
            || retained_after < retained_before
            || temporary_after != temporary_before
            || ledger.snapshot().terminal_reason != terminal_before
        {
            let rollback = rollback_recoverable_handler_concurrent_growth(
                ledger,
                retained_before,
                temporary_before,
            );
            return match (result, rollback) {
                (_, Err(error)) => Err(error),
                (Err(error), Ok(())) => Err(error),
                (Ok(_), Ok(())) => Err(StoreError::InvalidSealedCommit(
                    "finalized decision-work callback changed ledger ownership outside its retained result growth".to_string(),
                )),
            };
        }
        result
    }

    /// Borrows an immutable payload only after the complete pinned/overlay subject has finalized
    /// successfully. The borrow keeps ledger detachment from outliving retained document ownership.
    pub fn validated_commit_payload(&self, commit_id: &str) -> Result<&Value, StoreError> {
        if !self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "validated commit payload is unavailable before finalization".to_string(),
            ));
        }
        if self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session must be aborted after a validation failure".to_string(),
            ));
        }
        if !self
            .cache
            .validated_subjects
            .iter()
            .any(|(subject, id)| subject == &self.subject && id == commit_id)
        {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the finalized validation subject: {commit_id}"
            )));
        }
        let document = self
            .cache
            .retained_documents
            .get(commit_id)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "finalized commit document is not retained: {commit_id}"
                ))
            })?;
        Ok(&document.document.record.payload)
    }

    /// Borrows a semantically validated retained commit before final raw
    /// snapshot revalidation. Mutation planning may consume this view, but no
    /// result derived from it may be published unless the session is later
    /// moved into a publication proof and that proof passes revalidation.
    pub fn provisional_commit_payload(&self, commit_id: &str) -> Result<&Value, StoreError> {
        if self.finalized || self.validation_failed || self.terminal_failure.is_some() {
            return Err(StoreError::InvalidSealedCommit(
                "provisional commit payload requires a successful unfinalized session".to_string(),
            ));
        }
        let is_validated = self
            .staged_subjects
            .iter()
            .chain(self.cache.validated_subjects.iter())
            .any(|(subject, id)| subject == &self.subject && id == commit_id);
        if !is_validated {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the provisional validation subject: {commit_id}"
            )));
        }
        let document = self
            .cache
            .retained_documents
            .get(commit_id)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "provisional commit document is not retained: {commit_id}"
                ))
            })?;
        Ok(&document.document.record.payload)
    }

    fn provisional_subject_for_roots<'a>(
        &'a self,
        roots: &'a [String],
    ) -> Result<ProvisionalSealedValidationSubject<'a>, StoreError> {
        if roots.is_empty() {
            return Err(StoreError::InvalidSealedCommit(
                "provisional publication root set must not be empty".to_string(),
            ));
        }
        for root in roots {
            self.provisional_commit_payload(root)?;
        }
        let ledger_identity = self
            .cache
            .decision_work_ledger
            .as_ref()
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "provisional root subject requires a validation-session ledger".to_string(),
                )
            })?
            .identity();
        Ok(ProvisionalSealedValidationSubject {
            objects_root: &self.subject.canonical_root,
            subject: &self.subject,
            retained_documents: &self.cache.retained_documents,
            validated_subjects: &self.cache.validated_subjects,
            staged_subjects: &self.staged_subjects,
            roots,
            ledger_identity,
        })
    }

    pub fn bind_prepublication_validation(
        mut self,
        roots: &[String],
    ) -> Result<BoundPrepublicationValidationSession, BoundPrepublicationValidationSessionBuildError>
    {
        match self.prepare_prepublication_validation_binding(roots) {
            Ok(binding) => Ok(BoundPrepublicationValidationSession {
                session: Some(Box::new(self)),
                binding: Some(Box::new(binding)),
                active: true,
            }),
            Err(error) => Err(BoundPrepublicationValidationSessionBuildError {
                error: Some(Box::new(error)),
                session: Some(Box::new(self)),
            }),
        }
    }

    fn prepare_prepublication_validation_binding(
        &mut self,
        roots: &[String],
    ) -> Result<PrepublicationValidationBinding, StoreError> {
        self.prepare_prepublication_validation_binding_with_clone(roots, |roots| Ok(roots.to_vec()))
    }

    fn prepare_prepublication_validation_binding_with_clone<F>(
        &mut self,
        roots: &[String],
        clone_roots: F,
    ) -> Result<PrepublicationValidationBinding, StoreError>
    where
        F: FnOnce(&[String]) -> Result<Vec<String>, StoreError>,
    {
        if self.cache.prepublication_binding_bytes != 0 {
            return Err(StoreError::InvalidSealedCommit(
                "validation session already issued a prepublication binding".to_string(),
            ));
        }
        self.provisional_subject_for_roots(roots)?;
        let retained_bytes =
            roots
                .iter()
                .try_fold(codefire_core::LOGICAL_FRAME_BASE_BYTES, |total, root| {
                    codefire_core::logical_retained_frame_bytes(root)
                        .and_then(|bytes| {
                            total
                                .checked_add(bytes)
                                .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
                        })
                        .map_err(StoreError::from)
                })?;
        let ledger = self.cache.decision_work_ledger.as_mut().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "prepublication binding requires a validation-session ledger".to_string(),
            )
        })?;
        let next_retained_logical_bytes = self
            .cache
            .retained_logical_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "prepublication binding ownership overflow".to_string(),
                )
            })?;
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
        let owned_roots = match clone_roots(roots) {
            Ok(owned_roots) if owned_roots.as_slice() == roots => owned_roots,
            Ok(owned_roots) => {
                drop(owned_roots);
                ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )?;
                return Err(StoreError::InvalidSealedCommit(
                    "prepublication binding clone changed its root set".to_string(),
                ));
            }
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )?;
                return Err(error);
            }
        };
        self.cache.retained_logical_bytes = next_retained_logical_bytes;
        self.cache.prepublication_binding_bytes = retained_bytes;
        Ok(PrepublicationValidationBinding {
            subject: Some(Arc::clone(&self.subject)),
            roots: owned_roots,
            ledger_identity: ledger.identity(),
            retained_bytes,
            active: true,
        })
    }

    #[cfg(test)]
    fn cancel_prepublication_validation_binding(
        &mut self,
        binding: &mut PrepublicationValidationBinding,
    ) -> Result<(), StoreError> {
        self.cancel_prepublication_validation_binding_with_release(
            binding,
            &mut |ledger, dimension, amount| ledger.release(dimension, amount).map_err(Into::into),
        )
    }

    fn cancel_prepublication_validation_binding_with_release<R>(
        &mut self,
        binding: &mut PrepublicationValidationBinding,
        release: &mut R,
    ) -> Result<(), StoreError>
    where
        R: FnMut(
            &mut codefire_core::DecisionWorkLedger,
            codefire_core::DecisionWorkDimension,
            u64,
        ) -> Result<(), StoreError>,
    {
        if !binding.active
            && binding.retained_bytes == 0
            && binding.roots.is_empty()
            && binding.subject.is_none()
        {
            return Ok(());
        }
        let next_retained_logical_bytes = self
            .cache
            .retained_logical_bytes
            .checked_sub(binding.retained_bytes)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "prepublication binding retained accounting underflow".to_string(),
                )
            })?;
        let ledger = self.cache.decision_work_ledger.as_mut().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "prepublication binding requires its validation-session ledger".to_string(),
            )
        })?;
        if !ledger.matches_identity(binding.ledger_identity)
            || binding.retained_bytes != self.cache.prepublication_binding_bytes
            || (binding.subject.is_some() && binding.subject.as_ref() != Some(&self.subject))
            || (binding.subject.is_none() && !binding.roots.is_empty())
        {
            return Err(StoreError::InvalidSealedCommit(
                "prepublication binding does not belong to this validation session".to_string(),
            ));
        }
        let payload = (std::mem::take(&mut binding.roots), binding.subject.take());
        drop(payload);
        release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            binding.retained_bytes,
        )?;
        self.cache.retained_logical_bytes = next_retained_logical_bytes;
        self.cache.prepublication_binding_bytes = 0;
        binding.retained_bytes = 0;
        binding.active = false;
        Ok(())
    }

    /// Charges additional post-validation work on the same command/request ledger while the
    /// finalized immutable payload view remains alive.
    pub fn apply_decision_work(
        &mut self,
        reservation: codefire_core::DecisionWorkReservation,
    ) -> Result<(), StoreError> {
        if !self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "post-validation work cannot start before finalization".to_string(),
            ));
        }
        if reservation.concurrent_amount(codefire_core::DecisionWorkDimension::RetainedBytes) != 0
            || reservation.concurrent_amount(codefire_core::DecisionWorkDimension::TemporaryBytes)
                != 0
        {
            return Err(StoreError::InvalidSealedCommit(
                "post-validation concurrent memory must use a tracked ownership API".to_string(),
            ));
        }
        apply_decision_work(&mut self.cache, reservation)
    }

    pub fn acquire_retained_decision_work(
        &mut self,
        reservation: codefire_core::DecisionWorkReservation,
        retained_bytes: u64,
    ) -> Result<(), StoreError> {
        if !self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "post-validation work cannot start before finalization".to_string(),
            ));
        }
        if reservation.concurrent_amount(codefire_core::DecisionWorkDimension::RetainedBytes) != 0
            || reservation.concurrent_amount(codefire_core::DecisionWorkDimension::TemporaryBytes)
                != 0
        {
            return Err(StoreError::InvalidSealedCommit(
                "tracked retained reservation must not contain caller-supplied concurrent memory"
                    .to_string(),
            ));
        }
        let next_retained = self
            .cache
            .retained_logical_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "post-validation retained accounting overflow".to_string(),
                )
            })?;
        let next_post_validation_retained = self
            .cache
            .post_validation_retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "post-validation retained ownership overflow".to_string(),
                )
            })?;
        apply_decision_work(
            &mut self.cache,
            reservation.acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ),
        )?;
        self.cache.retained_logical_bytes = next_retained;
        self.cache.post_validation_retained_bytes = next_post_validation_retained;
        Ok(())
    }

    pub fn release_retained_decision_work(&mut self, amount: u64) -> Result<(), StoreError> {
        if !self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "post-validation work cannot release memory before finalization".to_string(),
            ));
        }
        let next_post_validation_retained = self
            .cache
            .post_validation_retained_bytes
            .checked_sub(amount)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "post-validation retained release exceeds acquired ownership".to_string(),
                )
            })?;
        let next_retained = self
            .cache
            .retained_logical_bytes
            .checked_sub(amount)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "post-validation retained accounting underflow".to_string(),
                )
            })?;
        if let Some(ledger) = self.cache.decision_work_ledger.as_mut() {
            ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)?;
        }
        self.cache.retained_logical_bytes = next_retained;
        self.cache.post_validation_retained_bytes = next_post_validation_retained;
        Ok(())
    }

    /// Moves an unfinalized successful session into the publication owner.
    /// The proof's pre-seal revalidation is the session's single final raw
    /// document pass; read-only callers may continue using [`Self::finalize`].
    /// Any rejected detachment returns the intact session through the error.
    fn into_prepublication_validation_proof_with_binding(
        mut self,
        mut binding: PrepublicationValidationBinding,
    ) -> Result<
        (
            PrepublicationSealedValidationProof,
            codefire_core::DecisionWorkLedger,
        ),
        PrepublicationValidationProofBindingDetachError,
    > {
        let binding_matches = binding.active
            && self
                .cache
                .decision_work_ledger
                .as_ref()
                .is_some_and(|ledger| ledger.matches_identity(binding.ledger_identity))
            && binding.subject.as_ref() == Some(&self.subject)
            && binding.retained_bytes == self.cache.prepublication_binding_bytes
            && !binding.roots.is_empty();
        if !binding_matches {
            return Err(prepublication_proof_binding_detach_error(
                StoreError::InvalidSealedCommit(
                    "prepublication binding does not match the session subject, roots, or ledger"
                        .to_string(),
                ),
                self,
                binding,
            ));
        }
        if let Err(error) = self.ensure_prepublication_proof_detachable(true) {
            return Err(prepublication_proof_binding_detach_error(
                error, self, binding,
            ));
        }
        if !binding
            .roots
            .iter()
            .all(|root| self.provisional_commit_payload(root).is_ok())
        {
            return Err(prepublication_proof_binding_detach_error(
                StoreError::InvalidSealedCommit(
                    "prepublication binding roots are no longer provisional session roots"
                        .to_string(),
                ),
                self,
                binding,
            ));
        }
        self.cache.prepublication_binding_bytes = 0;
        let roots = std::mem::take(&mut binding.roots);
        drop(binding.subject.take());
        binding.retained_bytes = 0;
        binding.active = false;
        let (mut proof, ledger) = self.detach_prepublication_validation_proof();
        proof.bound_roots = roots;
        Ok((proof, ledger))
    }

    #[cfg(test)]
    fn into_prepublication_validation_proof(
        self,
    ) -> Result<
        (
            PrepublicationSealedValidationProof,
            codefire_core::DecisionWorkLedger,
        ),
        PrepublicationValidationProofDetachError,
    > {
        if let Err(error) = self.ensure_prepublication_proof_detachable(false) {
            return Err(prepublication_proof_detach_error(error, self));
        }
        Ok(self.detach_prepublication_validation_proof())
    }

    fn ensure_prepublication_proof_detachable(
        &self,
        binding_attached: bool,
    ) -> Result<(), StoreError> {
        if self.finalized || self.validation_failed || self.terminal_failure.is_some() {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof requires a successful unfinalized session".to_string(),
            ));
        }
        if !self.has_existing_subject
            || !self.subject.identity.is_stable()
            || matches!(self.subject.identity, ObjectStoreIdentity::Overlay)
        {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof requires a stable existing object-store subject".to_string(),
            ));
        }
        if self.cache.temporary_scratch_bytes != 0 {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof still owns temporary scratch".to_string(),
            ));
        }
        if self.cache.post_validation_retained_bytes != 0 {
            return Err(StoreError::InvalidSealedCommit(
                "post-validation retained ownership cannot enter a publication proof".to_string(),
            ));
        }
        if !binding_attached && self.cache.prepublication_binding_bytes != 0 {
            return Err(StoreError::InvalidSealedCommit(
                "issued prepublication binding must be attached during proof detachment"
                    .to_string(),
            ));
        }
        if self.cache.decision_work_ledger.is_none() {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof requires a validation-session ledger".to_string(),
            ));
        }
        Ok(())
    }

    fn detach_prepublication_validation_proof(
        mut self,
    ) -> (
        PrepublicationSealedValidationProof,
        codefire_core::DecisionWorkLedger,
    ) {
        let ledger = self
            .cache
            .decision_work_ledger
            .take()
            .expect("publication proof ledger presence was checked");
        let ledger_identity = ledger.identity();
        let proof = PrepublicationSealedValidationProof {
            ledger_identity,
            bound_store: Some(self.bound_store),
            subject: Some(self.subject),
            cache: self.cache,
            staged_subjects: self.staged_subjects,
            bound_roots: Vec::new(),
            preseal_revalidated: false,
            validation_failed: false,
            active: true,
        };
        (proof, ledger)
    }

    /// Returns the unfinished command ledger to its owner after final snapshot
    /// revalidation. The command owner remains responsible for charging later
    /// phases and setting the terminal reason.
    pub fn take_decision_work_ledger(
        &mut self,
    ) -> Result<Option<codefire_core::DecisionWorkLedger>, StoreError> {
        if !self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation ledger cannot be detached before finalization".to_string(),
            ));
        }
        self.drop_retained_validation_payloads();
        self.release_decision_work_memory()?;
        Ok(self.cache.decision_work_ledger.take())
    }

    /// Discards a provisional session and returns its ledger without assigning
    /// a terminal reason. This is for diagnostic callers that deliberately
    /// continue bounded response planning after recording a validation error.
    /// No validation result or retained payload remains usable afterwards.
    pub fn discard_and_take_decision_work_ledger(
        &mut self,
    ) -> Result<Option<codefire_core::DecisionWorkLedger>, StoreError> {
        if self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session is already finalized".to_string(),
            ));
        }
        self.finalized = true;
        self.validation_failed = true;
        self.drop_retained_validation_payloads();
        self.release_decision_work_memory()?;
        Ok(self.cache.decision_work_ledger.take())
    }

    /// Aborts a provisional session without final snapshot revalidation and returns its ledger.
    /// This is the failure path for operation owners that must preserve one command-wide ledger.
    pub fn abort_and_take_decision_work_ledger(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<Option<codefire_core::DecisionWorkLedger>, StoreError> {
        if self.finalized {
            return Err(StoreError::InvalidSealedCommit(
                "sealed validation session is already finalized".to_string(),
            ));
        }
        self.finalized = true;
        self.drop_retained_validation_payloads();
        self.release_decision_work_memory()?;
        if let Some(ledger) = self.cache.decision_work_ledger.as_mut() {
            if ledger.snapshot().terminal_reason.is_none() {
                ledger.finish(reason)?;
            }
        }
        Ok(self.cache.decision_work_ledger.take())
    }

    fn release_decision_work_memory(&mut self) -> Result<(), StoreError> {
        self.release_decision_work_memory_with_release(&mut |ledger, dimension, amount| {
            ledger.release(dimension, amount).map_err(Into::into)
        })
    }

    fn release_decision_work_memory_with_release<R>(
        &mut self,
        release: &mut R,
    ) -> Result<(), StoreError>
    where
        R: FnMut(
            &mut codefire_core::DecisionWorkLedger,
            codefire_core::DecisionWorkDimension,
            u64,
        ) -> Result<(), StoreError>,
    {
        if self.cache.decision_work_ledger.is_none()
            && (self.cache.temporary_scratch_bytes > 0 || self.cache.retained_logical_bytes > 0)
        {
            return Err(StoreError::InvalidSealedCommit(
                "validation memory accounting lost its decision-work ledger".to_string(),
            ));
        }
        if let Some(ledger) = self.cache.decision_work_ledger.as_mut() {
            if self.cache.temporary_scratch_bytes > 0 {
                release(
                    ledger,
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    self.cache.temporary_scratch_bytes,
                )?;
                self.cache.temporary_scratch_bytes = 0;
            }
            if self.cache.retained_logical_bytes > 0 {
                release(
                    ledger,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    self.cache.retained_logical_bytes,
                )?;
                self.cache.retained_logical_bytes = 0;
                self.cache.post_validation_retained_bytes = 0;
                self.cache.prepublication_binding_bytes = 0;
            }
        }
        Ok(())
    }

    fn drop_retained_validation_payloads(&mut self) {
        let payloads = (
            std::mem::take(&mut self.cache.validated_subjects),
            std::mem::take(&mut self.cache.retained_documents),
            std::mem::take(&mut self.cache.overlay_documents),
            std::mem::take(&mut self.cache.overlay_document_ids),
            std::mem::take(&mut self.cache.pinned_object_stores),
            std::mem::take(&mut self.staged_subjects),
        );
        drop(payloads);
    }
}

impl BoundPrepublicationValidationSession {
    pub fn roots(&self) -> &[String] {
        self.binding
            .as_ref()
            .expect("active bound prepublication session retains its binding")
            .roots()
    }

    pub fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        self.binding
            .as_ref()
            .is_some_and(|binding| ledger.matches_identity(binding.ledger_identity))
    }

    /// Cancels every capability and retained payload owned by this bound
    /// session, finishes its command ledger, and returns that ledger.
    ///
    /// Cleanup is recoverable: if releasing decision-work accounting fails,
    /// the returned error owns the session and its ledger and can be retried.
    /// Payloads already destroyed before a failed release are represented by
    /// their retained scalar until that retry succeeds.
    pub fn cancel_and_take_decision_work_ledger(
        self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<
        codefire_core::DecisionWorkLedger,
        BoundPrepublicationValidationSessionCancellationError,
    > {
        self.cancel_and_take_decision_work_ledger_with_release(
            reason,
            &mut |ledger, dimension, amount| ledger.release(dimension, amount).map_err(Into::into),
        )
    }

    fn cancel_and_take_decision_work_ledger_with_release<R>(
        mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
        release: &mut R,
    ) -> Result<
        codefire_core::DecisionWorkLedger,
        BoundPrepublicationValidationSessionCancellationError,
    >
    where
        R: FnMut(
            &mut codefire_core::DecisionWorkLedger,
            codefire_core::DecisionWorkDimension,
            u64,
        ) -> Result<(), StoreError>,
    {
        match self.cancel_and_take_decision_work_ledger_inner(reason, release) {
            Ok(ledger) => Ok(ledger),
            Err(error) => Err(BoundPrepublicationValidationSessionCancellationError {
                error: Box::new(error),
                owner: Some(Box::new(self)),
            }),
        }
    }

    fn cancel_and_take_decision_work_ledger_inner<R>(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
        release: &mut R,
    ) -> Result<codefire_core::DecisionWorkLedger, StoreError>
    where
        R: FnMut(
            &mut codefire_core::DecisionWorkLedger,
            codefire_core::DecisionWorkDimension,
            u64,
        ) -> Result<(), StoreError>,
    {
        if !self.active {
            return Err(StoreError::InvalidSealedCommit(
                "bound prepublication session is no longer active".to_string(),
            ));
        }
        let session = self.session.as_deref_mut().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "bound prepublication session lost its validation session".to_string(),
            )
        })?;
        let binding = self.binding.as_deref_mut().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "bound prepublication session lost its binding".to_string(),
            )
        })?;

        session.cancel_prepublication_validation_binding_with_release(binding, release)?;
        session.finalized = true;
        session.validation_failed = true;
        session.drop_retained_validation_payloads();
        session.release_decision_work_memory_with_release(release)?;
        let ledger = session.cache.decision_work_ledger.as_mut().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "bound prepublication cancellation requires its decision-work ledger".to_string(),
            )
        })?;
        ensure_cleanup_ledger_current_zero(ledger)?;
        if ledger.snapshot().terminal_reason.is_none() {
            ledger.finish(reason)?;
        }
        let ledger = session
            .cache
            .decision_work_ledger
            .take()
            .expect("cancellation ledger presence was checked");

        drop(self.binding.take());
        drop(self.session.take());
        self.active = false;
        Ok(ledger)
    }

    pub fn into_prepublication_validation_proof(
        mut self,
    ) -> Result<
        (
            PrepublicationSealedValidationProof,
            codefire_core::DecisionWorkLedger,
        ),
        BoundPrepublicationValidationProofDetachError,
    > {
        let session = *self
            .session
            .take()
            .expect("active bound prepublication session retains its session");
        let binding = *self
            .binding
            .take()
            .expect("active bound prepublication session retains its binding");
        self.active = false;
        match session.into_prepublication_validation_proof_with_binding(binding) {
            Ok(detached) => Ok(detached),
            Err(error) => {
                let (error, session, binding) = error.into_parts();
                Err(BoundPrepublicationValidationProofDetachError {
                    error: Box::new(error),
                    owner: Some(Box::new(BoundPrepublicationValidationSession {
                        session: Some(Box::new(session)),
                        binding: Some(Box::new(binding)),
                        active: true,
                    })),
                })
            }
        }
    }
}

impl PrepublicationSealedValidationProof {
    pub fn objects_root(&self) -> &Path {
        &self
            .subject
            .as_ref()
            .expect("active publication proof retains its subject")
            .canonical_root
    }

    pub fn work(&self) -> SealedValidationWork {
        self.cache.work
    }

    pub fn bound_roots(&self) -> &[String] {
        &self.bound_roots
    }

    pub fn matches_ledger(&self, ledger: &codefire_core::DecisionWorkLedger) -> bool {
        ledger.matches_identity(self.ledger_identity)
    }

    /// Borrows the bound provisional roots for mutation planning before the
    /// proof's final raw-document revalidation. The returned view borrows only
    /// proof fields, so its lifetime is independent of the verified ledger
    /// borrow and the caller may subsequently pass that ledger mutably to its
    /// concrete planner.
    pub fn provisional_subject_for_planning<'a>(
        &'a self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<ProvisionalSealedValidationSubject<'a>, StoreError> {
        self.ensure_ledger_identity(ledger)?;
        self.ensure_usable()?;
        if self.preseal_revalidated {
            return Err(StoreError::InvalidSealedCommit(
                "provisional planning subject is unavailable after proof revalidation".to_string(),
            ));
        }
        if ledger.is_prepublication_sealed() {
            return Err(StoreError::InvalidSealedCommit(
                "provisional planning subject is unavailable after prepublication sealing"
                    .to_string(),
            ));
        }
        if self.bound_roots.is_empty() {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof has no bound planning roots".to_string(),
            ));
        }
        let subject = self.subject.as_ref().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "active publication proof does not retain its validation subject".to_string(),
            )
        })?;
        for root in &self.bound_roots {
            let belongs_to_subject = self
                .staged_subjects
                .iter()
                .chain(self.cache.validated_subjects.iter())
                .any(|(candidate, id)| candidate == subject && id == root);
            if !belongs_to_subject {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "bound root is not part of the publication proof subject: {root}"
                )));
            }
            if !self.cache.retained_documents.contains_key(root) {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "bound root document is not retained by the publication proof: {root}"
                )));
            }
        }
        Ok(ProvisionalSealedValidationSubject {
            objects_root: &subject.canonical_root,
            subject,
            retained_documents: &self.cache.retained_documents,
            validated_subjects: &self.cache.validated_subjects,
            staged_subjects: &self.staged_subjects,
            roots: &self.bound_roots,
            ledger_identity: self.ledger_identity,
        })
    }

    pub fn validated_object_store_subject(
        &self,
    ) -> Result<ValidatedObjectStoreSubject, StoreError> {
        self.ensure_usable()?;
        if !self.preseal_revalidated {
            return Err(StoreError::InvalidSealedCommit(
                "object store subject is unavailable before proof revalidation".to_string(),
            ));
        }
        let subject = self
            .subject
            .as_ref()
            .expect("active publication proof retains its subject");
        Ok(ValidatedObjectStoreSubject {
            identity: subject.identity,
        })
    }

    pub fn validated_commit_payload(&self, commit_id: &str) -> Result<&Value, StoreError> {
        self.ensure_usable()?;
        if !self.preseal_revalidated {
            return Err(StoreError::InvalidSealedCommit(
                "validated commit payload is unavailable before proof revalidation".to_string(),
            ));
        }
        if !self.bound_roots.is_empty() && !self.bound_roots.iter().any(|root| root == commit_id) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the proof's bound publication roots: {commit_id}"
            )));
        }
        let subject = self
            .subject
            .as_ref()
            .expect("active publication proof retains its subject");
        if !self
            .cache
            .validated_subjects
            .iter()
            .any(|(cached_subject, id)| cached_subject == subject && id == commit_id)
        {
            return Err(StoreError::InvalidSealedCommit(format!(
                "commit is not part of the prepublication validation proof: {commit_id}"
            )));
        }
        let document = self
            .cache
            .retained_documents
            .get(commit_id)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "publication proof does not retain commit document: {commit_id}"
                ))
            })?;
        Ok(&document.document.record.payload)
    }

    /// Rechecks root/subdirectory identity, every retained raw document, and
    /// root/subdirectory identity again. The ledger is returned on both
    /// success and failure so the combined owner never loses its command-wide
    /// accounting state.
    pub fn revalidate_preseal<C>(
        &mut self,
        mut ledger: codefire_core::DecisionWorkLedger,
        mut should_cancel: C,
    ) -> Result<codefire_core::DecisionWorkLedger, PrepublicationValidationProofRevalidationError>
    where
        C: FnMut() -> bool,
    {
        if let Err(error) = self.ensure_ledger_identity(&ledger) {
            return Err(prepublication_proof_revalidation_error(error, ledger));
        }
        if let Err(error) = self.ensure_usable() {
            return Err(prepublication_proof_revalidation_error(error, ledger));
        }
        if ledger.is_prepublication_sealed() {
            return Err(prepublication_proof_revalidation_error(
                StoreError::InvalidSealedCommit(
                    "publication proof must be revalidated before the ledger is sealed".to_string(),
                ),
                ledger,
            ));
        }
        debug_assert!(self.cache.decision_work_ledger.is_none());
        self.preseal_revalidated = false;
        self.cache.decision_work_ledger = Some(ledger);
        let result = finalize_sealed_validation_subject_with_cancellation(
            self.bound_store
                .as_ref()
                .expect("active publication proof retains its bound store"),
            &mut self.cache,
            &mut self.staged_subjects,
            &mut should_cancel,
        );
        ledger = self
            .cache
            .decision_work_ledger
            .take()
            .expect("proof revalidation always returns its attached ledger");
        match result {
            Ok(()) => {
                self.preseal_revalidated = true;
                Ok(ledger)
            }
            Err(error) => {
                self.validation_failed = true;
                Err(prepublication_proof_revalidation_error(error, ledger))
            }
        }
    }

    pub fn finish_after_seal(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger_identity(ledger)?;
        if !self.preseal_revalidated || self.validation_failed {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof must pass its final pre-seal revalidation".to_string(),
            ));
        }
        if !ledger.is_prepublication_sealed() {
            return Err(StoreError::InvalidSealedCommit(
                "publication proof cannot finish before the ledger is sealed".to_string(),
            ));
        }
        self.release_ownership(ledger)
    }

    pub fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        self.ensure_ledger_identity(ledger)?;
        self.release_ownership(ledger)
    }

    fn ensure_usable(&self) -> Result<(), StoreError> {
        if self.active && !self.validation_failed {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "prepublication validation proof is no longer usable".to_string(),
            ))
        }
    }

    fn ensure_ledger_identity(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(StoreError::InvalidSealedCommit(
                "prepublication validation proof belongs to a different decision-work ledger"
                    .to_string(),
            ))
        }
    }

    fn release_ownership(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), StoreError> {
        if !self.active {
            return Ok(());
        }

        // Destroy every payload and capability owner before returning its
        // logical retained-memory ownership to the command ledger.
        let payloads = (
            std::mem::take(&mut self.cache.validated_subjects),
            std::mem::take(&mut self.cache.retained_documents),
            std::mem::take(&mut self.cache.overlay_documents),
            std::mem::take(&mut self.cache.overlay_document_ids),
            std::mem::take(&mut self.cache.pinned_object_stores),
            std::mem::take(&mut self.staged_subjects),
            std::mem::take(&mut self.bound_roots),
            self.bound_store.take(),
            self.subject.take(),
        );
        drop(payloads);

        if self.cache.temporary_scratch_bytes > 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                self.cache.temporary_scratch_bytes,
            )?;
            self.cache.temporary_scratch_bytes = 0;
        }
        if self.cache.retained_logical_bytes > 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.cache.retained_logical_bytes,
            )?;
            self.cache.retained_logical_bytes = 0;
            self.cache.post_validation_retained_bytes = 0;
        }
        self.active = false;
        Ok(())
    }
}

fn prepublication_proof_revalidation_error(
    error: StoreError,
    ledger: codefire_core::DecisionWorkLedger,
) -> PrepublicationValidationProofRevalidationError {
    PrepublicationValidationProofRevalidationError {
        error: Box::new(error),
        ledger: Box::new(ledger),
    }
}

#[cfg(test)]
fn prepublication_proof_detach_error(
    error: StoreError,
    session: SealedCommitValidationSession,
) -> PrepublicationValidationProofDetachError {
    PrepublicationValidationProofDetachError {
        error: Box::new(error),
        session: Box::new(session),
    }
}

fn prepublication_proof_binding_detach_error(
    error: StoreError,
    session: SealedCommitValidationSession,
    binding: PrepublicationValidationBinding,
) -> PrepublicationValidationProofBindingDetachError {
    PrepublicationValidationProofBindingDetachError {
        error: Some(Box::new(error)),
        session: Some(Box::new(session)),
        binding: Some(Box::new(binding)),
    }
}

impl Drop for PrepublicationSealedValidationProof {
    fn drop(&mut self) {
        debug_assert!(
            !self.active,
            "prepublication validation proof dropped without its ledger owner releasing it"
        );
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ObjectStoreSubject {
    canonical_root: PathBuf,
    identity: ObjectStoreIdentity,
    subdirs: Vec<ObjectStoreSubdirSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ObjectStoreSubdirSubject {
    name: PathBuf,
    identity: Option<ObjectStoreIdentity>,
}

type ObjectStoreSubdirHandles = HashMap<PathBuf, Option<Dir>>;

struct ObjectStoreSubdirBinding {
    subjects: Vec<ObjectStoreSubdirSubject>,
    handles: ObjectStoreSubdirHandles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ObjectStoreIdentity {
    Overlay,
    #[cfg(unix)]
    Unix {
        device: u64,
        inode: u64,
    },
    #[cfg(windows)]
    Windows {
        volume: u32,
        file_index: u64,
    },
    #[cfg(not(any(unix, windows)))]
    Invocation(u64),
}

impl ObjectStoreIdentity {
    fn is_stable(self) -> bool {
        if matches!(self, Self::Overlay) {
            return true;
        }
        #[cfg(any(unix, windows))]
        {
            true
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }
}

#[derive(Debug)]
struct PinnedObjectStore {
    handle: Dir,
    subdirs: ObjectStoreSubdirHandles,
}

#[derive(Debug)]
struct BoundObjectStore {
    subject: Arc<ObjectStoreSubject>,
    handle: Dir,
    subdirs: ObjectStoreSubdirHandles,
}

fn read_bound_object(bound_store: &BoundObjectStore, object_id: &str) -> Result<Value, StoreError> {
    Ok(read_bound_object_record(bound_store, object_id)?.payload)
}

fn read_bound_object_memoized(
    bound_store: &BoundObjectStore,
    object_id: &str,
    cache: &mut SealedCommitValidationCache,
) -> Result<RetainedObjectPayload, StoreError> {
    if cache.retained_documents.contains_key(object_id) {
        charge_memo_hit(cache, codefire_core::DecisionWorkPhase::RecordRead)?;
        checked_work_add(&mut cache.work.document_memo_hits, 1)?;
        return Ok(RetainedObjectPayload(Arc::clone(
            &cache.retained_documents[object_id].document,
        )));
    }
    if let Some((document, size_bytes)) = cache.overlay_documents.remove(object_id) {
        apply_decision_work(
            cache,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .count(codefire_core::DecisionWorkCounter::MemoMisses, 1),
        )?;
        let payload = RetainedObjectPayload(Arc::clone(&document));
        cache.retained_documents.insert(
            object_id.to_string(),
            RetainedObjectDocument {
                document,
                relative_directory: PathBuf::new(),
                file_name: PathBuf::new(),
                display_path: PathBuf::from(format!("<incoming:{object_id}>")),
                size_bytes,
                logical_bytes: size_bytes,
                document_digest: String::new(),
            },
        );
        return Ok(payload);
    }
    let retained =
        read_bound_object_document_with_hook(bound_store, object_id, &mut |size_bytes| {
            charge_document_read(cache, size_bytes)
        })?;
    transfer_document_read_to_retained(cache, retained.size_bytes, retained.logical_bytes)?;
    checked_work_add(&mut cache.work.document_reads, 1)?;
    checked_work_add(&mut cache.work.document_read_bytes, retained.size_bytes)?;
    let payload = RetainedObjectPayload(Arc::clone(&retained.document));
    cache
        .retained_documents
        .insert(object_id.to_string(), retained);
    Ok(payload)
}

fn checked_work_add(counter: &mut u64, amount: u64) -> Result<(), StoreError> {
    *counter = counter.checked_add(amount).ok_or_else(|| {
        StoreError::InvalidSealedCommit("sealed validation work counter overflow".to_string())
    })?;
    Ok(())
}

fn apply_decision_work(
    cache: &mut SealedCommitValidationCache,
    reservation: codefire_core::DecisionWorkReservation,
) -> Result<(), StoreError> {
    if let Some(ledger) = cache.decision_work_ledger.as_mut() {
        ledger.apply(reservation)?;
    }
    Ok(())
}

fn charge_document_read(
    cache: &mut SealedCommitValidationCache,
    size_bytes: u64,
) -> Result<(), StoreError> {
    let scratch_bytes = size_bytes.checked_mul(2).ok_or_else(|| {
        StoreError::InvalidSealedCommit("sealed validation read scratch size overflow".to_string())
    })?;
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )
        .charge(
            codefire_core::DecisionWorkDimension::AggregateBytes,
            size_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
        .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
        .count(
            codefire_core::DecisionWorkCounter::FileBytesRead,
            size_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::BytesHashed, size_bytes)
        .count(codefire_core::DecisionWorkCounter::MemoMisses, 1)
        .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
    )?;
    cache.temporary_scratch_bytes = cache
        .temporary_scratch_bytes
        .checked_add(scratch_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation temporary accounting overflow".to_string(),
            )
        })?;
    Ok(())
}

fn transfer_document_read_to_retained(
    cache: &mut SealedCommitValidationCache,
    physical_bytes: u64,
    logical_bytes: u64,
) -> Result<(), StoreError> {
    let scratch_bytes = physical_bytes.checked_mul(2).ok_or_else(|| {
        StoreError::InvalidSealedCommit("sealed validation read scratch size overflow".to_string())
    })?;
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            logical_bytes,
        ),
    )?;
    cache.retained_logical_bytes = cache
        .retained_logical_bytes
        .checked_add(logical_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation retained accounting overflow".to_string(),
            )
        })?;
    if let Some(ledger) = cache.decision_work_ledger.as_mut() {
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
        )?;
    }
    cache.temporary_scratch_bytes = cache
        .temporary_scratch_bytes
        .checked_sub(scratch_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation temporary accounting underflow".to_string(),
            )
        })?;
    Ok(())
}

fn charge_memo_hit(
    cache: &mut SealedCommitValidationCache,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<(), StoreError> {
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .count(codefire_core::DecisionWorkCounter::MemoHits, 1),
    )
}

fn charge_semantic_validation(cache: &mut SealedCommitValidationCache) -> Result<(), StoreError> {
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::SemanticValidation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphNodes, 1)
        .count(codefire_core::DecisionWorkCounter::SemanticValidations, 1),
    )
}

fn charge_snapshot_revalidation(
    cache: &mut SealedCommitValidationCache,
    size_bytes: u64,
) -> Result<(), StoreError> {
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(
            codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
            1,
        )
        .charge(
            codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
            size_bytes,
        )
        .count(
            codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
            1,
        )
        .count(
            codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
            size_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
        .count(
            codefire_core::DecisionWorkCounter::FileBytesRead,
            size_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::BytesHashed, size_bytes)
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            size_bytes,
        ),
    )?;
    cache.temporary_scratch_bytes = cache
        .temporary_scratch_bytes
        .checked_add(size_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "snapshot revalidation scratch accounting overflow".to_string(),
            )
        })?;
    Ok(())
}

fn release_snapshot_revalidation_scratch(
    cache: &mut SealedCommitValidationCache,
    size_bytes: u64,
) -> Result<(), StoreError> {
    if let Some(ledger) = cache.decision_work_ledger.as_mut() {
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            size_bytes,
        )?;
    }
    cache.temporary_scratch_bytes = cache
        .temporary_scratch_bytes
        .checked_sub(size_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "snapshot revalidation scratch accounting underflow".to_string(),
            )
        })?;
    Ok(())
}

fn charge_traversal_frames(
    cache: &mut SealedCommitValidationCache,
    depth: u64,
    frontier: u64,
    edge_occurrence: bool,
    frame_ids: &[&str],
) -> Result<(), StoreError> {
    let retained_bytes = frame_ids.iter().try_fold(0_u64, |total, frame_id| {
        let frame_bytes = codefire_core::logical_retained_frame_bytes(frame_id)?;
        total.checked_add(frame_bytes).ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation retained frame size overflow".to_string(),
            )
        })
    })?;
    let mut reservation = codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::GraphDiscovery,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .observe_maximum(codefire_core::DecisionWorkDimension::LogicalDepth, depth)
    .observe_maximum(
        codefire_core::DecisionWorkDimension::MaximumFrontier,
        frontier,
    )
    .acquire(
        codefire_core::DecisionWorkDimension::RetainedBytes,
        retained_bytes,
    )
    .count(
        codefire_core::DecisionWorkCounter::FrontierInsertionAttempts,
        u64::try_from(frame_ids.len()).map_err(|_| {
            StoreError::InvalidSealedCommit(
                "sealed validation frame count does not fit u64".to_string(),
            )
        })?,
    );
    if edge_occurrence {
        reservation = reservation
            .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
            .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1);
    }
    apply_decision_work(cache, reservation)?;
    cache.retained_logical_bytes = cache
        .retained_logical_bytes
        .checked_add(retained_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation retained accounting overflow".to_string(),
            )
        })?;
    Ok(())
}

fn release_traversal_frame(
    cache: &mut SealedCommitValidationCache,
    frame_id: &str,
) -> Result<(), StoreError> {
    let retained_bytes = codefire_core::logical_retained_frame_bytes(frame_id)?;
    if let Some(ledger) = cache.decision_work_ledger.as_mut() {
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        )?;
    }
    cache.retained_logical_bytes = cache
        .retained_logical_bytes
        .checked_sub(retained_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "sealed validation retained frame accounting underflow".to_string(),
            )
        })?;
    Ok(())
}

fn release_tracked_traversal_frame(
    cache: &mut SealedCommitValidationCache,
    charged_frame_ids: &mut Vec<String>,
    frame_id: &str,
) -> Result<(), StoreError> {
    let position = charged_frame_ids
        .iter()
        .rposition(|charged| charged == frame_id)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(format!(
                "sealed validation traversal frame was not retained: {frame_id}"
            ))
        })?;
    charged_frame_ids.remove(position);
    release_traversal_frame(cache, frame_id)
}

fn charge_root_edge(cache: &mut SealedCommitValidationCache) -> Result<(), StoreError> {
    apply_decision_work(
        cache,
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
        .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1),
    )
}

fn read_bound_object_record(
    bound_store: &BoundObjectStore,
    object_id: &str,
) -> Result<ObjectRecord, StoreError> {
    Ok(read_bound_object_document(bound_store, object_id)?
        .document
        .record
        .clone())
}

fn read_bound_object_document(
    bound_store: &BoundObjectStore,
    object_id: &str,
) -> Result<RetainedObjectDocument, StoreError> {
    read_bound_object_document_with_hook(bound_store, object_id, &mut |_| Ok(()))
}

fn read_bound_object_document_with_hook<F>(
    bound_store: &BoundObjectStore,
    object_id: &str,
    before_read: &mut F,
) -> Result<RetainedObjectDocument, StoreError>
where
    F: FnMut(u64) -> Result<(), StoreError>,
{
    if let Some(subdir) = object_subdir_by_id(object_id) {
        let directory = match bound_store.subdirs.get(Path::new(subdir)) {
            Some(Some(directory)) => directory,
            Some(None) | None => return Err(StoreError::ObjectNotFound(object_id.to_string())),
        };
        return read_bound_object_document_from_directory(
            bound_store,
            directory,
            Path::new(subdir),
            object_id,
            before_read,
        )?
        .ok_or_else(|| StoreError::ObjectNotFound(object_id.to_string()));
    }
    if !allows_unknown_subdir_lookup(object_id) {
        return Err(StoreError::ObjectNotFound(object_id.to_string()));
    }

    let mut subdirs = bound_store.subdirs.iter().collect::<Vec<_>>();
    subdirs.sort_by(|left, right| left.0.cmp(right.0));
    for (name, directory) in subdirs {
        let Some(directory) = directory else {
            continue;
        };
        if let Some(record) = read_bound_object_document_from_directory(
            bound_store,
            directory,
            name,
            object_id,
            before_read,
        )? {
            return Ok(record);
        }
    }
    Err(StoreError::ObjectNotFound(object_id.to_string()))
}

fn read_bound_object_document_from_directory<F>(
    bound_store: &BoundObjectStore,
    directory: &Dir,
    relative_directory: &Path,
    object_id: &str,
    before_read: &mut F,
) -> Result<Option<RetainedObjectDocument>, StoreError>
where
    F: FnMut(u64) -> Result<(), StoreError>,
{
    let file_name = PathBuf::from(format!("{object_id}.json"));
    let path = bound_store
        .subject
        .canonical_root
        .join(relative_directory)
        .join(&file_name);
    let bytes = match read_object_record_bytes_from_directory_with_limit_and_hook(
        directory,
        &file_name,
        &path,
        MAX_OBJECT_RECORD_BYTES,
        before_read,
    ) {
        Ok(bytes) => bytes,
        Err(StoreError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None)
        }
        Err(error) => return Err(error),
    };
    let document_digest = codefire_util::sha256_hex(&bytes);
    let document = parse_object_record_document(&bytes)?;
    validate_object_record(&document.record, Some(object_id), Some(&path))?;
    let document_logical_bytes = compact_object_record_document_size(&document)?;
    let logical_bytes = retained_object_document_logical_bytes(
        document_logical_bytes,
        object_id,
        relative_directory,
        &file_name,
        &path,
        &document_digest,
    )?;
    Ok(Some(RetainedObjectDocument {
        document: Arc::new(document),
        relative_directory: relative_directory.to_path_buf(),
        file_name,
        display_path: path,
        size_bytes: u64::try_from(bytes.len()).map_err(|_| {
            StoreError::InvalidSealedCommit(
                "sealed validation document size does not fit u64".to_string(),
            )
        })?,
        logical_bytes,
        document_digest,
    }))
}

fn retained_object_document_logical_bytes(
    document_bytes: u64,
    object_id: &str,
    relative_directory: &Path,
    file_name: &Path,
    display_path: &Path,
    digest: &str,
) -> Result<u64, StoreError> {
    let owned_bytes = [
        object_id.len(),
        relative_directory.as_os_str().as_encoded_bytes().len(),
        file_name.as_os_str().as_encoded_bytes().len(),
        display_path.as_os_str().as_encoded_bytes().len(),
        digest.len(),
    ]
    .into_iter()
    .try_fold(document_bytes, |total, length| {
        total
            .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES)
            .and_then(|bytes| bytes.checked_add(length as u64))
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "retained object document ownership overflow".to_string(),
                )
            })
    })?;
    owned_bytes
        .checked_add(codefire_core::LOGICAL_FRAME_BASE_BYTES * 3)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "retained object document container ownership overflow".to_string(),
            )
        })
}

pub fn validate_sealed_commit_with_cache(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
) -> Result<(), StoreError> {
    validate_sealed_commit_bound_with_cache_and_hooks(
        objects_root,
        commit_id,
        cache,
        |_| Ok(()),
        |_| Ok(()),
    )
    .map(drop)
}

/// Validates one commit, performs final raw-document revalidation, and returns
/// only the work added to the supplied command-scoped cache by this call.
pub fn validate_sealed_commit_with_cache_and_work(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
) -> Result<SealedValidationWork, StoreError> {
    let earlier = cache.work;
    validate_sealed_commit_bound_with_cache_and_hooks(
        objects_root,
        commit_id,
        cache,
        |_| Ok(()),
        |_| Ok(()),
    )?;
    Ok(cache.work.difference(earlier))
}

/// Starts a command-scoped session for efficiently validating multiple heads
/// against one object-store snapshot.
pub fn begin_sealed_commit_validation_session(
    objects_root: &Path,
) -> Result<SealedCommitValidationSession, StoreError> {
    let mut cache = SealedCommitValidationCache::default();
    let bound_store = bind_object_store(objects_root.to_path_buf(), &mut cache)?;
    ensure_bound_object_store_identity(&bound_store, &cache)?;
    Ok(SealedCommitValidationSession {
        subject: Arc::clone(&bound_store.subject),
        bound_store,
        has_existing_subject: true,
        cache,
        staged_subjects: HashSet::new(),
        finalized: false,
        validation_failed: false,
        terminal_failure: None,
    })
}

/// Starts a sealed-validation session whose record reads, semantic
/// validations, memo reuse, and final snapshot revalidation are charged to
/// one caller-created command ledger before each corresponding effect.
pub fn begin_sealed_commit_validation_session_with_ledger(
    objects_root: &Path,
    ledger: codefire_core::DecisionWorkLedger,
) -> Result<SealedCommitValidationSession, StoreError> {
    begin_sealed_commit_validation_session_with_recoverable_ledger(objects_root, ledger).map_err(
        |error| {
            let (error, _) = error.into_parts();
            error
        },
    )
}

pub fn begin_sealed_commit_validation_session_with_recoverable_ledger(
    objects_root: &Path,
    ledger: codefire_core::DecisionWorkLedger,
) -> Result<SealedCommitValidationSession, SealedValidationSessionBeginError> {
    begin_sealed_commit_validation_session_with_recoverable_ledger_and_cancellation(
        objects_root,
        ledger,
        || false,
    )
}

pub fn begin_sealed_commit_validation_session_with_recoverable_ledger_and_cancellation<C>(
    objects_root: &Path,
    ledger: codefire_core::DecisionWorkLedger,
    mut should_cancel: C,
) -> Result<SealedCommitValidationSession, SealedValidationSessionBeginError>
where
    C: FnMut() -> bool,
{
    let mut cache = SealedCommitValidationCache {
        decision_work_ledger: Some(ledger),
        ..SealedCommitValidationCache::default()
    };
    let mut ledger = cache
        .decision_work_ledger
        .take()
        .expect("recoverable session begin retains its input ledger");
    let root_retained_bytes = match object_store_scan_name_frame_bytes(objects_root) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Err(SealedValidationSessionBeginError {
                error: Box::new(error),
                ledger: Box::new(ledger),
            });
        }
    };
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            root_retained_bytes,
        ),
    ) {
        return Err(SealedValidationSessionBeginError {
            error: Box::new(StoreError::from(error)),
            ledger: Box::new(ledger),
        });
    }
    let binding = bind_object_store_with_work_and_cancellation(
        objects_root.to_path_buf(),
        &mut cache,
        &mut ledger,
        &mut should_cancel,
    );
    cache.decision_work_ledger = Some(ledger);
    let bound_store = match binding {
        Ok((bound_store, retained_bytes)) => {
            cache.retained_logical_bytes = retained_bytes
                .checked_add(root_retained_bytes)
                .expect("bounded root and membership ownership fit u64");
            bound_store
        }
        Err(error) => {
            let release = cache
                .decision_work_ledger
                .as_mut()
                .expect("recoverable session begin retains its input ledger")
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    root_retained_bytes,
                );
            let error = match release {
                Ok(()) => error,
                Err(release_error) => StoreError::from(release_error),
            };
            return Err(SealedValidationSessionBeginError {
                error: Box::new(error),
                ledger: Box::new(
                    cache
                        .decision_work_ledger
                        .take()
                        .expect("recoverable session begin retains its input ledger"),
                ),
            });
        }
    };
    if let Err(error) =
        ensure_bound_object_store_identity_accounted(&bound_store, &mut cache, &mut should_cancel)
    {
        drop(bound_store);
        drop(std::mem::take(&mut cache.pinned_object_stores));
        drop(std::mem::take(&mut cache.validated_subjects));
        drop(std::mem::take(&mut cache.retained_documents));
        let retained_bytes = std::mem::take(&mut cache.retained_logical_bytes);
        let mut ledger = cache
            .decision_work_ledger
            .take()
            .expect("recoverable session begin retains its input ledger");
        let error = if retained_bytes == 0 {
            error
        } else {
            match ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            ) {
                Ok(()) => error,
                Err(release_error) => StoreError::from(release_error),
            }
        };
        return Err(SealedValidationSessionBeginError {
            error: Box::new(error),
            ledger: Box::new(ledger),
        });
    }
    Ok(SealedCommitValidationSession {
        subject: Arc::clone(&bound_store.subject),
        bound_store,
        has_existing_subject: true,
        cache,
        staged_subjects: HashSet::new(),
        finalized: false,
        validation_failed: false,
        terminal_failure: None,
    })
}

/// Starts one sealed-validation session over immutable incoming documents layered over an
/// optional existing object-store subject. Overlay documents win by public object ID and are not
/// filesystem-revalidated; existing documents retain the normal pinned-subject final check.
/// The opaque input binds the exact retained-byte reservation to its documents and ledger.
pub fn begin_sealed_commit_overlay_validation_session_with_recoverable_ledger(
    objects_root: &Path,
    input: SealedCommitOverlayValidationInput,
) -> Result<SealedCommitValidationSession, SealedCommitOverlayValidationError> {
    begin_sealed_commit_overlay_validation_session_with_recoverable_ledger_and_cancellation(
        objects_root,
        input,
        || false,
    )
}

pub fn begin_sealed_commit_overlay_validation_session_with_recoverable_ledger_and_cancellation<F>(
    objects_root: &Path,
    input: SealedCommitOverlayValidationInput,
    mut should_cancel: F,
) -> Result<SealedCommitValidationSession, SealedCommitOverlayValidationError>
where
    F: FnMut() -> bool,
{
    let SealedCommitOverlayValidationInput {
        documents,
        logical_bytes: overlay_logical_bytes,
        ledger,
    } = input;
    let mut cache = SealedCommitValidationCache {
        decision_work_ledger: Some(ledger),
        retained_logical_bytes: overlay_logical_bytes,
        ..SealedCommitValidationCache::default()
    };
    let (bound_store, subject, has_existing_subject) = if objects_root.exists() {
        let bound_store = match bind_object_store(objects_root.to_path_buf(), &mut cache) {
            Ok(bound_store) => bound_store,
            Err(error) => {
                return Err(SealedCommitOverlayValidationError {
                    error: Box::new(error),
                    documents: documents
                        .into_iter()
                        .map(|(document, _)| document)
                        .collect(),
                    ledger: Box::new(
                        cache
                            .decision_work_ledger
                            .take()
                            .expect("recoverable overlay begin retains its input ledger"),
                    ),
                    retained_bytes: overlay_logical_bytes,
                });
            }
        };
        let subject = Arc::clone(&bound_store.subject);
        (bound_store, subject, true)
    } else {
        let subject = Arc::new(ObjectStoreSubject {
            canonical_root: objects_root.to_path_buf(),
            identity: ObjectStoreIdentity::Overlay,
            subdirs: Vec::new(),
        });
        let parent = match objects_root.ancestors().skip(1).find(|path| path.is_dir()) {
            Some(parent) => parent,
            None => {
                return Err(SealedCommitOverlayValidationError {
                    error: Box::new(StoreError::InvalidSealedCommit(
                        "overlay object store path has no existing ancestor".to_string(),
                    )),
                    documents: documents
                        .into_iter()
                        .map(|(document, _)| document)
                        .collect(),
                    ledger: Box::new(
                        cache
                            .decision_work_ledger
                            .take()
                            .expect("recoverable overlay begin retains its input ledger"),
                    ),
                    retained_bytes: overlay_logical_bytes,
                });
            }
        };
        let handle = match open_absolute_directory_nofollow(parent) {
            Ok(handle) => handle,
            Err(error) => {
                return Err(SealedCommitOverlayValidationError {
                    error: Box::new(error),
                    documents: documents
                        .into_iter()
                        .map(|(document, _)| document)
                        .collect(),
                    ledger: Box::new(
                        cache
                            .decision_work_ledger
                            .take()
                            .expect("recoverable overlay begin retains its input ledger"),
                    ),
                    retained_bytes: overlay_logical_bytes,
                });
            }
        };
        (
            BoundObjectStore {
                subject: Arc::clone(&subject),
                handle,
                subdirs: HashMap::new(),
            },
            subject,
            false,
        )
    };
    let mut overlay_documents = BTreeMap::new();
    let mut overlay_document_ids = HashSet::new();
    let mut pending_documents = documents.into_iter();
    while let Some((document, logical_bytes)) = pending_documents.next() {
        if should_cancel() {
            let mut recovered = overlay_documents
                .into_values()
                .map(|(document, _)| {
                    Arc::into_inner(document)
                        .expect("unpublished overlay document must have one owner")
                })
                .collect::<Vec<_>>();
            recovered.push(document);
            recovered.extend(pending_documents.map(|(document, _)| document));
            return Err(SealedCommitOverlayValidationError {
                error: Box::new(StoreError::ValidationCancelled),
                documents: recovered,
                ledger: Box::new(
                    cache
                        .decision_work_ledger
                        .take()
                        .expect("cancelled overlay begin retains its input ledger"),
                ),
                retained_bytes: overlay_logical_bytes,
            });
        }
        let object_id = document.record.object_id.clone();
        overlay_document_ids.insert(object_id.clone());
        overlay_documents.insert(object_id, (Arc::new(document), logical_bytes));
    }
    cache.overlay_document_ids = overlay_document_ids;
    cache.overlay_documents = overlay_documents;
    Ok(SealedCommitValidationSession {
        bound_store,
        subject,
        has_existing_subject,
        cache,
        staged_subjects: HashSet::new(),
        finalized: false,
        validation_failed: false,
        terminal_failure: None,
    })
}

#[cfg(test)]
fn validate_sealed_commit_with_cache_and_hook<F>(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    before_final_identity_check: F,
) -> Result<(), StoreError>
where
    F: FnOnce(&Path) -> Result<(), StoreError>,
{
    validate_sealed_commit_bound_with_cache_and_hooks(
        objects_root,
        commit_id,
        cache,
        |_| Ok(()),
        before_final_identity_check,
    )
    .map(drop)
}

#[cfg(test)]
fn validate_sealed_commit_with_cache_and_hooks<B, F>(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    after_binding_before_validation: B,
    before_final_identity_check: F,
) -> Result<(), StoreError>
where
    B: FnOnce(&Path) -> Result<(), StoreError>,
    F: FnOnce(&Path) -> Result<(), StoreError>,
{
    validate_sealed_commit_bound_with_cache_and_hooks(
        objects_root,
        commit_id,
        cache,
        after_binding_before_validation,
        before_final_identity_check,
    )
    .map(drop)
}

fn validate_sealed_commit_bound_with_cache_and_hooks<B, F>(
    objects_root: &Path,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    after_binding_before_validation: B,
    before_final_identity_check: F,
) -> Result<BoundObjectStore, StoreError>
where
    B: FnOnce(&Path) -> Result<(), StoreError>,
    F: FnOnce(&Path) -> Result<(), StoreError>,
{
    let canonical_root = objects_root.to_path_buf();
    let bound_store = match bind_object_store(canonical_root, cache) {
        Ok(bound_store) => bound_store,
        Err(StoreError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(StoreError::ObjectNotFound(commit_id.to_string()))
        }
        Err(error) => return Err(error),
    };
    ensure_bound_object_store_identity(&bound_store, cache)?;
    after_binding_before_validation(&bound_store.subject.canonical_root)?;

    // Fully validated ancestors are staged until the filesystem subject is rechecked. This keeps
    // useful DAG cache entries after an ordinary validation failure without allowing a directory
    // replacement during traversal to poison the reusable cache.
    let mut staged_subjects = HashSet::new();
    let mut visiting = HashSet::new();
    let validation = validate_sealed_commit_inner(
        &bound_store,
        &bound_store.subject,
        commit_id,
        cache,
        &mut staged_subjects,
        &mut visiting,
    );
    before_final_identity_check(&bound_store.subject.canonical_root)?;
    finalize_sealed_validation_subject(&bound_store, cache, &mut staged_subjects)?;
    validation?;
    Ok(bound_store)
}

fn finalize_sealed_validation_subject(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
) -> Result<(), StoreError> {
    finalize_sealed_validation_subject_with_cancellation(
        bound_store,
        cache,
        staged_subjects,
        &mut || false,
    )
}

fn finalize_sealed_validation_subject_with_cancellation<F>(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    F: FnMut() -> bool,
{
    if let Err(error) =
        ensure_bound_object_store_identity_accounted(bound_store, cache, should_cancel)
    {
        invalidate_sealed_validation_subject(cache, &bound_store.subject);
        return Err(error);
    }
    if let Err(error) =
        revalidate_retained_object_documents_with_cancellation(bound_store, cache, should_cancel)
    {
        invalidate_sealed_validation_subject(cache, &bound_store.subject);
        return Err(error);
    }
    // The second identity check closes a directory/subdirectory replacement
    // window opened while the retained files were being read.
    if let Err(error) =
        ensure_bound_object_store_identity_accounted(bound_store, cache, should_cancel)
    {
        invalidate_sealed_validation_subject(cache, &bound_store.subject);
        return Err(error);
    }

    if bound_store.subject.identity.is_stable() {
        while let Some(staged_subject) = staged_subjects.iter().next().cloned() {
            if should_cancel() {
                invalidate_sealed_validation_subject(cache, &bound_store.subject);
                staged_subjects.clear();
                return Err(StoreError::ValidationCancelled);
            }
            staged_subjects.remove(&staged_subject);
            cache.validated_subjects.insert(staged_subject);
        }
    } else {
        // Platforms without a stable directory identity still get per-session DAG reuse, but
        // never carry path-based validation results, documents, or a synthetic pin forward.
        cache.pinned_object_stores.remove(&bound_store.subject);
        cache.retained_documents.clear();
        staged_subjects.clear();
    }
    Ok(())
}

fn ensure_bound_object_store_identity_accounted<C>(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    let mut ledger = cache.decision_work_ledger.take();
    let result = match ledger.as_mut() {
        Some(ledger) => ensure_bound_object_store_identity_with_work_and_cancellation(
            bound_store,
            cache,
            ledger,
            should_cancel,
        ),
        None => {
            poll_object_store_scan(should_cancel)?;
            let result = ensure_bound_object_store_identity(bound_store, cache);
            poll_object_store_scan(should_cancel)?;
            result
        }
    };
    cache.decision_work_ledger = ledger;
    result
}

fn invalidate_sealed_validation_subject(
    cache: &mut SealedCommitValidationCache,
    subject: &Arc<ObjectStoreSubject>,
) {
    cache
        .validated_subjects
        .retain(|(cached_subject, _)| cached_subject != subject);
    cache.retained_documents.clear();
}

fn revalidate_retained_object_documents_with_cancellation<F>(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    F: FnMut() -> bool,
{
    // Move the already-accounted retained documents out temporarily so the ledger and work
    // counters can be mutated while their immutable fingerprints are inspected. This avoids an
    // additional O(records * path length) fingerprint clone that would otherwise need its own
    // temporary-memory reservation.
    let retained_documents = std::mem::take(&mut cache.retained_documents);
    let result = revalidate_retained_object_documents_inner(
        bound_store,
        cache,
        &retained_documents,
        should_cancel,
    );
    cache.retained_documents = retained_documents;
    result
}

fn revalidate_retained_object_documents_inner<F>(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    retained_documents: &BTreeMap<String, RetainedObjectDocument>,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    F: FnMut() -> bool,
{
    for (object_id, retained) in retained_documents {
        if should_cancel() {
            return Err(StoreError::ValidationCancelled);
        }
        if cache.overlay_document_ids.contains(object_id) {
            continue;
        }
        let relative_directory = &retained.relative_directory;
        let file_name = &retained.file_name;
        let display_path = &retained.display_path;
        let expected_size = retained.size_bytes;
        let expected_digest = &retained.document_digest;
        let Some(directory) = bound_store
            .subdirs
            .get(relative_directory)
            .and_then(Option::as_ref)
        else {
            return Err(StoreError::SealedValidationSubjectChanged {
                object_id: object_id.clone(),
                path: display_path.clone(),
                kind: SealedValidationSubjectChangeKind::Unreadable,
                expected_size,
                actual_size: None,
            });
        };
        let mut scratch_bytes = 0_u64;
        let bytes = match read_object_record_bytes_from_directory_with_limit_and_hook(
            directory,
            file_name,
            display_path,
            MAX_OBJECT_RECORD_BYTES,
            |size_bytes| {
                charge_snapshot_revalidation(cache, size_bytes)?;
                scratch_bytes = size_bytes;
                checked_work_add(&mut cache.work.revalidated_documents, 1)
            },
        ) {
            Ok(bytes) => bytes,
            Err(error @ StoreError::DecisionWorkAccounting(_)) => return Err(error),
            Err(_) => {
                let actual_size = directory
                    .symlink_metadata(file_name)
                    .ok()
                    .map(|metadata| metadata.len());
                let kind = if actual_size.is_some_and(|size| size != expected_size) {
                    SealedValidationSubjectChangeKind::SizeChanged
                } else {
                    SealedValidationSubjectChangeKind::Unreadable
                };
                return Err(StoreError::SealedValidationSubjectChanged {
                    object_id: object_id.clone(),
                    path: display_path.clone(),
                    kind,
                    expected_size,
                    actual_size,
                });
            }
        };
        let actual_size = u64::try_from(bytes.len()).map_err(|_| {
            StoreError::InvalidSealedCommit(
                "sealed validation document size does not fit u64".to_string(),
            )
        })?;
        checked_work_add(&mut cache.work.revalidated_bytes, actual_size)?;
        let actual_digest = codefire_util::sha256_hex(&bytes);
        drop(bytes);
        release_snapshot_revalidation_scratch(cache, scratch_bytes)?;
        if actual_size != expected_size {
            return Err(StoreError::SealedValidationSubjectChanged {
                object_id: object_id.clone(),
                path: display_path.clone(),
                kind: SealedValidationSubjectChangeKind::SizeChanged,
                expected_size,
                actual_size: Some(actual_size),
            });
        }
        if actual_digest != *expected_digest {
            return Err(StoreError::SealedValidationSubjectChanged {
                object_id: object_id.clone(),
                path: display_path.clone(),
                kind: SealedValidationSubjectChangeKind::DigestChanged,
                expected_size,
                actual_size: Some(actual_size),
            });
        }
    }
    Ok(())
}

fn validate_sealed_commit_inner(
    bound_store: &BoundObjectStore,
    object_store_subject: &Arc<ObjectStoreSubject>,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
) -> Result<(), StoreError> {
    validate_sealed_commit_inner_with_cancellation(
        bound_store,
        object_store_subject,
        commit_id,
        cache,
        staged_subjects,
        visiting,
        &mut || false,
    )
}

fn validate_sealed_commit_inner_with_cancellation<F>(
    bound_store: &BoundObjectStore,
    object_store_subject: &Arc<ObjectStoreSubject>,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    F: FnMut() -> bool,
{
    validate_sealed_commit_inner_with_reader_and_cancellation(
        bound_store,
        (object_store_subject, commit_id),
        cache,
        staged_subjects,
        visiting,
        |object_id, cache| read_bound_object_memoized(bound_store, object_id, cache),
        should_cancel,
    )
}

struct PendingSealedCommitValidation<C> {
    commit_id: String,
    commit: C,
    semantic_validation_profile: SemanticValidationProfile,
    next_parent_index: usize,
    depth: u64,
}

enum SealedCommitValidationFrame<C> {
    Enter(String, u64),
    VisitParents(PendingSealedCommitValidation<C>),
    Exit(PendingSealedCommitValidation<C>),
}

#[cfg(test)]
fn validate_sealed_commit_inner_with_reader<R, C>(
    bound_store: &BoundObjectStore,
    object_store_subject: &Arc<ObjectStoreSubject>,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
    read_commit: R,
) -> Result<(), StoreError>
where
    R: FnMut(&str, &mut SealedCommitValidationCache) -> Result<C, StoreError>,
    C: ObjectPayloadView,
{
    validate_sealed_commit_inner_with_reader_and_cancellation(
        bound_store,
        (object_store_subject, commit_id),
        cache,
        staged_subjects,
        visiting,
        read_commit,
        &mut || false,
    )
}

fn validate_sealed_commit_inner_with_reader_and_cancellation<R, C, F>(
    bound_store: &BoundObjectStore,
    subject: (&Arc<ObjectStoreSubject>, &str),
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
    read_commit: R,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    R: FnMut(&str, &mut SealedCommitValidationCache) -> Result<C, StoreError>,
    C: ObjectPayloadView,
    F: FnMut() -> bool,
{
    let (object_store_subject, commit_id) = subject;
    validate_sealed_commit_inner_with_handlers_and_cancellation(
        (object_store_subject, commit_id),
        cache,
        staged_subjects,
        visiting,
        read_commit,
        |commit, semantic_validation_profile, cache| {
            validate_sealed_commit_postorder(
                bound_store,
                commit,
                semantic_validation_profile,
                cache,
            )
        },
        should_cancel,
    )
}

#[cfg(test)]
fn validate_sealed_commit_inner_with_handlers<R, P, C>(
    object_store_subject: &Arc<ObjectStoreSubject>,
    commit_id: &str,
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
    read_commit: R,
    validate_postorder: P,
) -> Result<(), StoreError>
where
    R: FnMut(&str, &mut SealedCommitValidationCache) -> Result<C, StoreError>,
    C: ObjectPayloadView,
    P: FnMut(
        &Value,
        SemanticValidationProfile,
        &mut SealedCommitValidationCache,
    ) -> Result<(), StoreError>,
{
    validate_sealed_commit_inner_with_handlers_and_cancellation(
        (object_store_subject, commit_id),
        cache,
        staged_subjects,
        visiting,
        read_commit,
        validate_postorder,
        &mut || false,
    )
}

fn validate_sealed_commit_inner_with_handlers_and_cancellation<R, P, C, F>(
    subject: (&Arc<ObjectStoreSubject>, &str),
    cache: &mut SealedCommitValidationCache,
    staged_subjects: &mut HashSet<(Arc<ObjectStoreSubject>, String)>,
    visiting: &mut HashSet<String>,
    mut read_commit: R,
    mut validate_postorder: P,
    should_cancel: &mut F,
) -> Result<(), StoreError>
where
    R: FnMut(&str, &mut SealedCommitValidationCache) -> Result<C, StoreError>,
    C: ObjectPayloadView,
    P: FnMut(
        &Value,
        SemanticValidationProfile,
        &mut SealedCommitValidationCache,
    ) -> Result<(), StoreError>,
    F: FnMut() -> bool,
{
    let (object_store_subject, commit_id) = subject;
    charge_traversal_frames(cache, 0, 1, false, &[commit_id])?;
    let mut charged_frame_ids = vec![commit_id.to_string()];
    let mut inserted_visiting_ids = Vec::new();
    let result = (|| {
        let mut frames = vec![SealedCommitValidationFrame::Enter(commit_id.to_string(), 0)];
        while let Some(frame) = frames.pop() {
            if should_cancel() {
                return Err(StoreError::ValidationCancelled);
            }
            match frame {
                SealedCommitValidationFrame::Enter(commit_id, depth) => {
                    let subject = (Arc::clone(object_store_subject), commit_id.clone());
                    if cache.validated_subjects.contains(&subject)
                        || staged_subjects.contains(&subject)
                    {
                        charge_memo_hit(
                            cache,
                            codefire_core::DecisionWorkPhase::SemanticValidation,
                        )?;
                        checked_work_add(&mut cache.work.validated_commit_hits, 1)?;
                        #[cfg(test)]
                        {
                            cache.cache_hits += 1;
                        }
                        release_tracked_traversal_frame(cache, &mut charged_frame_ids, &commit_id)?;
                        continue;
                    }
                    if !visiting.insert(commit_id.clone()) {
                        return Err(StoreError::InvalidSealedCommit(format!(
                            "commit parent cycle detected at {commit_id}"
                        )));
                    }
                    inserted_visiting_ids.push(commit_id.clone());
                    let commit = read_commit(&commit_id, cache)?;
                    if commit.payload().get("type").and_then(Value::as_str) != Some("commit") {
                        return Err(StoreError::InvalidSealedCommit(format!(
                            "not a commit object: {commit_id}"
                        )));
                    }
                    require_supported_version(commit.payload(), "commit")?;
                    let semantic_validation_profile =
                        commit_semantic_validation_profile(commit.payload())?;
                    commit
                        .payload()
                        .get("parents")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            StoreError::InvalidSealedCommit("parents must be a list".to_string())
                        })?;
                    frames.push(SealedCommitValidationFrame::VisitParents(
                        PendingSealedCommitValidation {
                            commit_id,
                            commit,
                            semantic_validation_profile,
                            next_parent_index: 0,
                            depth,
                        },
                    ));
                }
                SealedCommitValidationFrame::VisitParents(mut pending) => {
                    let parents = pending
                        .commit
                        .payload()
                        .get("parents")
                        .and_then(Value::as_array)
                        .expect("parent list was checked on enter");
                    let Some(parent) = parents.get(pending.next_parent_index) else {
                        frames.push(SealedCommitValidationFrame::Exit(pending));
                        continue;
                    };
                    let parent_index = pending.next_parent_index;
                    let parent_id = parent.as_str().ok_or_else(|| {
                        StoreError::InvalidSealedCommit(format!(
                            "parent {parent_index} must be a commit object id"
                        ))
                    })?;
                    if !parent_id.starts_with("CF-COMMIT-") {
                        return Err(StoreError::InvalidSealedCommit(format!(
                            "parent {parent_index} must be a commit object id"
                        )));
                    }
                    let parent_id = parent_id.to_string();
                    let parent_depth = pending.depth.checked_add(1).ok_or_else(|| {
                        StoreError::InvalidSealedCommit(
                            "sealed validation logical depth overflow".to_string(),
                        )
                    })?;
                    let frontier = u64::try_from(frames.len().saturating_add(2)).map_err(|_| {
                        StoreError::InvalidSealedCommit(
                            "sealed validation frontier does not fit u64".to_string(),
                        )
                    })?;
                    charge_traversal_frames(cache, parent_depth, frontier, true, &[&parent_id])?;
                    charged_frame_ids.push(parent_id.clone());
                    pending.next_parent_index += 1;
                    frames.push(SealedCommitValidationFrame::VisitParents(pending));
                    // Enter is deliberately pushed after the continuation. LIFO evaluation therefore
                    // preserves recursive DFS and validates the full current parent subtree before
                    // even parsing the next parent-array element.
                    frames.push(SealedCommitValidationFrame::Enter(parent_id, parent_depth));
                }
                SealedCommitValidationFrame::Exit(pending) => {
                    charge_semantic_validation(cache)?;
                    checked_work_add(&mut cache.work.semantic_commit_validations, 1)?;
                    validate_postorder(
                        pending.commit.payload(),
                        pending.semantic_validation_profile,
                        cache,
                    )?;
                    visiting.remove(&pending.commit_id);
                    if let Some(position) = inserted_visiting_ids
                        .iter()
                        .rposition(|inserted| inserted == &pending.commit_id)
                    {
                        inserted_visiting_ids.remove(position);
                    }
                    release_tracked_traversal_frame(
                        cache,
                        &mut charged_frame_ids,
                        &pending.commit_id,
                    )?;
                    staged_subjects.insert((Arc::clone(object_store_subject), pending.commit_id));
                }
            }
        }
        Ok(())
    })();

    if result.is_err() {
        for commit_id in inserted_visiting_ids.drain(..) {
            visiting.remove(&commit_id);
        }
        let mut cleanup_error = None;
        for frame_id in charged_frame_ids.drain(..).rev() {
            if let Err(error) = release_traversal_frame(cache, &frame_id) {
                if cleanup_error.is_none() {
                    cleanup_error = Some(error);
                }
            }
        }
        if let Some(error) = cleanup_error {
            return Err(error);
        }
    }
    result
}

fn validate_sealed_commit_postorder(
    bound_store: &BoundObjectStore,
    commit: &Value,
    semantic_validation_profile: SemanticValidationProfile,
    cache: &mut SealedCommitValidationCache,
) -> Result<(), StoreError> {
    let roots = commit
        .get("roots")
        .and_then(Value::as_object)
        .ok_or_else(|| StoreError::InvalidSealedCommit("roots must be an object".to_string()))?;
    let root_schema_version = commit_root_schema_version(commit)?;
    for root in required_commit_roots(root_schema_version) {
        if !roots.contains_key(*root) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "missing root {root}"
            )));
        }
    }
    let mut atom_index_root = None;
    let mut trace_graph_root = None;
    let mut fire_ledger_root = None;
    let mut resolution_ledger_root = None;
    let mut verification_root = None;
    let mut policy_root = None;
    for (root, object_id) in roots {
        let object_id = object_id.as_str().ok_or_else(|| {
            StoreError::InvalidSealedCommit(format!("root {root} must be an object id"))
        })?;
        if !object_id.starts_with("CF-") {
            return Err(StoreError::InvalidSealedCommit(format!(
                "root {root} must be an object id"
            )));
        }
        charge_root_edge(cache)?;
        let payload = read_bound_object_memoized(bound_store, object_id, cache)?;
        if let Some(expected_type) = expected_root_type(root) {
            let actual_type = payload
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("(missing)");
            if actual_type != expected_type {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "root {root} has type {actual_type}, expected {expected_type}"
                )));
            }
        }
        validate_root_payload(root, &payload, root_schema_version)?;
        validate_root_references(bound_store, cache, root, &payload)?;
        match root.as_str() {
            "atom_index" => atom_index_root = Some(payload),
            "trace_graph" => trace_graph_root = Some(payload),
            "fire_delta" => fire_ledger_root = Some(payload),
            "resolution_ledger" => resolution_ledger_root = Some(payload),
            "verification" => verification_root = Some(payload),
            "policy" => policy_root = Some(payload),
            _ => {}
        }
    }
    if semantic_validation_profile == SemanticValidationProfile::SealedSnapshotV1 {
        validate_trace_graph_atom_references(
            atom_index_root.as_ref().ok_or_else(|| {
                StoreError::InvalidSealedCommit("missing root atom_index".to_string())
            })?,
            trace_graph_root.as_ref().ok_or_else(|| {
                StoreError::InvalidSealedCommit("missing root trace_graph".to_string())
            })?,
        )?;
    }
    if root_schema_version == 2 {
        validate_current_sealed_snapshot_semantics(
            atom_index_root.as_ref().expect("required root was checked"),
            trace_graph_root
                .as_ref()
                .expect("required root was checked"),
            fire_ledger_root
                .as_ref()
                .expect("required root was checked"),
            resolution_ledger_root
                .as_ref()
                .expect("required root was checked"),
            verification_root
                .as_ref()
                .expect("required root was checked"),
            policy_root.as_ref().expect("required root was checked"),
        )?;
    }

    let certificate = commit
        .get("certificate")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit("certificate must be an object".to_string())
        })?;
    if !matches!(
        certificate.get("result").and_then(Value::as_str),
        Some("passed" | "consistent")
    ) {
        return Err(StoreError::InvalidSealedCommit(
            "certificate result is not passed".to_string(),
        ));
    }
    let verification = verification_root
        .as_ref()
        .ok_or_else(|| StoreError::InvalidSealedCommit("missing root verification".to_string()))?;
    validate_certificate_matches_verification(certificate, verification, root_schema_version)?;
    Ok(())
}

fn bind_object_store(
    canonical_root: PathBuf,
    cache: &mut SealedCommitValidationCache,
) -> Result<BoundObjectStore, StoreError> {
    let handle = open_absolute_directory_nofollow(&canonical_root)?;
    let metadata = handle.dir_metadata()?;
    require_capability_directory_metadata(&canonical_root, &metadata)?;
    let identity = capability_object_store_identity(&metadata, cache)?;
    let subdir_binding = bind_object_store_subdirs(&handle, &canonical_root, cache)?;
    let pinned = PinnedObjectStore {
        handle,
        subdirs: subdir_binding.handles,
    };

    let subject = Arc::new(ObjectStoreSubject {
        canonical_root,
        identity,
        subdirs: subdir_binding.subjects,
    });
    if !cache.pinned_object_stores.contains_key(&subject) {
        // A validation cache is command-scoped and only retains one filesystem subject. Keeping
        // prior subjects would retain their directory descriptors indefinitely as callers move
        // through unrelated repositories. Evict their validation results together with the pins;
        // a later return to that subject safely recomputes the graph.
        cache.validated_subjects.clear();
        cache.retained_documents.clear();
        cache.pinned_object_stores.clear();
    }
    let pinned = cache
        .pinned_object_stores
        .entry(Arc::clone(&subject))
        .or_insert(pinned);
    let mut subdirs = HashMap::with_capacity(pinned.subdirs.len());
    for (name, directory) in &pinned.subdirs {
        subdirs.insert(
            name.clone(),
            directory.as_ref().map(Dir::try_clone).transpose()?,
        );
    }
    Ok(BoundObjectStore {
        subject,
        handle: pinned.handle.try_clone()?,
        subdirs,
    })
}

fn bind_object_store_with_work_and_cancellation<C>(
    canonical_root: PathBuf,
    cache: &mut SealedCommitValidationCache,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(BoundObjectStore, u64), StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(should_cancel)?;
    let handle = open_absolute_directory_nofollow(&canonical_root)?;
    poll_object_store_scan(should_cancel)?;
    let metadata = handle.dir_metadata()?;
    require_capability_directory_metadata(&canonical_root, &metadata)?;
    let identity = capability_object_store_identity(&metadata, cache)?;
    let (subdir_binding, subdir_retained_bytes, name_owner_bytes) =
        bind_object_store_subdirs_with_work_and_cancellation(
            &handle,
            &canonical_root,
            cache,
            ledger,
            should_cancel,
        )?;
    let bound_owner_bytes = name_owner_bytes
        .checked_add(
            codefire_core::LOGICAL_FRAME_BASE_BYTES
                .checked_mul(2)
                .ok_or_else(|| {
                    StoreError::InvalidSealedCommit(
                        "object-store binding container ownership overflow".to_string(),
                    )
                })?,
        )
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit("object-store binding ownership overflow".to_string())
        })?;
    let retained_bytes = subdir_retained_bytes
        .checked_add(bound_owner_bytes)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "object-store binding retained ownership overflow".to_string(),
            )
        })?;
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            bound_owner_bytes,
        ),
    ) {
        release_object_store_scan_retained(ledger, subdir_retained_bytes)?;
        return Err(error.into());
    }
    let built = (|| {
        let pinned = PinnedObjectStore {
            handle,
            subdirs: subdir_binding.handles,
        };

        let subject = Arc::new(ObjectStoreSubject {
            canonical_root,
            identity,
            subdirs: subdir_binding.subjects,
        });
        if !cache.pinned_object_stores.contains_key(&subject) {
            cache.validated_subjects.clear();
            cache.retained_documents.clear();
            cache.pinned_object_stores.clear();
        }
        let pinned = cache
            .pinned_object_stores
            .entry(Arc::clone(&subject))
            .or_insert(pinned);
        let mut subdirs = HashMap::with_capacity(pinned.subdirs.len());
        for (name, directory) in &pinned.subdirs {
            poll_object_store_scan(should_cancel)?;
            subdirs.insert(
                name.clone(),
                directory.as_ref().map(Dir::try_clone).transpose()?,
            );
        }
        poll_object_store_scan(should_cancel)?;
        Ok(BoundObjectStore {
            subject,
            handle: pinned.handle.try_clone()?,
            subdirs,
        })
    })();
    match built {
        Ok(bound) => Ok((bound, retained_bytes)),
        Err(error) => {
            release_object_store_scan_retained(ledger, retained_bytes)?;
            Err(error)
        }
    }
}

fn bind_object_store_subdirs(
    handle: &Dir,
    canonical_root: &Path,
    cache: &mut SealedCommitValidationCache,
) -> Result<ObjectStoreSubdirBinding, StoreError> {
    let known_names = known_object_subdirs()
        .map(PathBuf::from)
        .collect::<BTreeSet<_>>();
    let mut extension_names = BTreeSet::new();
    for entry in handle.entries()? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = PathBuf::from(entry.file_name());
        if known_names.contains(&name) {
            continue;
        }
        extension_names.insert(name);
        if extension_names.len() > MAX_PINNED_EXTENSION_OBJECT_SUBDIRS {
            return Err(StoreError::InvalidSealedCommit(format!(
                "object store extension subdirectory limit exceeded: found at least {}, limit {}: {}",
                extension_names.len(),
                MAX_PINNED_EXTENSION_OBJECT_SUBDIRS,
                canonical_root.display()
            )));
        }
    }
    let mut names = known_names;
    names.extend(extension_names);

    let mut subjects = Vec::with_capacity(names.len());
    let mut pinned = HashMap::with_capacity(names.len());
    for name in names {
        let directory = match handle.open_dir_nofollow(&name) {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(StoreError::Io(error)),
        };
        let identity = directory
            .as_ref()
            .map(|directory| {
                let metadata = directory.dir_metadata()?;
                require_capability_directory_metadata(&canonical_root.join(&name), &metadata)?;
                capability_object_store_identity(&metadata, cache)
            })
            .transpose()?;
        subjects.push(ObjectStoreSubdirSubject {
            name: name.clone(),
            identity,
        });
        pinned.insert(name, directory);
    }
    Ok(ObjectStoreSubdirBinding {
        subjects,
        handles: pinned,
    })
}

fn collect_object_store_subdir_names_accounted<C>(
    handle: &Dir,
    canonical_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
    phase: codefire_core::DecisionWorkPhase,
) -> Result<(BTreeMap<PathBuf, u64>, u64), StoreError>
where
    C: FnMut() -> bool,
{
    let mut known_retained_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES;
    for name in known_object_subdirs() {
        known_retained_bytes = known_retained_bytes
            .checked_add(object_store_scan_name_frame_bytes(Path::new(name))?)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(
                    "object-store known subdirectory membership overflow".to_string(),
                )
            })?;
    }
    if known_retained_bytes > 0 {
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                known_retained_bytes,
            ),
        )?;
    }
    let mut names = known_object_subdirs()
        .map(|name| {
            let path = PathBuf::from(name);
            let bytes = object_store_scan_name_frame_bytes(&path)
                .expect("known object subdirectory frame was prevalidated");
            (path, bytes)
        })
        .collect::<BTreeMap<_, _>>();
    let mut retained_bytes = known_retained_bytes;
    let mut extension_count = 0usize;
    let mut iterator = match handle.entries() {
        Ok(iterator) => iterator,
        Err(error) => {
            release_object_store_scan_retained(ledger, retained_bytes)?;
            return Err(error.into());
        }
    };
    let collection = (|| -> Result<(), StoreError> {
        loop {
            let Some(accounted) = next_object_store_directory_entry_accounted(
                &mut iterator,
                ledger,
                should_cancel,
                phase,
            )?
            else {
                break;
            };
            let AccountedObjectStoreDirectoryEntry {
                entry,
                name,
                frame_bytes,
            } = accounted;
            poll_object_store_scan(should_cancel).or_else(|error| {
                release_object_store_scan_temporary(ledger, frame_bytes)?;
                Err(error)
            })?;
            let is_directory = match entry.file_type() {
                Ok(file_type) => file_type.is_dir(),
                Err(error) => {
                    release_object_store_scan_temporary(ledger, frame_bytes)?;
                    return Err(error.into());
                }
            };
            if !is_directory || names.contains_key(&name) {
                discard_object_store_directory_entry_frame(ledger, frame_bytes, phase)?;
                continue;
            }
            extension_count = extension_count.saturating_add(1);
            if extension_count > MAX_PINNED_EXTENSION_OBJECT_SUBDIRS {
                discard_object_store_directory_entry_frame(ledger, frame_bytes, phase)?;
                return Err(StoreError::InvalidSealedCommit(format!(
                    "object store extension subdirectory limit exceeded: found at least {}, limit {}: {}",
                    extension_count,
                    MAX_PINNED_EXTENSION_OBJECT_SUBDIRS,
                    canonical_root.display()
                )));
            }
            let next_retained = match retained_bytes.checked_add(frame_bytes) {
                Some(bytes) => bytes,
                None => {
                    release_object_store_scan_temporary(ledger, frame_bytes)?;
                    return Err(StoreError::InvalidSealedCommit(
                        "object-store subdirectory membership overflow".to_string(),
                    ));
                }
            };
            retain_object_store_directory_entry_frame(ledger, frame_bytes, phase)?;
            retained_bytes = next_retained;
            names.insert(name, frame_bytes);
        }
        Ok(())
    })();
    if let Err(error) = collection {
        drop(names);
        release_object_store_scan_retained(ledger, retained_bytes)?;
        return Err(error);
    }
    Ok((names, retained_bytes))
}

fn bind_object_store_subdirs_with_work_and_cancellation<C>(
    handle: &Dir,
    canonical_root: &Path,
    cache: &mut SealedCommitValidationCache,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(ObjectStoreSubdirBinding, u64, u64), StoreError>
where
    C: FnMut() -> bool,
{
    let (names, name_owner_bytes) = collect_object_store_subdir_names_accounted(
        handle,
        canonical_root,
        ledger,
        should_cancel,
        codefire_core::DecisionWorkPhase::GraphDiscovery,
    )?;
    let retained_bytes = name_owner_bytes.checked_mul(2).ok_or_else(|| {
        StoreError::InvalidSealedCommit(
            "object-store subdirectory binding ownership overflow".to_string(),
        )
    })?;
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            name_owner_bytes,
        ),
    ) {
        release_object_store_scan_retained(ledger, name_owner_bytes)?;
        return Err(error.into());
    }
    let built = (|| {
        let mut subjects = Vec::with_capacity(names.len());
        let mut pinned = HashMap::with_capacity(names.len());
        for (name, _) in names {
            poll_object_store_scan(should_cancel)?;
            let directory = match handle.open_dir_nofollow(&name) {
                Ok(directory) => Some(directory),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => return Err(StoreError::Io(error)),
            };
            let identity = directory
                .as_ref()
                .map(|directory| {
                    poll_object_store_scan(should_cancel)?;
                    let metadata = directory.dir_metadata()?;
                    require_capability_directory_metadata(&canonical_root.join(&name), &metadata)?;
                    capability_object_store_identity(&metadata, cache)
                })
                .transpose()?;
            subjects.push(ObjectStoreSubdirSubject {
                name: name.clone(),
                identity,
            });
            pinned.insert(name, directory);
        }
        Ok(ObjectStoreSubdirBinding {
            subjects,
            handles: pinned,
        })
    })();
    match built {
        Ok(binding) => Ok((binding, retained_bytes, name_owner_bytes)),
        Err(error) => {
            release_object_store_scan_retained(ledger, retained_bytes)?;
            Err(error)
        }
    }
}

fn require_capability_directory_metadata(
    path: &Path,
    metadata: &cap_std::fs::Metadata,
) -> Result<(), StoreError> {
    if metadata.is_dir() {
        Ok(())
    } else {
        Err(StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("object store root is not a directory: {}", path.display()),
        )))
    }
}

fn ensure_bound_object_store_identity(
    bound_store: &BoundObjectStore,
    cache: &SealedCommitValidationCache,
) -> Result<(), StoreError> {
    if !bound_store.subject.identity.is_stable() {
        return Ok(());
    }

    let pinned = cache
        .pinned_object_stores
        .get(&bound_store.subject)
        .ok_or_else(|| object_store_identity_changed(&bound_store.subject.canonical_root))?;
    let pinned_identity = stable_capability_object_store_identity(&pinned.handle.dir_metadata()?)?;
    if pinned_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    let bound_identity =
        stable_capability_object_store_identity(&bound_store.handle.dir_metadata()?)?;
    if bound_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    ensure_bound_object_subdir_identities(bound_store, pinned)?;

    let current = open_absolute_directory_nofollow(&bound_store.subject.canonical_root)
        .map_err(|_| object_store_identity_changed(&bound_store.subject.canonical_root))?;
    let current_identity = stable_capability_object_store_identity(&current.dir_metadata()?)?;
    if current_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    Ok(())
}

fn ensure_bound_object_store_identity_with_work_and_cancellation<C>(
    bound_store: &BoundObjectStore,
    cache: &SealedCommitValidationCache,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    poll_object_store_scan(should_cancel)?;
    if !bound_store.subject.identity.is_stable() {
        return Ok(());
    }

    let pinned = cache
        .pinned_object_stores
        .get(&bound_store.subject)
        .ok_or_else(|| object_store_identity_changed(&bound_store.subject.canonical_root))?;
    poll_object_store_scan(should_cancel)?;
    let pinned_identity = stable_capability_object_store_identity(&pinned.handle.dir_metadata()?)?;
    if pinned_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    poll_object_store_scan(should_cancel)?;
    let bound_identity =
        stable_capability_object_store_identity(&bound_store.handle.dir_metadata()?)?;
    if bound_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    ensure_bound_object_subdir_identities_with_work_and_cancellation(
        bound_store,
        pinned,
        ledger,
        should_cancel,
    )?;

    poll_object_store_scan(should_cancel)?;
    let current = open_absolute_directory_nofollow(&bound_store.subject.canonical_root)
        .map_err(|_| object_store_identity_changed(&bound_store.subject.canonical_root))?;
    let current_identity = stable_capability_object_store_identity(&current.dir_metadata()?)?;
    if current_identity != bound_store.subject.identity {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }
    poll_object_store_scan(should_cancel)
}

fn ensure_validated_subject_identity(bound_store: &BoundObjectStore) -> Result<(), StoreError> {
    if !bound_store.subject.identity.is_stable() {
        return Err(StoreError::InvalidSealedCommit(format!(
            "object store identity cannot be rechecked on this platform: {}",
            bound_store.subject.canonical_root.display()
        )));
    }

    // Reuse the validation identity predicate with an independently cloned pin.
    // The token owns `bound_store`; this temporary cache only supplies the
    // predicate's second retained-handle witness and never carries validation
    // results across calls.
    let mut cache = SealedCommitValidationCache::default();
    let mut subdirs = HashMap::with_capacity(bound_store.subdirs.len());
    for (name, directory) in &bound_store.subdirs {
        subdirs.insert(
            name.clone(),
            directory.as_ref().map(Dir::try_clone).transpose()?,
        );
    }
    cache.pinned_object_stores.insert(
        Arc::clone(&bound_store.subject),
        PinnedObjectStore {
            handle: bound_store.handle.try_clone()?,
            subdirs,
        },
    );
    ensure_bound_object_store_identity(bound_store, &cache)
}

fn ensure_bound_object_subdir_identities(
    bound_store: &BoundObjectStore,
    pinned: &PinnedObjectStore,
) -> Result<(), StoreError> {
    let expected_names = bound_store
        .subject
        .subdirs
        .iter()
        .map(|subject| subject.name.clone())
        .collect::<BTreeSet<_>>();
    let known_names = known_object_subdirs()
        .map(PathBuf::from)
        .collect::<BTreeSet<_>>();
    let mut current_names = known_names.clone();
    let mut extension_count = 0usize;
    for entry in bound_store.handle.entries()? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = PathBuf::from(entry.file_name());
        if !known_names.contains(&name) {
            extension_count = extension_count.saturating_add(1);
            if extension_count > MAX_PINNED_EXTENSION_OBJECT_SUBDIRS {
                return Err(object_store_identity_changed(
                    &bound_store.subject.canonical_root,
                ));
            }
        }
        current_names.insert(name);
    }
    if current_names != expected_names {
        return Err(object_store_identity_changed(
            &bound_store.subject.canonical_root,
        ));
    }

    for expected in &bound_store.subject.subdirs {
        let expected_path = bound_store.subject.canonical_root.join(&expected.name);
        let pinned_directory = pinned
            .subdirs
            .get(&expected.name)
            .ok_or_else(|| object_store_identity_changed(&bound_store.subject.canonical_root))?;
        let bound_directory = bound_store
            .subdirs
            .get(&expected.name)
            .ok_or_else(|| object_store_identity_changed(&bound_store.subject.canonical_root))?;
        match expected.identity {
            Some(identity) => {
                let pinned_identity = pinned_directory
                    .as_ref()
                    .ok_or_else(|| object_store_identity_changed(&expected_path))
                    .and_then(|directory| {
                        stable_capability_object_store_identity(&directory.dir_metadata()?)
                    })?;
                let bound_identity = bound_directory
                    .as_ref()
                    .ok_or_else(|| object_store_identity_changed(&expected_path))
                    .and_then(|directory| {
                        stable_capability_object_store_identity(&directory.dir_metadata()?)
                    })?;
                let current_identity = bound_store
                    .handle
                    .open_dir_nofollow(&expected.name)
                    .map_err(|_| object_store_identity_changed(&expected_path))
                    .and_then(|directory| {
                        stable_capability_object_store_identity(&directory.dir_metadata()?)
                    })?;
                if pinned_identity != identity
                    || bound_identity != identity
                    || current_identity != identity
                {
                    return Err(object_store_identity_changed(&expected_path));
                }
            }
            None => {
                if pinned_directory.is_some() || bound_directory.is_some() {
                    return Err(object_store_identity_changed(&expected_path));
                }
                match bound_store.handle.symlink_metadata(&expected.name) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    _ => return Err(object_store_identity_changed(&expected_path)),
                }
            }
        }
    }
    Ok(())
}

fn ensure_bound_object_subdir_identities_with_work_and_cancellation<C>(
    bound_store: &BoundObjectStore,
    pinned: &PinnedObjectStore,
    ledger: &mut codefire_core::DecisionWorkLedger,
    should_cancel: &mut C,
) -> Result<(), StoreError>
where
    C: FnMut() -> bool,
{
    let (current_names, current_retained_bytes) = collect_object_store_subdir_names_accounted(
        &bound_store.handle,
        &bound_store.subject.canonical_root,
        ledger,
        should_cancel,
        codefire_core::DecisionWorkPhase::SnapshotRevalidation,
    )?;
    let checked = (|| -> Result<(), StoreError> {
        if current_names.len() != bound_store.subject.subdirs.len()
            || current_names
                .keys()
                .zip(&bound_store.subject.subdirs)
                .any(|(current, expected)| current != &expected.name)
        {
            return Err(object_store_identity_changed(
                &bound_store.subject.canonical_root,
            ));
        }

        for expected in &bound_store.subject.subdirs {
            poll_object_store_scan(should_cancel)?;
            let expected_path = bound_store.subject.canonical_root.join(&expected.name);
            let pinned_directory = pinned.subdirs.get(&expected.name).ok_or_else(|| {
                object_store_identity_changed(&bound_store.subject.canonical_root)
            })?;
            let bound_directory = bound_store.subdirs.get(&expected.name).ok_or_else(|| {
                object_store_identity_changed(&bound_store.subject.canonical_root)
            })?;
            match expected.identity {
                Some(identity) => {
                    let pinned_identity = pinned_directory
                        .as_ref()
                        .ok_or_else(|| object_store_identity_changed(&expected_path))
                        .and_then(|directory| {
                            stable_capability_object_store_identity(&directory.dir_metadata()?)
                        })?;
                    poll_object_store_scan(should_cancel)?;
                    let bound_identity = bound_directory
                        .as_ref()
                        .ok_or_else(|| object_store_identity_changed(&expected_path))
                        .and_then(|directory| {
                            stable_capability_object_store_identity(&directory.dir_metadata()?)
                        })?;
                    poll_object_store_scan(should_cancel)?;
                    let current_identity = bound_store
                        .handle
                        .open_dir_nofollow(&expected.name)
                        .map_err(|_| object_store_identity_changed(&expected_path))
                        .and_then(|directory| {
                            stable_capability_object_store_identity(&directory.dir_metadata()?)
                        })?;
                    if pinned_identity != identity
                        || bound_identity != identity
                        || current_identity != identity
                    {
                        return Err(object_store_identity_changed(&expected_path));
                    }
                }
                None => {
                    if pinned_directory.is_some() || bound_directory.is_some() {
                        return Err(object_store_identity_changed(&expected_path));
                    }
                    match bound_store.handle.symlink_metadata(&expected.name) {
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        _ => return Err(object_store_identity_changed(&expected_path)),
                    }
                }
            }
        }
        Ok(())
    })();
    drop(current_names);
    let release = release_object_store_scan_retained(ledger, current_retained_bytes);
    match (checked, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn object_store_identity_changed(path: &Path) -> StoreError {
    StoreError::InvalidSealedCommit(format!(
        "object store identity changed during sealed validation: {}",
        path.display()
    ))
}

#[cfg(unix)]
fn capability_object_store_identity(
    metadata: &cap_std::fs::Metadata,
    _cache: &mut SealedCommitValidationCache,
) -> Result<ObjectStoreIdentity, StoreError> {
    stable_capability_object_store_identity(metadata)
}

#[cfg(windows)]
fn capability_object_store_identity(
    metadata: &cap_std::fs::Metadata,
    _cache: &mut SealedCommitValidationCache,
) -> Result<ObjectStoreIdentity, StoreError> {
    stable_capability_object_store_identity(metadata)
}

#[cfg(not(any(unix, windows)))]
fn capability_object_store_identity(
    _metadata: &cap_std::fs::Metadata,
    cache: &mut SealedCommitValidationCache,
) -> Result<ObjectStoreIdentity, StoreError> {
    let invocation = cache.next_invocation_identity;
    cache.next_invocation_identity = invocation.checked_add(1).ok_or_else(|| {
        StoreError::InvalidSealedCommit(
            "sealed validation cache invocation identity exhausted".to_string(),
        )
    })?;
    Ok(ObjectStoreIdentity::Invocation(invocation))
}

#[cfg(unix)]
fn stable_capability_object_store_identity(
    metadata: &cap_std::fs::Metadata,
) -> Result<ObjectStoreIdentity, StoreError> {
    use cap_std::fs::MetadataExt;

    Ok(ObjectStoreIdentity::Unix {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(windows)]
fn stable_capability_object_store_identity(
    metadata: &cap_std::fs::Metadata,
) -> Result<ObjectStoreIdentity, StoreError> {
    use cap_std::fs::MetadataExt;

    let volume = metadata.volume_serial_number().ok_or_else(|| {
        StoreError::InvalidSealedCommit("object store volume identity is unavailable".to_string())
    })?;
    let file_index = metadata.file_index().ok_or_else(|| {
        StoreError::InvalidSealedCommit("object store file identity is unavailable".to_string())
    })?;
    Ok(ObjectStoreIdentity::Windows { volume, file_index })
}

#[cfg(not(any(unix, windows)))]
fn stable_capability_object_store_identity(
    _metadata: &cap_std::fs::Metadata,
) -> Result<ObjectStoreIdentity, StoreError> {
    Err(StoreError::InvalidSealedCommit(
        "stable object store identity is unavailable on this platform".to_string(),
    ))
}

const LEGACY_REQUIRED_COMMIT_ROOTS: &[&str] = &[
    "content_manifest",
    "atom_index",
    "trace_graph",
    "fire_delta",
    "verification",
    "policy",
];

const CURRENT_REQUIRED_COMMIT_ROOTS: &[&str] = &[
    "content_manifest",
    "atom_index",
    "trace_graph",
    "fire_delta",
    "resolution_ledger",
    "verification",
    "policy",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticValidationProfile {
    LegacyUnversioned,
    SealedSnapshotV1,
}

fn commit_semantic_validation_profile(
    commit: &Value,
) -> Result<SemanticValidationProfile, StoreError> {
    match commit.get("semantic_validation_profile") {
        None => Ok(SemanticValidationProfile::LegacyUnversioned),
        Some(Value::String(profile)) if profile == CURRENT_SEMANTIC_VALIDATION_PROFILE => {
            Ok(SemanticValidationProfile::SealedSnapshotV1)
        }
        Some(Value::String(_)) => Err(StoreError::InvalidSealedCommit(format!(
            "unsupported commit.semantic_validation_profile; supported value is {CURRENT_SEMANTIC_VALIDATION_PROFILE}"
        ))),
        Some(_) => Err(StoreError::InvalidSealedCommit(
            "commit.semantic_validation_profile must be a string".to_string(),
        )),
    }
}

fn commit_root_schema_version(commit: &Value) -> Result<u64, StoreError> {
    let version = match commit.get("root_schema_version") {
        Some(value) => value.as_u64().ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "commit.root_schema_version must be an unsigned integer".to_string(),
            )
        })?,
        None => 1,
    };
    if !matches!(version, 1 | 2) {
        return Err(StoreError::InvalidSealedCommit(format!(
            "unsupported commit.root_schema_version {version}; supported versions are 1 and 2"
        )));
    }
    Ok(version)
}

fn required_commit_roots(root_schema_version: u64) -> &'static [&'static str] {
    match root_schema_version {
        1 => LEGACY_REQUIRED_COMMIT_ROOTS,
        2 => CURRENT_REQUIRED_COMMIT_ROOTS,
        _ => unreachable!("root schema version is validated before root selection"),
    }
}

fn expected_root_type(root: &str) -> Option<&'static str> {
    match root {
        "content_manifest" => Some("content_manifest"),
        "atom_index" => Some("atom_index"),
        "trace_graph" => Some("trace_graph"),
        "fire_delta" => Some("fire_ledger"),
        "resolution_ledger" => Some("resolution_ledger"),
        "verification" => Some("verification"),
        "policy" => Some("policy"),
        _ => None,
    }
}

fn validate_root_references(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    root: &str,
    payload: &Value,
) -> Result<(), StoreError> {
    match root {
        "content_manifest" => validate_manifest_references(bound_store, cache, payload),
        "resolution_ledger" => validate_resolution_ledger_references(bound_store, cache, payload),
        "verification" => validate_verification_references(bound_store, cache, payload),
        _ => Ok(()),
    }
}

fn validate_root_payload(
    root: &str,
    payload: &Value,
    root_schema_version: u64,
) -> Result<(), StoreError> {
    require_supported_version(payload, root)?;
    match root {
        "content_manifest" => validate_content_manifest_shape(payload),
        "atom_index" => validate_atom_index_shape(payload),
        "trace_graph" => validate_trace_graph_shape(payload),
        "fire_delta" => validate_fire_ledger_shape(payload),
        "resolution_ledger" => validate_resolution_ledger_shape(payload),
        "verification" => validate_verification_shape(payload, root_schema_version == 2),
        "policy" => validate_policy_shape(payload),
        _ => Ok(()),
    }
}

fn validate_content_manifest_shape(manifest: &Value) -> Result<(), StoreError> {
    let entries = required_array(manifest, "entries", "content_manifest")?;
    for (index, entry) in entries.iter().enumerate() {
        let context = format!("content_manifest.entries[{index}]");
        required_string(entry, "path", &context)?;
        required_string(entry, "kind", &context)?;
        if entry.get("kind").and_then(Value::as_str) == Some("file") {
            required_string(entry, "mode", &context)?;
            required_string(entry, "blob", &context)?;
        }
    }
    Ok(())
}

fn validate_atom_index_shape(atom_index: &Value) -> Result<(), StoreError> {
    let atoms = required_array(atom_index, "atoms", "atom_index")?;
    required_string_array(atom_index, "duplicate_atom_ids", "atom_index")?;
    for (index, atom) in atoms.iter().enumerate() {
        let context = format!("atom_index.atoms[{index}]");
        required_string(atom, "atom_id", &context)?;
        required_string(atom, "kind", &context)?;
        required_string(atom, "artifact_path", &context)?;
        required_string(atom, "content_hash", &context)?;
        let selector = atom
            .get("selector")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!("{context}.selector must be an object"))
            })?;
        required_string(
            &Value::Object(selector.clone()),
            "type",
            &format!("{context}.selector"),
        )?;
        required_string(
            &Value::Object(selector.clone()),
            "value",
            &format!("{context}.selector"),
        )?;
    }
    Ok(())
}

fn validate_trace_graph_shape(trace_graph: &Value) -> Result<(), StoreError> {
    let links = required_array(trace_graph, "links", "trace_graph")?;
    for (index, link) in links.iter().enumerate() {
        let context = format!("trace_graph.links[{index}]");
        required_string(link, "from", &context)?;
        required_string(link, "to", &context)?;
        required_string(link, "type", &context)?;
        required_string(link, "link_id", &context)?;
        required_string(link, "link_hash", &context)?;
    }
    Ok(())
}

fn validate_trace_graph_atom_references(
    atom_index: &Value,
    trace_graph: &Value,
) -> Result<(), StoreError> {
    let atoms = required_array(atom_index, "atoms", "atom_index")?;
    let mut atom_occurrences = HashMap::<&str, usize>::with_capacity(atoms.len());
    for atom in atoms {
        let atom_id = atom.get("atom_id").and_then(Value::as_str).ok_or_else(|| {
            StoreError::InvalidSealedCommit("atom_index atom_id must be a string".to_string())
        })?;
        *atom_occurrences.entry(atom_id).or_default() += 1;
    }

    for (link_index, link) in required_array(trace_graph, "links", "trace_graph")?
        .iter()
        .enumerate()
    {
        for endpoint in ["from", "to"] {
            let atom_id = link.get(endpoint).and_then(Value::as_str).ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "trace_graph.links[{link_index}].{endpoint} must be a string"
                ))
            })?;
            let occurrences = atom_occurrences.get(atom_id).copied().unwrap_or_default();
            if occurrences != 1 {
                let reason = if occurrences == 0 {
                    "is missing from atom_index".to_string()
                } else {
                    format!("is ambiguous across {occurrences} atoms")
                };
                let (atom_id_sample, omitted_bytes) =
                    bounded_sealed_reference_diagnostic_sample(atom_id);
                let omission = if omitted_bytes > 0 {
                    format!("... ({omitted_bytes} byte(s) omitted)")
                } else {
                    String::new()
                };
                return Err(StoreError::InvalidSealedCommit(format!(
                    "trace_graph.links[{link_index}].{endpoint} atom {atom_id_sample}{omission} {reason}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_current_sealed_snapshot_semantics(
    atom_index: &Value,
    trace_graph: &Value,
    fire_ledger: &Value,
    resolution_ledger: &Value,
    verification: &Value,
    policy_root: &Value,
) -> Result<(), StoreError> {
    let atom_index: codefire_core::AtomIndex = decode_current_root(atom_index, "atom_index")?;
    let trace_graph: codefire_core::TraceGraph = decode_current_root(trace_graph, "trace_graph")?;
    let fires: Vec<codefire_core::Fire> = decode_current_root(
        fire_ledger.get("fires").ok_or_else(|| {
            StoreError::InvalidSealedCommit("fire_delta.fires must be a list".to_string())
        })?,
        "fire_delta.fires",
    )?;
    let resolutions: Vec<codefire_core::Resolution> = decode_current_root(
        resolution_ledger.get("resolutions").ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "resolution_ledger.resolutions must be a list".to_string(),
            )
        })?,
        "resolution_ledger.resolutions",
    )?;
    let verification: codefire_core::Verification =
        decode_current_root(verification, "verification")?;
    let policy_value = policy_root.get("policy").ok_or_else(|| {
        StoreError::InvalidSealedCommit("policy.policy must be an object".to_string())
    })?;
    let policy = if policy_value.as_object().is_some_and(Map::is_empty) {
        codefire_core::default_verification_policy()
    } else {
        decode_current_root(policy_value, "policy.policy")?
    };
    let effective_policy = policy_root
        .get("effective_policy")
        .map(|value| decode_current_root(value, "policy.effective_policy"))
        .transpose()?;

    for (index, fire) in fires.iter().enumerate() {
        if fire.type_tag != "fire" || fire.version != 1 {
            return Err(StoreError::InvalidSealedCommit(format!(
                "fire_delta.fires[{index}] must be type fire version 1"
            )));
        }
    }
    for (index, resolution) in resolutions.iter().enumerate() {
        if resolution.type_tag != "resolution" || resolution.version != 1 {
            return Err(StoreError::InvalidSealedCommit(format!(
                "resolution_ledger.resolutions[{index}] must be type resolution version 1"
            )));
        }
    }

    codefire_core::validate_sealed_snapshot_semantics_with_contract(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
        effective_policy.as_ref(),
    )
    .map_err(|error| {
        StoreError::InvalidSealedCommit(format!(
            "sealed snapshot semantic integrity failed: {error}"
        ))
    })
}

fn decode_current_root<'de, T>(payload: &'de Value, context: &str) -> Result<T, StoreError>
where
    T: Deserialize<'de>,
{
    T::deserialize(payload).map_err(|error| {
        StoreError::InvalidSealedCommit(format!(
            "{context} does not match the supported domain schema: {error}"
        ))
    })
}

fn bounded_sealed_reference_diagnostic_sample(value: &str) -> (&str, usize) {
    if value.len() <= SEALED_REFERENCE_DIAGNOSTIC_SAMPLE_BYTE_LIMIT {
        return (value, 0);
    }
    let mut end = SEALED_REFERENCE_DIAGNOSTIC_SAMPLE_BYTE_LIMIT;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (&value[..end], value.len() - end)
}

fn validate_fire_ledger_shape(ledger: &Value) -> Result<(), StoreError> {
    required_array(ledger, "fires", "fire_delta")?;
    Ok(())
}

fn validate_resolution_ledger_shape(ledger: &Value) -> Result<(), StoreError> {
    required_array(ledger, "resolutions", "resolution_ledger")?;
    Ok(())
}

fn validate_verification_shape(verification: &Value, current: bool) -> Result<(), StoreError> {
    match verification.get("result").and_then(Value::as_str) {
        Some("passed" | "failed") => {}
        Some(_) => {
            return Err(StoreError::InvalidSealedCommit(
                "verification.result must be passed or failed".to_string(),
            ));
        }
        None if current => {
            return Err(StoreError::InvalidSealedCommit(
                "verification.result is required by root schema version 2".to_string(),
            ));
        }
        None => {}
    }
    if current {
        required_u64(verification, "open_required_fires", "verification")?;
        for field in [
            "failed_checks",
            "missing_required_links",
            "stale_resolutions",
            "duplicate_atom_ids",
        ] {
            required_array(verification, field, "verification")?;
        }
        // `Verification` intentionally omits this field when it is empty for
        // compatibility with records written before evidence references were
        // added. The current certificate still carries and matches the zero
        // count, so omission cannot erase a blocker.
        optional_array(verification, "missing_evidence_refs")?;
        // Merge conflicts are active-work facts and must be empty before a
        // verification can become a sealed root. Historical records predate
        // the field, so omission is the only supported empty encoding.
        optional_array(verification, "unresolved_merge_conflicts")?;
        required_string(verification, "verified_at", "verification")?;
    } else {
        optional_u64(verification, "open_required_fires")?;
        optional_array(verification, "failed_checks")?;
        optional_array(verification, "missing_required_links")?;
        optional_array(verification, "stale_resolutions")?;
        optional_array(verification, "missing_evidence_refs")?;
        optional_array(verification, "unresolved_merge_conflicts")?;
        optional_array(verification, "duplicate_atom_ids")?;
    }
    Ok(())
}

fn validate_policy_shape(policy: &Value) -> Result<(), StoreError> {
    policy
        .get("policy")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit("policy.policy must be an object".to_string())
        })?;
    if let Some(contract) = policy.get("effective_policy") {
        if !contract.is_object() {
            return Err(StoreError::InvalidSealedCommit(
                "policy.effective_policy must be an object".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_manifest_references(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    manifest: &Value,
) -> Result<(), StoreError> {
    let entries = optional_array(manifest, "entries")?;
    for (index, entry) in entries.iter().enumerate() {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let blob_id =
            required_reference(entry, "blob", &format!("content_manifest.entries[{index}]"))?;
        read_typed_reference(
            bound_store,
            cache,
            blob_id,
            "blob",
            &format!("content_manifest.entries[{index}].blob"),
        )?;
    }
    Ok(())
}

fn validate_resolution_ledger_references(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    ledger: &Value,
) -> Result<(), StoreError> {
    let resolutions = optional_array(ledger, "resolutions")?;
    for (resolution_index, resolution) in resolutions.iter().enumerate() {
        let evidence_refs = optional_array(resolution, "evidence_refs")?;
        for (evidence_index, evidence_ref) in evidence_refs.iter().enumerate() {
            let evidence_id = evidence_ref.as_str().ok_or_else(|| {
                StoreError::InvalidSealedCommit(format!(
                    "resolution_ledger.resolutions[{resolution_index}].evidence_refs[{evidence_index}] must be an evidence object id"
                ))
            })?;
            let evidence = read_typed_reference(
                bound_store,
                cache,
                evidence_id,
                "evidence",
                &format!(
                    "resolution_ledger.resolutions[{resolution_index}].evidence_refs[{evidence_index}]"
                ),
            )?;
            validate_evidence_references(bound_store, cache, evidence_id, &evidence)?;
        }
    }
    Ok(())
}

fn validate_verification_references(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    verification: &Value,
) -> Result<(), StoreError> {
    let missing_refs = optional_array(verification, "missing_evidence_refs")?;
    for (index, missing_ref) in missing_refs.iter().enumerate() {
        let evidence_id = required_reference(
            missing_ref,
            "evidence_id",
            &format!("verification.missing_evidence_refs[{index}]"),
        )?;
        read_typed_reference(
            bound_store,
            cache,
            evidence_id,
            "evidence",
            &format!("verification.missing_evidence_refs[{index}].evidence_id"),
        )?;
    }
    Ok(())
}

fn validate_evidence_references(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    evidence_id: &str,
    evidence: &Value,
) -> Result<(), StoreError> {
    let Some(artifact_ref) = evidence.get("artifact_ref") else {
        return Ok(());
    };
    if artifact_ref.is_null() {
        return Ok(());
    }
    let artifact_id = artifact_ref.as_str().ok_or_else(|| {
        StoreError::InvalidSealedCommit(format!(
            "evidence {evidence_id} artifact_ref must be an artifact_ref object id"
        ))
    })?;
    read_typed_reference(
        bound_store,
        cache,
        artifact_id,
        "artifact_ref",
        &format!("evidence {evidence_id}.artifact_ref"),
    )?;
    Ok(())
}

fn optional_array<'a>(payload: &'a Value, field: &str) -> Result<&'a [Value], StoreError> {
    match payload.get(field) {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(StoreError::InvalidSealedCommit(format!(
            "{field} must be a list"
        ))),
        None => Ok(&[]),
    }
}

fn required_array<'a>(
    payload: &'a Value,
    field: &str,
    context: &str,
) -> Result<&'a [Value], StoreError> {
    payload
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| StoreError::InvalidSealedCommit(format!("{context}.{field} must be a list")))
}

fn required_string<'a>(
    payload: &'a Value,
    field: &str,
    context: &str,
) -> Result<&'a str, StoreError> {
    payload.get(field).and_then(Value::as_str).ok_or_else(|| {
        StoreError::InvalidSealedCommit(format!("{context}.{field} must be a string"))
    })
}

fn required_string_array(payload: &Value, field: &str, context: &str) -> Result<(), StoreError> {
    let values = required_array(payload, field, context)?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_string() {
            return Err(StoreError::InvalidSealedCommit(format!(
                "{context}.{field}[{index}] must be a string"
            )));
        }
    }
    Ok(())
}

fn required_u64(payload: &Value, field: &str, context: &str) -> Result<u64, StoreError> {
    payload.get(field).and_then(Value::as_u64).ok_or_else(|| {
        StoreError::InvalidSealedCommit(format!("{context}.{field} must be an unsigned integer"))
    })
}

fn require_supported_version(payload: &Value, context: &str) -> Result<(), StoreError> {
    let version = required_u64(payload, "version", context)?;
    if version != 1 {
        return Err(StoreError::InvalidSealedCommit(format!(
            "unsupported {context}.version {version}; supported version is 1"
        )));
    }
    Ok(())
}

fn optional_u64(payload: &Value, field: &str) -> Result<(), StoreError> {
    match payload.get(field) {
        Some(value) if value.as_u64().is_some() => Ok(()),
        Some(_) => Err(StoreError::InvalidSealedCommit(format!(
            "{field} must be an unsigned integer"
        ))),
        None => Ok(()),
    }
}

fn validate_certificate_matches_verification(
    certificate: &Map<String, Value>,
    verification: &Value,
    root_schema_version: u64,
) -> Result<(), StoreError> {
    let verification_result = verification.get("result").and_then(Value::as_str);
    if root_schema_version == 2 && verification_result.is_none() {
        return Err(StoreError::InvalidSealedCommit(
            "current verification result is missing".to_string(),
        ));
    }
    if let Some(verification_result) = verification_result {
        let certificate_result = certificate
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("(missing)");
        if !certificate_result_matches_verification(certificate_result, verification_result) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "certificate result {certificate_result} does not match verification result {verification_result}"
            )));
        }
    }
    if let Some(subject) = verification.get("subject") {
        validate_assessment_subject(subject)?;
    }
    match (verification.get("subject"), certificate.get("subject")) {
        (Some(verification_subject), Some(certificate_subject))
            if verification_subject == certificate_subject => {}
        (Some(_), Some(_)) => {
            return Err(StoreError::InvalidSealedCommit(
                "certificate subject does not match verification subject".to_string(),
            ));
        }
        (Some(_), None) => {
            return Err(StoreError::InvalidSealedCommit(
                "certificate subject is required when verification has a subject".to_string(),
            ));
        }
        (None, Some(_)) => {
            return Err(StoreError::InvalidSealedCommit(
                "certificate subject is present without a verification subject".to_string(),
            ));
        }
        (None, None) => {}
    }
    for (field, verification_field) in [
        ("effective_policy_digest", "effective_policy"),
        ("change_coverage_digest", "change_coverage"),
    ] {
        let verification_digest = verification
            .get(verification_field)
            .and_then(|contract| contract.get("digest"))
            .and_then(Value::as_str);
        let certificate_digest = certificate.get(field).and_then(Value::as_str);
        match (verification_digest, certificate_digest) {
            (Some(expected), Some(recorded)) if expected == recorded => {}
            (Some(_), Some(_)) => {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "certificate {field} does not match verification {verification_field} digest"
                )));
            }
            (Some(_), None) => {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "certificate {field} is required when verification has {verification_field}"
                )));
            }
            (None, Some(_)) => {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "certificate {field} is present without verification {verification_field}"
                )));
            }
            (None, None) => {}
        }
    }
    for (field, verification_count) in [
        (
            "open_required_fires",
            verification
                .get("open_required_fires")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        ),
        (
            "failed_checks",
            array_len_u64(verification, "failed_checks"),
        ),
        (
            "missing_required_links",
            array_len_u64(verification, "missing_required_links"),
        ),
        (
            "stale_resolutions",
            array_len_u64(verification, "stale_resolutions"),
        ),
        (
            "missing_evidence_refs",
            array_len_u64(verification, "missing_evidence_refs"),
        ),
        (
            "duplicate_atom_ids",
            array_len_u64(verification, "duplicate_atom_ids"),
        ),
    ] {
        let certificate_count = certificate.get(field).and_then(Value::as_u64);
        let optional_empty_compatibility_field = field == "missing_evidence_refs";
        if root_schema_version == 2
            && certificate_count.is_none()
            && !(optional_empty_compatibility_field && verification_count == 0)
        {
            return Err(StoreError::InvalidSealedCommit(format!(
                "certificate {field} count is required by root schema version 2"
            )));
        }
        if certificate_count.is_some_and(|count| count != verification_count) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "certificate {field} count {} does not match verification count {verification_count}",
                certificate_count.expect("checked as present")
            )));
        }
    }
    Ok(())
}

fn validate_assessment_subject(subject: &Value) -> Result<(), StoreError> {
    let object = subject.as_object().ok_or_else(|| {
        StoreError::InvalidSealedCommit("verification subject must be an object".to_string())
    })?;
    const SUBJECT_FIELDS: &[&str] = &[
        "type",
        "version",
        "kind",
        "trust_kind",
        "subject_digest",
        "content_digest",
        "worktree_digest",
        "immutable",
        "repository_id",
        "branch",
        "open_instance_id",
        "base_commit",
        "source_commit",
    ];
    if let Some(field) = object
        .keys()
        .find(|field| !SUBJECT_FIELDS.contains(&field.as_str()))
    {
        return Err(StoreError::InvalidSealedCommit(format!(
            "verification subject contains unsupported field {field}"
        )));
    }
    if object.get("type").and_then(Value::as_str) != Some("codefire_assessment_subject")
        || object.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err(StoreError::InvalidSealedCommit(
            "verification subject type/version is unsupported".to_string(),
        ));
    }
    let content_digest = object
        .get("content_digest")
        .and_then(Value::as_str)
        .filter(|digest| valid_sha256_label(digest))
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "verification subject content digest is invalid".to_string(),
            )
        })?;
    let kind = required_subject_string(object, "kind")?;
    let trust_kind = required_subject_string(object, "trust_kind")?;
    let immutable = object
        .get("immutable")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(
                "verification subject immutable must be a boolean".to_string(),
            )
        })?;
    let repository_id = optional_subject_string(object, "repository_id")?;
    let branch = optional_subject_string(object, "branch")?;
    let open_instance_id = optional_subject_string(object, "open_instance_id")?;
    let base_commit = optional_subject_string(object, "base_commit")?;
    let source_commit = optional_subject_string(object, "source_commit")?;
    let worktree_digest = match optional_subject_string(object, "worktree_digest")? {
        Some(digest) if valid_sha256_label(digest) => Some(digest),
        Some(_) => {
            return Err(StoreError::InvalidSealedCommit(
                "verification subject worktree digest is invalid".to_string(),
            ));
        }
        None => None,
    };
    match kind {
        "live_open"
            if trust_kind == "local_unisolated"
                && !immutable
                && repository_id.is_some()
                && branch.is_some()
                && open_instance_id.is_some()
                && base_commit.is_some()
                && source_commit.is_none()
                && worktree_digest.is_some() => {}
        "sealed_commit_pair"
            if trust_kind == "sealed_validated"
                && immutable
                && branch.is_none()
                && open_instance_id.is_none()
                && base_commit.is_some()
                && source_commit.is_some()
                && worktree_digest.is_none() => {}
        "live_open" | "sealed_commit_pair" => {
            return Err(StoreError::InvalidSealedCommit(format!(
                "verification subject fields do not match {kind} trust semantics"
            )));
        }
        _ => {
            return Err(StoreError::InvalidSealedCommit(format!(
                "verification subject kind is unsupported: {kind}"
            )));
        }
    }
    let actual = object
        .get("subject_digest")
        .and_then(Value::as_str)
        .filter(|digest| valid_sha256_label(digest))
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit("verification subject digest is invalid".to_string())
        })?;
    let identity = serde_json::json!({
        "type": object.get("type").cloned().unwrap_or(Value::Null),
        "version": object.get("version").cloned().unwrap_or(Value::Null),
        "kind": object.get("kind").cloned().unwrap_or(Value::Null),
        "trust_kind": object.get("trust_kind").cloned().unwrap_or(Value::Null),
        "content_digest": content_digest,
        "worktree_digest": worktree_digest,
        "immutable": object.get("immutable").cloned().unwrap_or(Value::Null),
        "repository_id": object.get("repository_id").cloned().unwrap_or(Value::Null),
        "branch": object.get("branch").cloned().unwrap_or(Value::Null),
        "open_instance_id": object.get("open_instance_id").cloned().unwrap_or(Value::Null),
        "base_commit": object.get("base_commit").cloned().unwrap_or(Value::Null),
        "source_commit": object.get("source_commit").cloned().unwrap_or(Value::Null),
    });
    let expected = format!(
        "sha256:{}",
        codefire_util::sha256_hex(&canonical_json(&identity)?)
    );
    if actual != expected {
        return Err(StoreError::InvalidSealedCommit(
            "verification subject digest does not match its identity fields".to_string(),
        ));
    }
    Ok(())
}

fn required_subject_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, StoreError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit(format!(
                "verification subject {field} must be a non-empty string"
            ))
        })
}

fn optional_subject_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, StoreError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value)),
        Some(_) => Err(StoreError::InvalidSealedCommit(format!(
            "verification subject {field} must be a non-empty string or null"
        ))),
    }
}

fn valid_sha256_label(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn certificate_result_matches_verification(certificate: &str, verification: &str) -> bool {
    matches!(
        (certificate, verification),
        ("passed", "passed") | ("consistent", "passed")
    )
}

fn array_len_u64(payload: &Value, field: &str) -> u64 {
    payload
        .get(field)
        .and_then(Value::as_array)
        .map_or(0, |values| values.len() as u64)
}

fn required_reference<'a>(
    payload: &'a Value,
    field: &str,
    context: &str,
) -> Result<&'a str, StoreError> {
    payload.get(field).and_then(Value::as_str).ok_or_else(|| {
        StoreError::InvalidSealedCommit(format!("{context}.{field} must be an object id"))
    })
}

fn read_typed_reference(
    bound_store: &BoundObjectStore,
    cache: &mut SealedCommitValidationCache,
    object_id: &str,
    expected_type: &str,
    context: &str,
) -> Result<RetainedObjectPayload, StoreError> {
    if !object_id.starts_with("CF-") {
        return Err(StoreError::InvalidSealedCommit(format!(
            "{context} must be an object id"
        )));
    }
    let payload = read_bound_object_memoized(bound_store, object_id, cache).map_err(|error| {
        StoreError::InvalidSealedCommit(format!("{context} invalid reference {object_id}: {error}"))
    })?;
    let actual_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("(missing)");
    if actual_type != expected_type {
        return Err(StoreError::InvalidSealedCommit(format!(
            "{context} has type {actual_type}, expected {expected_type}"
        )));
    }
    Ok(payload)
}

pub fn commit_payload(
    parents: Vec<String>,
    roots: Map<String, Value>,
    certificate: Map<String, Value>,
) -> Value {
    let mut payload = Map::new();
    payload.insert("type".to_string(), Value::String("commit".to_string()));
    payload.insert("version".to_string(), Value::Number(1.into()));
    payload.insert("root_schema_version".to_string(), Value::Number(2.into()));
    payload.insert(
        "semantic_validation_profile".to_string(),
        Value::String(CURRENT_SEMANTIC_VALIDATION_PROFILE.to_string()),
    );
    payload.insert(
        "parents".to_string(),
        Value::Array(parents.into_iter().map(Value::String).collect()),
    );
    payload.insert(
        "message".to_string(),
        Value::String("test commit".to_string()),
    );
    payload.insert("roots".to_string(), Value::Object(roots));
    payload.insert("certificate".to_string(), Value::Object(certificate));
    Value::Object(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    const ARTIFICIAL_COLLISION_PREFIX: &str = "0123456789abcdef01234567";

    fn artificial_collision_digest(
        _type_tag: &str,
        payload: &Value,
    ) -> Result<String, serde_json::Error> {
        let suffix = match payload.get("collision_variant").and_then(Value::as_str) {
            Some("existing") => "0000000000000000000000000000000000000000",
            Some("requested") => "1111111111111111111111111111111111111111",
            other => panic!("unexpected artificial collision variant: {other:?}"),
        };
        Ok(format!("{ARTIFICIAL_COLLISION_PREFIX}{suffix}"))
    }

    fn artificial_collision_payload(variant: &str) -> Value {
        json!({
            "type": "policy",
            "version": 1,
            "policy": {"collision_variant": variant},
            "collision_variant": variant
        })
    }

    fn artificial_full_digest_collision(
        _type_tag: &str,
        _payload: &Value,
    ) -> Result<String, serde_json::Error> {
        Ok(format!("{ARTIFICIAL_COLLISION_PREFIX}{}", "a".repeat(40)))
    }

    fn write_record_fixture(path: &Path, record: &ObjectRecord) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut rendered = serde_json::to_string_pretty(record).unwrap();
        rendered.push('\n');
        fs::write(path, rendered).unwrap();
    }

    fn cache_contains_commit(cache: &SealedCommitValidationCache, commit_id: &str) -> bool {
        cache
            .validated_subjects
            .iter()
            .any(|(_, cached_commit_id)| cached_commit_id == commit_id)
    }

    fn install_record_extension(path: &Path, marker: &str) -> Vec<u8> {
        let mut document: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        document
            .as_object_mut()
            .unwrap()
            .insert("remote".to_string(), json!({"generation": marker}));
        let mut bytes = serde_json::to_vec_pretty(&document).unwrap();
        bytes.push(b'\n');
        fs::write(path, &bytes).unwrap();
        bytes
    }

    fn rewrite_marker_preserving_file_identity_and_times(
        path: &Path,
        from: &[u8],
        to: &[u8],
    ) -> Vec<u8> {
        assert_eq!(from.len(), to.len());
        let before = fs::metadata(path).unwrap();
        let modified = before.modified().unwrap();
        let accessed = before.accessed().unwrap();
        let mut bytes = fs::read(path).unwrap();
        let offset = bytes
            .windows(from.len())
            .position(|window| window == from)
            .expect("marker must be present in record document");
        bytes[offset..offset + from.len()].copy_from_slice(to);
        fs::write(path, &bytes).unwrap();
        OpenOptions::new()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_accessed(accessed)
                    .set_modified(modified),
            )
            .unwrap();
        let after = fs::metadata(path).unwrap();
        assert_eq!(after.len(), before.len());
        assert_eq!(after.modified().unwrap(), modified);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(after.dev(), before.dev());
            assert_eq!(after.ino(), before.ino());
        }
        bytes
    }

    fn assert_collision_error(
        error: StoreError,
        requested: &ObjectRecord,
        existing: &ObjectRecord,
        expected_payload_match: bool,
    ) {
        match error {
            StoreError::ObjectCollision {
                object_id,
                requested_type,
                existing_type,
                requested_hash,
                existing_hash,
                canonical_payload_matches,
            } => {
                assert_eq!(object_id, requested.object_id);
                assert_eq!(requested_type, requested.type_tag);
                assert_eq!(existing_type, existing.type_tag);
                assert_eq!(requested_hash, requested.hash);
                assert_eq!(existing_hash, existing.hash);
                assert_eq!(canonical_payload_matches, expected_payload_match);
            }
            other => panic!("expected object collision, got {other}"),
        }
    }

    fn prepared_policy_document(marker: &str) -> ObjectRecordDocument {
        ObjectRecordDocument {
            record: object_record(
                "policy",
                json!({
                    "type": "policy",
                    "version": 1,
                    "policy": {"prepared": marker}
                }),
            )
            .unwrap(),
            extensions: Map::new(),
        }
    }

    fn object_transfer_ledger(command: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            command,
        )
        .unwrap()
    }

    fn detached_prepublication_proof(
        objects: &Path,
        command: &str,
    ) -> (
        String,
        PrepublicationSealedValidationProof,
        codefire_core::DecisionWorkLedger,
    ) {
        let commit_id = write_valid_commit(objects, vec![]);
        let ledger = object_transfer_ledger(command);
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();
        let (proof, ledger) = bound.into_prepublication_validation_proof().unwrap();
        (commit_id, proof, ledger)
    }

    fn write_prepared_recovery_stage(objects: &Path, document: &ObjectRecordDocument) -> PathBuf {
        let subdir = object_subdir(&document.record.type_tag).unwrap();
        let destination = PathBuf::from(format!("{}.json", document.record.object_id));
        let name = unique_prepared_object_stage_name(&destination).unwrap();
        let path = objects.join(subdir).join(name);
        let bytes =
            serialize_object_record_document_with_limit(document, &path, MAX_OBJECT_RECORD_BYTES)
                .unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }

    fn create_prepared_object_store(objects: &Path, documents: &[ObjectRecordDocument]) {
        fs::create_dir_all(objects).unwrap();
        for document in documents {
            fs::create_dir_all(objects.join(object_subdir(&document.record.type_tag).unwrap()))
                .unwrap();
        }
    }

    fn assert_prepared_batch_current_zero(ledger: &codefire_core::DecisionWorkLedger) {
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0,
                "prepared object batch leaked {dimension:?}"
            );
        }
    }

    struct PreparedBatchTestOwner<'a> {
        batch: Option<PreparedObjectRecordBatch>,
        ledger: &'a mut codefire_core::DecisionWorkLedger,
    }

    impl Drop for PreparedBatchTestOwner<'_> {
        fn drop(&mut self) {
            if let Some(mut batch) = self.batch.take() {
                let cancelled = batch.cancel(self.ledger);
                debug_assert!(cancelled.is_ok());
            }
        }
    }

    #[test]
    fn prepared_object_batch_publishes_missing_record_after_one_external_seal() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("missing");
        let object_id = document.record.object_id.clone();
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut ledger = object_transfer_ledger("prepared-object-missing");

        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        assert_eq!(
            batch.disposition(&object_id),
            Some(PreparedObjectRecordDisposition::Missing)
        );
        assert!(object_record_path(&objects, &object_id).is_none());

        batch.revalidate_preseal(&mut ledger, || false).unwrap();
        ledger.seal_prepublication().unwrap();
        batch.commit_after_seal(&mut ledger).unwrap();

        assert_eq!(
            read_object_record_document_path(&object_record_path(&objects, &object_id).unwrap())
                .unwrap(),
            document
        );
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_skips_exact_record_without_rewriting_it() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("exact");
        let object_id = store_object_record_document(&objects, document.clone()).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let before = fs::read(&path).unwrap();
        let mut ledger = object_transfer_ledger("prepared-object-exact");

        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        assert_eq!(
            batch.disposition(&object_id),
            Some(PreparedObjectRecordDisposition::Exact)
        );
        batch.revalidate_preseal(&mut ledger, || false).unwrap();
        ledger.seal_prepublication().unwrap();
        batch.commit_after_seal(&mut ledger).unwrap();

        assert_eq!(fs::read(path).unwrap(), before);
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_rejects_divergent_existing_extension() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut existing = prepared_policy_document("divergent");
        existing
            .extensions
            .insert("remote".to_string(), json!({"generation": 1}));
        let object_id = store_object_record_document(&objects, existing).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let before = fs::read(&path).unwrap();
        let mut requested = prepared_policy_document("divergent");
        requested
            .extensions
            .insert("remote".to_string(), json!({"generation": 2}));
        let mut ledger = object_transfer_ledger("prepared-object-divergent");

        let error = match PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![requested],
            &mut ledger,
            || false,
        ) {
            Ok(mut batch) => {
                batch.cancel(&mut ledger).unwrap();
                panic!("divergent existing object unexpectedly prepared")
            }
            Err(error) => error,
        };

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("extension fields diverge")
        ));
        assert_eq!(fs::read(path).unwrap(), before);
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_detects_exact_record_preseal_drift() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("drift");
        let object_id = store_object_record_document(&objects, document.clone()).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let mut ledger = object_transfer_ledger("prepared-object-drift");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        let expected_size = fs::metadata(&path).unwrap().len();
        let mut changed = fs::read(&path).unwrap();
        changed.push(b' ');
        fs::write(&path, changed).unwrap();

        let error = batch.revalidate_preseal(&mut ledger, || false).unwrap_err();

        assert!(matches!(
            error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::InvalidData
        ));
        assert_eq!(fs::metadata(&path).unwrap().len(), expected_size + 1);
        batch.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn prepared_object_batch_closing_fence_detects_subdir_swap_after_final_read() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("closing-subdir-fence");
        store_object_record_document(&objects, document.clone()).unwrap();
        let policies = objects.join("policies");
        let displaced = objects.join("policies-old");
        let mut ledger = object_transfer_ledger("prepared-object-closing-subdir-fence");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .unwrap();

        let error = batch
            .revalidate_preseal_with_closing_hook(
                &mut ledger,
                || false,
                || {
                    fs::rename(&policies, &displaced)?;
                    fs::create_dir(&policies)?;
                    Ok(())
                },
            )
            .unwrap_err();

        assert!(matches!(
            error,
            StoreError::Io(_) | StoreError::InvalidSealedCommit(_)
        ));
        batch.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_postseal_collision_requires_durable_retry() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("postseal-race");
        let object_id = document.record.object_id.clone();
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut ledger = object_transfer_ledger("prepared-object-postseal-race");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        batch.revalidate_preseal(&mut ledger, || false).unwrap();
        store_object_record_document(&objects, document.clone()).unwrap();
        ledger.seal_prepublication().unwrap();

        let error = batch.commit_after_seal(&mut ledger).unwrap_err();

        assert!(matches!(error, StoreError::ObjectPublicationRetryRequired));
        assert_eq!(
            read_object_record_document_path(&object_record_path(&objects, &object_id).unwrap())
                .unwrap(),
            document
        );
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_charges_only_attempted_prefix_before_failure() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut documents = vec![
            prepared_policy_document("prefix-a"),
            prepared_policy_document("prefix-b"),
            prepared_policy_document("prefix-c"),
        ];
        documents
            .sort_unstable_by(|left, right| left.record.object_id.cmp(&right.record.object_id));
        let ids = documents
            .iter()
            .map(|document| document.record.object_id.clone())
            .collect::<Vec<_>>();
        create_prepared_object_store(&objects, &documents);
        let mut ledger = object_transfer_ledger("prepared-object-prefix-failure");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            documents.clone(),
            &mut ledger,
            || false,
        )
        .unwrap();
        batch.revalidate_preseal(&mut ledger, || false).unwrap();
        store_object_record_document(&objects, documents[1].clone()).unwrap();
        ledger.seal_prepublication().unwrap();

        let error = batch.commit_after_seal(&mut ledger).unwrap_err();

        assert!(matches!(error, StoreError::ObjectPublicationRetryRequired));
        assert!(object_record_path(&objects, &ids[0]).is_some());
        assert!(object_record_path(&objects, &ids[1]).is_some());
        assert!(object_record_path(&objects, &ids[2]).is_none());
        assert_eq!(
            ledger.snapshot().phase_counters
                [&codefire_core::DecisionWorkPhase::PublicationPreflight]
                [&codefire_core::DecisionWorkCounter::ObjectRecords],
            2
        );
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_cancel_and_owner_drop_release_all_current_work() {
        let explicit_temp = tempdir().unwrap();
        let explicit_objects = explicit_temp.path().join("objects");
        let explicit_document = prepared_policy_document("cancel");
        let explicit_id = explicit_document.record.object_id.clone();
        create_prepared_object_store(&explicit_objects, std::slice::from_ref(&explicit_document));
        let mut explicit_ledger = object_transfer_ledger("prepared-object-cancel");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &explicit_objects,
            vec![explicit_document],
            &mut explicit_ledger,
            || false,
        )
        .unwrap();
        batch.cancel(&mut explicit_ledger).unwrap();
        assert!(object_record_path(&explicit_objects, &explicit_id).is_none());
        assert_prepared_batch_current_zero(&explicit_ledger);
        explicit_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();

        let drop_temp = tempdir().unwrap();
        let drop_objects = drop_temp.path().join("objects");
        let drop_document = prepared_policy_document("drop");
        let drop_id = drop_document.record.object_id.clone();
        create_prepared_object_store(&drop_objects, std::slice::from_ref(&drop_document));
        let mut drop_ledger = object_transfer_ledger("prepared-object-drop");
        let batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &drop_objects,
            vec![drop_document],
            &mut drop_ledger,
            || false,
        )
        .unwrap();
        {
            let _owner = PreparedBatchTestOwner {
                batch: Some(batch),
                ledger: &mut drop_ledger,
            };
        }
        assert!(object_record_path(&drop_objects, &drop_id).is_none());
        assert_prepared_batch_current_zero(&drop_ledger);
        drop_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_rejects_foreign_ledger_without_mutating_either_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("ledger-identity");
        store_object_record_document(&objects, document.clone()).unwrap();

        for operation in ["revalidate", "commit", "cancel"] {
            let mut owner = object_transfer_ledger(&format!("prepared-owner-{operation}"));
            let mut foreign = object_transfer_ledger(&format!("prepared-foreign-{operation}"));
            let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &objects,
                vec![document.clone()],
                &mut owner,
                || false,
            )
            .unwrap();
            assert!(
                batch.permit.is_none(),
                "fixture must exercise exact-only ownership"
            );
            let owner_before = owner.snapshot().clone();
            let foreign_before = foreign.snapshot().clone();

            let error = match operation {
                "revalidate" => batch.revalidate_preseal(&mut foreign, || false),
                "commit" => batch.commit_after_seal(&mut foreign),
                "cancel" => batch.cancel(&mut foreign),
                _ => unreachable!(),
            }
            .unwrap_err();

            assert!(matches!(
                error,
                StoreError::InvalidSealedCommit(message)
                    if message.contains("different decision-work ledger")
            ));
            assert_eq!(owner.snapshot(), &owner_before);
            assert_eq!(foreign.snapshot(), &foreign_before);
            batch.cancel(&mut owner).unwrap();
            assert_prepared_batch_current_zero(&owner);
            owner
                .finish(codefire_core::DecisionWorkTerminalReason::Complete)
                .unwrap();
            foreign
                .finish(codefire_core::DecisionWorkTerminalReason::Complete)
                .unwrap();
        }
    }

    #[test]
    fn prepared_object_batch_rejects_occurrence_limit_before_scanning_or_sorting_duplicates() {
        const TOO_MANY_RECORDS: usize = 100_001;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("occurrence-limit");
        let documents = std::iter::repeat_n(document, TOO_MANY_RECORDS).collect();
        let mut ledger = object_transfer_ledger("prepared-object-occurrence-limit");
        let mut polls = 0_u64;

        let error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            documents,
            &mut ledger,
            || {
                polls += 1;
                false
            },
        )
        .err()
        .expect("over-limit occurrence count must fail");

        assert_eq!(polls, 0, "input must be rejected before its first scan");
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::ObjectRecords
                        && budget.observed == TOO_MANY_RECORDS as u64
                        && budget.limit == 100_000
                })
        ));
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn prepared_object_batch_never_creates_missing_store_layout() {
        let document = prepared_policy_document("missing-layout");

        let root_temp = tempdir().unwrap();
        let missing_root = root_temp.path().join("absent/objects");
        let mut root_ledger = object_transfer_ledger("prepared-object-missing-root");
        let root_error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &missing_root,
            vec![document.clone()],
            &mut root_ledger,
            || false,
        )
        .err()
        .expect("missing root must fail");
        assert!(matches!(root_error, StoreError::Io(_)));
        assert!(!missing_root.exists());
        assert_prepared_batch_current_zero(&root_ledger);
        root_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();

        let subdir_temp = tempdir().unwrap();
        let missing_subdir_root = subdir_temp.path().join("objects");
        fs::create_dir_all(&missing_subdir_root).unwrap();
        let expected_subdir =
            missing_subdir_root.join(object_subdir(&document.record.type_tag).unwrap());
        let mut subdir_ledger = object_transfer_ledger("prepared-object-missing-subdir");
        let subdir_error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &missing_subdir_root,
            vec![document],
            &mut subdir_ledger,
            || false,
        )
        .err()
        .expect("missing subdirectory must fail");
        assert!(matches!(subdir_error, StoreError::Io(_)));
        assert!(!expected_subdir.exists());
        assert_prepared_batch_current_zero(&subdir_ledger);
        subdir_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_accounts_long_root_component_scratch() {
        let temp = tempdir().unwrap();
        let mut objects = temp.path().to_path_buf();
        for index in 0..24 {
            objects.push(format!("component-{index:02}-{}", "x".repeat(32)));
        }
        objects.push("objects");
        let document = prepared_policy_document("long-root");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let expected_scratch = objects.components().fold(
            codefire_core::LOGICAL_FRAME_BASE_BYTES,
            |total, component| {
                total
                    + codefire_core::LOGICAL_FRAME_BASE_BYTES
                    + component.as_os_str().as_encoded_bytes().len() as u64
            },
        );
        let mut ledger = object_transfer_ledger("prepared-object-long-root");

        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .unwrap();

        assert!(
            ledger.snapshot().peak[&codefire_core::DecisionWorkDimension::TemporaryBytes]
                >= expected_scratch
        );
        batch.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn prepared_object_batch_stage_is_private_and_non_authoritative() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("private-stage");
        let object_id = document.record.object_id.clone();
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut ledger = object_transfer_ledger("prepared-object-private-stage");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .unwrap();
        let entry = &batch.entries[0];
        let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
            panic!("missing object must own a stage")
        };
        let stage_path = objects.join(entry.subdir).join(stage_name);
        assert_eq!(
            fs::metadata(&stage_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            stage_path.extension().and_then(|value| value.to_str()),
            Some("tmp")
        );
        assert!(object_record_path(&objects, &object_id).is_none());

        batch.cancel(&mut ledger).unwrap();
        assert!(!stage_path.exists());
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_does_not_remove_foreign_stage_name_on_create_collision() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("foreign-stage-collision");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let stage_name = PathBuf::from(".foreign-stage.tmp");
        let stage_path = objects.join("policies").join(&stage_name);
        fs::write(&stage_path, b"foreign-owner").unwrap();
        let mut ledger = object_transfer_ledger("prepared-object-foreign-stage-collision");

        let error = PreparedObjectRecordBatch::prepare_with_ledger_poll_and_stage_name(
            &objects,
            vec![document],
            &mut ledger,
            || false,
            |_| Ok(stage_name.clone()),
        )
        .err()
        .expect("create-new collision must fail preparation");

        assert!(matches!(
            error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert_eq!(fs::read(stage_path).unwrap(), b"foreign-owner");
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_keeps_ownership_when_stage_remove_fails_and_retries_cancel() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("retry-stage-cleanup");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut ledger = object_transfer_ledger("prepared-object-retry-stage-cleanup");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .unwrap();
        let stage_path = {
            let entry = &batch.entries[0];
            let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                panic!("fixture must own a missing stage")
            };
            objects.join(entry.subdir).join(stage_name)
        };
        let ledger_before = ledger.snapshot().clone();

        let error = batch
            .cancel_inner_with_remove(&mut ledger, |_| {
                Err(StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected stage cleanup failure",
                )))
            })
            .unwrap_err();

        assert!(matches!(
            error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied
        ));
        assert!(batch.active);
        assert!(stage_path.exists());
        assert_eq!(ledger.snapshot(), &ledger_before);
        assert!(batch.permit.is_some());

        batch.cancel(&mut ledger).unwrap();
        assert!(!stage_path.exists());
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_cleanup_failure_can_abandon_private_stage_and_retry_release() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("abandon-private-stage");
        let object_id = document.record.object_id.clone();
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut ledger = object_transfer_ledger("prepared-object-abandon-private-stage");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        let stage_path = {
            let entry = &batch.entries[0];
            let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                panic!("fixture must own a missing stage")
            };
            objects.join(entry.subdir).join(stage_name)
        };
        let retained_bytes = batch.retained_bytes;
        ledger.seal_prepublication().unwrap();

        let cleanup_error = batch
            .cancel_inner_with_remove(&mut ledger, |_| {
                Err(StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected durable stage cleanup failure",
                )))
            })
            .unwrap_err();
        assert!(matches!(
            cleanup_error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied
        ));
        assert!(batch.active);
        assert!(batch.permit.is_some());
        assert!(stage_path.exists());

        let release_error = batch
            .abandon_for_durable_retry_with_release(&mut ledger, |_, amount| {
                assert_eq!(amount, retained_bytes);
                Err(StoreError::Io(std::io::Error::other(
                    "injected retained release failure",
                )))
            })
            .unwrap_err();
        assert!(matches!(
            release_error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::Other
        ));
        assert!(batch.active);
        assert!(batch.permit.is_none());
        assert!(batch.entries.is_empty());
        assert!(batch.pending_documents.is_none());
        assert!(batch.root.is_none());
        assert!(batch.objects_root.as_os_str().is_empty());
        assert_eq!(batch.reserved_records, 0);
        assert_eq!(batch.reserved_bytes, 0);
        assert_eq!(batch.retained_bytes, retained_bytes);
        assert!(!batch.revalidated);
        assert!(!ledger.has_open_permits());
        assert!(stage_path.exists());
        assert!(object_record_path(&objects, &object_id).is_none());

        batch.abandon_for_durable_retry(&mut ledger).unwrap();
        assert!(!batch.active);
        assert_eq!(batch.retained_bytes, 0);
        assert_prepared_batch_current_zero(&ledger);
        assert!(!ledger.has_open_permits());
        drop(batch);
        assert!(stage_path.exists());
        assert!(object_record_path(&objects, &object_id).is_none());
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();

        let mut retry_ledger = object_transfer_ledger("prepared-object-recover-private-stage");
        let mut retry = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut retry_ledger,
            || false,
        )
        .unwrap();
        assert!(!stage_path.exists());
        retry
            .revalidate_preseal(&mut retry_ledger, || false)
            .unwrap();
        retry_ledger.seal_prepublication().unwrap();
        retry.commit_after_seal(&mut retry_ledger).unwrap();
        assert_eq!(
            read_object_record_document_path(&object_record_path(&objects, &object_id).unwrap())
                .unwrap(),
            document
        );
        assert_prepared_batch_current_zero(&retry_ledger);
        retry_ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn prepared_object_stage_recovery_preserves_unknown_future_and_mismatched_candidates() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("recovery-preserve");
        let other = prepared_policy_document("recovery-mismatch");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let linked = write_prepared_recovery_stage(&objects, &document);
        let linked_alias = objects.join("policies/linked-stage-alias");
        fs::hard_link(&linked, &linked_alias).unwrap();
        let mismatch_name = unique_prepared_object_stage_name(Path::new(&format!(
            "{}.json",
            document.record.object_id
        )))
        .unwrap();
        let mismatch = objects.join("policies").join(mismatch_name);
        let mismatch_bytes =
            serialize_object_record_document_with_limit(&other, &mismatch, MAX_OBJECT_RECORD_BYTES)
                .unwrap();
        fs::write(&mismatch, mismatch_bytes).unwrap();
        let unknown = objects.join("policies/.foreign-stage.tmp");
        let future = objects.join("policies/.codefire-stage-v2.future.tmp");
        let malformed = objects.join("policies/.codefire-stage-v1.malformed.tmp");
        let directory_candidate = objects.join("policies").join(
            unique_prepared_object_stage_name(Path::new(&format!(
                "{}.json",
                document.record.object_id
            )))
            .unwrap(),
        );
        fs::write(&unknown, b"foreign").unwrap();
        fs::write(&future, b"future").unwrap();
        fs::write(&malformed, b"malformed").unwrap();
        fs::create_dir(&directory_candidate).unwrap();

        let mut ledger = object_transfer_ledger("prepared-stage-preserve");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .unwrap();

        for path in [
            &linked,
            &linked_alias,
            &mismatch,
            &unknown,
            &future,
            &malformed,
            &directory_candidate,
        ] {
            assert!(
                path.exists(),
                "recovery removed preserved candidate: {}",
                path.display()
            );
        }
        batch.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_stage_recovery_preserves_name_replacement_before_unlink() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("recovery-identity-swap");
        store_object_record_document(&objects, document.clone()).unwrap();
        let mut ledger = object_transfer_ledger("prepared-stage-identity-swap");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut ledger,
            || false,
        )
        .unwrap();
        let candidate = write_prepared_recovery_stage(&objects, &document);
        let replacement = b"foreign replacement after preflight";

        recover_prepared_object_stages(&batch.entries, &mut ledger, &mut || false, || {
            fs::remove_file(&candidate)?;
            fs::write(&candidate, replacement)?;
            Ok(())
        })
        .unwrap();

        assert_eq!(fs::read(&candidate).unwrap(), replacement);
        batch.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_stage_recovery_limit_and_cancel_do_not_unlink_candidates() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("recovery-preflight");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let first = write_prepared_recovery_stage(&objects, &document);
        let second = write_prepared_recovery_stage(&objects, &document);
        let mut limited = object_transfer_ledger("prepared-stage-limit");
        let limit = limited
            .profile()
            .limit(codefire_core::DecisionWorkDimension::ObjectRecords)
            .unwrap()
            .limit;
        limited
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::ObjectRecords,
                    limit - 2,
                ),
            )
            .unwrap();

        let error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut limited,
            || false,
        )
        .err()
        .expect("second recovery candidate must exceed the object-record budget");
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::ObjectRecords
                        && budget.observed == limit + 1
                        && budget.limit == limit
                })
        ));
        assert!(first.exists());
        assert!(second.exists());
        assert_prepared_batch_current_zero(&limited);

        let mut cancelled = object_transfer_ledger("prepared-stage-cancel");
        let cancel_error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut cancelled,
            || true,
        )
        .err()
        .expect("cancelled recovery must return its typed cancellation");
        assert!(matches!(cancel_error, StoreError::ValidationCancelled));
        assert!(first.exists());
        assert!(second.exists());
        assert_prepared_batch_current_zero(&cancelled);
    }

    #[test]
    fn prepared_object_stage_lock_serializes_batches_and_polls_waiter() {
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("stage-lock");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut owner_ledger = object_transfer_ledger("prepared-stage-lock-owner");
        let mut owner = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document.clone()],
            &mut owner_ledger,
            || false,
        )
        .unwrap();
        let polls = Arc::new(AtomicUsize::new(0));
        let waiter_polls = Arc::clone(&polls);
        let waiter_objects = objects.clone();
        let waiter = std::thread::spawn(move || {
            let mut ledger = object_transfer_ledger("prepared-stage-lock-waiter");
            let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &waiter_objects,
                vec![document],
                &mut ledger,
                || {
                    waiter_polls.fetch_add(1, AtomicOrdering::Relaxed);
                    false
                },
            )
            .unwrap();
            batch.cancel(&mut ledger).unwrap();
            assert_prepared_batch_current_zero(&ledger);
        });

        for _ in 0..100 {
            if polls.load(AtomicOrdering::Relaxed) > 8 {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(polls.load(AtomicOrdering::Relaxed) > 8);
        assert!(!waiter.is_finished());
        let waiting_polls = polls.load(AtomicOrdering::Relaxed);
        std::thread::sleep(Duration::from_millis(50));
        assert!(polls.load(AtomicOrdering::Relaxed) > waiting_polls);
        assert!(!waiter.is_finished());
        owner.cancel(&mut owner_ledger).unwrap();
        assert_prepared_batch_current_zero(&owner_ledger);
        waiter.join().unwrap();
    }

    #[test]
    fn prepared_object_batch_durable_abandon_rejects_foreign_ledger_without_mutation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("abandon-foreign-ledger");
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let mut owner = object_transfer_ledger("prepared-object-abandon-owner");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut owner,
            || false,
        )
        .unwrap();
        let stage_path = {
            let entry = &batch.entries[0];
            let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                panic!("fixture must own a missing stage")
            };
            objects.join(entry.subdir).join(stage_name)
        };
        owner.seal_prepublication().unwrap();
        let mut foreign = object_transfer_ledger("prepared-object-abandon-foreign");
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        let open_permits_before = owner.open_permit_count();
        let retained_bytes = batch.retained_bytes;

        let error = batch.abandon_for_durable_retry(&mut foreign).unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("different decision-work ledger")
        ));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(owner.open_permit_count(), open_permits_before);
        assert!(batch.active);
        assert!(batch.permit.is_some());
        assert_eq!(batch.retained_bytes, retained_bytes);
        assert!(!batch.entries.is_empty());
        assert!(stage_path.exists());

        batch.abandon_for_durable_retry(&mut owner).unwrap();
        batch.abandon_for_durable_retry(&mut owner).unwrap();
        assert_prepared_batch_current_zero(&owner);
        assert!(!owner.has_open_permits());
        assert!(stage_path.exists());
        owner
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_durable_abandon_preserves_partial_publication() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut documents = vec![
            prepared_policy_document("abandon-partial-a"),
            prepared_policy_document("abandon-partial-b"),
        ];
        documents
            .sort_unstable_by(|left, right| left.record.object_id.cmp(&right.record.object_id));
        create_prepared_object_store(&objects, &documents);
        let mut ledger = object_transfer_ledger("prepared-object-abandon-partial");
        let mut batch = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            documents.clone(),
            &mut ledger,
            || false,
        )
        .unwrap();
        let stage_paths = batch
            .entries
            .iter()
            .map(|entry| {
                let PreparedObjectRecordState::Missing { stage_name, .. } = &entry.state else {
                    panic!("fixture entries must own missing stages")
                };
                objects.join(entry.subdir).join(stage_name)
            })
            .collect::<Vec<_>>();
        let first_destination =
            prepared_object_record_path(&objects, &documents[0].record).unwrap();
        let second_destination =
            prepared_object_record_path(&objects, &documents[1].record).unwrap();
        batch.revalidate_preseal(&mut ledger, || false).unwrap();
        ledger.seal_prepublication().unwrap();
        let collision_bytes = b"post-seal foreign collision";
        fs::write(&second_destination, collision_bytes).unwrap();

        let commit_error = batch.commit_inner(&mut ledger).unwrap_err();
        assert!(matches!(
            commit_error,
            StoreError::ObjectPublicationRetryRequired
        ));
        assert!(matches!(
            batch.entries[0].state,
            PreparedObjectRecordState::Published
        ));
        assert!(matches!(
            batch.entries[1].state,
            PreparedObjectRecordState::Missing {
                stage_owned: true,
                ..
            }
        ));
        assert!(!stage_paths[0].exists());
        assert!(stage_paths[1].exists());

        let cleanup_error = batch
            .cancel_inner_with_remove(&mut ledger, |entry| {
                if matches!(
                    entry.state,
                    PreparedObjectRecordState::Missing {
                        stage_owned: true,
                        ..
                    }
                ) {
                    Err(StoreError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "injected partial-publication cleanup failure",
                    )))
                } else {
                    remove_prepared_stage(entry)
                }
            })
            .unwrap_err();
        assert!(matches!(
            cleanup_error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied
        ));

        batch.abandon_for_durable_retry(&mut ledger).unwrap();
        assert_eq!(
            read_object_record_document_path(&first_destination).unwrap(),
            documents[0]
        );
        assert_eq!(fs::read(&second_destination).unwrap(), collision_bytes);
        assert!(!stage_paths[0].exists());
        assert!(stage_paths[1].exists());
        assert_prepared_batch_current_zero(&ledger);
        assert!(!ledger.has_open_permits());
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepared_object_batch_preflights_stage_read_before_creating_stage_effect() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document(&"x".repeat(1024 * 1024));
        create_prepared_object_store(&objects, std::slice::from_ref(&document));
        let rendered_bytes = rendered_object_record_document_size(&document).unwrap();
        let stage_scratch = rendered_bytes.checked_mul(2).unwrap();
        let temporary_limit = codefire_core::DecisionWorkProfile::local_v1()
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let baseline = temporary_limit - stage_scratch + 1;
        let mut ledger = object_transfer_ledger("prepared-object-stage-read-preflight");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ),
            )
            .unwrap();

        let error = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
            &objects,
            vec![document],
            &mut ledger,
            || false,
        )
        .err()
        .expect("stage read limit+1 must fail");

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::TemporaryBytes
                        && budget.observed == temporary_limit + 1
                        && budget.limit == temporary_limit
                })
        ));
        let stage_files = fs::read_dir(objects.join("policies"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(stage_files, 0);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                baseline,
            )
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn prepared_object_batch_retained_limit_minus_one_exact_and_plus_one() {
        let document = prepared_policy_document("retained-boundary");
        let retained_limit = codefire_core::DecisionWorkProfile::local_v1()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for (label, delta, succeeds) in [
            ("minus-one", -1_i64, false),
            ("exact", 0, true),
            ("plus-one", 1, true),
        ] {
            let temp = tempdir().unwrap();
            let objects = temp.path().join("objects");
            create_prepared_object_store(&objects, std::slice::from_ref(&document));
            let (retained, _) = prepared_object_batch_retained_upper(
                &objects,
                std::slice::from_ref(&document),
                &mut || false,
            )
            .unwrap();
            assert!(retained > 1);
            let available = match delta {
                -1 => retained - 1,
                0 => retained,
                1 => retained + 1,
                _ => unreachable!(),
            };
            let mut ledger = object_transfer_ledger(label);
            let baseline = retained_limit - available;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let prepared = PreparedObjectRecordBatch::prepare_with_ledger_and_poll(
                &objects,
                vec![document.clone()],
                &mut ledger,
                || false,
            );
            if succeeds {
                prepared.unwrap().cancel(&mut ledger).unwrap();
            } else {
                let error = match prepared {
                    Ok(mut batch) => {
                        batch.cancel(&mut ledger).unwrap();
                        panic!("retained limit-minus-one unexpectedly prepared")
                    }
                    Err(error) => error,
                };
                assert!(matches!(
                    error,
                    StoreError::DecisionWorkAccounting(error)
                        if error.budget_exceeded().is_some_and(|budget| {
                            budget.dimension
                                == codefire_core::DecisionWorkDimension::RetainedBytes
                                && budget.observed == retained_limit + 1
                                && budget.limit == retained_limit
                        })
                ));
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_prepared_batch_current_zero(&ledger);
            if succeeds {
                ledger
                    .finish(codefire_core::DecisionWorkTerminalReason::Complete)
                    .unwrap();
            }
        }
    }

    #[test]
    fn canonical_json_sorts_keys_and_uses_compact_utf8() {
        let payload = json!({
            "z": 1,
            "a": "火",
            "nested": {
                "b": true,
                "a": null
            }
        });

        let rendered = String::from_utf8(canonical_json(&payload).unwrap()).unwrap();

        assert_eq!(rendered, r#"{"a":"火","nested":{"a":null,"b":true},"z":1}"#);
    }

    #[test]
    fn object_digest_matches_python_golden_payloads() {
        let blob = json!({
            "path": "docs/spec/auth.md",
            "content_hash": "sha256:aaa111",
            "bytes": "hello"
        });
        let atom_index = json!({
            "type": "atom_index",
            "version": 1,
            "duplicate_atom_ids": [],
            "atoms": [
                {
                    "atom_id": "REQ-AUTH-001",
                    "kind": "requirement",
                    "artifact_path": "docs/spec/auth.md",
                    "selector": {"type": "markdown_heading", "value": "REQ-AUTH-001"},
                    "content_hash": "sha256:aaa111"
                }
            ]
        });
        let commit = json!({
            "type": "commit",
            "version": 1,
            "parents": ["CF-COMMIT-parent"],
            "message": "golden",
            "roots": {
                "content_manifest": "CF-MANIFEST-abc",
                "atom_index": "CF-ATOMINDEX-def",
                "trace_graph": "CF-TRACE-ghi",
                "verification": "CF-VERIFY-jkl",
                "policy": "CF-POLICY-mno"
            },
            "certificate": {
                "result": "consistent",
                "open_required_fires": 0,
                "failed_checks": 0,
                "missing_required_links": 0,
                "stale_resolutions": 0,
                "duplicate_atom_ids": 0
            }
        });

        assert_eq!(
            object_digest("blob", &blob).unwrap(),
            "3b9124b51836f77522a243c6caa86d1a8773dc443246ce9860a1c0f630343514"
        );
        assert_eq!(
            object_id("blob", &blob).unwrap(),
            "CF-BLOB-3b9124b51836f77522a243c6"
        );
        assert_eq!(
            object_digest("atom_index", &atom_index).unwrap(),
            "6f78247e9b25327f61e2b86d57bca8f63a0dc781ae661ae50361d997b23ac0ea"
        );
        assert_eq!(
            object_id("atom_index", &atom_index).unwrap(),
            "CF-ATOMINDEX-6f78247e9b25327f61e2b86d"
        );
        assert_eq!(
            object_digest("commit", &commit).unwrap(),
            "f2c6dacf47ce757c611162a7e545cb6ea643aa27e877996f19e9268b1746c8c4"
        );
        assert_eq!(
            object_id("commit", &commit).unwrap(),
            "CF-COMMIT-f2c6dacf47ce757c611162a7"
        );
    }

    #[test]
    fn unknown_object_type_uses_generic_prefix() {
        let payload = json!({"type": "future", "version": 1});

        assert!(object_id("future", &payload)
            .unwrap()
            .starts_with("CF-OBJECT-"));
    }

    #[test]
    fn known_object_subdirs_are_single_source_of_truth() {
        let subdirs = known_object_subdirs().collect::<Vec<_>>();
        assert!(subdirs.contains(&"blobs"));
        assert!(subdirs.contains(&"commits"));
        assert!(subdirs.contains(&"evidence"));
        assert_eq!(object_subdir("commit"), Some("commits"));
        assert_eq!(object_prefix("commit"), "CF-COMMIT");
        assert_eq!(OBJECT_ID_DIGEST_HEX_LENGTH, 24);
        assert_eq!(LEGACY_OBJECT_ID_DIGEST_HEX_LENGTH, 12);
    }

    #[test]
    fn object_record_path_routes_known_prefix_to_subdir() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commits = objects.join("commits");
        fs::create_dir_all(&commits).unwrap();
        let commit_id = "CF-COMMIT-123456789abc";
        let path = commits.join(format!("{commit_id}.json"));
        fs::write(&path, "{}\n").unwrap();

        assert_eq!(object_record_path(&objects, commit_id), Some(path));
        assert_eq!(object_record_path(&objects, "CF-COMMIT-missing"), None);
    }

    #[test]
    fn object_record_path_rejects_unknown_codefire_prefix_without_subdir_scan() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let custom = objects.join("custom");
        fs::create_dir_all(&custom).unwrap();
        let future_id = "CF-FUTURE-123456789abc";
        let future_path = custom.join(format!("{future_id}.json"));
        fs::write(future_path, "{}\n").unwrap();

        assert_eq!(object_record_path(&objects, future_id), None);

        let generic_id = "CF-OBJECT-123456789abc";
        let generic_path = custom.join(format!("{generic_id}.json"));
        fs::write(generic_path.as_path(), "{}\n").unwrap();
        assert_eq!(object_record_path(&objects, generic_id), Some(generic_path));
    }

    #[test]
    fn object_record_reader_rejects_oversize_regular_file_before_reading() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commits = objects.join("commits");
        fs::create_dir_all(&commits).unwrap();
        let object_id = "CF-COMMIT-oversize";
        let path = commits.join(format!("{object_id}.json"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.set_len(MAX_OBJECT_RECORD_BYTES + 1).unwrap();
        drop(file);

        let error = read_object(&objects, object_id).unwrap_err();

        assert!(matches!(
            error,
            StoreError::ObjectRecordTooLarge {
                path: rejected_path,
                size,
                limit,
            } if rejected_path == path
                && size == MAX_OBJECT_RECORD_BYTES + 1
                && limit == MAX_OBJECT_RECORD_BYTES
        ));
    }

    #[test]
    fn object_record_reader_rejects_growth_after_metadata_check() {
        let temp = tempdir().unwrap();
        let directory_path = temp.path().join("objects/policies");
        fs::create_dir_all(&directory_path).unwrap();
        let path = directory_path.join("CF-POLICY-growing.json");
        let record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"growing": true}}),
        )
        .unwrap();
        write_record_fixture(&path, &record);
        let initial_size = fs::metadata(&path).unwrap().len();
        let directory = Dir::open_ambient_dir(&directory_path, ambient_authority()).unwrap();

        let error = read_object_record_document_from_directory_with_limit_and_hook(
            &directory,
            path.file_name().unwrap(),
            &path,
            initial_size,
            |_| {
                OpenOptions::new()
                    .write(true)
                    .open(&path)?
                    .set_len(initial_size + 1)?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::ObjectRecordTooLarge {
                path: rejected_path,
                size,
                limit,
            } if rejected_path == path
                && size == initial_size + 1
                && limit == initial_size
        ));
    }

    #[test]
    fn object_record_reader_rejects_a_directory() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let object_id = "CF-COMMIT-directory";
        let path = objects.join("commits").join(format!("{object_id}.json"));
        fs::create_dir_all(path).unwrap();

        let error = read_object(&objects, object_id).unwrap_err();

        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("regular no-follow file") && message.contains(object_id)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn object_record_reader_rejects_fifo_without_waiting_for_a_writer() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let object_id = "CF-COMMIT-fifo";
        let path = objects.join("commits").join(format!("{object_id}.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let status = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap();
        assert!(status.success());

        let started = std::time::Instant::now();
        let error = read_object(&objects, object_id).unwrap_err();

        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("regular no-follow file") && message.contains(object_id)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn object_record_reader_rejects_device_file() {
        let path = Path::new("/dev/null");
        if !path.exists() {
            return;
        }

        let error = read_object_record_path(path).unwrap_err();

        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("regular no-follow file")
                    && message.contains("/dev/null")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn object_record_reader_does_not_follow_final_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commits = objects.join("commits");
        fs::create_dir_all(&commits).unwrap();
        let object_id = "CF-COMMIT-symlink";
        let target = temp.path().join("outside.json");
        fs::write(&target, b"{}\n").unwrap();
        symlink(&target, commits.join(format!("{object_id}.json"))).unwrap();

        let error = read_object(&objects, object_id).unwrap_err();

        assert!(matches!(error, StoreError::Io(_)));
    }

    #[test]
    fn read_object_accepts_legacy_twelve_hex_object_ids() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let policies = objects.join("policies");
        fs::create_dir_all(&policies).unwrap();
        let payload = json!({"type": "policy", "version": 1, "policy": {"legacy": true}});
        let hash = object_digest("policy", &payload).unwrap();
        let object_id = legacy_object_id("policy", &hash);
        let record = ObjectRecord {
            object_id: object_id.clone(),
            type_tag: "policy".to_string(),
            hash,
            payload: payload.clone(),
        };
        let path = policies.join(format!("{object_id}.json"));
        fs::write(path, serde_json::to_string_pretty(&record).unwrap()).unwrap();

        let loaded = read_object(&objects, &object_id).unwrap();

        assert_eq!(loaded, payload);
    }

    #[test]
    fn object_document_reader_retains_top_level_extensions_without_known_fields() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"remote": true}}),
        )
        .unwrap();
        let path = objects
            .join("policies")
            .join(format!("{}.json", record.object_id));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut document = serde_json::to_value(&record).unwrap();
        document["remote"] = json!({
            "first_seen_generation": 3,
            "last_seen_generation": 7,
        });
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        let loaded = read_object_record_document_path(&path).unwrap();

        assert_eq!(loaded.record, record);
        assert_eq!(loaded.extensions.len(), 1);
        assert_eq!(loaded.extensions["remote"]["first_seen_generation"], 3);
        assert_eq!(loaded.extensions["remote"]["last_seen_generation"], 7);
        validate_object_record(&loaded.record, None, Some(&path)).unwrap();
    }

    #[test]
    fn object_document_store_preserves_noncritical_extension_descriptors() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": true}}),
        )
        .unwrap();
        let mut extensions = Map::new();
        extensions.insert(
            "extensions".to_string(),
            json!([{
                "namespace": "example.test",
                "name": "annotation",
                "version": 1,
                "critical": false,
                "references": [],
                "payload": {"label": "kept"}
            }]),
        );
        let document = ObjectRecordDocument {
            record: record.clone(),
            extensions: extensions.clone(),
        };

        let object_id = store_object_record_document(&objects, document).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let loaded = read_object_record_document_path(&path).unwrap();

        assert_eq!(loaded.record, record);
        assert_eq!(loaded.extensions, extensions);
    }

    #[test]
    fn object_document_reader_rejects_unknown_critical_extension() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"portable": false}}),
        )
        .unwrap();
        let path = objects
            .join("policies")
            .join(format!("{}.json", record.object_id));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut value = serde_json::to_value(record).unwrap();
        value["extensions"] = json!([{
            "namespace": "example.test",
            "name": "required-proof",
            "version": 1,
            "critical": true,
            "references": []
        }]);
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let error = read_object_record_document_path(&path).unwrap_err();

        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("unsupported critical object extension")
        ));
    }

    #[test]
    fn object_store_snapshot_scan_is_deterministic_and_keeps_entry_errors() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let policy_id = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let extension_record = object_record(
            "extension",
            json!({"type": "extension", "version": 1, "value": "kept"}),
        )
        .unwrap();
        let extension_path = objects
            .join("extension_objects")
            .join(format!("{}.json", extension_record.object_id));
        fs::create_dir_all(extension_path.parent().unwrap()).unwrap();
        let mut extension_document = serde_json::to_value(&extension_record).unwrap();
        extension_document["remote"] = json!({"last_seen_generation": 9});
        fs::write(
            &extension_path,
            serde_json::to_vec_pretty(&extension_document).unwrap(),
        )
        .unwrap();
        let malformed_path = objects.join("policies/zz-malformed.json");
        fs::write(&malformed_path, b"{\"object_id\":").unwrap();
        let mut entries = Vec::new();

        scan_object_store_record_documents(&objects, |entry| entries.push(entry)).unwrap();

        assert!(entries.windows(2).all(|pair| pair[0].path < pair[1].path));
        let policy_path = objects
            .canonicalize()
            .unwrap()
            .join("policies")
            .join(format!("{policy_id}.json"));
        assert!(entries
            .iter()
            .any(|entry| entry.path == policy_path && entry.document.is_ok()));
        let extension = entries
            .iter()
            .find(|entry| entry.path == extension_path)
            .unwrap()
            .document
            .as_ref()
            .unwrap();
        assert_eq!(extension.extensions["remote"]["last_seen_generation"], 9);
        assert!(entries.iter().any(|entry| {
            entry.path == malformed_path && matches!(entry.document, Err(StoreError::Json(_)))
        }));
    }

    fn object_store_scan_ledger_with_remaining_records(
        remaining: u64,
    ) -> codefire_core::DecisionWorkLedger {
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::ObjectTransfer,
            "object-store-scan-test",
        )
        .unwrap();
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::ObjectRecords)
            .unwrap()
            .limit;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::ObjectRecords,
                    limit - remaining,
                ),
            )
            .unwrap();
        ledger
    }

    fn assert_scan_memory_released(ledger: &codefire_core::DecisionWorkLedger) {
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0,
                "scanner leaked {dimension:?}"
            );
        }
    }

    #[test]
    fn accounted_object_store_scan_is_ordered_and_stops_at_record_boundary() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        store_object(
            &objects,
            "blob",
            json!({"type": "blob", "version": 1, "data": "scan"}),
        )
        .unwrap();

        let mut exact = object_store_scan_ledger_with_remaining_records(2);
        let mut paths = Vec::new();
        scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut exact,
            || false,
            |entry, _| {
                assert!(entry.document.is_ok());
                paths.push(entry.path);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            exact.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
            2
        );
        assert_eq!(
            exact.snapshot().counters[&codefire_core::DecisionWorkCounter::ObjectRecords],
            2
        );
        assert_eq!(
            exact.snapshot().counters[&codefire_core::DecisionWorkCounter::SemanticValidations],
            2
        );
        assert!(exact.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed] > 0);
        assert_scan_memory_released(&exact);

        let mut plus_one = object_store_scan_ledger_with_remaining_records(1);
        let mut visited = 0_u64;
        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut plus_one,
            || false,
            |_, _| {
                visited += 1;
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(visited, 1);
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::ObjectRecords
                })
        ));
        assert_scan_memory_released(&plus_one);
    }

    #[test]
    fn accounted_object_store_scan_cancellation_releases_owned_memory() {
        use std::cell::Cell;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let visits = Cell::new(0_u64);
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || visits.get() > 0,
            |entry, _| {
                assert!(entry.document.is_ok());
                visits.set(visits.get() + 1);
                Ok(())
            },
        )
        .unwrap_err();

        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(visits.get(), 1);
        assert_scan_memory_released(&ledger);
        assert_eq!(ledger.snapshot().terminal_reason, None);
    }

    #[test]
    fn accounted_object_store_scan_visitor_error_releases_owned_memory() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |entry, _| {
                assert!(entry.document.is_ok());
                Err(StoreError::InvalidSealedCommit(
                    "diagnostic visitor stopped".to_string(),
                ))
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message == "diagnostic visitor stopped"
        ));
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_accounts_bind_directory_entries_without_leaks() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let graph_limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::GraphNodes)
            .unwrap()
            .limit;
        ledger
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

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |_, _| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::GraphNodes
                })
        ));
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_hash_validation_is_accounted_and_reported() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let object_id = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let path = objects.join("policies").join(format!("{object_id}.json"));
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["hash"] = Value::String("0".repeat(64));
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let mut saw_hash_error = false;

        scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |entry, _| {
                saw_hash_error =
                    matches!(entry.document, Err(StoreError::ObjectHashMismatch { .. }));
                Ok(())
            },
        )
        .unwrap();

        assert!(saw_hash_error);
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::SemanticValidations],
            1
        );
        assert!(ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed] > 0);
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_final_identity_enumeration_releases_each_frame() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let mut cache = SealedCommitValidationCache::default();
        let bound = bind_object_store(objects.clone(), &mut cache).unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let graph_limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::GraphNodes)
            .unwrap()
            .limit;
        ledger
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

        let error = ensure_bound_object_store_identity_with_work_and_cancellation(
            &bound,
            &cache,
            &mut ledger,
            &mut || false,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::GraphNodes
                })
        ));
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_preallocates_name_before_directory_entry_materialization() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let entry_preallocation = OBJECT_STORE_DIRECTORY_ITERATOR_SCRATCH_BYTES
            + OBJECT_STORE_DIRECTORY_ENTRY_PREALLOCATION_BYTES;
        let baseline = limit - (entry_preallocation - 1);
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::GraphDiscovery,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ),
            )
            .unwrap();

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |_, _| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::TemporaryBytes
                        && exhaustion.observed == exhaustion.limit + 1
                        && exhaustion.phase
                            == codefire_core::DecisionWorkPhase::GraphDiscovery
                        && exhaustion.command == "object-store-scan-test"
                })
        ));
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied(),
            Some(baseline)
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
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                baseline,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_preallocates_callback_path_before_join() {
        let temp = tempdir().unwrap();
        let canonical_root = temp.path().join("objects");
        let subdir = Path::new("policies");
        let name = Path::new("CF-POLICY-callback-path.json");
        let preallocated_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
            + 2
            + u64::try_from(canonical_root.as_os_str().as_encoded_bytes().len()).unwrap()
            + u64::try_from(subdir.as_os_str().as_encoded_bytes().len()).unwrap()
            + u64::try_from(name.as_os_str().as_encoded_bytes().len()).unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(0);
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let baseline = limit - (preallocated_bytes - 1);
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                ),
            )
            .unwrap();

        let error =
            build_object_store_scan_path_accounted(&canonical_root, subdir, name, &mut ledger)
                .unwrap_err();

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::RetainedBytes
                        && exhaustion.observed == exhaustion.limit + 1
                        && exhaustion.phase == codefire_core::DecisionWorkPhase::RecordRead
                        && exhaustion.command == "object-store-scan-test"
                })
        ));
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            baseline
        );
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                baseline,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_reserves_full_root_owners_before_cloning() {
        let temp = tempdir().unwrap();
        let objects = temp
            .path()
            .join("a".repeat(200))
            .join("b".repeat(200))
            .join("c".repeat(200))
            .join("objects");
        let retained_root_bytes = object_store_scan_name_frame_bytes(&objects)
            .unwrap()
            .checked_mul(2)
            .unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(0);
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let baseline = limit - (retained_root_bytes - 1);
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                ),
            )
            .unwrap();

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |_, _| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::RetainedBytes
                        && exhaustion.observed == exhaustion.limit + 1
                        && exhaustion.phase
                            == codefire_core::DecisionWorkPhase::GraphDiscovery
                        && exhaustion.command == "object-store-scan-test"
                })
        ));
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            baseline
        );
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                baseline,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_scan_holds_membership_through_final_revalidation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let late_member = objects.join("policies/late-member.tmp");
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let mut visits = 0_u64;

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |entry, _| {
                assert!(entry.document.is_ok());
                visits += 1;
                fs::write(&late_member, b"late").unwrap();
                Ok(())
            },
        )
        .unwrap_err();

        assert_eq!(visits, 1);
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed during sealed validation")
                    && message.contains("policies")
        ));
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_final_membership_budget_is_current_and_exact() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let object_id = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        for index in 0..8 {
            fs::write(
                objects
                    .join("policies")
                    .join(format!("000-{index}-{}.member", "x".repeat(220))),
                b"member",
            )
            .unwrap();
        }
        let document = read_object_record_document_path(
            &objects.join("policies").join(format!("{object_id}.json")),
        )
        .unwrap();
        let document_bytes = compact_object_record_document_size(&document).unwrap();
        let name_owner_bytes = codefire_core::LOGICAL_FRAME_BASE_BYTES
            + known_object_subdirs()
                .map(|name| object_store_scan_name_frame_bytes(Path::new(name)).unwrap())
                .sum::<u64>();
        let binding_bytes = name_owner_bytes * 3 + codefire_core::LOGICAL_FRAME_BASE_BYTES * 2;
        let membership_bytes = fs::read_dir(objects.join("policies"))
            .unwrap()
            .map(|entry| {
                let name = PathBuf::from(entry.unwrap().file_name());
                object_store_scan_name_frame_bytes(&name).unwrap()
            })
            .sum::<u64>();
        assert!(membership_bytes > document_bytes);
        let record_path = objects.join("policies").join(format!("{object_id}.json"));
        let path_bytes = object_store_scan_name_frame_bytes(&record_path).unwrap();
        let root_owner_bytes = object_store_scan_name_frame_bytes(&objects).unwrap() * 2;
        let required_before_final =
            binding_bytes + root_owner_bytes + membership_bytes + document_bytes + path_bytes;
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let baseline = limit - required_before_final;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                ),
            )
            .unwrap();
        let mut visits = 0_u64;

        let error = scan_object_store_record_documents_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
            |entry, _| {
                assert!(entry.document.is_ok());
                visits += 1;
                Ok(())
            },
        )
        .unwrap_err();

        assert_eq!(visits, 1);
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|exhaustion| {
                    exhaustion.dimension == codefire_core::DecisionWorkDimension::RetainedBytes
                        && exhaustion.observed == exhaustion.limit + 1
                        && exhaustion.phase
                            == codefire_core::DecisionWorkPhase::SnapshotRevalidation
                })
        ));
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(baseline)
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                baseline,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_object_store_record_read_and_parse_poll_between_chunks() {
        use std::cell::Cell;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let object_id = store_object(
            &objects,
            "policy",
            json!({
                "type": "policy",
                "version": 1,
                "policy": {"large": "x".repeat(OBJECT_STORE_TRAVERSAL_POLL_BYTES * 3)},
            }),
        )
        .unwrap();
        let directory_path = objects.join("policies");
        let file_name = PathBuf::from(format!("{object_id}.json"));
        let display_path = directory_path.join(&file_name);
        let directory = Dir::open_ambient_dir(&directory_path, ambient_authority()).unwrap();
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);
        let read_polls = Cell::new(0_u64);

        let error = read_object_store_scan_document_accounted(
            &directory,
            &file_name,
            &display_path,
            &mut ledger,
            &mut || {
                read_polls.set(read_polls.get() + 1);
                read_polls.get() >= 2
            },
        )
        .unwrap_err();

        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(read_polls.get(), 2);
        assert_scan_memory_released(&ledger);

        let bytes = fs::read(&display_path).unwrap();
        let parse_polls = Cell::new(0_u64);
        let mut parse_ledger = object_store_scan_ledger_with_remaining_records(1);
        let error = parse_object_record_document_with_work_and_cancellation(
            &bytes,
            &mut parse_ledger,
            &mut || {
                parse_polls.set(parse_polls.get() + 1);
                parse_polls.get() >= 2
            },
        )
        .unwrap_err();
        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(parse_polls.get(), 2);
        assert_scan_memory_released(&parse_ledger);
    }

    #[test]
    fn object_store_record_reference_traversal_is_cancellable() {
        use std::cell::Cell;

        let references = (0..256)
            .map(|index| Value::String(format!("CF-OBJECT-{index:024}")))
            .collect::<Vec<_>>();
        let bytes = serde_json::to_vec(&json!({
            "object_id": "CF-POLICY-placeholder",
            "type": "policy",
            "hash": "placeholder",
            "payload": {"type": "policy", "version": 1},
            "extensions": [{
                "namespace": "example.test",
                "name": "references",
                "version": 1,
                "critical": false,
                "references": references,
            }],
        }))
        .unwrap();
        let polls = Cell::new(0_u64);
        let mut ledger = object_store_scan_ledger_with_remaining_records(1);

        let error = parse_object_record_document_with_work_and_cancellation(
            &bytes,
            &mut ledger,
            &mut || {
                polls.set(polls.get() + 1);
                polls.get() > 32
            },
        )
        .unwrap_err();

        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(polls.get(), 33);
        assert_scan_memory_released(&ledger);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn object_store_snapshot_scan_rejects_subdir_replacement() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"scan": true}}),
        )
        .unwrap();
        let moved = temp.path().join("policies-before-swap");
        let mut visited = 0usize;

        let error = scan_object_store_record_documents_with_hook(
            &objects,
            |_| visited += 1,
            |_| {
                fs::rename(objects.join("policies"), &moved)?;
                fs::create_dir(objects.join("policies"))?;
                Ok(())
            },
        )
        .unwrap_err();

        assert_eq!(visited, 1);
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn object_store_snapshot_scan_rejects_new_extension_namespace() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();

        let error = scan_object_store_record_documents_with_hook(
            &objects,
            |_| {},
            |_| {
                fs::create_dir(objects.join("late-extension"))?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
    }

    #[test]
    fn store_object_writes_and_reuses_immutable_record() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload =
            json!({"type": "policy", "version": 1, "policy": {"require_commit_signature": false}});

        let first_id = store_object(&objects, "policy", payload.clone()).unwrap();
        let second_id = store_object(&objects, "policy", payload.clone()).unwrap();
        let loaded = read_object(&objects, &first_id).unwrap();

        assert_eq!(first_id, second_id);
        assert_eq!(loaded, payload);
        assert!(objects
            .join("policies")
            .join(format!("{first_id}.json"))
            .exists());
        let leftovers = fs::read_dir(objects.join("policies"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[test]
    fn store_object_record_preserves_and_reuses_valid_legacy_public_id() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload = json!({"type": "policy", "version": 1, "policy": {"legacy": true}});
        let mut legacy = object_record("policy", payload.clone()).unwrap();
        legacy.object_id = legacy_object_id(&legacy.type_tag, &legacy.hash);
        let legacy_id = legacy.object_id.clone();

        let first = store_object_record(&objects, legacy.clone()).unwrap();
        let path = objects.join("policies").join(format!("{legacy_id}.json"));
        let first_bytes = fs::read(&path).unwrap();
        let second = store_object_record(&objects, legacy.clone()).unwrap();

        assert_eq!(first, legacy_id);
        assert_eq!(second, legacy_id);
        assert_eq!(fs::read(&path).unwrap(), first_bytes);
        assert_eq!(read_object_record(&objects, &legacy_id).unwrap(), legacy);

        let current_id = store_object(&objects, "policy", payload).unwrap();
        assert_ne!(current_id, legacy_id);
        assert_eq!(current_id.rsplit_once('-').unwrap().1.len(), 24);
        assert_eq!(legacy_id.rsplit_once('-').unwrap().1.len(), 12);
    }

    #[test]
    fn store_object_record_rejects_invalid_identity_before_creating_store() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"valid": true}}),
        )
        .unwrap();
        record.object_id = "CF-POLICY-000000000000".to_string();

        let error = store_object_record(&objects, record).unwrap_err();

        assert!(matches!(error, StoreError::ObjectIdMismatch { .. }));
        assert!(!objects.exists());
    }

    #[test]
    fn exact_legacy_record_publication_rejects_short_id_collision() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut existing = object_record_with_digest(
            "policy",
            artificial_collision_payload("existing"),
            &artificial_collision_digest,
        )
        .unwrap();
        existing.object_id = legacy_object_id(&existing.type_tag, &existing.hash);
        let mut requested = object_record_with_digest(
            "policy",
            artificial_collision_payload("requested"),
            &artificial_collision_digest,
        )
        .unwrap();
        requested.object_id = legacy_object_id(&requested.type_tag, &requested.hash);
        assert_eq!(existing.object_id, requested.object_id);
        assert_ne!(existing.hash, requested.hash);
        let path = objects
            .join("policies")
            .join(format!("{}.json", existing.object_id));
        write_record_fixture(&path, &existing);

        let error = store_object_record_with_identity_and_link(
            &objects,
            requested.clone(),
            &artificial_collision_digest,
            |_, _, _, _| panic!("existing-path collision must not attempt publication"),
        )
        .unwrap_err();

        assert_collision_error(error, &requested, &existing, false);
        assert_eq!(read_object_record_path(&path).unwrap(), existing);
    }

    #[test]
    fn object_writer_applies_the_reader_limit_before_publication() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("objects/policies/CF-POLICY-sized.json");
        let record = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"required": true}}),
        )
        .unwrap();
        let rendered = serialize_object_record_with_limit(&record, &path, u64::MAX).unwrap();
        let rejected_limit = rendered.len() as u64 - 1;

        let error = serialize_object_record_with_limit(&record, &path, rejected_limit).unwrap_err();

        assert!(matches!(
            error,
            StoreError::ObjectRecordTooLarge {
                path: rejected_path,
                size,
                limit,
            } if rejected_path == path
                && size == rendered.len() as u64
                && limit == rejected_limit
        ));
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn object_writer_rejects_symlinked_store_components_without_outside_writes() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&objects).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, objects.join("blobs")).unwrap();

        let error = store_object(
            &objects,
            "blob",
            json!({"type": "blob", "version": 1, "encoding": "base64", "data": "c2FmZQ=="}),
        )
        .unwrap_err();

        assert!(matches!(error, StoreError::Io(_)));
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn object_writer_detects_subdir_replacement_after_pinned_publication() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let moved = temp.path().join("moved-policies");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        let payload = json!({"type": "policy", "version": 1, "policy": {"pinned": true}});

        let error = store_object_with_identity_and_link(
            &objects,
            "policy",
            payload,
            &object_digest,
            |directory, source, destination, display_path| {
                let policies = display_path.parent().unwrap();
                fs::rename(policies, &moved)?;
                symlink(&outside, policies)?;
                directory.hard_link(source, directory, destination)
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("identity changed during object publication")
        ));
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
        assert_eq!(
            fs::read_dir(&moved)
                .unwrap()
                .filter_map(Result::ok)
                .filter(
                    |entry| entry.path().extension().and_then(|value| value.to_str())
                        == Some("json")
                )
                .count(),
            1
        );
    }

    #[test]
    fn object_reuse_requires_type_hash_and_canonical_payload_identity() {
        let requested = object_record(
            "policy",
            json!({"type": "policy", "version": 1, "policy": {"required": true}}),
        )
        .unwrap();

        require_same_object_identity(&requested, &requested).unwrap();

        let mut wrong_type = requested.clone();
        wrong_type.type_tag = "verification".to_string();
        let error = require_same_object_identity(&requested, &wrong_type).unwrap_err();
        assert_collision_error(error, &requested, &wrong_type, true);

        let mut wrong_hash = requested.clone();
        wrong_hash.hash = "f".repeat(64);
        let error = require_same_object_identity(&requested, &wrong_hash).unwrap_err();
        assert_collision_error(error, &requested, &wrong_hash, true);

        let mut wrong_payload = requested.clone();
        wrong_payload.payload["policy"]["required"] = json!(false);
        let error = require_same_object_identity(&requested, &wrong_payload).unwrap_err();
        assert_collision_error(error, &requested, &wrong_payload, false);
    }

    #[test]
    fn store_object_rejects_artificial_collision_at_existing_path() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let existing_payload = artificial_collision_payload("existing");
        let requested_payload = artificial_collision_payload("requested");
        let existing =
            object_record_with_digest("policy", existing_payload, &artificial_collision_digest)
                .unwrap();
        let requested = object_record_with_digest(
            "policy",
            requested_payload.clone(),
            &artificial_collision_digest,
        )
        .unwrap();
        assert_eq!(existing.object_id, requested.object_id);
        assert_ne!(existing.hash, requested.hash);
        assert_eq!(
            requested.object_id.rsplit_once('-').unwrap().1.len(),
            OBJECT_ID_DIGEST_HEX_LENGTH
        );
        let path = objects
            .join("policies")
            .join(format!("{}.json", existing.object_id));
        write_record_fixture(&path, &existing);

        let error = store_object_with_identity_and_link(
            &objects,
            "policy",
            requested_payload,
            &artificial_collision_digest,
            |_, _, _, _| panic!("existing-path reuse must not attempt publication"),
        )
        .unwrap_err();

        assert_collision_error(error, &requested, &existing, false);
        assert_eq!(read_object_record_path(&path).unwrap(), existing);
    }

    #[test]
    fn store_object_rejects_canonical_payload_mismatch_even_when_full_hash_matches() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let existing = object_record_with_digest(
            "policy",
            artificial_collision_payload("existing"),
            &artificial_full_digest_collision,
        )
        .unwrap();
        let requested_payload = artificial_collision_payload("requested");
        let requested = object_record_with_digest(
            "policy",
            requested_payload.clone(),
            &artificial_full_digest_collision,
        )
        .unwrap();
        assert_eq!(existing.object_id, requested.object_id);
        assert_eq!(existing.hash, requested.hash);
        assert_ne!(existing.payload, requested.payload);
        let path = objects
            .join("policies")
            .join(format!("{}.json", existing.object_id));
        write_record_fixture(&path, &existing);

        let error = store_object_with_identity_and_link(
            &objects,
            "policy",
            requested_payload,
            &artificial_full_digest_collision,
            |_, _, _, _| panic!("existing-path reuse must not attempt publication"),
        )
        .unwrap_err();

        assert_collision_error(error, &requested, &existing, false);
        assert_eq!(read_object_record_path(&path).unwrap(), existing);
    }

    #[test]
    fn store_object_reports_corrupt_existing_record_before_collision() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut existing = object_record_with_digest(
            "policy",
            artificial_collision_payload("existing"),
            &artificial_collision_digest,
        )
        .unwrap();
        let computed_hash = existing.hash.clone();
        existing.hash = "f".repeat(64);
        let path = objects
            .join("policies")
            .join(format!("{}.json", existing.object_id));
        write_record_fixture(&path, &existing);

        let error = store_object_with_identity_and_link(
            &objects,
            "policy",
            artificial_collision_payload("requested"),
            &artificial_collision_digest,
            |_, _, _, _| panic!("existing-path reuse must not attempt publication"),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::ObjectHashMismatch { expected, actual }
                if expected == computed_hash && actual == "f".repeat(64)
        ));
    }

    #[test]
    fn store_object_rejects_artificial_collision_after_link_race() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let existing_payload = artificial_collision_payload("existing");
        let requested_payload = artificial_collision_payload("requested");
        let existing =
            object_record_with_digest("policy", existing_payload, &artificial_collision_digest)
                .unwrap();
        let requested = object_record_with_digest(
            "policy",
            requested_payload.clone(),
            &artificial_collision_digest,
        )
        .unwrap();
        assert_eq!(existing.object_id, requested.object_id);
        assert_ne!(existing.hash, requested.hash);
        let path = objects
            .join("policies")
            .join(format!("{}.json", existing.object_id));
        assert!(!path.exists());
        let fixture = existing.clone();

        let error = store_object_with_identity_and_link(
            &objects,
            "policy",
            requested_payload,
            &artificial_collision_digest,
            move |_, _, _, destination| {
                write_record_fixture(destination, &fixture);
                Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists))
            },
        )
        .unwrap_err();

        assert_collision_error(error, &requested, &existing, false);
        assert_eq!(read_object_record_path(&path).unwrap(), existing);
        let leftovers = fs::read_dir(objects.join("policies"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[test]
    fn store_object_supports_artifact_ref_and_evidence_records() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let artifact = json!({
            "type": "artifact_ref",
            "version": 1,
            "uri": "/tmp/model.bin",
            "hash_algorithm": "sha256",
            "content_hash": "sha256:abc",
            "size_bytes": 3
        });
        let artifact_id = store_object(&objects, "artifact_ref", artifact.clone()).unwrap();
        let evidence = json!({
            "type": "evidence",
            "version": 1,
            "artifact_ref": artifact_id,
            "created_at": "2026-06-05T00:00:00Z"
        });
        let evidence_id = store_object(&objects, "evidence", evidence).unwrap();

        assert!(artifact_id.starts_with("CF-ARTIFACT-"));
        assert!(evidence_id.starts_with("CF-EVIDENCE-"));
        assert!(objects
            .join("artifact_refs")
            .join(format!("{artifact_id}.json"))
            .exists());
        assert!(objects
            .join("evidence")
            .join(format!("{evidence_id}.json"))
            .exists());
    }

    #[test]
    fn read_object_rejects_tampered_record() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload = json!({"type": "verification", "version": 1, "checks": []});
        let object_id = store_object(&objects, "verification", payload).unwrap();
        let path = objects
            .join("verifications")
            .join(format!("{object_id}.json"));
        let mut record: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        record["payload"]["version"] = json!(2);
        fs::write(&path, serde_json::to_string_pretty(&record).unwrap()).unwrap();

        let error = read_object(&objects, &object_id).unwrap_err();

        assert!(matches!(error, StoreError::ObjectIdMismatch { .. }));
    }

    #[test]
    fn accounted_exact_path_read_transfers_then_releases_retained_document() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-exact-read");
        let object_id = store_object_record_document(&objects, document.clone()).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let mut ledger = object_transfer_ledger("accounted-exact-read");

        let mut accounted =
            read_object_record_document_path_with_work_and_cancellation(&path, &mut ledger, || {
                false
            })
            .unwrap();

        assert_eq!(accounted.document(), &document);
        assert!(accounted.retained_bytes() > 0);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        accounted.release(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn accounted_exact_path_read_preflights_tiny_parse_scratch() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("tiny-parse-scratch");
        let object_id = store_object_record_document(&objects, document).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let scratch_bytes =
            accounted_object_parse_scratch_bytes(fs::metadata(&path).unwrap().len()).unwrap();

        for remaining_delta in [-1_i64, 0, 1] {
            let mut ledger = object_transfer_ledger("accounted-tiny-parse-scratch");
            let limit = ledger
                .profile()
                .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
                .unwrap()
                .limit;
            let target_remaining = if remaining_delta < 0 {
                scratch_bytes - 1
            } else {
                scratch_bytes + remaining_delta as u64
            };
            let baseline = limit - target_remaining;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let result = read_object_record_document_path_with_work_and_cancellation(
                &path,
                &mut ledger,
                || false,
            );
            if remaining_delta < 0 {
                assert!(matches!(result, Err(StoreError::DecisionWorkAccounting(_))));
            } else {
                let mut document = result.unwrap();
                document.release(&mut ledger).unwrap();
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_prepared_batch_current_zero(&ledger);
        }
    }

    #[cfg(unix)]
    #[test]
    fn accounted_exact_path_read_rejects_same_size_path_replacement_and_releases_scratch() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-same-size-swap");
        let object_id = store_object_record_document(&objects, document).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let original = fs::read(&path).unwrap();
        let backup = path.with_extension("original");
        let mut ledger = object_transfer_ledger("accounted-same-size-swap");
        let mut polls = 0_u64;

        let error =
            read_object_record_document_path_with_work_and_cancellation(&path, &mut ledger, || {
                polls += 1;
                if polls == 2 {
                    fs::rename(&path, &backup).unwrap();
                    fs::write(&path, &original).unwrap();
                }
                false
            })
            .err()
            .expect("same-size path replacement must fail");

        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("path identity changed")
        ));
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn accounted_exact_path_read_rejects_opened_file_aba_replacement() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-opened-aba-a");
        let object_id = store_object_record_document(&objects, document).unwrap();
        let path = object_record_path(&objects, &object_id).unwrap();
        let original = fs::read(&path).unwrap();
        let mut replacement = original.clone();
        let marker = b"accounted-opened-aba-a";
        let replacement_marker = b"accounted-opened-aba-b";
        let offset = replacement
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();
        replacement[offset..offset + marker.len()].copy_from_slice(replacement_marker);
        let backup = path.with_extension("aba-original");
        let mut ledger = object_transfer_ledger("accounted-opened-file-aba");
        let mut polls = 0_u64;

        let error =
            read_object_record_document_path_with_work_and_cancellation(&path, &mut ledger, || {
                polls += 1;
                if polls == 3 {
                    fs::rename(&path, &backup).unwrap();
                    fs::write(&path, &replacement).unwrap();
                } else if polls == 4 {
                    fs::remove_file(&path).unwrap();
                    fs::rename(&backup, &path).unwrap();
                }
                false
            })
            .err()
            .expect("A-B-A opened-file replacement must fail");

        assert!(matches!(
            error,
            StoreError::IncompleteRecord(message)
                if message.contains("path identity changed")
        ));
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn accounted_object_id_resolver_routes_known_and_bounds_unknown_namespaces() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-id-resolver");
        let object_id = store_object_record_document(&objects, document.clone()).unwrap();
        for index in 0..32 {
            fs::create_dir_all(objects.join(format!("extension-{index:02}"))).unwrap();
        }
        let mut ledger = object_transfer_ledger("accounted-id-resolver");

        let mut accounted = read_object_record_document_with_work_and_cancellation(
            &objects,
            &object_id,
            &mut ledger,
            || false,
        )
        .unwrap();
        assert_eq!(accounted.document(), &document);
        accounted.release(&mut ledger).unwrap();
        let error = read_object_record_document_with_work_and_cancellation(
            &objects,
            "CF-FUTURE-000000000000000000000000",
            &mut ledger,
            || false,
        )
        .err()
        .expect("unknown missing object must be bounded and absent");
        assert!(matches!(error, StoreError::ObjectNotFound(_)));
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn accounted_object_store_reader_rejects_root_swap_at_closing_fence() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-reader-root-fence");
        let object_id = store_object_record_document(&objects, document.clone()).unwrap();
        let moved = temp.path().join("objects-before-swap");
        let mut ledger = object_transfer_ledger("accounted-reader-root-fence");
        let mut reader = AccountedObjectStoreReader::begin_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
        )
        .unwrap();

        let mut first = reader
            .read_with_work_and_cancellation(&object_id, &mut ledger, || false)
            .unwrap();
        assert_eq!(first.document(), &document);
        first.release(&mut ledger).unwrap();
        fs::rename(&objects, &moved).unwrap();
        fs::create_dir_all(&objects).unwrap();
        let mut second = reader
            .read_with_work_and_cancellation(&object_id, &mut ledger, || false)
            .unwrap();
        assert_eq!(second.document(), &document);
        second.release(&mut ledger).unwrap();

        let error = reader
            .finish_with_cancellation(&mut ledger, || false)
            .unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
        reader.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn accounted_object_store_reader_preflights_filename_before_allocation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-reader-name-preflight");
        store_object_record_document(&objects, document).unwrap();
        let object_id = format!("CF-FUTURE-{}", "a".repeat(200));
        let name_scratch_bytes =
            codefire_core::LOGICAL_FRAME_BASE_BYTES * 2 + (object_id.len() as u64) * 2 + 5;
        let mut ledger = object_transfer_ledger("accounted-reader-name-preflight");
        let mut reader = AccountedObjectStoreReader::begin_with_work_and_cancellation(
            &objects,
            &mut ledger,
            || false,
        )
        .unwrap();
        let limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let baseline = limit - name_scratch_bytes + 1;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ),
            )
            .unwrap();
        let graph_before = ledger
            .snapshot()
            .cumulative
            .get(&codefire_core::DecisionWorkDimension::GraphNodes)
            .copied()
            .unwrap_or_default();

        let error = reader
            .read_with_work_and_cancellation(&object_id, &mut ledger, || false)
            .err()
            .expect("filename allocation must be rejected by its preflight");
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::TemporaryBytes
                        && budget.observed == limit + 1
                })
        ));
        assert_eq!(
            ledger
                .snapshot()
                .cumulative
                .get(&codefire_core::DecisionWorkDimension::GraphNodes)
                .copied()
                .unwrap_or_default(),
            graph_before
        );
        reader.cancel(&mut ledger).unwrap();
        ledger
            .release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                baseline,
            )
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn accounted_object_id_resolver_preflights_root_path_owner() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let document = prepared_policy_document("accounted-resolver-root-bound");
        let object_id = store_object_record_document(&objects, document).unwrap();
        let root_bytes = object_store_scan_name_frame_bytes(&objects).unwrap();
        let limit = codefire_core::DecisionWorkProfile::local_v1()
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let baseline = limit - root_bytes + 1;
        let mut ledger = object_transfer_ledger("accounted-resolver-root-bound");
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                ),
            )
            .unwrap();

        let error = read_object_record_document_with_work_and_cancellation(
            &objects,
            &object_id,
            &mut ledger,
            || false,
        )
        .err()
        .expect("root owner limit+1 must fail before binding allocation");
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::RetainedBytes
                        && budget.observed == limit + 1
                })
        ));
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                baseline,
            )
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn store_object_rejects_unknown_type_for_writes() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");

        let error = store_object(&objects, "future", json!({"type": "future"})).unwrap_err();

        assert!(matches!(error, StoreError::UnknownObjectType(type_tag) if type_tag == "future"));
    }

    #[test]
    fn validate_sealed_commit_accepts_valid_commit_graph() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let parent_id = write_valid_commit(&objects, vec![]);
        let child_id = write_valid_commit(&objects, vec![parent_id]);

        validate_sealed_commit(&objects, &child_id).unwrap();
    }

    #[test]
    fn sealed_commit_validation_missing_objects_root_is_object_not_found() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("missing-objects");
        let commit_id = "CF-COMMIT-missing";
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache(&objects, commit_id, &mut cache).unwrap_err();

        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == commit_id));
        assert!(cache.validated_subjects.is_empty());
    }

    #[test]
    fn sealed_commit_validation_does_not_cache_failed_subject() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let parent_id = write_valid_commit(&objects, vec![]);
        let mut child_roots = write_required_roots(&objects);
        let child_manifest = store_object(
            &objects,
            "content_manifest",
            json!({
                "type": "content_manifest",
                "version": 1,
                "entries": [],
                "fixture": "failed-child"
            }),
        )
        .unwrap();
        child_roots.insert(
            "content_manifest".to_string(),
            Value::String(child_manifest),
        );
        let commit_id = store_object(
            &objects,
            "commit",
            commit_payload(
                vec![parent_id.clone()],
                child_roots,
                consistent_certificate(),
            ),
        )
        .unwrap();
        let commit = read_object(&objects, &commit_id).unwrap();
        let missing_root = commit["roots"]["content_manifest"]
            .as_str()
            .unwrap()
            .to_string();
        let missing_path = object_record_path(&objects, &missing_root).unwrap();
        let record_bytes = fs::read(&missing_path).unwrap();
        fs::remove_file(&missing_path).unwrap();
        let mut cache = SealedCommitValidationCache::default();

        for _ in 0..2 {
            let error =
                validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache).unwrap_err();
            assert!(matches!(error, StoreError::ObjectNotFound(id) if id == missing_root));
            assert!(cache_contains_commit(&cache, &parent_id));
            assert!(!cache_contains_commit(&cache, &commit_id));
        }

        fs::write(&missing_path, record_bytes).unwrap();
        validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache).unwrap();
        assert_eq!(cache.validated_subjects.len(), 2);
        assert!(cache_contains_commit(&cache, &commit_id));
    }

    #[test]
    fn sealed_commit_validation_cache_reuses_shared_dag_ancestors() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let base = write_valid_commit(&objects, vec![]);

        let mut left_payload = commit_payload(
            vec![base.clone()],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        left_payload["message"] = json!("left");
        let left = store_object(&objects, "commit", left_payload).unwrap();
        let mut right_payload = commit_payload(
            vec![base],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        right_payload["message"] = json!("right");
        let right = store_object(&objects, "commit", right_payload).unwrap();
        let merge = store_object(
            &objects,
            "commit",
            commit_payload(
                vec![left, right],
                write_required_roots(&objects),
                consistent_certificate(),
            ),
        )
        .unwrap();
        let mut cache = SealedCommitValidationCache::default();

        validate_sealed_commit_with_cache(&objects, &merge, &mut cache).unwrap();

        assert_eq!(cache.validated_subjects.len(), 4);
        assert_eq!(cache.cache_hits, 1);
        validate_sealed_commit_with_cache(&objects, &merge, &mut cache).unwrap();
        assert_eq!(cache.cache_hits, 2);
    }

    #[test]
    fn sealed_validation_session_detects_extension_only_same_metadata_rewrite() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let commit_path = object_record_path(&objects, &commit_id).unwrap();
        install_record_extension(&commit_path, "aaaa");

        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "subject-change-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let provisional_work = session.work();
        assert_eq!(provisional_work.semantic_commit_validations, 1);
        assert_eq!(provisional_work.revalidated_documents, 0);

        rewrite_marker_preserving_file_identity_and_times(&commit_path, b"aaaa", b"bbbb");
        let error = session.finalize().unwrap_err();

        assert!(matches!(
            error,
            StoreError::SealedValidationSubjectChanged {
                object_id,
                kind: SealedValidationSubjectChangeKind::DigestChanged,
                expected_size,
                actual_size: Some(actual_size),
                ..
            } if object_id == commit_id && expected_size == actual_size
        ));
        assert!(session.work().revalidated_documents > 0);
        assert!(session.work().revalidated_bytes > 0);
        assert_eq!(
            session.decision_work_snapshot().unwrap().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::SubjectChanged)
        );
    }

    #[test]
    fn sealed_validation_changed_subject_does_not_poison_reusable_cache() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let base = write_commit_fixture(&objects, vec![], &roots, &certificate, "base");
        let left = write_commit_fixture(&objects, vec![base.clone()], &roots, &certificate, "left");
        let right =
            write_commit_fixture(&objects, vec![base.clone()], &roots, &certificate, "right");
        let base_path = object_record_path(&objects, &base).unwrap();
        let original = install_record_extension(&base_path, "aaaa");
        let mut cache = SealedCommitValidationCache::default();
        validate_sealed_commit_with_cache(&objects, &left, &mut cache).unwrap();
        assert!(cache_contains_commit(&cache, &base));

        rewrite_marker_preserving_file_identity_and_times(&base_path, b"aaaa", b"bbbb");
        let error = validate_sealed_commit_with_cache(&objects, &right, &mut cache).unwrap_err();
        assert!(matches!(
            error,
            StoreError::SealedValidationSubjectChanged {
                object_id,
                kind: SealedValidationSubjectChangeKind::DigestChanged,
                ..
            } if object_id == base
        ));
        assert!(cache.validated_subjects.is_empty());
        assert!(cache.retained_documents.is_empty());

        fs::write(&base_path, original).unwrap();
        validate_sealed_commit_with_cache(&objects, &right, &mut cache).unwrap();
        assert!(cache_contains_commit(&cache, &base));
        assert!(cache_contains_commit(&cache, &right));
    }

    #[test]
    fn sealed_validation_session_reuses_many_head_shared_dag_and_revalidates_once() {
        const HEADS: usize = 32;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let base = write_commit_fixture(&objects, vec![], &roots, &certificate, "base");
        let heads = (0..HEADS)
            .map(|index| {
                write_commit_fixture(
                    &objects,
                    vec![base.clone()],
                    &roots,
                    &certificate,
                    &format!("head-{index}"),
                )
            })
            .collect::<Vec<_>>();

        let mut session = begin_sealed_commit_validation_session(&objects).unwrap();
        for head in &heads {
            session.validate_commit(head).unwrap();
        }
        let provisional = session.work();
        assert_eq!(provisional.semantic_commit_validations, (HEADS + 1) as u64);
        assert_eq!(provisional.validated_commit_hits, (HEADS - 1) as u64);
        assert_eq!(provisional.document_reads, (HEADS + 1 + 7) as u64);
        assert_eq!(provisional.revalidated_documents, 0);

        let finalized = session.finalize().unwrap();
        assert_eq!(finalized.revalidated_documents, finalized.document_reads);
        assert_eq!(finalized.revalidated_documents, (HEADS + 1 + 7) as u64);
        assert_eq!(finalized.revalidated_bytes, finalized.document_read_bytes);
    }

    #[test]
    fn sealed_validation_one_shot_work_reports_final_snapshot_io_on_cache_hits() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let mut cache = SealedCommitValidationCache::default();

        let cold =
            validate_sealed_commit_with_cache_and_work(&objects, &commit_id, &mut cache).unwrap();
        assert_eq!(cold.semantic_commit_validations, 1);
        assert_eq!(cold.document_reads, 8);
        assert_eq!(cold.revalidated_documents, cold.document_reads);
        assert_eq!(cold.revalidated_bytes, cold.document_read_bytes);

        let warm =
            validate_sealed_commit_with_cache_and_work(&objects, &commit_id, &mut cache).unwrap();
        assert_eq!(warm.semantic_commit_validations, 0);
        assert_eq!(warm.validated_commit_hits, 1);
        assert_eq!(warm.document_reads, 0);
        assert_eq!(warm.revalidated_documents, cold.document_reads);
        assert_eq!(warm.revalidated_bytes, cold.document_read_bytes);
        let cumulative = cache.work();
        assert_eq!(cumulative.semantic_commit_validations, 1);
        assert_eq!(cumulative.validated_commit_hits, 1);
        assert_eq!(cumulative.document_reads, cold.document_reads);
        assert_eq!(cumulative.document_read_bytes, cold.document_read_bytes);
        assert_eq!(
            cumulative.revalidated_documents,
            cold.revalidated_documents + warm.revalidated_documents
        );
        assert_eq!(
            cumulative.revalidated_bytes,
            cold.revalidated_bytes + warm.revalidated_bytes
        );
    }

    #[test]
    fn sealed_validation_session_charges_one_ledger_and_finishes_complete() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();

        session.validate_commit(&commit_id).unwrap();
        let provisional = session.decision_work_snapshot().unwrap();
        assert_eq!(
            provisional.cumulative[&codefire_core::DecisionWorkDimension::ObjectRecords],
            8
        );
        assert!(!provisional
            .cumulative
            .contains_key(&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords));
        assert_eq!(provisional.terminal_reason, None);
        assert_eq!(
            provisional
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(provisional
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .is_some_and(|bytes| *bytes > 0));
        assert!(provisional
            .peak
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .is_some_and(|bytes| *bytes > 0));

        let work = session.finalize().unwrap();
        assert_eq!(
            session.decision_work_snapshot().unwrap().terminal_reason,
            None
        );
        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        let finalized = ledger.snapshot();
        assert_eq!(
            finalized
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert_eq!(
            finalized.cumulative
                [&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords],
            work.revalidated_documents
        );
        assert_eq!(
            finalized.cumulative[&codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes],
            work.revalidated_bytes
        );
        assert_eq!(
            finalized.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::Complete)
        );
    }

    #[test]
    fn prepublication_validation_proof_performs_the_only_final_revalidation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-proof-success");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        assert_eq!(
            session.provisional_commit_payload(&commit_id).unwrap()["type"],
            "commit"
        );
        let roots = vec![commit_id.clone()];
        {
            let provisional = session.provisional_subject_for_roots(&roots).unwrap();
            assert_eq!(provisional.objects_root(), objects);
            assert_eq!(provisional.roots(), roots);
            assert_eq!(
                provisional.commit_payload(&commit_id).unwrap()["type"],
                "commit"
            );
            assert!(provisional.commit_payload("CF-COMMIT-unbound").is_err());
            let foreign = object_transfer_ledger("provisional-root-foreign");
            assert!(provisional.ensure_ledger(&foreign).is_err());
        }
        assert!(session.provisional_subject_for_roots(&[]).is_err());
        let invalid_roots = vec!["CF-COMMIT-unvalidated".to_string()];
        assert!(session
            .provisional_subject_for_roots(&invalid_roots)
            .is_err());
        let initial_document_reads = session.work().document_reads;
        assert_eq!(session.work().revalidated_documents, 0);
        let bound = session.bind_prepublication_validation(&roots).unwrap();
        assert_eq!(bound.roots(), roots);
        let (mut proof, mut ledger) = bound.into_prepublication_validation_proof().unwrap();
        assert_eq!(proof.bound_roots(), roots);
        assert_eq!(proof.objects_root(), objects);
        {
            let subject = proof.provisional_subject_for_planning(&ledger).unwrap();
            subject.ensure_ledger(&ledger).unwrap();
            assert_eq!(
                subject.commit_payload(&commit_id).unwrap()["type"],
                "commit"
            );
        }
        assert!(proof.validated_commit_payload(&commit_id).is_err());
        assert!(!ledger
            .snapshot()
            .cumulative
            .contains_key(&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords));

        ledger = proof.revalidate_preseal(ledger, || false).unwrap();

        assert_eq!(proof.work().revalidated_documents, initial_document_reads);
        assert_eq!(
            ledger.snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords],
            initial_document_reads
        );
        assert_eq!(
            proof.validated_commit_payload(&commit_id).unwrap()["type"],
            "commit"
        );
        ledger.seal_prepublication().unwrap();
        proof.finish_after_seal(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn provisional_subject_retains_validated_ancestor_without_cross_subject_cache_leak() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let commit_a = write_commit_fixture(
            &objects,
            vec![],
            &roots,
            &certificate,
            "publication ancestor A",
        );
        let commit_b = write_commit_fixture(
            &objects,
            vec![commit_a.clone()],
            &roots,
            &certificate,
            "publication ancestor B",
        );
        let commit_c = write_commit_fixture(
            &objects,
            vec![commit_b.clone()],
            &roots,
            &certificate,
            "publication root C",
        );
        let unrelated = write_commit_fixture(
            &objects,
            vec![],
            &roots,
            &certificate,
            "unrelated unvalidated commit",
        );
        let cross_subject = write_commit_fixture(
            &objects,
            vec![],
            &roots,
            &certificate,
            "cross-subject cached commit",
        );

        let foreign_objects = temp.path().join("foreign-objects");
        let foreign_roots = write_required_roots(&foreign_objects);
        let foreign_cross_subject = write_commit_fixture(
            &foreign_objects,
            vec![],
            &foreign_roots,
            &certificate,
            "cross-subject cached commit",
        );
        assert_eq!(foreign_cross_subject, cross_subject);
        let mut foreign_session = begin_sealed_commit_validation_session(&foreign_objects).unwrap();
        foreign_session
            .validate_commit(&foreign_cross_subject)
            .unwrap();
        foreign_session.finalize().unwrap();
        let foreign_subject = Arc::clone(&foreign_session.subject);
        assert!(foreign_session
            .cache
            .validated_subjects
            .contains(&(Arc::clone(&foreign_subject), cross_subject.clone())));

        let ledger = object_transfer_ledger("provisional-ancestor-closure");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_c).unwrap();
        session.validate_commit(&cross_subject).unwrap();

        let current_subject = Arc::clone(&session.subject);
        assert!(session
            .staged_subjects
            .remove(&(current_subject, cross_subject.clone())));
        assert!(session
            .cache
            .validated_subjects
            .insert((foreign_subject, cross_subject.clone())));

        let publication_roots = vec![commit_c.clone(), commit_a.clone()];
        {
            let subject = session
                .provisional_subject_for_roots(&publication_roots)
                .unwrap();
            assert_eq!(subject.roots(), publication_roots);
            assert!(subject.commit_payload(&commit_b).is_err());
            assert_eq!(
                subject.retained_commit_payload(&commit_b).unwrap()["message"],
                "publication ancestor B"
            );
            assert!(subject.retained_commit_payload(&unrelated).is_err());
            assert!(subject.retained_commit_payload(&cross_subject).is_err());
        }

        let bound = session
            .bind_prepublication_validation(&publication_roots)
            .unwrap();
        let (mut proof, mut ledger) = bound.into_prepublication_validation_proof().unwrap();
        {
            let subject = proof.provisional_subject_for_planning(&ledger).unwrap();
            assert_eq!(subject.roots(), publication_roots);
            assert_eq!(
                subject.retained_commit_payload(&commit_b).unwrap()["message"],
                "publication ancestor B"
            );
            assert!(subject.retained_commit_payload(&unrelated).is_err());
            assert!(subject.retained_commit_payload(&cross_subject).is_err());
        }

        proof.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_planning_subject_is_available_before_revalidation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let (commit_id, mut proof, mut ledger) =
            detached_prepublication_proof(&objects, "proof-planning-success");

        {
            let subject = proof.provisional_subject_for_planning(&ledger).unwrap();
            assert_eq!(subject.objects_root(), objects);
            assert_eq!(subject.roots(), std::slice::from_ref(&commit_id));
            assert_eq!(
                subject.commit_payload(&commit_id).unwrap()["type"],
                "commit"
            );
            subject.ensure_ledger(&ledger).unwrap();
        }

        ledger = proof.revalidate_preseal(ledger, || false).unwrap();
        proof.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_planning_subject_rejects_foreign_ledger_without_mutation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let (_, mut proof, mut owner) =
            detached_prepublication_proof(&objects, "proof-planning-owner");
        let mut foreign = object_transfer_ledger("proof-planning-foreign");
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let error = match proof.provisional_subject_for_planning(&foreign) {
            Ok(_) => panic!("foreign ledger unexpectedly received the planning subject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("different decision-work ledger")
        ));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);

        proof.cancel(&mut owner).unwrap();
        assert_prepared_batch_current_zero(&owner);
        owner
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_planning_subject_rejects_sealed_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let (_, mut proof, mut ledger) =
            detached_prepublication_proof(&objects, "proof-planning-sealed");
        ledger.seal_prepublication().unwrap();

        let error = match proof.provisional_subject_for_planning(&ledger) {
            Ok(_) => panic!("sealed ledger unexpectedly received the planning subject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("after prepublication sealing")
        ));

        proof.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_planning_subject_rejects_revalidated_proof() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let (_, mut proof, ledger) =
            detached_prepublication_proof(&objects, "proof-planning-revalidated");
        let mut ledger = proof.revalidate_preseal(ledger, || false).unwrap();

        let error = match proof.provisional_subject_for_planning(&ledger) {
            Ok(_) => panic!("revalidated proof unexpectedly retained its planning subject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("after proof revalidation")
        ));

        proof.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_planning_subject_rejects_inactive_proof() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let (_, mut proof, mut ledger) =
            detached_prepublication_proof(&objects, "proof-planning-inactive");
        proof.cancel(&mut ledger).unwrap();

        let error = match proof.provisional_subject_for_planning(&ledger) {
            Ok(_) => panic!("inactive proof unexpectedly retained its planning subject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("no longer usable")
        ));

        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_validation_proof_detects_document_drift_and_can_cancel() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let commit_path = object_record_path(&objects, &commit_id).unwrap();
        install_record_extension(&commit_path, "aaaa");
        let ledger = object_transfer_ledger("prepublication-proof-drift");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let binding = session
            .prepare_prepublication_validation_binding(std::slice::from_ref(&commit_id))
            .unwrap();
        let (mut proof, ledger) = session
            .into_prepublication_validation_proof_with_binding(binding)
            .unwrap();
        rewrite_marker_preserving_file_identity_and_times(&commit_path, b"aaaa", b"bbbb");

        let (error, mut ledger) = proof
            .revalidate_preseal(ledger, || false)
            .unwrap_err()
            .into_parts();

        assert!(matches!(
            error,
            StoreError::SealedValidationSubjectChanged {
                object_id,
                kind: SealedValidationSubjectChangeKind::DigestChanged,
                ..
            } if object_id == commit_id
        ));
        proof.cancel(&mut ledger).unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::SubjectChanged)
            .unwrap();
    }

    #[test]
    fn prepublication_validation_proof_rejects_foreign_ledger_without_mutation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-proof-owner");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let roots = vec![commit_id];
        let binding = session
            .prepare_prepublication_validation_binding(&roots)
            .unwrap();
        let (mut proof, mut owner) = session
            .into_prepublication_validation_proof_with_binding(binding)
            .unwrap();
        let foreign = object_transfer_ledger("prepublication-proof-foreign");
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let (error, mut foreign) = proof
            .revalidate_preseal(foreign, || false)
            .unwrap_err()
            .into_parts();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("different decision-work ledger")
        ));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert!(matches!(
            proof.cancel(&mut foreign),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("different decision-work ledger")
        ));
        assert!(matches!(
            proof.finish_after_seal(&mut foreign),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("different decision-work ledger")
        ));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);

        proof.cancel(&mut owner).unwrap();
        assert_prepared_batch_current_zero(&owner);
        owner
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
        foreign
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn prepublication_proof_detach_error_returns_the_intact_session_and_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-proof-recover-detach");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        session.finalize().unwrap();
        let before = session.decision_work_snapshot().unwrap().clone();

        let detach_error = match session.into_prepublication_validation_proof() {
            Err(error) => error,
            Ok((mut proof, mut ledger)) => {
                proof.cancel(&mut ledger).unwrap();
                panic!("finalized session unexpectedly detached")
            }
        };
        let (error, mut session) = detach_error.into_parts();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("successful unfinalized session")
        ));
        assert_eq!(session.decision_work_snapshot(), Some(&before));
        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn binding_detach_error_returns_finalized_session_and_binding_intact() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-binding-recover-finalized");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let roots = vec![commit_id];
        let mut binding = session
            .prepare_prepublication_validation_binding(&roots)
            .unwrap();
        session.finalize().unwrap();
        session
            .acquire_retained_decision_work(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                ),
                17,
            )
            .unwrap();

        let detach_error = match session.into_prepublication_validation_proof_with_binding(binding)
        {
            Err(error) => error,
            Ok((mut proof, mut ledger)) => {
                proof.cancel(&mut ledger).unwrap();
                panic!("finalized session unexpectedly detached")
            }
        };
        let (error, mut session, returned_binding) = detach_error.into_parts();
        binding = returned_binding;
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("successful unfinalized session")
        ));
        assert!(binding.active);
        assert_eq!(
            binding.retained_bytes,
            session.cache.prepublication_binding_bytes
        );

        session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap();
        session.release_retained_decision_work(17).unwrap();
        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        assert_prepared_batch_current_zero(&ledger);
        ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .unwrap();
    }

    #[test]
    fn binding_detach_error_returns_failed_session_and_binding_intact() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-binding-recover-invalid");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let roots = vec![commit_id];
        let binding = session
            .prepare_prepublication_validation_binding(&roots)
            .unwrap();
        assert!(session.validate_commit("CF-COMMIT-missing").is_err());

        let detach_error = match session.into_prepublication_validation_proof_with_binding(binding)
        {
            Err(error) => error,
            Ok((mut proof, mut ledger)) => {
                proof.cancel(&mut ledger).unwrap();
                panic!("failed session unexpectedly detached")
            }
        };
        let (error, mut session, mut binding) = detach_error.into_parts();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("successful unfinalized session")
        ));
        assert!(binding.active);
        session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap();
        let mut ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
        if ledger.snapshot().terminal_reason.is_none() {
            ledger
                .finish(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
                .unwrap();
        }
    }

    #[test]
    fn bound_prepublication_session_cancel_returns_finished_zero_current_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-cancel");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();

        let ledger = bound
            .cancel_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap();

        assert_prepared_batch_current_zero(&ledger);
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[test]
    fn bound_detach_failure_owner_can_cancel_and_recover_finished_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-detach-cancel");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let mut bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();
        bound.session.as_deref_mut().unwrap().finalize().unwrap();

        let detach_error = match bound.into_prepublication_validation_proof() {
            Err(error) => error,
            Ok((mut proof, mut ledger)) => {
                proof.cancel(&mut ledger).unwrap();
                panic!("finalized bound session unexpectedly detached")
            }
        };
        let (error, owner) = detach_error.into_parts();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("successful unfinalized session")
        ));
        let ledger = owner
            .cancel_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap();

        assert_prepared_batch_current_zero(&ledger);
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[test]
    fn prepublication_binding_cancel_is_idempotent() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-binding-double-cancel");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let mut binding = session
            .prepare_prepublication_validation_binding(std::slice::from_ref(&commit_id))
            .unwrap();

        session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap();
        let after_first = session.decision_work_snapshot().unwrap().clone();
        session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap();
        assert_eq!(session.decision_work_snapshot(), Some(&after_first));

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn prepublication_binding_cancel_preflights_scalar_underflow_before_release() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-binding-underflow-preflight");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let mut binding = session
            .prepare_prepublication_validation_binding(std::slice::from_ref(&commit_id))
            .unwrap();
        let retained_logical_bytes = session.cache.retained_logical_bytes;
        session.cache.retained_logical_bytes = binding.retained_bytes - 1;
        let before = session.decision_work_snapshot().unwrap().clone();

        let error = session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("retained accounting underflow")
        ));
        assert_eq!(session.decision_work_snapshot(), Some(&before));
        assert!(binding.active);
        assert!(binding.subject.is_some());
        assert!(!binding.roots.is_empty());
        session.cache.retained_logical_bytes = retained_logical_bytes;
        session
            .cancel_prepublication_validation_binding(&mut binding)
            .unwrap();
        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn bound_cancel_binding_release_failure_retains_scalar_tail_for_retry() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-binding-tail");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();
        let retained_bytes = bound.binding.as_ref().unwrap().retained_bytes;

        let cancellation_error = bound
            .cancel_and_take_decision_work_ledger_with_release(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                &mut |_, dimension, amount| {
                    assert_eq!(
                        dimension,
                        codefire_core::DecisionWorkDimension::RetainedBytes
                    );
                    assert_eq!(amount, retained_bytes);
                    Err(StoreError::Io(std::io::Error::other(
                        "injected binding release failure",
                    )))
                },
            )
            .unwrap_err();
        let (error, owner) = cancellation_error.into_parts();
        assert!(matches!(
            error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::Other
        ));
        let binding = owner.binding.as_ref().unwrap();
        assert!(binding.active);
        assert!(binding.subject.is_none());
        assert!(binding.roots.is_empty());
        assert_eq!(binding.retained_bytes, retained_bytes);
        assert_eq!(
            owner
                .session
                .as_ref()
                .unwrap()
                .cache
                .prepublication_binding_bytes,
            retained_bytes
        );

        let ledger = owner
            .cancel_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[test]
    fn bound_cancel_retained_failure_does_not_double_release_temporary_on_retry() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-dimension-tail");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let temporary_bytes = 19;
        session
            .cache
            .decision_work_ledger
            .as_mut()
            .unwrap()
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
            .unwrap();
        session.cache.temporary_scratch_bytes = temporary_bytes;
        let bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();
        let mut saw_temporary = false;
        let mut retained_releases = 0;

        let cancellation_error = bound
            .cancel_and_take_decision_work_ledger_with_release(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                &mut |ledger, dimension, amount| match dimension {
                    codefire_core::DecisionWorkDimension::TemporaryBytes => {
                        assert_eq!(amount, temporary_bytes);
                        assert!(!saw_temporary);
                        saw_temporary = true;
                        ledger.release(dimension, amount).map_err(Into::into)
                    }
                    codefire_core::DecisionWorkDimension::RetainedBytes => {
                        retained_releases += 1;
                        if retained_releases == 2 {
                            Err(StoreError::Io(std::io::Error::other(
                                "injected session retained release failure",
                            )))
                        } else {
                            ledger.release(dimension, amount).map_err(Into::into)
                        }
                    }
                    unexpected => panic!("unexpected release dimension: {unexpected:?}"),
                },
            )
            .unwrap_err();
        assert!(saw_temporary);
        assert_eq!(retained_releases, 2);
        let (error, owner) = cancellation_error.into_parts();
        assert!(matches!(
            error,
            StoreError::Io(error) if error.kind() == std::io::ErrorKind::Other
        ));
        let session = owner.session.as_ref().unwrap();
        assert_eq!(session.cache.temporary_scratch_bytes, 0);
        assert!(session.cache.retained_logical_bytes > 0);
        assert_eq!(
            session.decision_work_snapshot().unwrap().current
                [&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );

        let ledger = owner
            .cancel_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap();
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn dropping_bound_prepublication_session_cancels_internally_without_panicking() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-drop");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();

        let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(bound)));

        assert!(dropped.is_ok());
    }

    #[test]
    fn dropping_bound_prepublication_session_fails_closed_on_release_error() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-drop-fail-closed");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let mut bound = session
            .bind_prepublication_validation(std::slice::from_ref(&commit_id))
            .unwrap();
        let ledger = bound
            .session
            .as_deref_mut()
            .unwrap()
            .cache
            .decision_work_ledger
            .as_mut()
            .unwrap();
        let retained_current =
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_current,
            )
            .unwrap();

        let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(bound)));

        assert!(dropped.is_err());
    }

    #[test]
    fn dropping_bound_prepublication_build_error_cancels_internally() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("bound-prepublication-build-error-drop");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();

        let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let error = match session.bind_prepublication_validation(&[]) {
                Err(error) => error,
                Ok(owner) => {
                    drop(owner);
                    panic!("empty roots unexpectedly produced a bound session")
                }
            };
            drop(error);
        }));

        assert!(dropped.is_ok());
    }

    #[test]
    fn dropping_binding_detach_error_recovers_its_owned_session_without_panicking() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = object_transfer_ledger("prepublication-binding-drop-error");
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        let roots = vec![commit_id];
        let binding = session
            .prepare_prepublication_validation_binding(&roots)
            .unwrap();
        session.finalize().unwrap();

        let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let error = match session.into_prepublication_validation_proof_with_binding(binding) {
                Err(error) => error,
                Ok((mut proof, mut ledger)) => {
                    proof.cancel(&mut ledger).unwrap();
                    panic!("finalized session unexpectedly detached")
                }
            };
            drop(error);
        }));
        assert!(dropped.is_ok());
    }

    #[test]
    fn prepublication_binding_preflights_owned_root_clone() {
        for remaining_delta in [-1_i64, 0, 1] {
            let temp = tempdir().unwrap();
            let objects = temp.path().join("objects");
            let commit_id = write_valid_commit(&objects, vec![]);
            let ledger = object_transfer_ledger("prepublication-binding-clone-preflight");
            let mut session =
                begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
            session.validate_commit(&commit_id).unwrap();
            let roots = vec![commit_id];
            let binding_bytes = roots
                .iter()
                .try_fold(codefire_core::LOGICAL_FRAME_BASE_BYTES, |total, root| {
                    let root_bytes = codefire_core::logical_retained_frame_bytes(root)?;
                    total
                        .checked_add(root_bytes)
                        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
                })
                .unwrap();
            let ledger = session.cache.decision_work_ledger.as_mut().unwrap();
            let current =
                ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
            let limit = ledger
                .profile()
                .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
                .unwrap()
                .limit;
            let target_remaining = if remaining_delta < 0 {
                binding_bytes - 1
            } else {
                binding_bytes + remaining_delta as u64
            };
            let baseline = limit - current - target_remaining;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let clone_called = std::cell::Cell::new(false);
            let result =
                session.prepare_prepublication_validation_binding_with_clone(&roots, |roots| {
                    clone_called.set(true);
                    Ok(roots.to_vec())
                });
            if remaining_delta < 0 {
                assert!(matches!(result, Err(StoreError::DecisionWorkAccounting(_))));
                assert!(!clone_called.get());
            } else {
                let mut binding = result.unwrap();
                assert!(clone_called.get());
                session
                    .cancel_prepublication_validation_binding(&mut binding)
                    .unwrap();
            }
            session
                .cache
                .decision_work_ledger
                .as_mut()
                .unwrap()
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            let ledger = session
                .abort_and_take_decision_work_ledger(
                    codefire_core::DecisionWorkTerminalReason::Complete,
                )
                .unwrap()
                .unwrap();
            assert_prepared_batch_current_zero(&ledger);
        }
    }

    #[test]
    fn ledger_backed_session_bounds_non_directory_membership_entries() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();
        for name in ["noise-a", "noise-b", "noise-c"] {
            fs::write(objects.join(name), b"noise").unwrap();
        }
        let mut ledger = object_transfer_ledger("session-nondirectory-membership-bound");
        let graph_limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::GraphNodes)
            .unwrap()
            .limit;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(
                    codefire_core::DecisionWorkDimension::GraphNodes,
                    graph_limit - 2,
                ),
            )
            .unwrap();
        let mut polls = 0_u64;

        let error =
            begin_sealed_commit_validation_session_with_recoverable_ledger_and_cancellation(
                &objects,
                ledger,
                || {
                    polls += 1;
                    false
                },
            )
            .err()
            .expect("third non-directory member must exceed the membership budget");
        let (error, ledger) = error.into_parts();

        assert!(polls > 0);
        assert!(matches!(
            error,
            StoreError::DecisionWorkAccounting(error)
                if error.budget_exceeded().is_some_and(|budget| {
                    budget.dimension == codefire_core::DecisionWorkDimension::GraphNodes
                        && budget.observed == graph_limit + 1
                })
        ));
        assert_prepared_batch_current_zero(&ledger);
    }

    #[test]
    fn sealed_validation_abort_releases_memory_and_returns_same_terminal_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-abort-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        session.validate_commit(&commit_id).unwrap();

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
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
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn sealed_validation_cancellation_is_abort_only_and_releases_memory() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-cancellation-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();

        assert!(matches!(
            session.validate_commit_with_cancellation(&commit_id, || true),
            Err(StoreError::ValidationCancelled)
        ));
        assert!(matches!(
            session.finalize(),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));
        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::DeadlineExceeded)
        );
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0
            );
        }
    }

    #[test]
    fn sealed_validation_failure_is_abort_only_and_releases_failed_frames() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let valid = write_commit_fixture(&objects, vec![], &roots, &certificate, "valid");
        let mut invalid_roots = roots;
        invalid_roots.remove("policy");
        let invalid =
            write_commit_fixture(&objects, vec![], &invalid_roots, &certificate, "invalid");
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-failed-frame-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();

        assert!(matches!(
            session.validate_commit(&invalid),
            Err(StoreError::InvalidSealedCommit(message)) if message == "missing root policy"
        ));
        assert!(matches!(
            session.validate_commit(&valid),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));
        assert!(matches!(
            session.finalize(),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0
            );
        }
    }

    #[test]
    fn recoverable_sealed_validation_continues_after_a_corrupt_candidate() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let valid = write_commit_fixture(&objects, vec![], &roots, &certificate, "valid");
        let mut invalid_roots = roots;
        invalid_roots.remove("policy");
        let invalid =
            write_commit_fixture(&objects, vec![], &invalid_roots, &certificate, "invalid");
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-recoverable-candidate-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();

        assert!(matches!(
            session.validate_commit_recoverable(&invalid),
            Err(StoreError::InvalidSealedCommit(message)) if message == "missing root policy"
        ));
        assert!(!session.validation_failed);
        assert!(!session
            .staged_subjects
            .contains(&(Arc::clone(&session.subject), invalid.clone())));
        let retained_after_invalid = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        assert!(retained_after_invalid > 0);

        assert!(matches!(
            session.validate_commit_recoverable(&invalid),
            Err(StoreError::InvalidSealedCommit(message)) if message == "missing root policy"
        ));
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            retained_after_invalid,
            "a recoverable retry must not leak a traversal frame"
        );

        session.validate_commit_recoverable(&valid).unwrap();
        session.finalize().unwrap();
        assert_eq!(
            session.validated_commit_payload(&valid).unwrap()["message"],
            json!("valid")
        );

        let ledger = session.take_decision_work_ledger().unwrap().unwrap();
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0
            );
        }
    }

    #[test]
    fn recoverable_sealed_validation_cancellation_remains_abort_only() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-recoverable-cancellation-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();

        assert!(matches!(
            session.validate_commit_recoverable_with_cancellation(&commit_id, || true),
            Err(StoreError::ValidationCancelled)
        ));
        assert!(matches!(
            session.validate_commit_recoverable(&commit_id),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
            )
            .unwrap()
            .unwrap();
        for dimension in [
            codefire_core::DecisionWorkDimension::RetainedBytes,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
        ] {
            assert_eq!(
                ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default(),
                0
            );
        }
    }

    #[test]
    fn recoverable_validation_classifies_subject_change_as_terminal() {
        assert!(recoverable_validation_error_is_terminal(
            &StoreError::SealedValidationSubjectChanged {
                object_id: "CF-COMMIT-changed".to_string(),
                path: PathBuf::from("commits/CF-COMMIT-changed.json"),
                kind: SealedValidationSubjectChangeKind::DigestChanged,
                expected_size: 1,
                actual_size: Some(1),
            }
        ));
        assert!(!recoverable_validation_error_is_terminal(
            &StoreError::InvalidSealedCommit("candidate is corrupt".to_string())
        ));
    }

    #[test]
    fn accounted_recoverable_validation_handles_error_without_detaching_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let valid = write_commit_fixture(&objects, vec![], &roots, &certificate, "valid");
        let mut invalid_roots = roots;
        invalid_roots.remove("policy");
        let invalid =
            write_commit_fixture(&objects, vec![], &invalid_roots, &certificate, "invalid");
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-accounted-recoverable-handler-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        let diagnostic_bytes = 37;

        let valid_candidate = session
            .validate_commit_recoverable_with_cancellation_and_error_handler(
                &invalid,
                || false,
                |error, ledger| {
                    assert!(matches!(
                        error,
                        StoreError::InvalidSealedCommit(message) if message == "missing root policy"
                    ));
                    ledger.apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Validation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            diagnostic_bytes,
                        ),
                    )?;
                    Ok(())
                },
            )
            .unwrap();
        assert!(!valid_candidate);
        assert!(!session.validation_failed);

        assert!(session
            .validate_commit_recoverable_with_cancellation_and_error_handler(
                &valid,
                || false,
                |_, _| panic!("a valid candidate must not call its error handler"),
            )
            .unwrap());
        session.finalize().unwrap();
        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            diagnostic_bytes
        );
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                diagnostic_bytes,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn provisional_decision_work_retains_report_growth_across_valid_candidates() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let first = write_commit_fixture(&objects, vec![], &roots, &certificate, "first valid");
        let second = write_commit_fixture(
            &objects,
            vec![first.clone()],
            &roots,
            &certificate,
            "second valid",
        );
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-provisional-work-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        let report_bytes = 41;

        session.validate_commit_recoverable(&first).unwrap();
        let retained_before = session.decision_work_snapshot().unwrap().current
            [&codefire_core::DecisionWorkDimension::RetainedBytes];
        session
            .with_provisional_decision_work_ledger(|ledger| {
                ledger.apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        report_bytes,
                    ),
                )?;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            session.decision_work_snapshot().unwrap().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            retained_before + report_bytes
        );
        session.validate_commit_recoverable(&second).unwrap();
        session.finalize().unwrap();

        let mut ledger = session.take_decision_work_ledger().unwrap().unwrap();
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            report_bytes
        );
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                report_bytes,
            )
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn provisional_decision_work_callback_error_rolls_back_and_requires_abort() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-provisional-error-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        session.validate_commit_recoverable(&commit_id).unwrap();
        let retained_before = session.decision_work_snapshot().unwrap().current
            [&codefire_core::DecisionWorkDimension::RetainedBytes];
        let temporary_before = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();

        let error = session
            .with_provisional_decision_work_ledger(|ledger| {
                ledger.apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, 29)
                    .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 31),
                )?;
                Err(StoreError::ValidationCancelled)
            })
            .unwrap_err();
        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(
            session.decision_work_snapshot().unwrap().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            retained_before
        );
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            temporary_before
        );
        assert!(matches!(
            session.validate_commit_recoverable(&commit_id),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
            )
            .unwrap()
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn provisional_decision_work_temporary_imbalance_rolls_back_and_requires_abort() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-provisional-imbalance-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        session.validate_commit_recoverable(&commit_id).unwrap();
        let retained_before = session.decision_work_snapshot().unwrap().current
            [&codefire_core::DecisionWorkDimension::RetainedBytes];
        let temporary_before = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();

        let error = session
            .with_provisional_decision_work_ledger(|ledger| {
                ledger.apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, 37)
                    .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 43),
                )?;
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("changed ledger ownership")
        ));
        assert_eq!(
            session.decision_work_snapshot().unwrap().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            retained_before
        );
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            temporary_before
        );

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_recoverable_validation_handler_failure_is_abort_only() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let valid = write_commit_fixture(&objects, vec![], &roots, &certificate, "valid");
        let mut invalid_roots = roots;
        invalid_roots.remove("policy");
        let invalid =
            write_commit_fixture(&objects, vec![], &invalid_roots, &certificate, "invalid");
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-accounted-handler-failure-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();
        let diagnostic_bytes = 23;
        let scratch_bytes = 17;

        assert!(matches!(
            session.validate_commit_recoverable_with_cancellation_and_error_handler(
                &invalid,
                || false,
                |_, ledger| {
                    ledger.apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Validation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            diagnostic_bytes,
                        )
                        .acquire(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            scratch_bytes,
                        ),
                    )?;
                    Err(StoreError::ValidationCancelled)
                },
            ),
            Err(StoreError::ValidationCancelled)
        ));
        assert!(matches!(
            session.validate_commit_recoverable(&valid),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));

        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::DeadlineExceeded,
            )
            .unwrap()
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn accounted_recoverable_validation_rejects_unbalanced_handler_temporary() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let mut invalid_roots = roots;
        invalid_roots.remove("policy");
        let invalid =
            write_commit_fixture(&objects, vec![], &invalid_roots, &certificate, "invalid");
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-unbalanced-handler-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_recoverable_ledger(&objects, ledger)
                .unwrap();

        let error = session
            .validate_commit_recoverable_with_cancellation_and_error_handler(
                &invalid,
                || false,
                |_, ledger| {
                    ledger.apply(
                        codefire_core::DecisionWorkReservation::new(
                            codefire_core::DecisionWorkPhase::Validation,
                            codefire_core::DecisionWorkSurface::Atomic,
                        )
                        .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 1),
                    )?;
                    Ok(())
                },
            )
            .unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("changed ledger ownership")
        ));
        let ledger = session
            .abort_and_take_decision_work_ledger(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            )
            .unwrap()
            .unwrap();
        assert_scan_memory_released(&ledger);
    }

    #[test]
    fn sealed_validation_depth_counts_edges_from_root_zero() {
        fn validate_and_observe_depth(objects: &Path, commit_id: &str) -> u64 {
            let ledger = codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::local_v1(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "store-session-depth-test",
            )
            .unwrap();
            let mut session =
                begin_sealed_commit_validation_session_with_ledger(objects, ledger).unwrap();
            session.validate_commit(commit_id).unwrap();
            session
                .decision_work_snapshot()
                .unwrap()
                .maximum
                .get(&codefire_core::DecisionWorkDimension::LogicalDepth)
                .copied()
                .unwrap_or_default()
        }

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let root = write_valid_commit(&objects, vec![]);
        let child = write_valid_commit(&objects, vec![root.clone()]);

        assert_eq!(validate_and_observe_depth(&objects, &root), 0);
        assert_eq!(validate_and_observe_depth(&objects, &child), 1);
    }

    #[test]
    fn post_validation_concurrent_memory_must_use_matching_tracked_api() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-retained-api-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        session.finalize().unwrap();
        let before = session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        let caller_supplied_acquire = codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, 100);

        assert!(matches!(
            session.acquire_retained_decision_work(caller_supplied_acquire, 0),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must not contain caller-supplied concurrent memory")
        ));
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            before
        );
        assert!(matches!(
            session.release_retained_decision_work(1),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("exceeds acquired ownership")
        ));
        let tracked = codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            codefire_core::DecisionWorkSurface::Atomic,
        );
        session.acquire_retained_decision_work(tracked, 11).unwrap();
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(before + 11)
        );
        session.release_retained_decision_work(11).unwrap();
        assert_eq!(
            session
                .decision_work_snapshot()
                .unwrap()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(before)
        );
    }

    #[test]
    fn overlay_begin_failure_releases_only_its_owned_retained_bytes() {
        let temp = tempdir().unwrap();
        let source_objects = temp.path().join("source-objects");
        let commit_id = write_valid_commit(&source_objects, vec![]);
        let document = read_object_record_document_path(
            &object_record_path(&source_objects, &commit_id).unwrap(),
        )
        .unwrap();
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-overlay-owned-release-test",
        )
        .unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, 37),
            )
            .unwrap();
        let input = prepare_sealed_commit_overlay_validation_input(vec![document], ledger).unwrap();
        let not_a_directory = temp.path().join("not-a-directory");
        fs::write(&not_a_directory, b"file").unwrap();

        let error = match begin_sealed_commit_overlay_validation_session_with_recoverable_ledger(
            &not_a_directory,
            input,
        ) {
            Ok(_) => panic!("a regular file cannot be bound as an overlay object store"),
            Err(error) => error,
        };
        let (_, ledger) = error.discard_documents_and_recover_ledger();

        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied(),
            Some(37)
        );
    }

    #[test]
    fn overlay_preparation_and_begin_cancellation_recover_the_ledger() {
        let temp = tempdir().unwrap();
        let source_objects = temp.path().join("source-objects");
        let commit_id = write_valid_commit(&source_objects, vec![]);
        let document = read_object_record_document_path(
            &object_record_path(&source_objects, &commit_id).unwrap(),
        )
        .unwrap();
        let new_ledger = || {
            codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::local_v1(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "store-overlay-cancellation-test",
            )
            .unwrap()
        };

        let error = prepare_sealed_commit_overlay_validation_input_with_cancellation(
            vec![document.clone()],
            new_ledger(),
            || true,
        )
        .unwrap_err();
        let (error, ledger) = error.discard_documents_and_recover_ledger();
        assert!(matches!(error, StoreError::ValidationCancelled));
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
                .copied()
                .unwrap_or_default(),
            0
        );

        let input =
            prepare_sealed_commit_overlay_validation_input(vec![document], new_ledger()).unwrap();
        let error =
            match begin_sealed_commit_overlay_validation_session_with_recoverable_ledger_and_cancellation(
                &source_objects,
                input,
                || true,
            ) {
                Ok(_) => panic!("cancelled overlay begin must not return a session"),
                Err(error) => error,
            };
        let (error, ledger) = error.discard_documents_and_recover_ledger();
        assert!(matches!(error, StoreError::ValidationCancelled));
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
    fn sealed_validation_overlay_validates_without_layout_or_filesystem_revalidation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let mut documents = Vec::new();
        for directory in fs::read_dir(&objects).unwrap() {
            let directory = directory.unwrap();
            if !directory.file_type().unwrap().is_dir() {
                continue;
            }
            for entry in fs::read_dir(directory.path()).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                documents.push(read_object_record_document_path(&path).unwrap());
            }
        }
        fs::remove_dir_all(&objects).unwrap();
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-overlay-test",
        )
        .unwrap();
        let input = prepare_sealed_commit_overlay_validation_input(documents, ledger).unwrap();
        let mut session =
            begin_sealed_commit_overlay_validation_session_with_recoverable_ledger(&objects, input)
                .unwrap();

        session.validate_commit(&commit_id).unwrap();
        session.finalize().unwrap();
        let ledger = session.take_decision_work_ledger().unwrap().unwrap();

        assert!(!objects.exists());
        assert!(!ledger
            .snapshot()
            .cumulative
            .contains_key(&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords));
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
    fn overlay_finalize_cancellation_hides_partial_subject_promotion() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let parent_id = write_valid_commit(&objects, vec![]);
        let commit_id = write_valid_commit(&objects, vec![parent_id]);
        let mut documents = Vec::new();
        for directory in fs::read_dir(&objects).unwrap() {
            let directory = directory.unwrap();
            if !directory.file_type().unwrap().is_dir() {
                continue;
            }
            for entry in fs::read_dir(directory.path()).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|value| value.to_str()) == Some("json") {
                    documents.push(read_object_record_document_path(&path).unwrap());
                }
            }
        }
        fs::remove_dir_all(&objects).unwrap();
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-overlay-finalize-cancellation-test",
        )
        .unwrap();
        let input = prepare_sealed_commit_overlay_validation_input(documents, ledger).unwrap();
        let mut session =
            begin_sealed_commit_overlay_validation_session_with_recoverable_ledger(&objects, input)
                .unwrap();
        session.validate_commit(&commit_id).unwrap();
        let mut polls = 0_u64;

        assert!(matches!(
            session.finalize_with_cancellation(|| {
                polls += 1;
                polls >= 2
            }),
            Err(StoreError::ValidationCancelled)
        ));
        assert!(matches!(
            session.validated_commit_payload(&commit_id),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));
        let ledger = session.take_decision_work_ledger().unwrap().unwrap();
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
    fn sealed_validation_discard_after_failure_returns_unfinished_zero_memory_ledger() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);
        let ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "store-session-diagnostic-discard-test",
        )
        .unwrap();
        let mut session =
            begin_sealed_commit_validation_session_with_ledger(&objects, ledger).unwrap();
        session.validate_commit(&commit_id).unwrap();
        assert!(session
            .decision_work_snapshot()
            .unwrap()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .is_some_and(|bytes| *bytes > 0));
        assert!(matches!(
            session.validate_commit("CF-COMMIT-missing"),
            Err(StoreError::ObjectNotFound(_))
        ));

        let ledger = session
            .discard_and_take_decision_work_ledger()
            .unwrap()
            .unwrap();

        assert_scan_memory_released(&ledger);
        assert_eq!(ledger.snapshot().terminal_reason, None);
        assert!(matches!(
            session.validated_commit_payload(&commit_id),
            Err(StoreError::InvalidSealedCommit(message))
                if message.contains("must be aborted after a validation failure")
        ));
        assert!(session.discard_and_take_decision_work_ledger().is_err());
    }

    #[test]
    fn sealed_validation_snapshot_revalidation_budget_accepts_exact_and_stops_plus_one() {
        fn ledger_with_remaining_revalidations(
            remaining: u64,
        ) -> codefire_core::DecisionWorkLedger {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                codefire_core::DecisionWorkProfile::local_v1(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "snapshot-revalidation-boundary",
            )
            .unwrap();
            let limit = ledger
                .profile()
                .limit(codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords)
                .unwrap()
                .limit;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(
                        codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                        limit - remaining,
                    ),
                )
                .unwrap();
            ledger
        }

        let exact_temp = tempdir().unwrap();
        let exact_objects = exact_temp.path().join("objects");
        let exact_commit = write_valid_commit(&exact_objects, vec![]);
        let mut exact = begin_sealed_commit_validation_session_with_ledger(
            &exact_objects,
            ledger_with_remaining_revalidations(8),
        )
        .unwrap();
        exact.validate_commit(&exact_commit).unwrap();
        exact.finalize().unwrap();
        assert_eq!(
            exact.decision_work_snapshot().unwrap().cumulative
                [&codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords],
            100_000
        );

        let exceeded_temp = tempdir().unwrap();
        let exceeded_objects = exceeded_temp.path().join("objects");
        let exceeded_commit = write_valid_commit(&exceeded_objects, vec![]);
        let mut exceeded = begin_sealed_commit_validation_session_with_ledger(
            &exceeded_objects,
            ledger_with_remaining_revalidations(7),
        )
        .unwrap();
        exceeded.validate_commit(&exceeded_commit).unwrap();
        let error = exceeded.finalize().unwrap_err();
        let StoreError::DecisionWorkAccounting(accounting) = error else {
            panic!("expected typed snapshot revalidation budget error");
        };
        let exhaustion = accounting.budget_exceeded().unwrap();
        assert_eq!(
            exhaustion.dimension,
            codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords
        );
        assert_eq!(exhaustion.observed, 100_001);
        assert_eq!(exhaustion.limit, 100_000);
        assert_eq!(
            exceeded.decision_work_snapshot().unwrap().terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::BudgetExceeded)
        );
        assert_eq!(exceeded.work().revalidated_documents, 7);
    }

    #[test]
    fn iterative_sealed_validation_handles_deep_linear_history() {
        const DEPTH: usize = 20_000;

        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();
        let mut cache = SealedCommitValidationCache::default();
        let bound_store = bind_object_store(objects, &mut cache).unwrap();
        let object_store_subject = Arc::clone(&bound_store.subject);
        let mut staged_subjects = HashSet::new();
        let mut visiting = HashSet::new();
        let mut exit_order = Vec::with_capacity(DEPTH);
        let head = format!("CF-COMMIT-{:05}", DEPTH - 1);

        validate_sealed_commit_inner_with_handlers(
            &object_store_subject,
            &head,
            &mut cache,
            &mut staged_subjects,
            &mut visiting,
            |commit_id, _cache| {
                let index = commit_id
                    .strip_prefix("CF-COMMIT-")
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|index| *index < DEPTH)
                    .ok_or_else(|| StoreError::ObjectNotFound(commit_id.to_string()))?;
                let parents = if index == 0 {
                    Vec::new()
                } else {
                    vec![Value::String(format!("CF-COMMIT-{:05}", index - 1))]
                };
                Ok(json!({
                    "type": "commit",
                    "version": 1,
                    "parents": parents,
                    "sequence": index
                }))
            },
            |commit, _, _cache| {
                exit_order.push(commit["sequence"].as_u64().unwrap() as usize);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(staged_subjects.len(), DEPTH);
        assert!(visiting.is_empty());
        assert_eq!(cache.cache_hits, 0);
        assert_eq!(exit_order.len(), DEPTH);
        assert_eq!(exit_order.first(), Some(&0));
        assert_eq!(exit_order.last(), Some(&(DEPTH - 1)));
    }

    #[test]
    fn iterative_sealed_validation_stages_diamond_in_postorder() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let base = write_commit_fixture(&objects, vec![], &roots, &certificate, "base");
        let left = write_commit_fixture(&objects, vec![base.clone()], &roots, &certificate, "left");
        let right =
            write_commit_fixture(&objects, vec![base.clone()], &roots, &certificate, "right");
        let mut broken_roots = roots.clone();
        broken_roots.remove("policy");
        let merge = write_commit_fixture(
            &objects,
            vec![left.clone(), right.clone()],
            &broken_roots,
            &certificate,
            "broken-merge",
        );
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache(&objects, &merge, &mut cache).unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message) if message == "missing root policy"
        ));
        assert!(cache_contains_commit(&cache, &base));
        assert!(cache_contains_commit(&cache, &left));
        assert!(cache_contains_commit(&cache, &right));
        assert!(!cache_contains_commit(&cache, &merge));
        assert_eq!(cache.validated_subjects.len(), 3);
        assert_eq!(cache.cache_hits, 1);
    }

    #[test]
    fn iterative_sealed_validation_reuses_repeated_parent() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let base = write_commit_fixture(&objects, vec![], &roots, &certificate, "base");
        let head = write_commit_fixture(
            &objects,
            vec![base.clone(), base.clone(), base.clone()],
            &roots,
            &certificate,
            "repeated-parent",
        );
        let mut cache = SealedCommitValidationCache::default();

        validate_sealed_commit_with_cache(&objects, &head, &mut cache).unwrap();

        assert_eq!(cache.validated_subjects.len(), 2);
        assert_eq!(cache.cache_hits, 2);
    }

    #[test]
    fn iterative_sealed_validation_detects_cycle_before_later_parent_error() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();
        let mut cache = SealedCommitValidationCache::default();
        let bound_store = bind_object_store(objects, &mut cache).unwrap();
        let object_store_subject = Arc::clone(&bound_store.subject);
        let commits = BTreeMap::from([
            (
                "CF-COMMIT-A".to_string(),
                json!({
                    "type": "commit",
                    "version": 1,
                    "parents": ["CF-COMMIT-B", 7]
                }),
            ),
            (
                "CF-COMMIT-B".to_string(),
                json!({
                    "type": "commit",
                    "version": 1,
                    "parents": ["CF-COMMIT-A"]
                }),
            ),
        ]);
        let mut staged_subjects = HashSet::new();
        let mut visiting = HashSet::new();

        let error = validate_sealed_commit_inner_with_reader(
            &bound_store,
            &object_store_subject,
            "CF-COMMIT-A",
            &mut cache,
            &mut staged_subjects,
            &mut visiting,
            |commit_id, _cache| {
                commits
                    .get(commit_id)
                    .cloned()
                    .ok_or_else(|| StoreError::ObjectNotFound(commit_id.to_string()))
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message == "commit parent cycle detected at CF-COMMIT-A"
        ));
        assert!(staged_subjects.is_empty());
    }

    #[test]
    fn iterative_sealed_validation_preserves_first_parent_error() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_required_roots(&objects);
        let certificate = consistent_certificate();
        let mut corrupt_roots = roots.clone();
        corrupt_roots.remove("policy");
        let corrupt = write_commit_fixture(
            &objects,
            vec![],
            &corrupt_roots,
            &certificate,
            "corrupt-parent",
        );
        let missing = "CF-COMMIT-missing".to_string();
        let corrupt_first = write_commit_fixture(
            &objects,
            vec![corrupt.clone(), missing.clone()],
            &roots,
            &certificate,
            "corrupt-first",
        );
        let missing_first = write_commit_fixture(
            &objects,
            vec![missing.clone(), corrupt],
            &roots,
            &certificate,
            "missing-first",
        );

        let corrupt_error = validate_sealed_commit(&objects, &corrupt_first).unwrap_err();
        let missing_error = validate_sealed_commit(&objects, &missing_first).unwrap_err();

        assert!(matches!(
            corrupt_error,
            StoreError::InvalidSealedCommit(message) if message == "missing root policy"
        ));
        assert!(matches!(
            missing_error,
            StoreError::ObjectNotFound(commit_id) if commit_id == missing
        ));
    }

    #[test]
    fn sealed_commit_validation_cache_is_scoped_to_the_object_store() {
        let temp = tempdir().unwrap();
        let first_objects = temp.path().join("first-objects");
        let second_objects = temp.path().join("second-objects");
        let first_commit = write_valid_commit(&first_objects, vec![]);
        let second_commit = write_valid_commit(&second_objects, vec![]);
        assert_eq!(first_commit, second_commit);

        let second_payload = read_object(&second_objects, &second_commit).unwrap();
        let missing_root = second_payload["roots"]["content_manifest"]
            .as_str()
            .unwrap();
        fs::remove_file(object_record_path(&second_objects, missing_root).unwrap()).unwrap();

        let mut cache = SealedCommitValidationCache::default();
        validate_sealed_commit_with_cache(&first_objects, &first_commit, &mut cache).unwrap();
        let error = validate_sealed_commit_with_cache(&second_objects, &second_commit, &mut cache)
            .unwrap_err();
        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == missing_root));
        assert_eq!(cache.pinned_object_stores.len(), 1);
        assert!(!cache_contains_commit(&cache, &first_commit));
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn validated_sealed_commit_consumes_commit_and_roots_from_its_pinned_subject() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit_id = write_valid_commit(&objects, vec![]);

        let validated = validate_sealed_commit_pinned(&objects, &commit_id).unwrap();
        let commit = validated.read_commit().unwrap();
        let atom_index_id = commit["roots"]["atom_index"].as_str().unwrap();

        assert_eq!(validated.commit_id(), commit_id);
        assert_eq!(validated.objects_root(), objects);
        assert_eq!(commit["type"], "commit");
        assert_eq!(
            validated.read_root("atom_index").unwrap()["type"],
            "atom_index"
        );
        assert_eq!(
            validated.read_object(atom_index_id).unwrap(),
            validated.read_root("atom_index").unwrap()
        );
        validated.ensure_subject_identity().unwrap();
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn validated_sealed_commit_rejects_same_content_root_swap_after_validation() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let original = temp.path().join("objects-original");
        let commit_id = write_valid_commit(&objects, vec![]);
        let validated = validate_sealed_commit_pinned(&objects, &commit_id).unwrap();
        validated.ensure_subject_identity().unwrap();

        fs::rename(&objects, &original).unwrap();
        let replacement_commit = write_valid_commit(&objects, vec![]);
        assert_eq!(replacement_commit, commit_id);

        let error = validated
            .read_commit()
            .expect_err("an equal-content replacement root is not the validated subject");
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn validated_sealed_commit_rejects_same_content_root_subdir_swap() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let atom_indexes = objects.join("atom_indexes");
        let original_atom_indexes = objects.join("atom_indexes-original");
        let commit_id = write_valid_commit(&objects, vec![]);
        let validated = validate_sealed_commit_pinned(&objects, &commit_id).unwrap();

        fs::rename(&atom_indexes, &original_atom_indexes).unwrap();
        fs::create_dir(&atom_indexes).unwrap();
        for entry in fs::read_dir(&original_atom_indexes).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), atom_indexes.join(entry.file_name())).unwrap();
        }

        let error = validated
            .read_root("atom_index")
            .expect_err("an equal-content replacement root subdirectory must fail closed");
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
    }

    #[test]
    fn sealed_validation_preserves_bounded_extension_object_lookup() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload = json!({"type": "extension", "version": 1, "value": "kept"});
        let record = object_record("extension", payload.clone()).unwrap();
        let path = objects
            .join("extension_objects")
            .join(format!("{}.json", record.object_id));
        write_record_fixture(&path, &record);

        let mut cache = SealedCommitValidationCache::default();
        let bound = bind_object_store(fs::canonicalize(&objects).unwrap(), &mut cache).unwrap();

        assert_eq!(
            read_bound_object(&bound, &record.object_id).unwrap(),
            payload
        );
        ensure_bound_object_store_identity(&bound, &cache).unwrap();
        assert_eq!(cache.pinned_object_stores.len(), 1);
    }

    #[test]
    fn sealed_validation_rejects_unbounded_extension_object_subdirectories() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();
        for index in 0..=MAX_PINNED_EXTENSION_OBJECT_SUBDIRS {
            fs::create_dir(objects.join(format!("extension-{index:03}"))).unwrap();
        }
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache(&objects, "CF-COMMIT-missing", &mut cache)
            .expect_err("an unbounded extension namespace must fail before retaining handles");

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("extension subdirectory limit exceeded")
                    && message.contains(&MAX_PINNED_EXTENSION_OBJECT_SUBDIRS.to_string())
        ));
        assert!(cache.pinned_object_stores.is_empty());
        assert!(cache.validated_subjects.is_empty());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn sealed_commit_validation_cache_rejects_same_path_store_replacement() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let moved_objects = temp.path().join("objects-before-replacement");
        let commit_id = write_valid_commit(&objects, vec![]);
        let mut cache = SealedCommitValidationCache::default();
        validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache).unwrap();

        fs::rename(&objects, moved_objects).unwrap();
        fs::create_dir(&objects).unwrap();
        let error = validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache)
            .expect_err("a replacement directory must not reuse the old validation result");

        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == commit_id));
        assert!(cache.validated_subjects.is_empty());
        assert_eq!(cache.pinned_object_stores.len(), 1);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn sealed_commit_validation_does_not_cache_after_store_identity_changes() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let moved_objects = temp.path().join("objects-during-validation");
        let commit_id = write_valid_commit(&objects, vec![]);
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache_and_hook(
            &objects,
            &commit_id,
            &mut cache,
            |canonical_root| {
                fs::rename(canonical_root, &moved_objects)?;
                fs::create_dir(canonical_root)?;
                Ok(())
            },
        )
        .expect_err("a directory swap before the final identity check must fail closed");

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("object store identity changed")
        ));
        assert!(cache.validated_subjects.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn sealed_validation_reads_the_pinned_root_during_swap_back_aba() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let moved_objects = temp.path().join("objects-original");
        let alternate_objects = temp.path().join("objects-alternate");
        let commit_id = write_valid_commit(&objects, vec![]);
        let alternate_commit = write_valid_commit(&alternate_objects, vec![]);
        assert_eq!(alternate_commit, commit_id);
        let commit = read_object(&objects, &commit_id).unwrap();
        let missing_root = commit["roots"]["content_manifest"]
            .as_str()
            .unwrap()
            .to_string();
        fs::remove_file(object_record_path(&objects, &missing_root).unwrap()).unwrap();
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache_and_hooks(
            &objects,
            &commit_id,
            &mut cache,
            |_| {
                fs::rename(objects.as_path(), moved_objects.as_path())?;
                fs::rename(alternate_objects.as_path(), objects.as_path())?;
                Ok(())
            },
            |_| {
                fs::rename(objects.as_path(), alternate_objects.as_path())?;
                fs::rename(moved_objects.as_path(), objects.as_path())?;
                Ok(())
            },
        )
        .expect_err("a temporary alternate root must not satisfy the pinned subject");

        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == missing_root));
        assert!(!cache_contains_commit(&cache, &commit_id));
    }

    #[cfg(unix)]
    #[test]
    fn sealed_validation_reads_the_pinned_subdir_during_swap_back_aba() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let empty_commits = objects.join("commits");
        let valid_commits = objects.join("valid-commits-away");
        let moved_empty_commits = objects.join("empty-commits-away");
        let commit_id = write_valid_commit(&objects, vec![]);
        fs::rename(empty_commits.as_path(), valid_commits.as_path()).unwrap();
        fs::create_dir(empty_commits.as_path()).unwrap();
        let mut cache = SealedCommitValidationCache::default();

        let error = validate_sealed_commit_with_cache_and_hooks(
            &objects,
            &commit_id,
            &mut cache,
            |_| {
                fs::rename(empty_commits.as_path(), moved_empty_commits.as_path())?;
                fs::rename(valid_commits.as_path(), empty_commits.as_path())?;
                Ok(())
            },
            |_| {
                fs::rename(empty_commits.as_path(), valid_commits.as_path())?;
                fs::rename(moved_empty_commits.as_path(), empty_commits.as_path())?;
                Ok(())
            },
        )
        .expect_err("a temporary alternate commits directory must not satisfy the pinned subject");

        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == commit_id));
        assert!(!cache_contains_commit(&cache, &commit_id));
    }

    #[cfg(unix)]
    #[test]
    fn sealed_validation_cache_subject_includes_object_subdir_identity() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commits = objects.join("commits");
        let original_commits = objects.join("commits-before-replacement");
        let commit_id = write_valid_commit(&objects, vec![]);
        let mut cache = SealedCommitValidationCache::default();
        validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache).unwrap();

        fs::rename(commits.as_path(), original_commits.as_path()).unwrap();
        fs::create_dir(commits.as_path()).unwrap();
        let error = validate_sealed_commit_with_cache(&objects, &commit_id, &mut cache)
            .expect_err("a replacement commits directory must not reuse the old cache entry");

        assert!(matches!(error, StoreError::ObjectNotFound(id) if id == commit_id));
        assert!(cache.validated_subjects.is_empty());
        assert_eq!(cache.pinned_object_stores.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn sealed_commit_validation_rejects_a_symlinked_store_root() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let first_objects = temp.path().join("first-objects");
        let first_commit = write_valid_commit(&first_objects, vec![]);
        let current_objects = temp.path().join("current-objects");
        symlink(&first_objects, &current_objects).unwrap();
        let mut cache = SealedCommitValidationCache::default();
        let error = validate_sealed_commit_with_cache(&current_objects, &first_commit, &mut cache)
            .expect_err("a symlinked object-store root must not become a trusted subject");

        assert!(matches!(error, StoreError::Io(_)));
        assert!(cache.pinned_object_stores.is_empty());
        assert!(cache.validated_subjects.is_empty());
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_required_root() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        roots.remove("fire_delta");
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("missing root fire_delta"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_resolution_ledger_root() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        roots.remove("resolution_ledger");
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("missing root resolution_ledger"))
        );
    }

    #[test]
    fn validate_sealed_commit_accepts_legacy_missing_resolution_ledger_root() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        roots.remove("resolution_ledger");
        let mut commit = commit_payload(vec![], roots, consistent_certificate());
        commit
            .as_object_mut()
            .unwrap()
            .remove("root_schema_version");
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        validate_sealed_commit(&objects, &commit_id).unwrap();
    }

    #[test]
    fn validate_sealed_commit_rejects_wrong_root_type() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let wrong_id = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {}}),
        )
        .unwrap();
        roots.insert("verification".to_string(), Value::String(wrong_id));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("root verification has type policy"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_dangling_trace_graph_endpoint() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let roots = write_dangling_trace_roots(&objects);
        let commit_id = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots, consistent_certificate()),
        )
        .unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("trace_graph.links[0].to atom CODE-missing is missing")
        ));
    }

    #[test]
    fn validate_sealed_commit_accepts_legacy_unversioned_dangling_trace_graph() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut legacy_commit = commit_payload(
            vec![],
            write_dangling_trace_roots(&objects),
            consistent_certificate(),
        );
        legacy_commit
            .as_object_mut()
            .unwrap()
            .remove("semantic_validation_profile");
        let legacy_commit_id = store_object(&objects, "commit", legacy_commit).unwrap();

        validate_sealed_commit(&objects, &legacy_commit_id).unwrap();
    }

    #[test]
    fn validate_sealed_commit_accepts_current_child_with_legacy_unversioned_parent() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut legacy_parent = commit_payload(
            vec![],
            write_dangling_trace_roots(&objects),
            consistent_certificate(),
        );
        legacy_parent
            .as_object_mut()
            .unwrap()
            .remove("semantic_validation_profile");
        let legacy_parent_id = store_object(&objects, "commit", legacy_parent).unwrap();
        let current_child_id = write_valid_commit(&objects, vec![legacy_parent_id]);

        validate_sealed_commit(&objects, &current_child_id).unwrap();
    }

    #[test]
    fn validate_sealed_commit_rejects_unknown_or_non_string_semantic_profile() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");

        for (profile, expected_message) in [
            (
                json!("future-sealed-profile"),
                "unsupported commit.semantic_validation_profile",
            ),
            (
                json!({"profile": CURRENT_SEMANTIC_VALIDATION_PROFILE}),
                "commit.semantic_validation_profile must be a string",
            ),
        ] {
            let mut commit = commit_payload(
                vec![],
                write_required_roots(&objects),
                consistent_certificate(),
            );
            commit["semantic_validation_profile"] = profile;
            let commit_id = store_object(&objects, "commit", commit).unwrap();

            let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();
            assert!(matches!(
                error,
                StoreError::InvalidSealedCommit(message)
                    if message.contains(expected_message)
            ));
        }
    }

    #[test]
    fn legacy_unversioned_profile_does_not_relax_unrelated_sealed_invariants() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_dangling_trace_roots(&objects);
        roots.remove("fire_delta");
        let mut legacy_commit = commit_payload(vec![], roots, consistent_certificate());
        legacy_commit
            .as_object_mut()
            .unwrap()
            .remove("semantic_validation_profile");
        let legacy_commit_id = store_object(&objects, "commit", legacy_commit).unwrap();

        let error = validate_sealed_commit(&objects, &legacy_commit_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("missing root fire_delta")
        ));
    }

    #[test]
    fn sealed_trace_graph_endpoint_diagnostic_is_utf8_safe_and_byte_bounded() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let long_atom_id = format!("DES-{}{}", "x".repeat(250), "界".repeat(10_000));
        let (sample, omitted_bytes) = bounded_sealed_reference_diagnostic_sample(&long_atom_id);
        let sample = sample.to_string();
        assert!(omitted_bytes > 0);
        assert!(sample.len() <= SEALED_REFERENCE_DIAGNOSTIC_SAMPLE_BYTE_LIMIT);
        assert_eq!(omitted_bytes, long_atom_id.len() - sample.len());

        let atom_index = store_object(
            &objects,
            "atom_index",
            json!({
                "type": "atom_index",
                "version": 1,
                "atoms": [{
                    "atom_id": "REQ-present",
                    "kind": "requirement",
                    "artifact_path": "requirements.md",
                    "selector": {"type": "heading", "value": "REQ-present"},
                    "content_hash": "sha256:present"
                }, {
                    // Validation must resolve the original endpoint, not its bounded sample.
                    "atom_id": sample,
                    "kind": "design",
                    "artifact_path": "design.md",
                    "selector": {"type": "heading", "value": sample},
                    "content_hash": "sha256:sample-only"
                }],
                "duplicate_atom_ids": []
            }),
        )
        .unwrap();
        let trace_graph = store_object(
            &objects,
            "trace_graph",
            json!({
                "type": "trace_graph",
                "version": 1,
                "links": [{
                    "from": "REQ-present",
                    "to": long_atom_id,
                    "type": "refined_by",
                    "link_id": "LINK-long-dangling",
                    "link_hash": "sha256:long-dangling"
                }]
            }),
        )
        .unwrap();
        let mut roots = write_required_roots(&objects);
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        roots.insert("trace_graph".to_string(), Value::String(trace_graph));
        let commit_id = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots, consistent_certificate()),
        )
        .unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();
        let StoreError::InvalidSealedCommit(message) = error else {
            panic!("expected a sealed endpoint integrity error");
        };

        assert!(message.contains(&sample));
        assert!(message.contains(&format!("{omitted_bytes} byte(s) omitted")));
        assert!(!message.contains(&long_atom_id));
        assert!(message.len() < 1_024);
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_manifest_blob() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let manifest = store_object(
            &objects,
            "content_manifest",
            json!({
                "type": "content_manifest",
                "version": 1,
                "entries": [
                    {
                        "path": "docs/spec.md",
                        "kind": "file",
                        "mode": "100644",
                        "blob": "CF-BLOB-missing"
                    }
                ]
            }),
        )
        .unwrap();
        roots.insert("content_manifest".to_string(), Value::String(manifest));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("content_manifest.entries[0].blob invalid reference CF-BLOB-missing"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_resolution_evidence() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let resolution_ledger = store_object(
            &objects,
            "resolution_ledger",
            json!({
                "type": "resolution_ledger",
                "version": 1,
                "resolutions": [
                    {
                        "resolution_uid": "res_001",
                        "fire_uid": "fire_old",
                        "evidence_refs": ["CF-EVIDENCE-missing"]
                    }
                ]
            }),
        )
        .unwrap();
        roots.insert(
            "resolution_ledger".to_string(),
            Value::String(resolution_ledger),
        );
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("resolution_ledger.resolutions[0].evidence_refs[0] invalid reference CF-EVIDENCE-missing"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_evidence_artifact_ref() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let evidence = store_object(
            &objects,
            "evidence",
            json!({
                "type": "evidence",
                "version": 1,
                "artifact_ref": "CF-ARTIFACT-missing"
            }),
        )
        .unwrap();
        let resolution_ledger = store_object(
            &objects,
            "resolution_ledger",
            json!({
                "type": "resolution_ledger",
                "version": 1,
                "resolutions": [
                    {
                        "resolution_uid": "res_001",
                        "fire_uid": "fire_old",
                        "evidence_refs": [evidence]
                    }
                ]
            }),
        )
        .unwrap();
        roots.insert(
            "resolution_ledger".to_string(),
            Value::String(resolution_ledger),
        );
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("evidence") && message.contains("artifact_ref invalid reference CF-ARTIFACT-missing"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_malformed_root_payloads() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let atom_index = store_object(
            &objects,
            "atom_index",
            json!({"type": "atom_index", "version": 1, "atoms": "not-a-list", "duplicate_atom_ids": []}),
        )
        .unwrap();
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("atom_index.atoms must be a list"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_certificate_verification_mismatch() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let verification = store_object(
            &objects,
            "verification",
            json!({
                "type": "verification",
                "version": 1,
                "result": "failed",
                "open_required_fires": 0,
                "failed_checks": [{"id": "unit", "command": "false", "output": "failed"}],
                "missing_required_links": [],
                "stale_resolutions": [],
                "missing_evidence_refs": [],
                "duplicate_atom_ids": [],
                "verified_at": "2026-06-10T00:00:00Z"
            }),
        )
        .unwrap();
        roots.insert("verification".to_string(), Value::String(verification));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("certificate result consistent does not match verification result failed"))
        );
    }

    #[test]
    fn sealed_contract_binds_effective_policy_and_change_coverage_across_roots_and_certificate() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        fs::create_dir_all(&objects).unwrap();
        let mut roots = write_required_roots(&objects);
        let policy = codefire_core::default_verification_policy();
        let effective =
            codefire_core::effective_policy(&policy, &codefire_core::PolicyProvenance::default())
                .unwrap();
        let empty_index = codefire_core::AtomIndex {
            type_tag: "atom_index".to_string(),
            version: 1,
            hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
            atoms: Vec::new(),
            duplicate_atom_ids: Vec::new(),
        };
        let coverage =
            codefire_core::build_change_coverage(Vec::new(), &empty_index, &empty_index, &[], None)
                .unwrap();
        let verification = json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "trace_completeness_required": true,
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "missing_evidence_refs": [],
            "duplicate_atom_ids": [],
            "verified_at": "2026-07-19T00:00:00Z",
            "effective_policy": &effective,
            "change_coverage": &coverage,
        });
        roots.insert(
            "verification".to_string(),
            Value::String(store_object(&objects, "verification", verification).unwrap()),
        );
        roots.insert(
            "policy".to_string(),
            Value::String(
                store_object(
                    &objects,
                    "policy",
                    json!({
                        "type": "policy",
                        "version": 1,
                        "policy": &policy,
                        "effective_policy": &effective,
                    }),
                )
                .unwrap(),
            ),
        );
        let mut certificate = consistent_certificate();
        certificate.insert(
            "effective_policy_digest".to_string(),
            Value::String(effective.digest.clone()),
        );
        certificate.insert(
            "change_coverage_digest".to_string(),
            Value::String(coverage.digest.clone()),
        );
        let valid = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots.clone(), certificate.clone()),
        )
        .unwrap();
        validate_sealed_commit(&objects, &valid).unwrap();

        let mut wrong_certificate = certificate;
        wrong_certificate.insert(
            "change_coverage_digest".to_string(),
            Value::String("0".repeat(64)),
        );
        let wrong = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots.clone(), wrong_certificate),
        )
        .unwrap();
        let error = validate_sealed_commit(&objects, &wrong).unwrap_err();
        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("certificate change_coverage_digest does not match"))
        );

        roots.insert(
            "policy".to_string(),
            Value::String(
                store_object(
                    &objects,
                    "policy",
                    json!({"type": "policy", "version": 1, "policy": &policy}),
                )
                .unwrap(),
            ),
        );
        let missing_projection = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots, consistent_certificate()),
        )
        .unwrap();
        let error = validate_sealed_commit(&objects, &missing_projection).unwrap_err();
        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("effective_policy_projection_mismatch") || message.contains("effective_policy_digest is required"))
        );
    }

    #[test]
    fn validate_sealed_commit_binds_certificate_to_the_verification_subject() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let subject = assessment_subject();
        let mut roots = write_required_roots(&objects);
        let verification = store_object(
            &objects,
            "verification",
            json!({
                "type": "verification",
                "version": 1,
                "result": "passed",
                "open_required_fires": 0,
                "failed_checks": [],
                "missing_required_links": [],
                "stale_resolutions": [],
                "missing_evidence_refs": [],
                "duplicate_atom_ids": [],
                "verified_at": "2026-07-12T00:00:00Z",
                "subject": subject.clone(),
            }),
        )
        .unwrap();
        roots.insert("verification".to_string(), Value::String(verification));

        let mut certificate = consistent_certificate();
        certificate.insert("subject".to_string(), subject.clone());
        let valid = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots.clone(), certificate),
        )
        .unwrap();
        validate_sealed_commit(&objects, &valid).unwrap();

        let mut mismatched_certificate = consistent_certificate();
        let mut mismatched_subject = subject.clone();
        mismatched_subject["trust_kind"] = json!("untrusted");
        mismatched_certificate.insert("subject".to_string(), mismatched_subject);
        let mismatch = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots.clone(), mismatched_certificate),
        )
        .unwrap();
        let error = validate_sealed_commit(&objects, &mismatch).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("certificate subject does not match verification subject")
        ));

        let mut tampered_subject = subject;
        tampered_subject["branch"] = json!("tampered");
        let tampered_verification = store_object(
            &objects,
            "verification",
            json!({
                "type": "verification",
                "version": 1,
                "result": "passed",
                "open_required_fires": 0,
                "failed_checks": [],
                "missing_required_links": [],
                "stale_resolutions": [],
                "missing_evidence_refs": [],
                "duplicate_atom_ids": [],
                "verified_at": "2026-07-12T00:00:00Z",
                "subject": tampered_subject.clone(),
            }),
        )
        .unwrap();
        roots.insert(
            "verification".to_string(),
            Value::String(tampered_verification),
        );
        let mut tampered_certificate = consistent_certificate();
        tampered_certificate.insert("subject".to_string(), tampered_subject);
        let tampered = store_object(
            &objects,
            "commit",
            commit_payload(vec![], roots, tampered_certificate),
        )
        .unwrap();
        let error = validate_sealed_commit(&objects, &tampered).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("subject digest does not match its identity fields")
        ));
    }

    #[test]
    fn validate_assessment_subject_enforces_the_versioned_variant_contract() {
        let valid = assessment_subject();
        validate_assessment_subject(&valid).unwrap();

        let mut missing_worktree = valid.clone();
        missing_worktree
            .as_object_mut()
            .unwrap()
            .remove("worktree_digest");
        let error = validate_assessment_subject(&missing_worktree).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("fields do not match live_open trust semantics")
        ));

        let mut extension = valid;
        extension["unbound_claim"] = json!("trusted");
        let error = validate_assessment_subject(&extension).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("unsupported field unbound_claim")
        ));
    }

    #[test]
    fn validate_sealed_commit_rejects_unresolved_merge_conflict_verification() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let verification = store_object(
            &objects,
            "verification",
            json!({
                "type": "verification",
                "version": 1,
                "result": "passed",
                "open_required_fires": 0,
                "failed_checks": [],
                "missing_required_links": [],
                "stale_resolutions": [],
                "missing_evidence_refs": [],
                "unresolved_merge_conflicts": [
                    {"kind": "text", "subject": "README.md"}
                ],
                "duplicate_atom_ids": [],
                "verified_at": "2026-07-12T00:00:00Z"
            }),
        )
        .unwrap();
        roots.insert("verification".to_string(), Value::String(verification));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("verification_result_mismatch"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_unknown_commit_root_and_payload_versions() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");

        let mut unknown_commit = commit_payload(
            vec![],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        unknown_commit["version"] = json!(999);
        let unknown_commit_id = store_object(&objects, "commit", unknown_commit).unwrap();
        let error = validate_sealed_commit(&objects, &unknown_commit_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("unsupported commit.version 999")
        ));

        let mut unknown_root_schema = commit_payload(
            vec![],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        unknown_root_schema["root_schema_version"] = json!(999);
        let unknown_root_schema_id = store_object(&objects, "commit", unknown_root_schema).unwrap();
        let error = validate_sealed_commit(&objects, &unknown_root_schema_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("unsupported commit.root_schema_version 999")
        ));

        let mut roots = write_required_roots(&objects);
        let future_policy = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 999, "policy": {}}),
        )
        .unwrap();
        roots.insert("policy".to_string(), Value::String(future_policy));
        let future_root = commit_payload(vec![], roots, consistent_certificate());
        let future_root_id = store_object(&objects, "commit", future_root).unwrap();
        let error = validate_sealed_commit(&objects, &future_root_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("unsupported policy.version 999")
        ));
    }

    #[test]
    fn validate_sealed_commit_requires_complete_current_verification_and_certificate_facts() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");

        let mut roots = write_required_roots(&objects);
        let verification_id = roots["verification"].as_str().unwrap();
        let mut verification = read_object(&objects, verification_id).unwrap();
        verification.as_object_mut().unwrap().remove("result");
        let incomplete_verification = store_object(&objects, "verification", verification).unwrap();
        roots.insert(
            "verification".to_string(),
            Value::String(incomplete_verification),
        );
        let incomplete_commit = commit_payload(vec![], roots, consistent_certificate());
        let incomplete_commit_id = store_object(&objects, "commit", incomplete_commit).unwrap();
        let error = validate_sealed_commit(&objects, &incomplete_commit_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("verification.result is required")
        ));

        let mut certificate = consistent_certificate();
        certificate.remove("failed_checks");
        let missing_certificate_fact =
            commit_payload(vec![], write_required_roots(&objects), certificate);
        let missing_certificate_fact_id =
            store_object(&objects, "commit", missing_certificate_fact).unwrap();
        let error = validate_sealed_commit(&objects, &missing_certificate_fact_id).unwrap_err();
        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("certificate failed_checks count is required")
        ));
    }

    #[test]
    fn validate_sealed_commit_rejects_cross_root_fire_policy_verification_inconsistency() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let fire_ledger = store_object(
            &objects,
            "fire_ledger",
            json!({
                "type": "fire_ledger",
                "version": 1,
                "fires": [{
                    "type": "fire",
                    "version": 1,
                    "fire_uid": "fire_semantic_counterexample",
                    "display_id": "FIRE-counterexample",
                    "status": "open",
                    "severity": "required",
                    "source": {"atom_id": "REQ-counterexample"},
                    "target": {"atom_id": "DES-counterexample"},
                    "reason": "semantic counterexample",
                    "trace_path": [],
                    "created_by": "test",
                    "created_at": "2026-07-12T00:00:00Z",
                    "key": "semantic-counterexample"
                }]
            }),
        )
        .unwrap();
        roots.insert("fire_delta".to_string(), Value::String(fire_ledger));
        let mut unsafe_policy = codefire_core::default_verification_policy();
        unsafe_policy.require_no_required_fires = false;
        let policy = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": unsafe_policy}),
        )
        .unwrap();
        roots.insert("policy".to_string(), Value::String(policy));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(matches!(
            error,
            StoreError::InvalidSealedCommit(message)
                if message.contains("open_required_fire_count_mismatch")
                    && message.contains("verification_result_mismatch")
        ));
    }

    #[test]
    fn validate_sealed_commit_reads_clean_historical_false_safety_policy() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let mut historical_policy = codefire_core::default_verification_policy();
        historical_policy.require_no_required_fires = false;
        historical_policy.require_no_stale_resolutions = false;
        historical_policy.require_verification_success = false;
        historical_policy.reject_duplicate_atom_ids = false;
        let policy = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": historical_policy}),
        )
        .unwrap();
        roots.insert("policy".to_string(), Value::String(policy));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        validate_sealed_commit(&objects, &commit_id).unwrap();
    }

    #[test]
    fn current_required_commit_roots_have_payload_validators() {
        for root in CURRENT_REQUIRED_COMMIT_ROOTS {
            assert!(
                expected_root_type(root).is_some(),
                "{root} must have an expected type"
            );
            let malformed = malformed_payload_for_root(root);
            assert!(
                validate_root_payload(root, &malformed, 2).is_err(),
                "{root} validator must reject incomplete payloads"
            );
        }
    }

    fn malformed_payload_for_root(root: &str) -> Value {
        match root {
            "content_manifest" => json!({"type": "content_manifest", "version": 1}),
            "atom_index" => {
                json!({"type": "atom_index", "version": 1, "atoms": "bad", "duplicate_atom_ids": []})
            }
            "trace_graph" => json!({"type": "trace_graph", "version": 1, "links": "bad"}),
            "fire_delta" => json!({"type": "fire_ledger", "version": 1, "fires": "bad"}),
            "resolution_ledger" => {
                json!({"type": "resolution_ledger", "version": 1, "resolutions": "bad"})
            }
            "verification" => json!({"type": "verification", "version": 1, "result": "maybe"}),
            "policy" => json!({"type": "policy", "version": 1}),
            _ => json!({"type": expected_root_type(root).unwrap_or("unknown"), "version": 1}),
        }
    }

    #[test]
    fn validate_sealed_commit_rejects_invalid_parent_id() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit = commit_payload(
            vec!["CF-BLOB-not-a-parent".to_string()],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("parent 0 must be a commit object id"))
        );
    }

    fn write_valid_commit(objects: &Path, parents: Vec<String>) -> String {
        store_object(
            objects,
            "commit",
            commit_payload(
                parents,
                write_required_roots(objects),
                consistent_certificate(),
            ),
        )
        .unwrap()
    }

    fn write_commit_fixture(
        objects: &Path,
        parents: Vec<String>,
        roots: &Map<String, Value>,
        certificate: &Map<String, Value>,
        message: &str,
    ) -> String {
        let mut payload = commit_payload(parents, roots.clone(), certificate.clone());
        payload["message"] = Value::String(message.to_string());
        store_object(objects, "commit", payload).unwrap()
    }

    fn write_dangling_trace_roots(objects: &Path) -> Map<String, Value> {
        let atom_index = store_object(
            objects,
            "atom_index",
            json!({
                "type": "atom_index",
                "version": 1,
                "atoms": [{
                    "atom_id": "CODE-present",
                    "kind": "code",
                    "artifact_path": "src/lib.rs",
                    "selector": {"type": "explicit_cf_atom", "value": "CODE-present"},
                    "content_hash": "sha256:present"
                }],
                "duplicate_atom_ids": []
            }),
        )
        .unwrap();
        let trace_graph = store_object(
            objects,
            "trace_graph",
            json!({
                "type": "trace_graph",
                "version": 1,
                "links": [{
                    "from": "CODE-present",
                    "to": "CODE-missing",
                    "type": "related_to",
                    "link_id": "LINK-dangling",
                    "link_hash": "sha256:dangling"
                }]
            }),
        )
        .unwrap();
        let mut roots = write_required_roots(objects);
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        roots.insert("trace_graph".to_string(), Value::String(trace_graph));
        roots
    }

    fn write_required_roots(objects: &Path) -> Map<String, Value> {
        let content_manifest = store_object(
            objects,
            "content_manifest",
            json!({"type": "content_manifest", "version": 1, "entries": []}),
        )
        .unwrap();
        let atom_index = store_object(
            objects,
            "atom_index",
            json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
        )
        .unwrap();
        let trace_graph = store_object(
            objects,
            "trace_graph",
            json!({"type": "trace_graph", "version": 1, "links": []}),
        )
        .unwrap();
        let fire_delta = store_object(
            objects,
            "fire_ledger",
            json!({"type": "fire_ledger", "version": 1, "fires": []}),
        )
        .unwrap();
        let resolution_ledger = store_object(
            objects,
            "resolution_ledger",
            json!({"type": "resolution_ledger", "version": 1, "resolutions": []}),
        )
        .unwrap();
        let verification = store_object(
            objects,
            "verification",
            json!({
                "type": "verification",
                "version": 1,
                "result": "passed",
                "open_required_fires": 0,
                "failed_checks": [],
                "missing_required_links": [],
                "stale_resolutions": [],
                "missing_evidence_refs": [],
                "duplicate_atom_ids": [],
                "verified_at": "2026-06-10T00:00:00Z"
            }),
        )
        .unwrap();
        let policy = store_object(
            objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {}}),
        )
        .unwrap();

        let mut roots = Map::new();
        roots.insert(
            "content_manifest".to_string(),
            Value::String(content_manifest),
        );
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        roots.insert("trace_graph".to_string(), Value::String(trace_graph));
        roots.insert("fire_delta".to_string(), Value::String(fire_delta));
        roots.insert(
            "resolution_ledger".to_string(),
            Value::String(resolution_ledger),
        );
        roots.insert("verification".to_string(), Value::String(verification));
        roots.insert("policy".to_string(), Value::String(policy));
        roots
    }

    fn consistent_certificate() -> Map<String, Value> {
        let mut certificate = Map::new();
        certificate.insert(
            "result".to_string(),
            Value::String("consistent".to_string()),
        );
        certificate.insert("open_required_fires".to_string(), Value::Number(0.into()));
        certificate.insert("failed_checks".to_string(), Value::Number(0.into()));
        certificate.insert(
            "missing_required_links".to_string(),
            Value::Number(0.into()),
        );
        certificate.insert("stale_resolutions".to_string(), Value::Number(0.into()));
        certificate.insert("missing_evidence_refs".to_string(), Value::Number(0.into()));
        certificate.insert("duplicate_atom_ids".to_string(), Value::Number(0.into()));
        certificate
    }

    fn assessment_subject() -> Value {
        let content_digest = format!("sha256:{}", "0".repeat(64));
        let worktree_digest = format!("sha256:{}", "1".repeat(64));
        let identity = json!({
            "type": "codefire_assessment_subject",
            "version": 1,
            "kind": "live_open",
            "trust_kind": "local_unisolated",
            "content_digest": &content_digest,
            "worktree_digest": &worktree_digest,
            "immutable": false,
            "repository_id": "repo-fixture",
            "branch": "main",
            "open_instance_id": "open-fixture",
            "base_commit": "CF-COMMIT-base",
            "source_commit": null,
        });
        let subject_digest = format!(
            "sha256:{}",
            codefire_util::sha256_hex(&canonical_json(&identity).unwrap())
        );
        json!({
            "type": "codefire_assessment_subject",
            "version": 1,
            "kind": "live_open",
            "trust_kind": "local_unisolated",
            "subject_digest": subject_digest,
            "content_digest": content_digest,
            "worktree_digest": worktree_digest,
            "immutable": false,
            "repository_id": "repo-fixture",
            "branch": "main",
            "open_instance_id": "open-fixture",
            "base_commit": "CF-COMMIT-base",
        })
    }
}
