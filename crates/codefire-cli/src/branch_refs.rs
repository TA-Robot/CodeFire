use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde_json::Value;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::secure_worktree::{open_absolute_dir_nofollow, StableFsIdentity};

pub(crate) const DIRECTORY_ITERATION_SCRATCH_BYTES: u64 = 4096;
const BRANCH_REF_PARSE_SCRATCH_MULTIPLIER: u64 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct BranchRefWork {
    pub(crate) records: u64,
    pub(crate) bytes: u64,
    pub(crate) revalidated_records: u64,
    pub(crate) revalidated_bytes: u64,
}

#[derive(Debug)]
pub(crate) struct BranchRefDocument {
    file_name: OsString,
    display_path: PathBuf,
    size_bytes: u64,
    digest: String,
    value: Value,
}

impl BranchRefDocument {
    pub(crate) fn value(&self) -> &Value {
        &self.value
    }

    pub(crate) fn head(&self) -> &str {
        self.value["head"]
            .as_str()
            .expect("captured branch head is a non-empty string")
    }

    pub(crate) fn name(&self) -> &str {
        self.value["name"]
            .as_str()
            .expect("captured branch name is a string")
    }
}

#[derive(Debug)]
pub(crate) struct BranchRefSnapshot {
    display_root: PathBuf,
    directory: Dir,
    directory_identity: StableFsIdentity,
    membership: Vec<OsString>,
    documents: Vec<BranchRefDocument>,
    heads: BTreeSet<String>,
    retained_bytes: u64,
    work: BranchRefWork,
}

#[derive(Debug)]
pub(crate) struct OpenedRegistryDocument {
    file_name: OsString,
    display_path: PathBuf,
    size_bytes: u64,
    digest: String,
    value: Value,
}

impl OpenedRegistryDocument {
    pub(crate) fn value(&self) -> &Value {
        &self.value
    }

    pub(crate) fn display_path(&self) -> &Path {
        &self.display_path
    }
}

#[derive(Debug)]
pub(crate) struct OpenedRegistrySnapshot {
    display_parent: PathBuf,
    display_root: PathBuf,
    parent: Dir,
    parent_identity: StableFsIdentity,
    directory: Option<Dir>,
    directory_identity: Option<StableFsIdentity>,
    membership: Vec<OsString>,
    documents: Vec<OpenedRegistryDocument>,
    retained_bytes: u64,
    work: BranchRefWork,
}

#[derive(Debug)]
pub(crate) struct RepositoryMetadataSnapshot {
    display_root: PathBuf,
    directory: Dir,
    identity: StableFsIdentity,
}

impl RepositoryMetadataSnapshot {
    pub(crate) fn capture(display_root: &Path) -> Result<Self, BranchRefSnapshotError> {
        let directory = open_absolute_dir_nofollow(display_root)?;
        let identity = StableFsIdentity::for_directory(&directory)?;
        ensure_ambient_directory_identity(display_root, identity)?;
        Ok(Self {
            display_root: display_root.to_path_buf(),
            directory,
            identity,
        })
    }

    pub(crate) fn ensure_current(&self) -> Result<(), BranchRefSnapshotError> {
        self.identity
            .ensure_directory(&self.directory, "repository metadata directory")
            .map_err(|error| changed(&self.display_root, error.to_string()))?;
        ensure_ambient_directory_identity(&self.display_root, self.identity)
    }

    pub(crate) fn identity(&self) -> StableFsIdentity {
        self.identity
    }

