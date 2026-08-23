use super::*;
use crate::compatibility::{validate_record_version, REMOTE_DOCTOR_SCHEMA};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_DIAGNOSTIC_SAMPLES: usize = 128;
const MAX_REMOTE_RECORDS: usize = 131_072;
const DOCTOR_DIRECTORY_ITERATOR_SCRATCH_BYTES: u64 = 4 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RemoteDoctorIssue {
    pub(crate) kind: String,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RemoteDoctorReport {
    #[serde(rename = "type")]
    pub(crate) type_tag: String,
    pub(crate) version: u64,
    pub(crate) schema: String,
    pub(crate) project: String,
    pub(crate) transport: String,
    pub(crate) semantic_profile: String,
    pub(crate) project_exists: bool,
    pub(crate) layout: BTreeMap<String, bool>,
    pub(crate) checked_objects: usize,
    pub(crate) branch_count: usize,
    pub(crate) merge_request_count: usize,
    pub(crate) object_hash_errors: usize,
    pub(crate) missing_object_references: usize,
    pub(crate) invalid_sealed_commit_references: usize,
    pub(crate) invalid_record_versions: usize,
    pub(crate) diagnostics_truncated: bool,
    pub(crate) quick: bool,
    pub(crate) skipped_checks: Vec<String>,
    pub(crate) issues: Vec<RemoteDoctorIssue>,
    pub(crate) ok: bool,
}

impl RemoteDoctorReport {
    fn empty(project: &str, transport: &str, quick: bool) -> Self {
        Self {
            type_tag: "codefire_remote_doctor_report".to_string(),
            version: 1,
            schema: REMOTE_DOCTOR_SCHEMA.to_string(),
            project: project.to_string(),
            transport: transport.to_string(),
            semantic_profile: codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE.to_string(),
            project_exists: false,
            layout: BTreeMap::new(),
            checked_objects: 0,
            branch_count: 0,
            merge_request_count: 0,
            object_hash_errors: 0,
            missing_object_references: 0,
            invalid_sealed_commit_references: 0,
            invalid_record_versions: 0,
            diagnostics_truncated: false,
            quick,
            skipped_checks: Vec::new(),
            issues: Vec::new(),
            ok: false,
        }
    }

    fn push_issue(&mut self, kind: &str, message: impl Into<String>, subject: Option<String>) {
        if self.issues.len() < MAX_DIAGNOSTIC_SAMPLES {
            self.issues.push(RemoteDoctorIssue {
                kind: kind.to_string(),
                message: message.into(),
                subject,
            });
        } else {
            self.diagnostics_truncated = true;
        }
    }
}

#[derive(Serialize)]
struct EmptyRemoteDoctorReport<'a> {
    #[serde(rename = "type")]
    type_tag: &'static str,
    version: u64,
    schema: &'static str,
    project: &'a str,
    transport: &'a str,
    semantic_profile: &'static str,
    project_exists: bool,
    layout: BTreeMap<String, bool>,
    checked_objects: usize,
    branch_count: usize,
    merge_request_count: usize,
    object_hash_errors: usize,
    missing_object_references: usize,
    invalid_sealed_commit_references: usize,
    invalid_record_versions: usize,
    diagnostics_truncated: bool,
    quick: bool,
    skipped_checks: Vec<String>,
    issues: Vec<RemoteDoctorIssue>,
    ok: bool,
}

pub(crate) struct AccountedRemoteDoctorReport {
    report: RemoteDoctorReport,
    retained_bytes: u64,
}

#[derive(Clone, Copy)]
enum DoctorReportCounter {
    CheckedObjects,
    BranchCount,
    MergeRequestCount,
    ObjectHashErrors,
    MissingObjectReferences,
    InvalidSealedCommitReferences,
    InvalidRecordVersions,
}

#[derive(Default)]
struct JsonLengthWriter {
    bytes: u64,
    overflowed: bool,
}

impl Write for JsonLengthWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("remote doctor JSON size overflow"))?;
        self.bytes = match self.bytes.checked_add(amount) {
            Some(bytes) => bytes,
            None => {
                self.overflowed = true;
                return Err(std::io::Error::other("remote doctor JSON size overflow"));
            }
        };
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct JsonStringLengthWriter {
    bytes: u64,
    overflowed: bool,
}

impl JsonStringLengthWriter {
    fn add(&mut self, amount: u64) -> fmt::Result {
        self.bytes = self.bytes.checked_add(amount).ok_or_else(|| {
            self.overflowed = true;
            fmt::Error
        })?;
        Ok(())
    }
}

impl fmt::Write for JsonStringLengthWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for character in value.chars() {
            let bytes = match character {
                '"' | '\\' | '\u{0008}' | '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' => 2,
                '\u{0000}'..='\u{001F}' => 6,
                _ => u64::try_from(character.len_utf8()).map_err(|_| fmt::Error)?,
            };
            self.add(bytes)?;
        }
        Ok(())
    }
}

fn compact_json_len<T: Serialize + ?Sized>(value: &T) -> Result<u64, CliError> {
    let mut writer = JsonLengthWriter::default();
    if let Err(error) = serde_json::to_writer(&mut writer, value) {
        if writer.overflowed {
            return Err(CliError::InvalidRepository(
                "remote doctor report size overflow".to_string(),
            ));
        }
        return Err(CliError::from(error));
    }
    Ok(writer.bytes)
}

fn formatted_json_string_len(
    arguments: fmt::Arguments<'_>,
) -> Result<u64, codefire_core::DecisionWorkAccountingError> {
    use std::fmt::Write as _;

    let mut writer = JsonStringLengthWriter::default();
    writer
        .write_fmt(arguments)
        .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
    writer
        .bytes
        .checked_add(2)
        .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)
}

fn decimal_len(value: usize) -> u64 {
    if value == 0 {
        return 1;
    }
    u64::from(value.ilog10()) + 1
}

