#![expect(
    clippy::result_large_err,
    reason = "Doctor opened-registry accounting preserves the StoreError contract"
)]

use crate::secure_worktree::{open_absolute_dir_nofollow, StableFsIdentity};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};
use serde_json::Value;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Write as _;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

#[cfg(test)]
use std::fs;

const OWNER_FRAME_BYTES: u64 = codefire_core::LOGICAL_FRAME_BASE_BYTES;
const DIRECTORY_ITERATION_SCRATCH_BYTES: u64 = 4096;
const COMPONENT_SOURCE_UPPER_BYTES: u64 = 64 * 1024;
const ABSOLUTE_OPEN_TRAVERSAL_HANDLES: u64 = 2;
const BOUNDED_READ_SENTINEL_BYTES: u64 = 1;
const REGISTRY_RECORD_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;
const JSON_PARSE_SCRATCH_MULTIPLIER: u64 = 2;
const JSON_VALUE_RETAINED_UPPER_MULTIPLIER: u64 = 4;
const JSON_VALUE_RETAINED_SLACK_BYTES: u64 = 16;
const JSON_ERROR_RETAINED_UPPER_BYTES: u64 = OWNER_FRAME_BYTES + 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Directory,
    Regular,
    Symlink,
    Special,
}

#[derive(Debug, PartialEq, Eq)]
struct EntryEvidence {
    path: PathBuf,
    kind: EntryKind,
    identity: StableFsIdentity,
    size_bytes: Option<u64>,
    json_candidate: bool,
    raw: Option<CandidateRaw>,
}

