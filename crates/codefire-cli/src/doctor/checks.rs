use super::{
    DoctorIssue, DoctorPendingAuthority, DoctorPendingAuthorityKind, DoctorPendingOperation,
    DoctorPendingRequestIdentity, DoctorReportAccountingTransfer,
};
use crate::close_transaction::doctor_close_transaction_summary_from_record;
use crate::open_transaction::doctor_open_transaction_summary_from_record;
use crate::repo_layout::{object_subdir_requirement, LayoutRequirement, REPO_DIRS};
use crate::CliError;
use cap_fs_ext::DirExt;
use serde_json::Value;
use std::fmt::{self, Write as _};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};

const ACTIVE_CONTROL_FILE_BYTE_LIMIT: u64 = 1024 * 1024;
const ACTIVE_DATA_FILE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;
const MAX_DOCTOR_IDENTITY_BYTES: usize = 256;
const DOCTOR_OWNER_FRAME_BYTES: u64 = codefire_core::LOGICAL_FRAME_BASE_BYTES;
const DOCTOR_TRANSACTION_PARSE_MULTIPLIER: u64 = 8;
const DOCTOR_TRANSACTION_CONFLICT_KIND: &str = "conflicting_repository_transactions";
const DOCTOR_TRANSACTION_CONFLICT_MESSAGE: &str = "simultaneous pending open and close transaction authorities require manual inspection before either request can recover";

const ACTIVE_STATE_FILES: &[&str] = &[
    "state.json",
    "fires.json",
    "resolutions.json",
    "scan.json",
    "verification.json",
    "commit_transaction.json",
    "state_transaction.json",
    crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE,
];

#[derive(Debug)]
pub(super) struct DoctorChecks {
    pub(super) objects: usize,
    pub(super) branches: usize,
    pub(super) opened: usize,
    pub(super) active_files: usize,
    pub(super) skipped_checks: Vec<String>,
    pub(super) issues: Vec<DoctorIssue>,
    pub(super) ledger: codefire_core::DecisionWorkLedger,
    pub(super) report_accounting: DoctorReportAccountingTransfer,
}

/// Keeps the closed repository-layout batch separate from later residual
/// collectors until an accounted final Vec can be allocated and populated.
#[must_use = "Doctor issue ownership must be transferred or cancelled"]
struct DoctorIssueCollection {
    layout_issues: Option<Vec<DoctorIssue>>,
    pretransaction_issues: Option<Vec<DoctorIssue>>,
    transaction_issues: Option<Vec<DoctorIssue>>,
    residual_issues: Option<Vec<DoctorIssue>>,
    accounted_retained_bytes: u64,
    accounted_residual_slots: u64,
    accounted_pretransaction_slots: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

struct DoctorLayoutIssueSpec<'a> {
    severity: super::DoctorSeverity,
    kind: &'a str,
    message_parts: &'a [&'a str],
    path_base: &'a Path,
    path_parts: &'a [&'a Path],
}

struct DoctorTransactionIssueSpec<'a> {
    severity: super::DoctorSeverity,
    kind: &'static str,
    message: &'static str,
    path: &'a Path,
}

struct DoctorObjectIssueSpec<'a> {
    severity: super::DoctorSeverity,
    kind: &'static str,
    message: fmt::Arguments<'a>,
    path: &'a Path,
}

impl DoctorIssueCollection {
    fn new(ledger: &mut codefire_core::DecisionWorkLedger) -> Result<Self, CliError> {
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    DOCTOR_OWNER_FRAME_BYTES,
                ),
            )
            .map_err(|error| doctor_issue_accounting_error(error, "retain Doctor issue vector"))?;
        Ok(Self {
            layout_issues: Some(Vec::new()),
            pretransaction_issues: Some(Vec::new()),
            transaction_issues: Some(Vec::new()),
            residual_issues: Some(Vec::new()),
            accounted_retained_bytes: DOCTOR_OWNER_FRAME_BYTES,
            accounted_residual_slots: 0,
            accounted_pretransaction_slots: 0,
            ledger_identity: ledger.identity(),
        })
    }

    fn ensure_ledger_identity(
        &self,
        ledger: &codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if ledger.matches_identity(self.ledger_identity) {
            Ok(())
        } else {
            Err(CliError::InvalidRepository(
                "Doctor issue collection belongs to a different decision-work ledger".to_string(),
            ))
        }
    }

    fn ensure_identity(
        &self,
        identity: codefire_core::DecisionWorkLedgerIdentity,
    ) -> Result<(), CliError> {
        if identity == self.ledger_identity {
            Ok(())
        } else {
            Err(CliError::InvalidRepository(
                "Doctor issue collection belongs to a different decision-work ledger".to_string(),
            ))
        }
    }

    fn residual_mut(
        &mut self,
        identity: codefire_core::DecisionWorkLedgerIdentity,
    ) -> Result<&mut Vec<DoctorIssue>, CliError> {
        self.ensure_identity(identity)?;
        Ok(self
            .residual_issues
            .as_mut()
            .expect("active Doctor issue collection retains its residual payload"))
    }

    #[cfg(test)]
    fn layout(&self) -> &[DoctorIssue] {
        self.layout_issues
            .as_deref()
            .expect("active Doctor issue collection retains its layout payload")
    }

    #[cfg(test)]
    fn transaction(&self) -> &[DoctorIssue] {
        self.transaction_issues
            .as_deref()
            .expect("active Doctor issue collection retains its transaction payload")
    }

    #[cfg(test)]
    fn pretransaction(&self) -> &[DoctorIssue] {
        self.pretransaction_issues
            .as_deref()
            .expect("active Doctor issue collection retains its pretransaction payload")
    }

    fn push_layout_issue<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorLayoutIssueSpec<'_>,
        after_reserve: &mut F,
    ) -> Result<(), CliError>
    where
        F: FnMut() -> Result<(), CliError>,
    {
        self.ensure_ledger_identity(ledger)?;
        let message_bytes = spec.message_parts.iter().try_fold(0usize, |total, part| {
            total
                .checked_add(part.len())
                .ok_or_else(doctor_issue_size_overflow)
        })?;
        let path_upper_bytes = spec.path_parts.iter().try_fold(
            spec.path_base.as_os_str().as_encoded_bytes().len(),
            |total, relative| {
                total
                    .checked_add(1)
                    .and_then(|value| {
                        value.checked_add(relative.as_os_str().as_encoded_bytes().len())
                    })
                    .ok_or_else(doctor_issue_size_overflow)
            },
        )?;
        let upper = doctor_issue_logical_bytes(spec.kind.len(), message_bytes, path_upper_bytes)?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, upper),
            )
            .map_err(|error| doctor_issue_accounting_error(error, "reserve Doctor layout issue"))?;

        let issue = (|| {
            after_reserve()?;
            self.layout_issues
                .as_mut()
                .expect("active Doctor issue collection retains its layout payload")
                .try_reserve(1)
                .map_err(doctor_issue_allocation_error)?;
            let kind = try_string_from_parts(&[spec.kind])?;
            let message = try_string_from_parts(spec.message_parts)?;
            let mut path = PathBuf::new();
            path.as_mut_os_string()
                .try_reserve_exact(path_upper_bytes)
                .map_err(doctor_issue_allocation_error)?;
            path.push(spec.path_base);
            for relative in spec.path_parts {
                path.push(relative);
            }
            Ok::<_, CliError>(DoctorIssue {
                severity: spec.severity,
                kind,
                message,
                path: Some(path),
                open_snapshot: None,
                pending_authority: None,
            })
        })();
        let issue = match issue {
            Ok(issue) => issue,
            Err(error) => {
                ledger.cancel_reserved(&permit).map_err(|cleanup| {
                    doctor_issue_accounting_error(cleanup, "cancel failed Doctor layout issue")
                })?;
                return Err(error);
            }
        };
        let actual = doctor_issue_logical_bytes(
            issue.kind.len(),
            issue.message.len(),
            issue
                .path
                .as_deref()
                .expect("layout issue retains its path")
                .as_os_str()
                .as_encoded_bytes()
                .len(),
        )?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(actual)
            .ok_or_else(doctor_issue_size_overflow)?;
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, actual),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel rejected Doctor layout issue")
            })?;
            return Err(doctor_issue_accounting_error(
                error,
                "materialize Doctor layout issue",
            ));
        }
        if actual < upper {
            ledger.cancel_reserved(&permit).map_err(|error| {
                doctor_issue_accounting_error(error, "release unused Doctor layout reservation")
            })?;
        }
        self.layout_issues
            .as_mut()
            .expect("active Doctor issue collection retains its layout payload")
            .push(issue);
        self.accounted_retained_bytes = next_retained;
        Ok(())
    }

    fn push_transaction_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorTransactionIssueSpec<'_>,
    ) -> Result<(), CliError> {
        self.ensure_ledger_identity(ledger)?;
        let path_bytes = spec.path.as_os_str().as_encoded_bytes().len();
        let retained = doctor_issue_logical_bytes(spec.kind.len(), spec.message.len(), path_bytes)?
            .checked_add(1)
            .ok_or_else(doctor_issue_size_overflow)?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(retained)
            .ok_or_else(doctor_issue_size_overflow)?;
        let next_slots = self
            .accounted_residual_slots
            .checked_add(1)
            .ok_or_else(doctor_issue_size_overflow)?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .map_err(|error| {
                doctor_issue_accounting_error(error, "reserve Doctor transaction conflict issue")
            })?;
        let issue = (|| {
            let kind = try_string_from_parts(&[spec.kind])?;
            let message = try_string_from_parts(&[spec.message])?;
            let path = try_path_from_parts(path_bytes, spec.path, &[])?;
            self.transaction_issues
                .as_mut()
                .expect("active Doctor issue collection retains its transaction payload")
                .try_reserve(1)
                .map_err(doctor_issue_allocation_error)?;
            Ok::<_, CliError>(DoctorIssue {
                severity: spec.severity,
                kind,
                message,
                path: Some(path),
                open_snapshot: None,
                pending_authority: None,
            })
        })();
        let issue = match issue {
            Ok(issue) => issue,
            Err(error) => {
                ledger.cancel_reserved(&permit).map_err(|cleanup| {
                    doctor_issue_accounting_error(
                        cleanup,
                        "cancel failed Doctor transaction conflict issue",
                    )
                })?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            ),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(
                    cleanup,
                    "cancel rejected Doctor transaction conflict issue",
                )
            })?;
            return Err(doctor_issue_accounting_error(
                error,
                "materialize Doctor transaction conflict issue",
            ));
        }
        self.transaction_issues
            .as_mut()
            .expect("active Doctor issue collection retains its transaction payload")
            .push(issue);
        self.accounted_retained_bytes = next_retained;
        self.accounted_residual_slots = next_slots;
        Ok(())
    }

    fn cancel(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        self.ensure_ledger_identity(ledger)?;
        drop(self.layout_issues.take());
        drop(self.pretransaction_issues.take());
        drop(self.transaction_issues.take());
        drop(self.residual_issues.take());
        let retained = self.accounted_retained_bytes;
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            )
            .map_err(|error| {
                doctor_issue_accounting_error(error, "cancel Doctor issue collection")
            })?;
        self.accounted_retained_bytes = 0;
        self.accounted_residual_slots = 0;
        self.accounted_pretransaction_slots = 0;
        Ok(())
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store diagnostic callback requires StoreError"
    )]
    fn push_accounted_object_scan_issue<H>(
        &mut self,
        work: codefire_store::DiagnosticScanWork<'_>,
        spec: DoctorObjectIssueSpec<'_>,
        after_publish: H,
    ) -> codefire_store::DiagnosticScanVisitOutcome
    where
        H: FnOnce(),
    {
        let (message_bytes, path_bytes, retained, next_retained, next_slots) =
            match self.prepare_object_issue_accounting(spec.kind, spec.message, spec.path) {
                Ok(accounting) => accounting,
                Err(error) => return work.fail(error),
            };
        work.publish_retained(retained, || {
            let issue = self.materialize_object_issue(spec, message_bytes, path_bytes)?;
            Ok((issue, move |issue| {
                self.pretransaction_issues
                    .as_mut()
                    .expect("active Doctor issue collection retains its pretransaction payload")
                    .push(issue);
                self.accounted_retained_bytes = next_retained;
                self.accounted_pretransaction_slots = next_slots;
                after_publish();
            }))
        })
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store provisional-ledger callback requires StoreError"
    )]
    fn push_accounted_object_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorObjectIssueSpec<'_>,
    ) -> Result<(), codefire_store::StoreError> {
        self.push_accounted_object_issue_with_hook(ledger, spec, || Ok(()))
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store provisional-ledger callback requires StoreError"
    )]
    fn push_accounted_object_issue_with_hook<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorObjectIssueSpec<'_>,
        before_slot_allocation: F,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnOnce() -> Result<(), codefire_store::StoreError>,
    {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor issue collection belongs to a different decision-work ledger".to_string(),
            ));
        }
        let (message_bytes, path_bytes, retained, next_retained, next_slots) =
            self.prepare_object_issue_accounting(spec.kind, spec.message, spec.path)?;
        let reservation = codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::PublicationPreflight,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained,
        );
        let mut permit = ledger.reserve(reservation)?;
        let issue = match self.materialize_object_issue_with_hook(
            spec,
            message_bytes,
            path_bytes,
            before_slot_allocation,
        ) {
            Ok(issue) => issue,
            Err(error) => {
                ledger.cancel_reserved(&permit)?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            ),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit)?;
            return Err(error.into());
        }
        self.pretransaction_issues
            .as_mut()
            .expect("active Doctor issue collection retains its pretransaction payload")
            .push(issue);
        self.accounted_retained_bytes = next_retained;
        self.accounted_pretransaction_slots = next_slots;
        Ok(())
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store diagnostic callback requires StoreError"
    )]
    fn prepare_object_issue_accounting(
        &self,
        kind: &str,
        message: fmt::Arguments<'_>,
        path: &Path,
    ) -> Result<(usize, usize, u64, u64, u64), codefire_store::StoreError> {
        let mut counter = DoctorFormatByteCounter::default();
        counter.write_fmt(message).map_err(|_| {
            codefire_store::StoreError::InvalidSealedCommit(
                "Doctor object diagnostic size overflow".to_string(),
            )
        })?;
        let message_bytes = counter.0;
        let path_bytes = path.as_os_str().as_encoded_bytes().len();
        let issue_bytes = doctor_issue_logical_bytes(kind.len(), message_bytes, path_bytes)
            .map_err(cli_error_to_branch_store_error)?;
        let retained = issue_bytes.checked_add(1).ok_or_else(|| {
            codefire_store::StoreError::InvalidSealedCommit(
                "Doctor object diagnostic ownership size overflow".to_string(),
            )
        })?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(retained)
            .ok_or_else(|| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor object diagnostic ownership size overflow".to_string(),
                )
            })?;
        let next_slots = self
            .accounted_pretransaction_slots
            .checked_add(1)
            .ok_or_else(|| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor object diagnostic slot ownership overflow".to_string(),
                )
            })?;
        Ok((
            message_bytes,
            path_bytes,
            retained,
            next_retained,
            next_slots,
        ))
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store diagnostic callback requires StoreError"
    )]
    fn materialize_object_issue(
        &mut self,
        spec: DoctorObjectIssueSpec<'_>,
        message_bytes: usize,
        path_bytes: usize,
    ) -> Result<DoctorIssue, codefire_store::StoreError> {
        self.materialize_object_issue_with_hook(spec, message_bytes, path_bytes, || Ok(()))
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store diagnostic callback requires StoreError"
    )]
    fn materialize_object_issue_with_hook<F>(
        &mut self,
        spec: DoctorObjectIssueSpec<'_>,
        message_bytes: usize,
        path_bytes: usize,
        before_slot_allocation: F,
    ) -> Result<DoctorIssue, codefire_store::StoreError>
    where
        F: FnOnce() -> Result<(), codefire_store::StoreError>,
    {
        let kind = try_string_from_parts(&[spec.kind]).map_err(cli_error_to_branch_store_error)?;
        let mut message = String::new();
        message.try_reserve_exact(message_bytes).map_err(|error| {
            codefire_store::StoreError::Io(std::io::Error::other(format!(
                "Doctor object diagnostic allocation failed: {error}"
            )))
        })?;
        {
            let mut bounded = DoctorBoundedFormatWriter {
                output: &mut message,
                expected_bytes: message_bytes,
                written_bytes: 0,
            };
            bounded.write_fmt(spec.message).map_err(|_| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor object diagnostic rendering exceeded its reserved size".to_string(),
                )
            })?;
            if bounded.written_bytes != message_bytes {
                return Err(codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor object diagnostic rendering changed after sizing".to_string(),
                ));
            }
        }
        let path = try_path_from_parts(path_bytes, spec.path, &[])
            .map_err(cli_error_to_branch_store_error)?;
        before_slot_allocation()?;
        // This is deliberately the last fallible allocation. A failed String
        // or Path allocation therefore cannot leave spare Vec capacity after
        // the scoped retained reservation is cancelled, while a failed
        // `try_reserve` leaves the Vec unchanged by contract. Once it succeeds,
        // the immediate push below cannot allocate.
        self.pretransaction_issues
            .as_mut()
            .expect("active Doctor issue collection retains its pretransaction payload")
            .try_reserve(1)
            .map_err(|error| {
                codefire_store::StoreError::Io(std::io::Error::other(format!(
                    "Doctor object diagnostic vector allocation failed: {error}"
                )))
            })?;
        Ok(DoctorIssue {
            severity: spec.severity,
            kind,
            message,
            path: Some(path),
            open_snapshot: None,
            pending_authority: None,
        })
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store provisional-ledger callback requires StoreError"
    )]
    fn push_accounted_opened_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorObjectIssueSpec<'_>,
    ) -> Result<(), codefire_store::StoreError> {
        self.push_accounted_opened_issue_with_hook(ledger, spec, || Ok(()))
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store provisional-ledger callback requires StoreError"
    )]
    fn push_accounted_opened_issue_with_hook<F>(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        spec: DoctorObjectIssueSpec<'_>,
        before_slot_allocation: F,
    ) -> Result<(), codefire_store::StoreError>
    where
        F: FnOnce() -> Result<(), codefire_store::StoreError>,
    {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor issue collection belongs to a different decision-work ledger".to_string(),
            ));
        }
        let mut counter = DoctorFormatByteCounter::default();
        counter.write_fmt(spec.message).map_err(|_| {
            codefire_store::StoreError::InvalidSealedCommit(
                "Doctor opened diagnostic size overflow".to_string(),
            )
        })?;
        let message_bytes = counter.0;
        let path_bytes = spec.path.as_os_str().as_encoded_bytes().len();
        let issue_bytes = doctor_issue_logical_bytes(spec.kind.len(), message_bytes, path_bytes)
            .map_err(cli_error_to_branch_store_error)?;
        let retained = issue_bytes.checked_add(1).ok_or_else(|| {
            codefire_store::StoreError::InvalidSealedCommit(
                "Doctor opened diagnostic ownership size overflow".to_string(),
            )
        })?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(retained)
            .ok_or_else(|| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor opened diagnostic ownership size overflow".to_string(),
                )
            })?;
        let next_slots = self
            .accounted_residual_slots
            .checked_add(1)
            .ok_or_else(|| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor opened diagnostic slot ownership overflow".to_string(),
                )
            })?;
        let mut permit = ledger.reserve(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            ),
        )?;
        let issue = (|| {
            let kind =
                try_string_from_parts(&[spec.kind]).map_err(cli_error_to_branch_store_error)?;
            let mut message = String::new();
            message.try_reserve_exact(message_bytes).map_err(|error| {
                codefire_store::StoreError::Io(std::io::Error::other(format!(
                    "Doctor opened diagnostic allocation failed: {error}"
                )))
            })?;
            {
                let mut bounded = DoctorBoundedFormatWriter {
                    output: &mut message,
                    expected_bytes: message_bytes,
                    written_bytes: 0,
                };
                bounded.write_fmt(spec.message).map_err(|_| {
                    codefire_store::StoreError::InvalidSealedCommit(
                        "Doctor opened diagnostic rendering exceeded its reserved size".to_string(),
                    )
                })?;
                if bounded.written_bytes != message_bytes {
                    return Err(codefire_store::StoreError::InvalidSealedCommit(
                        "Doctor opened diagnostic rendering changed after sizing".to_string(),
                    ));
                }
            }
            let path = try_path_from_parts(path_bytes, spec.path, &[])
                .map_err(cli_error_to_branch_store_error)?;
            before_slot_allocation()?;
            // Keep the destination allocation last so every earlier failure
            // leaves the residual Vec unchanged when the permit is cancelled.
            self.residual_issues
                .as_mut()
                .expect("active Doctor issue collection retains its residual payload")
                .try_reserve(1)
                .map_err(|error| {
                    codefire_store::StoreError::Io(std::io::Error::other(format!(
                        "Doctor opened diagnostic vector allocation failed: {error}"
                    )))
                })?;
            Ok::<_, codefire_store::StoreError>(DoctorIssue {
                severity: spec.severity,
                kind,
                message,
                path: Some(path),
                open_snapshot: None,
                pending_authority: None,
            })
        })();
        let issue = match issue {
            Ok(issue) => issue,
            Err(error) => {
                ledger.cancel_reserved(&permit)?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            ),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit)?;
            return Err(error.into());
        }
        self.residual_issues
            .as_mut()
            .expect("active Doctor issue collection retains its residual payload")
            .push(issue);
        self.accounted_retained_bytes = next_retained;
        self.accounted_residual_slots = next_slots;
        Ok(())
    }

    fn push_accounted_residual_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        issue: DoctorIssue,
    ) -> Result<(), CliError> {
        self.ensure_ledger_identity(ledger)?;
        let path_bytes = issue
            .path
            .as_deref()
            .map(|path| path.as_os_str().as_encoded_bytes().len())
            .unwrap_or_default();
        let issue_bytes =
            doctor_issue_logical_bytes(issue.kind.len(), issue.message.len(), path_bytes)?;
        let retained = issue_bytes
            .checked_add(1)
            .ok_or_else(doctor_issue_size_overflow)?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(retained)
            .ok_or_else(doctor_issue_size_overflow)?;
        let next_slots = self
            .accounted_residual_slots
            .checked_add(1)
            .ok_or_else(doctor_issue_size_overflow)?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .map_err(|error| {
                doctor_issue_accounting_error(error, "reserve Doctor transaction issue")
            })?;
        if let Err(error) = self
            .transaction_issues
            .as_mut()
            .expect("active Doctor issue collection retains its transaction payload")
            .try_reserve(1)
        {
            drop(issue);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel Doctor transaction issue")
            })?;
            return Err(doctor_issue_allocation_error(error));
        }
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            ),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel rejected Doctor transaction issue")
            })?;
            return Err(doctor_issue_accounting_error(
                error,
                "materialize Doctor transaction issue",
            ));
        }
        self.transaction_issues
            .as_mut()
            .expect("active Doctor issue collection retains its transaction payload")
            .push(issue);
        self.accounted_retained_bytes = next_retained;
        self.accounted_residual_slots = next_slots;
        Ok(())
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store validation callbacks require StoreError"
    )]
    fn push_accounted_branch_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        severity: super::DoctorSeverity,
        kind: &str,
        message: fmt::Arguments<'_>,
        path: &Path,
    ) -> Result<(), codefire_store::StoreError> {
        let index = self
            .residual_issues
            .as_ref()
            .expect("active Doctor issue collection retains its residual payload")
            .len();
        self.insert_accounted_branch_issue(ledger, index, severity, kind, message, path)
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store validation callbacks require StoreError"
    )]
    fn insert_accounted_branch_issue(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        index: usize,
        severity: super::DoctorSeverity,
        kind: &str,
        message: fmt::Arguments<'_>,
        path: &Path,
    ) -> Result<(), codefire_store::StoreError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(codefire_store::StoreError::InvalidSealedCommit(
                "Doctor issue collection belongs to a different decision-work ledger".to_string(),
            ));
        }
        let mut counter = DoctorFormatByteCounter::default();
        counter
            .write_fmt(message)
            .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        let message_bytes = counter.0;
        let path_bytes = path.as_os_str().as_encoded_bytes().len();
        let issue_bytes = DOCTOR_OWNER_FRAME_BYTES
            .checked_add(
                u64::try_from(kind.len())
                    .map_err(|_| codefire_core::DecisionWorkAccountingError::SizeOverflow)?,
            )
            .and_then(|value| value.checked_add(u64::try_from(message_bytes).ok()?))
            .and_then(|value| value.checked_add(u64::try_from(path_bytes).ok()?))
            .and_then(|value| value.checked_add(1))
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        let next_retained = self
            .accounted_retained_bytes
            .checked_add(issue_bytes)
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        let next_slots = self
            .accounted_residual_slots
            .checked_add(1)
            .ok_or(codefire_core::DecisionWorkAccountingError::SizeOverflow)?;
        let mut permit = ledger.reserve(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                issue_bytes,
            ),
        )?;
        let issue = (|| {
            let residual = self
                .residual_issues
                .as_mut()
                .expect("active Doctor issue collection retains its residual payload");
            if index > residual.len() {
                return Err(codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor branch issue insertion index is outside the residual report"
                        .to_string(),
                ));
            }
            residual.try_reserve(1).map_err(|error| {
                codefire_store::StoreError::Io(std::io::Error::other(format!(
                    "Doctor branch issue vector allocation failed: {error}"
                )))
            })?;
            let kind = try_string_from_parts(&[kind]).map_err(cli_error_to_branch_store_error)?;
            let mut rendered = String::new();
            rendered.try_reserve_exact(message_bytes).map_err(|error| {
                codefire_store::StoreError::Io(std::io::Error::other(format!(
                    "Doctor branch diagnostic allocation failed: {error}"
                )))
            })?;
            rendered.write_fmt(message).map_err(|_| {
                codefire_store::StoreError::InvalidSealedCommit(
                    "Doctor branch diagnostic rendering failed".to_string(),
                )
            })?;
            let path = try_path_from_parts(path_bytes, path, &[])
                .map_err(cli_error_to_branch_store_error)?;
            Ok::<_, codefire_store::StoreError>(DoctorIssue {
                severity,
                kind,
                message: rendered,
                path: Some(path),
                open_snapshot: None,
                pending_authority: None,
            })
        })();
        let issue = match issue {
            Ok(issue) => issue,
            Err(error) => {
                ledger.cancel_reserved(&permit)?;
                return Err(error);
            }
        };
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                issue_bytes,
            ),
        ) {
            drop(issue);
            ledger.cancel_reserved(&permit)?;
            return Err(error.into());
        }
        self.residual_issues
            .as_mut()
            .expect("active Doctor issue collection retains its residual payload")
            .insert(index, issue);
        self.accounted_retained_bytes = next_retained;
        self.accounted_residual_slots = next_slots;
        Ok(())
    }

    fn residual_len(
        &self,
        identity: codefire_core::DecisionWorkLedgerIdentity,
    ) -> Result<usize, CliError> {
        self.ensure_identity(identity)?;
        Ok(self
            .residual_issues
            .as_ref()
            .expect("active Doctor issue collection retains its residual payload")
            .len())
    }

    fn finalize_report_issues(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(Vec<DoctorIssue>, DoctorReportAccountingTransfer), CliError> {
        self.ensure_ledger_identity(ledger)?;
        let layout_len = self
            .layout_issues
            .as_ref()
            .expect("active Doctor issue collection retains its layout payload")
            .len();
        let residual_len = self
            .residual_issues
            .as_ref()
            .expect("active Doctor issue collection retains its residual payload")
            .len();
        let pretransaction_len = self
            .pretransaction_issues
            .as_ref()
            .expect("active Doctor issue collection retains its pretransaction payload")
            .len();
        let transaction_len = self
            .transaction_issues
            .as_ref()
            .expect("active Doctor issue collection retains its transaction payload")
            .len();
        let total_len = layout_len
            .checked_add(pretransaction_len)
            .and_then(|value| value.checked_add(transaction_len))
            .and_then(|value| value.checked_add(residual_len))
            .ok_or_else(doctor_issue_size_overflow)?;
        let old_vector_bytes = DOCTOR_OWNER_FRAME_BYTES
            .checked_add(u64::try_from(layout_len).map_err(|_| doctor_issue_size_overflow())?)
            .and_then(|value| value.checked_add(self.accounted_pretransaction_slots))
            .and_then(|value| value.checked_add(self.accounted_residual_slots))
            .ok_or_else(doctor_issue_size_overflow)?;
        let final_vector_bytes = DOCTOR_OWNER_FRAME_BYTES
            .checked_add(u64::try_from(total_len).map_err(|_| doctor_issue_size_overflow())?)
            .ok_or_else(doctor_issue_size_overflow)?;
        let final_retained = self
            .accounted_retained_bytes
            .checked_sub(old_vector_bytes)
            .and_then(|value| value.checked_add(final_vector_bytes))
            .ok_or_else(doctor_issue_size_overflow)?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    final_vector_bytes,
                ),
            )
            .map_err(|error| {
                doctor_issue_accounting_error(error, "reserve final Doctor issue vector")
            })?;
        let mut issues = Vec::new();
        if let Err(error) = issues.try_reserve_exact(total_len) {
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel failed final Doctor issue vector")
            })?;
            return Err(doctor_issue_allocation_error(error));
        }
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                final_vector_bytes,
            ),
        ) {
            drop(issues);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel rejected final Doctor issue vector")
            })?;
            return Err(doctor_issue_accounting_error(
                error,
                "materialize final Doctor issue vector",
            ));
        }
        let mut layout = self
            .layout_issues
            .take()
            .expect("active Doctor issue collection retains its layout payload");
        let mut pretransaction = self
            .pretransaction_issues
            .take()
            .expect("active Doctor issue collection retains its pretransaction payload");
        let mut transaction = self
            .transaction_issues
            .take()
            .expect("active Doctor issue collection retains its transaction payload");
        let mut residual = self
            .residual_issues
            .take()
            .expect("active Doctor issue collection retains its residual payload");
        issues.append(&mut layout);
        issues.append(&mut pretransaction);
        issues.append(&mut transaction);
        issues.append(&mut residual);
        drop(layout);
        drop(pretransaction);
        drop(transaction);
        drop(residual);
        if let Err(error) = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            old_vector_bytes,
        ) {
            drop(issues);
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    final_vector_bytes,
                )
                .map_err(|cleanup| {
                    doctor_issue_accounting_error(
                        cleanup,
                        "release rejected final Doctor issue vector",
                    )
                })?;
            return Err(doctor_issue_accounting_error(
                error,
                "transfer final Doctor issue vector",
            ));
        }
        self.accounted_retained_bytes = 0;
        self.accounted_residual_slots = 0;
        self.accounted_pretransaction_slots = 0;
        Ok((
            issues,
            DoctorReportAccountingTransfer::new(self.ledger_identity, final_retained),
        ))
    }
}