impl AccountedRemoteDoctorReport {
    fn new(
        project: &str,
        transport: &str,
        quick: bool,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<Self, CliError> {
        let retained_bytes = compact_json_len(&EmptyRemoteDoctorReport {
            type_tag: "codefire_remote_doctor_report",
            version: 1,
            schema: REMOTE_DOCTOR_SCHEMA,
            project,
            transport,
            semantic_profile: codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE,
            project_exists: false,
            layout: BTreeMap::new(),
            checked_objects: 0,
            branch_count: 0,
            merge_request_count: 0,
            object_hash_errors: 0,
            missing_object_references: 0,
            invalid_sealed_commit_references: 0,
            invalid_record_versions: 0,
            diagnostics_truncated: false,
            quick,
            skipped_checks: Vec::new(),
            issues: Vec::new(),
            ok: false,
        })?;
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Encoding,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained_bytes,
                ),
            )
            .map_err(|error| doctor_accounting_error(error, "retaining initial doctor report"))?;
        Ok(Self {
            report: RemoteDoctorReport::empty(project, transport, quick),
            retained_bytes,
        })
    }

    pub(crate) fn report(&self) -> &RemoteDoctorReport {
        &self.report
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn release_owner(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        let retained_bytes = self.retained_bytes;
        if retained_bytes == 0 {
            return Ok(());
        }
        ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        )?;
        self.retained_bytes = 0;
        Ok(())
    }

    fn reserve_growth(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        amount: u64,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::Encoding,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount),
        )?;
        self.retained_bytes = self
            .retained_bytes
            .checked_add(amount)
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        Ok(())
    }

    fn release_shrink(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        amount: u64,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        ledger.release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)?;
        self.retained_bytes = self
            .retained_bytes
            .checked_sub(amount)
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        Ok(())
    }

    fn set_project_exists(
        &mut self,
        exists: bool,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        if self.report.project_exists == exists {
            return Ok(());
        }
        let old_len = if self.report.project_exists { 4 } else { 5 };
        let new_len = if exists { 4 } else { 5 };
        if new_len > old_len {
            self.reserve_growth(ledger, new_len - old_len)?;
        }
        self.report.project_exists = exists;
        if old_len > new_len {
            self.release_shrink(ledger, old_len - new_len)?;
        }
        Ok(())
    }

    fn insert_layout(
        &mut self,
        name: &str,
        exists: bool,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        if self.report.layout.contains_key(name) {
            return Err(codefire_core::DecisionWorkAccountingError::SizeOverflow);
        }
        let growth = formatted_json_string_len(format_args!("{name}"))?
            .checked_add(1)
            .and_then(|bytes| bytes.checked_add(if exists { 4 } else { 5 }))
            .and_then(|bytes| bytes.checked_add(u64::from(!self.report.layout.is_empty())))
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        self.reserve_growth(ledger, growth)?;
        self.report.layout.insert(name.to_string(), exists);
        Ok(())
    }

    fn increment_counter(
        &mut self,
        counter: DoctorReportCounter,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        let current = match counter {
            DoctorReportCounter::CheckedObjects => self.report.checked_objects,
            DoctorReportCounter::BranchCount => self.report.branch_count,
            DoctorReportCounter::MergeRequestCount => self.report.merge_request_count,
            DoctorReportCounter::ObjectHashErrors => self.report.object_hash_errors,
            DoctorReportCounter::MissingObjectReferences => self.report.missing_object_references,
            DoctorReportCounter::InvalidSealedCommitReferences => {
                self.report.invalid_sealed_commit_references
            }
            DoctorReportCounter::InvalidRecordVersions => self.report.invalid_record_versions,
        };
        let next = current
            .checked_add(1)
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        let old_len = decimal_len(current);
        let new_len = decimal_len(next);
        if new_len > old_len {
            self.reserve_growth(ledger, new_len - old_len)?;
        }
        match counter {
            DoctorReportCounter::CheckedObjects => self.report.checked_objects = next,
            DoctorReportCounter::BranchCount => self.report.branch_count = next,
            DoctorReportCounter::MergeRequestCount => self.report.merge_request_count = next,
            DoctorReportCounter::ObjectHashErrors => self.report.object_hash_errors = next,
            DoctorReportCounter::MissingObjectReferences => {
                self.report.missing_object_references = next
            }
            DoctorReportCounter::InvalidSealedCommitReferences => {
                self.report.invalid_sealed_commit_references = next
            }
            DoctorReportCounter::InvalidRecordVersions => {
                self.report.invalid_record_versions = next
            }
        }
        Ok(())
    }

    fn push_issue(
        &mut self,
        kind: &str,
        message: fmt::Arguments<'_>,
        subject: Option<fmt::Arguments<'_>>,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        if self.report.issues.len() >= MAX_DIAGNOSTIC_SAMPLES {
            if !self.report.diagnostics_truncated {
                self.report.diagnostics_truncated = true;
                self.release_shrink(ledger, 1)?;
            }
            return Ok(());
        }
        let mut issue_bytes = 8_u64
            .checked_add(formatted_json_string_len(format_args!("{kind}"))?)
            .and_then(|bytes| bytes.checked_add(11))
            .and_then(|bytes| {
                formatted_json_string_len(message)
                    .ok()
                    .and_then(|message| bytes.checked_add(message))
            })
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        if let Some(subject) = subject {
            issue_bytes = issue_bytes
                .checked_add(11)
                .and_then(|bytes| {
                    formatted_json_string_len(subject)
                        .ok()
                        .and_then(|subject| bytes.checked_add(subject))
                })
                .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        }
        issue_bytes = issue_bytes
            .checked_add(1 + u64::from(!self.report.issues.is_empty()))
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        self.reserve_growth(ledger, issue_bytes)?;
        self.report.issues.push(RemoteDoctorIssue {
            kind: kind.to_string(),
            message: message.to_string(),
            subject: subject.map(|value| value.to_string()),
        });
        Ok(())
    }

    fn push_skipped_check(
        &mut self,
        check: &str,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        let growth = formatted_json_string_len(format_args!("{check}"))?
            .checked_add(u64::from(!self.report.skipped_checks.is_empty()))
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        self.reserve_growth(ledger, growth)?;
        self.report.skipped_checks.push(check.to_string());
        Ok(())
    }

    fn finish(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), codefire_core::DecisionWorkAccountingError> {
        let ok = self.report.issues.is_empty();
        if ok && !self.report.ok {
            self.report.ok = true;
            self.release_shrink(ledger, 1)?;
        }
        Ok(())
    }
}

pub(crate) fn run_remote_doctor(
    target: &str,
    quick: bool,
) -> Result<Option<RemoteDoctorReport>, CliError> {
    if is_cf_http_url(target) {
        let project = parse_cf_http_project_url(target)?;
        let mut path = http_remote_path(&project.endpoint, &project.org, &project.app, &["doctor"]);
        if quick {
            path.push_str("?quick=1");
        }
        let value = http_json("GET", &path, None)?;
        validate_record_version(&value, "remote_doctor")?;
        let mut report: RemoteDoctorReport = serde_json::from_value(value).map_err(|error| {
            CliError::InvalidRepository(format!(
                "HTTP remote doctor returned an invalid {REMOTE_DOCTOR_SCHEMA} report: {error}"
            ))
        })?;
        if report.schema != REMOTE_DOCTOR_SCHEMA
            || report.type_tag != "codefire_remote_doctor_report"
            || !matches!(report.transport.as_str(), "http" | "https")
        {
            return Err(CliError::InvalidRepository(
                "HTTP remote doctor report capability/profile does not match the request"
                    .to_string(),
            ));
        }
        report.project = target.to_string();
        report.transport = project.endpoint_scheme().to_string();
        if report.quick != quick {
            return Err(CliError::InvalidRepository(
                "HTTP remote doctor report mode does not match the request".to_string(),
            ));
        }
        return Ok(Some(report));
    }
    if target.starts_with("cf://") {
        let project = parse_cf_project_url(target)?;
        return Ok(Some(diagnose_remote_project(
            &project.project_root,
            target,
            "file",
            quick,
        )));
    }
    Ok(None)
}

