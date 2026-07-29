#![expect(
    clippy::result_large_err,
    reason = "Doctor branch accounting preserves the StoreError contract across ledger-bound helpers"
)]

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};
use serde_json::Value;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::secure_worktree::{open_absolute_dir_nofollow, StableFsIdentity};

const DIRECTORY_ITERATION_SCRATCH_BYTES: u64 = 4096;
const BOUNDED_READ_SENTINEL_BYTES: u64 = 1;
const DIGEST_HEX_SCRATCH_BYTES: u64 = 64;
const JSON_PARSE_SCRATCH_MULTIPLIER: u64 = 2;
const MAX_SYMLINK_TARGET_BYTES: u64 = 64 * 1024;
const OWNER_FRAME_BYTES: u64 = codefire_core::LOGICAL_FRAME_BASE_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateKind {
    Regular,
    Oversize,
    Symlink,
    Special,
}

struct CandidateEvidence {
    member_index: usize,
    display_path: PathBuf,
    kind: CandidateKind,
    entry_identity: StableFsIdentity,
    symlink_target_bytes: Option<u64>,
    symlink_digest: Option<[u8; 64]>,
    file_identity: Option<StableFsIdentity>,
    size_bytes: u64,
    digest: Option<[u8; 64]>,
    head: Option<String>,
}

struct AccountedCandidatePath {
    path: Option<PathBuf>,
    temporary_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

impl AccountedCandidatePath {
    fn prepare(
        base: &Path,
        name: &OsStr,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, codefire_store::StoreError> {
        let path_bytes = joined_path_bytes(base, name)?;
        let temporary_bytes = OWNER_FRAME_BYTES
            .checked_add(u64::try_from(path_bytes).map_err(|_| size_overflow())?)
            .ok_or_else(size_overflow)?;
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            ),
        )?;
        let mut path = PathBuf::new();
        if let Err(error) = path.as_mut_os_string().try_reserve_exact(path_bytes) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(io::Error::other(format!(
                "Doctor branch candidate path allocation failed: {error}"
            ))
            .into());
        }
        path.push(base);
        path.push(name);
        Ok(Self {
            path: Some(path),
            temporary_bytes,
            ledger_identity: ledger.identity(),
        })
    }

    fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("active candidate path owner retains its path")
    }

    fn take_for_transfer(&mut self) -> PathBuf {
        self.path
            .take()
            .expect("candidate path transfers only once")
    }

    fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor branch candidate path belongs to a different decision-work ledger"
                    .to_string(),
            ));
        }
        drop(self.path.take());
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            self.temporary_bytes,
        )?;
        self.temporary_bytes = 0;
        Ok(())
    }
}

impl Drop for AccountedCandidatePath {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.temporary_bytes, 0,
            "Doctor branch candidate path dropped without ledger cleanup"
        );
    }
}

pub(super) struct DoctorBranchSnapshot {
    display_parent: PathBuf,
    display_root: PathBuf,
    parent: Option<Dir>,
    parent_identity: StableFsIdentity,
    directory: Option<Dir>,
    directory_identity: Option<StableFsIdentity>,
    membership: Vec<OsString>,
    candidates: Vec<CandidateEvidence>,
    retained_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

impl DoctorBranchSnapshot {
    pub(super) fn capture<F>(
        cf: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut diagnose: F,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(
            &Path,
            fmt::Arguments<'_>,
            &mut codefire_core::DecisionWorkLedger,
        ) -> Result<(), codefire_store::StoreError>,
    {
        let ledger_identity = ledger.identity();
        let mut retained_bytes = 0_u64;
        let result = Self::capture_inner(
            cf,
            record_limit,
            ledger,
            ledger_identity,
            &mut retained_bytes,
            &mut diagnose,
        );
        match result {
            Ok(snapshot) => Ok(snapshot),
            Err(error) => {
                if retained_bytes != 0 {
                    ledger.release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    )?;
                }
                Err(error)
            }
        }
    }