pub(super) struct DoctorOpenedRegistryCandidate<'a> {
    pub(super) path: &'a Path,
    pub(super) source: DoctorOpenedRegistrySource<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DoctorOpenedRegistrySource<'a> {
    Bytes(&'a [u8]),
    Unavailable(CandidateRawUnavailable),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CandidateRawUnavailable {
    Open(CandidateIoEvidence),
    Metadata(CandidateIoEvidence),
    Identity(CandidateIoEvidence),
    Rejected {
        size: u64,
    },
    MultipleLinks,
    #[cfg(not(unix))]
    UnprovableLinks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidateIoEvidence {
    kind: io::ErrorKind,
    raw_os_error: Option<i32>,
}

impl CandidateIoEvidence {
    fn capture(error: &io::Error) -> Self {
        Self {
            kind: error.kind(),
            raw_os_error: error.raw_os_error(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CandidateRaw {
    Bytes(Vec<u8>),
    Unavailable(CandidateRawUnavailable),
}

struct ParsedCandidate {
    entry_index: usize,
    payload: ParsedCandidatePayload,
}

struct ParsedCandidateTransition {
    entry_index: usize,
    insertion_index: usize,
    payload: Option<ParsedCandidatePayload>,
    retained_bytes: u64,
    temporary_bytes: u64,
}

impl Drop for ParsedCandidateTransition {
    fn drop(&mut self) {
        debug_assert!(
            self.payload.is_none() && self.retained_bytes == 0 && self.temporary_bytes == 0,
            "Doctor opened-registry parse transition dropped without cleanup"
        );
    }
}

enum ParsedCandidatePayload {
    Value(Value),
    Error(String),
}

#[derive(Debug)]
pub(super) enum DoctorOpenedRegistryParseResult {
    Parsed,
    Invalid,
    CleanupFailed(codefire_store::StoreError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateParseStage {
    BeforeMaterialization,
    AfterParseAttempt,
}

impl CandidateRawUnavailable {
    pub(super) fn at(self, path: &Path) -> CandidateRawUnavailableDisplay<'_> {
        CandidateRawUnavailableDisplay { reason: self, path }
    }
}

pub(super) struct CandidateRawUnavailableDisplay<'a> {
    reason: CandidateRawUnavailable,
    path: &'a Path,
}

impl fmt::Display for CandidateRawUnavailableDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            CandidateRawUnavailable::Open(error) => write!(
                formatter,
                "bounded no-follow reader could not open {}: {error}",
                self.path.display()
            ),
            CandidateRawUnavailable::Metadata(error) => write!(
                formatter,
                "bounded no-follow reader could not inspect {}: {error}",
                self.path.display()
            ),
            CandidateRawUnavailable::Identity(error) => write!(
                formatter,
                "bounded no-follow reader could not prove identity for {}: {error}",
                self.path.display()
            ),
            CandidateRawUnavailable::Rejected { size } => write!(
                formatter,
                "bounded no-follow reader rejected {} (size {size}, limit {REGISTRY_RECORD_LIMIT})",
                self.path.display()
            ),
            CandidateRawUnavailable::MultipleLinks => {
                #[cfg(unix)]
                return write!(
                    formatter,
                    "bounded authority must have exactly one filesystem link: {}",
                    self.path.display()
                );
                #[cfg(windows)]
                return write!(
                    formatter,
                    "bounded authority must have one provable filesystem link: {}",
                    self.path.display()
                );
                #[cfg(not(any(unix, windows)))]
                return write!(
                    formatter,
                    "bounded authority link count cannot be proven: {}",
                    self.path.display()
                );
            }
            #[cfg(not(unix))]
            CandidateRawUnavailable::UnprovableLinks => write!(
                formatter,
                "bounded authority link count cannot be proven: {}",
                self.path.display()
            ),
        }
    }
}

impl fmt::Display for CandidateIoEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(raw_os_error) = self.raw_os_error {
            io::Error::from_raw_os_error(raw_os_error).fmt(formatter)
        } else {
            write!(formatter, "{:?}", self.kind)
        }
    }
}

#[must_use = "opened-registry snapshot ownership must be revalidated or cancelled"]
pub(super) struct DoctorOpenedRegistrySnapshot {
    display_parent: PathBuf,
    display_root: PathBuf,
    parent_identity: StableFsIdentity,
    root_identity: Option<StableFsIdentity>,
    parent: Option<Dir>,
    root: Option<Dir>,
    entries: Vec<EntryEvidence>,
    parsed_candidates: Vec<ParsedCandidate>,
    parse_cleanup_temporary_bytes: u64,
    retained_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

impl DoctorOpenedRegistrySnapshot {
    pub(super) fn capture(
        parent: &Path,
        root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, codefire_store::StoreError> {
        Self::capture_with_hook(parent, root, ledger, |_| Ok(()))
    }

    fn capture_with_hook<F>(
        parent: &Path,
        root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut before_entry_slot: F,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        Self::capture_with_hooks(
            parent,
            root,
            ledger,
            &mut |_| Ok(()),
            &mut before_entry_slot,
        )
    }

    fn capture_with_hooks<F, G>(
        parent: &Path,
        root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
        before_entry_slot: &mut G,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        Self::capture_with_all_hooks(
            parent,
            root,
            ledger,
            before_absolute_open,
            before_entry_slot,
            &mut |_| Ok(()),
        )
    }

    #[cfg(test)]
    fn capture_with_read_hook<F>(
        parent: &Path,
        root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut before_candidate_read: F,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        Self::capture_with_all_hooks(
            parent,
            root,
            ledger,
            &mut |_| Ok(()),
            &mut |_| Ok(()),
            &mut before_candidate_read,
        )
    }

    fn capture_with_all_hooks<F, G, H>(
        parent: &Path,
        root: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
        before_entry_slot: &mut G,
        before_candidate_read: &mut H,
    ) -> Result<Self, codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        H: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        let parent_bytes = parent.as_os_str().as_encoded_bytes().len();
        let root_bytes = root.as_os_str().as_encoded_bytes().len();
        let base_upper = OWNER_FRAME_BYTES
            .checked_mul(5)
            .and_then(|value| value.checked_add(u64::try_from(parent_bytes).ok()?))
            .and_then(|value| value.checked_add(u64::try_from(root_bytes).ok()?))
            .ok_or_else(size_overflow)?;
        let mut permit = ledger.reserve(reservation(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            base_upper,
        ))?;
        let allocated = (|| {
            let parent_capability = open_absolute_dir_accounted(
                parent,
                codefire_core::DecisionWorkPhase::RecordRead,
                ledger,
                before_absolute_open,
                codefire_store::StoreError::from,
            )?;
            let parent_identity = StableFsIdentity::for_directory(&parent_capability)?;
            let root_capability = match parent_capability.open_dir_nofollow("opened") {
                Ok(directory) => Some(directory),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(error.into()),
            };
            let root_identity = root_capability
                .as_ref()
                .map(StableFsIdentity::for_directory)
                .transpose()?;
            let display_parent = copy_path(parent, parent_bytes)?;
            let display_root = copy_path(root, root_bytes)?;
            let base_bytes = if root_identity.is_some() {
                base_upper
            } else {
                base_upper
                    .checked_sub(OWNER_FRAME_BYTES)
                    .ok_or_else(size_overflow)?
            };
            Ok::<_, codefire_store::StoreError>((
                parent_capability,
                parent_identity,
                root_capability,
                root_identity,
                display_parent,
                display_root,
                Vec::new(),
                base_bytes,
            ))
        })();
        let (
            parent_capability,
            parent_identity,
            root_capability,
            root_identity,
            display_parent,
            display_root,
            mut entries,
            base_bytes,
        ) = match allocated {
            Ok(allocated) => allocated,
            Err(error) => {
                ledger.cancel_reserved(&permit)?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            reservation(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                base_bytes,
            ),
        ) {
            drop(entries);
            drop(display_root);
            drop(display_parent);
            drop(root_capability);
            drop(parent_capability);
            ledger.cancel_reserved(&permit)?;
            return Err(error.into());
        }
        if base_bytes < base_upper {
            ledger.cancel_reserved(&permit)?;
        }
        let mut retained_bytes = base_bytes;
        let collected = if let Some(root_capability) = root_capability.as_ref() {
            collect_entries(
                root_capability,
                &display_root,
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                ledger,
                &mut retained_bytes,
                &mut entries,
                before_entry_slot,
                before_candidate_read,
            )
        } else {
            Ok(())
        };
        if let Err(error) = collected {
            drop(entries);
            drop(display_root);
            drop(display_parent);
            drop(root_capability);
            drop(parent_capability);
            ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            )?;
            return Err(error);
        }
        Ok(Self {
            display_parent,
            display_root,
            parent_identity,
            root_identity,
            parent: Some(parent_capability),
            root: root_capability,
            entries,
            parsed_candidates: Vec::new(),
            parse_cleanup_temporary_bytes: 0,
            retained_bytes,
            ledger_identity: ledger.identity(),
        })
    }

    pub(super) fn exists(&self) -> bool {
        self.root_identity.is_some()
    }

    #[cfg(test)]
    pub(super) fn candidates(&self) -> impl Iterator<Item = DoctorOpenedRegistryCandidate<'_>> {
        self.entries
            .iter()
            .filter(|entry| entry.json_candidate)
            .map(|entry| DoctorOpenedRegistryCandidate {
                path: entry.path.as_path(),
                source: match entry
                    .raw
                    .as_ref()
                    .expect("JSON candidate retains bytes or typed unavailable evidence")
                {
                    CandidateRaw::Bytes(raw) => DoctorOpenedRegistrySource::Bytes(raw),
                    CandidateRaw::Unavailable(reason) => {
                        DoctorOpenedRegistrySource::Unavailable(*reason)
                    }
                },
            })
    }

    pub(super) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn candidate(
        &self,
        entry_index: usize,
    ) -> Option<DoctorOpenedRegistryCandidate<'_>> {
        let entry = self.entries.get(entry_index)?;
        if !entry.json_candidate {
            return None;
        }
        Some(DoctorOpenedRegistryCandidate {
            path: entry.path.as_path(),
            source: match entry
                .raw
                .as_ref()
                .expect("JSON candidate retains bytes or typed unavailable evidence")
            {
                CandidateRaw::Bytes(raw) => DoctorOpenedRegistrySource::Bytes(raw),
                CandidateRaw::Unavailable(reason) => {
                    DoctorOpenedRegistrySource::Unavailable(*reason)
                }
            },
        })
    }

    pub(super) fn parsed_candidate(&self, entry_index: usize) -> Option<(&Path, &Value)> {
        let candidate = self
            .parsed_candidates
            .binary_search_by_key(&entry_index, |candidate| candidate.entry_index)
            .ok()
            .and_then(|index| self.parsed_candidates.get(index))?;
        Some((
            self.entries.get(entry_index)?.path.as_path(),
            match &candidate.payload {
                ParsedCandidatePayload::Value(value) => value,
                ParsedCandidatePayload::Error(_) => return None,
            },
        ))
    }

    pub(super) fn invalid_candidate(&self, entry_index: usize) -> Option<(&Path, &str)> {
        let candidate = self
            .parsed_candidates
            .binary_search_by_key(&entry_index, |candidate| candidate.entry_index)
            .ok()
            .and_then(|index| self.parsed_candidates.get(index))?;
        Some((
            self.entries.get(entry_index)?.path.as_path(),
            match &candidate.payload {
                ParsedCandidatePayload::Error(message) => message,
                ParsedCandidatePayload::Value(_) => return None,
            },
        ))
    }

    pub(super) fn parse_candidate(
        &mut self,
        entry_index: usize,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<DoctorOpenedRegistryParseResult, codefire_store::StoreError> {
        self.parse_candidate_with_hooks(
            entry_index,
            ledger,
            |_, _| Ok(()),
            |ledger, temporary_bytes| {
                ledger
                    .release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        temporary_bytes,
                    )
                    .map_err(Into::into)
            },
        )
    }

    #[cfg(test)]
    fn parse_candidate_with_hook<F>(
        &mut self,
        entry_index: usize,
        ledger: &mut codefire_core::DecisionWorkLedger,
        poll: F,
    ) -> Result<DoctorOpenedRegistryParseResult, codefire_store::StoreError>
    where
        F: FnMut(
            CandidateParseStage,
            &codefire_core::DecisionWorkLedger,
        ) -> Result<(), codefire_store::StoreError>,
    {
        self.parse_candidate_with_hooks(entry_index, ledger, poll, |ledger, temporary_bytes| {
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    temporary_bytes,
                )
                .map_err(Into::into)
        })
    }

    #[cfg(test)]
    pub(super) fn parse_candidate_with_release_hook<R>(
        &mut self,
        entry_index: usize,
        ledger: &mut codefire_core::DecisionWorkLedger,
        release_temporary: R,
    ) -> Result<DoctorOpenedRegistryParseResult, codefire_store::StoreError>
    where
        R: FnOnce(
            &mut codefire_core::DecisionWorkLedger,
            u64,
        ) -> Result<(), codefire_store::StoreError>,
    {
        self.parse_candidate_with_hooks(entry_index, ledger, |_, _| Ok(()), release_temporary)
    }

    fn parse_candidate_with_hooks<F, R>(
        &mut self,
        entry_index: usize,
        ledger: &mut codefire_core::DecisionWorkLedger,
        mut poll: F,
        release_temporary: R,
    ) -> Result<DoctorOpenedRegistryParseResult, codefire_store::StoreError>
    where
        F: FnMut(
            CandidateParseStage,
            &codefire_core::DecisionWorkLedger,
        ) -> Result<(), codefire_store::StoreError>,
        R: FnOnce(
            &mut codefire_core::DecisionWorkLedger,
            u64,
        ) -> Result<(), codefire_store::StoreError>,
    {
        self.ensure_ledger(ledger)?;
        let insertion_index = match self
            .parsed_candidates
            .binary_search_by_key(&entry_index, |candidate| candidate.entry_index)
        {
            Ok(_) => {
                return Err(codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor opened-registry candidate was parsed more than once".to_string(),
                ))
            }
            Err(insertion_index) => insertion_index,
        };
        let raw = match self
            .entries
            .get(entry_index)
            .and_then(|entry| entry.raw.as_ref())
        {
            Some(CandidateRaw::Bytes(raw)) => raw.as_slice(),
            _ => {
                return Err(codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor opened-registry parse requires a captured byte candidate".to_string(),
                ))
            }
        };
        let raw_bytes = u64::try_from(raw.len()).map_err(|_| size_overflow())?;
        let temporary_bytes = json_parse_temporary_bytes(raw_bytes)?;
        let retained_upper = json_candidate_retained_upper(raw_bytes)?;
        self.retained_bytes
            .checked_add(retained_upper)
            .ok_or_else(size_overflow)?;
        self.parse_cleanup_temporary_bytes
            .checked_add(temporary_bytes)
            .ok_or_else(size_overflow)?;
        let phase = codefire_core::DecisionWorkPhase::RecordRead;
        let mut permit = ledger.reserve(
            codefire_core::DecisionWorkReservation::new(
                phase,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_upper,
            ),
        )?;
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            reservation(
                phase,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            ),
        ) {
            ledger.cancel_reserved(&permit)?;
            return Err(error.into());
        }
        if let Err(error) = poll(CandidateParseStage::BeforeMaterialization, ledger) {
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
        let parsed = serde_json::from_slice::<Value>(raw);
        if let Err(error) = poll(CandidateParseStage::AfterParseAttempt, ledger) {
            drop(parsed);
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
        let value = match parsed {
            Ok(value) => value,
            Err(error) => {
                let message = match materialize_json_error(&error) {
                    Ok(message) => message,
                    Err(materialization_error) => {
                        drop(error);
                        ledger.cancel_reserved(&permit)?;
                        ledger.release(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            temporary_bytes,
                        )?;
                        return Err(materialization_error);
                    }
                };
                let message_bytes = match u64::try_from(message.len()) {
                    Ok(message_bytes) => message_bytes,
                    Err(_) => {
                        drop(message);
                        drop(error);
                        ledger.cancel_reserved(&permit)?;
                        ledger.release(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            temporary_bytes,
                        )?;
                        return Err(size_overflow());
                    }
                };
                let retained_bytes = match OWNER_FRAME_BYTES.checked_add(message_bytes) {
                    Some(retained_bytes) if retained_bytes <= retained_upper => retained_bytes,
                    _ => {
                        drop(message);
                        drop(error);
                        ledger.cancel_reserved(&permit)?;
                        ledger.release(
                            codefire_core::DecisionWorkDimension::TemporaryBytes,
                            temporary_bytes,
                        )?;
                        return Err(codefire_store::StoreError::InvalidSealedCommit(
                            "Doctor opened-registry JSON error exceeded its retained upper bound"
                                .to_string(),
                        ));
                    }
                };
                if let Err(reserve_error) = self.parsed_candidates.try_reserve(1) {
                    drop(message);
                    drop(error);
                    ledger.cancel_reserved(&permit)?;
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        temporary_bytes,
                    )?;
                    return Err(allocation(
                        "opened-registry parsed JSON error",
                        reserve_error,
                    ));
                }
                let next_retained = self
                    .retained_bytes
                    .checked_add(retained_bytes)
                    .expect("parse-error retained actual is bounded by the checked upper");
                drop(error);
                ledger.cancel_reserved(&permit)?;
                if let Err(accounting_error) = ledger.apply(reservation(
                    phase,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                )) {
                    drop(message);
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        temporary_bytes,
                    )?;
                    return Err(accounting_error.into());
                }
                return self.finish_parse_transition(
                    ParsedCandidateTransition {
                        entry_index,
                        insertion_index,
                        payload: Some(ParsedCandidatePayload::Error(message)),
                        retained_bytes,
                        temporary_bytes,
                    },
                    next_retained,
                    DoctorOpenedRegistryParseResult::Invalid,
                    ledger,
                    release_temporary,
                );
            }
        };
        let compact_bytes = match compact_json_size(&value) {
            Ok(compact_bytes) => compact_bytes,
            Err(error) => {
                drop(value);
                ledger.cancel_reserved(&permit)?;
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    temporary_bytes,
                )?;
                return Err(error);
            }
        };
        let retained_bytes = match OWNER_FRAME_BYTES.checked_add(compact_bytes) {
            Some(retained_bytes) => retained_bytes,
            None => {
                drop(value);
                ledger.cancel_reserved(&permit)?;
                ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    temporary_bytes,
                )?;
                return Err(size_overflow());
            }
        };
        if retained_bytes > retained_upper {
            drop(value);
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor opened-registry parsed JSON exceeded its retained upper bound".to_string(),
            ));
        }
        if let Err(error) = self.parsed_candidates.try_reserve(1) {
            drop(value);
            ledger.cancel_reserved(&permit)?;
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(allocation("opened-registry parsed JSON", error));
        }
        let next_retained = self
            .retained_bytes
            .checked_add(retained_bytes)
            .expect("parsed retained actual is bounded by the checked upper");
        ledger.cancel_reserved(&permit)?;
        if let Err(error) = ledger.apply(reservation(
            phase,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        )) {
            drop(value);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error.into());
        }
        self.finish_parse_transition(
            ParsedCandidateTransition {
                entry_index,
                insertion_index,
                payload: Some(ParsedCandidatePayload::Value(value)),
                retained_bytes,
                temporary_bytes,
            },
            next_retained,
            DoctorOpenedRegistryParseResult::Parsed,
            ledger,
            release_temporary,
        )
    }

    fn finish_parse_transition<R>(
        &mut self,
        mut transition: ParsedCandidateTransition,
        next_retained: u64,
        success: DoctorOpenedRegistryParseResult,
        ledger: &mut codefire_core::DecisionWorkLedger,
        release_temporary: R,
    ) -> Result<DoctorOpenedRegistryParseResult, codefire_store::StoreError>
    where
        R: FnOnce(
            &mut codefire_core::DecisionWorkLedger,
            u64,
        ) -> Result<(), codefire_store::StoreError>,
    {
        match release_temporary(ledger, transition.temporary_bytes) {
            Ok(()) => {
                transition.temporary_bytes = 0;
                let payload = transition
                    .payload
                    .take()
                    .expect("active parse transition retains its payload");
                self.parsed_candidates.insert(
                    transition.insertion_index,
                    ParsedCandidate {
                        entry_index: transition.entry_index,
                        payload,
                    },
                );
                transition.retained_bytes = 0;
                self.retained_bytes = next_retained;
                Ok(success)
            }
            Err(primary_error) => {
                drop(transition.payload.take());
                let mut cleanup_error = None;
                match ledger.release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    transition.retained_bytes,
                ) {
                    Ok(()) => transition.retained_bytes = 0,
                    Err(error) => {
                        self.retained_bytes = next_retained;
                        transition.retained_bytes = 0;
                        cleanup_error = Some(codefire_store::StoreError::from(error));
                    }
                }
                match ledger.release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    transition.temporary_bytes,
                ) {
                    Ok(()) => transition.temporary_bytes = 0,
                    Err(error) => {
                        self.parse_cleanup_temporary_bytes = self
                            .parse_cleanup_temporary_bytes
                            .checked_add(transition.temporary_bytes)
                            .expect("parse cleanup Temporary was checked before reservation");
                        transition.temporary_bytes = 0;
                        cleanup_error = Some(codefire_store::StoreError::from(error));
                    }
                }
                Ok(DoctorOpenedRegistryParseResult::CleanupFailed(
                    cleanup_error.unwrap_or(primary_error),
                ))
            }
        }
    }

    #[cfg(test)]
    fn candidate_paths(&self) -> impl Iterator<Item = &Path> {
        self.candidates().map(|candidate| candidate.path)
    }

    pub(super) fn revalidate(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        self.revalidate_with_hook(ledger, &mut |_| Ok(()))
    }

    fn revalidate_with_hook<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        self.revalidate_with_hooks(ledger, before_absolute_open, &mut |_| Ok(()))
    }

    #[cfg(test)]
    fn revalidate_with_read_hook<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_candidate_read: &mut F,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        self.revalidate_with_hooks(ledger, &mut |_| Ok(()), before_candidate_read)
    }

    fn revalidate_with_hooks<F, G>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
        before_candidate_read: &mut G,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        self.ensure_ledger(ledger)?;
        let result = self.revalidate_inner(ledger, before_absolute_open, before_candidate_read);
        self.release_payload_then_scalar(ledger)?;
        result
    }

    pub(super) fn cancel(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        self.ensure_ledger(ledger)?;
        self.release_payload_then_scalar(ledger)
    }

    fn ensure_ledger(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor opened-registry snapshot belongs to a different decision-work ledger"
                    .to_string(),
            ))
        }
    }

    fn revalidate_inner<F, G>(
        &self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
        before_candidate_read: &mut G,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        let capability_bytes = OWNER_FRAME_BYTES.checked_mul(2).ok_or_else(size_overflow)?;
        ledger.apply(reservation(
            codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            capability_bytes,
        ))?;
        let result =
            self.revalidate_capabilities(ledger, before_absolute_open, before_candidate_read);
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            capability_bytes,
        )?;
        result
    }

    fn revalidate_capabilities<F, G>(
        &self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        before_absolute_open: &mut F,
        before_candidate_read: &mut G,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
        G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    {
        let parent = self
            .parent
            .as_ref()
            .expect("active opened-registry snapshot retains its parent capability");
        self.parent_identity
            .ensure_directory(parent, "Doctor opened-registry parent")
            .map_err(|error| changed(&self.display_parent, error))?;
        let ambient_parent = open_absolute_dir_accounted(
            &self.display_parent,
            codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            ledger,
            before_absolute_open,
            |error| changed(&self.display_parent, error),
        )?;
        let ambient_parent_identity = StableFsIdentity::for_directory(&ambient_parent)
            .map_err(|error| changed(&self.display_parent, error))?;
        if ambient_parent_identity != self.parent_identity {
            return Err(changed(
                &self.display_parent,
                "opened-registry parent identity changed",
            ));
        }
        let current_root = match parent.open_dir_nofollow("opened") {
            Ok(directory) => Some(directory),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(changed(&self.display_root, error)),
        };
        let root_identity = current_root
            .as_ref()
            .map(StableFsIdentity::for_directory)
            .transpose()
            .map_err(|error| changed(&self.display_root, error))?;
        if root_identity != self.root_identity {
            return Err(changed(
                &self.display_root,
                "opened-registry root identity changed",
            ));
        }
        let mut current = Vec::new();
        let vector_bytes = OWNER_FRAME_BYTES;
        ledger.apply(reservation(
            codefire_core::DecisionWorkPhase::SnapshotRevalidation,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            vector_bytes,
        ))?;
        let mut temporary_bytes = vector_bytes;
        let collected = if let Some(root) = self.root.as_ref() {
            self.root_identity
                .expect("present opened-registry root retains identity")
                .ensure_directory(root, "Doctor opened-registry root")
                .map_err(|error| changed(&self.display_root, error))?;
            collect_entries(
                root,
                &self.display_root,
                codefire_core::DecisionWorkPhase::SnapshotRevalidation,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                ledger,
                &mut temporary_bytes,
                &mut current,
                &mut |_| Ok(()),
                before_candidate_read,
            )
        } else {
            Ok(())
        };
        if let Err(error) = collected {
            drop(current);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary_bytes,
            )?;
            return Err(error);
        }
        let matches = current == self.entries;
        drop(current);
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_bytes,
        )?;
        if !matches {
            return Err(changed(
                &self.display_root,
                "opened-registry membership or entry identity changed",
            ));
        }
        Ok(())
    }

    fn release_payload_then_scalar(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_store::StoreError> {
        let retained = self.retained_bytes;
        let temporary = self.parse_cleanup_temporary_bytes;
        drop(std::mem::take(&mut self.parsed_candidates));
        drop(std::mem::take(&mut self.entries));
        drop(self.root.take());
        drop(self.parent.take());
        drop(std::mem::take(&mut self.display_root));
        drop(std::mem::take(&mut self.display_parent));
        if temporary != 0 {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                temporary,
            )?;
            self.parse_cleanup_temporary_bytes = 0;
        }
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained,
        )?;
        self.retained_bytes = 0;
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}