pub(crate) fn diagnose_remote_project(
    project_root: &Path,
    project_url: &str,
    transport: &str,
    quick: bool,
) -> RemoteDoctorReport {
    let mut report = RemoteDoctorReport::empty(project_url, transport, quick);
    report.project_exists = project_root.is_dir();
    let dirs = remote_dirs(project_root);
    report.layout = BTreeMap::from([
        ("objects".to_string(), dirs.objects.is_dir()),
        ("branches".to_string(), dirs.branches.is_dir()),
        ("merge_requests".to_string(), dirs.merge_requests.is_dir()),
        ("locks".to_string(), dirs.locks.is_dir()),
        ("idempotency".to_string(), dirs.idempotency.is_dir()),
    ]);
    if !report.project_exists {
        report.push_issue(
            "remote_project_missing",
            "remote project does not exist",
            Some(project_url.to_string()),
        );
        return report;
    }
    let gate = match RemoteProjectGate::acquire_shared(project_root, &LockOptions::default()) {
        Ok(gate) => gate,
        Err(error) => {
            report.push_issue(
                "remote_project_gate_invalid",
                error.to_string(),
                Some(project_url.to_string()),
            );
            return report;
        }
    };
    if let Err(error) = gate.ensure_project(project_root) {
        report.push_issue(
            "remote_project_gate_invalid",
            error.to_string(),
            Some(project_url.to_string()),
        );
        return report;
    }
    for (name, exists) in report.layout.clone() {
        if !exists {
            report.push_issue(
                "remote_layout_missing",
                format!("remote layout component is missing: {name}"),
                Some(name),
            );
        }
    }

    let mut roots = BTreeSet::new();
    inspect_branches(&dirs, &mut roots, &mut report);
    inspect_merge_requests(&dirs, &mut roots, &mut report);
    if quick {
        report
            .skipped_checks
            .push("object_hash_and_reference_graph".to_string());
    } else {
        inspect_objects(&dirs, &mut report);
        inspect_sealed_roots(&dirs.objects, roots, &mut report);
    }
    report.ok = report.issues.is_empty();
    report
}

fn doctor_accounting_error(
    error: codefire_core::DecisionWorkAccountingError,
    operation: &str,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while {operation}: {error}"
        )),
    }
}

fn doctor_store_error(error: codefire_store::StoreError) -> CliError {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
            doctor_accounting_error(*accounting, "validating remote doctor sealed roots")
        }
        error => CliError::from(error),
    }
}

fn doctor_release(
    ledger: &mut codefire_core::DecisionWorkLedger,
    dimension: codefire_core::DecisionWorkDimension,
    amount: u64,
    operation: &str,
) -> Result<(), CliError> {
    ledger
        .release(dimension, amount)
        .map_err(|error| doctor_accounting_error(error, operation))
}

fn doctor_directory_entries<F>(
    directory: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: &mut F,
) -> Result<BTreeMap<PathBuf, u64>, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    poll()?;
    let mut iterator = fs::read_dir(directory)?;
    let mut entries = BTreeMap::new();
    let collection = (|| -> Result<(), CliError> {
        loop {
            poll()?;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        DOCTOR_DIRECTORY_ITERATOR_SCRATCH_BYTES,
                    ),
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "reserving doctor directory scan")
                })?;
            let next = iterator.next().transpose();
            doctor_release(
                ledger,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                DOCTOR_DIRECTORY_ITERATOR_SCRATCH_BYTES,
                "releasing doctor directory scan",
            )?;
            let Some(entry) = next? else {
                break;
            };
            let name = entry.file_name().into_string().map_err(|_| {
                CliError::InvalidRepository(
                    "remote doctor encountered a non-UTF-8 directory entry".to_string(),
                )
            })?;
            let frame_bytes =
                codefire_core::logical_retained_frame_bytes(&name).map_err(|error| {
                    doctor_accounting_error(error, "accounting doctor directory entry")
                })?;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::RecordRead,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        frame_bytes,
                    ),
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining doctor directory entry")
                })?;
            if let Some(replaced) = entries.insert(entry.path(), frame_bytes) {
                doctor_release(
                    ledger,
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    replaced,
                    "releasing duplicate doctor directory entry",
                )?;
            }
            poll()?;
        }
        Ok(())
    })();
    match collection {
        Ok(()) => Ok(entries),
        Err(error) => {
            release_doctor_entry_frames(
                entries,
                ledger,
                "releasing failed doctor directory entries",
            )?;
            Err(error)
        }
    }
}

fn release_doctor_entry_frames(
    entries: BTreeMap<PathBuf, u64>,
    ledger: &mut codefire_core::DecisionWorkLedger,
    operation: &str,
) -> Result<(), CliError> {
    for (_, frame_bytes) in entries {
        doctor_release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
            operation,
        )?;
    }
    Ok(())
}

enum DoctorJsonRead {
    Value { value: Value, retained_bytes: u64 },
    Invalid(CliError),
}

struct DoctorJsonCounter<'a, F> {
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    poll: &'a mut F,
    bytes: u64,
    failure: Option<CliError>,
}

impl<F> Write for DoctorJsonCounter<'_, F>
where
    F: FnMut() -> Result<(), CliError>,
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Err(error) = (self.poll)() {
            self.failure = Some(error);
            return Err(std::io::Error::other(
                "remote doctor JSON counting cancelled",
            ));
        }
        let amount = match u64::try_from(bytes.len()) {
            Ok(amount) => amount,
            Err(_) => {
                self.failure = Some(CliError::InvalidRepository(
                    "remote doctor JSON size overflow".to_string(),
                ));
                return Err(std::io::Error::other("remote doctor JSON size overflow"));
            }
        };
        if let Err(error) = self.ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .charge(codefire_core::DecisionWorkDimension::AggregateBytes, amount),
        ) {
            self.failure = Some(doctor_accounting_error(
                error,
                "counting remote doctor JSON",
            ));
            return Err(std::io::Error::other(
                "remote doctor JSON accounting failed",
            ));
        }
        self.bytes = match self.bytes.checked_add(amount) {
            Some(total) => total,
            None => {
                self.failure = Some(CliError::InvalidRepository(
                    "remote doctor JSON size overflow".to_string(),
                ));
                return Err(std::io::Error::other("remote doctor JSON size overflow"));
            }
        };
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_doctor_json<F>(
    path: &Path,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: &mut F,
) -> Result<DoctorJsonRead, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    poll()?;
    let physical_bytes = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) => return Ok(DoctorJsonRead::Invalid(CliError::Io(error))),
    };
    let scratch_bytes = physical_bytes.checked_mul(2).ok_or_else(|| {
        CliError::InvalidRepository("remote doctor JSON scratch overflow".to_string())
    })?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::RecordRead,
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
            .count(codefire_core::DecisionWorkCounter::FilesRead, 1)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                physical_bytes,
            )
            .count(codefire_core::DecisionWorkCounter::RecordReads, 1)
            .charge(codefire_core::DecisionWorkDimension::ObjectRecords, 1),
        )
        .map_err(|error| doctor_accounting_error(error, "reserving remote doctor JSON read"))?;
    let value = match read_json(path) {
        Ok(value) => value,
        Err(error) => {
            doctor_release(
                ledger,
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                scratch_bytes,
                "releasing invalid remote doctor JSON scratch",
            )?;
            return Ok(DoctorJsonRead::Invalid(error));
        }
    };
    let mut counter = DoctorJsonCounter {
        ledger,
        poll,
        bytes: 0,
        failure: None,
    };
    let encoded = serde_json::to_writer(&mut counter, &value);
    if let Some(error) = counter.failure.take() {
        doctor_release(
            counter.ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
            "releasing failed remote doctor JSON scratch",
        )?;
        return Err(error);
    }
    if let Err(error) = encoded {
        doctor_release(
            counter.ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
            "releasing unencoded remote doctor JSON scratch",
        )?;
        return Err(CliError::Json(error));
    }
    let retained_bytes = counter.bytes;
    if let Err(error) = counter.ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::RecordRead,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        ),
    ) {
        doctor_release(
            counter.ledger,
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            scratch_bytes,
            "releasing unretained remote doctor JSON scratch",
        )?;
        return Err(doctor_accounting_error(
            error,
            "retaining remote doctor JSON",
        ));
    }
    if let Err(error) = doctor_release(
        counter.ledger,
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        scratch_bytes,
        "releasing remote doctor JSON scratch",
    ) {
        doctor_release(
            counter.ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
            "releasing retained remote doctor JSON after scratch failure",
        )?;
        return Err(error);
    }
    if let Err(error) = (counter.poll)() {
        doctor_release(
            counter.ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
            "releasing cancelled remote doctor JSON",
        )?;
        return Err(error);
    }
    Ok(DoctorJsonRead::Value {
        value,
        retained_bytes,
    })
}