#[derive(Default)]
struct DoctorFormatByteCounter(usize);

impl fmt::Write for DoctorFormatByteCounter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0 = self.0.checked_add(value.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}

struct DoctorBoundedFormatWriter<'a> {
    output: &'a mut String,
    expected_bytes: usize,
    written_bytes: usize,
}

impl fmt::Write for DoctorBoundedFormatWriter<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let next = self
            .written_bytes
            .checked_add(value.len())
            .ok_or(fmt::Error)?;
        if next > self.expected_bytes {
            return Err(fmt::Error);
        }
        self.output.push_str(value);
        self.written_bytes = next;
        Ok(())
    }
}

fn cli_error_to_branch_store_error(error: CliError) -> codefire_store::StoreError {
    match error {
        CliError::Io(error) => codefire_store::StoreError::Io(error),
        CliError::Json(error) => codefire_store::StoreError::Json(error),
        error => codefire_store::StoreError::InvalidSealedCommit(error.to_string()),
    }
}

impl Drop for DoctorIssueCollection {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.accounted_retained_bytes, 0,
            "Doctor issue collection dropped without ledger transfer or cancellation"
        );
    }
}

#[must_use = "Doctor path ownership must be explicitly released"]
struct AccountedDoctorPath {
    path: Option<PathBuf>,
    temporary_bytes: u64,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

impl AccountedDoctorPath {
    fn prepare(
        ledger: &mut codefire_core::DecisionWorkLedger,
        base: &Path,
        parts: &[&Path],
    ) -> Result<Self, CliError> {
        let path_upper_bytes = path_upper_bytes(base, parts)?;
        let upper = DOCTOR_OWNER_FRAME_BYTES
            .checked_add(u64::try_from(path_upper_bytes).map_err(|_| doctor_issue_size_overflow())?)
            .ok_or_else(doctor_issue_size_overflow)?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, upper),
            )
            .map_err(|error| doctor_issue_accounting_error(error, "reserve Doctor check path"))?;
        let path = try_path_from_parts(path_upper_bytes, base, parts);
        let path = match path {
            Ok(path) => path,
            Err(error) => {
                ledger.cancel_reserved(&permit).map_err(|cleanup| {
                    doctor_issue_accounting_error(cleanup, "cancel failed Doctor check path")
                })?;
                return Err(error);
            }
        };
        let actual = DOCTOR_OWNER_FRAME_BYTES
            .checked_add(
                u64::try_from(path.as_os_str().as_encoded_bytes().len())
                    .map_err(|_| doctor_issue_size_overflow())?,
            )
            .ok_or_else(doctor_issue_size_overflow)?;
        if let Err(error) = ledger.consume_reserved(
            &mut permit,
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, actual),
        ) {
            drop(path);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_issue_accounting_error(cleanup, "cancel rejected Doctor check path")
            })?;
            return Err(doctor_issue_accounting_error(
                error,
                "materialize Doctor check path",
            ));
        }
        if actual < upper {
            ledger.cancel_reserved(&permit).map_err(|error| {
                doctor_issue_accounting_error(error, "release unused Doctor check path reservation")
            })?;
        }
        Ok(Self {
            path: Some(path),
            temporary_bytes: actual,
            ledger_identity: ledger.identity(),
        })
    }

    fn as_path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("active Doctor path owner retains its path")
    }

    fn release(&mut self, ledger: &mut codefire_core::DecisionWorkLedger) -> Result<(), CliError> {
        if !ledger.matches_identity(self.ledger_identity) {
            return Err(CliError::InvalidRepository(
                "Doctor check path belongs to a different decision-work ledger".to_string(),
            ));
        }
        drop(self.path.take());
        let amount = self.temporary_bytes;
        ledger
            .release(codefire_core::DecisionWorkDimension::TemporaryBytes, amount)
            .map_err(|error| doctor_issue_accounting_error(error, "release Doctor check path"))?;
        self.temporary_bytes = 0;
        Ok(())
    }
}

impl Drop for AccountedDoctorPath {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.temporary_bytes, 0,
            "Doctor check path dropped without ledger release"
        );
    }
}

fn path_upper_bytes(base: &Path, parts: &[&Path]) -> Result<usize, CliError> {
    parts.iter().try_fold(
        base.as_os_str().as_encoded_bytes().len(),
        |total, relative| {
            total
                .checked_add(1)
                .and_then(|value| value.checked_add(relative.as_os_str().as_encoded_bytes().len()))
                .ok_or_else(doctor_issue_size_overflow)
        },
    )
}

fn try_path_from_parts(
    upper_bytes: usize,
    base: &Path,
    parts: &[&Path],
) -> Result<PathBuf, CliError> {
    let mut path = PathBuf::new();
    path.as_mut_os_string()
        .try_reserve_exact(upper_bytes)
        .map_err(doctor_issue_allocation_error)?;
    path.push(base);
    for part in parts {
        path.push(part);
    }
    Ok(path)
}

fn try_string_from_parts(parts: &[&str]) -> Result<String, CliError> {
    let bytes = parts.iter().try_fold(0usize, |total, part| {
        total
            .checked_add(part.len())
            .ok_or_else(doctor_issue_size_overflow)
    })?;
    let mut value = String::new();
    value
        .try_reserve_exact(bytes)
        .map_err(doctor_issue_allocation_error)?;
    for part in parts {
        value.push_str(part);
    }
    Ok(value)
}

fn doctor_issue_logical_bytes(
    kind_bytes: usize,
    message_bytes: usize,
    path_bytes: usize,
) -> Result<u64, CliError> {
    let content = kind_bytes
        .checked_add(message_bytes)
        .and_then(|value| value.checked_add(path_bytes))
        .ok_or_else(doctor_issue_size_overflow)?;
    let content = u64::try_from(content).map_err(|_| doctor_issue_size_overflow())?;
    let frames = DOCTOR_OWNER_FRAME_BYTES
        .checked_mul(3)
        .ok_or_else(doctor_issue_size_overflow)?;
    content
        .checked_add(1)
        .and_then(|value| value.checked_add(frames))
        .ok_or_else(doctor_issue_size_overflow)
}

fn doctor_issue_size_overflow() -> CliError {
    CliError::InvalidRepository("Doctor issue ownership size overflow".to_string())
}

fn doctor_issue_allocation_error(error: std::collections::TryReserveError) -> CliError {
    CliError::InvalidRepository(format!("Doctor issue allocation failed: {error}"))
}

fn doctor_issue_accounting_error(
    error: codefire_core::DecisionWorkAccountingError,
    operation: &str,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "Doctor issue accounting failed while {operation}: {error}"
        )),
    }
}

pub(super) fn run_checks(cf: &Path, quick: bool) -> Result<DoctorChecks, CliError> {
    run_checks_with_sealed_validation_hook(cf, quick, || Ok(()))
}

fn run_checks_with_sealed_validation_hook<F>(
    cf: &Path,
    quick: bool,
    before_finalize: F,
) -> Result<DoctorChecks, CliError>
where
    F: FnOnce() -> Result<(), CliError>,
{
    let mut ledger = codefire_core::DecisionWorkLedger::new(
        codefire_core::DecisionWorkProfile::local_v1(),
        codefire_core::DecisionWorkCommandKind::LocalGraph,
        "doctor",
    )
    .map_err(|error| doctor_store_error(error.into()))?;
    let mut issues = DoctorIssueCollection::new(&mut ledger)?;
    if let Err(error) = check_repo_layout(cf, &mut issues, &mut ledger) {
        cancel_doctor_issue_run(
            &mut issues,
            &mut ledger,
            terminal_reason_for_cli_error(&error),
        )?;
        return Err(error);
    }
    let open_present = match check_open_transaction(cf, &mut issues, &mut ledger) {
        Ok(present) => present,
        Err(error) => {
            cancel_doctor_issue_run(
                &mut issues,
                &mut ledger,
                terminal_reason_for_cli_error(&error),
            )?;
            return Err(error);
        }
    };
    let close_present = match check_close_transaction(cf, &mut issues, &mut ledger) {
        Ok(present) => present,
        Err(error) => {
            cancel_doctor_issue_run(
                &mut issues,
                &mut ledger,
                terminal_reason_for_cli_error(&error),
            )?;
            return Err(error);
        }
    };
    if let Err(error) =
        check_transaction_conflict(open_present, close_present, cf, &mut issues, &mut ledger)
    {
        cancel_doctor_issue_run(
            &mut issues,
            &mut ledger,
            terminal_reason_for_cli_error(&error),
        )?;
        return Err(error);
    }
    let mut skipped_checks = Vec::new();
    let mut sealed_validation = match DoctorSealedValidation::new(&cf.join("objects"), ledger) {
        Ok(validation) => validation,
        Err(start) => {
            let start = *start;
            let mut ledger = start.ledger;
            cancel_doctor_issue_run(
                &mut issues,
                &mut ledger,
                terminal_reason_for_cli_error(&start.error),
            )?;
            return Err(start.error);
        }
    };
    let result = (|| {
        let ledger_identity = sealed_validation.ledger_identity();
        let objects = if quick {
            skipped_checks.push("object_store_integrity".to_string());
            0
        } else {
            sealed_validation.scan_object_records(&mut issues)?
        };
        check_branch_heads(cf, &mut issues, &mut sealed_validation)?;
        let opened = check_opened_registry(cf, &mut issues, &mut sealed_validation)?;
        let active_files = check_active_state_files(cf, issues.residual_mut(ledger_identity)?)?;
        before_finalize()?;
        let (validated_branches, ledger) = sealed_validation.finish(&mut issues)?;
        Ok::<_, CliError>((objects, opened, active_files, validated_branches, ledger))
    })();
    let (objects, opened, active_files, validated_branches, ledger) = match result {
        Ok(result) => result,
        Err(error) => {
            let reason = terminal_reason_for_cli_error(&error);
            let mut ledger = sealed_validation.abort_and_take_ledger(reason)?;
            cancel_doctor_issue_run(&mut issues, &mut ledger, reason)?;
            return Err(error);
        }
    };
    ensure_doctor_ledger_report_retained(&ledger, issues.accounted_retained_bytes)?;
    let mut ledger = ledger;
    let (final_issues, report_accounting) = match issues.finalize_report_issues(&mut ledger) {
        Ok(parts) => parts,
        Err(error) => {
            cancel_doctor_issue_run(
                &mut issues,
                &mut ledger,
                terminal_reason_for_cli_error(&error),
            )?;
            return Err(error);
        }
    };
    let preaccounted_report_bytes = report_accounting.retained_bytes_for(&ledger)?;
    ensure_doctor_ledger_report_retained(&ledger, preaccounted_report_bytes)?;
    Ok(DoctorChecks {
        objects,
        branches: validated_branches,
        opened,
        active_files,
        skipped_checks,
        issues: final_issues,
        ledger,
        report_accounting,
    })
}

fn check_transaction_conflict(
    open_present: bool,
    close_present: bool,
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    issues.ensure_ledger_identity(ledger)?;
    if !open_present || !close_present {
        return Ok(());
    }
    issues.push_transaction_issue(
        ledger,
        DoctorTransactionIssueSpec {
            severity: super::DoctorSeverity::Error,
            kind: DOCTOR_TRANSACTION_CONFLICT_KIND,
            message: DOCTOR_TRANSACTION_CONFLICT_MESSAGE,
            path: cf,
        },
    )
}