impl Drop for DoctorOpenedRegistrySnapshot {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.parse_cleanup_temporary_bytes, 0,
            "Doctor opened-registry snapshot dropped with pending parse cleanup"
        );
        debug_assert_eq!(
            self.retained_bytes, 0,
            "Doctor opened-registry snapshot dropped without ledger cleanup"
        );
    }
}

fn absolute_open_temporary_upper(path: &Path) -> Result<u64, codefire_store::StoreError> {
    let path_bytes =
        u64::try_from(path.as_os_str().as_encoded_bytes().len()).map_err(|_| size_overflow())?;
    let mut component_count = 0_u64;
    let mut component_bytes = 0_u64;
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                component_count = component_count.checked_add(1).ok_or_else(size_overflow)?;
                component_bytes = component_bytes
                    .checked_add(
                        u64::try_from(value.as_encoded_bytes().len())
                            .map_err(|_| size_overflow())?,
                    )
                    .and_then(|value| value.checked_add(1))
                    .ok_or_else(size_overflow)?;
            }
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                return Err(codefire_store::StoreError::Io(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "secure directory boundary contains parent traversal",
                )));
            }
        }
    }
    let frames = component_count
        .checked_add(2)
        .and_then(|value| value.checked_add(ABSOLUTE_OPEN_TRAVERSAL_HANDLES))
        .ok_or_else(size_overflow)?;
    OWNER_FRAME_BYTES
        .checked_mul(frames)
        .and_then(|value| value.checked_add(path_bytes))
        .and_then(|value| value.checked_add(1))
        .and_then(|value| value.checked_add(component_bytes))
        .ok_or_else(size_overflow)
}