    fn capture_inner<F>(
        cf: &Path,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
        ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
        retained_bytes: &mut u64,
        diagnose: &mut F,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(
            &Path,
            fmt::Arguments<'_>,
            &mut codefire_core::DecisionWorkLedger,
        ) -> Result<(), codefire_store::StoreError>,
    {
        let parent = open_absolute_dir_nofollow(cf)?;
        let parent_identity = StableFsIdentity::for_directory(&parent)?;
        ensure_ambient_directory_identity(cf, parent_identity)?;
        let display_parent = retain_path(cf, ledger, retained_bytes)?;
        let branches_name = OsStr::new("branches");
        let display_root = retain_joined_path(cf, branches_name, ledger, retained_bytes)?;
        let directory = match parent.open_dir_nofollow(branches_name) {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let Some(directory) = directory else {
            ensure_branch_slot(&parent, branches_name, &display_root, None)?;
            ensure_ambient_directory_identity(cf, parent_identity)?;
            return Ok(Self {
                display_parent,
                display_root,
                parent: Some(parent),
                parent_identity,
                directory: None,
                directory_identity: None,
                membership: Vec::new(),
                candidates: Vec::new(),
                retained_bytes: *retained_bytes,
                ledger_identity,
            });
        };
        let directory_identity = StableFsIdentity::for_directory(&directory)?;
        let membership = collect_membership(
            &directory,
            &display_root,
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkDimension::ObjectRecords,
            ledger,
            retained_bytes,
        )?;
        let mut candidates = Vec::new();
        for (member_index, file_name) in membership.iter().enumerate() {
            if !is_json(file_name) {
                continue;
            }
            let mut path_owner = AccountedCandidatePath::prepare(&display_root, file_name, ledger)?;
            let metadata = match directory.symlink_metadata(file_name) {
                Ok(metadata) => metadata,
                Err(error) => {
                    let diagnosed = diagnose(path_owner.path(), format_args!("{error}"), ledger);
                    path_owner.release(ledger)?;
                    diagnosed?;
                    continue;
                }
            };
            let entry_identity = match stable_metadata_identity(&metadata) {
                Ok(identity) => identity,
                Err(error) => {
                    path_owner.release(ledger)?;
                    return Err(error.into());
                }
            };
            let kind = if metadata.file_type().is_symlink() {
                CandidateKind::Symlink
            } else if metadata.is_file() {
                CandidateKind::Regular
            } else {
                CandidateKind::Special
            };
            let symlink_evidence = if kind == CandidateKind::Symlink {
                let evidence =
                    read_symlink_evidence(&directory, file_name, path_owner.path(), false, ledger);
                match evidence {
                    Ok(evidence) => Some(evidence),
                    Err(error) => {
                        path_owner.release(ledger)?;
                        return Err(error);
                    }
                }
            } else {
                None
            };
            if kind != CandidateKind::Regular || metadata.len() > record_limit {
                let evidence_kind = if kind == CandidateKind::Regular {
                    CandidateKind::Oversize
                } else {
                    kind
                };
                let evidence = match retain_candidate(
                    member_index,
                    &mut path_owner,
                    evidence_kind,
                    entry_identity,
                    symlink_evidence.map(|evidence| evidence.target_bytes),
                    symlink_evidence.map(|evidence| evidence.digest),
                    None,
                    metadata.len(),
                    None,
                    None,
                    ledger,
                    retained_bytes,
                    &mut candidates,
                ) {
                    Ok(evidence) => evidence,
                    Err(error) => {
                        path_owner.release(ledger)?;
                        return Err(error);
                    }
                };
                path_owner.release(ledger)?;
                let path = &evidence.display_path;
                diagnose(
                    path,
                    format_args!(
                        "branch record must be a regular no-follow file no larger than {record_limit} bytes (size {})",
                        metadata.len()
                    ),
                    ledger,
                )?;
                continue;
            }
            let initial = match read_regular_candidate(
                &directory,
                file_name,
                path_owner.path(),
                metadata.len(),
                record_limit,
                ledger,
            ) {
                Ok(initial) => initial,
                Err(error) => {
                    path_owner.release(ledger)?;
                    return Err(error);
                }
            };
            let mut initial = initial;
            let identity = initial.identity;
            if identity != entry_identity {
                let error = changed(
                    path_owner.path(),
                    "branch record changed before identity capture",
                );
                path_owner.release(ledger)?;
                initial.release(ledger)?;
                return Err(error);
            }
            let digest = initial.digest;
            let evidence = match retain_candidate(
                member_index,
                &mut path_owner,
                CandidateKind::Regular,
                entry_identity,
                None,
                None,
                Some(identity),
                metadata.len(),
                Some(digest),
                initial.head.take(),
                ledger,
                retained_bytes,
                &mut candidates,
            ) {
                Ok(evidence) => evidence,
                Err(error) => {
                    path_owner.release(ledger)?;
                    initial.release(ledger)?;
                    return Err(error);
                }
            };
            path_owner.release(ledger)?;
            let diagnosed = match initial.error.as_ref() {
                Some(error) => diagnose(&evidence.display_path, format_args!("{error}"), ledger),
                None => Ok(()),
            };
            initial.release(ledger)?;
            diagnosed?;
        }
        ensure_branch_slot(
            &parent,
            branches_name,
            &display_root,
            Some(directory_identity),
        )?;
        ensure_ambient_directory_identity(cf, parent_identity)?;
        Ok(Self {
            display_parent,
            display_root,
            parent: Some(parent),
            parent_identity,
            directory: Some(directory),
            directory_identity: Some(directory_identity),
            membership,
            candidates,
            retained_bytes: *retained_bytes,
            ledger_identity,
        })
    }

    pub(super) fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    pub(super) fn candidate_head_and_path(&self, index: usize) -> Option<(&str, &Path)> {
        let candidate = self.candidates.get(index)?;
        Some((candidate.head.as_deref()?, &candidate.display_path))
    }

    pub(super) fn revalidate(
        &mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor branch snapshot belongs to a different decision-work ledger".to_string(),
            ));
        }
        let result = self.revalidate_inner(record_limit, ledger);
        self.release_payload_then_scalar(ledger)?;
        result
    }

    pub(super) fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor branch snapshot belongs to a different decision-work ledger".to_string(),
            ));
        }
        self.release_payload_then_scalar(ledger)
    }

    fn release_payload_then_scalar(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        let retained = self.retained_bytes;
        drop(std::mem::take(&mut self.candidates));
        drop(std::mem::take(&mut self.membership));
        drop(self.directory.take());
        drop(self.parent.take());
        drop(std::mem::take(&mut self.display_root));
        drop(std::mem::take(&mut self.display_parent));
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained,
        )?;
        self.retained_bytes = 0;
        Ok(())
    }

    fn revalidate_inner(
        &mut self,
        record_limit: u64,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        let parent = self
            .parent
            .as_ref()
            .expect("active Doctor branch snapshot retains its parent capability");
        self.parent_identity
            .ensure_directory(parent, "Doctor branch parent")?;
        ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)?;
        let branches_name = OsStr::new("branches");
        ensure_branch_slot(
            parent,
            branches_name,
            &self.display_root,
            self.directory_identity,
        )?;
        let Some(directory) = self.directory.as_ref() else {
            ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)?;
            return Ok(());
        };
        for candidate in &self.candidates {
            let file_name = &self.membership[candidate.member_index];
            revalidate_candidate(directory, file_name, candidate, record_limit, ledger)?;
        }
        let (membership, temporary_bytes) =
            collect_temporary_membership(directory, &self.display_root, ledger)?;
        let matches = membership == self.membership;
        drop(membership);
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        if !matches {
            return Err(changed(
                &self.display_root,
                "branch directory membership changed",
            ));
        }
        ensure_branch_slot(
            parent,
            branches_name,
            &self.display_root,
            self.directory_identity,
        )?;
        ensure_ambient_directory_identity(&self.display_parent, self.parent_identity)
    }

    #[cfg(test)]
    pub(super) fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}

impl Drop for DoctorBranchSnapshot {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.retained_bytes, 0,
            "Doctor branch snapshot dropped without ledger cleanup"
        );
    }
}