    pub(crate) fn capture_branch_refs(
        &self,
        display_root: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<BranchRefSnapshot, BranchRefSnapshotError> {
        self.capture_branch_refs_with_hook_inner(display_root, record_limit, ledger, || {})
    }

    #[cfg(test)]
    pub(crate) fn capture_branch_refs_with_hook<F>(
        &self,
        display_root: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        after_capture: F,
    ) -> Result<BranchRefSnapshot, BranchRefSnapshotError>
    where
        F: FnOnce(),
    {
        self.capture_branch_refs_with_hook_inner(display_root, record_limit, ledger, after_capture)
    }

    fn capture_branch_refs_with_hook_inner<F>(
        &self,
        display_root: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        after_capture: F,
    ) -> Result<BranchRefSnapshot, BranchRefSnapshotError>
    where
        F: FnOnce(),
    {
        self.ensure_current()?;
        if display_root.parent() != Some(self.display_root.as_path()) {
            return Err(BranchRefSnapshotError::InvalidRecord {
                path: display_root.to_path_buf(),
                reason: "branch reference root is outside the pinned repository metadata root"
                    .to_string(),
            });
        }
        let slot_name =
            display_root
                .file_name()
                .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
                    path: display_root.to_path_buf(),
                    reason: "branch reference root has no slot name".to_string(),
                })?;
        let directory = self.directory.open_dir_nofollow(slot_name)?;
        let snapshot = BranchRefSnapshot::capture_bound_directory(
            display_root,
            directory,
            record_limit,
            ledger,
        )?;
        after_capture();
        if let Err(error) = self.ensure_current() {
            // Preserve the subject-change diagnosis even if accounting cleanup
            // itself reports a secondary failure, matching snapshot revalidation.
            let _ = snapshot.release(ledger);
            return Err(error);
        }
        Ok(snapshot)
    }
}

#[derive(Debug)]
pub(crate) enum BranchRefSnapshotError {
    Io(io::Error),
    Json(serde_json::Error),
    Accounting(codefire_core::DecisionWorkAccountingError),
    SubjectChanged { subject: PathBuf, reason: String },
    InvalidRecord { path: PathBuf, reason: String },
}

impl fmt::Display for BranchRefSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "branch reference I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "branch reference JSON is invalid: {error}"),
            Self::Accounting(error) => {
                write!(formatter, "branch reference accounting failed: {error}")
            }
            Self::SubjectChanged { subject, reason } => write!(
                formatter,
                "branch reference snapshot changed at {}: {reason}",
                subject.display()
            ),
            Self::InvalidRecord { path, reason } => write!(
                formatter,
                "invalid branch reference at {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for BranchRefSnapshotError {}

impl From<io::Error> for BranchRefSnapshotError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for BranchRefSnapshotError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<codefire_core::DecisionWorkAccountingError> for BranchRefSnapshotError {
    fn from(error: codefire_core::DecisionWorkAccountingError) -> Self {
        Self::Accounting(error)
    }
}

impl BranchRefSnapshot {
    #[cfg(test)]
    pub(crate) fn capture(
        display_root: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, BranchRefSnapshotError> {
        let directory = open_absolute_dir_nofollow(display_root)?;
        Self::capture_bound_directory(display_root, directory, record_limit, ledger)
    }

    fn capture_bound_directory(
        display_root: &Path,
        directory: Dir,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, BranchRefSnapshotError> {
        let mut retained_bytes = 0_u64;
        let result = Self::capture_inner(
            display_root,
            directory,
            record_limit,
            ledger,
            &mut retained_bytes,
        );
        match result {
            Ok(snapshot) => Ok(snapshot),
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )?;
                Err(error)
            }
        }
    }

    fn capture_inner(
        display_root: &Path,
        directory: Dir,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        retained_bytes: &mut u64,
    ) -> Result<Self, BranchRefSnapshotError> {
        let directory_identity = StableFsIdentity::for_directory(&directory)?;
        let membership =
            directory_membership_initial(&directory, display_root, ledger, retained_bytes)?;
        let mut documents = Vec::new();
        let mut heads = BTreeSet::new();
        let mut work = BranchRefWork::default();

        for file_name in membership.iter().filter(|name| is_json_ref(name)) {
            let display_path = display_root.join(file_name);
            let initial =
                read_initial_document(&directory, file_name, &display_path, record_limit, ledger)?;
            *retained_bytes = checked_add(*retained_bytes, initial.retained_bytes, &display_path)?;
            work.records = checked_add(work.records, 1, &display_path)?;
            work.bytes = checked_add(work.bytes, initial.size_bytes, &display_path)?;
            let head = initial
                .value
                .get("head")
                .and_then(Value::as_str)
                .filter(|head| !head.is_empty())
                .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
                    path: display_path.clone(),
                    reason: "head must be a non-empty string".to_string(),
                })?;
            initial
                .value
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
                    path: display_path.clone(),
                    reason: "name must be a string".to_string(),
                })?;
            if !heads.contains(head) {
                let head_frame = codefire_core::logical_retained_frame_bytes(head)?;
                let next_retained = checked_add(*retained_bytes, head_frame, &display_path)?;
                ledger.apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        head_frame,
                    ),
                )?;
                *retained_bytes = next_retained;
                heads.insert(head.to_string());
            }
            documents.push(BranchRefDocument {
                file_name: file_name.clone(),
                display_path,
                size_bytes: initial.size_bytes,
                digest: initial.digest,
                value: initial.value,
            });
        }
        ensure_ambient_directory_identity(display_root, directory_identity)?;
        Ok(Self {
            display_root: display_root.to_path_buf(),
            directory,
            directory_identity,
            membership,
            documents,
            heads,
            retained_bytes: *retained_bytes,
            work,
        })
    }

    pub(crate) fn documents(&self) -> &[BranchRefDocument] {
        &self.documents
    }

    pub(crate) fn heads(&self) -> &BTreeSet<String> {
        &self.heads
    }

    pub(crate) fn revalidate(
        mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<BranchRefWork, BranchRefSnapshotError> {
        let result = self.revalidate_inner(record_limit, ledger);
        let release = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        );
        match (result, release) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error.into()),
            (Ok(()), Ok(())) => Ok(self.work),
        }
    }

    pub(crate) fn release(
        self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), BranchRefSnapshotError> {
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        )?;
        Ok(())
    }

    fn revalidate_inner(
        &mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), BranchRefSnapshotError> {
        ensure_ambient_directory_identity(&self.display_root, self.directory_identity)?;
        for document in &self.documents {
            let size_bytes = read_revalidation_document(
                &self.directory,
                &document.file_name,
                &document.display_path,
                record_limit,
                document.size_bytes,
                &document.digest,
                ledger,
            )?;
            self.work.revalidated_records =
                checked_add(self.work.revalidated_records, 1, &document.display_path)?;
            self.work.revalidated_bytes = checked_add(
                self.work.revalidated_bytes,
                size_bytes,
                &document.display_path,
            )?;
        }
        let (current_membership, temporary_membership_bytes) =
            directory_membership_revalidation(&self.directory, &self.display_root, ledger)?;
        let membership_matches = current_membership == self.membership;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_membership_bytes,
        )?;
        if !membership_matches {
            return Err(changed(&self.display_root, "directory membership changed"));
        }
        ensure_ambient_directory_identity(&self.display_root, self.directory_identity)?;
        Ok(())
    }
}