fn open_absolute_dir_accounted<F, E>(
    path: &Path,
    phase: codefire_core::DecisionWorkPhase,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_open: &mut F,
    map_open_error: E,
) -> Result<Dir, codefire_store::StoreError>
where
    F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    E: FnOnce(io::Error) -> codefire_store::StoreError,
{
    let upper = absolute_open_temporary_upper(path)?;
    let mut permit = ledger.reserve(reservation(
        phase,
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        upper,
    ))?;
    if let Err(error) = ledger.consume_reserved(
        &mut permit,
        reservation(
            phase,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            upper,
        ),
    ) {
        ledger.cancel_reserved(&permit)?;
        return Err(error.into());
    }
    let opened =
        before_open(path).and_then(|()| open_absolute_dir_nofollow(path).map_err(map_open_error));
    ledger.release(codefire_core::DecisionWorkDimension::TemporaryBytes, upper)?;
    opened
}

struct DirectoryWork {
    display_path: PathBuf,
    directory: Dir,
    temporary_bytes: u64,
}

enum FixedRawRead {
    Complete(Vec<u8>),
    Changed {
        error: codefire_store::StoreError,
        bytes_read: u64,
    },
}

#[cfg(unix)]
fn single_link_unavailable(metadata: &cap_std::fs::Metadata) -> Option<CandidateRawUnavailable> {
    use cap_std::fs::MetadataExt;
    (metadata.nlink() != 1).then_some(CandidateRawUnavailable::MultipleLinks)
}

#[cfg(windows)]
fn single_link_unavailable(metadata: &cap_std::fs::Metadata) -> Option<CandidateRawUnavailable> {
    use cap_std::fs::MetadataExt;
    match metadata.number_of_links() {
        Some(1) => None,
        Some(_) => Some(CandidateRawUnavailable::MultipleLinks),
        None => Some(CandidateRawUnavailable::UnprovableLinks),
    }
}

#[cfg(not(any(unix, windows)))]
fn single_link_unavailable(_metadata: &cap_std::fs::Metadata) -> Option<CandidateRawUnavailable> {
    Some(CandidateRawUnavailable::UnprovableLinks)
}

#[allow(clippy::too_many_arguments)]
fn read_candidate_raw<F>(
    directory: &Dir,
    name: &OsStr,
    display_path: &Path,
    expected_identity: StableFsIdentity,
    size_bytes: u64,
    phase: codefire_core::DecisionWorkPhase,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_read: F,
) -> Result<CandidateRaw, codefire_store::StoreError>
where
    F: FnOnce() -> Result<(), codefire_store::StoreError>,
{
    if size_bytes > REGISTRY_RECORD_LIMIT {
        return Ok(CandidateRaw::Unavailable(
            CandidateRawUnavailable::Rejected { size: size_bytes },
        ));
    }
    let read_upper = size_bytes
        .checked_add(BOUNDED_READ_SENTINEL_BYTES)
        .ok_or_else(size_overflow)?;
    let mut permit = ledger.reserve(raw_read_work(phase, read_upper))?;
    if let Err(error) = ledger.apply(reservation(
        phase,
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        OWNER_FRAME_BYTES,
    )) {
        ledger.cancel_reserved(&permit)?;
        return Err(error.into());
    }
    let opened = (|| {
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let file = match directory.open_with(name, &options) {
            Ok(file) => file,
            Err(error) => {
                return Ok(Err(CandidateRawUnavailable::Open(
                    CandidateIoEvidence::capture(&error),
                )));
            }
        };
        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                return Ok(Err(CandidateRawUnavailable::Metadata(
                    CandidateIoEvidence::capture(&error),
                )));
            }
        };
        if !metadata.is_file() || metadata.len() != size_bytes {
            return Err(changed(
                display_path,
                "opened-registry candidate changed before bounded read",
            ));
        }
        if let Some(reason) = single_link_unavailable(&metadata) {
            return Ok(Err(reason));
        }
        let identity = match StableFsIdentity::for_file(&file) {
            Ok(identity) => identity,
            Err(error) => {
                return Ok(Err(CandidateRawUnavailable::Identity(
                    CandidateIoEvidence::capture(&error),
                )));
            }
        };
        if identity != expected_identity {
            return Err(changed(
                display_path,
                "opened-registry candidate identity changed before bounded read",
            ));
        }
        Ok(Ok(file))
    })();
    let mut file = match opened {
        Ok(Ok(file)) => file,
        Ok(Err(reason)) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                OWNER_FRAME_BYTES,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Ok(CandidateRaw::Unavailable(reason));
        }
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                OWNER_FRAME_BYTES,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let fixed = match read_fixed_raw(&mut file, display_path, size_bytes, before_read) {
        Ok(fixed) => fixed,
        Err(error) => {
            drop(file);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                OWNER_FRAME_BYTES,
            )?;
            ledger.cancel_reserved(&permit)?;
            return Err(error);
        }
    };
    let bytes = match fixed {
        FixedRawRead::Complete(bytes) => bytes,
        FixedRawRead::Changed { error, bytes_read } => {
            ledger.consume_reserved(&mut permit, raw_read_work(phase, bytes_read))?;
            if bytes_read < read_upper {
                ledger.cancel_reserved(&permit)?;
            }
            drop(file);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                OWNER_FRAME_BYTES,
            )?;
            return Err(error);
        }
    };
    if let Err(error) = expected_identity.ensure_file(&file, "Doctor opened-registry candidate") {
        drop(bytes);
        drop(file);
        ledger.consume_reserved(&mut permit, raw_read_work(phase, size_bytes))?;
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            OWNER_FRAME_BYTES,
        )?;
        return Err(changed(display_path, error));
    }
    let final_unavailable = match file.metadata() {
        Ok(metadata) => single_link_unavailable(&metadata),
        Err(error) => Some(CandidateRawUnavailable::Metadata(
            CandidateIoEvidence::capture(&error),
        )),
    };
    if let Some(reason) = final_unavailable {
        drop(bytes);
        drop(file);
        ledger.consume_reserved(&mut permit, raw_read_work(phase, size_bytes))?;
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            OWNER_FRAME_BYTES,
        )?;
        return Ok(CandidateRaw::Unavailable(reason));
    }
    if let Err(error) = ledger.consume_reserved(&mut permit, raw_read_work(phase, size_bytes)) {
        drop(bytes);
        drop(file);
        ledger.cancel_reserved(&permit)?;
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            OWNER_FRAME_BYTES,
        )?;
        return Err(error.into());
    }
    ledger.cancel_reserved(&permit)?;
    drop(file);
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        OWNER_FRAME_BYTES,
    )?;
    Ok(CandidateRaw::Bytes(bytes))
}

fn read_fixed_raw<F>(
    file: &mut cap_std::fs::File,
    display_path: &Path,
    size_bytes: u64,
    before_read: F,
) -> Result<FixedRawRead, codefire_store::StoreError>
where
    F: FnOnce() -> Result<(), codefire_store::StoreError>,
{
    let size = usize::try_from(size_bytes).map_err(|_| size_overflow())?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|error| allocation("opened-registry raw record", error))?;
    bytes.resize(size, 0);
    before_read()?;
    let mut offset = 0_usize;
    while offset < size {
        match file.read(&mut bytes[offset..]) {
            Ok(0) => {
                return Ok(FixedRawRead::Changed {
                    error: changed(display_path, "candidate changed during fixed-size read"),
                    bytes_read: u64::try_from(offset).map_err(|_| size_overflow())?,
                });
            }
            Ok(read) => offset = offset.checked_add(read).ok_or_else(size_overflow)?,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Ok(FixedRawRead::Changed {
                    error: changed(display_path, error),
                    bytes_read: u64::try_from(offset).map_err(|_| size_overflow())?,
                });
            }
        }
    }
    let mut sentinel = [0_u8; 1];
    let trailing = loop {
        match file.read(&mut sentinel) {
            Ok(read) => break read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Ok(FixedRawRead::Changed {
                    error: changed(display_path, error),
                    bytes_read: size_bytes,
                });
            }
        }
    };
    if trailing != 0 {
        return Ok(FixedRawRead::Changed {
            error: changed(display_path, "candidate grew during fixed-size read"),
            bytes_read: size_bytes
                .checked_add(u64::try_from(trailing).map_err(|_| size_overflow())?)
                .ok_or_else(size_overflow)?,
        });
    }
    Ok(FixedRawRead::Complete(bytes))
}