fn retain_doctor_root(
    roots: &mut BTreeMap<String, u64>,
    root: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    if roots.contains_key(root) {
        return Ok(());
    }
    let frame_bytes = codefire_core::logical_retained_frame_bytes(root)
        .map_err(|error| doctor_accounting_error(error, "accounting remote doctor root"))?;
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::GraphDiscovery,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                frame_bytes,
            ),
        )
        .map_err(|error| doctor_accounting_error(error, "retaining remote doctor root"))?;
    roots.insert(root.to_string(), frame_bytes);
    Ok(())
}

fn inspect_branches_with_work<F>(
    dirs: &RemoteDirs,
    roots: &mut BTreeMap<String, u64>,
    report: &mut AccountedRemoteDoctorReport,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: &mut F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let mut entries = match doctor_directory_entries(&dirs.branches, ledger, poll) {
        Ok(entries) => entries,
        Err(CliError::Io(error)) => {
            report
                .push_issue(
                    "remote_branch_directory_invalid",
                    format_args!("{error}"),
                    None,
                    ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining branch-directory issue")
                })?;
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    while let Some((path, path_frame_bytes)) = entries.pop_first() {
        let item_result = (|| -> Result<(), CliError> {
            poll()?;
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return Ok(());
            }
            report
                .increment_counter(DoctorReportCounter::BranchCount, ledger)
                .map_err(|error| doctor_accounting_error(error, "incrementing branch count"))?;
            match read_doctor_json(&path, ledger, poll)? {
                DoctorJsonRead::Invalid(error) => report
                    .push_issue(
                        "remote_branch_record_invalid",
                        format_args!("{error}"),
                        Some(format_args!("{}", path.display())),
                        ledger,
                    )
                    .map_err(|error| {
                        doctor_accounting_error(error, "retaining invalid branch issue")
                    })?,
                DoctorJsonRead::Value {
                    value: branch,
                    retained_bytes,
                } => {
                    let branch_result = (|| -> Result<(), CliError> {
                        if let Err(error) = validate_record_version(&branch, "remote_branch") {
                            report
                                .increment_counter(
                                    DoctorReportCounter::InvalidRecordVersions,
                                    ledger,
                                )
                                .map_err(|error| {
                                    doctor_accounting_error(
                                        error,
                                        "incrementing invalid branch version",
                                    )
                                })?;
                            report
                                .push_issue(
                                    "remote_record_version_unsupported",
                                    format_args!("{error}"),
                                    Some(format_args!("{}", path.display())),
                                    ledger,
                                )
                                .map_err(|error| {
                                    doctor_accounting_error(
                                        error,
                                        "retaining invalid branch version issue",
                                    )
                                })?;
                        } else {
                            match (
                                branch.get("name").and_then(Value::as_str),
                                branch.get("head").and_then(Value::as_str),
                            ) {
                                (Some(name), Some(head)) => {
                                    if let Err(error) = validate_remote_component(name, "branch") {
                                        report
                                            .push_issue(
                                                "remote_branch_record_invalid",
                                                format_args!("{error}"),
                                                Some(format_args!("{}", path.display())),
                                                ledger,
                                            )
                                            .map_err(|error| {
                                                doctor_accounting_error(
                                                    error,
                                                    "retaining invalid branch component issue",
                                                )
                                            })?;
                                    }
                                    retain_doctor_root(roots, head, ledger)?;
                                }
                                _ => report
                                    .push_issue(
                                        "remote_branch_record_invalid",
                                        format_args!(
                                            "remote branch must contain string name and head"
                                        ),
                                        Some(format_args!("{}", path.display())),
                                        ledger,
                                    )
                                    .map_err(|error| {
                                        doctor_accounting_error(
                                            error,
                                            "retaining malformed branch issue",
                                        )
                                    })?,
                            }
                        }
                        Ok(())
                    })();
                    drop(branch);
                    doctor_release(
                        ledger,
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                        "releasing doctor branch JSON",
                    )?;
                    branch_result?;
                }
            }
            Ok(())
        })();
        doctor_release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            path_frame_bytes,
            "releasing doctor branch entry",
        )?;
        if let Err(error) = item_result {
            release_doctor_entry_frames(
                entries,
                ledger,
                "releasing unvisited doctor branch entries",
            )?;
            return Err(error);
        }
    }
    Ok(())
}

fn inspect_merge_requests_with_work<F>(
    dirs: &RemoteDirs,
    roots: &mut BTreeMap<String, u64>,
    report: &mut AccountedRemoteDoctorReport,
    ledger: &mut codefire_core::DecisionWorkLedger,
    poll: &mut F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let mut entries = match doctor_directory_entries(&dirs.merge_requests, ledger, poll) {
        Ok(entries) => entries,
        Err(CliError::Io(error)) => {
            report
                .push_issue(
                    "remote_merge_request_directory_invalid",
                    format_args!("{error}"),
                    None,
                    ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining merge-request-directory issue")
                })?;
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    while let Some((path, path_frame_bytes)) = entries.pop_first() {
        let item_result = (|| -> Result<(), CliError> {
            poll()?;
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return Ok(());
            }
            report
                .increment_counter(DoctorReportCounter::MergeRequestCount, ledger)
                .map_err(|error| {
                    doctor_accounting_error(error, "incrementing merge-request count")
                })?;
            match read_doctor_json(&path, ledger, poll)? {
                DoctorJsonRead::Invalid(error) => report
                    .push_issue(
                        "remote_merge_request_record_invalid",
                        format_args!("{error}"),
                        Some(format_args!("{}", path.display())),
                        ledger,
                    )
                    .map_err(|error| {
                        doctor_accounting_error(error, "retaining invalid merge-request issue")
                    })?,
                DoctorJsonRead::Value {
                    value: mr,
                    retained_bytes,
                } => {
                    let mr_result = (|| -> Result<(), CliError> {
                        if let Err(error) = validate_record_version(&mr, "remote_merge_request") {
                            report
                                .increment_counter(
                                    DoctorReportCounter::InvalidRecordVersions,
                                    ledger,
                                )
                                .map_err(|error| {
                                    doctor_accounting_error(
                                        error,
                                        "incrementing invalid merge-request version",
                                    )
                                })?;
                            report
                                .push_issue(
                                    "remote_record_version_unsupported",
                                    format_args!("{error}"),
                                    Some(format_args!("{}", path.display())),
                                    ledger,
                                )
                                .map_err(|error| {
                                    doctor_accounting_error(
                                        error,
                                        "retaining invalid merge-request version issue",
                                    )
                                })?;
                        } else {
                            let valid_id = mr.get("id").and_then(Value::as_str).is_some_and(|id| {
                                validate_remote_component(id, "merge-request ID").is_ok()
                            });
                            let valid_status = matches!(
                                mr.get("status").and_then(Value::as_str),
                                Some("open" | "approved" | "rejected" | "applied")
                            );
                            if !valid_id || !valid_status {
                                report
                                    .push_issue(
                                        "remote_merge_request_record_invalid",
                                        format_args!(
                                            "remote merge request has an invalid id or status"
                                        ),
                                        Some(format_args!("{}", path.display())),
                                        ledger,
                                    )
                                    .map_err(|error| {
                                        doctor_accounting_error(
                                            error,
                                            "retaining malformed merge-request issue",
                                        )
                                    })?;
                            }
                            for field in ["source_head", "target_head_at_request", "applied_head"] {
                                poll()?;
                                if let Some(head) = mr.get(field).and_then(Value::as_str) {
                                    retain_doctor_root(roots, head, ledger)?;
                                } else if field != "applied_head" {
                                    report
                                        .push_issue(
                                            "remote_merge_request_record_invalid",
                                            format_args!("remote merge request is missing {field}"),
                                            Some(format_args!("{}", path.display())),
                                            ledger,
                                        )
                                        .map_err(|error| {
                                            doctor_accounting_error(
                                                error,
                                                "retaining incomplete merge-request issue",
                                            )
                                        })?;
                                }
                            }
                        }
                        Ok(())
                    })();
                    drop(mr);
                    doctor_release(
                        ledger,
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                        "releasing doctor merge-request JSON",
                    )?;
                    mr_result?;
                }
            }
            Ok(())
        })();
        doctor_release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            path_frame_bytes,
            "releasing doctor merge-request entry",
        )?;
        if let Err(error) = item_result {
            release_doctor_entry_frames(
                entries,
                ledger,
                "releasing unvisited doctor merge-request entries",
            )?;
            return Err(error);
        }
    }
    Ok(())
}