impl OpenedRegistrySnapshot {
    pub(crate) fn capture(
        display_root: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, BranchRefSnapshotError> {
        let display_parent =
            display_root
                .parent()
                .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
                    path: display_root.to_path_buf(),
                    reason: "opened registry root has no parent".to_string(),
                })?;
        let slot_name =
            display_root
                .file_name()
                .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
                    path: display_root.to_path_buf(),
                    reason: "opened registry root has no slot name".to_string(),
                })?;
        let parent = open_absolute_dir_nofollow(display_parent)?;
        let parent_identity = StableFsIdentity::for_directory(&parent)?;
        let directory = match parent.open_dir_nofollow(slot_name) {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let Some(directory) = directory else {
            ensure_ambient_directory_identity(display_parent, parent_identity)?;
            return Ok(Self {
                display_parent: display_parent.to_path_buf(),
                display_root: display_root.to_path_buf(),
                parent,
                parent_identity,
                directory: None,
                directory_identity: None,
                membership: Vec::new(),
                documents: Vec::new(),
                retained_bytes: 0,
                work: BranchRefWork::default(),
            });
        };
        let directory_identity = StableFsIdentity::for_directory(&directory)?;
        let mut retained_bytes = 0_u64;
        let result = (|| {
            let membership = directory_membership_initial(
                &directory,
                display_root,
                ledger,
                &mut retained_bytes,
            )?;
            let mut documents = Vec::new();
            let mut work = BranchRefWork::default();
            for file_name in membership.iter().filter(|name| is_json_ref(name)) {
                let display_path = display_root.join(file_name);
                let initial = read_initial_document(
                    &directory,
                    file_name,
                    &display_path,
                    record_limit,
                    ledger,
                )?;
                retained_bytes =
                    checked_add(retained_bytes, initial.retained_bytes, &display_path)?;
                work.records = checked_add(work.records, 1, &display_path)?;
                work.bytes = checked_add(work.bytes, initial.size_bytes, &display_path)?;
                documents.push(OpenedRegistryDocument {
                    file_name: file_name.clone(),
                    display_path,
                    size_bytes: initial.size_bytes,
                    digest: initial.digest,
                    value: initial.value,
                });
            }
            ensure_opened_slot_identity(
                &parent,
                slot_name,
                display_root,
                Some(directory_identity),
            )?;
            ensure_ambient_directory_identity(display_parent, parent_identity)?;
            Ok(Self {
                display_parent: display_parent.to_path_buf(),
                display_root: display_root.to_path_buf(),
                parent,
                parent_identity,
                directory: Some(directory),
                directory_identity: Some(directory_identity),
                membership,
                documents,
                retained_bytes,
                work,
            })
        })();
        if result.is_err() {
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )?;
        }
        result
    }

    pub(crate) fn documents(&self) -> &[OpenedRegistryDocument] {
        &self.documents
    }

    pub(crate) fn parent_identity(&self) -> StableFsIdentity {
        self.parent_identity
    }

    pub(crate) fn revalidate(
        mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<BranchRefWork, BranchRefSnapshotError> {
        let result = self.revalidate_inner(record_limit, ledger, || Ok(()));
        let release = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        );
        match (result, release) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error.into()),
            (Ok(()), Ok(())) => Ok(self.work),
        }
    }

    #[cfg(test)]
    pub(crate) fn revalidate_with_hook<F>(
        mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_closing_fence: F,
    ) -> Result<BranchRefWork, BranchRefSnapshotError>
    where
        F: FnOnce() -> Result<(), BranchRefSnapshotError>,
    {
        let result = self.revalidate_inner(record_limit, ledger, before_closing_fence);
        let release = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        );
        match (result, release) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error.into()),
            (Ok(()), Ok(())) => Ok(self.work),
        }
    }

    pub(crate) fn release(
        self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), BranchRefSnapshotError> {
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            self.retained_bytes,
        )?;
        Ok(())
    }

    fn revalidate_inner<F>(
        &mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_closing_fence: F,
    ) -> Result<(), BranchRefSnapshotError>
    where
        F: FnOnce() -> Result<(), BranchRefSnapshotError>,
    {
        ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)?;
        let slot_name = self
            .display_root
            .file_name()
            .expect("captured opened registry root has a slot name");
        ensure_opened_slot_identity(
            &self.parent,
            slot_name,
            &self.display_root,
            self.directory_identity,
        )?;
        let Some(directory) = self.directory.as_ref() else {
            before_closing_fence()?;
            ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)?;
            ensure_opened_slot_identity(
                &self.parent,
                slot_name,
                &self.display_root,
                self.directory_identity,
            )?;
            return Ok(());
        };
        for document in &self.documents {
            let size_bytes = read_revalidation_document(
                directory,
                &document.file_name,
                &document.display_path,
                record_limit,
                document.size_bytes,
                &document.digest,
                ledger,
            )?;
            self.work.revalidated_records =
                checked_add(self.work.revalidated_records, 1, &document.display_path)?;
            self.work.revalidated_bytes = checked_add(
                self.work.revalidated_bytes,
                size_bytes,
                &document.display_path,
            )?;
        }
        let (current_membership, temporary_membership_bytes) =
            directory_membership_revalidation(directory, &self.display_root, ledger)?;
        let membership_matches = current_membership == self.membership;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_membership_bytes,
        )?;
        if !membership_matches {
            return Err(changed(
                &self.display_root,
                "opened registry directory membership changed",
            ));
        }
        before_closing_fence()?;
        ensure_opened_slot_identity(
            &self.parent,
            slot_name,
            &self.display_root,
            self.directory_identity,
        )?;
        ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)
    }
}