fn raw_read_work(
    phase: codefire_core::DecisionWorkPhase,
    bytes_read: u64,
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
    .count(codefire_core::DecisionWorkCounter::RecordReads, 1);
    if phase == codefire_core::DecisionWorkPhase::SnapshotRevalidation {
        reservation.charge(
            codefire_core::DecisionWorkDimension::SnapshotRevalidationBytes,
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
fn collect_entries<F, G>(
    root_directory: &Dir,
    root: &Path,
    phase: codefire_core::DecisionWorkPhase,
    dimension: codefire_core::DecisionWorkDimension,
    ledger: &mut codefire_core::DecisionWorkLedger,
    owned_bytes: &mut u64,
    entries: &mut Vec<EntryEvidence>,
    before_entry_slot: &mut F,
    before_candidate_read: &mut G,
) -> Result<(), codefire_store::StoreError>
where
    F: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
    G: FnMut(&Path) -> Result<(), codefire_store::StoreError>,
{
    let root_path_bytes = root.as_os_str().as_encoded_bytes().len();
    let root_work_bytes = OWNER_FRAME_BYTES
        .checked_mul(2)
        .and_then(|value| value.checked_add(u64::try_from(root_path_bytes).ok()?))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(size_overflow)?;
    let queue_base = OWNER_FRAME_BYTES
        .checked_add(root_work_bytes)
        .ok_or_else(size_overflow)?;
    ledger.apply(reservation(
        phase,
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        queue_base,
    ))?;
    let mut queue: Vec<DirectoryWork> = Vec::new();
    if let Err(error) = queue.try_reserve(1) {
        ledger.release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            queue_base,
        )?;
        return Err(allocation("opened-registry traversal queue", error));
    }
    let root_copy = match copy_path(root, root_path_bytes) {
        Ok(path) => path,
        Err(error) => {
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                queue_base,
            )?;
            return Err(error);
        }
    };
    let root_clone = match root_directory.try_clone() {
        Ok(directory) => directory,
        Err(error) => {
            drop(root_copy);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                queue_base,
            )?;
            return Err(error.into());
        }
    };
    queue.push(DirectoryWork {
        display_path: root_copy,
        directory: root_clone,
        temporary_bytes: root_work_bytes,
    });
    let mut queue_bytes = queue_base;
    let result = (|| {
        while let Some(work) = queue.pop() {
            ledger.apply(reservation(
                phase,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            ))?;
            let directory_entries = work.directory.entries();
            let mut directory_entries = match directory_entries {
                Ok(entries) => entries,
                Err(error) => {
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        DIRECTORY_ITERATION_SCRATCH_BYTES,
                    )?;
                    return Err(error.into());
                }
            };
            let directory_result = (|| {
                for entry in &mut directory_entries {
                    let entry = entry?;
                    ledger.apply(entry_work(phase))?;
                    let name_source_bytes = OWNER_FRAME_BYTES
                        .checked_add(COMPONENT_SOURCE_UPPER_BYTES)
                        .ok_or_else(size_overflow)?;
                    ledger.apply(reservation(
                        phase,
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        name_source_bytes,
                    ))?;
                    let entry_result = (|| {
                        let name = entry.file_name();
                        let name_bytes = name.as_encoded_bytes().len();
                        if u64::try_from(name_bytes).map_err(|_| size_overflow())?
                            > COMPONENT_SOURCE_UPPER_BYTES
                        {
                            return Err(codefire_store::StoreError::Io(io::Error::other(
                                "opened-registry component exceeds bounded source ownership",
                            )));
                        }
                        let metadata = work.directory.symlink_metadata(&name)?;
                        let kind = entry_kind(&metadata);
                        let json_candidate = kind == EntryKind::Regular && is_json(&name);
                        let path_upper = work
                            .display_path
                            .as_os_str()
                            .as_encoded_bytes()
                            .len()
                            .checked_add(1)
                            .and_then(|value| value.checked_add(name_bytes))
                            .ok_or_else(size_overflow)?;
                        let path_owner_upper = OWNER_FRAME_BYTES
                            .checked_add(u64::try_from(path_upper).map_err(|_| size_overflow())?)
                            .and_then(|value| value.checked_add(1))
                            .ok_or_else(size_overflow)?;
                        let raw_owner_upper = (json_candidate
                            && metadata.len() <= REGISTRY_RECORD_LIMIT)
                            .then(|| {
                                OWNER_FRAME_BYTES
                                    .checked_add(metadata.len())
                                    .ok_or_else(size_overflow)
                            })
                            .transpose()?
                            .unwrap_or(0);
                        let upper_bytes = path_owner_upper
                            .checked_add(raw_owner_upper)
                            .ok_or_else(size_overflow)?;
                        let child_work_upper = (kind == EntryKind::Directory)
                            .then(|| {
                                OWNER_FRAME_BYTES
                                    .checked_mul(2)
                                    .and_then(|value| {
                                        value.checked_add(u64::try_from(path_upper).ok()?)
                                    })
                                    .and_then(|value| value.checked_add(1))
                                    .ok_or_else(size_overflow)
                            })
                            .transpose()?;
                        let mut child_permit = if let Some(child_work_upper) = child_work_upper {
                            Some(ledger.reserve(reservation(
                                phase,
                                codefire_core::DecisionWorkDimension::TemporaryBytes,
                                child_work_upper,
                            ))?)
                        } else {
                            None
                        };
                        let upper_reservation = reservation(phase, dimension, upper_bytes);
                        let mut permit = match ledger.reserve(upper_reservation) {
                            Ok(permit) => permit,
                            Err(error) => {
                                if let Some(child_permit) = child_permit.as_ref() {
                                    ledger.cancel_reserved(child_permit)?;
                                }
                                return Err(error.into());
                            }
                        };
                        let materialized = (|| {
                            let mut path = PathBuf::new();
                            path.as_mut_os_string()
                                .try_reserve_exact(path_upper)
                                .map_err(|error| {
                                    allocation("opened-registry candidate path", error)
                                })?;
                            path.push(&work.display_path);
                            path.push(&name);
                            let identity = stable_metadata_identity(&metadata)?;
                            let child_directory = if kind == EntryKind::Directory {
                                let child = work.directory.open_dir_nofollow(&name)?;
                                if StableFsIdentity::for_directory(&child)? != identity {
                                    return Err(changed(
                                        &path,
                                        "opened-registry directory changed during capture",
                                    ));
                                }
                                Some(child)
                            } else {
                                None
                            };
                            let raw = if json_candidate {
                                Some(read_candidate_raw(
                                    &work.directory,
                                    &name,
                                    &path,
                                    identity,
                                    metadata.len(),
                                    phase,
                                    ledger,
                                    || before_candidate_read(&path),
                                )?)
                            } else {
                                None
                            };
                            let actual_bytes = OWNER_FRAME_BYTES
                                .checked_add(
                                    u64::try_from(path.as_os_str().as_encoded_bytes().len())
                                        .map_err(|_| size_overflow())?,
                                )
                                .and_then(|value| value.checked_add(1))
                                .and_then(|value| {
                                    raw.as_ref().map_or(Some(value), |raw| match raw {
                                        CandidateRaw::Bytes(raw) => value
                                            .checked_add(OWNER_FRAME_BYTES)?
                                            .checked_add(u64::try_from(raw.len()).ok()?),
                                        CandidateRaw::Unavailable(_) => Some(value),
                                    })
                                })
                                .ok_or_else(size_overflow)?;
                            let next_owned = owned_bytes
                                .checked_add(actual_bytes)
                                .ok_or_else(size_overflow)?;
                            before_entry_slot(&path)?;
                            entries
                                .try_reserve(1)
                                .map_err(|error| allocation("opened-registry membership", error))?;
                            Ok::<_, codefire_store::StoreError>((
                                EntryEvidence {
                                    path,
                                    kind,
                                    identity,
                                    size_bytes: (kind == EntryKind::Regular)
                                        .then_some(metadata.len()),
                                    json_candidate,
                                    raw,
                                },
                                actual_bytes,
                                next_owned,
                                child_directory,
                            ))
                        })();
                        let (evidence, actual_bytes, next_owned, child_directory) =
                            match materialized {
                                Ok(materialized) => materialized,
                                Err(error) => {
                                    ledger.cancel_reserved(&permit)?;
                                    if let Some(child_permit) = child_permit.as_ref() {
                                        ledger.cancel_reserved(child_permit)?;
                                    }
                                    return Err(error);
                                }
                            };
                        if let Err(error) = ledger.consume_reserved(
                            &mut permit,
                            reservation(phase, dimension, actual_bytes),
                        ) {
                            drop(evidence);
                            ledger.cancel_reserved(&permit)?;
                            if let Some(child_permit) = child_permit.as_ref() {
                                ledger.cancel_reserved(child_permit)?;
                            }
                            return Err(error.into());
                        }
                        if actual_bytes < upper_bytes {
                            ledger.cancel_reserved(&permit)?;
                        }
                        let index = entries.len();
                        entries.push(evidence);
                        *owned_bytes = next_owned;
                        if let Some(child_directory) = child_directory {
                            let child_path = &entries[index].path;
                            let child_path_bytes = child_path.as_os_str().as_encoded_bytes().len();
                            let child_work_bytes = OWNER_FRAME_BYTES
                                .checked_mul(2)
                                .and_then(|value| {
                                    value.checked_add(u64::try_from(child_path_bytes).ok()?)
                                })
                                .and_then(|value| value.checked_add(1))
                                .ok_or_else(size_overflow)?;
                            let next_queue_bytes = queue_bytes
                                .checked_add(child_work_bytes)
                                .ok_or_else(size_overflow)?;
                            let child_display_path = match copy_path(child_path, child_path_bytes) {
                                Ok(path) => path,
                                Err(error) => {
                                    drop(child_directory);
                                    ledger.cancel_reserved(child_permit.as_ref().expect(
                                        "directory entry reserves its child work before open",
                                    ))?;
                                    return Err(error);
                                }
                            };
                            if let Err(error) = queue.try_reserve(1) {
                                drop(child_display_path);
                                drop(child_directory);
                                ledger.cancel_reserved(child_permit.as_ref().expect(
                                    "directory entry reserves its child work before open",
                                ))?;
                                return Err(allocation("opened-registry traversal queue", error));
                            }
                            let child_permit = child_permit
                                .as_mut()
                                .expect("directory entry reserves its child work before open");
                            if let Err(error) = ledger.consume_reserved(
                                child_permit,
                                reservation(
                                    phase,
                                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                                    child_work_bytes,
                                ),
                            ) {
                                drop(child_display_path);
                                drop(child_directory);
                                ledger.cancel_reserved(child_permit)?;
                                return Err(error.into());
                            }
                            if child_work_bytes
                                < child_work_upper.expect(
                                    "directory entry computes its reserved child-work upper",
                                )
                            {
                                ledger.cancel_reserved(child_permit)?;
                            }
                            queue.push(DirectoryWork {
                                display_path: child_display_path,
                                directory: child_directory,
                                temporary_bytes: child_work_bytes,
                            });
                            queue_bytes = next_queue_bytes;
                        } else if let Some(child_permit) = child_permit.as_ref() {
                            ledger.cancel_reserved(child_permit)?;
                        }
                        Ok::<_, codefire_store::StoreError>(())
                    })();
                    ledger.release(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        name_source_bytes,
                    )?;
                    entry_result?;
                }
                Ok::<_, codefire_store::StoreError>(())
            })();
            drop(directory_entries);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DIRECTORY_ITERATION_SCRATCH_BYTES,
            )?;
            let work_bytes = work.temporary_bytes;
            drop(work);
            ledger.release(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                work_bytes,
            )?;
            queue_bytes = queue_bytes
                .checked_sub(work_bytes)
                .ok_or_else(size_overflow)?;
            directory_result?;
        }
        entries.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok::<_, codefire_store::StoreError>(())
    })();
    drop(queue);
    ledger.release(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        queue_bytes,
    )?;
    result
}