fn check_close_transaction(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<bool, CliError> {
    check_close_transaction_with_hooks(cf, issues, ledger, &mut || {}, &mut || {})
}

fn check_close_transaction_with_hooks<B, A>(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_read: &mut B,
    after_read: &mut A,
) -> Result<bool, CliError>
where
    B: FnMut(),
    A: FnMut(),
{
    let presence = check_transaction_marker_with_hooks(
        cf,
        issues,
        ledger,
        DoctorTransactionMarkerSpec {
            file_name: "close_transaction.json",
            reported_file_name: "close_transaction.json",
            conflicting_file_name: None,
            marker_limit: crate::close_transaction::CLOSE_TRANSACTION_BYTE_LIMIT,
            pending_kind: "pending_close_transaction",
            invalid_kind: "invalid_close_transaction",
            subject: "close",
            invalid_json_prefix: "",
            invalid_json_detail: "marker is not valid JSON",
            reserve_context: "reserve Doctor close transaction inspection",
            cancel_context: "cancel Doctor close transaction inspection",
            account_context: "account Doctor close transaction inspection",
            release_context: "release Doctor close transaction inspection",
            unused_context: "cancel unused close transaction reservation",
        },
        doctor_close_transaction_summary_from_record,
        before_read,
        after_read,
    )?;
    Ok(presence.primary)
}

fn check_open_transaction(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<bool, CliError> {
    check_open_transaction_with_hooks(cf, issues, ledger, &mut || {}, &mut || {})
}

fn check_open_transaction_with_hooks<B, A>(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
    before_read: &mut B,
    after_read: &mut A,
) -> Result<bool, CliError>
where
    B: FnMut(),
    A: FnMut(),
{
    let presence = check_transaction_marker_with_hooks(
        cf,
        issues,
        ledger,
        DoctorTransactionMarkerSpec {
            file_name: "open_transaction.json",
            reported_file_name: "open_transaction.json",
            conflicting_file_name: Some("open_transaction.removing.json"),
            marker_limit: crate::open_transaction::OPEN_TRANSACTION_BYTE_LIMIT,
            pending_kind: "pending_open_transaction",
            invalid_kind: "invalid_open_transaction",
            subject: "open",
            invalid_json_prefix: "invalid CodeFire repository: ",
            invalid_json_detail: "invalid JSON",
            reserve_context: "reserve Doctor open transaction inspection",
            cancel_context: "cancel Doctor open transaction inspection",
            account_context: "account Doctor open transaction inspection",
            release_context: "release Doctor open transaction inspection",
            unused_context: "cancel unused open transaction reservation",
        },
        doctor_open_transaction_summary_from_record,
        before_read,
        after_read,
    )?;
    Ok(presence.primary)
}

#[derive(Clone, Copy)]
struct DoctorTransactionMarkerSpec {
    file_name: &'static str,
    reported_file_name: &'static str,
    conflicting_file_name: Option<&'static str>,
    marker_limit: u64,
    pending_kind: &'static str,
    invalid_kind: &'static str,
    subject: &'static str,
    invalid_json_prefix: &'static str,
    invalid_json_detail: &'static str,
    reserve_context: &'static str,
    cancel_context: &'static str,
    account_context: &'static str,
    release_context: &'static str,
    unused_context: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DoctorTransactionMarkerPresence {
    primary: bool,
    companion: bool,
}

fn check_transaction_marker_with_hooks<S, B, A>(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
    spec: DoctorTransactionMarkerSpec,
    summarize: S,
    before_read: &mut B,
    after_read: &mut A,
) -> Result<DoctorTransactionMarkerPresence, CliError>
where
    S: FnOnce(&Path, PathBuf, Value) -> Result<String, CliError>,
    B: FnMut(),
    A: FnMut(),
{
    issues.ensure_ledger_identity(ledger)?;
    let Some(repo_root) = cf.parent() else {
        return Ok(DoctorTransactionMarkerPresence::default());
    };
    let mut path = AccountedDoctorPath::prepare(ledger, cf, &[Path::new(spec.file_name)])?;
    let mut conflicting_path = if let Some(file_name) = spec.conflicting_file_name {
        match AccountedDoctorPath::prepare(ledger, cf, &[Path::new(file_name)]) {
            Ok(candidate) => Some(candidate),
            Err(error) => {
                path.release(ledger)?;
                return Err(error);
            }
        }
    } else {
        None
    };
    let reported_path_upper = match path_upper_bytes(cf, &[Path::new(spec.reported_file_name)]) {
        Ok(upper) => upper,
        Err(error) => {
            if let Some(conflicting) = conflicting_path.as_mut() {
                conflicting.release(ledger)?;
            }
            path.release(ledger)?;
            return Err(error);
        }
    };
    let temporary_upper = match spec
        .marker_limit
        .checked_mul(DOCTOR_TRANSACTION_PARSE_MULTIPLIER)
        .and_then(|value| value.checked_add(64 * 1024))
    {
        Some(upper) => upper,
        None => {
            if let Some(conflicting) = conflicting_path.as_mut() {
                conflicting.release(ledger)?;
            }
            path.release(ledger)?;
            return Err(doctor_issue_size_overflow());
        }
    };
    let candidate_count = if conflicting_path.is_some() { 2 } else { 1 };
    let aggregate_upper = match spec.marker_limit.checked_mul(candidate_count) {
        Some(upper) => upper,
        None => {
            if let Some(conflicting) = conflicting_path.as_mut() {
                conflicting.release(ledger)?;
            }
            path.release(ledger)?;
            return Err(doctor_issue_size_overflow());
        }
    };
    let mut permit = match ledger.reserve(
        codefire_core::DecisionWorkReservation::new(
            codefire_core::DecisionWorkPhase::Validation,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary_upper,
        )
        .charge(
            codefire_core::DecisionWorkDimension::AggregateBytes,
            aggregate_upper,
        )
        .charge(
            codefire_core::DecisionWorkDimension::ObjectRecords,
            candidate_count,
        )
        .count(
            codefire_core::DecisionWorkCounter::FilesRead,
            candidate_count,
        )
        .count(
            codefire_core::DecisionWorkCounter::FileBytesRead,
            aggregate_upper,
        )
        .count(
            codefire_core::DecisionWorkCounter::RecordReads,
            candidate_count,
        ),
    ) {
        Ok(permit) => permit,
        Err(error) => {
            if let Some(conflicting) = conflicting_path.as_mut() {
                conflicting.release(ledger)?;
            }
            path.release(ledger)?;
            return Err(doctor_issue_accounting_error(error, spec.reserve_context));
        }
    };
    let reported_path = match try_path_from_parts(
        reported_path_upper,
        cf,
        &[Path::new(spec.reported_file_name)],
    ) {
        Ok(path) => path,
        Err(error) => {
            ledger
                .cancel_reserved(&permit)
                .map_err(|cleanup| doctor_issue_accounting_error(cleanup, spec.cancel_context))?;
            if let Some(conflicting) = conflicting_path.as_mut() {
                conflicting.release(ledger)?;
            }
            path.release(ledger)?;
            return Err(error);
        }
    };
    before_read();
    let read_optional = |candidate: &Path| {
        match crate::secure_worktree::read_file_nofollow_bounded_with_path_identity(
            candidate,
            spec.marker_limit,
        ) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    };
    let primary = read_optional(path.as_path());
    let mut presence = DoctorTransactionMarkerPresence::default();
    let (result, bytes_read, records_read) = match primary {
        Err(error) => {
            presence.primary = true;
            (Err(CliError::Io(error)), 0, 0)
        }
        Ok(primary) => {
            presence.primary = primary.is_some();
            if primary.is_some() || conflicting_path.is_some() {
                after_read();
            }
            let conflicting = match conflicting_path.as_ref() {
                Some(conflicting) => read_optional(conflicting.as_path()),
                None => Ok(None),
            };
            match conflicting {
                Err(error) => {
                    presence.companion = true;
                    match primary {
                        Some((bytes, marker_identity)) => {
                            let bytes_read = u64::try_from(bytes.len())
                                .map_err(|_| doctor_issue_size_overflow())?;
                            drop(bytes);
                            let result = match crate::secure_worktree::ensure_file_nofollow_path_identity(
                                path.as_path(),
                                marker_identity,
                            ) {
                                Ok(()) => Err(CliError::Io(error)),
                                Err(error) => Err(CliError::InvalidRepository(format!(
                                    "{} transaction authority changed during Doctor inspection: {error}",
                                    spec.subject
                                ))),
                            };
                            (result, bytes_read, 1)
                        }
                        None => (Err(CliError::Io(error)), 0, 0),
                    }
                }
                Ok(conflicting) => {
                    presence.companion = conflicting.is_some();
                    match (primary, conflicting) {
                        (None, None) => {
                            drop(reported_path);
                            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                                doctor_issue_accounting_error(cleanup, spec.cancel_context)
                            })?;
                            if let Some(conflicting) = conflicting_path.as_mut() {
                                conflicting.release(ledger)?;
                            }
                            path.release(ledger)?;
                            return Ok(presence);
                        }
                        (Some((bytes, marker_identity)), None) => {
                            let bytes_read = u64::try_from(bytes.len())
                                .map_err(|_| doctor_issue_size_overflow())?;
                            let result = match serde_json::from_slice::<Value>(&bytes) {
                                Ok(record) => {
                                    summarize(repo_root, path.as_path().to_path_buf(), record)
                                        .map(|summary| (spec.pending_kind, summary))
                                }
                                Err(error) => Ok((
                                    spec.invalid_kind,
                                    format!(
                                        "{}invalid pending {} transaction {}: {}: {error}",
                                        spec.invalid_json_prefix,
                                        spec.subject,
                                        path.as_path().display(),
                                        spec.invalid_json_detail
                                    ),
                                )),
                            };
                            drop(bytes);
                            let result = match crate::secure_worktree::ensure_file_nofollow_path_identity(
                                path.as_path(),
                                marker_identity,
                            ) {
                                Ok(()) => result,
                                Err(error) => Err(CliError::InvalidRepository(format!(
                                    "{} transaction authority changed during Doctor inspection: {error}",
                                    spec.subject
                                ))),
                            };
                            (result, bytes_read, 1)
                        }
                        (None, Some((bytes, marker_identity))) => {
                            let conflicting = conflicting_path
                                .as_ref()
                                .expect("a companion read retains its accounted path");
                            let bytes_read = u64::try_from(bytes.len())
                                .map_err(|_| doctor_issue_size_overflow())?;
                            let result = match serde_json::from_slice::<Value>(&bytes) {
                                Ok(record) => summarize(
                                    repo_root,
                                    conflicting.as_path().to_path_buf(),
                                    record,
                                )
                                .map(|summary| (spec.pending_kind, summary)),
                                Err(error) => Ok((
                                    spec.invalid_kind,
                                    format!(
                                        "{}invalid pending {} transaction {}: {}: {error}",
                                        spec.invalid_json_prefix,
                                        spec.subject,
                                        conflicting.as_path().display(),
                                        spec.invalid_json_detail
                                    ),
                                )),
                            };
                            drop(bytes);
                            let result = match crate::secure_worktree::ensure_file_nofollow_path_identity(
                                conflicting.as_path(),
                                marker_identity,
                            ) {
                                Ok(()) => result,
                                Err(error) => Err(CliError::InvalidRepository(format!(
                                    "{} transaction authority changed during Doctor inspection: {error}",
                                    spec.subject
                                ))),
                            };
                            (result, bytes_read, 1)
                        }
                        (
                            Some((primary_bytes, primary_identity)),
                            Some((conflicting_bytes, conflicting_identity)),
                        ) => {
                            let conflicting = conflicting_path
                                .as_ref()
                                .expect("a companion read retains its accounted path");
                            let primary_bytes_read = u64::try_from(primary_bytes.len())
                                .map_err(|_| doctor_issue_size_overflow())?;
                            let conflicting_bytes_read = u64::try_from(conflicting_bytes.len())
                                .map_err(|_| doctor_issue_size_overflow())?;
                            drop(conflicting_bytes);
                            let result = match crate::secure_worktree::ensure_file_nofollow_path_identity(
                                conflicting.as_path(),
                                conflicting_identity,
                            ) {
                                Ok(()) => Err(CliError::InvalidRepository(format!(
                                    "invalid pending open transaction {}: both the canonical marker and its removal quarantine exist",
                                    path.as_path().display()
                                ))),
                                Err(error) => Err(CliError::InvalidRepository(format!(
                                    "open transaction removal authority changed during Doctor inspection: {error}"
                                ))),
                            };
                            drop(primary_bytes);
                            let result = match crate::secure_worktree::ensure_file_nofollow_path_identity(
                                path.as_path(),
                                primary_identity,
                            ) {
                                Ok(()) => result,
                                Err(error) => Err(CliError::InvalidRepository(format!(
                                    "{} transaction authority changed during Doctor inspection: {error}",
                                    spec.subject
                                ))),
                            };
                            (
                                result,
                                primary_bytes_read
                                    .checked_add(conflicting_bytes_read)
                                    .ok_or_else(doctor_issue_size_overflow)?,
                                2,
                            )
                        }
                    }
                }
            }
        }
    };
    let issue = match result {
        Ok((kind, message)) => DoctorIssue::error(kind, message, Some(reported_path)),
        Err(error) => DoctorIssue::error(spec.invalid_kind, error.to_string(), Some(reported_path)),
    };
    let temporary = doctor_issue_logical_bytes(
        issue.kind.len(),
        issue.message.len(),
        issue
            .path
            .as_deref()
            .expect("transaction marker issue retains its path")
            .as_os_str()
            .as_encoded_bytes()
            .len(),
    )?;
    let mut actual = codefire_core::DecisionWorkReservation::new(
        codefire_core::DecisionWorkPhase::Validation,
        codefire_core::DecisionWorkSurface::Atomic,
    )
    .acquire(
        codefire_core::DecisionWorkDimension::TemporaryBytes,
        temporary,
    );
    if records_read > 0 {
        actual = actual
            .charge(
                codefire_core::DecisionWorkDimension::AggregateBytes,
                bytes_read,
            )
            .charge(
                codefire_core::DecisionWorkDimension::ObjectRecords,
                records_read,
            )
            .count(codefire_core::DecisionWorkCounter::FilesRead, records_read)
            .count(
                codefire_core::DecisionWorkCounter::FileBytesRead,
                bytes_read,
            )
            .count(
                codefire_core::DecisionWorkCounter::RecordReads,
                records_read,
            );
    }
    if let Err(error) = ledger.consume_reserved(&mut permit, actual) {
        drop(issue);
        ledger
            .cancel_reserved(&permit)
            .map_err(|cleanup| doctor_issue_accounting_error(cleanup, spec.cancel_context))?;
        if let Some(conflicting) = conflicting_path.as_mut() {
            conflicting.release(ledger)?;
        }
        path.release(ledger)?;
        return Err(doctor_issue_accounting_error(error, spec.account_context));
    }
    let push_result = issues.push_accounted_residual_issue(ledger, issue);
    ledger
        .release(
            codefire_core::DecisionWorkDimension::TemporaryBytes,
            temporary,
        )
        .map_err(|error| doctor_issue_accounting_error(error, spec.release_context))?;
    ledger
        .cancel_reserved(&permit)
        .map_err(|error| doctor_issue_accounting_error(error, spec.unused_context))?;
    if let Some(conflicting) = conflicting_path.as_mut() {
        conflicting.release(ledger)?;
    }
    path.release(ledger)?;
    push_result?;
    Ok(presence)
}

fn check_repo_layout(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(), CliError> {
    check_repo_layout_with_hook(cf, issues, ledger, &mut || Ok(()))
}

fn check_repo_layout_with_hook<F>(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
    after_reserve: &mut F,
) -> Result<(), CliError>
where
    F: FnMut() -> Result<(), CliError>,
{
    issues.ensure_ledger_identity(ledger)?;
    if !cf.is_dir() {
        issues.push_layout_issue(
            ledger,
            DoctorLayoutIssueSpec {
                severity: super::DoctorSeverity::Error,
                kind: "missing_codefire_dir",
                message_parts: &["missing .codefire directory"],
                path_base: cf,
                path_parts: &[],
            },
            after_reserve,
        )?;
        return Ok(());
    }
    for entry in REPO_DIRS {
        let relative = Path::new(entry.relative);
        let mut path = AccountedDoctorPath::prepare(ledger, cf, &[relative])?;
        let issue_result = if !path.as_path().is_dir() {
            let severity = match entry.requirement {
                LayoutRequirement::Required => super::DoctorSeverity::Error,
                LayoutRequirement::AutoCreate => super::DoctorSeverity::Warning,
            };
            let kind = match entry.requirement {
                LayoutRequirement::Required => "missing_repo_directory",
                LayoutRequirement::AutoCreate => "missing_repairable_directory",
            };
            issues.push_layout_issue(
                ledger,
                DoctorLayoutIssueSpec {
                    severity,
                    kind,
                    message_parts: &["missing .codefire/", entry.relative, " directory"],
                    path_base: path.as_path(),
                    path_parts: &[],
                },
                after_reserve,
            )
        } else {
            Ok(())
        };
        path.release(ledger)?;
        issue_result?;
    }
    for subdir in codefire_store::known_object_subdirs() {
        let objects = Path::new("objects");
        let relative = Path::new(subdir);
        let mut path = AccountedDoctorPath::prepare(ledger, cf, &[objects, relative])?;
        let issue_result = if !path.as_path().is_dir() {
            let severity = match object_subdir_requirement(subdir) {
                LayoutRequirement::Required => super::DoctorSeverity::Error,
                LayoutRequirement::AutoCreate => super::DoctorSeverity::Warning,
            };
            let kind = match object_subdir_requirement(subdir) {
                LayoutRequirement::Required => "missing_object_subdirectory",
                LayoutRequirement::AutoCreate => "missing_repairable_object_subdirectory",
            };
            issues.push_layout_issue(
                ledger,
                DoctorLayoutIssueSpec {
                    severity,
                    kind,
                    message_parts: &["missing .codefire/objects/", subdir, " directory"],
                    path_base: path.as_path(),
                    path_parts: &[],
                },
                after_reserve,
            )
        } else {
            Ok(())
        };
        path.release(ledger)?;
        issue_result?;
    }
    Ok(())
}

fn check_object_records(
    objects: &Path,
    session: Option<&mut codefire_store::SealedCommitValidationSession>,
    issues: &mut DoctorIssueCollection,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
) -> Result<usize, CliError> {
    check_object_records_with_hooks(objects, session, issues, ledger_identity, || false, || {})
}

fn check_object_records_with_hooks<C, H>(
    objects: &Path,
    session: Option<&mut codefire_store::SealedCommitValidationSession>,
    issues: &mut DoctorIssueCollection,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
    mut should_cancel: C,
    mut after_visit: H,
) -> Result<usize, CliError>
where
    C: FnMut() -> bool,
    H: FnMut(),
{
    issues.ensure_identity(ledger_identity)?;
    if !objects.exists() {
        return Ok(0);
    }
    let session = session.ok_or_else(|| {
        CliError::InvalidRepository(
            "Doctor object scan lost its sealed validation session".to_string(),
        )
    })?;
    let mut checked = 0usize;
    session
        .scan_record_documents_with_cancellation(&mut should_cancel, |entry, work| {
            let path = entry.path;
            let document = match entry.document {
                Ok(document) => document,
                Err(error) => {
                    let outcome = issues.push_accounted_object_scan_issue(
                        work,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "invalid_object_record",
                            message: format_args!("{error}"),
                            path: &path,
                        },
                        &mut after_visit,
                    );
                    drop(error);
                    drop(path);
                    return outcome;
                }
            };
            checked += 1;
            let record = &document.record;
            let outcome =
                if let Some(expected_subdir) = codefire_store::object_subdir(&record.type_tag) {
                    if path
                        .parent()
                        .and_then(Path::file_name)
                        .and_then(|name| name.to_str())
                        != Some(expected_subdir)
                    {
                        issues.push_accounted_object_scan_issue(
                            work,
                            DoctorObjectIssueSpec {
                                severity: super::DoctorSeverity::Error,
                                kind: "object_type_directory_mismatch",
                                message: format_args!(
                                    "{} object should be stored under objects/{expected_subdir}",
                                    record.type_tag
                                ),
                                path: &path,
                            },
                            &mut after_visit,
                        )
                    } else {
                        drop(document);
                        drop(path);
                        return work.complete();
                    }
                } else {
                    issues.push_accounted_object_scan_issue(
                        work,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "unknown_object_type",
                            message: format_args!("unknown object type: {}", record.type_tag),
                            path: &path,
                        },
                        &mut after_visit,
                    )
                };
            drop(document);
            drop(path);
            outcome
        })
        .map_err(doctor_store_error)?;
    if checked == 0 {
        session
            .with_provisional_decision_work_ledger(
                #[expect(
                    clippy::result_large_err,
                    reason = "the Store provisional-ledger callback requires StoreError"
                )]
                |ledger| {
                    issues.push_accounted_object_issue(
                        ledger,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Warning,
                            kind: "empty_object_store",
                            message: format_args!("object store has no valid object records"),
                            path: objects,
                        },
                    )
                },
            )
            .map_err(doctor_store_error)?;
    }
    Ok(checked)
}

fn check_branch_heads(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    sealed_validation: &mut DoctorSealedValidation,
) -> Result<(), CliError> {
    sealed_validation.capture_and_validate_branch_heads(cf, issues)
}