fn ensure_opened_slot_identity(
    parent: &Dir,
    slot_name: &OsStr,
    display_root: &Path,
    expected: Option<StableFsIdentity>,
) -> Result<(), BranchRefSnapshotError> {
    match (expected, parent.open_dir_nofollow(slot_name)) {
        (None, Err(error)) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        (None, Ok(_)) => Err(changed(display_root, "opened registry appeared")),
        (None, Err(error)) => Err(changed(display_root, error.to_string())),
        (Some(_), Err(error)) => Err(changed(display_root, error.to_string())),
        (Some(expected), Ok(directory)) => expected
            .ensure_directory(&directory, "opened registry directory")
            .map_err(|error| changed(display_root, error.to_string())),
    }
}

struct InitialDocumentRead {
    size_bytes: u64,
    retained_bytes: u64,
    digest: String,
    value: Value,
}

fn read_initial_document(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    limit: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<InitialDocumentRead, BranchRefSnapshotError> {
    let (mut file, size_bytes, identity) =
        open_regular_file(directory, file_name, display_path, limit)?;
    let temporary_bytes = size_bytes
        .checked_mul(BRANCH_REF_PARSE_SCRATCH_MULTIPLIER)
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(
            codefire_core::DecisionWorkDimension::AggregateBytes,
            size_bytes,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
        .count(
            codefire_core::DecisionWorkCounter::FileBytesRead,
            size_bytes,
        )
        .count(codefire_core::DecisionWorkCounter::BytesHashed, size_bytes),
    )?;
    let result: Result<InitialDocumentRead, BranchRefSnapshotError> = (|| {
        let bytes = read_exact_sized(&mut file, size_bytes, limit, display_path)?;
        identity.ensure_file(&file, "branch reference")?;
        let digest = codefire_util::sha256_hex(&bytes);
        let value: Value = serde_json::from_slice(&bytes)?;
        let retained_bytes = compact_json_size(&value)?;
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
        Ok(InitialDocumentRead {
            size_bytes,
            retained_bytes,
            digest,
            value,
        })
    })();
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary_bytes,
    )?;
    result
}