fn reservation(
    phase: codefire_core::DecisionWorkPhase,
    dimension: codefire_core::DecisionWorkDimension,
    amount: u64,
) -> codefire_core::DecisionWorkReservation {
    codefire_core::DecisionWorkReservation::new(phase, codefire_core::DecisionWorkSurface::Atomic)
        .acquire(dimension, amount)
}

fn entry_work(phase: codefire_core::DecisionWorkPhase) -> codefire_core::DecisionWorkReservation {
    if phase == codefire_core::DecisionWorkPhase::SnapshotRevalidation {
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(
            codefire_core::DecisionWorkDimension::SnapshotRevalidationRecords,
            1,
        )
        .count(
            codefire_core::DecisionWorkCounter::SnapshotRevalidationRecords,
            1,
        )
    } else {
        codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1)
        .count(codefire_core::DecisionWorkCounter::ObjectRecords, 1)
    }
}

fn copy_path(path: &Path, bytes: usize) -> Result<PathBuf, codefire_store::StoreError> {
    let mut copy = PathBuf::new();
    copy.as_mut_os_string()
        .try_reserve_exact(bytes)
        .map_err(|error| allocation("opened-registry snapshot path", error))?;
    copy.push(path);
    Ok(copy)
}

fn entry_kind(metadata: &cap_std::fs::Metadata) -> EntryKind {
    let file_type = metadata.file_type();
    if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::Regular
    } else if file_type.is_symlink() {
        EntryKind::Symlink
    } else {
        EntryKind::Special
    }
}

fn is_json(name: &OsStr) -> bool {
    Path::new(name).extension().and_then(|value| value.to_str()) == Some("json")
}

fn changed(path: &Path, detail: impl std::fmt::Display) -> codefire_store::StoreError {
    codefire_store::StoreError::InvalidSealedCommit(format!(
        "Doctor opened-registry snapshot changed at {}: {detail}",
        path.display()
    ))
}

fn allocation(
    subject: &str,
    error: std::collections::TryReserveError,
) -> codefire_store::StoreError {
    io::Error::other(format!("{subject} allocation failed: {error}")).into()
}

#[derive(Default)]
struct JsonByteCounter(u64);

impl io::Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| io::Error::other("opened-registry JSON size overflow"))?;
        self.0 = self
            .0
            .checked_add(amount)
            .ok_or_else(|| io::Error::other("opened-registry JSON size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn compact_json_size(value: &Value) -> Result<u64, codefire_store::StoreError> {
    let mut counter = JsonByteCounter::default();
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.0)
}

fn json_parse_temporary_bytes(raw_bytes: u64) -> Result<u64, codefire_store::StoreError> {
    OWNER_FRAME_BYTES
        .checked_add(
            raw_bytes
                .checked_mul(JSON_PARSE_SCRATCH_MULTIPLIER)
                .ok_or_else(size_overflow)?,
        )
        .ok_or_else(size_overflow)
}

fn json_candidate_retained_upper(raw_bytes: u64) -> Result<u64, codefire_store::StoreError> {
    let value_upper = OWNER_FRAME_BYTES
        .checked_add(
            raw_bytes
                .checked_mul(JSON_VALUE_RETAINED_UPPER_MULTIPLIER)
                .ok_or_else(size_overflow)?,
        )
        .and_then(|value| value.checked_add(JSON_VALUE_RETAINED_SLACK_BYTES))
        .ok_or_else(size_overflow)?;
    Ok(value_upper.max(JSON_ERROR_RETAINED_UPPER_BYTES))
}

#[derive(Default)]
struct FormatByteCounter(usize);

impl fmt::Write for FormatByteCounter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0 = self.0.checked_add(value.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}