#[expect(
    clippy::result_large_err,
    reason = "this helper participates directly in the Store scanner callback contract"
)]
fn retain_scanned_identifier(
    values: &mut BTreeMap<String, u64>,
    value: &str,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), codefire_store::StoreError> {
    if values.contains_key(value) {
        return Ok(());
    }
    let frame_bytes = codefire_core::logical_retained_frame_bytes(value)?;
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
        ),
    )?;
    values.insert(value.to_string(), frame_bytes);
    Ok(())
}

fn release_scanned_identifiers(
    values: BTreeMap<String, u64>,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    for (_, frame_bytes) in values {
        doctor_release(
            ledger,
            codefire_core::DecisionWorkDimension::RetainedBytes,
            frame_bytes,
            "releasing remote doctor scanned identifier",
        )?;
    }
    Ok(())
}

#[expect(
    clippy::result_large_err,
    reason = "this helper participates directly in the Store scanner callback contract"
)]
fn charge_doctor_reference(
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), codefire_store::StoreError> {
    ledger.apply(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::GraphDiscovery,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .charge(codefire_core::DecisionWorkDimension::GraphEdges, 1)
        .count(codefire_core::DecisionWorkCounter::EdgeOccurrences, 1),
    )?;
    Ok(())
}

fn inspect_objects_and_roots_with_work<F>(
    dirs: &RemoteDirs,
    roots: BTreeMap<String, u64>,
    report: &mut AccountedRemoteDoctorReport,
    ledger: &mut Option<codefire_core::DecisionWorkLedger>,
    poll: &mut F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let mut object_ids = BTreeMap::new();
    let mut references = BTreeMap::new();
    let mut encountered_ids = BTreeMap::new();
    let scan_and_validation = (|| -> Result<(), CliError> {
        let scan_result = {
            use std::cell::RefCell;

            let poll_cell = RefCell::new(&mut *poll);
            let cancellation_error = RefCell::new(None);
            let result =
                codefire_store::scan_object_store_record_documents_with_work_and_cancellation(
                    &dirs.objects,
                    ledger
                        .as_mut()
                        .expect("remote doctor ledger must be present for object scan"),
                    || match (*poll_cell.borrow_mut())() {
                        Ok(()) => false,
                        Err(error) => {
                            *cancellation_error.borrow_mut() = Some(error);
                            true
                        }
                    },
                    #[expect(
                        clippy::result_large_err,
                        reason = "the Store scanner visitor contract requires StoreError"
                    )]
                    |entry, ledger| {
                        report.increment_counter(DoctorReportCounter::CheckedObjects, ledger)?;
                        if let Some(object_id) =
                            entry.path.file_stem().and_then(|value| value.to_str())
                        {
                            retain_scanned_identifier(&mut encountered_ids, object_id, ledger)?;
                        }
                        match entry.document {
                            Ok(document) => {
                                retain_scanned_identifier(
                                    &mut object_ids,
                                    &document.record.object_id,
                                    ledger,
                                )?;
                                try_for_each_object_document_reference(
                                    &document,
                                    #[expect(
                                        clippy::result_large_err,
                                        reason = "the Store scanner visitor propagates StoreError"
                                    )]
                                    |reference| {
                                        if let Err(error) = (*poll_cell.borrow_mut())() {
                                            *cancellation_error.borrow_mut() = Some(error);
                                            return Err(
                                                codefire_store::StoreError::ValidationCancelled,
                                            );
                                        }
                                        charge_doctor_reference(ledger)?;
                                        retain_scanned_identifier(
                                            &mut references,
                                            reference,
                                            ledger,
                                        )
                                    },
                                )?;
                            }
                            Err(error) => {
                                report.increment_counter(
                                    DoctorReportCounter::ObjectHashErrors,
                                    ledger,
                                )?;
                                report.push_issue(
                                    "remote_object_record_invalid",
                                    format_args!("{error}"),
                                    Some(format_args!("{}", entry.path.display())),
                                    ledger,
                                )?;
                            }
                        }
                        Ok(())
                    },
                );
            let cancellation_error = cancellation_error.into_inner();
            (result, cancellation_error)
        };
        if let Some(error) = scan_result.1 {
            return Err(error);
        }
        if let Err(error) = scan_result.0 {
            if matches!(
                error,
                codefire_store::StoreError::ValidationCancelled
                    | codefire_store::StoreError::DecisionWorkAccounting(_)
            ) {
                return Err(doctor_store_error(error));
            }
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must remain after object-store failure");
            report
                .push_issue(
                    "remote_object_store_invalid",
                    format_args!("{error}"),
                    None,
                    active_ledger,
                )
                .map_err(|error| doctor_accounting_error(error, "retaining object-store issue"))?;
        }
        for missing in references.keys() {
            poll()?;
            if !object_ids.contains_key(missing) {
                let active_ledger = ledger
                    .as_mut()
                    .expect("remote doctor ledger must remain while diagnosing references");
                report
                    .increment_counter(DoctorReportCounter::MissingObjectReferences, active_ledger)
                    .map_err(|error| {
                        doctor_accounting_error(error, "incrementing missing reference count")
                    })?;
                report
                    .push_issue(
                        "remote_object_reference_missing",
                        format_args!("remote object reference is missing: {missing}"),
                        Some(format_args!("{missing}")),
                        active_ledger,
                    )
                    .map_err(|error| {
                        doctor_accounting_error(error, "retaining missing reference issue")
                    })?;
            }
        }
        validate_doctor_roots_with_work(
            &dirs.objects,
            &roots,
            &encountered_ids,
            report,
            ledger,
            poll,
        )
    })();
    let active_ledger = ledger
        .as_mut()
        .expect("remote doctor ledger must return for object/root cleanup");
    release_scanned_identifiers(references, active_ledger)?;
    release_scanned_identifiers(object_ids, active_ledger)?;
    release_scanned_identifiers(encountered_ids, active_ledger)?;
    release_scanned_identifiers(roots, active_ledger)?;
    scan_and_validation
}