struct CandidateRead {
    identity: StableFsIdentity,
    digest: [u8; 64],
    head: Option<String>,
    error: Option<CandidateReadError>,
    temporary_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

enum CandidateReadError {
    Json(serde_json::Error),
    MissingHead,
}

impl fmt::Display for CandidateReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => error.fmt(formatter),
            Self::MissingHead => {
                formatter.write_str("branch record must contain a non-empty string head")
            }
        }
    }
}

impl CandidateRead {
    fn release(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor branch read belongs to a different decision-work ledger".to_string(),
            ));
        }
        drop(self.head.take());
        drop(self.error.take());
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            self.temporary_bytes,
        )?;
        self.temporary_bytes = 0;
        Ok(())
    }
}

impl Drop for CandidateRead {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.temporary_bytes, 0,
            "Doctor branch read dropped without ledger cleanup"
        );
    }
}

fn read_regular_candidate(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    size_bytes: u64,
    limit: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<CandidateRead, codefire_store::StoreError> {
    read_regular_candidate_with_hook(
        directory,
        file_name,
        display_path,
        size_bytes,
        limit,
        ledger,
        || Ok(()),
    )
}

fn read_regular_candidate_with_hook<F>(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    size_bytes: u64,
    limit: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_read: F,
) -> Result<CandidateRead, codefire_store::StoreError>
where
    F: FnOnce() -> Result<(), codefire_store::StoreError>,
{
    let temporary_bytes = size_bytes
        .checked_mul(JSON_PARSE_SCRATCH_MULTIPLIER)
        .and_then(|value| value.checked_add(DIRECTORY_ITERATION_SCRATCH_BYTES))
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    let read_upper = size_bytes
        .checked_add(BOUNDED_READ_SENTINEL_BYTES)
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    let phase = codefire_core::DecisionWorkPhase::RecordRead;
    let mut permit = ledger.reserve(regular_read_work_reservation(
        phase, read_upper, size_bytes, false,
    ))?;
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        ),
    ) {
        ledger.cancel_reserved(&permit)?;
        return Err(error.into());
    }
    let opened = (|| {
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let file = directory.open_with(file_name, &options)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() != size_bytes || metadata.len() > limit {
            return Err(changed(
                display_path,
                "branch record changed before bounded read",
            ));
        }
        let identity = StableFsIdentity::for_file(&file)?;
        Ok::<_, codefire_store::StoreError>((file, identity))
    })();
    let (mut file, identity) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let fixed = match read_fixed_record_with_eof(&mut file, display_path, size_bytes, before_read) {
        Ok(fixed) => fixed,
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let bytes = match fixed {
        FixedRecordRead::Changed { error, bytes_read } => {
            ledger.consume_reserved(
                &mut permit,
                regular_read_work_reservation(phase, bytes_read, 0, false),
            )?;
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
        FixedRecordRead::Complete(bytes) => bytes,
    };
    if let Err(error) = identity.ensure_file(&file, "Doctor branch record") {
        drop(bytes);
        ledger.consume_reserved(
            &mut permit,
            regular_read_work_reservation(phase, size_bytes, 0, false),
        )?;
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        return Err(error.into());
    }
    if let Err(error) = ledger.consume_reserved(
        &mut permit,
        regular_read_work_reservation(phase, size_bytes, size_bytes, false),
    ) {
        drop(bytes);
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        return Err(error.into());
    }
    ledger.cancel_reserved(&permit)?;
    let digest: [u8; 64] = codefire_util::sha256_hex(&bytes)
        .into_bytes()
        .try_into()
        .expect("SHA-256 hex digest has a fixed length");
    let parsed = serde_json::from_slice::<Value>(&bytes);
    let (head, error) = match parsed {
        Ok(mut value) => match value.get_mut("head").map(Value::take) {
            Some(Value::String(head)) => (Some(head), None),
            _ => (None, Some(CandidateReadError::MissingHead)),
        },
        Err(error) => (None, Some(CandidateReadError::Json(error))),
    };
    Ok(CandidateRead {
        identity,
        digest,
        head,
        error,
        temporary_bytes,
        ledger_identity: ledger.identity(),
    })
}

enum FixedRecordRead {
    Complete(Vec<u8>),
    Changed {
        error: codefire_store::StoreError,
        bytes_read: u64,
    },
}

fn read_fixed_record_with_eof<F>(
    file: &mut cap_std::fs::File,
    display_path: &Path,
    size_bytes: u64,
    before_read: F,
) -> Result<FixedRecordRead, codefire_store::StoreError>
where
    F: FnOnce() -> Result<(), codefire_store::StoreError>,
{
    let size = usize::try_from(size_bytes)
        .map_err(|_| io::Error::other("branch record size does not fit usize"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|error| io::Error::other(format!("branch record allocation failed: {error}")))?;
    bytes.resize(size, 0);
    before_read()?;
    let mut offset = 0_usize;
    while offset < size {
        match file.read(&mut bytes[offset..]) {
            Ok(0) => {
                return Ok(FixedRecordRead::Changed {
                    error: changed(display_path, "branch record changed during fixed-size read"),
                    bytes_read: u64::try_from(offset)
                        .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?,
                });
            }
            Ok(read) => offset = offset.checked_add(read).ok_or_else(size_overflow)?,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return Ok(FixedRecordRead::Changed {
                    error: changed(display_path, "branch record changed during fixed-size read"),
                    bytes_read: u64::try_from(offset)
                        .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?,
                });
            }
        }
    }
    let mut sentinel = [0_u8; 1];
    let trailing = loop {
        match file.read(&mut sentinel) {
            Ok(read) => break read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return Ok(FixedRecordRead::Changed {
                    error: changed(display_path, "branch record changed during EOF check"),
                    bytes_read: size_bytes,
                });
            }
        }
    };
    if trailing != 0 {
        return Ok(FixedRecordRead::Changed {
            error: changed(display_path, "branch record grew during fixed-size read"),
            bytes_read: size_bytes
                .checked_add(u64::try_from(trailing).map_err(|_| size_overflow())?)
                .ok_or_else(size_overflow)?,
        });
    }
    Ok(FixedRecordRead::Complete(bytes))
}