fn materialize_json_error(error: &serde_json::Error) -> Result<String, codefire_store::StoreError> {
    let mut counter = FormatByteCounter::default();
    write!(&mut counter, "{error}").map_err(|_| size_overflow())?;
    let mut message = String::new();
    message
        .try_reserve_exact(counter.0)
        .map_err(|error| allocation("opened-registry JSON error", error))?;
    write!(&mut message, "{error}").expect("reserved JSON error formatting is infallible");
    if message.len() != counter.0 {
        return Err(codefire_store::StoreError::InvalidSealedCommit(
            "Doctor opened-registry JSON error changed while formatting".to_string(),
        ));
    }
    Ok(message)
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
            "filesystem identity cannot be proven",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn stable_metadata_identity(_metadata: &cap_std::fs::Metadata) -> io::Result<StableFsIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "stable filesystem identity is unavailable on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn ledger(command: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            command,
        )
        .unwrap()
    }

    fn assert_zero(ledger: &codefire_core::DecisionWorkLedger) {
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
    }

    fn deep_parent(base: &Path) -> PathBuf {
        let mut path = base.to_path_buf();
        for index in 0..16 {
            path.push(format!("segment_{index:02}_abcdefghij"));
        }
        path.push(".codefire");
        path
    }

    #[test]
    fn snapshot_preserves_recursive_sorted_candidates_and_releases_after_fence() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("z.json"), b"{}").unwrap();
        fs::write(root.join("nested/a.json"), b"{}").unwrap();
        fs::write(root.join("ignored.txt"), b"{}").unwrap();
        let mut ledger = ledger("opened snapshot recursive order");

        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        assert_eq!(
            snapshot.candidate_paths().collect::<Vec<_>>(),
            vec![root.join("nested/a.json"), root.join("z.json")]
        );
        assert!(snapshot
            .candidates()
            .all(|candidate| candidate.source == DoctorOpenedRegistrySource::Bytes(b"{}")));
        snapshot.revalidate(&mut ledger).unwrap();
        assert_zero(&ledger);
    }

    #[test]
    fn snapshot_rejects_membership_and_identity_replacement_and_cleans_up() {
        for replacement in [false, true] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join(".codefire");
            let root = cf.join("opened");
            fs::create_dir_all(&root).unwrap();
            let candidate = root.join("main.json");
            fs::write(&candidate, b"{}").unwrap();
            let mut ledger = ledger("opened snapshot changed subject");
            let mut snapshot =
                DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
            if replacement {
                let original = root.join("original.json");
                fs::rename(&candidate, &original).unwrap();
                fs::write(&candidate, b"{}").unwrap();
                fs::remove_file(original).unwrap();
            } else {
                fs::write(root.join("added.json"), b"{}").unwrap();
            }
            assert!(matches!(
                snapshot.revalidate(&mut ledger),
                Err(codefire_store::StoreError::InvalidSealedCommit(message))
                    if message.contains("snapshot changed")
            ));
            assert_zero(&ledger);
        }
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_root_symlink_without_traversing_its_target() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&cf).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("external.json"), b"{}").unwrap();
        symlink(&outside, cf.join("opened")).unwrap();
        let mut ledger = ledger("opened snapshot root symlink");

        assert!(
            DoctorOpenedRegistrySnapshot::capture(&cf, &cf.join("opened"), &mut ledger).is_err()
        );
        assert_eq!(
            ledger
                .snapshot()
                .counters
                .get(&codefire_core::DecisionWorkCounter::ObjectRecords),
            None
        );
        assert_zero(&ledger);
    }

    #[test]
    fn snapshot_pins_nested_directory_before_replacement() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("original.json"), b"{}").unwrap();
        let moved = root.join("moved");
        let mut swapped = false;
        let mut ledger = ledger("opened snapshot nested replacement");

        let mut snapshot =
            DoctorOpenedRegistrySnapshot::capture_with_hook(&cf, &root, &mut ledger, |path| {
                if path == nested && !swapped {
                    fs::rename(&nested, &moved)?;
                    fs::create_dir(&nested)?;
                    for index in 0..32 {
                        fs::write(nested.join(format!("external-{index:02}.json")), b"{}")?;
                    }
                    swapped = true;
                }
                Ok(())
            })
            .unwrap();
        assert!(swapped);
        assert_eq!(
            snapshot.candidate_paths().collect::<Vec<_>>(),
            vec![nested.join("original.json")]
        );
        assert_eq!(
            snapshot.candidates().next().unwrap().source,
            DoctorOpenedRegistrySource::Bytes(b"{}")
        );
        assert!(matches!(
            snapshot.revalidate(&mut ledger),
            Err(codefire_store::StoreError::InvalidSealedCommit(message))
                if message.contains("snapshot changed")
        ));
        assert_zero(&ledger);
    }

    #[test]
    fn snapshot_raw_source_ignores_ambient_root_replacement() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), b"{\"source\":\"original\"}").unwrap();
        let mut ledger = ledger("opened snapshot raw root replacement");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();

        let moved = cf.join("opened-original");
        fs::rename(&root, &moved).unwrap();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.json"), b"{\"source\":\"replacement\"}").unwrap();
        assert_eq!(
            snapshot.candidates().next().unwrap().source,
            DoctorOpenedRegistrySource::Bytes(b"{\"source\":\"original\"}")
        );
        assert!(matches!(
            snapshot.revalidate(&mut ledger),
            Err(codefire_store::StoreError::InvalidSealedCommit(message))
                if message.contains("root identity changed")
        ));
        assert_zero(&ledger);
    }

    #[test]
    fn snapshot_raw_fence_rejects_same_inode_rewrite_growth_and_read_race() {
        for replacement in [b"{\"value\":1}".as_slice(), b"{\"value\":100}".as_slice()] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join(".codefire");
            let root = cf.join("opened");
            fs::create_dir_all(&root).unwrap();
            let candidate = root.join("main.json");
            fs::write(&candidate, b"{\"value\":0}").unwrap();
            let mut ledger = ledger("opened snapshot raw content fence");
            let mut snapshot =
                DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
            fs::write(&candidate, replacement).unwrap();
            assert!(matches!(
                snapshot.revalidate(&mut ledger),
                Err(codefire_store::StoreError::InvalidSealedCommit(message))
                    if message.contains("membership or entry identity changed")
            ));
            assert_zero(&ledger);
        }

        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        let candidate = root.join("main.json");
        fs::write(&candidate, b"{}").unwrap();
        let mut ledger = ledger("opened snapshot raw growth during read");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        let mut injected = false;
        let result = snapshot.revalidate_with_read_hook(&mut ledger, &mut |path| {
            if path == candidate {
                use std::io::Write as _;
                fs::OpenOptions::new()
                    .append(true)
                    .open(&candidate)?
                    .write_all(b"x")?;
                injected = true;
            }
            Ok(())
        });
        assert!(injected);
        assert!(
            matches!(
                result,
                Err(codefire_store::StoreError::InvalidSealedCommit(ref message))
                    if message.contains("grew during fixed-size read")
            ),
            "unexpected growth result: {result:?}"
        );
        assert_zero(&ledger);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn snapshot_raw_fence_rejects_hardlink_added_after_capture() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        let candidate = root.join("main.json");
        fs::write(&candidate, b"{}").unwrap();
        let mut ledger = ledger("opened snapshot hardlink added after capture");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();

        fs::hard_link(&candidate, temp.path().join("outside-link.json")).unwrap();
        assert!(matches!(
            snapshot.revalidate(&mut ledger),
            Err(codefire_store::StoreError::InvalidSealedCommit(message))
                if message.contains("membership or entry identity changed")
        ));
        assert_zero(&ledger);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn snapshot_raw_fence_rechecks_link_authority_after_final_read_hook() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        let candidate = root.join("main.json");
        fs::write(&candidate, b"{}").unwrap();
        let mut ledger = ledger("opened snapshot hardlink during final read");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        let outside = temp.path().join("outside-link.json");
        let mut injected = false;

        let result = snapshot.revalidate_with_read_hook(&mut ledger, &mut |path| {
            if path == candidate {
                fs::hard_link(&candidate, &outside)?;
                injected = true;
            }
            Ok(())
        });

        assert!(injected);
        assert!(matches!(
            result,
            Err(codefire_store::StoreError::InvalidSealedCommit(message))
                if message.contains("membership or entry identity changed")
        ));
        assert_zero(&ledger);
    }

    #[test]
    fn snapshot_foreign_error_and_cancel_paths_leave_no_owner_drift() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), b"{}").unwrap();

        let mut owner = ledger("opened snapshot owner");
        let mut foreign = ledger("opened snapshot foreign");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        assert!(snapshot.cancel(&mut foreign).is_err());
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        snapshot.cancel(&mut owner).unwrap();
        assert_zero(&owner);
        assert_zero(&foreign);

        let mut entry_error_ledger = ledger("opened snapshot injected allocation error");
        let error = DoctorOpenedRegistrySnapshot::capture_with_hook(
            &cf,
            &root,
            &mut entry_error_ledger,
            |_| {
                Err(codefire_store::StoreError::Io(io::Error::other(
                    "injected opened snapshot allocation error",
                )))
            },
        )
        .err()
        .unwrap();
        assert!(
            matches!(error, codefire_store::StoreError::Io(error) if error.to_string().contains("injected"))
        );
        assert_eq!(
            entry_error_ledger
                .snapshot()
                .counters
                .get(&codefire_core::DecisionWorkCounter::ObjectRecords),
            Some(&1)
        );
        assert_zero(&entry_error_ledger);

        let mut open_hook_ledger = ledger("opened snapshot absolute-open hook error");
        let error = DoctorOpenedRegistrySnapshot::capture_with_hooks(
            &cf,
            &root,
            &mut open_hook_ledger,
            &mut |_| {
                Err(codefire_store::StoreError::Io(io::Error::other(
                    "injected absolute-open error",
                )))
            },
            &mut |_| Ok(()),
        )
        .err()
        .unwrap();
        assert!(
            matches!(error, codefire_store::StoreError::Io(error) if error.to_string().contains("absolute-open"))
        );
        assert_zero(&open_hook_ledger);

        let missing = temp.path().join("missing").join(".codefire");
        let mut open_error_ledger = ledger("opened snapshot absolute-open io error");
        assert!(DoctorOpenedRegistrySnapshot::capture(
            &missing,
            &missing.join("opened"),
            &mut open_error_ledger,
        )
        .is_err());
        assert_zero(&open_error_ledger);
    }

    #[test]
    fn parsed_candidate_reserves_before_materialization_and_stays_with_snapshot() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        let raw = br#"{"branch":{"name":"main"},"open":{}}"#;
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), raw).unwrap();
        let mut ledger = ledger("opened parsed candidate ownership");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        let retained_before =
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes];
        let temporary_bytes = json_parse_temporary_bytes(raw.len() as u64).unwrap();
        let mut stages = Vec::new();

        let outcome = snapshot
            .parse_candidate_with_hook(0, &mut ledger, |stage, ledger| {
                stages.push(stage);
                assert!(ledger.has_open_permits());
                assert_eq!(
                    ledger.snapshot().current
                        [&codefire_core::DecisionWorkDimension::TemporaryBytes],
                    temporary_bytes
                );
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    retained_before
                );
                Ok(())
            })
            .unwrap();

        assert!(matches!(outcome, DoctorOpenedRegistryParseResult::Parsed));
        assert_eq!(
            stages,
            vec![
                CandidateParseStage::BeforeMaterialization,
                CandidateParseStage::AfterParseAttempt
            ]
        );
        assert_eq!(
            snapshot.parsed_candidate(0).unwrap().1,
            &serde_json::from_slice::<Value>(raw).unwrap()
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
        assert!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes]
                > retained_before
        );
        assert!(!ledger.has_open_permits());
        snapshot.cancel(&mut ledger).unwrap();
        assert_zero(&ledger);
    }

    #[test]
    fn parsed_candidate_parse_error_and_deadline_drop_all_parse_work() {
        for (raw, cancel_after_parse) in [(&b"{"[..], false), (&b"{}"[..], true)] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join(".codefire");
            let root = cf.join("opened");
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("main.json"), raw).unwrap();
            let mut ledger = ledger("opened parsed candidate failure cleanup");
            let mut snapshot =
                DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
            let before = ledger.snapshot().current.clone();

            let result = snapshot.parse_candidate_with_hook(0, &mut ledger, |stage, _| {
                if cancel_after_parse && stage == CandidateParseStage::AfterParseAttempt {
                    Err(codefire_store::StoreError::ValidationCancelled)
                } else {
                    Ok(())
                }
            });

            if cancel_after_parse {
                assert!(matches!(
                    result,
                    Err(codefire_store::StoreError::ValidationCancelled)
                ));
            } else {
                assert!(matches!(
                    result.unwrap(),
                    DoctorOpenedRegistryParseResult::Invalid
                ));
                assert_eq!(
                    snapshot.invalid_candidate(0).unwrap().1,
                    serde_json::from_slice::<Value>(raw)
                        .unwrap_err()
                        .to_string()
                );
            }
            assert!(snapshot.parsed_candidate(0).is_none());
            if cancel_after_parse {
                assert!(snapshot.invalid_candidate(0).is_none());
                assert_eq!(ledger.snapshot().current, before);
            } else {
                assert!(snapshot.invalid_candidate(0).is_some());
                assert!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes]
                        > before[&codefire_core::DecisionWorkDimension::RetainedBytes]
                );
                assert_eq!(
                    ledger
                        .snapshot()
                        .current
                        .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                        .copied()
                        .unwrap_or_default(),
                    before
                        .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                        .copied()
                        .unwrap_or_default()
                );
            }
            assert!(!ledger.has_open_permits());
            snapshot.cancel(&mut ledger).unwrap();
            assert_zero(&ledger);
        }
    }

    #[test]
    fn malformed_candidate_temporary_release_failure_rolls_back_parse_ownership() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        let raw = b"{";
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), raw).unwrap();
        let mut ledger = ledger("opened malformed candidate release failure");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        let before = ledger.snapshot().current.clone();

        let outcome = snapshot
            .parse_candidate_with_hooks(
                0,
                &mut ledger,
                |_, _| Ok(()),
                |_, _| {
                    Err(codefire_store::StoreError::Io(io::Error::other(
                        "injected parse Temporary release failure",
                    )))
                },
            )
            .unwrap();

        assert!(matches!(
            outcome,
            DoctorOpenedRegistryParseResult::CleanupFailed(
                codefire_store::StoreError::Io(ref error)
            ) if error.to_string().contains("injected parse Temporary release failure")
        ));
        assert!(snapshot.parsed_candidate(0).is_none());
        assert!(snapshot.invalid_candidate(0).is_none());
        assert_eq!(ledger.snapshot().current, before);
        assert!(!ledger.has_open_permits());
        snapshot.cancel(&mut ledger).unwrap();
        assert_zero(&ledger);
    }

    #[test]
    fn malformed_candidate_failed_retained_rollback_keeps_scalar_cleanup_owner() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        let raw = b"{";
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), raw).unwrap();
        let mut ledger = ledger("opened malformed candidate rollback owner");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
        let retained_before = snapshot.retained_bytes();

        let outcome = snapshot
            .parse_candidate_with_hooks(
                0,
                &mut ledger,
                |_, _| Ok(()),
                |ledger, _| {
                    let retained = ledger.snapshot().current
                        [&codefire_core::DecisionWorkDimension::RetainedBytes];
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained,
                        )
                        .unwrap();
                    Err(codefire_store::StoreError::Io(io::Error::other(
                        "injected release after external Retained cleanup",
                    )))
                },
            )
            .unwrap();

        assert!(matches!(
            outcome,
            DoctorOpenedRegistryParseResult::CleanupFailed(_)
        ));
        assert!(snapshot.invalid_candidate(0).is_none());
        assert!(snapshot.retained_bytes() > retained_before);
        assert_eq!(snapshot.parse_cleanup_temporary_bytes, 0);
        assert!(!ledger.has_open_permits());
        ledger
            .apply(reservation(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkDimension::RetainedBytes,
                snapshot.retained_bytes(),
            ))
            .unwrap();
        snapshot.cancel(&mut ledger).unwrap();
        assert_zero(&ledger);
    }

    #[test]
    fn parsed_candidate_foreign_and_failed_release_preserve_retry_cleanup() {
        for (raw, valid) in [(&b"{}"[..], true), (&b"{"[..], false)] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join(".codefire");
            let root = cf.join("opened");
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("main.json"), raw).unwrap();
            let mut owner = ledger("opened parsed candidate release owner");
            let mut foreign = ledger("opened parsed candidate release foreign");
            let mut snapshot =
                DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut owner).unwrap();
            let outcome = snapshot.parse_candidate(0, &mut owner).unwrap();
            assert_eq!(
                matches!(outcome, DoctorOpenedRegistryParseResult::Parsed),
                valid
            );
            let owner_before = owner.snapshot().clone();
            let foreign_before = foreign.snapshot().clone();

            assert!(snapshot.cancel(&mut foreign).is_err());
            assert_eq!(snapshot.parsed_candidate(0).is_some(), valid);
            assert_eq!(snapshot.invalid_candidate(0).is_some(), !valid);
            assert_eq!(owner.snapshot(), &owner_before);
            assert_eq!(foreign.snapshot(), &foreign_before);

            let scalar = snapshot.retained_bytes();
            owner
                .release(codefire_core::DecisionWorkDimension::RetainedBytes, scalar)
                .unwrap();
            assert!(snapshot.cancel(&mut owner).is_err());
            assert!(snapshot.parsed_candidate(0).is_none());
            assert!(snapshot.invalid_candidate(0).is_none());
            assert_eq!(snapshot.retained_bytes(), scalar);
            owner
                .apply(reservation(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    scalar,
                ))
                .unwrap();
            snapshot.cancel(&mut owner).unwrap();
            assert_zero(&owner);
            assert_zero(&foreign);
        }
    }

    #[test]
    fn parsed_candidate_combined_reservation_stops_before_materialization_at_limits() {
        let raw = b"{}";
        for dimension in [
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            codefire_core::DecisionWorkDimension::RetainedBytes,
        ] {
            for over in [false, true] {
                let temp = tempdir().unwrap();
                let cf = temp.path().join(".codefire");
                let root = cf.join("opened");
                fs::create_dir_all(&root).unwrap();
                fs::write(root.join("main.json"), raw).unwrap();
                let mut ledger = ledger("opened parsed candidate boundary");
                let mut snapshot =
                    DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
                let upper = match dimension {
                    codefire_core::DecisionWorkDimension::TemporaryBytes => {
                        json_parse_temporary_bytes(raw.len() as u64).unwrap()
                    }
                    codefire_core::DecisionWorkDimension::RetainedBytes => {
                        json_candidate_retained_upper(raw.len() as u64).unwrap()
                    }
                    _ => unreachable!(),
                };
                let limit = ledger.profile().limit(dimension).unwrap().limit;
                let current = ledger
                    .snapshot()
                    .current
                    .get(&dimension)
                    .copied()
                    .unwrap_or_default();
                let filler = limit - current - upper + u64::from(over);
                ledger
                    .apply(reservation(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        dimension,
                        filler,
                    ))
                    .unwrap();
                let mut materialized = false;

                let result = snapshot.parse_candidate_with_hook(0, &mut ledger, |stage, _| {
                    if stage == CandidateParseStage::BeforeMaterialization {
                        materialized = true;
                    }
                    Ok(())
                });

                if over {
                    let codefire_store::StoreError::DecisionWorkAccounting(error) =
                        result.unwrap_err()
                    else {
                        panic!("expected typed parsed candidate exhaustion");
                    };
                    assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                    assert!(!materialized);
                } else {
                    assert!(matches!(
                        result.unwrap(),
                        DoctorOpenedRegistryParseResult::Parsed
                    ));
                    assert!(materialized);
                }
                assert!(!ledger.has_open_permits());
                snapshot.cancel(&mut ledger).unwrap();
                ledger.release(dimension, filler).unwrap();
                assert_zero(&ledger);
            }
        }
    }

    #[test]
    fn snapshot_retained_limit_minus_one_exact_plus_one_is_preallocation() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.json"), b"{}").unwrap();
        let mut probe = ledger("opened snapshot retained probe");
        let mut snapshot = DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut probe).unwrap();
        let retained = snapshot.retained_bytes();
        snapshot.cancel(&mut probe).unwrap();
        assert_zero(&probe);
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "opened snapshot retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - retained - 1,
                0 => limit - retained,
                1 => limit - retained + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(reservation(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                ))
                .unwrap();
            let mut read_called = false;
            let result = DoctorOpenedRegistrySnapshot::capture_with_read_hook(
                &cf,
                &root,
                &mut ledger,
                |_| {
                    read_called = true;
                    Ok(())
                },
            );
            if delta <= 0 {
                let mut snapshot = result.unwrap();
                assert!(read_called);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if delta == 0 { limit } else { limit - 1 }
                );
                snapshot.cancel(&mut ledger).unwrap();
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) =
                    result.err().unwrap()
                else {
                    panic!("expected retained resource exhaustion");
                };
                assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                assert!(!read_called);
                assert!(!ledger.has_open_permits());
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_zero(&ledger);
        }
    }

    #[test]
    fn deep_absolute_parent_open_scratch_is_preopen_for_capture_and_fence() {
        let temp = tempdir().unwrap();
        let cf = deep_parent(temp.path());
        fs::create_dir_all(&cf).unwrap();
        let root = cf.join("opened");
        let open_upper = absolute_open_temporary_upper(&cf).unwrap();
        let capability_bytes = OWNER_FRAME_BYTES * 2;
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "opened deep absolute capture boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - open_upper - 1,
                0 => limit - open_upper,
                1 => limit - open_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(reservation(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ))
                .unwrap();
            let mut open_called = false;
            let result = DoctorOpenedRegistrySnapshot::capture_with_hooks(
                &cf,
                &root,
                &mut ledger,
                &mut |path| {
                    assert_eq!(path, cf);
                    open_called = true;
                    Ok(())
                },
                &mut |_| Ok(()),
            );
            if delta <= 0 {
                let mut snapshot = result.unwrap();
                assert!(open_called);
                snapshot.cancel(&mut ledger).unwrap();
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) =
                    result.err().unwrap()
                else {
                    panic!("expected temporary resource exhaustion");
                };
                assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                assert!(!open_called);
                assert!(!ledger.has_open_permits());
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_zero(&ledger);
        }

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "opened deep absolute fence boundary",
            )
            .unwrap();
            let mut snapshot =
                DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut ledger).unwrap();
            let fence_upper = open_upper + capability_bytes;
            let baseline = match delta {
                -1 => limit - fence_upper - 1,
                0 => limit - fence_upper,
                1 => limit - fence_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(reservation(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ))
                .unwrap();
            let mut open_called = false;
            let result = snapshot.revalidate_with_hook(&mut ledger, &mut |path| {
                assert_eq!(path, cf);
                open_called = true;
                Ok(())
            });
            if delta <= 0 {
                result.unwrap();
                assert!(open_called);
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) = result.unwrap_err()
                else {
                    panic!("expected temporary resource exhaustion");
                };
                assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                assert!(!open_called);
                assert!(!ledger.has_open_permits());
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_zero(&ledger);
        }
    }

    #[test]
    fn nested_handle_temporary_limit_stops_before_open_and_hook() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        let root = cf.join("opened");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("nested/main.json"), b"{}").unwrap();
        let mut probe = ledger("opened nested temporary probe");
        let mut probe_snapshot =
            DoctorOpenedRegistrySnapshot::capture(&cf, &root, &mut probe).unwrap();
        let peak = probe.snapshot().peak[&codefire_core::DecisionWorkDimension::TemporaryBytes];
        probe_snapshot.cancel(&mut probe).unwrap();
        assert_zero(&probe);
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "opened nested temporary boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - peak - 1,
                0 => limit - peak,
                1 => limit - peak + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(reservation(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                ))
                .unwrap();
            let mut nested_hook_called = false;
            let result =
                DoctorOpenedRegistrySnapshot::capture_with_hook(&cf, &root, &mut ledger, |path| {
                    if path == root.join("nested") {
                        nested_hook_called = true;
                    }
                    Ok(())
                });
            if delta <= 0 {
                let mut snapshot = result.unwrap();
                assert!(nested_hook_called);
                snapshot.cancel(&mut ledger).unwrap();
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) =
                    result.err().unwrap()
                else {
                    panic!("expected temporary resource exhaustion");
                };
                assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                assert!(!nested_hook_called);
                assert!(!ledger.has_open_permits());
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_zero(&ledger);
        }
    }
}