fn check_opened_registry(
    cf: &Path,
    issues: &mut DoctorIssueCollection,
    sealed_validation: &mut DoctorSealedValidation,
) -> Result<usize, CliError> {
    let ledger_identity = sealed_validation.ledger_identity();
    let opened = cf.join("opened");
    let mut snapshot = sealed_validation.capture_opened_registry_snapshot(cf, &opened)?;
    let result = (|| {
        if !snapshot.exists() {
            return Ok(0);
        }
        let mut checked = 0usize;
        for entry_index in 0..snapshot.entry_count() {
            let Some(candidate) = snapshot.candidate(entry_index) else {
                continue;
            };
            match candidate.source {
                super::opened_registry::DoctorOpenedRegistrySource::Bytes(_) => {
                    match sealed_validation
                        .parse_opened_registry_candidate(&mut snapshot, entry_index)?
                    {
                        super::opened_registry::DoctorOpenedRegistryParseResult::Parsed => {}
                        super::opened_registry::DoctorOpenedRegistryParseResult::Invalid => {
                            let (path, error) = snapshot
                                .invalid_candidate(entry_index)
                                .expect("invalid JSON entry retains its parsed error");
                            sealed_validation.push_accounted_opened_issue(
                                issues,
                                DoctorObjectIssueSpec {
                                    severity: super::DoctorSeverity::Error,
                                    kind: "invalid_open_registry",
                                    message: format_args!("{error}"),
                                    path,
                                },
                            )?;
                            continue;
                        }
                        super::opened_registry::DoctorOpenedRegistryParseResult::CleanupFailed(
                            error,
                        ) => return Err(doctor_store_error(error)),
                    }
                }
                super::opened_registry::DoctorOpenedRegistrySource::Unavailable(reason) => {
                    let path = candidate.path;
                    let error = reason.at(path);
                    sealed_validation.push_accounted_opened_issue(
                        issues,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "invalid_open_registry",
                            message: format_args!("{error}"),
                            path,
                        },
                    )?;
                    continue;
                }
            }
            let (path, registry) = snapshot.parsed_candidate(entry_index).ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor opened-registry parsed candidate lost its snapshot owner".to_string(),
                )
            })?;
            let branch_name = match registry
                .get("branch")
                .and_then(|branch| branch.get("name"))
                .and_then(Value::as_str)
            {
                Some(branch) => Some(branch.to_string()),
                None => {
                    sealed_validation.push_accounted_opened_issue(
                        issues,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "missing_open_branch_name",
                            message: format_args!("opened registry must contain branch.name"),
                            path,
                        },
                    )?;
                    None
                }
            };
            let open_instance_id = match registry
                .get("open")
                .and_then(|open| open.get("open_instance_id"))
                .and_then(Value::as_str)
            {
                Some(open_instance_id) => Some(open_instance_id.to_string()),
                None => {
                    sealed_validation.push_accounted_opened_issue(
                        issues,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "missing_open_instance_id",
                            message: format_args!(
                                "opened registry must contain open.open_instance_id"
                            ),
                            path,
                        },
                    )?;
                    None
                }
            };
            let open_path = match registry
                .get("open")
                .and_then(|open| open.get("path"))
                .and_then(Value::as_str)
            {
                Some(open_path) => Some(PathBuf::from(open_path)),
                None => {
                    sealed_validation.push_accounted_opened_issue(
                        issues,
                        DoctorObjectIssueSpec {
                            severity: super::DoctorSeverity::Error,
                            kind: "missing_open_path",
                            message: format_args!("opened registry must contain open.path"),
                            path,
                        },
                    )?;
                    None
                }
            };
            match registry
                .get("open")
                .and_then(|open| open.get("current_base_commit"))
                .and_then(Value::as_str)
            {
                Some(base) => sealed_validation.validate_open_base(
                    base,
                    path.to_path_buf(),
                    open_path.clone(),
                    issues.residual_mut(ledger_identity)?,
                )?,
                None => sealed_validation.push_accounted_opened_issue(
                    issues,
                    DoctorObjectIssueSpec {
                        severity: super::DoctorSeverity::Error,
                        kind: "missing_open_base_commit",
                        message: format_args!(
                            "opened registry must contain open.current_base_commit"
                        ),
                        path,
                    },
                )?,
            }
            if let Some(open_path) = open_path.as_deref() {
                if !open_path.is_dir() {
                    issues
                        .residual_mut(ledger_identity)?
                        .push(DoctorIssue::error(
                            "missing_open_path_dir",
                            format!(
                                "opened registry path does not exist: {}",
                                open_path.display()
                            ),
                            Some(path.to_path_buf()),
                        ));
                } else {
                    check_open_marker(
                        cf,
                        path,
                        registry,
                        open_path,
                        branch_name.as_deref(),
                        open_instance_id.as_deref(),
                        issues.residual_mut(ledger_identity)?,
                    );
                }
            }
            match registry
                .get("open")
                .and_then(|open| open.get("active_state_path"))
                .and_then(Value::as_str)
            {
                Some(active_path) => {
                    match crate::active_state::ActiveStateDir::bind(Path::new(active_path)) {
                        Ok(active_state) => {
                            checked += 1;
                            check_verified_open_snapshot(
                                cf,
                                path,
                                registry,
                                &active_state,
                                issues.residual_mut(ledger_identity)?,
                            );
                        }
                        Err(error) if error.is_not_found() => issues
                            .residual_mut(ledger_identity)?
                            .push(DoctorIssue::error(
                                "missing_active_state_dir",
                                format!("active state directory does not exist: {active_path}"),
                                Some(path.to_path_buf()),
                            )),
                        Err(error) => {
                            issues
                                .residual_mut(ledger_identity)?
                                .push(DoctorIssue::error(
                                    "invalid_active_state_dir",
                                    error.to_string(),
                                    Some(path.to_path_buf()),
                                ))
                        }
                    }
                }
                None => sealed_validation.push_accounted_opened_issue(
                    issues,
                    DoctorObjectIssueSpec {
                        severity: super::DoctorSeverity::Error,
                        kind: "missing_active_state_path",
                        message: format_args!(
                            "opened registry must contain open.active_state_path"
                        ),
                        path,
                    },
                )?,
            }
        }
        Ok(checked)
    })();
    sealed_validation.attach_opened_registry_snapshot(snapshot);
    result
}

fn check_verified_open_snapshot(
    cf: &Path,
    registry_path: &Path,
    registry: &Value,
    active_state: &crate::active_state::ActiveStateDir,
    issues: &mut Vec<DoctorIssue>,
) {
    let Some(repo_root) = cf.parent() else {
        return;
    };
    let repository_record_path = cf.join("repo.json");
    let Some(branch_name) = value_string(registry, &["branch", "name"]) else {
        return;
    };
    let branch_record_path = cf
        .join("branches")
        .join(format!("{}.json", crate::ref_file_name(&branch_name)));
    let Some(open_path) = path_string(registry, &["open", "path"]) else {
        return;
    };
    let marker_path = open_path.join(".codefire-open");
    let (repository, branch, marker, active_state_record) = match (
        read_doctor_json(&repository_record_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT),
        read_doctor_json(&branch_record_path, ACTIVE_DATA_FILE_BYTE_LIMIT),
        read_doctor_json(&marker_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT),
        read_active_state_json(active_state, Path::new("state.json"), "state.json"),
    ) {
        (Ok(repository), Ok(branch), Ok(marker), Ok(active_state_record)) => {
            (repository, branch, marker, active_state_record)
        }
        _ => return,
    };
    let Some(open_instance_id) = value_string(registry, &["open", "open_instance_id"]) else {
        return;
    };
    let expected_active_state_path = cf
        .join("active")
        .join(crate::ref_file_name(&open_instance_id));
    let pending_commit_transaction = read_optional_active_transaction(
        active_state,
        "commit_transaction.json",
        crate::COMMIT_TRANSACTION_BYTE_LIMIT,
    );
    let pending_state_transaction = read_optional_active_transaction(
        active_state,
        "state_transaction.json",
        crate::STATE_TRANSACTION_BYTE_LIMIT,
    );

    let values = (
        value_string(&repository, &["repository_id"]),
        path_string(&marker, &["repository", "path"]),
        value_string(&marker, &["repository", "repository_id"]),
        value_string(&marker, &["branch", "name"]),
        value_string(&branch, &["name"]),
        path_string(&marker, &["open", "opened_path"]),
        value_string(&marker, &["open", "open_instance_id"]),
        value_string(&marker, &["branch", "opened_from_commit"]),
        value_string(registry, &["open", "opened_from_commit"]),
        path_string(registry, &["open", "active_state_path"]),
        value_string(&branch, &["head"]),
        value_string(registry, &["open", "current_base_commit"]),
        value_string(&branch, &["state"]),
        value_string(registry, &["state", "last_known"]),
        value_string(&active_state_record, &["state"]),
    );
    let (
        Some(repository_id),
        Some(marker_repository_path),
        Some(marker_repository_id),
        Some(marker_branch),
        Some(branch_record_name),
        Some(marker_open_path),
        Some(marker_open_instance_id),
        Some(marker_opened_from_commit),
        Some(registry_opened_from_commit),
        Some(registry_active_state_path),
        Some(branch_head),
        Some(registry_current_base),
        Some(branch_state),
        Some(registry_state),
        Some(active_state_value),
    ) = values
    else {
        return;
    };

    let active_state_record_path = active_state.display_path(Path::new("state.json"));
    let active_ledger_descriptor =
        crate::fire::active_ledger_transaction::validate_active_ledger_descriptor(active_state)
            .ok()
            .flatten();
    let identity = crate::open_snapshot::OpenSnapshotIdentityAuthorities {
        repo_root,
        repository_record_path: &repository_record_path,
        marker_path: &marker_path,
        branch_record_path: &branch_record_path,
        registry_path,
        expected_repository_path: cf,
        marker_repository_path: &marker_repository_path,
        repository_id: &repository_id,
        marker_repository_id: &marker_repository_id,
        branch: &branch_name,
        marker_branch: &marker_branch,
        registry_branch: &branch_name,
        branch_record_name: &branch_record_name,
        open_dir: &open_path,
        marker_open_path: &marker_open_path,
        registry_open_path: &open_path,
        marker_open_instance_id: &marker_open_instance_id,
        registry_open_instance_id: &open_instance_id,
        marker_opened_from_commit: &marker_opened_from_commit,
        registry_opened_from_commit: &registry_opened_from_commit,
        expected_active_state_path: &expected_active_state_path,
        registry_active_state_path: &registry_active_state_path,
        branch_head: &branch_head,
        registry_current_base: &registry_current_base,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
    };
    let state = crate::open_snapshot::OpenSnapshotStateAuthorities {
        active_state_record_path: &active_state_record_path,
        branch_state: &branch_state,
        registry_state: &registry_state,
        active_state: &active_state_value,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        pending_state_transaction: pending_state_transaction.as_ref(),
        active_ledger_descriptor: active_ledger_descriptor.as_ref(),
    };
    if let Err(divergence) = crate::open_snapshot::verify_open_snapshot(&identity, &state) {
        issues.push(DoctorIssue::open_snapshot_divergence(*divergence));
    }
}

fn read_optional_active_transaction(
    active_state: &crate::active_state::ActiveStateDir,
    file_name: &str,
    limit: u64,
) -> Option<Value> {
    let relative = Path::new(file_name);
    match active_state.exists(relative) {
        Ok(true) => active_state
            .read_required(relative, limit)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
        Ok(false) | Err(_) => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitSemanticProfile {
    Current,
    LegacyUnversioned,
}

enum DoctorSealedCandidateKind {
    BranchHead {
        candidate_index: usize,
    },
    OpenBase {
        registry_path: PathBuf,
        open_path: Option<PathBuf>,
    },
}

struct SuccessfulDoctorSealedCandidate {
    commit_id: String,
    issue_index: usize,
    kind: DoctorSealedCandidateKind,
}

struct DoctorSealedValidation {
    session: Option<codefire_store::SealedCommitValidationSession>,
    detached_ledger: Option<codefire_core::DecisionWorkLedger>,
    objects: PathBuf,
    begin_error: Option<String>,
    successful: Vec<SuccessfulDoctorSealedCandidate>,
    branch_snapshot: Option<super::branch_heads::DoctorBranchSnapshot>,
    opened_snapshot: Option<super::opened_registry::DoctorOpenedRegistrySnapshot>,
    branch_success_retained_bytes: u64,
    finalize_failed: bool,
    ledger_identity: codefire_core::DecisionWorkLedgerIdentity,
}

#[derive(Debug)]
struct DoctorSealedValidationStartError {
    error: CliError,
    ledger: codefire_core::DecisionWorkLedger,
}

impl DoctorSealedValidation {
    fn new(
        objects: &Path,
        ledger: codefire_core::DecisionWorkLedger,
    ) -> Result<Self, Box<DoctorSealedValidationStartError>> {
        let ledger_identity = ledger.identity();
        if !objects.exists() {
            return Ok(Self {
                session: None,
                detached_ledger: Some(ledger),
                objects: objects.to_path_buf(),
                begin_error: Some(format!(
                    "object store does not exist: {}",
                    objects.display()
                )),
                successful: Vec::new(),
                branch_snapshot: None,
                opened_snapshot: None,
                branch_success_retained_bytes: 0,
                finalize_failed: false,
                ledger_identity,
            });
        }
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            objects, ledger,
        ) {
            Ok(session) => Ok(Self {
                session: Some(session),
                detached_ledger: None,
                objects: objects.to_path_buf(),
                begin_error: None,
                successful: Vec::new(),
                branch_snapshot: None,
                opened_snapshot: None,
                branch_success_retained_bytes: 0,
                finalize_failed: false,
                ledger_identity,
            }),
            Err(error) => {
                let (error, ledger) = error.into_parts();
                Err(Box::new(DoctorSealedValidationStartError {
                    error: doctor_store_error(error),
                    ledger,
                }))
            }
        }
    }

    fn ledger_identity(&self) -> codefire_core::DecisionWorkLedgerIdentity {
        self.ledger_identity
    }

    fn scan_object_records(
        &mut self,
        issues: &mut DoctorIssueCollection,
    ) -> Result<usize, CliError> {
        check_object_records(
            &self.objects,
            self.session.as_mut(),
            issues,
            self.ledger_identity,
        )
    }

    #[cfg(test)]
    fn scan_object_records_with_hooks<C, H>(
        &mut self,
        issues: &mut DoctorIssueCollection,
        should_cancel: C,
        after_visit: H,
    ) -> Result<usize, CliError>
    where
        C: FnMut() -> bool,
        H: FnMut(),
    {
        check_object_records_with_hooks(
            &self.objects,
            self.session.as_mut(),
            issues,
            self.ledger_identity,
            should_cancel,
            after_visit,
        )
    }

    #[expect(
        clippy::result_large_err,
        reason = "the Store validation callbacks require StoreError"
    )]
    fn capture_and_validate_branch_heads(
        &mut self,
        cf: &Path,
        issues: &mut DoctorIssueCollection,
    ) -> Result<(), CliError> {
        let identity = self.ledger_identity;
        let mut capture = |ledger: &mut codefire_core::DecisionWorkLedger| {
            super::branch_heads::DoctorBranchSnapshot::capture(
                cf,
                ACTIVE_DATA_FILE_BYTE_LIMIT,
                ledger,
                |path, message, ledger| {
                    issues.push_accounted_branch_issue(
                        ledger,
                        super::DoctorSeverity::Error,
                        "invalid_branch_record",
                        message,
                        path,
                    )
                },
            )
        };
        let snapshot = if let Some(session) = self.session.as_mut() {
            let mut captured = None;
            session
                .with_provisional_decision_work_ledger(|ledger| {
                    captured = Some(capture(ledger)?);
                    Ok(())
                })
                .map_err(doctor_store_error)?;
            captured.ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor branch capture did not return its snapshot".to_string(),
                )
            })?
        } else {
            capture(self.detached_ledger.as_mut().ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor branch capture lost its detached ledger".to_string(),
                )
            })?)
            .map_err(doctor_store_error)?
        };
        self.branch_snapshot = Some(snapshot);

        let candidate_count = self
            .branch_snapshot
            .as_ref()
            .expect("captured Doctor branch snapshot is attached")
            .candidate_count();
        for candidate_index in 0..candidate_count {
            let Some((head, path)) = self
                .branch_snapshot
                .as_ref()
                .expect("captured Doctor branch snapshot is attached")
                .candidate_head_and_path(candidate_index)
            else {
                continue;
            };
            let issue_index = issues.residual_len(identity)?;
            let Some(session) = self.session.as_mut() else {
                let message = self
                    .begin_error
                    .as_deref()
                    .unwrap_or("sealed validation session is unavailable");
                issues
                    .push_accounted_branch_issue(
                        self.detached_ledger.as_mut().ok_or_else(|| {
                            CliError::InvalidRepository(
                                "Doctor branch validation lost its detached ledger".to_string(),
                            )
                        })?,
                        super::DoctorSeverity::Error,
                        "invalid_branch_head",
                        format_args!("{message}"),
                        path,
                    )
                    .map_err(doctor_store_error)?;
                continue;
            };
            let valid = session
                .validate_commit_recoverable_with_cancellation_and_error_handler(
                    head,
                    || false,
                    #[expect(
                        clippy::result_large_err,
                        reason = "the recoverable Store validation handler requires StoreError"
                    )]
                    |error, ledger| {
                        issues.push_accounted_branch_issue(
                            ledger,
                            super::DoctorSeverity::Error,
                            "invalid_branch_head",
                            format_args!("{error}"),
                            path,
                        )
                    },
                )
                .map_err(doctor_store_error)?;
            if valid {
                let frame_bytes = DOCTOR_OWNER_FRAME_BYTES
                    .checked_add(1)
                    .ok_or_else(doctor_issue_size_overflow)?;
                session
                    .with_provisional_decision_work_ledger(|ledger| {
                        let mut permit = ledger.reserve(
                            codefire_core::DecisionWorkReservation::new(
                                codefire_core::DecisionWorkPhase::PublicationPreflight,
                                codefire_core::DecisionWorkSurface::Atomic,
                            )
                            .acquire(
                                codefire_core::DecisionWorkDimension::RetainedBytes,
                                frame_bytes,
                            ),
                        )?;
                        if let Err(error) = self.successful.try_reserve(1) {
                            ledger.cancel_reserved(&permit)?;
                            return Err(codefire_store::StoreError::Io(std::io::Error::other(
                                format!("Doctor branch success allocation failed: {error}"),
                            )));
                        }
                        ledger.consume_reserved(
                            &mut permit,
                            codefire_core::DecisionWorkReservation::new(
                                codefire_core::DecisionWorkPhase::PublicationPreflight,
                                codefire_core::DecisionWorkSurface::Atomic,
                            )
                            .acquire(
                                codefire_core::DecisionWorkDimension::RetainedBytes,
                                frame_bytes,
                            ),
                        )?;
                        Ok(())
                    })
                    .map_err(doctor_store_error)?;
                self.successful.push(SuccessfulDoctorSealedCandidate {
                    commit_id: String::new(),
                    issue_index,
                    kind: DoctorSealedCandidateKind::BranchHead { candidate_index },
                });
                self.branch_success_retained_bytes = self
                    .branch_success_retained_bytes
                    .checked_add(frame_bytes)
                    .ok_or_else(doctor_issue_size_overflow)?;
            }
        }
        Ok(())
    }

    fn validate_open_base(
        &mut self,
        commit_id: &str,
        registry_path: PathBuf,
        open_path: Option<PathBuf>,
        issues: &mut Vec<DoctorIssue>,
    ) -> Result<(), CliError> {
        self.validate(
            commit_id,
            DoctorSealedCandidateKind::OpenBase {
                registry_path,
                open_path,
            },
            issues,
        )
    }

    fn capture_opened_registry_snapshot(
        &mut self,
        cf: &Path,
        opened: &Path,
    ) -> Result<super::opened_registry::DoctorOpenedRegistrySnapshot, CliError> {
        #[expect(
            clippy::result_large_err,
            reason = "the opened-registry capture preserves the Store callback error"
        )]
        let capture = |ledger: &mut codefire_core::DecisionWorkLedger| {
            super::opened_registry::DoctorOpenedRegistrySnapshot::capture(cf, opened, ledger)
        };
        if let Some(session) = self.session.as_mut() {
            let mut snapshot = None;
            session
                .with_provisional_decision_work_ledger(
                    #[expect(
                        clippy::result_large_err,
                        reason = "the provisional Store ledger callback requires StoreError"
                    )]
                    |ledger| {
                        snapshot = Some(capture(ledger)?);
                        Ok(())
                    },
                )
                .map_err(doctor_store_error)?;
            snapshot.ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor opened-registry capture did not return its snapshot".to_string(),
                )
            })
        } else {
            capture(self.detached_ledger.as_mut().ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor opened-registry capture lost its detached ledger".to_string(),
                )
            })?)
            .map_err(doctor_store_error)
        }
    }

    fn attach_opened_registry_snapshot(
        &mut self,
        snapshot: super::opened_registry::DoctorOpenedRegistrySnapshot,
    ) {
        assert!(
            self.opened_snapshot.is_none(),
            "Doctor opened-registry snapshot attaches only once"
        );
        self.opened_snapshot = Some(snapshot);
    }

    fn parse_opened_registry_candidate(
        &mut self,
        snapshot: &mut super::opened_registry::DoctorOpenedRegistrySnapshot,
        entry_index: usize,
    ) -> Result<super::opened_registry::DoctorOpenedRegistryParseResult, CliError> {
        let mut parsed = None;
        #[expect(
            clippy::result_large_err,
            reason = "the provisional Store callback requires StoreError"
        )]
        let mut parse = |ledger: &mut codefire_core::DecisionWorkLedger| {
            parsed = Some(snapshot.parse_candidate(entry_index, ledger)?);
            Ok(())
        };
        if let Some(session) = self.session.as_mut() {
            session
                .with_provisional_decision_work_ledger(&mut parse)
                .map_err(doctor_store_error)?;
        } else {
            parse(self.detached_ledger.as_mut().ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor opened-registry parsing lost its detached ledger".to_string(),
                )
            })?)
            .map_err(doctor_store_error)?;
        }
        parsed.ok_or_else(|| {
            CliError::InvalidRepository(
                "Doctor opened-registry parsing did not return an outcome".to_string(),
            )
        })
    }

    fn push_accounted_opened_issue(
        &mut self,
        issues: &mut DoctorIssueCollection,
        spec: DoctorObjectIssueSpec<'_>,
    ) -> Result<(), CliError> {
        if let Some(session) = self.session.as_mut() {
            session
                .with_provisional_decision_work_ledger(
                    #[expect(
                        clippy::result_large_err,
                        reason = "the provisional Store ledger callback requires StoreError"
                    )]
                    |ledger| issues.push_accounted_opened_issue(ledger, spec),
                )
                .map_err(doctor_store_error)
        } else {
            issues
                .push_accounted_opened_issue(
                    self.detached_ledger.as_mut().ok_or_else(|| {
                        CliError::InvalidRepository(
                            "Doctor opened diagnostics lost their detached ledger".to_string(),
                        )
                    })?,
                    spec,
                )
                .map_err(doctor_store_error)
        }
    }

    fn validate(
        &mut self,
        commit_id: &str,
        kind: DoctorSealedCandidateKind,
        issues: &mut Vec<DoctorIssue>,
    ) -> Result<(), CliError> {
        let Some(session) = self.session.as_mut() else {
            let message = self
                .begin_error
                .clone()
                .unwrap_or_else(|| "sealed validation session is unavailable".to_string());
            push_doctor_candidate_error(issues, &kind, message);
            return Ok(());
        };
        let result = session.validate_commit_recoverable(commit_id);
        match result {
            Ok(()) => self.successful.push(SuccessfulDoctorSealedCandidate {
                commit_id: commit_id.to_string(),
                issue_index: issues.len(),
                kind,
            }),
            Err(error)
                if matches!(
                    error,
                    codefire_store::StoreError::DecisionWorkAccounting(_)
                        | codefire_store::StoreError::ValidationCancelled
                        | codefire_store::StoreError::SealedValidationSubjectChanged { .. }
                ) =>
            {
                return Err(doctor_store_error(error));
            }
            Err(error) => push_doctor_candidate_error(issues, &kind, error.to_string()),
        }
        Ok(())
    }

    fn finish(
        &mut self,
        issues: &mut DoctorIssueCollection,
    ) -> Result<(usize, codefire_core::DecisionWorkLedger), CliError> {
        if let Some(session) = self.session.as_mut() {
            if let Err(error) = session.finalize() {
                self.finalize_failed = true;
                return Err(doctor_store_error(error));
            }
        }
        let mut checked_branches = 0usize;
        let mut inserted = 0usize;
        for candidate in std::mem::take(&mut self.successful) {
            let session = self.session.as_ref().ok_or_else(|| {
                CliError::InvalidRepository(
                    "successful Doctor candidate lost its validation session".to_string(),
                )
            })?;
            match &candidate.kind {
                DoctorSealedCandidateKind::BranchHead { candidate_index } => {
                    let (commit_id, path) = self
                        .branch_snapshot
                        .as_ref()
                        .and_then(|snapshot| snapshot.candidate_head_and_path(*candidate_index))
                        .ok_or_else(|| {
                            CliError::InvalidRepository(
                                "successful Doctor branch candidate lost its snapshot evidence"
                                    .to_string(),
                            )
                        })?;
                    let profile = session
                        .validated_commit_payload(commit_id)
                        .map_err(Box::new)
                        .and_then(commit_semantic_profile);
                    match profile {
                        Ok(CommitSemanticProfile::Current) => checked_branches += 1,
                        Ok(CommitSemanticProfile::LegacyUnversioned) => {
                            checked_branches += 1;
                            self.session
                                .as_mut()
                                .expect("successful branch retains its validation session")
                                .with_finalized_decision_work_ledger(#[expect(
                                    clippy::result_large_err,
                                    reason = "the finalized Store ledger callback requires StoreError"
                                )] |_, ledger| {
                                    issues.insert_accounted_branch_issue(
                                        ledger,
                                        candidate.issue_index + inserted,
                                        super::DoctorSeverity::Warning,
                                        "legacy_branch_head_semantic_profile",
                                        format_args!(
                                            "branch head {commit_id} is a valid legacy-unversioned sealed commit, but it does not carry {} semantic proof",
                                            codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
                                        ),
                                        path,
                                    )
                                })
                                .map_err(doctor_store_error)?;
                            inserted += 1;
                        }
                        Err(error) => {
                            self.session
                                .as_mut()
                                .expect("successful branch retains its validation session")
                                .with_finalized_decision_work_ledger(#[expect(
                                    clippy::result_large_err,
                                    reason = "the finalized Store ledger callback requires StoreError"
                                )] |_, ledger| {
                                    issues.insert_accounted_branch_issue(
                                        ledger,
                                        candidate.issue_index + inserted,
                                        super::DoctorSeverity::Error,
                                        "invalid_branch_head",
                                        format_args!("{error}"),
                                        path,
                                    )
                                })
                                .map_err(doctor_store_error)?;
                            inserted += 1;
                        }
                    }
                }
                DoctorSealedCandidateKind::OpenBase { .. } => {
                    let payload = session.validated_commit_payload(&candidate.commit_id);
                    let diagnostic =
                        match payload.map_err(Box::new).and_then(commit_semantic_profile) {
                            Ok(CommitSemanticProfile::Current) => None,
                            Ok(CommitSemanticProfile::LegacyUnversioned) => {
                                Some(legacy_profile_issue(&candidate))
                            }
                            Err(error) => Some(DoctorIssue::error(
                                candidate_error_kind(&candidate.kind),
                                error.to_string(),
                                Some(candidate_error_path(&candidate.kind).to_path_buf()),
                            )),
                        };
                    if let Some(diagnostic) = diagnostic {
                        issues
                            .residual_mut(self.ledger_identity)?
                            .insert(candidate.issue_index + inserted, diagnostic);
                        inserted += 1;
                    }
                }
            }
        }
        let ledger = if let Some(session) = self.session.as_mut() {
            session
                .take_decision_work_ledger()
                .map_err(doctor_store_error)?
        } else {
            self.detached_ledger.take()
        };
        let mut ledger = ledger.ok_or_else(|| {
            CliError::InvalidRepository(
                "Doctor graph validation lost its decision-work ledger".to_string(),
            )
        })?;
        self.session = None;
        if let Some(mut snapshot) = self.branch_snapshot.take() {
            if let Err(error) = snapshot.revalidate(ACTIVE_DATA_FILE_BYTE_LIMIT, &mut ledger) {
                self.branch_snapshot = Some(snapshot);
                let frame_cleanup = self.release_branch_success_frames(&mut ledger);
                self.detached_ledger = Some(ledger);
                frame_cleanup?;
                return Err(doctor_store_error(error));
            }
        }
        if let Some(mut snapshot) = self.opened_snapshot.take() {
            if let Err(error) = snapshot.revalidate(&mut ledger) {
                self.opened_snapshot = Some(snapshot);
                let frame_cleanup = self.release_branch_success_frames(&mut ledger);
                self.detached_ledger = Some(ledger);
                frame_cleanup?;
                return Err(doctor_store_error(error));
            }
        }
        if let Err(error) = self.release_branch_success_frames(&mut ledger) {
            self.detached_ledger = Some(ledger);
            return Err(error);
        }
        Ok((checked_branches, ledger))
    }

    fn abort_and_take_ledger(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<codefire_core::DecisionWorkLedger, CliError> {
        let ledger = if let Some(session) = self.session.as_mut() {
            if self.finalize_failed {
                session
                    .take_decision_work_ledger()
                    .map_err(doctor_store_error)?
            } else {
                session
                    .abort_and_take_decision_work_ledger(reason)
                    .map_err(doctor_store_error)?
            }
        } else {
            self.detached_ledger.take()
        };
        let mut ledger = ledger.ok_or_else(|| {
            CliError::InvalidRepository(
                "Doctor graph validation lost its decision-work ledger during abort".to_string(),
            )
        })?;
        self.session = None;
        if let Some(mut snapshot) = self.branch_snapshot.take() {
            if let Err(error) = snapshot.cancel(&mut ledger) {
                self.branch_snapshot = Some(snapshot);
                self.detached_ledger = Some(ledger);
                return Err(doctor_store_error(error));
            }
        }
        if let Some(mut snapshot) = self.opened_snapshot.take() {
            if let Err(error) = snapshot.cancel(&mut ledger) {
                self.opened_snapshot = Some(snapshot);
                self.detached_ledger = Some(ledger);
                return Err(doctor_store_error(error));
            }
        }
        if let Err(error) = self.release_branch_success_frames(&mut ledger) {
            self.detached_ledger = Some(ledger);
            return Err(error);
        }
        Ok(ledger)
    }

    fn release_branch_success_frames(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        if self.branch_success_retained_bytes == 0 {
            return Ok(());
        }
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.branch_success_retained_bytes,
            )
            .map_err(|error| doctor_store_error(error.into()))?;
        self.branch_success_retained_bytes = 0;
        Ok(())
    }
}