fn regular_read_work_reservation(
    phase: codefire_core::DecisionWorkPhase,
    bytes_read: u64,
    bytes_hashed: u64,
    revalidation: bool,
) -> codefire_core::DecisionWorkReservation {
    let reservation = codefire_core::DecisionWorkReservation::new(
        phase,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
    .count(
        codefire_core::DecisionWorkCounter::FileBytesRead,
        bytes_read,
    )
    .count(
        codefire_core::DecisionWorkCounter::BytesHashed,
        bytes_hashed,
    );
    if revalidation {
        reservation
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                1,
            )
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                bytes_read,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
                1,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
                bytes_read,
            )
    } else {
        reservation.charge(
            codefire_core::DecisionWorkDimension::AggregateBytes,
            bytes_read,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn retain_candidate<'a>(
    member_index: usize,
    path_owner: &mut AccountedCandidatePath,
    kind: CandidateKind,
    entry_identity: StableFsIdentity,
    symlink_target_bytes: Option<u64>,
    symlink_digest: Option<[u8; 64]>,
    file_identity: Option<StableFsIdentity>,
    size_bytes: u64,
    digest: Option<[u8; 64]>,
    head: Option<String>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
    candidates: &'a mut Vec<CandidateEvidence>,
) -> Result<&'a CandidateEvidence, codefire_store::StoreError> {
    let path_bytes = path_owner.path().as_os_str().as_encoded_bytes().len();
    let digest_bytes = digest.as_ref().map_or(0, |_| 64);
    let head_bytes = head.as_ref().map_or(0, String::len);
    let amount = OWNER_FRAME_BYTES
        .checked_add(u64::try_from(path_bytes).map_err(|_| size_overflow())?)
        .and_then(|value| value.checked_add(u64::try_from(digest_bytes).ok()?))
        .and_then(|value| value.checked_add(u64::try_from(head_bytes).ok()?))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(size_overflow)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount),
    )?;
    let result = (|| {
        candidates.try_reserve(1).map_err(|error| {
            io::Error::other(format!("branch candidate allocation failed: {error}"))
        })?;
        let display_path = path_owner.take_for_transfer();
        candidates.push(CandidateEvidence {
            member_index,
            display_path,
            kind,
            entry_identity,
            symlink_target_bytes,
            symlink_digest,
            file_identity,
            size_bytes,
            digest,
            head,
        });
        Ok::<_, codefire_store::StoreError>(())
    })();
    if let Err(error) = result {
        ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)?;
        return Err(error);
    }
    *retained_bytes = retained_bytes
        .checked_add(amount)
        .ok_or_else(size_overflow)?;
    Ok(candidates.last().expect("candidate was just appended"))
}

fn revalidate_candidate(
    directory: &Dir,
    file_name: &OsStr,
    candidate: &CandidateEvidence,
    limit: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), codefire_store::StoreError> {
    revalidate_candidate_with_hook(directory, file_name, candidate, limit, ledger, || Ok(()))
}

fn revalidate_candidate_with_hook<F>(
    directory: &Dir,
    file_name: &OsStr,
    candidate: &CandidateEvidence,
    limit: u64,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_read: F,
) -> Result<(), codefire_store::StoreError>
where
    F: FnOnce() -> Result<(), codefire_store::StoreError>,
{
    let metadata = directory
        .symlink_metadata(file_name)
        .map_err(|error| changed(&candidate.display_path, error.to_string()))?;
    let current_identity = stable_metadata_identity(&metadata)
        .map_err(|error| changed(&candidate.display_path, error.to_string()))?;
    let current_kind = if metadata.file_type().is_symlink() {
        CandidateKind::Symlink
    } else if metadata.is_file() {
        if metadata.len() > limit {
            CandidateKind::Oversize
        } else {
            CandidateKind::Regular
        }
    } else {
        CandidateKind::Special
    };
    if current_identity != candidate.entry_identity
        || current_kind != candidate.kind
        || metadata.len() != candidate.size_bytes
    {
        return Err(changed(
            &candidate.display_path,
            "branch candidate identity changed",
        ));
    }
    if candidate.kind == CandidateKind::Symlink {
        let evidence =
            read_symlink_evidence(directory, file_name, &candidate.display_path, true, ledger)?;
        if candidate.symlink_target_bytes != Some(evidence.target_bytes)
            || candidate.symlink_digest != Some(evidence.digest)
        {
            return Err(changed(
                &candidate.display_path,
                "branch symlink target changed",
            ));
        }
        return Ok(());
    }
    if candidate.kind != CandidateKind::Regular {
        return Ok(());
    }
    let temporary_bytes = candidate
        .size_bytes
        .checked_add(BOUNDED_READ_SENTINEL_BYTES)
        .and_then(|value| value.checked_add(DIGEST_HEX_SCRATCH_BYTES))
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    let read_upper = candidate
        .size_bytes
        .checked_add(BOUNDED_READ_SENTINEL_BYTES)
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    let phase = codefire_core::DecisionWorkPhase::SnapshotRevalidation;
    let mut permit = ledger.reserve(regular_read_work_reservation(
        phase,
        read_upper,
        candidate.size_bytes,
        true,
    ))?;
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        ),
    ) {
        ledger.cancel_reserved(&permit)?;
        return Err(error.into());
    }
    let opened = (|| {
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let file = directory
            .open_with(file_name, &options)
            .map_err(|error| changed(&candidate.display_path, error.to_string()))?;
        candidate
            .file_identity
            .expect("regular candidate retains file identity")
            .ensure_file(&file, "Doctor branch revalidation")
            .map_err(|error| changed(&candidate.display_path, error.to_string()))?;
        Ok::<_, codefire_store::StoreError>(file)
    })();
    let mut file = match opened {
        Ok(file) => file,
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let fixed = match read_fixed_record_with_eof(
        &mut file,
        &candidate.display_path,
        candidate.size_bytes,
        before_read,
    ) {
        Ok(fixed) => fixed,
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let bytes = match fixed {
        FixedRecordRead::Changed { error, bytes_read } => {
            ledger.consume_reserved(
                &mut permit,
                regular_read_work_reservation(phase, bytes_read, 0, true),
            )?;
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
        FixedRecordRead::Complete(bytes) => bytes,
    };
    if let Err(error) = candidate
        .file_identity
        .expect("regular candidate retains file identity")
        .ensure_file(&file, "Doctor branch revalidation")
    {
        drop(bytes);
        ledger.consume_reserved(
            &mut permit,
            regular_read_work_reservation(phase, candidate.size_bytes, 0, true),
        )?;
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        return Err(changed(&candidate.display_path, error.to_string()));
    }
    if let Err(error) = ledger.consume_reserved(
        &mut permit,
        regular_read_work_reservation(phase, candidate.size_bytes, candidate.size_bytes, true),
    ) {
        drop(bytes);
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        return Err(error.into());
    }
    ledger.cancel_reserved(&permit)?;
    let result = (|| {
        let digest = codefire_util::sha256_hex(&bytes);
        if candidate.digest.as_ref().map(<[u8; 64]>::as_slice) != Some(digest.as_bytes()) {
            return Err(changed(
                &candidate.display_path,
                "branch record content changed",
            ));
        }
        candidate
            .file_identity
            .expect("regular candidate retains file identity")
            .ensure_file(&file, "Doctor branch revalidation")
            .map_err(|error| changed(&candidate.display_path, error.to_string()))
    })();
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary_bytes,
    )?;
    result
}

#[derive(Clone, Copy)]
struct SymlinkEvidence {
    target_bytes: u64,
    digest: [u8; 64],
}

fn read_symlink_evidence(
    directory: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    revalidation: bool,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<SymlinkEvidence, codefire_store::StoreError> {
    let temporary_bytes = OWNER_FRAME_BYTES
        .checked_add(MAX_SYMLINK_TARGET_BYTES)
        .and_then(|value| value.checked_add(BOUNDED_READ_SENTINEL_BYTES))
        .and_then(|value| value.checked_add(DIGEST_HEX_SCRATCH_BYTES))
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    let phase = if revalidation {
        codefire_core::DecisionWorkPhase::SnapshotRevalidation
    } else {
        codefire_core::DecisionWorkPhase::RecordRead
    };
    let mut permit = ledger.reserve(symlink_work_reservation(
        phase,
        MAX_SYMLINK_TARGET_BYTES,
        revalidation,
    ))?;
    if let Err(error) = ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        ),
    ) {
        ledger.cancel_reserved(&permit)?;
        return Err(error.into());
    }
    let result = (|| {
        let target = directory
            .read_link(file_name)
            .map_err(|error| changed(display_path, error.to_string()))?;
        let bytes = target.as_os_str().as_encoded_bytes();
        let target_bytes = u64::try_from(bytes.len())
            .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        if target_bytes > MAX_SYMLINK_TARGET_BYTES {
            return Err(changed(
                display_path,
                format!(
                    "branch symlink target exceeds bounded read limit {MAX_SYMLINK_TARGET_BYTES}"
                ),
            ));
        }
        let digest: [u8; 64] = codefire_util::sha256_hex(bytes)
            .into_bytes()
            .try_into()
            .expect("SHA-256 hex digest has a fixed length");
        Ok(SymlinkEvidence {
            target_bytes,
            digest,
        })
    })();
    let evidence = match result {
        Ok(evidence) => evidence,
        Err(error) => {
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
    };
    if let Err(error) = ledger.consume_reserved(
        &mut permit,
        symlink_work_reservation(phase, evidence.target_bytes, revalidation),
    ) {
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        return Err(error.into());
    }
    ledger.cancel_reserved(&permit)?;
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary_bytes,
    )?;
    Ok(evidence)
}