fn read_revalidation_document(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    limit: u64,
    expected_size: u64,
    expected_digest: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<u64, BranchRefSnapshotError> {
    let (mut file, size_bytes, identity) =
        open_regular_file(directory, file_name, display_path, limit)
            .map_err(|error| changed(display_path, error.to_string()))?;
    ledger.apply(
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
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
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
        .count(codefire_core::DecisionWorkCounter::BytesHashed, size_bytes),
    )?;
    let read = read_exact_sized(&mut file, size_bytes, limit, display_path)
        .and_then(|bytes| {
            identity.ensure_file(&file, "branch reference revalidation")?;
            if size_bytes != expected_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("size changed from {expected_size} to {size_bytes}"),
                ));
            }
            if codefire_util::sha256_hex(&bytes) != expected_digest {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "content digest changed",
                ));
            }
            Ok(())
        })
        .map_err(|error| changed(display_path, error.to_string()));
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        size_bytes,
    )?;
    read.map(|()| size_bytes)
}

fn open_regular_file(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    limit: u64,
) -> Result<(cap_std::fs::File, u64, StableFsIdentity), BranchRefSnapshotError> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let file = directory.open_with(file_name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(BranchRefSnapshotError::InvalidRecord {
            path: display_path.to_path_buf(),
            reason: format!(
                "expected a regular file no larger than {limit} bytes, got {} bytes",
                metadata.len()
            ),
        });
    }
    let identity = StableFsIdentity::for_file(&file)?;
    Ok((file, metadata.len(), identity))
}

fn read_exact_sized(
    file: &mut cap_std::fs::File,
    expected_size: u64,
    limit: u64,
    display_path: &Path,
) -> io::Result<Vec<u8>> {
    let capacity = usize::try_from(expected_size)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "file size does not fit usize"))?;
    let mut bytes = Vec::with_capacity(capacity.saturating_add(1));
    file.take(limit + 1).read_to_end(&mut bytes)?;
    let actual_size = u64::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "file size does not fit u64"))?;
    if actual_size != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "branch reference size changed while reading {}: expected {expected_size}, got {actual_size}",
                display_path.display()
            ),
        ));
    }
    Ok(bytes)
}