fn doctor_store_error(error: codefire_store::StoreError) -> CliError {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
            match accounting.budget_exceeded() {
                Some(exhaustion) => CliError::from(exhaustion.clone()),
                None => CliError::from(codefire_store::StoreError::DecisionWorkAccounting(
                    accounting,
                )),
            }
        }
        error => CliError::from(error),
    }
}

fn terminal_reason_for_store_error(
    error: &codefire_store::StoreError,
) -> codefire_core::DecisionWorkTerminalReason {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(_) => {
            codefire_core::DecisionWorkTerminalReason::BudgetExceeded
        }
        codefire_store::StoreError::ValidationCancelled => {
            codefire_core::DecisionWorkTerminalReason::DeadlineExceeded
        }
        codefire_store::StoreError::SealedValidationSubjectChanged { .. } => {
            codefire_core::DecisionWorkTerminalReason::SubjectChanged
        }
        _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
    }
}

fn terminal_reason_for_cli_error(error: &CliError) -> codefire_core::DecisionWorkTerminalReason {
    match error {
        CliError::ResourceBudget(_) => codefire_core::DecisionWorkTerminalReason::BudgetExceeded,
        CliError::Store(error) => terminal_reason_for_store_error(error),
        _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
    }
}

fn cancel_doctor_issue_run(
    issues: &mut DoctorIssueCollection,
    ledger: &mut codefire_core::DecisionWorkLedger,
    reason: codefire_core::DecisionWorkTerminalReason,
) -> Result<(), CliError> {
    issues.cancel(ledger)?;
    ensure_doctor_ledger_report_retained(ledger, 0)?;
    if ledger.snapshot().terminal_reason.is_none() {
        ledger
            .finish(reason)
            .map_err(|error| doctor_store_error(error.into()))?;
    }
    Ok(())
}

fn ensure_doctor_ledger_report_retained(
    ledger: &codefire_core::DecisionWorkLedger,
    expected_retained: u64,
) -> Result<(), CliError> {
    let retained = ledger
        .snapshot()
        .current
        .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
        .copied()
        .unwrap_or_default();
    let other_current = ledger.snapshot().current.iter().any(|(dimension, amount)| {
        *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
    });
    if retained != expected_retained || other_current || ledger.has_open_permits() {
        return Err(CliError::InvalidRepository(
            "Doctor session retained unexpected decision work outside its report owner".to_string(),
        ));
    }
    Ok(())
}

fn candidate_error_kind(kind: &DoctorSealedCandidateKind) -> &'static str {
    match kind {
        DoctorSealedCandidateKind::BranchHead { .. } => "invalid_branch_head",
        DoctorSealedCandidateKind::OpenBase { .. } => "invalid_open_base_commit",
    }
}

fn candidate_error_path(kind: &DoctorSealedCandidateKind) -> &Path {
    match kind {
        DoctorSealedCandidateKind::BranchHead { .. } => {
            panic!("accounted Doctor branch candidates resolve their path from the snapshot")
        }
        DoctorSealedCandidateKind::OpenBase { registry_path, .. } => registry_path,
    }
}

fn push_doctor_candidate_error(
    issues: &mut Vec<DoctorIssue>,
    kind: &DoctorSealedCandidateKind,
    message: String,
) {
    issues.push(DoctorIssue::error(
        candidate_error_kind(kind),
        message,
        Some(candidate_error_path(kind).to_path_buf()),
    ));
}

fn legacy_profile_issue(candidate: &SuccessfulDoctorSealedCandidate) -> DoctorIssue {
    match &candidate.kind {
        DoctorSealedCandidateKind::BranchHead { .. } => {
            panic!("accounted Doctor branch diagnostics are materialized from snapshot evidence")
        }
        DoctorSealedCandidateKind::OpenBase { open_path, .. } => DoctorIssue::warning(
            "legacy_open_base_semantic_profile",
            format!(
                "open base {} is a valid legacy-unversioned sealed commit, but it does not carry {} semantic proof; scan this workspace to construct current-profile state",
                candidate.commit_id,
                codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
            ),
            open_path.clone(),
        ),
    }
}

fn commit_semantic_profile(
    commit: &Value,
) -> Result<CommitSemanticProfile, Box<codefire_store::StoreError>> {
    match commit.get("semantic_validation_profile") {
        None => Ok(CommitSemanticProfile::LegacyUnversioned),
        Some(Value::String(profile))
            if profile == codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE =>
        {
            Ok(CommitSemanticProfile::Current)
        }
        Some(Value::String(_)) => Err(Box::new(codefire_store::StoreError::InvalidSealedCommit(
            format!(
                "unsupported commit.semantic_validation_profile; supported value is {}",
                codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
            ),
        ))),
        Some(_) => Err(Box::new(codefire_store::StoreError::InvalidSealedCommit(
            "commit.semantic_validation_profile must be a string".to_string(),
        ))),
    }
}

fn check_open_marker(
    cf: &Path,
    registry_path: &Path,
    registry: &Value,
    open_path: &Path,
    branch_name: Option<&str>,
    open_instance_id: Option<&str>,
    issues: &mut Vec<DoctorIssue>,
) {
    let marker_path = open_path.join(".codefire-open");
    if !marker_path.exists() {
        issues.push(DoctorIssue::error(
            "missing_open_marker",
            "open path is missing .codefire-open marker",
            Some(marker_path),
        ));
        return;
    }
    let marker = match read_doctor_json(&marker_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT) {
        Ok(marker) => marker,
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_open_marker",
                error.to_string(),
                Some(marker_path),
            ));
            return;
        }
    };
    if path_string(&marker, &["repository", "path"]).as_deref() != Some(cf) {
        issues.push(DoctorIssue::error(
            "open_marker_repository_mismatch",
            "open marker repository.path does not match registry repository",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["branch", "name"]).as_deref() != branch_name {
        issues.push(DoctorIssue::error(
            "open_marker_branch_mismatch",
            "open marker branch.name does not match opened registry branch.name",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["open", "open_instance_id"]).as_deref() != open_instance_id {
        issues.push(DoctorIssue::error(
            "open_marker_instance_mismatch",
            "open marker open_instance_id does not match opened registry",
            Some(marker_path.clone()),
        ));
    }
    if path_string(&marker, &["open", "opened_path"]).as_deref() != Some(open_path) {
        issues.push(DoctorIssue::error(
            "open_marker_path_mismatch",
            "open marker opened_path does not match opened registry open.path",
            Some(marker_path),
        ));
    }
    if value_string(registry, &["state", "last_known"]).is_none() {
        issues.push(DoctorIssue::error(
            "missing_open_registry_state",
            "opened registry must contain state.last_known",
            Some(registry_path.to_path_buf()),
        ));
    }
}

fn check_active_state_files(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let active = cf.join("active");
    let active_root = match crate::secure_worktree::open_absolute_dir_nofollow(&active) {
        Ok(active_root) => active_root,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                format!("active-state root cannot be pinned no-follow: {error}"),
                Some(active),
            ));
            return Ok(0);
        }
    };
    let root_identity = crate::secure_worktree::StableFsIdentity::for_directory(&active_root)?;
    let issue_start = issues.len();
    let mut names = active_root
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut checked = 0usize;
    for name in names {
        let mut state_dir = active.clone();
        state_dir.push(&name);
        let metadata = match active_root.symlink_metadata(&name) {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    format!("active-state entry cannot be inspected no-follow: {error}"),
                    Some(state_dir),
                ));
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                "active-state instance must not be a symlink",
                Some(state_dir),
            ));
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        let pinned_instance = match active_root.open_dir_nofollow(&name) {
            Ok(instance) => instance,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    format!("active-state instance cannot be pinned no-follow: {error}"),
                    Some(state_dir),
                ));
                continue;
            }
        };
        let instance_identity =
            crate::secure_worktree::StableFsIdentity::for_directory(&pinned_instance)?;
        let instance_issue_start = issues.len();
        let instance_checked = check_active_state_dir(&state_dir, issues);
        let rebound = active_root.open_dir_nofollow(&name);
        if rebound
            .and_then(|directory| {
                instance_identity.ensure_directory(&directory, "active-state instance")
            })
            .is_err()
        {
            issues.truncate(instance_issue_start);
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                "active-state instance identity changed during doctor inspection",
                Some(state_dir),
            ));
        } else {
            checked += instance_checked;
        }
    }
    if root_identity
        .ensure_directory(&active_root, "active-state root")
        .and_then(|()| crate::secure_worktree::ensure_absolute_dir_identity(&active_root, &active))
        .is_err()
    {
        issues.truncate(issue_start);
        issues.push(DoctorIssue::error(
            "invalid_active_state_dir",
            "active-state root identity changed during doctor inspection",
            Some(active),
        ));
        return Ok(0);
    }
    Ok(checked)
}

fn check_active_state_dir(state_dir: &Path, issues: &mut Vec<DoctorIssue>) -> usize {
    let active_state = match crate::active_state::ActiveStateDir::bind(state_dir) {
        Ok(active_state) => active_state,
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                error.to_string(),
                Some(state_dir.to_path_buf()),
            ));
            return 0;
        }
    };
    let state_path = active_state.display_path(Path::new("state.json"));
    match active_state.exists(Path::new("state.json")) {
        Ok(true) => {}
        Ok(false) => issues.push(DoctorIssue::error(
            "missing_active_state_file",
            "active state directory is missing required state.json",
            Some(state_path),
        )),
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                error.to_string(),
                Some(state_dir.to_path_buf()),
            ));
            return 0;
        }
    }
    let mut checked = 0usize;
    let mut state_value = None::<String>;
    let mut open_fire_count = None::<usize>;
    for &file_name in ACTIVE_STATE_FILES {
        let relative = Path::new(file_name);
        let path = active_state.display_path(relative);
        match active_state.exists(relative) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    error.to_string(),
                    Some(state_dir.to_path_buf()),
                ));
                break;
            }
        }
        match read_active_state_json(&active_state, relative, file_name) {
            Ok(value) => match validate_active_state_shape(file_name, value.clone()) {
                Ok(()) => {
                    if file_name == "state.json" {
                        state_value = value
                            .get("state")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    } else if file_name == "fires.json" {
                        open_fire_count = value.as_array().map(|fires| {
                            fires
                                .iter()
                                .filter(|fire| {
                                    fire.get("status").and_then(Value::as_str) == Some("open")
                                })
                                .count()
                        });
                    } else if file_name == "commit_transaction.json"
                        && value.get("status").and_then(Value::as_str) == Some("pending")
                    {
                        let phase = value
                            .get("phase")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let (stored_objects, planned_objects) = value
                            .get("objects")
                            .and_then(Value::as_array)
                            .map_or((0, 0), |objects| {
                                objects.iter().fold((0, 0), |(stored, planned), object| {
                                    if object.is_string()
                                        || object.get("state").and_then(Value::as_str)
                                            == Some("stored")
                                    {
                                        (stored + 1, planned)
                                    } else if object.get("state").and_then(Value::as_str)
                                        == Some("planned")
                                    {
                                        (stored, planned + 1)
                                    } else {
                                        (stored, planned)
                                    }
                                })
                            });
                        let pending_authority = pending_commit_authority(&value);
                        issues.push(DoctorIssue::pending_authority(
                            "pending_commit_transaction",
                            format!(
                                "commit transaction is pending at phase {phase} with {stored_objects} stored object(s) and {planned_objects} planned object(s)"
                            ),
                            path.clone(),
                            pending_authority,
                        ));
                    } else if file_name == "state_transaction.json"
                        && value.get("status").and_then(Value::as_str) == Some("pending")
                    {
                        let operation = value
                            .get("operation")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let phase = value
                            .get("phase")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let intended_state = value
                            .get("intended_state")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let pending_authority = pending_state_authority(&value);
                        issues.push(DoctorIssue::pending_authority(
                            "pending_state_transaction",
                            format!(
                                "{operation} state transaction is pending at phase {phase} for intended state {intended_state}"
                            ),
                            path.clone(),
                            pending_authority,
                        ));
                    } else if file_name
                        == crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE
                    {
                        match crate::fire::active_ledger_transaction::validate_active_ledger_descriptor(&active_state) {
                            Ok(Some(descriptor)) if descriptor.publication_status != crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => {
                                let pending_authority = pending_active_ledger_authority(&descriptor);
                                let message = match descriptor.publication_status {
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::LedgerPending => format!(
                                        "active ledger transaction {} ({}) is pending at phase {}",
                                        descriptor.request_hash, descriptor.operation, descriptor.phase
                                    ),
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::StatePublicationPending => format!(
                                        "active ledger transaction {} ({}) completed ledger phase {} and is awaiting state publication",
                                        descriptor.request_hash, descriptor.operation, descriptor.phase
                                    ),
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => unreachable!("released descriptors were filtered above"),
                                };
                                issues.push(DoctorIssue::pending_authority(
                                    "pending_active_ledger_transaction",
                                    message,
                                    path.clone(),
                                    pending_authority,
                                ));
                            }
                            Ok(Some(_)) | Ok(None) => {}
                            Err(error) => issues.push(DoctorIssue::error(
                                "invalid_active_ledger_transaction",
                                error.to_string(),
                                Some(path.clone()),
                            )),
                        }
                    }
                    checked += 1;
                }
                Err(message) => issues.push(DoctorIssue::error(
                    "invalid_active_state_shape",
                    message,
                    Some(path),
                )),
            },
            Err(error) => issues.push(DoctorIssue::error(
                "invalid_active_state_json",
                error.to_string(),
                Some(path),
            )),
        }
    }
    if matches!(
        state_value.as_deref(),
        Some("open-clean" | "open-consistent")
    ) && open_fire_count.unwrap_or(0) > 0
    {
        issues.push(DoctorIssue::error(
            "active_state_invariant_violation",
            "state is clean/consistent but fires.json contains open fires",
            Some(state_dir.to_path_buf()),
        ));
    }
    checked
}

fn pending_commit_authority(value: &Value) -> DoctorPendingAuthority {
    let phase = value
        .get("phase")
        .and_then(Value::as_str)
        .expect("shared commit transaction validator requires phase");
    let transaction_id = value
        .get("transaction_id")
        .and_then(Value::as_str)
        .expect("shared commit transaction validator requires transaction_id");
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::Commit,
        operation: DoctorPendingOperation::Commit,
        phase: phase.to_string(),
        request_identity: bounded_doctor_identity(transaction_id)
            .map(|transaction_id| DoctorPendingRequestIdentity::TransactionId { transaction_id })
            .unwrap_or(DoctorPendingRequestIdentity::Withheld {
                reason: "identifier_not_safe_for_diagnostics",
            }),
        intended_state: None,
    }
}