fn symlink_work_reservation(
    phase: codefire_core::DecisionWorkPhase,
    bytes: u64,
    revalidation: bool,
) -> codefire_core::DecisionWorkReservation {
    let reservation = codefire_core::DecisionWorkReservation::new(
        phase,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
    .count(codefire_core::DecisionWorkCounter::FileBytesRead, bytes)
    .count(codefire_core::DecisionWorkCounter::BytesHashed, bytes);
    if revalidation {
        reservation
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
                1,
            )
            .charge(
                codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                bytes,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
                1,
            )
            .count(
                codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
                bytes,
            )
    } else {
        reservation.charge(codefire_core::DecisionWorkDimension::AggregateBytes, bytes)
    }
}

fn collect_membership(
    directory: &Dir,
    display_root: &Path,
    phase: codefire_core::DecisionWorkPhase,
    record_dimension: codefire_core::DecisionWorkDimension,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
) -> Result<Vec<OsString>, codefire_store::StoreError> {
    let mut entries = directory.entries()?;
    let mut names = Vec::new();
    loop {
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            ),
        )?;
        let next = entries.next().transpose();
        let Some(entry) = (match next {
            Ok(next) => next,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error.into());
            }
        }) else {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            break;
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(record_dimension, 1)
            .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        let name = entry.file_name();
        let amount = match name_frame_bytes(&name) {
            Ok(amount) => amount,
            Err(error) => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error);
            }
        };
        let next_retained = match retained_bytes.checked_add(amount) {
            Some(next) => next,
            None => {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(size_overflow());
            }
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount),
        ) {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(error.into());
        }
        if let Err(error) = names.try_reserve(1) {
            ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            return Err(io::Error::other(format!(
                "branch membership allocation failed at {}: {error}",
                display_root.display()
            ))
            .into());
        }
        names.push(name);
        drop(entry);
        *retained_bytes = next_retained;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            DIRECTORY_ITERATION_SCRATCH_BYTES,
        )?;
    }
    names.sort_unstable();
    Ok(names)
}