fn directory_membership_initial(
    directory: &Dir,
    display_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
) -> Result<Vec<OsString>, BranchRefSnapshotError> {
    let mut entries = directory.entries()?;
    let mut names = Vec::new();
    loop {
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            ),
        )?;
        let next = match entries.next().transpose() {
            Ok(next) => next,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error.into());
            }
        };
        let Some(entry) = next else {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            break;
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
            .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        let name = entry.file_name();
        let frame_bytes = match directory_member_frame_bytes(&name, display_root) {
            Ok(frame_bytes) => frame_bytes,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error);
            }
        };
        let next_retained = match checked_add(*retained_bytes, frame_bytes, display_root) {
            Ok(next) => next,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                frame_bytes,
            ),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        *retained_bytes = next_retained;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            DIRECTORY_ITERATION_SCRATCH_BYTES,
        )?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn directory_membership_revalidation(
    directory: &Dir,
    display_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<OsString>, u64), BranchRefSnapshotError> {
    let mut temporary_bytes = 0_u64;
    let result = directory_membership_revalidation_inner(
        directory,
        display_root,
        ledger,
        &mut temporary_bytes,
    );
    match result {
        Ok(names) => Ok((names, temporary_bytes)),
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            Err(error)
        }
    }
}

fn directory_membership_revalidation_inner(
    directory: &Dir,
    display_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    temporary_bytes: &mut u64,
) -> Result<Vec<OsString>, BranchRefSnapshotError> {
    let mut entries = directory
        .entries()
        .map_err(|error| changed(display_root, error.to_string()))?;
    let mut names = Vec::new();
    loop {
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            ),
        )?;
        let next = match entries.next().transpose() {
            Ok(next) => next,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(changed(display_root, error.to_string()));
            }
        };
        let Some(entry) = next else {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            break;
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                1,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
                1,
            ),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        let name = entry.file_name();
        let frame_bytes = match directory_member_frame_bytes(&name, display_root) {
            Ok(frame_bytes) => frame_bytes,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error);
            }
        };
        let next_temporary = match checked_add(*temporary_bytes, frame_bytes, display_root) {
            Ok(next) => next,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                frame_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                frame_bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
                frame_bytes,
            ),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        *temporary_bytes = next_temporary;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            DIRECTORY_ITERATION_SCRATCH_BYTES,
        )?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn directory_member_frame_bytes(
    name: &OsStr,
    display_root: &Path,
) -> Result<u64, BranchRefSnapshotError> {
    let name_bytes = u64::try_from(name.as_encoded_bytes().len()).map_err(|_| {
        BranchRefSnapshotError::InvalidRecord {
            path: display_root.to_path_buf(),
            reason: "directory member name length overflow".to_string(),
        }
    })?;
    codefire_core::LOGICAL_FRAME_BASE_BYTES
        .checked_add(name_bytes)
        .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
            path: display_root.to_path_buf(),
            reason: "directory member logical frame overflow".to_string(),
        })
}

#[derive(Default)]
struct CountingWriter(u64);

impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| io::Error::other("canonical JSON size overflow"))?;
        self.0 = self
            .0
            .checked_add(amount)
            .ok_or_else(|| io::Error::other("canonical JSON size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn compact_json_size(value: &Value) -> Result<u64, serde_json::Error> {
    let mut writer = CountingWriter::default();
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.0)
}

fn ensure_ambient_directory_identity(
    display_root: &Path,
    expected: StableFsIdentity,
) -> Result<(), BranchRefSnapshotError> {
    let rebound = open_absolute_dir_nofollow(display_root)
        .map_err(|error| changed(display_root, error.to_string()))?;
    expected
        .ensure_directory(&rebound, "branch reference directory")
        .map_err(|error| changed(display_root, error.to_string()))
}

fn is_json_ref(name: &OsStr) -> bool {
    Path::new(name).extension().and_then(OsStr::to_str) == Some("json")
}

fn checked_add(left: u64, right: u64, subject: &Path) -> Result<u64, BranchRefSnapshotError> {
    left.checked_add(right)
        .ok_or_else(|| BranchRefSnapshotError::InvalidRecord {
            path: subject.to_path_buf(),
            reason: "branch reference work count overflow".to_string(),
        })
}

fn changed(subject: &Path, reason: impl Into<String>) -> BranchRefSnapshotError {
    BranchRefSnapshotError::SubjectChanged {
        subject: subject.to_path_buf(),
        reason: reason.into(),
    }
}