fn pending_state_authority(value: &Value) -> DoctorPendingAuthority {
    let operation = value
        .get("operation")
        .and_then(Value::as_str)
        .and_then(DoctorPendingOperation::from_validated)
        .expect("shared state transaction validator requires a supported operation");
    let phase = value
        .get("phase")
        .and_then(Value::as_str)
        .expect("shared state transaction validator requires phase");
    let intended_state = value
        .get("intended_state")
        .and_then(Value::as_str)
        .expect("shared state transaction validator requires intended_state");
    let request_identity = if value.get("version").and_then(Value::as_u64) == Some(2) {
        if let Some(request_hash) = value.get("request_hash").and_then(Value::as_str) {
            DoctorPendingRequestIdentity::RequestHash {
                request_hash: request_hash.to_string(),
            }
        } else {
            let subject = (
                value.pointer("/subject/branch").and_then(Value::as_str),
                value
                    .pointer("/subject/open_instance_id")
                    .and_then(Value::as_str),
                value
                    .pointer("/subject/current_base_commit")
                    .and_then(Value::as_str),
            );
            match subject {
                (Some(branch), Some(open_instance_id), Some(current_base_commit)) => {
                    match (
                        bounded_doctor_identity(branch),
                        bounded_doctor_identity(open_instance_id),
                        bounded_doctor_identity(current_base_commit),
                    ) {
                        (Some(branch), Some(open_instance_id), Some(current_base_commit)) => {
                            DoctorPendingRequestIdentity::StateSubject {
                                branch,
                                open_instance_id,
                                current_base_commit,
                            }
                        }
                        _ => DoctorPendingRequestIdentity::Withheld {
                            reason: "identifier_not_safe_for_diagnostics",
                        },
                    }
                }
                _ => unreachable!("shared version-2 state transaction validator requires subject"),
            }
        }
    } else {
        DoctorPendingRequestIdentity::LegacyUnbound
    };
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::State,
        operation,
        phase: phase.to_string(),
        request_identity,
        intended_state: Some(intended_state.to_string()),
    }
}

fn pending_active_ledger_authority(
    descriptor: &crate::fire::active_ledger_transaction::ValidatedActiveLedgerDescriptor,
) -> DoctorPendingAuthority {
    let operation = DoctorPendingOperation::from_validated(&descriptor.operation)
        .expect("shared active-ledger descriptor requires a supported operation");
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::ActiveLedger,
        operation,
        phase: descriptor.phase.clone(),
        request_identity: DoctorPendingRequestIdentity::RequestHash {
            request_hash: descriptor.request_hash.clone(),
        },
        intended_state: None,
    }
}

fn bounded_doctor_identity(value: &str) -> Option<String> {
    (!value.is_empty()
        && value.len() <= MAX_DOCTOR_IDENTITY_BYTES
        && !value.chars().any(char::is_control))
    .then(|| value.to_string())
}