fn collect_temporary_membership(
    directory: &Dir,
    display_root: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<OsString>, u64), codefire_store::StoreError> {
    let mut entries = directory
        .entries()
        .map_err(|error| changed(display_root, error.to_string()))?;
    let mut names = Vec::new();
    let mut temporary_bytes = 0_u64;
    let result = (|| {
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
            let next = entries.next().transpose();
            let Some(entry) = (match next {
                Ok(next) => next,
                Err(error) => {
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        DIRECTORY_ITERATION_SCRATCH_BYTES,
                    )?;
                    return Err(changed(display_root, error.to_string()));
                }
            }) else {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                break;
            };
            let name = entry.file_name();
            let amount = match name_frame_bytes(&name) {
                Ok(amount) => amount,
                Err(error) => {
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        DIRECTORY_ITERATION_SCRATCH_BYTES,
                    )?;
                    return Err(error);
                }
            };
            let next_temporary = match temporary_bytes.checked_add(amount) {
                Some(next) => next,
                None => {
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        DIRECTORY_ITERATION_SCRATCH_BYTES,
                    )?;
                    return Err(size_overflow());
                }
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
                .charge(
                    codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
                    amount,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, amount)
                .count(
                    codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
                    1,
                )
                .count(
                    codefire_core::DecisionWorkCounter::SnapshotRevalidationBytes,
                    amount,
                ),
            ) {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(error.into());
            }
            if let Err(error) = names.try_reserve(1) {
                ledger.release(codefire_core::DecisionWorkDimension::TemporaryBytes, amount)?;
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    DIRECTORY_ITERATION_SCRATCH_BYTES,
                )?;
                return Err(changed(display_root, error.to_string()));
            }
            names.push(name);
            drop(entry);
            temporary_bytes = next_temporary;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
        }
        names.sort_unstable();
        Ok::<_, codefire_store::StoreError>(())
    })();
    match result {
        Ok(()) => Ok((names, temporary_bytes)),
        Err(error) => {
            if temporary_bytes != 0 {
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    temporary_bytes,
                )?;
            }
            Err(error)
        }
    }
}

fn retain_path(
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
) -> Result<PathBuf, codefire_store::StoreError> {
    let bytes = path.as_os_str().as_encoded_bytes().len();
    retain_path_with_bytes(bytes, path, None, ledger, retained_bytes)
}

fn retain_joined_path(
    base: &Path,
    name: &OsStr,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
) -> Result<PathBuf, codefire_store::StoreError> {
    let bytes = joined_path_bytes(base, name)?;
    retain_path_with_bytes(bytes, base, Some(name), ledger, retained_bytes)
}

fn retain_path_with_bytes(
    bytes: usize,
    base: &Path,
    name: Option<&OsStr>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    retained_bytes: &mut u64,
) -> Result<PathBuf, codefire_store::StoreError> {
    let amount = OWNER_FRAME_BYTES
        .checked_add(u64::try_from(bytes).map_err(|_| size_overflow())?)
        .ok_or_else(size_overflow)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount),
    )?;
    let mut path = PathBuf::new();
    if let Err(error) = path.as_mut_os_string().try_reserve_exact(bytes) {
        ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)?;
        return Err(
            io::Error::other(format!("Doctor branch path allocation failed: {error}")).into(),
        );
    }
    path.push(base);
    if let Some(name) = name {
        path.push(name);
    }
    *retained_bytes = retained_bytes
        .checked_add(amount)
        .ok_or_else(size_overflow)?;
    Ok(path)
}

fn joined_path_bytes(base: &Path, name: &OsStr) -> Result<usize, codefire_store::StoreError> {
    base.as_os_str()
        .as_encoded_bytes()
        .len()
        .checked_add(1)
        .and_then(|value| value.checked_add(name.as_encoded_bytes().len()))
        .ok_or_else(size_overflow)
}

fn name_frame_bytes(name: &OsStr) -> Result<u64, codefire_store::StoreError> {
    OWNER_FRAME_BYTES
        .checked_add(u64::try_from(name.as_encoded_bytes().len()).map_err(|_| size_overflow())?)
        .ok_or_else(size_overflow)
}

fn ensure_branch_slot(
    parent: &Dir,
    name: &OsStr,
    display_root: &Path,
    expected: Option<StableFsIdentity>,
) -> Result<(), codefire_store::StoreError> {
    match (expected, parent.open_dir_nofollow(name)) {
        (None, Err(error)) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        (None, Ok(_)) => Err(changed(display_root, "branch directory appeared")),
        (None, Err(error)) => Err(changed(display_root, error.to_string())),
        (Some(_), Err(error)) => Err(changed(display_root, error.to_string())),
        (Some(expected), Ok(directory)) => expected
            .ensure_directory(&directory, "Doctor branch directory")
            .map_err(|error| changed(display_root, error.to_string())),
    }
}

fn ensure_ambient_directory_identity(
    display_path: &Path,
    expected: StableFsIdentity,
) -> Result<(), codefire_store::StoreError> {
    let rebound = open_absolute_dir_nofollow(display_path)
        .map_err(|error| changed(display_path, error.to_string()))?;
    expected
        .ensure_directory(&rebound, "Doctor branch ambient directory")
        .map_err(|error| changed(display_path, error.to_string()))
}

fn changed(path: &Path, reason: impl fmt::Display) -> codefire_store::StoreError {
    codefire_store::StoreError::InvalidSealedCommit(format!(
        "Doctor branch snapshot changed at {}: {reason}",
        path.display()
    ))
}

fn size_overflow() -> codefire_store::StoreError {
    codefire_core::DecisionWorkAccountingError::SizeOverflow.into()
}

#[cfg(unix)]
fn stable_metadata_identity(metadata: &cap_std::fs::Metadata) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    StableFsIdentity::from_parts("unix_dev_inode", metadata.dev(), metadata.ino())
}

#[cfg(windows)]
fn stable_metadata_identity(metadata: &cap_std::fs::Metadata) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    match (metadata.volume_serial_number(), metadata.file_index()) {
        (Some(volume), Some(index)) => {
            StableFsIdentity::from_parts("windows_volume_file_index", u64::from(volume), index)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filesystem entry identity cannot be proven",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn stable_metadata_identity(_metadata: &cap_std::fs::Metadata) -> io::Result<StableFsIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem entry identity cannot be proven on this platform",
    ))
}