fn validate_doctor_roots_with_work<F>(
    objects: &Path,
    roots: &BTreeMap<String, u64>,
    encountered_ids: &BTreeMap<String, u64>,
    report: &mut AccountedRemoteDoctorReport,
    ledger: &mut Option<codefire_core::DecisionWorkLedger>,
    poll: &mut F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    let has_encountered_root = roots.keys().any(|root| encountered_ids.contains_key(root));
    if !has_encountered_root {
        let active_ledger = ledger
            .as_mut()
            .expect("remote doctor ledger must be present for missing-root diagnostics");
        for root in roots.keys() {
            poll()?;
            report
                .increment_counter(
                    DoctorReportCounter::InvalidSealedCommitReferences,
                    active_ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "incrementing missing sealed-root count")
                })?;
            report
                .push_issue(
                    "remote_sealed_commit_reference_invalid",
                    format_args!("object not found: {root}"),
                    Some(format_args!("{root}")),
                    active_ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining missing sealed-root issue")
                })?;
        }
        return Ok(());
    }

    poll()?;
    let owned_ledger = ledger
        .take()
        .expect("remote doctor ledger must be present for root validation");
    let mut session =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            objects,
            owned_ledger,
        ) {
            Ok(session) => session,
            Err(error) => {
                let (error, recovered) = error.into_parts();
                *ledger = Some(recovered);
                return Err(doctor_store_error(error));
            }
        };
    for root in roots.keys() {
        if let Err(error) = poll() {
            *ledger = session.discard_and_take_decision_work_ledger()?;
            return Err(error);
        }
        let mut cancellation_error = None;
        let validation = session.validate_commit_recoverable_with_cancellation_and_error_handler(
            root,
            || match poll() {
                Ok(()) => false,
                Err(error) => {
                    cancellation_error = Some(error);
                    true
                }
            },
            #[expect(
                clippy::result_large_err,
                reason = "the recoverable Store validation handler requires StoreError"
            )]
            |error, active_ledger| {
                report.increment_counter(
                    DoctorReportCounter::InvalidSealedCommitReferences,
                    active_ledger,
                )?;
                report.push_issue(
                    "remote_sealed_commit_reference_invalid",
                    format_args!("{error}"),
                    Some(format_args!("{root}")),
                    active_ledger,
                )?;
                Ok(())
            },
        );
        if let Some(error) = cancellation_error {
            *ledger = session.discard_and_take_decision_work_ledger()?;
            return Err(error);
        }
        if let Err(error) = validation {
            *ledger = session.discard_and_take_decision_work_ledger()?;
            return Err(doctor_store_error(error));
        }
    }
    let mut cancellation_error = None;
    let finalized = session.finalize_with_cancellation(|| match poll() {
        Ok(()) => false,
        Err(error) => {
            cancellation_error = Some(error);
            true
        }
    });
    if let Err(error) = finalized {
        *ledger = session.take_decision_work_ledger()?;
        return Err(cancellation_error.unwrap_or_else(|| doctor_store_error(error)));
    }
    *ledger = session.take_decision_work_ledger()?;
    Ok(())
}

pub(crate) fn diagnose_remote_project_with_work_and_poll<F>(
    project_root: &Path,
    project_url: &str,
    transport: &str,
    quick: bool,
    ledger: &mut Option<codefire_core::DecisionWorkLedger>,
    mut poll: F,
) -> Result<AccountedRemoteDoctorReport, CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    poll()?;
    let mut report = AccountedRemoteDoctorReport::new(
        project_url,
        transport,
        quick,
        ledger
            .as_mut()
            .expect("remote doctor decision-work ledger must be present"),
    )?;
    let diagnosis = (|| -> Result<(), CliError> {
        let dirs = remote_dirs(project_root);
        {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must be present for initial report");
            report
                .set_project_exists(project_root.is_dir(), active_ledger)
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining project-exists state")
                })?;
            for (name, exists) in [
                ("objects", dirs.objects.is_dir()),
                ("branches", dirs.branches.is_dir()),
                ("merge_requests", dirs.merge_requests.is_dir()),
                ("locks", dirs.locks.is_dir()),
                ("idempotency", dirs.idempotency.is_dir()),
            ] {
                report
                    .insert_layout(name, exists, active_ledger)
                    .map_err(|error| doctor_accounting_error(error, "retaining doctor layout"))?;
            }
        }
        if !report.report.project_exists {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must be present for missing-project issue");
            report
                .push_issue(
                    "remote_project_missing",
                    format_args!("remote project does not exist"),
                    Some(format_args!("{project_url}")),
                    active_ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining missing-project issue")
                })?;
            return Ok(());
        }
        poll()?;
        let gate = match RemoteProjectGate::acquire_shared(project_root, &LockOptions::default()) {
            Ok(gate) => gate,
            Err(error) => {
                let active_ledger = ledger
                    .as_mut()
                    .expect("remote doctor ledger must be present for gate issue");
                report
                    .push_issue(
                        "remote_project_gate_invalid",
                        format_args!("{error}"),
                        Some(format_args!("{project_url}")),
                        active_ledger,
                    )
                    .map_err(|error| {
                        doctor_accounting_error(error, "retaining invalid project-gate issue")
                    })?;
                return Ok(());
            }
        };
        poll()?;
        if let Err(error) = gate.ensure_project(project_root) {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must be present for changed-gate issue");
            report
                .push_issue(
                    "remote_project_gate_invalid",
                    format_args!("{error}"),
                    Some(format_args!("{project_url}")),
                    active_ledger,
                )
                .map_err(|error| {
                    doctor_accounting_error(error, "retaining changed project-gate issue")
                })?;
            return Ok(());
        }
        for name in [
            "objects",
            "branches",
            "merge_requests",
            "locks",
            "idempotency",
        ] {
            poll()?;
            if !report.report.layout.get(name).copied().unwrap_or(false) {
                let active_ledger = ledger
                    .as_mut()
                    .expect("remote doctor ledger must be present for layout issue");
                report
                    .push_issue(
                        "remote_layout_missing",
                        format_args!("remote layout component is missing: {name}"),
                        Some(format_args!("{name}")),
                        active_ledger,
                    )
                    .map_err(|error| {
                        doctor_accounting_error(error, "retaining missing-layout issue")
                    })?;
            }
        }

        let mut roots = BTreeMap::new();
        let authority_result = {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must be present for authority scan");
            (|| -> Result<(), CliError> {
                inspect_branches_with_work(
                    &dirs,
                    &mut roots,
                    &mut report,
                    active_ledger,
                    &mut poll,
                )?;
                inspect_merge_requests_with_work(
                    &dirs,
                    &mut roots,
                    &mut report,
                    active_ledger,
                    &mut poll,
                )
            })()
        };
        if let Err(error) = authority_result {
            release_scanned_identifiers(
                roots,
                ledger
                    .as_mut()
                    .expect("remote doctor ledger must be present for authority cleanup"),
            )?;
            return Err(error);
        }
        if quick {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must be present for quick cleanup");
            let quick_result = report
                .push_skipped_check("object_hash_and_reference_graph", active_ledger)
                .map_err(|error| doctor_accounting_error(error, "retaining skipped doctor check"));
            release_scanned_identifiers(roots, active_ledger)?;
            quick_result?;
        } else {
            inspect_objects_and_roots_with_work(&dirs, roots, &mut report, ledger, &mut poll)?;
        }
        poll()?;
        let active_ledger = ledger
            .as_mut()
            .expect("remote doctor decision-work ledger must return before completion");
        report.finish(active_ledger).map_err(|error| {
            doctor_accounting_error(error, "finalizing doctor report ownership")
        })?;
        debug_assert_eq!(
            compact_json_len(report.report()).ok(),
            Some(report.retained_bytes())
        );
        Ok(())
    })();
    match diagnosis {
        Ok(()) => Ok(report),
        Err(error) => {
            let active_ledger = ledger
                .as_mut()
                .expect("remote doctor ledger must return before report cleanup");
            report.release_owner(active_ledger).map_err(|cleanup| {
                doctor_accounting_error(cleanup, "releasing failed doctor report")
            })?;
            Err(error)
        }
    }
}