fn read_active_state_json(
    active_state: &crate::active_state::ActiveStateDir,
    relative: &Path,
    file_name: &str,
) -> Result<Value, CliError> {
    let limit = match file_name {
        "state.json" => ACTIVE_CONTROL_FILE_BYTE_LIMIT,
        "state_transaction.json" => crate::STATE_TRANSACTION_BYTE_LIMIT,
        "commit_transaction.json" => crate::COMMIT_TRANSACTION_BYTE_LIMIT,
        crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE => {
            crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT
        }
        _ => ACTIVE_DATA_FILE_BYTE_LIMIT,
    };
    let bytes = active_state.read_required(relative, limit)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn read_doctor_json(path: &Path, limit: u64) -> Result<Value, CliError> {
    let bytes = crate::secure_worktree::read_file_nofollow_bounded(path, limit)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate_active_state_shape(file_name: &str, value: Value) -> Result<(), String> {
    match file_name {
        "state.json" => {
            let state = value
                .get("state")
                .and_then(Value::as_str)
                .ok_or_else(|| "state.json must contain string field state".to_string())?;
            if !matches!(state, "open-clean" | "open-burning" | "open-consistent") {
                return Err(format!("state.json contains unsupported state: {state}"));
            }
            if value
                .get("pending_merge_parent")
                .is_some_and(|parent| !parent.is_string())
            {
                return Err("state.json pending_merge_parent must be a string".to_string());
            }
            Ok(())
        }
        "fires.json" => serde_json::from_value::<Vec<codefire_core::Fire>>(value)
            .map(|_| ())
            .map_err(|error| format!("fires.json shape is invalid: {error}")),
        "resolutions.json" => serde_json::from_value::<Vec<codefire_core::Resolution>>(value)
            .map(|_| ())
            .map_err(|error| format!("resolutions.json shape is invalid: {error}")),
        "scan.json" => serde_json::from_value::<codefire_core::ScanResult>(value)
            .map(|_| ())
            .map_err(|error| format!("scan.json shape is invalid: {error}")),
        "verification.json" => serde_json::from_value::<codefire_core::Verification>(value)
            .map(|_| ())
            .map_err(|error| format!("verification.json shape is invalid: {error}")),
        "commit_transaction.json" => validate_commit_transaction_shape(value),
        "state_transaction.json" => validate_state_transaction_shape(value),
        _ => Ok(()),
    }
}

fn validate_state_transaction_shape(value: Value) -> Result<(), String> {
    crate::validate_state_transaction_record(&value)
}

fn validate_commit_transaction_shape(value: Value) -> Result<(), String> {
    crate::validate_commit_transaction_record(&value)
}

fn value_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(ToString::to_string)
}

fn path_string(value: &Value, path: &[&str]) -> Option<PathBuf> {
    value_string(value, path).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn local_ledger(command: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            command,
        )
        .unwrap()
    }

    fn assert_no_current_or_permits(ledger: &codefire_core::DecisionWorkLedger) {
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
    }

    fn leave_pending_open_transaction(repo: &Path, target: &Path, failpoint: &str) -> Vec<u8> {
        crate::init_repo(repo, false).unwrap();
        let options = crate::OpenOptions {
            branch: "main".to_string(),
            path: target.to_path_buf(),
            dry_run: false,
            json_output: true,
            lock: crate::LockOptions::default(),
            idempotency_key: None,
        };
        crate::open_transaction::set_open_transaction_failure_for_test(repo, failpoint);
        crate::open_transaction::open_branch_from(repo, &options).unwrap_err();
        fs::read(repo.join(".codefire/open_transaction.json")).unwrap()
    }

    #[test]
    fn layout_issue_retained_limit_minus_one_exact_and_plus_one_is_preallocation() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("missing/.codefire");
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let issue_bytes = doctor_issue_logical_bytes(
            "missing_codefire_dir".len(),
            "missing .codefire directory".len(),
            cf.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap();
        let collection_bytes = DOCTOR_OWNER_FRAME_BYTES + issue_bytes;

        for peak_delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor layout retained boundary",
            )
            .unwrap();
            let baseline = match peak_delta {
                -1 => limit - collection_bytes - 1,
                0 => limit - collection_bytes,
                1 => limit - collection_bytes + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            let result = check_repo_layout(&cf, &mut issues, &mut ledger);

            if peak_delta <= 0 {
                result.unwrap();
                assert_eq!(issues.layout().len(), 1);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if peak_delta == 0 { limit } else { limit - 1 }
                );
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("Doctor layout limit+1 must be typed resource exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, limit + 1);
                assert!(issues.layout().is_empty());
                assert!(!ledger.has_open_permits());
            }
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn branch_issue_retained_limit_minus_one_exact_plus_one_and_foreign_are_preallocation() {
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let kind = "invalid_branch_record";
        let message = "branch fixture is malformed";
        let path = Path::new("/repo/.codefire/branches/main.json");
        let issue_bytes = DOCTOR_OWNER_FRAME_BYTES
            + kind.len() as u64
            + message.len() as u64
            + path.as_os_str().as_encoded_bytes().len() as u64
            + 1;
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "Doctor branch issue retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - DOCTOR_OWNER_FRAME_BYTES - issue_bytes - 1,
                0 => limit - DOCTOR_OWNER_FRAME_BYTES - issue_bytes,
                1 => limit - DOCTOR_OWNER_FRAME_BYTES - issue_bytes + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = issues.push_accounted_branch_issue(
                &mut ledger,
                super::super::DoctorSeverity::Error,
                kind,
                format_args!("{message}"),
                path,
            );
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.residual_mut(ledger.identity()).unwrap().len(), 1);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if delta == 0 { limit } else { limit - 1 }
                );
            } else {
                let error = result.unwrap_err();
                let codefire_store::StoreError::DecisionWorkAccounting(accounting) = error else {
                    panic!("expected typed branch diagnostic accounting error");
                };
                let exhaustion = accounting.budget_exceeded().unwrap();
                assert_eq!(exhaustion.observed, limit + 1);
                assert!(issues.residual_mut(ledger.identity()).unwrap().is_empty());
                assert!(!ledger.has_open_permits());
            }
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("Doctor branch issue owner");
        let mut foreign = local_ledger("Doctor branch issue foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        assert!(issues
            .push_accounted_branch_issue(
                &mut foreign,
                super::super::DoctorSeverity::Error,
                kind,
                format_args!("{message}"),
                path,
            )
            .is_err());
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);
    }

    #[test]
    fn huge_missing_layout_path_and_multiple_layout_issues_stay_owned() {
        let huge = "x".repeat(256 * 1024);
        let cf = PathBuf::from("/missing").join(huge);
        let mut ledger = local_ledger("doctor huge missing layout");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_repo_layout(&cf, &mut issues, &mut ledger).unwrap();

        assert_eq!(issues.layout().len(), 1);
        assert_eq!(issues.layout()[0].path.as_deref(), Some(cf.as_path()));
        assert!(issues.accounted_retained_bytes > 256 * 1024);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(!ledger.has_open_permits());
        issues
            .residual_mut(ledger.identity())
            .unwrap()
            .push(DoctorIssue::warning(
                "residual_fixture",
                "later collector fixture",
                None,
            ));
        let layout_count = issues.layout().len();
        let (final_issues, preaccounted) = issues.finalize_report_issues(&mut ledger).unwrap();
        let preaccounted_bytes = preaccounted.retained_bytes_for(&ledger).unwrap();
        assert_eq!(final_issues.len(), layout_count + 1);
        assert_eq!(final_issues[layout_count].kind, "residual_fixture");
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            preaccounted_bytes
        );
        drop(final_issues);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                preaccounted_bytes,
            )
            .unwrap();
        assert_no_current_or_permits(&ledger);

        let temp = tempdir().unwrap();
        let cf = temp.path().join(".codefire");
        fs::create_dir(&cf).unwrap();
        let mut ledger = local_ledger("doctor multiple missing layout");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_repo_layout(&cf, &mut issues, &mut ledger).unwrap();

        assert!(issues.layout().len() > 2);
        assert_eq!(issues.layout()[0].kind, "missing_repo_directory");
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn layout_issue_fault_cancels_permit_and_drops_partial_collection() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("missing/.codefire");
        let mut ledger = local_ledger("doctor layout injected failure");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut fault = || {
            Err(CliError::Io(std::io::Error::other(
                "injected Doctor issue allocation failure",
            )))
        };

        let error =
            check_repo_layout_with_hook(&cf, &mut issues, &mut ledger, &mut fault).unwrap_err();

        assert!(matches!(error, CliError::Io(_)));
        assert!(issues.layout().is_empty());
        assert!(!ledger.has_open_permits());
        assert_eq!(
            ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
            DOCTOR_OWNER_FRAME_BYTES
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn accounted_layout_path_rejects_foreign_ledger_and_size_overflow() {
        let mut owner = local_ledger("doctor layout path owner");
        let mut foreign = local_ledger("doctor layout path foreign");
        let mut path = AccountedDoctorPath::prepare(
            &mut owner,
            Path::new("/repo/.codefire"),
            &[Path::new("objects")],
        )
        .unwrap();

        let error = path.release(&mut foreign).unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(owner.snapshot().current.values().any(|amount| *amount > 0));
        assert_no_current_or_permits(&foreign);
        path.release(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert!(doctor_issue_logical_bytes(usize::MAX, 1, 1).is_err());
    }

    #[test]
    fn issue_collection_rejects_foreign_ledger_without_changing_either_owner() {
        let mut owner = local_ledger("doctor issue collection owner");
        let mut foreign = local_ledger("doctor issue collection foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        let mut hook_called = false;
        let mut hook = || {
            hook_called = true;
            Ok(())
        };

        let error = issues
            .push_layout_issue(
                &mut foreign,
                DoctorLayoutIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind: "foreign_ledger_fixture",
                    message_parts: &["must not allocate"],
                    path_base: Path::new("/repo/.codefire"),
                    path_parts: &[],
                },
                &mut hook,
            )
            .unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(!hook_called);
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(owner.open_permit_count(), 0);
        assert_eq!(foreign.open_permit_count(), 0);
        assert!(issues.layout().is_empty());
        assert_eq!(issues.accounted_retained_bytes, DOCTOR_OWNER_FRAME_BYTES);

        let error = issues.residual_mut(foreign.identity()).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);

        let error = issues.cancel(&mut foreign).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert!(issues.layout().is_empty());
        assert_eq!(issues.accounted_retained_bytes, DOCTOR_OWNER_FRAME_BYTES);

        let error = issues.finalize_report_issues(&mut foreign).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(owner.open_permit_count(), 0);
        assert_eq!(foreign.open_permit_count(), 0);

        check_repo_layout(Path::new("/missing/.codefire"), &mut issues, &mut owner).unwrap();
        assert_eq!(issues.layout().len(), 1);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);
    }

    #[test]
    fn open_transaction_marker_accounts_clean_pending_malformed_unknown_and_removing() {
        let temp = tempdir().unwrap();
        let bare_cf = temp.path().join("bare/.codefire");
        fs::create_dir_all(&bare_cf).unwrap();
        let mut ledger = local_ledger("doctor open transaction clean");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_open_transaction(&bare_cf, &mut issues, &mut ledger).unwrap();

        assert!(issues.transaction().is_empty());
        assert_eq!(issues.accounted_retained_bytes, DOCTOR_OWNER_FRAME_BYTES);
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        for (name, bytes) in [
            ("malformed", b"{".as_slice()),
            (
                "unknown-version",
                br#"{"type":"codefire_open_transaction","version":999,"status":"pending"}"#
                    .as_slice(),
            ),
        ] {
            fs::write(bare_cf.join("open_transaction.json"), bytes).unwrap();
            let mut ledger = local_ledger(&format!("doctor open transaction {name}"));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            check_open_transaction(&bare_cf, &mut issues, &mut ledger).unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
            assert_eq!(
                ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
                bytes.len() as u64
            );
            assert_eq!(
                ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
                1
            );
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let canonical = fs::read(bare_cf.join("open_transaction.json")).unwrap();
        fs::write(bare_cf.join("open_transaction.removing.json"), b"{}").unwrap();
        let mut ledger = local_ledger("doctor conflicting open marker quarantine");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_open_transaction(&bare_cf, &mut issues, &mut ledger).unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
        assert!(issues.transaction()[0]
            .message
            .contains("both the canonical marker and its removal quarantine exist"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            (canonical.len() + 2) as u64
        );
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
        fs::remove_file(bare_cf.join("open_transaction.json")).unwrap();
        fs::remove_file(bare_cf.join("open_transaction.removing.json")).unwrap();

        let pending_repo = temp.path().join("pending-repo");
        let pending_target = temp.path().join("pending-open");
        let pending =
            leave_pending_open_transaction(&pending_repo, &pending_target, "after_prepared");
        let mut ledger = local_ledger("doctor open transaction pending");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_open_transaction(&pending_repo.join(".codefire"), &mut issues, &mut ledger).unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "pending_open_transaction");
        assert!(issues.transaction()[0].message.contains("phase prepared"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            pending.len() as u64
        );
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        let removing_repo = temp.path().join("removing-repo");
        let removing_target = temp.path().join("removing-open");
        let removing = leave_pending_open_transaction(
            &removing_repo,
            &removing_target,
            "after_complete_archive_before_marker_removal",
        );
        fs::rename(
            removing_repo.join(".codefire/open_transaction.json"),
            removing_repo.join(".codefire/open_transaction.removing.json"),
        )
        .unwrap();
        let mut ledger = local_ledger("doctor removing open transaction");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        check_open_transaction(&removing_repo.join(".codefire"), &mut issues, &mut ledger).unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "pending_open_transaction");
        assert!(issues.transaction()[0]
            .message
            .contains("phase idempotency_written"));
        assert_eq!(
            issues.transaction()[0].path.as_deref(),
            Some(
                removing_repo
                    .join(".codefire/open_transaction.json")
                    .as_path()
            )
        );
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            removing.len() as u64
        );
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn simultaneous_open_close_markers_keep_legacy_transaction_issue_order() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        fs::write(cf.join("open_transaction.json"), b"{").unwrap();
        fs::write(cf.join("close_transaction.json"), b"{").unwrap();
        let mut ledger = local_ledger("doctor simultaneous open close order");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

        let open_present = check_open_transaction(&cf, &mut issues, &mut ledger).unwrap();
        let close_present = check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();
        check_transaction_conflict(open_present, close_present, &cf, &mut issues, &mut ledger)
            .unwrap();

        assert_eq!(
            issues
                .transaction()
                .iter()
                .map(|issue| issue.kind.as_str())
                .collect::<Vec<_>>(),
            vec![
                "invalid_open_transaction",
                "invalid_close_transaction",
                "conflicting_repository_transactions"
            ]
        );
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn transaction_conflict_uses_accounted_collector_presence_for_every_combination() {
        for (open_expected, close_expected) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let temp = tempdir().unwrap();
            let cf = temp.path().join("repo/.codefire");
            fs::create_dir_all(&cf).unwrap();
            if open_expected {
                fs::write(cf.join("open_transaction.json"), b"{").unwrap();
            }
            if close_expected {
                fs::write(cf.join("close_transaction.json"), b"{").unwrap();
            }
            let mut ledger = local_ledger("doctor transaction conflict presence combinations");
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            let open_present = check_open_transaction(&cf, &mut issues, &mut ledger).unwrap();
            let close_present = check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();
            check_transaction_conflict(open_present, close_present, &cf, &mut issues, &mut ledger)
                .unwrap();

            assert_eq!(open_present, open_expected);
            assert_eq!(close_present, close_expected);
            let conflicts = issues
                .transaction()
                .iter()
                .filter(|issue| issue.kind == DOCTOR_TRANSACTION_CONFLICT_KIND)
                .collect::<Vec<_>>();
            assert_eq!(
                conflicts.len(),
                usize::from(open_expected && close_expected)
            );
            if let Some(conflict) = conflicts.first() {
                assert_eq!(conflict.message, DOCTOR_TRANSACTION_CONFLICT_MESSAGE);
                assert_eq!(conflict.path.as_deref(), Some(cf.as_path()));
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn transaction_conflict_presence_is_stable_across_candidate_races() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("removed-after-open/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let open = cf.join("open_transaction.json");
        fs::write(&open, b"{").unwrap();
        fs::write(cf.join("close_transaction.json"), b"{").unwrap();
        let mut ledger = local_ledger("doctor conflict removed open candidate");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let open_present = check_open_transaction_with_hooks(
            &cf,
            &mut issues,
            &mut ledger,
            &mut || {},
            &mut || fs::remove_file(&open).unwrap(),
        )
        .unwrap();
        let close_present = check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();

        check_transaction_conflict(open_present, close_present, &cf, &mut issues, &mut ledger)
            .unwrap();

        assert!(open_present);
        assert!(close_present);
        assert!(!open.exists());
        assert_eq!(
            issues
                .transaction()
                .iter()
                .filter(|issue| issue.kind == DOCTOR_TRANSACTION_CONFLICT_KIND)
                .count(),
            1
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        let cf = temp.path().join("inserted-after-open/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let open = cf.join("open_transaction.json");
        fs::write(cf.join("close_transaction.json"), b"{").unwrap();
        let mut ledger = local_ledger("doctor conflict late open candidate");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let open_present = check_open_transaction(&cf, &mut issues, &mut ledger).unwrap();
        let close_present = check_close_transaction_with_hooks(
            &cf,
            &mut issues,
            &mut ledger,
            &mut || fs::write(&open, b"{").unwrap(),
            &mut || {},
        )
        .unwrap();

        check_transaction_conflict(open_present, close_present, &cf, &mut issues, &mut ledger)
            .unwrap();

        assert!(!open_present);
        assert!(close_present);
        assert!(open.exists());
        assert!(issues
            .transaction()
            .iter()
            .all(|issue| issue.kind != DOCTOR_TRANSACTION_CONFLICT_KIND));
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn transaction_conflict_retained_boundary_and_foreign_ledger_are_preallocation_safe() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let conflict_retained = doctor_issue_logical_bytes(
            DOCTOR_TRANSACTION_CONFLICT_KIND.len(),
            DOCTOR_TRANSACTION_CONFLICT_MESSAGE.len(),
            cf.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap()
            + 1;
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let retained_limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor conflict retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - conflict_retained - 1,
                0 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - conflict_retained,
                1 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - conflict_retained + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            let result = check_transaction_conflict(true, true, &cf, &mut issues, &mut ledger);

            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("transaction conflict retained limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, retained_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("doctor conflict owner");
        let mut foreign = local_ledger("doctor conflict foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let error =
            check_transaction_conflict(true, true, &cf, &mut issues, &mut foreign).unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert!(issues.transaction().is_empty());
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);
    }

    #[test]
    fn close_transaction_marker_accounts_clean_malformed_and_unknown_candidates() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");

        let mut ledger = local_ledger("doctor close transaction clean");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();
        assert!(issues.transaction().is_empty());
        assert_eq!(issues.accounted_retained_bytes, DOCTOR_OWNER_FRAME_BYTES);
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        for (name, bytes) in [
            ("malformed", b"{".as_slice()),
            (
                "unknown-version",
                br#"{"type":"codefire_close_transaction","version":999,"status":"pending"}"#
                    .as_slice(),
            ),
        ] {
            fs::write(&marker, bytes).unwrap();
            let mut ledger = local_ledger(&format!("doctor close transaction {name}"));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
            assert_eq!(
                ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
                bytes.len() as u64
            );
            assert_eq!(
                ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
                1
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
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn close_transaction_marker_rejects_special_and_oversized_files_without_read_charge() {
        use std::os::unix::fs::symlink;

        for kind in ["symlink", "fifo", "oversize"] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join("repo/.codefire");
            fs::create_dir_all(&cf).unwrap();
            let marker = cf.join("close_transaction.json");
            match kind {
                "symlink" => symlink(temp.path().join("outside"), &marker).unwrap(),
                "fifo" => {
                    let status = std::process::Command::new("mkfifo")
                        .arg(&marker)
                        .status()
                        .unwrap();
                    assert!(status.success());
                }
                "oversize" => {
                    let file = fs::File::create(&marker).unwrap();
                    file.set_len(crate::close_transaction::CLOSE_TRANSACTION_BYTE_LIMIT + 1)
                        .unwrap();
                }
                _ => unreachable!(),
            }
            let mut ledger = local_ledger(&format!("doctor close transaction {kind}"));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            check_close_transaction(&cf, &mut issues, &mut ledger).unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
            assert_eq!(
                ledger
                    .snapshot()
                    .cumulative
                    .get(&codefire_core::DecisionWorkDimension::AggregateBytes)
                    .copied()
                    .unwrap_or_default(),
                0
            );
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn open_transaction_marker_rejects_special_and_oversized_files_without_read_charge() {
        use std::os::unix::fs::symlink;

        for kind in ["symlink", "fifo", "oversize"] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join("repo/.codefire");
            fs::create_dir_all(&cf).unwrap();
            let marker = cf.join("open_transaction.json");
            match kind {
                "symlink" => symlink(temp.path().join("outside"), &marker).unwrap(),
                "fifo" => {
                    let status = std::process::Command::new("mkfifo")
                        .arg(&marker)
                        .status()
                        .unwrap();
                    assert!(status.success());
                }
                "oversize" => {
                    let file = fs::File::create(&marker).unwrap();
                    file.set_len(crate::open_transaction::OPEN_TRANSACTION_BYTE_LIMIT + 1)
                        .unwrap();
                }
                _ => unreachable!(),
            }
            let mut ledger = local_ledger(&format!("doctor open transaction {kind}"));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            check_open_transaction(&cf, &mut issues, &mut ledger).unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
            assert_eq!(
                ledger
                    .snapshot()
                    .cumulative
                    .get(&codefire_core::DecisionWorkDimension::AggregateBytes)
                    .copied()
                    .unwrap_or_default(),
                0
            );
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn canonical_open_marker_preserves_removing_special_file_error_precedence() {
        use std::os::unix::fs::symlink;

        for kind in ["symlink", "fifo", "oversize"] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join("repo/.codefire");
            fs::create_dir_all(&cf).unwrap();
            let canonical = cf.join("open_transaction.json");
            let removing = cf.join("open_transaction.removing.json");
            let canonical_bytes = b"{";
            fs::write(&canonical, canonical_bytes).unwrap();
            match kind {
                "symlink" => symlink(temp.path().join("outside"), &removing).unwrap(),
                "fifo" => {
                    let status = std::process::Command::new("mkfifo")
                        .arg(&removing)
                        .status()
                        .unwrap();
                    assert!(status.success());
                }
                "oversize" => {
                    let file = fs::File::create(&removing).unwrap();
                    file.set_len(crate::open_transaction::OPEN_TRANSACTION_BYTE_LIMIT + 1)
                        .unwrap();
                }
                _ => unreachable!(),
            }
            let mut ledger = local_ledger(&format!(
                "doctor canonical and special removing open marker {kind}"
            ));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();

            check_open_transaction(&cf, &mut issues, &mut ledger).unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
            assert!(!issues.transaction()[0]
                .message
                .contains("both the canonical marker and its removal quarantine exist"));
            assert!(!issues.transaction()[0].message.contains("invalid JSON"));
            assert_eq!(
                ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
                canonical_bytes.len() as u64
            );
            assert_eq!(
                ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
                1
            );
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn open_marker_attempt_orders_canonical_then_removing_across_appearance_races() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let canonical = cf.join("open_transaction.json");
        let removing = cf.join("open_transaction.removing.json");

        fs::write(&canonical, b"{").unwrap();
        let mut ledger = local_ledger("doctor removing appears after canonical read");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        check_open_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut || {}, &mut || {
            fs::write(&removing, b"{}").unwrap()
        })
        .unwrap();
        assert!(issues.transaction()[0]
            .message
            .contains("both the canonical marker and its removal quarantine exist"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            3
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
            2
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        fs::write(&canonical, b"{").unwrap();
        fs::write(&removing, b"{}").unwrap();
        let mut ledger = local_ledger("doctor removing disappears after canonical read");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        check_open_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut || {}, &mut || {
            fs::remove_file(&removing).unwrap()
        })
        .unwrap();
        assert!(issues.transaction()[0].message.contains("invalid JSON"));
        assert!(!issues.transaction()[0]
            .message
            .contains("both the canonical marker and its removal quarantine exist"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
            1
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        fs::remove_file(&canonical).unwrap();
        let mut ledger = local_ledger("doctor removing appears after missing canonical read");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        check_open_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut || {}, &mut || {
            fs::write(&removing, b"{").unwrap()
        })
        .unwrap();
        assert!(issues.transaction()[0].message.contains("invalid JSON"));
        assert!(issues.transaction()[0]
            .message
            .contains(&removing.display().to_string()));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            1
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);

        fs::write(&removing, b"{").unwrap();
        let mut ledger = local_ledger("doctor canonical appears after missing canonical read");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        check_open_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut || {}, &mut || {
            fs::write(&canonical, b"{}").unwrap()
        })
        .unwrap();
        assert!(canonical.exists());
        assert!(issues.transaction()[0].message.contains("invalid JSON"));
        assert!(issues.transaction()[0]
            .message
            .contains(&removing.display().to_string()));
        assert!(!issues.transaction()[0]
            .message
            .contains("both the canonical marker and its removal quarantine exist"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            1
        );
        assert_eq!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::FilesRead],
            1
        );
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn close_transaction_marker_subject_change_is_invalid_and_fully_released() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");
        let moved = cf.join("close_transaction.original.json");
        let original = br#"{"type":"codefire_close_transaction","version":999,"status":"pending"}"#;
        fs::write(&marker, original).unwrap();
        let mut ledger = local_ledger("doctor close transaction subject change");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut replace = || {
            fs::rename(&marker, &moved).unwrap();
            fs::write(&marker, b"{}").unwrap();
        };

        check_close_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut || {}, &mut replace)
            .unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
        assert!(issues.transaction()[0]
            .message
            .contains("authority changed during Doctor inspection"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            original.len() as u64
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
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn close_transaction_marker_parent_replacement_rejects_same_inode_move() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let cf = repo.join(".codefire");
        let old_cf = repo.join(".codefire-original");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");
        let original = br#"{"type":"codefire_close_transaction","version":999,"status":"pending"}"#;
        fs::write(&marker, original).unwrap();
        let mut ledger = local_ledger("doctor close transaction parent replacement");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut replace_parent = || {
            fs::rename(&cf, &old_cf).unwrap();
            fs::create_dir(&cf).unwrap();
            fs::rename(old_cf.join("close_transaction.json"), &marker).unwrap();
        };

        check_close_transaction_with_hooks(
            &cf,
            &mut issues,
            &mut ledger,
            &mut || {},
            &mut replace_parent,
        )
        .unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
        assert!(issues.transaction()[0]
            .message
            .contains("authority changed during Doctor inspection"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            original.len() as u64
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
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn open_transaction_marker_parent_replacement_rejects_same_inode_move() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let cf = repo.join(".codefire");
        let old_cf = repo.join(".codefire-original");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("open_transaction.json");
        let original = br#"{"type":"codefire_open_transaction","version":999,"status":"pending"}"#;
        fs::write(&marker, original).unwrap();
        let mut ledger = local_ledger("doctor open transaction parent replacement");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut replace_parent = || {
            fs::rename(&cf, &old_cf).unwrap();
            fs::create_dir(&cf).unwrap();
            fs::rename(old_cf.join("open_transaction.json"), &marker).unwrap();
        };

        check_open_transaction_with_hooks(
            &cf,
            &mut issues,
            &mut ledger,
            &mut || {},
            &mut replace_parent,
        )
        .unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
        assert!(issues.transaction()[0]
            .message
            .contains("authority changed during Doctor inspection"));
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            original.len() as u64
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
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn close_transaction_marker_growth_after_metadata_stays_within_fixed_reservation() {
        use std::io::Write;

        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");
        fs::write(&marker, b"{").unwrap();
        let grown_bytes = 256 * 1024_u64;
        let mut ledger = local_ledger("doctor close transaction metadata growth");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut grow = || {
            let mut file = fs::OpenOptions::new().append(true).open(&marker).unwrap();
            let chunk = vec![b' '; usize::try_from(grown_bytes - 1).unwrap()];
            file.write_all(&chunk).unwrap();
        };

        check_close_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut grow, &mut || {})
            .unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            grown_bytes
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
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn close_transaction_marker_replacement_after_metadata_stays_within_fixed_reservation() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");
        let old_marker = cf.join("close_transaction.original.json");
        fs::write(&marker, b"{").unwrap();
        let replacement_bytes = 256 * 1024_u64;
        let mut ledger = local_ledger("doctor close transaction metadata replacement");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut replace = || {
            fs::rename(&marker, &old_marker).unwrap();
            let mut bytes = vec![b' '; usize::try_from(replacement_bytes).unwrap()];
            bytes[0] = b'{';
            fs::write(&marker, bytes).unwrap();
        };

        check_close_transaction_with_hooks(&cf, &mut issues, &mut ledger, &mut replace, &mut || {})
            .unwrap();

        assert_eq!(issues.transaction().len(), 1);
        assert_eq!(issues.transaction()[0].kind, "invalid_close_transaction");
        assert_eq!(
            ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
            replacement_bytes
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
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn open_transaction_marker_metadata_growth_and_replacement_use_fixed_reservation() {
        use std::io::Write;

        for mode in ["append", "replace"] {
            let temp = tempdir().unwrap();
            let cf = temp.path().join("repo/.codefire");
            fs::create_dir_all(&cf).unwrap();
            let marker = cf.join("open_transaction.json");
            let old_marker = cf.join("open_transaction.original.json");
            fs::write(&marker, b"{").unwrap();
            let grown_bytes = 256 * 1024_u64;
            let mut ledger = local_ledger(&format!("doctor open transaction metadata {mode}"));
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let mut mutate = || {
                let mut chunk = vec![b' '; usize::try_from(grown_bytes).unwrap()];
                chunk[0] = b'{';
                match mode {
                    "append" => {
                        let mut file = fs::OpenOptions::new().append(true).open(&marker).unwrap();
                        file.write_all(&chunk[1..]).unwrap();
                    }
                    "replace" => {
                        fs::rename(&marker, &old_marker).unwrap();
                        fs::write(&marker, chunk).unwrap();
                    }
                    _ => unreachable!(),
                }
            };

            check_open_transaction_with_hooks(
                &cf,
                &mut issues,
                &mut ledger,
                &mut mutate,
                &mut || {},
            )
            .unwrap();

            assert_eq!(issues.transaction().len(), 1);
            assert_eq!(issues.transaction()[0].kind, "invalid_open_transaction");
            assert_eq!(
                ledger.snapshot().cumulative[&codefire_core::DecisionWorkDimension::AggregateBytes],
                grown_bytes
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
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn close_transaction_issue_retained_boundary_is_preallocation_and_foreign_safe() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("close_transaction.json");
        let bytes = b"{";
        fs::write(&marker, bytes).unwrap();
        let parse_error = serde_json::from_slice::<Value>(bytes).unwrap_err();
        let message = format!(
            "invalid pending close transaction {}: marker is not valid JSON: {parse_error}",
            marker.display()
        );
        let retained = doctor_issue_logical_bytes(
            "invalid_close_transaction".len(),
            message.len(),
            marker.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap()
            + 1;
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor close retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained - 1,
                0 => limit - DOCTOR_OWNER_FRAME_BYTES - retained,
                1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_close_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
            } else {
                assert!(matches!(result.unwrap_err(), CliError::ResourceBudget(_)));
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let aggregate_limit = profile
            .limit(codefire_core::DecisionWorkDimension::AggregateBytes)
            .unwrap()
            .limit;
        let marker_upper = crate::close_transaction::CLOSE_TRANSACTION_BYTE_LIMIT;
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor close aggregate boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => aggregate_limit - marker_upper - 1,
                0 => aggregate_limit - marker_upper,
                1 => aggregate_limit - marker_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(
                        codefire_core::DecisionWorkDimension::AggregateBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_close_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
                assert_eq!(
                    ledger.snapshot().cumulative
                        [&codefire_core::DecisionWorkDimension::AggregateBytes],
                    baseline + 1
                );
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("close marker aggregate limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::AggregateBytes
                );
                assert_eq!(exhaustion.observed, aggregate_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let temporary_limit = profile
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let marker_temporary_upper = marker_upper
            .checked_mul(DOCTOR_TRANSACTION_PARSE_MULTIPLIER)
            .and_then(|value| value.checked_add(64 * 1024))
            .unwrap();
        let marker_path_temporary = DOCTOR_OWNER_FRAME_BYTES
            + u64::try_from(marker.as_os_str().as_encoded_bytes().len()).unwrap();
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor close temporary boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => temporary_limit - marker_path_temporary - marker_temporary_upper - 1,
                0 => temporary_limit - marker_path_temporary - marker_temporary_upper,
                1 => temporary_limit - marker_path_temporary - marker_temporary_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_close_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("close marker temporary limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::TemporaryBytes
                );
                assert_eq!(exhaustion.observed, temporary_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("doctor close owner");
        let mut foreign = local_ledger("doctor close foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        let error = check_close_transaction(&cf, &mut issues, &mut foreign).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);
    }

    #[test]
    fn open_transaction_issue_limits_are_preallocation_and_foreign_safe() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        fs::create_dir_all(&cf).unwrap();
        let marker = cf.join("open_transaction.json");
        let bytes = b"{";
        fs::write(&marker, bytes).unwrap();
        let parse_error = serde_json::from_slice::<Value>(bytes).unwrap_err();
        let message = format!(
            "invalid CodeFire repository: invalid pending open transaction {}: invalid JSON: {parse_error}",
            marker.display()
        );
        let retained = doctor_issue_logical_bytes(
            "invalid_open_transaction".len(),
            message.len(),
            marker.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap()
            + 1;
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let retained_limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;

        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor open retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - retained - 1,
                0 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - retained,
                1 => retained_limit - DOCTOR_OWNER_FRAME_BYTES - retained + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_open_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("open marker retained limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, retained_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let aggregate_limit = profile
            .limit(codefire_core::DecisionWorkDimension::AggregateBytes)
            .unwrap()
            .limit;
        let marker_upper = crate::open_transaction::OPEN_TRANSACTION_BYTE_LIMIT;
        let inspection_aggregate_upper = marker_upper.checked_mul(2).unwrap();
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor open aggregate boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => aggregate_limit - inspection_aggregate_upper - 1,
                0 => aggregate_limit - inspection_aggregate_upper,
                1 => aggregate_limit - inspection_aggregate_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::Validation,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .charge(
                        codefire_core::DecisionWorkDimension::AggregateBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_open_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
                assert_eq!(
                    ledger.snapshot().cumulative
                        [&codefire_core::DecisionWorkDimension::AggregateBytes],
                    baseline + 1
                );
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("open marker aggregate limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::AggregateBytes
                );
                assert_eq!(exhaustion.observed, aggregate_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let temporary_limit = profile
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .unwrap()
            .limit;
        let marker_temporary_upper = marker_upper
            .checked_mul(DOCTOR_TRANSACTION_PARSE_MULTIPLIER)
            .and_then(|value| value.checked_add(64 * 1024))
            .unwrap();
        let removal_marker = cf.join("open_transaction.removing.json");
        let marker_path_temporary = DOCTOR_OWNER_FRAME_BYTES
            .checked_mul(2)
            .and_then(|value| {
                value.checked_add(
                    u64::try_from(marker.as_os_str().as_encoded_bytes().len()).unwrap(),
                )
            })
            .and_then(|value| {
                value.checked_add(
                    u64::try_from(removal_marker.as_os_str().as_encoded_bytes().len()).unwrap(),
                )
            })
            .unwrap();
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "doctor open temporary boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => temporary_limit - marker_path_temporary - marker_temporary_upper - 1,
                0 => temporary_limit - marker_path_temporary - marker_temporary_upper,
                1 => temporary_limit - marker_path_temporary - marker_temporary_upper + 1,
                _ => unreachable!(),
            };
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::TemporaryBytes,
                        baseline,
                    ),
                )
                .unwrap();
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = check_open_transaction(&cf, &mut issues, &mut ledger);
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.transaction().len(), 1);
            } else {
                let CliError::ResourceBudget(exhaustion) = result.unwrap_err() else {
                    panic!("open marker temporary limit+1 must be typed exhaustion");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::TemporaryBytes
                );
                assert_eq!(exhaustion.observed, temporary_limit + 1);
                assert!(issues.transaction().is_empty());
            }
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::TemporaryBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("doctor open owner");
        let mut foreign = local_ledger("doctor open foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        let error = check_open_transaction(&cf, &mut issues, &mut foreign).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);
    }

    #[test]
    fn accounted_doctor_scan_preserves_invalid_record_classification() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let initialized = crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let object_path =
            codefire_store::object_record_path(&cf.join("objects"), &initialized.main_commit)
                .unwrap();
        let mut record: Value = serde_json::from_slice(&fs::read(&object_path).unwrap()).unwrap();
        record["extensions"] = Value::String("bad".to_string());
        fs::write(&object_path, serde_json::to_vec(&record).unwrap()).unwrap();

        let checks = run_checks(&cf, false).unwrap();

        assert!(checks.issues.iter().any(|issue| {
            issue.kind == "invalid_object_record" && issue.path.as_ref() == Some(&object_path)
        }));
        assert!(checks
            .issues
            .iter()
            .all(|issue| issue.kind != "object_integrity_error"));
    }

    #[test]
    fn opened_registry_entry_diagnostics_preserve_wire_order() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let opened = cf.join("opened");
        let invalid = opened.join("000-invalid.json");
        let missing = opened.join("001-missing.json");
        fs::write(&invalid, b"{").unwrap();
        fs::write(&missing, b"{}").unwrap();

        let checks = run_checks(&cf, false).unwrap();
        assert_eq!(
            checks
                .issues
                .iter()
                .find(|issue| issue.path.as_ref() == Some(&invalid))
                .unwrap()
                .message,
            serde_json::from_slice::<Value>(b"{")
                .unwrap_err()
                .to_string()
        );
        let entry_kinds = checks
            .issues
            .iter()
            .filter(|issue| {
                issue.path.as_ref() == Some(&invalid) || issue.path.as_ref() == Some(&missing)
            })
            .map(|issue| issue.kind.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            entry_kinds,
            vec![
                "invalid_open_registry",
                "missing_open_branch_name",
                "missing_open_instance_id",
                "missing_open_path",
                "missing_open_base_commit",
                "missing_active_state_path",
            ]
        );
        assert!(!checks.ledger.has_open_permits());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn opened_registry_hardlink_preserves_invalid_classification_detail_and_order() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let opened = cf.join("opened");
        let source = cf.join("hardlink-source.json");
        let hardlink = opened.join("000-hardlink.json");
        let missing = opened.join("001-missing.json");
        fs::write(&source, b"{}").unwrap();
        fs::hard_link(&source, &hardlink).unwrap();
        fs::write(&missing, b"{}").unwrap();

        let checks = run_checks(&cf, false).unwrap();
        let entry_issues = checks
            .issues
            .iter()
            .filter(|issue| {
                issue.path.as_ref() == Some(&hardlink) || issue.path.as_ref() == Some(&missing)
            })
            .collect::<Vec<_>>();

        assert_eq!(entry_issues[0].kind, "invalid_open_registry");
        assert_eq!(entry_issues[0].path.as_ref(), Some(&hardlink));
        #[cfg(unix)]
        assert!(entry_issues[0]
            .message
            .contains("bounded authority must have exactly one filesystem link"));
        #[cfg(windows)]
        assert!(entry_issues[0]
            .message
            .contains("bounded authority must have one provable filesystem link"));
        assert!(entry_issues[0]
            .message
            .contains(&hardlink.display().to_string()));
        assert_eq!(
            entry_issues[1..]
                .iter()
                .map(|issue| issue.kind.as_str())
                .collect::<Vec<_>>(),
            vec![
                "missing_open_branch_name",
                "missing_open_instance_id",
                "missing_open_path",
                "missing_open_base_commit",
                "missing_active_state_path",
            ]
        );
        assert!(!checks.ledger.has_open_permits());
    }

    #[test]
    fn opened_issue_exact_reservation_foreign_and_error_cleanup_are_owner_safe() {
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let kind = "invalid_open_registry";
        let message = "opened registry fixture is malformed";
        let path = Path::new("/repo/.codefire/opened/invalid.json");
        let retained = doctor_issue_logical_bytes(
            kind.len(),
            message.len(),
            path.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap()
        .checked_add(1)
        .unwrap();
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "Doctor opened issue retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained - 1,
                0 => limit - DOCTOR_OWNER_FRAME_BYTES - retained,
                1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = issues.push_accounted_opened_issue(
                &mut ledger,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
            );
            if delta <= 0 {
                result.unwrap();
                let residual = issues.residual_mut(ledger.identity()).unwrap();
                assert_eq!(residual.len(), 1);
                assert_eq!(residual[0].message, message);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if delta == 0 { limit } else { limit - 1 }
                );
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) = result.unwrap_err()
                else {
                    panic!("expected typed opened diagnostic accounting error");
                };
                assert_eq!(error.budget_exceeded().unwrap().observed, limit + 1);
                assert!(issues.residual_mut(ledger.identity()).unwrap().is_empty());
                assert!(!ledger.has_open_permits());
            }
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("Doctor opened issue owner");
        let mut foreign = local_ledger("Doctor opened issue foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        assert!(issues
            .push_accounted_opened_issue(
                &mut foreign,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
            )
            .is_err());
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);

        let mut ledger = local_ledger("Doctor opened issue injected allocation failure");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let capacity_before = issues.residual_issues.as_ref().unwrap().capacity();
        let snapshot_before = ledger.snapshot().clone();
        let error = issues
            .push_accounted_opened_issue_with_hook(
                &mut ledger,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
                #[expect(
                    clippy::result_large_err,
                    reason = "the allocation fault fixture uses the Store callback error"
                )]
                || {
                    Err(codefire_store::StoreError::Io(std::io::Error::other(
                        "injected opened diagnostic allocation failure",
                    )))
                },
            )
            .unwrap_err();
        assert!(
            matches!(error, codefire_store::StoreError::Io(error) if error.to_string().contains("injected"))
        );
        assert_eq!(
            issues.residual_issues.as_ref().unwrap().capacity(),
            capacity_before
        );
        assert!(issues.residual_mut(ledger.identity()).unwrap().is_empty());
        assert_eq!(ledger.snapshot(), &snapshot_before);
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn object_issue_retained_limit_minus_one_exact_plus_one_and_foreign_are_preallocation() {
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let kind = "invalid_object_record";
        let message = "object fixture is malformed";
        let path = Path::new("/repo/.codefire/objects/atoms/invalid.json");
        let retained = doctor_issue_logical_bytes(
            kind.len(),
            message.len(),
            path.as_os_str().as_encoded_bytes().len(),
        )
        .unwrap()
        .checked_add(1)
        .unwrap();
        for delta in [-1_i64, 0, 1] {
            let mut ledger = codefire_core::DecisionWorkLedger::new(
                profile.clone(),
                codefire_core::DecisionWorkCommandKind::LocalGraph,
                "Doctor object issue retained boundary",
            )
            .unwrap();
            let baseline = match delta {
                -1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained - 1,
                0 => limit - DOCTOR_OWNER_FRAME_BYTES - retained,
                1 => limit - DOCTOR_OWNER_FRAME_BYTES - retained + 1,
                _ => unreachable!(),
            };
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
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let result = issues.push_accounted_object_issue(
                &mut ledger,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
            );
            if delta <= 0 {
                result.unwrap();
                assert_eq!(issues.pretransaction().len(), 1);
                assert_eq!(
                    ledger.snapshot().current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if delta == 0 { limit } else { limit - 1 }
                );
            } else {
                let codefire_store::StoreError::DecisionWorkAccounting(error) = result.unwrap_err()
                else {
                    panic!("expected typed object diagnostic accounting error");
                };
                let exhaustion = error.budget_exceeded().unwrap();
                assert_eq!(exhaustion.observed, limit + 1);
                assert!(issues.pretransaction().is_empty());
                assert!(!ledger.has_open_permits());
            }
            issues.cancel(&mut ledger).unwrap();
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert_no_current_or_permits(&ledger);
        }

        let mut owner = local_ledger("Doctor object issue owner");
        let mut foreign = local_ledger("Doctor object issue foreign");
        let mut issues = DoctorIssueCollection::new(&mut owner).unwrap();
        let owner_before = owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();
        assert!(issues
            .push_accounted_object_issue(
                &mut foreign,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
            )
            .is_err());
        assert_eq!(owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        issues.cancel(&mut owner).unwrap();
        assert_no_current_or_permits(&owner);
        assert_no_current_or_permits(&foreign);

        let mut ledger = local_ledger("Doctor object issue injected allocation failure");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let capacity_before = issues.pretransaction_issues.as_ref().unwrap().capacity();
        let snapshot_before = ledger.snapshot().clone();
        let error = issues
            .push_accounted_object_issue_with_hook(
                &mut ledger,
                DoctorObjectIssueSpec {
                    severity: super::super::DoctorSeverity::Error,
                    kind,
                    message: format_args!("{message}"),
                    path,
                },
                #[expect(
                    clippy::result_large_err,
                    reason = "the allocation fault fixture uses the Store callback error"
                )]
                || {
                    Err(codefire_store::StoreError::Io(std::io::Error::other(
                        "injected object diagnostic allocation failure",
                    )))
                },
            )
            .unwrap_err();
        assert!(
            matches!(error, codefire_store::StoreError::Io(error) if error.to_string().contains("injected"))
        );
        assert_eq!(
            issues.pretransaction_issues.as_ref().unwrap().capacity(),
            capacity_before
        );
        assert!(issues.pretransaction().is_empty());
        assert_eq!(ledger.snapshot(), &snapshot_before);
        assert!(!ledger.has_open_permits());
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_object_scan_keeps_diagnostic_order_and_continues_to_valid_records() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let commits = cf.join("objects/commits");
        let valid_commit = fs::read_dir(&commits)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.is_file() && path.extension().is_some_and(|value| value == "json"))
            .unwrap();
        let malformed = commits.join("000-malformed.json");
        let unreadable = commits.join("001-unreadable.json");
        let corrupt = commits.join("002-corrupt.json");
        fs::write(&malformed, b"{").unwrap();
        fs::create_dir(&unreadable).unwrap();
        let mut corrupt_record: Value =
            serde_json::from_slice(&fs::read(valid_commit).unwrap()).unwrap();
        corrupt_record["hash"] = Value::String("0".repeat(64));
        fs::write(&corrupt, serde_json::to_vec(&corrupt_record).unwrap()).unwrap();

        let checks = run_checks(&cf, false).unwrap();
        let fixture_paths = checks
            .issues
            .iter()
            .filter(|issue| issue.kind == "invalid_object_record")
            .filter_map(|issue| issue.path.as_ref())
            .filter(|path| *path == &malformed || *path == &unreadable || *path == &corrupt)
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(fixture_paths, vec![malformed, unreadable, corrupt]);
        assert!(
            checks.objects > 0,
            "valid records after corrupt entries must remain visible"
        );
        assert!(!checks.ledger.has_open_permits());
    }

    #[test]
    fn object_issue_rejects_stateful_display_growth_and_shrink_without_owner_drift() {
        use std::cell::Cell;

        struct ChangingDisplay {
            calls: Cell<u8>,
            grows: bool,
        }

        impl fmt::Display for ChangingDisplay {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let call = self.calls.get();
                self.calls.set(call + 1);
                let long = "message-that-is-longer-than-its-first-render";
                if self.grows == (call == 0) {
                    formatter.write_str("x")
                } else {
                    formatter.write_str(long)
                }
            }
        }

        for grows in [true, false] {
            let mut ledger = local_ledger(if grows {
                "Doctor object growing diagnostic"
            } else {
                "Doctor object shrinking diagnostic"
            });
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let snapshot_before = ledger.snapshot().clone();
            let capacity_before = issues.pretransaction_issues.as_ref().unwrap().capacity();
            let display = ChangingDisplay {
                calls: Cell::new(0),
                grows,
            };

            let error = issues
                .push_accounted_object_issue(
                    &mut ledger,
                    DoctorObjectIssueSpec {
                        severity: super::super::DoctorSeverity::Error,
                        kind: "invalid_object_record",
                        message: format_args!("{display}"),
                        path: Path::new("/repo/.codefire/objects/commits/invalid.json"),
                    },
                )
                .unwrap_err();

            assert!(matches!(
                error,
                codefire_store::StoreError::InvalidSealedCommit(message)
                    if message.contains("rendering")
            ));
            assert_eq!(display.calls.get(), 2);
            assert!(issues.pretransaction().is_empty());
            assert_eq!(
                issues.pretransaction_issues.as_ref().unwrap().capacity(),
                capacity_before
            );
            assert_eq!(ledger.snapshot(), &snapshot_before);
            assert!(!ledger.has_open_permits());
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    fn doctor_object_scan_rejects_foreign_issue_owner_before_scanning() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let scan_ledger = local_ledger("Doctor object scan owner");
        let mut validation = DoctorSealedValidation::new(&cf.join("objects"), scan_ledger).unwrap();
        let mut foreign_ledger = local_ledger("Doctor object scan foreign issue owner");
        let mut issues = DoctorIssueCollection::new(&mut foreign_ledger).unwrap();
        let scan_before = validation
            .session
            .as_ref()
            .unwrap()
            .decision_work_snapshot()
            .unwrap()
            .clone();
        let foreign_before = foreign_ledger.snapshot().clone();

        let error = validation.scan_object_records(&mut issues).unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(
            validation
                .session
                .as_ref()
                .unwrap()
                .decision_work_snapshot()
                .unwrap(),
            &scan_before
        );
        assert_eq!(foreign_ledger.snapshot(), &foreign_before);
        let scan_ledger = validation
            .abort_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
            .unwrap();
        assert_no_current_or_permits(&scan_ledger);
        issues.cancel(&mut foreign_ledger).unwrap();
        assert_no_current_or_permits(&foreign_ledger);
    }

    #[test]
    fn doctor_object_scan_cancellation_and_final_membership_race_release_report_ownership() {
        use std::cell::Cell;

        for final_race in [false, true] {
            let temp = tempdir().unwrap();
            let repo = temp.path().join("repo");
            crate::init_repo(&repo, false).unwrap();
            let cf = repo.join(".codefire");
            let commits = cf.join("objects/commits");
            fs::write(commits.join("000-malformed.json"), b"{").unwrap();
            let cancel = Cell::new(false);
            let injected = Cell::new(false);
            let mut ledger = local_ledger(if final_race {
                "Doctor object scan final race"
            } else {
                "Doctor object scan cancellation"
            });
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();
            let late_member = commits.join("late-member.tmp");

            let error = validation
                .scan_object_records_with_hooks(
                    &mut issues,
                    || cancel.get(),
                    || {
                        if !injected.replace(true) {
                            if final_race {
                                fs::write(&late_member, b"late").unwrap();
                            } else {
                                cancel.set(true);
                            }
                        }
                    },
                )
                .unwrap_err();
            let had_issue = !issues.pretransaction().is_empty();
            let expected_error = if final_race {
                matches!(
                    &error,
                    CliError::Store(error)
                        if matches!(&**error, codefire_store::StoreError::InvalidSealedCommit(message)
                            if message.contains("object store identity changed"))
                )
            } else {
                matches!(
                    &error,
                    CliError::Store(error)
                        if matches!(&**error, codefire_store::StoreError::ValidationCancelled)
                )
            };
            let reason = terminal_reason_for_cli_error(&error);
            let mut ledger = validation.abort_and_take_ledger(reason).unwrap();
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
            assert!(had_issue);
            assert!(expected_error, "unexpected scan failure: {error:?}");
        }
    }

    #[test]
    fn doctor_scan_propagates_budget_exhaustion_as_a_typed_fatal_error() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let objects = repo.join(".codefire/objects");
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::ObjectRecords)
            .unwrap()
            .limit;
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            profile,
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "doctor-budget-test",
        )
        .unwrap();
        ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::Validation,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .charge(codefire_core::DecisionWorkDimension::ObjectRecords, limit),
            )
            .unwrap();
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let ledger_identity = ledger.identity();
        let session =
            codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
                &objects, ledger,
            )
            .unwrap();
        let mut validation = DoctorSealedValidation {
            session: Some(session),
            detached_ledger: None,
            objects,
            begin_error: None,
            successful: Vec::new(),
            branch_snapshot: None,
            opened_snapshot: None,
            branch_success_retained_bytes: 0,
            finalize_failed: false,
            ledger_identity,
        };

        let error = validation.scan_object_records(&mut issues).unwrap_err();

        assert!(
            matches!(error, CliError::ResourceBudget(_)),
            "unexpected Doctor budget error: {error:?}"
        );
        let mut ledger = validation
            .abort_and_take_ledger(codefire_core::DecisionWorkTerminalReason::BudgetExceeded)
            .unwrap();
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_scan_and_candidates_share_one_accounted_recoverable_session() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let branch_path = cf.join("branches/main.json");
        let mut ledger = codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::local_v1(),
            codefire_core::DecisionWorkCommandKind::LocalGraph,
            "doctor test",
        )
        .unwrap();
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let ledger_identity = ledger.identity();
        let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();

        let checked_objects = validation.scan_object_records(&mut issues).unwrap();
        let after_scan = validation
            .session
            .as_ref()
            .unwrap()
            .decision_work_snapshot()
            .unwrap();
        assert!(checked_objects > 0);
        assert!(after_scan.counters[&codefire_core::DecisionWorkCounter::RecordReads] > 0);
        let reads_after_scan =
            after_scan.counters[&codefire_core::DecisionWorkCounter::RecordReads];
        let semantics_after_scan =
            after_scan.counters[&codefire_core::DecisionWorkCounter::SemanticValidations];

        let mut missing = read_doctor_json(&branch_path, ACTIVE_DATA_FILE_BYTE_LIMIT).unwrap();
        missing["head"] = Value::String("CF-COMMIT-missing".to_string());
        fs::write(
            cf.join("branches/missing.json"),
            serde_json::to_vec_pretty(&missing).unwrap(),
        )
        .unwrap();
        fs::copy(&branch_path, cf.join("branches/valid-copy.json")).unwrap();
        validation
            .capture_and_validate_branch_heads(&cf, &mut issues)
            .unwrap();
        let snapshot = validation
            .session
            .as_ref()
            .unwrap()
            .decision_work_snapshot()
            .unwrap();
        assert!(snapshot.counters[&codefire_core::DecisionWorkCounter::MemoHits] > 0);
        assert_eq!(
            snapshot.counters[&codefire_core::DecisionWorkCounter::RecordReads],
            reads_after_scan
        );
        assert_eq!(
            snapshot.counters[&codefire_core::DecisionWorkCounter::SemanticValidations],
            semantics_after_scan
        );

        let (checked_branches, mut ledger) = validation.finish(&mut issues).unwrap();
        assert_eq!(checked_branches, 2);
        assert!(!ledger.has_open_permits());
        assert_eq!(
            issues.residual_mut(ledger_identity).unwrap()[0].kind,
            "invalid_branch_head"
        );
        assert!(issues.residual_mut(ledger_identity).unwrap()[0]
            .message
            .contains("object not found"));
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_empty_branch_head_preserves_sealed_validation_diagnostic_path() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let branch_path = cf.join("branches/main.json");
        let mut branch = read_doctor_json(&branch_path, ACTIVE_DATA_FILE_BYTE_LIMIT).unwrap();
        branch["head"] = Value::String(String::new());
        fs::write(&branch_path, serde_json::to_vec_pretty(&branch).unwrap()).unwrap();

        let mut ledger = local_ledger("Doctor empty branch head");
        let ledger_identity = ledger.identity();
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();
        validation
            .capture_and_validate_branch_heads(&cf, &mut issues)
            .unwrap();

        let (checked_branches, mut ledger) = validation.finish(&mut issues).unwrap();
        assert_eq!(checked_branches, 0);
        let residual = issues.residual_mut(ledger_identity).unwrap();
        assert_eq!(residual.len(), 1);
        assert_eq!(residual[0].kind, "invalid_branch_head");
        assert!(!residual[0]
            .message
            .contains("must contain string field head"));
        issues.cancel(&mut ledger).unwrap();
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_branch_snapshot_abort_and_final_race_release_all_current_ownership() {
        for final_race in [false, true] {
            let temp = tempdir().unwrap();
            let repo = temp.path().join("repo");
            crate::init_repo(&repo, false).unwrap();
            let cf = repo.join(".codefire");
            let branch_path = cf.join("branches/main.json");
            let mut ledger = local_ledger(if final_race {
                "Doctor branch final race"
            } else {
                "Doctor branch explicit abort"
            });
            let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
            let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();
            validation
                .capture_and_validate_branch_heads(&cf, &mut issues)
                .unwrap();

            let mut ledger = if final_race {
                let original = fs::read(&branch_path).unwrap();
                fs::write(
                    &branch_path,
                    original.into_iter().chain(*b" ").collect::<Vec<_>>(),
                )
                .unwrap();
                assert!(validation.finish(&mut issues).is_err());
                validation
                    .abort_and_take_ledger(
                        codefire_core::DecisionWorkTerminalReason::SubjectChanged,
                    )
                    .unwrap()
            } else {
                validation
                    .abort_and_take_ledger(
                        codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    )
                    .unwrap()
            };
            issues.cancel(&mut ledger).unwrap();
            assert_no_current_or_permits(&ledger);
        }
    }

    #[test]
    #[expect(
        clippy::result_large_err,
        reason = "the injected parse cleanup preserves the StoreError contract"
    )]
    fn doctor_opened_parse_cleanup_abort_keeps_same_retry_owner() {
        let temp = tempdir().unwrap();
        let cf = temp.path().join("repo/.codefire");
        let opened = cf.join("opened");
        fs::create_dir_all(&opened).unwrap();
        fs::write(opened.join("main.json"), b"{").unwrap();
        let ledger = local_ledger("Doctor opened parse cleanup abort retry");
        let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();
        assert!(validation.session.is_none());
        let mut snapshot = validation
            .capture_opened_registry_snapshot(&cf, &opened)
            .unwrap();

        let outcome = snapshot
            .parse_candidate_with_release_hook(
                0,
                validation.detached_ledger.as_mut().unwrap(),
                |ledger, _| {
                    let retained = ledger.snapshot().current
                        [&codefire_core::DecisionWorkDimension::RetainedBytes];
                    ledger
                        .release(
                            codefire_core::DecisionWorkDimension::RetainedBytes,
                            retained,
                        )
                        .unwrap();
                    Err(codefire_store::StoreError::Io(std::io::Error::other(
                        "injected parse cleanup release failure",
                    )))
                },
            )
            .unwrap();
        assert!(matches!(
            outcome,
            super::super::opened_registry::DoctorOpenedRegistryParseResult::CleanupFailed(_)
        ));
        validation.attach_opened_registry_snapshot(snapshot);

        assert!(validation
            .abort_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
            .is_err());
        assert!(validation.opened_snapshot.is_some());
        assert!(validation.detached_ledger.is_some());
        let retained = validation
            .opened_snapshot
            .as_ref()
            .unwrap()
            .retained_bytes();
        validation
            .detached_ledger
            .as_mut()
            .unwrap()
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();

        let ledger = validation
            .abort_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
            .unwrap();
        assert!(validation.opened_snapshot.is_none());
        assert!(validation.detached_ledger.is_none());
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_opened_finish_cleanup_keeps_owner_for_abort_retry() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let opened = cf.join("opened");
        fs::write(opened.join("main.json"), b"{}").unwrap();
        let mut ledger = local_ledger("Doctor opened finish cleanup retry");
        let mut issues = DoctorIssueCollection::new(&mut ledger).unwrap();
        let mut validation = DoctorSealedValidation::new(&cf.join("objects"), ledger).unwrap();
        let mut snapshot = validation
            .capture_opened_registry_snapshot(&cf, &opened)
            .unwrap();
        assert!(matches!(
            validation
                .parse_opened_registry_candidate(&mut snapshot, 0)
                .unwrap(),
            super::super::opened_registry::DoctorOpenedRegistryParseResult::Parsed
        ));
        validation.attach_opened_registry_snapshot(snapshot);
        let retained = validation
            .opened_snapshot
            .as_ref()
            .unwrap()
            .retained_bytes();
        validation.session.as_mut().unwrap().finalize().unwrap();
        let ledger = validation
            .session
            .as_mut()
            .unwrap()
            .take_decision_work_ledger()
            .unwrap()
            .unwrap();
        validation.session = None;
        validation.detached_ledger = Some(ledger);
        validation
            .detached_ledger
            .as_mut()
            .unwrap()
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained,
            )
            .unwrap();

        assert!(validation.finish(&mut issues).is_err());
        assert!(validation.opened_snapshot.is_some());
        assert!(validation.detached_ledger.is_some());
        validation
            .detached_ledger
            .as_mut()
            .unwrap()
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::RecordRead,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    retained,
                ),
            )
            .unwrap();

        let mut ledger = validation
            .abort_and_take_ledger(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
            .unwrap();
        issues.cancel(&mut ledger).unwrap();
        assert!(validation.opened_snapshot.is_none());
        assert!(validation.detached_ledger.is_none());
        assert_no_current_or_permits(&ledger);
    }

    #[test]
    fn doctor_sealed_session_attributes_final_subject_change_to_valid_candidate() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let initialized = crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let objects = cf.join("objects");
        let commit_path =
            codefire_store::object_record_path(&objects, &initialized.main_commit).unwrap();

        let error = run_checks_with_sealed_validation_hook(&cf, true, || {
            let mut bytes = fs::read(&commit_path)?;
            bytes.push(b' ');
            fs::write(&commit_path, bytes)?;
            Ok(())
        })
        .unwrap_err();

        assert!(matches!(
            error,
            CliError::Store(error)
                if matches!(*error, codefire_store::StoreError::SealedValidationSubjectChanged { .. })
        ));
    }
}