fn is_json(name: &OsStr) -> bool {
    Path::new(name).extension().and_then(OsStr::to_str) == Some("json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn ledger(command: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            command,
        )
        .unwrap()
    }

    fn assert_clean(ledger: &codefire_core::DecisionWorkLedger) {
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
    }

    fn write_branch(path: &Path, head: &str) {
        fs::write(
            path,
            serde_json::to_vec(&serde_json::json!({"name": "main", "head": head})).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn candidate_errors_continue_and_unchanged_invalid_entries_close_cleanly() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let branches = cf.join("branches");
        fs::create_dir_all(&branches).unwrap();
        write_branch(&branches.join("a-valid.json"), "CF-COMMIT-valid");
        fs::write(branches.join("b-malformed.json"), b"{").unwrap();
        fs::write(branches.join("c-missing.json"), b"{}").unwrap();
        fs::write(branches.join("d-oversize.json"), vec![b'x'; 129]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("a-valid.json", branches.join("e-symlink.json")).unwrap();
        fs::create_dir(branches.join("f-special.json")).unwrap();
        let mut ledger = ledger("Doctor branch candidate continuation");
        let mut diagnostics = 0usize;

        let mut snapshot = DoctorBranchSnapshot::capture(&cf, 128, &mut ledger, |_, _, _| {
            diagnostics += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(snapshot.candidate_count(), if cfg!(unix) { 6 } else { 5 });
        assert_eq!(diagnostics, if cfg!(unix) { 5 } else { 4 });
        let heads = (0..snapshot.candidate_count())
            .filter_map(|index| snapshot.candidate_head_and_path(index).map(|value| value.0))
            .collect::<Vec<_>>();
        assert_eq!(heads, ["CF-COMMIT-valid"]);
        snapshot.revalidate(128, &mut ledger).unwrap();
        assert_clean(&ledger);
    }

    #[test]
    fn snapshot_rejects_file_membership_root_and_missing_root_races() {
        fn capture(
            cf: &Path,
            ledger: &mut codefire_core::DecisionWorkLedger,
        ) -> DoctorBranchSnapshot {
            DoctorBranchSnapshot::capture(cf, 1024, ledger, |_, _, _| Ok(())).unwrap()
        }

        for race in ["replace-file", "appear", "disappear", "replace-root"] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join(".codefire");
            let branches = cf.join("branches");
            fs::create_dir_all(&branches).unwrap();
            write_branch(&branches.join("main.json"), "CF-COMMIT-a");
            let mut ledger = ledger(&format!("Doctor branch {race}"));
            let mut snapshot = capture(&cf, &mut ledger);
            match race {
                "replace-file" => {
                    write_branch(&branches.join("replacement"), "CF-COMMIT-a");
                    fs::rename(branches.join("replacement"), branches.join("main.json")).unwrap();
                }
                "appear" => fs::write(branches.join("appeared.txt"), b"x").unwrap(),
                "disappear" => fs::remove_file(branches.join("main.json")).unwrap(),
                "replace-root" => {
                    fs::rename(&branches, cf.join("branches-old")).unwrap();
                    fs::create_dir(&branches).unwrap();
                    write_branch(&branches.join("main.json"), "CF-COMMIT-a");
                }
                _ => unreachable!(),
            }

            let error = snapshot.revalidate(1024, &mut ledger).unwrap_err();
            assert!(error.to_string().contains("Doctor branch"));
            assert_clean(&ledger);
        }

        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        fs::create_dir(&cf).unwrap();
        let mut ledger = ledger("Doctor missing branch root appearance");
        let mut snapshot = capture(&cf, &mut ledger);
        fs::create_dir(cf.join("branches")).unwrap();
        assert!(snapshot.revalidate(1024, &mut ledger).is_err());
        assert_clean(&ledger);
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_same_size_symlink_replacement() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let branches = cf.join("branches");
        fs::create_dir_all(&branches).unwrap();
        let link = branches.join("invalid.json");
        std::os::unix::fs::symlink("../target-a", &link).unwrap();
        let mut ledger = ledger("Doctor same-size symlink replacement");
        let mut snapshot =
            DoctorBranchSnapshot::capture(&cf, 1024, &mut ledger, |_, _, _| Ok(())).unwrap();
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("../target-b", &link).unwrap();

        let error = snapshot.revalidate(1024, &mut ledger).unwrap_err();

        assert!(error.to_string().contains("Doctor branch snapshot changed"));
        assert_clean(&ledger);
    }

    #[test]
    fn snapshot_rejects_foreign_ledger_without_consuming_owner() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        fs::create_dir_all(cf.join("branches")).unwrap();
        write_branch(&cf.join("branches/main.json"), "CF-COMMIT-a");
        let mut owner = ledger("Doctor branch owner");
        let mut foreign = ledger("Doctor branch foreign");
        let mut snapshot =
            DoctorBranchSnapshot::capture(&cf, 1024, &mut owner, |_, _, _| Ok(())).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let error = snapshot.cancel(&mut foreign).unwrap_err();

        assert!(error.to_string().contains("different decision-work ledger"));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert!(snapshot.retained_bytes() > 0);
        snapshot.cancel(&mut owner).unwrap();
        assert_clean(&owner);
        assert_clean(&foreign);
    }

    #[test]
    fn source_owners_reject_foreign_release_and_preserve_failed_scalar_for_retry() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let branches = cf.join("branches");
        fs::create_dir_all(&branches).unwrap();
        let record_path = branches.join("main.json");
        write_branch(&record_path, "CF-COMMIT-a");
        let mut owner = ledger("Doctor branch source owner");
        let mut foreign = ledger("Doctor branch source foreign");
        let mut path =
            AccountedCandidatePath::prepare(&branches, OsStr::new("main.json"), &mut owner)
                .unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        assert!(path.release(&mut foreign).is_err());
        assert_eq!(path.path(), record_path);
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        path.release(&mut owner).unwrap();

        let directory = open_absolute_dir_nofollow(&branches).unwrap();
        let size = fs::metadata(&record_path).unwrap().len();
        let mut read = read_regular_candidate(
            &directory,
            OsStr::new("main.json"),
            &record_path,
            size,
            1024,
            &mut owner,
        )
        .unwrap();
        assert!(read.release(&mut foreign).is_err());
        assert_eq!(read.head.as_deref(), Some("CF-COMMIT-a"));
        let scalar = read.temporary_bytes;
        owner
            .release(codefire_core::DecisionWorkDimension::TemporaryBytes, scalar)
            .unwrap();

        assert!(read.release(&mut owner).is_err());
        assert!(read.head.is_none());
        assert_eq!(read.temporary_bytes, scalar);
        owner
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, scalar),
            )
            .unwrap();
        read.release(&mut owner).unwrap();
        read.release(&mut owner).unwrap();
        assert_clean(&owner);
        assert_clean(&foreign);
    }

    #[test]
    fn initial_fixed_read_rejects_same_inode_append_without_counter_or_vec_growth() {
        let temp = tempdir().unwrap();
        let branches = temp.path().join("branches");
        fs::create_dir(&branches).unwrap();
        let record_path = branches.join("main.json");
        write_branch(&record_path, "CF-COMMIT-a");
        let size = fs::metadata(&record_path).unwrap().len();
        let directory = open_absolute_dir_nofollow(&branches).unwrap();
        let mut ledger = ledger("Doctor branch initial same-inode append");

        let error = read_regular_candidate_with_hook(
            &directory,
            OsStr::new("main.json"),
            &record_path,
            size,
            1024,
            &mut ledger,
            || {
                use std::io::Write as _;

                fs::OpenOptions::new()
                    .append(true)
                    .open(&record_path)?
                    .write_all(b" ")?;
                Ok(())
            },
        )
        .err()
        .expect("same-inode append must reject the initial fixed-size read");

        assert!(error.to_string().contains("grew during fixed-size read"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            size + 1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FileBytesRead],
            size + 1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            0
        );
        assert_clean(&ledger);
    }

    #[test]
    fn final_fixed_read_rejects_same_inode_append_with_exact_physical_accounting() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let branches = cf.join("branches");
        fs::create_dir_all(&branches).unwrap();
        let record_path = branches.join("main.json");
        write_branch(&record_path, "CF-COMMIT-a");
        let size = fs::metadata(&record_path).unwrap().len();
        let mut ledger = ledger("Doctor branch final same-inode append");
        let mut snapshot =
            DoctorBranchSnapshot::capture(&cf, 1024, &mut ledger, |_, _, _| Ok(())).unwrap();
        let candidate = &snapshot.candidates[0];
        let file_name = &snapshot.membership[candidate.member_index];

        let error = revalidate_candidate_with_hook(
            snapshot.directory.as_ref().unwrap(),
            file_name,
            candidate,
            1024,
            &mut ledger,
            || {
                use std::io::Write as _;

                fs::OpenOptions::new()
                    .append(true)
                    .open(&record_path)?
                    .write_all(b" ")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("grew during fixed-size read"));
        assert_eq!(
            ledger.snapshot().cumulative
                [&codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes],
            size + 1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FileBytesRead],
            size * 2 + 1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::BytesHashed],
            size
        );
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::TemporaryBytes],
            0
        );
        snapshot.cancel(&mut ledger).unwrap();
        assert_clean(&ledger);
    }

    #[test]
    fn retained_boundary_accepts_limit_minus_one_and_exact_then_rejects_plus_one() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        fs::create_dir_all(cf.join("branches")).unwrap();
        write_branch(&cf.join("branches/main.json"), "CF-COMMIT-a");
        let mut probe = ledger("Doctor branch retained probe");
        let mut probe_snapshot =
            DoctorBranchSnapshot::capture(&cf, 1024, &mut probe, |_, _, _| Ok(())).unwrap();
        let owned = probe_snapshot.retained_bytes();
        probe_snapshot.cancel(&mut probe).unwrap();
        assert_clean(&probe);
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "Doctor branch retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - owned - 1,
                0 => limit - owned,
                1 => limit - owned + 1,
                _ => unreachable!(),
            };
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
            let result = DoctorBranchSnapshot::capture(&cf, 1024, &mut ledger, |_, _, _| Ok(()));
            if delta <= 0 {
                let mut snapshot = result.unwrap();
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if delta == 0 { limit } else { limit - 1 }
                );
                snapshot.cancel(&mut ledger).unwrap();
            } else {
                let error = result.err().expect("limit+1 must fail");
                let codefire_store::StoreError::DecisionWorkAccounting(accounting) = error else {
                    panic!("expected typed accounting failure");
                };
                let exhaustion = accounting.budget_exceeded().unwrap();
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, limit + 1);
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_clean(&ledger);
        }
    }

    #[test]
    fn short_record_revalidation_accounts_digest_scratch_at_boundary() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let branches = cf.join("branches");
        fs::create_dir_all(&branches).unwrap();
        let record = br#"{"head":""}"#;
        fs::write(branches.join("main.json"), record).unwrap();
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let revalidation_temporary = u64::try_from(record.len())
            .unwrap()
            .checked_add(BOUNDED_READ_SENTINEL_BYTES)
            .unwrap()
            .checked_add(DIGEST_HEX_SCRATCH_BYTES)
            .unwrap();

        for exceeds_by_one in [false, true] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "Doctor short branch revalidation boundary",
            )
            .unwrap();
            let mut snapshot =
                DoctorBranchSnapshot::capture(&cf, 1024, &mut ledger, |_, _, _| Ok(())).unwrap();
            let baseline = limit - revalidation_temporary + u64::from(exceeds_by_one);
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        baseline,
                    ),
                )
                .unwrap();

            let file_name = &snapshot.membership[snapshot.candidates[0].member_index];
            let result = revalidate_candidate(
                snapshot.directory.as_ref().unwrap(),
                file_name,
                &snapshot.candidates[0],
                1024,
                &mut ledger,
            );
            if exceeds_by_one {
                let error = result.unwrap_err();
                let codefire_store::StoreError::DecisionWorkAccounting(accounting) = error else {
                    panic!("expected typed TemporaryBytes exhaustion");
                };
                let exhaustion = accounting.budget_exceeded().unwrap();
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::TemporaryBytes
                );
                assert_eq!(exhaustion.observed, limit + 1);
            } else {
                result.unwrap();
                assert_eq!(
                    ledger.snapshot().peak[&codefire_core::DecisionWorkDimension::TemporaryBytes],
                    limit
                );
                assert_eq!(
                    ledger.snapshot().cumulative
                        [&codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes],
                    u64::try_from(record.len()).unwrap()
                );
                assert_eq!(
                    ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FileBytesRead],
                    u64::try_from(record.len()).unwrap() * 2
                );
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            snapshot.cancel(&mut ledger).unwrap();
            assert_clean(&ledger);
        }
    }
}