fn sorted_json_paths(directory: &Path) -> Result<Vec<PathBuf>, CliError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn inspect_branches(
    dirs: &RemoteDirs,
    roots: &mut BTreeSet<String>,
    report: &mut RemoteDoctorReport,
) {
    let paths = match sorted_json_paths(&dirs.branches) {
        Ok(paths) => paths,
        Err(error) => {
            report.push_issue("remote_branch_directory_invalid", error.to_string(), None);
            return;
        }
    };
    report.branch_count = paths.len();
    for path in paths.into_iter().take(MAX_REMOTE_RECORDS) {
        let branch = match read_json(&path) {
            Ok(branch) => branch,
            Err(error) => {
                report.push_issue(
                    "remote_branch_record_invalid",
                    error.to_string(),
                    Some(path.display().to_string()),
                );
                continue;
            }
        };
        if let Err(error) = validate_record_version(&branch, "remote_branch") {
            report.invalid_record_versions += 1;
            report.push_issue(
                "remote_record_version_unsupported",
                error.to_string(),
                Some(path.display().to_string()),
            );
            continue;
        }
        match (
            branch.get("name").and_then(Value::as_str),
            branch.get("head").and_then(Value::as_str),
        ) {
            (Some(name), Some(head)) => {
                if let Err(error) = validate_remote_component(name, "branch") {
                    report.push_issue(
                        "remote_branch_record_invalid",
                        error.to_string(),
                        Some(path.display().to_string()),
                    );
                }
                roots.insert(head.to_string());
            }
            _ => report.push_issue(
                "remote_branch_record_invalid",
                "remote branch must contain string name and head",
                Some(path.display().to_string()),
            ),
        }
    }
    if report.branch_count > MAX_REMOTE_RECORDS {
        report.push_issue(
            "remote_record_budget_exceeded",
            format!(
                "remote branch count {} exceeds the supported inspection budget {MAX_REMOTE_RECORDS}",
                report.branch_count
            ),
            None,
        );
    }
}

fn inspect_merge_requests(
    dirs: &RemoteDirs,
    roots: &mut BTreeSet<String>,
    report: &mut RemoteDoctorReport,
) {
    let paths = match sorted_json_paths(&dirs.merge_requests) {
        Ok(paths) => paths,
        Err(error) => {
            report.push_issue(
                "remote_merge_request_directory_invalid",
                error.to_string(),
                None,
            );
            return;
        }
    };
    report.merge_request_count = paths.len();
    for path in paths.into_iter().take(MAX_REMOTE_RECORDS) {
        let mr = match read_json(&path) {
            Ok(mr) => mr,
            Err(error) => {
                report.push_issue(
                    "remote_merge_request_record_invalid",
                    error.to_string(),
                    Some(path.display().to_string()),
                );
                continue;
            }
        };
        if let Err(error) = validate_record_version(&mr, "remote_merge_request") {
            report.invalid_record_versions += 1;
            report.push_issue(
                "remote_record_version_unsupported",
                error.to_string(),
                Some(path.display().to_string()),
            );
            continue;
        }
        let valid_id = mr
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| validate_remote_component(id, "merge-request ID").is_ok());
        let valid_status = matches!(
            mr.get("status").and_then(Value::as_str),
            Some("open" | "approved" | "rejected" | "applied")
        );
        if !valid_id || !valid_status {
            report.push_issue(
                "remote_merge_request_record_invalid",
                "remote merge request has an invalid id or status",
                Some(path.display().to_string()),
            );
        }
        for field in ["source_head", "target_head_at_request", "applied_head"] {
            if let Some(head) = mr.get(field).and_then(Value::as_str) {
                roots.insert(head.to_string());
            } else if field != "applied_head" {
                report.push_issue(
                    "remote_merge_request_record_invalid",
                    format!("remote merge request is missing {field}"),
                    Some(path.display().to_string()),
                );
            }
        }
    }
    if report.merge_request_count > MAX_REMOTE_RECORDS {
        report.push_issue(
            "remote_record_budget_exceeded",
            format!(
                "remote merge-request count {} exceeds the supported inspection budget {MAX_REMOTE_RECORDS}",
                report.merge_request_count
            ),
            None,
        );
    }
}

fn inspect_objects(dirs: &RemoteDirs, report: &mut RemoteDoctorReport) {
    let mut object_ids = BTreeSet::new();
    let mut references = BTreeSet::new();
    let result = codefire_store::scan_object_store_record_documents(&dirs.objects, |entry| {
        report.checked_objects += 1;
        if report.checked_objects > MAX_REMOTE_RECORDS {
            return;
        }
        match entry.document {
            Ok(document) => {
                object_ids.insert(document.record.object_id.clone());
                references.extend(object_document_references(&document));
            }
            Err(error) => {
                report.object_hash_errors += 1;
                report.push_issue(
                    "remote_object_record_invalid",
                    error.to_string(),
                    Some(entry.path.display().to_string()),
                );
            }
        }
    });
    if let Err(error) = result {
        report.push_issue("remote_object_store_invalid", error.to_string(), None);
    }
    if report.checked_objects > MAX_REMOTE_RECORDS {
        report.push_issue(
            "remote_record_budget_exceeded",
            format!(
                "remote object count {} exceeds the supported inspection budget {MAX_REMOTE_RECORDS}",
                report.checked_objects
            ),
            None,
        );
    }
    for missing in references.difference(&object_ids) {
        report.missing_object_references += 1;
        report.push_issue(
            "remote_object_reference_missing",
            format!("remote object reference is missing: {missing}"),
            Some(missing.clone()),
        );
    }
}

fn inspect_sealed_roots(objects: &Path, roots: BTreeSet<String>, report: &mut RemoteDoctorReport) {
    let roots = roots
        .into_iter()
        .take(MAX_REMOTE_RECORDS)
        .collect::<Vec<_>>();
    if roots.is_empty() {
        return;
    }
    let mut session = match codefire_store::begin_sealed_commit_validation_session(objects) {
        Ok(session) => session,
        Err(error) => {
            let message = error.to_string();
            for root in roots {
                report.invalid_sealed_commit_references += 1;
                report.push_issue(
                    "remote_sealed_commit_reference_invalid",
                    message.clone(),
                    Some(root),
                );
            }
            return;
        }
    };
    for root in roots {
        if let Err(error) = session.validate_commit_recoverable(&root) {
            let terminal = matches!(
                error,
                codefire_store::StoreError::DecisionWorkAccounting(_)
                    | codefire_store::StoreError::ValidationCancelled
                    | codefire_store::StoreError::SealedValidationSubjectChanged { .. }
            );
            report.invalid_sealed_commit_references += 1;
            report.push_issue(
                "remote_sealed_commit_reference_invalid",
                error.to_string(),
                Some(root),
            );
            if terminal {
                return;
            }
        }
    }
    if let Err(error) = session.finalize() {
        report.push_issue("remote_object_store_invalid", error.to_string(), None);
    }
}

pub(crate) fn remote_doctor_data_json(report: &RemoteDoctorReport) -> Value {
    serde_json::to_value(report).expect("remote doctor report serialization is infallible")
}

pub(crate) fn remote_doctor_diagnostics_json(report: &RemoteDoctorReport) -> Vec<Value> {
    report
        .issues
        .iter()
        .map(|issue| {
            json!({
                "kind": issue.kind,
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "message": issue.message,
                "subject": issue.subject,
            })
        })
        .collect()
}

pub(crate) fn print_remote_doctor_report(report: &RemoteDoctorReport) {
    println!("CodeFire remote doctor");
    println!("project: {}", report.project);
    println!("transport: {}", report.transport);
    println!(
        "project: {}",
        if report.project_exists {
            "found"
        } else {
            "missing"
        }
    );
    for (name, present) in &report.layout {
        println!("{name}: {}", if *present { "ok" } else { "missing" });
    }
    for issue in &report.issues {
        println!("{}: {}", issue.kind, issue.message);
    }
    println!("branches: {}", report.branch_count);
    println!("merge_requests: {}", report.merge_request_count);
    println!("object hash errors: {}", report.object_hash_errors);
    println!(
        "missing object references: {}",
        report.missing_object_references
    );
    println!(
        "invalid sealed commit references: {}",
        report.invalid_sealed_commit_references
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_valid_commit(objects: &Path) -> String {
        let roots = [
            (
                "content_manifest",
                "content_manifest",
                json!({"type": "content_manifest", "version": 1, "entries": []}),
            ),
            (
                "atom_index",
                "atom_index",
                json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
            ),
            (
                "trace_graph",
                "trace_graph",
                json!({"type": "trace_graph", "version": 1, "links": []}),
            ),
            (
                "fire_delta",
                "fire_ledger",
                json!({"type": "fire_ledger", "version": 1, "fires": []}),
            ),
            (
                "resolution_ledger",
                "resolution_ledger",
                json!({"type": "resolution_ledger", "version": 1, "resolutions": []}),
            ),
            (
                "verification",
                "verification",
                json!({
                    "type": "verification",
                    "version": 1,
                    "result": "passed",
                    "trace_completeness_required": false,
                    "open_required_fires": 0,
                    "failed_checks": [],
                    "missing_required_links": [],
                    "stale_resolutions": [],
                    "missing_evidence_refs": [],
                    "duplicate_atom_ids": [],
                    "verified_at": "2026-07-20T00:00:00Z"
                }),
            ),
            (
                "policy",
                "policy",
                json!({
                    "type": "policy",
                    "version": 1,
                    "policy": {
                        "require_no_required_fires": true,
                        "require_no_stale_resolutions": true,
                        "require_trace_completeness": false,
                        "require_verification_success": true,
                        "reject_duplicate_atom_ids": true,
                        "no_change_required_requires_rationale": true,
                        "required_links": {},
                        "verification": []
                    }
                }),
            ),
        ]
        .into_iter()
        .map(|(root, kind, payload)| {
            let object_id = codefire_store::store_object(objects, kind, payload).unwrap();
            (root.to_string(), Value::String(object_id))
        })
        .collect::<serde_json::Map<_, _>>();
        let certificate = [
            ("result", json!("consistent")),
            ("open_required_fires", json!(0)),
            ("failed_checks", json!(0)),
            ("missing_required_links", json!(0)),
            ("stale_resolutions", json!(0)),
            ("missing_evidence_refs", json!(0)),
            ("duplicate_atom_ids", json!(0)),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect::<serde_json::Map<_, _>>();
        codefire_store::store_object(
            objects,
            "commit",
            codefire_store::commit_payload(Vec::new(), roots, certificate),
        )
        .unwrap()
    }

    fn commit_id_between(left: &str, right: &str) -> String {
        let (low, high) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        let prefix = "CF-COMMIT-";
        let low_suffix = low.strip_prefix(prefix).unwrap().as_bytes();
        let high_suffix = high.strip_prefix(prefix).unwrap().as_bytes();
        let mut candidate = low_suffix.to_vec();
        for index in (0..candidate.len()).rev() {
            let next = match candidate[index] {
                b'0'..=b'8' | b'a'..=b'e' => Some(candidate[index] + 1),
                b'9' => Some(b'a'),
                _ => None,
            };
            let Some(next) = next else {
                continue;
            };
            candidate[index] = next;
            candidate[index + 1..].fill(b'0');
            if candidate.as_slice() < high_suffix {
                return format!("{prefix}{}", String::from_utf8(candidate).unwrap());
            }
            candidate.copy_from_slice(low_suffix);
        }
        panic!("test commit IDs have no representable ID between them");
    }

    #[test]
    fn legacy_doctor_continues_one_session_after_a_corrupt_root() {
        let temp = tempdir().unwrap();
        let project_root = temp.path().join("project");
        ensure_remote_project_layout(&project_root).unwrap();
        let objects = remote_dirs(&project_root).objects;
        let base = write_valid_commit(&objects);
        let base_record = codefire_store::read_object_record(&objects, &base).unwrap();
        let child = codefire_store::store_object(
            &objects,
            "commit",
            codefire_store::commit_payload(
                vec![base.clone()],
                base_record.payload["roots"].as_object().unwrap().clone(),
                base_record.payload["certificate"]
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .unwrap();
        let corrupt = commit_id_between(&base, &child);
        fs::copy(
            objects.join("commits").join(format!("{base}.json")),
            objects.join("commits").join(format!("{corrupt}.json")),
        )
        .unwrap();
        for (branch, head) in [("base", &base), ("corrupt", &corrupt), ("child", &child)] {
            write_json_atomic(
                &remote_branch_path(&project_root, branch),
                &json!({"version": 1, "name": branch, "head": head}),
            )
            .unwrap();
        }

        let report = diagnose_remote_project(&project_root, "cf://local/org/app", "file", false);

        assert_eq!(report.invalid_sealed_commit_references, 1);
        let subjects = report
            .issues
            .iter()
            .filter(|issue| issue.kind == "remote_sealed_commit_reference_invalid")
            .map(|issue| issue.subject.as_deref().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(subjects, vec![corrupt.as_str()]);
    }
}
