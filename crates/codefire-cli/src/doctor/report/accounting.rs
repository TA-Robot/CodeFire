use super::*;
use crate::automation::command_result_envelope;
use crate::CliError;
use serde_json::Value;
use std::fmt;
use std::io::{self, Write};
#[cfg(test)]
use std::ops::Deref;
use std::path::Path;

const REPORT_OWNER_FRAME_BYTES: u64 = codefire_core::LOGICAL_FRAME_BASE_BYTES;

/// Owns a completed Doctor report and the same command ledger used by Store
/// validation. Rendering is deliberately part of this owner: callers cannot
/// detach the report payload from its retained-memory accounting.
#[must_use = "a Doctor report must be rendered or explicitly dropped"]
pub(crate) struct AccountedDoctorReport {
    report: Option<DoctorReport>,
    ledger: Option<codefire_core::DecisionWorkLedger>,
    report_bytes: u64,
}

#[cfg(test)]
impl Deref for AccountedDoctorReport {
    type Target = DoctorReport;

    fn deref(&self) -> &Self::Target {
        self.report
            .as_ref()
            .expect("an active Doctor report retains its payload")
    }
}

impl AccountedDoctorReport {
    #[cfg(test)]
    pub(super) fn new(
        report: DoctorReport,
        ledger: codefire_core::DecisionWorkLedger,
    ) -> Result<Self, CliError> {
        let report_accounting = DoctorReportAccountingTransfer::unowned(&ledger);
        Self::new_with_preaccounted(report, ledger, report_accounting)
    }

    pub(super) fn new_with_preaccounted(
        report: DoctorReport,
        mut ledger: codefire_core::DecisionWorkLedger,
        report_accounting: DoctorReportAccountingTransfer,
    ) -> Result<Self, CliError> {
        let preaccounted_report_bytes = match report_accounting.retained_bytes_for(&ledger) {
            Ok(bytes) => bytes,
            Err(error) => {
                drop(report);
                return Err(error);
            }
        };
        let report_bytes = match doctor_report_owned_bytes(&report) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(cancel_failed_report_construction(
                    report,
                    ledger,
                    preaccounted_report_bytes,
                    error,
                ));
            }
        };
        let retained_current = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
            .copied()
            .unwrap_or_default();
        if retained_current != preaccounted_report_bytes {
            drop(report);
            return Err(CliError::InvalidRepository(
                "Doctor preaccounted report ownership does not match the transferred ledger"
                    .to_string(),
            ));
        }
        let newly_retained = match report_bytes.checked_sub(preaccounted_report_bytes) {
            Some(bytes) => bytes,
            None => {
                let error = CliError::InvalidRepository(
                    "Doctor preaccounted report ownership exceeds the final report".to_string(),
                );
                return Err(cancel_failed_report_construction(
                    report,
                    ledger,
                    preaccounted_report_bytes,
                    error,
                ));
            }
        };
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                newly_retained,
            ),
        ) {
            let error = doctor_report_accounting_error(error, "retain Doctor report");
            return Err(cancel_failed_report_construction(
                report,
                ledger,
                preaccounted_report_bytes,
                error,
            ));
        }
        Ok(Self {
            report: Some(report),
            ledger: Some(ledger),
            report_bytes,
        })
    }

    pub(crate) fn is_ok(&self) -> bool {
        self.report
            .as_ref()
            .expect("an active Doctor report retains its payload")
            .ok
    }

    pub(crate) fn write_json<W: Write>(mut self, writer: &mut W) -> Result<(), CliError> {
        self.write_json_inner(writer)
    }

    fn write_json_inner<W: Write>(&mut self, writer: &mut W) -> Result<(), CliError> {
        let exit_code = if self.is_ok() {
            crate::exit_code::ExitCode::Success.code()
        } else {
            crate::exit_code::ExitCode::RepositoryCorruption.code()
        };
        let (envelope, envelope_bytes) = self.prepare_json_projection(exit_code)?;
        let encoded_result = {
            let ledger = self
                .ledger
                .as_mut()
                .expect("Doctor report rendering retains its ledger");
            encode_pretty_json_accounted(&envelope, ledger)
        };
        let (encoded, encoded_bytes) = match encoded_result {
            Ok(encoded) => encoded,
            Err(error) => {
                drop(envelope);
                self.release_retained(
                    envelope_bytes,
                    "release Doctor JSON projection after encoding failure",
                )?;
                return Err(error);
            }
        };
        drop(envelope);
        self.release_retained(envelope_bytes, "release Doctor JSON projection")?;
        self.release_report()?;
        let write_result = writer.write_all(&encoded).map_err(CliError::from);
        drop(encoded);
        self.release_retained(encoded_bytes, "release Doctor JSON output")?;
        self.finish(if write_result.is_ok() {
            codefire_core::DecisionWorkTerminalReason::Complete
        } else {
            codefire_core::DecisionWorkTerminalReason::CorruptSubject
        })?;
        write_result
    }

    pub(crate) fn write_text<W: Write>(mut self, writer: &mut W) -> Result<(), CliError> {
        self.write_text_inner(writer)
    }

    #[allow(
        dead_code,
        reason = "consuming cancellation is part of the Doctor owner safety contract"
    )]
    pub(crate) fn cancel(mut self) -> Result<(), CliError> {
        self.cancel_inner()
    }

    fn write_text_inner<W: Write>(&mut self, writer: &mut W) -> Result<(), CliError> {
        let (text, text_bytes) = {
            let report = self
                .report
                .as_ref()
                .expect("Doctor text rendering retains its report");
            let ledger = self
                .ledger
                .as_mut()
                .expect("Doctor text rendering retains its ledger");
            render_doctor_text_accounted(report, ledger)?
        };
        self.release_report()?;
        let write_result = writer.write_all(text.as_bytes()).map_err(CliError::from);
        drop(text);
        self.release_retained(text_bytes, "release Doctor text output")?;
        self.finish(if write_result.is_ok() {
            codefire_core::DecisionWorkTerminalReason::Complete
        } else {
            codefire_core::DecisionWorkTerminalReason::CorruptSubject
        })?;
        write_result
    }

    #[cfg(test)]
    fn write_json_with_snapshot<W: Write>(
        mut self,
        writer: &mut W,
    ) -> (
        Result<(), CliError>,
        codefire_core::DecisionWorkSnapshot,
        usize,
    ) {
        let result = self.write_json_inner(writer);
        let ledger = self
            .ledger
            .as_ref()
            .expect("test Doctor owner retains its ledger");
        let snapshot = ledger.snapshot().clone();
        (result, snapshot, ledger.open_permit_count())
    }

    #[cfg(test)]
    fn write_text_with_snapshot<W: Write>(
        mut self,
        writer: &mut W,
    ) -> (
        Result<(), CliError>,
        codefire_core::DecisionWorkSnapshot,
        usize,
    ) {
        let result = self.write_text_inner(writer);
        let ledger = self
            .ledger
            .as_ref()
            .expect("test Doctor owner retains its ledger");
        let snapshot = ledger.snapshot().clone();
        (result, snapshot, ledger.open_permit_count())
    }

    #[cfg(test)]
    fn cancel_with_snapshot(mut self) -> (codefire_core::DecisionWorkSnapshot, usize) {
        self.cancel_inner().unwrap();
        let ledger = self.ledger.as_ref().unwrap();
        (ledger.snapshot().clone(), ledger.open_permit_count())
    }

    fn cancel_inner(&mut self) -> Result<(), CliError> {
        self.release_report()?;
        self.finish(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
    }

    fn prepare_json_projection(&mut self, exit_code: i32) -> Result<(Value, u64), CliError> {
        let report = self
            .report
            .as_ref()
            .expect("Doctor JSON projection retains its report");
        let ledger = self
            .ledger
            .as_mut()
            .expect("Doctor JSON projection retains its ledger");
        let temporary_limit = ledger
            .profile()
            .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor report profile has no temporary-byte limit".to_string(),
                )
            })?
            .limit;
        let temporary_current = ledger
            .snapshot()
            .current
            .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
            .copied()
            .unwrap_or_default();
        let upper = temporary_limit
            .checked_sub(temporary_current)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "Doctor report temporary ownership exceeds its profile".to_string(),
                )
            })?;
        let mut permit = ledger
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, upper),
            )
            .map_err(|error| {
                doctor_report_accounting_error(error, "reserve Doctor JSON projection")
            })?;

        // Every String/Path/Value/Vec allocated by these existing projections
        // is created only while the request ledger owns the complete remaining
        // Temporary headroom. The compact result is then transferred to exact
        // Retained ownership before that scratch reservation is released.
        let envelope = command_result_envelope(
            "doctor",
            report.ok,
            exit_code,
            Some(&report.repo_root),
            doctor_report_data_json(report),
            doctor_report_diagnostics_json(report),
            doctor_report_next_actions(report),
        );
        let actual = match compact_json_size(&envelope) {
            Ok(actual) if actual <= upper => actual,
            Ok(_) => {
                drop(envelope);
                ledger.cancel_reserved(&permit).map_err(|error| {
                    doctor_report_accounting_error(error, "cancel oversized Doctor JSON projection")
                })?;
                return Err(force_temporary_budget_failure(ledger));
            }
            Err(error) => {
                drop(envelope);
                ledger.cancel_reserved(&permit).map_err(|cleanup| {
                    doctor_report_accounting_error(cleanup, "cancel failed Doctor JSON projection")
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
            .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, actual),
        ) {
            drop(envelope);
            ledger.cancel_reserved(&permit).map_err(|cleanup| {
                doctor_report_accounting_error(
                    cleanup,
                    "cancel rejected Doctor JSON materialization",
                )
            })?;
            return Err(doctor_report_accounting_error(
                error,
                "materialize Doctor JSON projection",
            ));
        }
        if actual < upper {
            ledger.cancel_reserved(&permit).map_err(|error| {
                doctor_report_accounting_error(error, "release unused Doctor JSON reservation")
            })?;
        }
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, actual),
        ) {
            drop(envelope);
            ledger
                .release(codefire_core::DecisionWorkDimension::TemporaryBytes, actual)
                .map_err(|cleanup| {
                    doctor_report_accounting_error(
                        cleanup,
                        "release rejected Doctor JSON projection",
                    )
                })?;
            return Err(doctor_report_accounting_error(
                error,
                "retain Doctor JSON projection",
            ));
        }
        ledger
            .release(codefire_core::DecisionWorkDimension::TemporaryBytes, actual)
            .map_err(|error| {
                doctor_report_accounting_error(error, "transfer Doctor JSON projection")
            })?;
        Ok((envelope, actual))
    }

    fn release_report(&mut self) -> Result<(), CliError> {
        if self.report_bytes == 0 {
            self.report = None;
            return Ok(());
        }
        drop(self.report.take());
        let amount = self.report_bytes;
        self.report_bytes = 0;
        self.release_retained(amount, "release Doctor report payload")
    }

    fn release_retained(&mut self, amount: u64, operation: &str) -> Result<(), CliError> {
        self.ledger
            .as_mut()
            .expect("Doctor report owner retains its ledger")
            .release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)
            .map_err(|error| doctor_report_accounting_error(error, operation))
    }

    fn finish(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) -> Result<(), CliError> {
        let ledger = self
            .ledger
            .as_mut()
            .expect("Doctor report owner retains its ledger until completion");
        if ledger
            .snapshot()
            .current
            .values()
            .any(|amount| *amount != 0)
            || ledger.has_open_permits()
        {
            return Err(CliError::InvalidRepository(
                "Doctor report completed with live decision-work ownership".to_string(),
            ));
        }
        if ledger.snapshot().terminal_reason.is_none() {
            ledger
                .finish(reason)
                .map_err(|error| doctor_report_accounting_error(error, "finish Doctor report"))?;
        }
        Ok(())
    }
}

fn cancel_failed_report_construction(
    report: DoctorReport,
    mut ledger: codefire_core::DecisionWorkLedger,
    preaccounted_report_bytes: u64,
    original: CliError,
) -> CliError {
    drop(report);
    let retained = ledger
        .snapshot()
        .current
        .get(&codefire_core::DecisionWorkDimension::RetainedBytes)
        .copied()
        .unwrap_or_default();
    let other_current = ledger.snapshot().current.iter().any(|(dimension, amount)| {
        *dimension != codefire_core::DecisionWorkDimension::RetainedBytes && *amount != 0
    });
    if retained != preaccounted_report_bytes || other_current || ledger.has_open_permits() {
        return CliError::InvalidRepository(
            "failed Doctor report construction retained unexpected decision work".to_string(),
        );
    }
    if preaccounted_report_bytes > 0 {
        if let Err(error) = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            preaccounted_report_bytes,
        ) {
            return doctor_report_accounting_error(error, "cancel failed Doctor report");
        }
    }
    if ledger.snapshot().terminal_reason.is_none() {
        let reason = if matches!(original, CliError::ResourceBudget(_)) {
            codefire_core::DecisionWorkTerminalReason::BudgetExceeded
        } else {
            codefire_core::DecisionWorkTerminalReason::CorruptSubject
        };
        if let Err(error) = ledger.finish(reason) {
            return doctor_report_accounting_error(error, "finish failed Doctor report");
        }
    }
    original
}

impl Drop for AccountedDoctorReport {
    fn drop(&mut self) {
        let cleanup = (|| -> Result<(), CliError> {
            self.release_report()?;
            if let Some(ledger) = self.ledger.as_mut() {
                if ledger
                    .snapshot()
                    .current
                    .values()
                    .any(|amount| *amount != 0)
                    || ledger.has_open_permits()
                {
                    return Err(CliError::InvalidRepository(
                        "Doctor report drop found unreturned ownership".to_string(),
                    ));
                }
                if ledger.snapshot().terminal_reason.is_none() && !ledger.has_open_permits() {
                    ledger
                        .finish(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
                        .map_err(|error| {
                            doctor_report_accounting_error(error, "drop Doctor report ledger")
                        })?;
                }
            }
            Ok(())
        })();
        debug_assert!(
            cleanup.is_ok(),
            "Doctor report drop cleanup failed: {cleanup:?}"
        );
    }
}

fn doctor_report_accounting_error(
    error: codefire_core::DecisionWorkAccountingError,
    operation: &str,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "Doctor report accounting failed while {operation}: {error}"
        )),
    }
}

fn force_temporary_budget_failure(ledger: &mut codefire_core::DecisionWorkLedger) -> CliError {
    let limit = ledger
        .profile()
        .limit(codefire_core::DecisionWorkDimension::TemporaryBytes)
        .expect("Doctor report profile declares TemporaryBytes")
        .limit;
    let current = ledger
        .snapshot()
        .current
        .get(&codefire_core::DecisionWorkDimension::TemporaryBytes)
        .copied()
        .unwrap_or_default();
    let attempted = limit.saturating_sub(current).saturating_add(1);
    let error = ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::TemporaryBytes,
                attempted,
            ),
        )
        .expect_err("limit+1 Doctor projection must fail");
    doctor_report_accounting_error(error, "bound Doctor JSON projection")
}

fn checked_owned_add(total: &mut u64, amount: usize) -> Result<(), CliError> {
    let amount = u64::try_from(amount)
        .map_err(|_| CliError::InvalidRepository("Doctor report size overflow".to_string()))?;
    *total = total
        .checked_add(REPORT_OWNER_FRAME_BYTES)
        .and_then(|value| value.checked_add(amount))
        .ok_or_else(|| CliError::InvalidRepository("Doctor report size overflow".to_string()))?;
    Ok(())
}

fn add_owned_path(total: &mut u64, path: &Path) -> Result<(), CliError> {
    checked_owned_add(total, path.as_os_str().as_encoded_bytes().len())
}

/// Portable logical ownership for the already-collected report payload.
/// Each live String/Path/Vec owner contributes one standard frame plus its
/// encoded content. Output Values and byte buffers are accounted separately
/// during projection and rendering.
fn doctor_report_owned_bytes(report: &DoctorReport) -> Result<u64, CliError> {
    let mut total = REPORT_OWNER_FRAME_BYTES;
    add_owned_path(&mut total, &report.repo_root)?;
    checked_owned_add(&mut total, report.skipped_checks.len())?;
    for skipped in &report.skipped_checks {
        checked_owned_add(&mut total, skipped.len())?;
    }
    checked_owned_add(&mut total, report.issues.len())?;
    for issue in &report.issues {
        checked_owned_add(&mut total, issue.kind.len())?;
        checked_owned_add(&mut total, issue.message.len())?;
        if let Some(path) = issue.path.as_deref() {
            add_owned_path(&mut total, path)?;
        }
        if let Some(divergence) = issue.open_snapshot.as_ref() {
            checked_owned_add(&mut total, divergence.phase.len())?;
            checked_owned_add(&mut total, divergence.expected.len())?;
            checked_owned_add(&mut total, divergence.actual.len())?;
            add_owned_path(&mut total, &divergence.expected_record)?;
            add_owned_path(&mut total, &divergence.actual_record)?;
            add_owned_path(&mut total, &divergence.repo_root)?;
            checked_owned_add(&mut total, divergence.branch.len())?;
        }
        if let Some(pending) = issue.pending_authority.as_ref() {
            checked_owned_add(&mut total, pending.phase.len())?;
            if let Some(intended) = pending.intended_state.as_ref() {
                checked_owned_add(&mut total, intended.len())?;
            }
            match &pending.request_identity {
                DoctorPendingRequestIdentity::TransactionId { transaction_id } => {
                    checked_owned_add(&mut total, transaction_id.len())?;
                }
                DoctorPendingRequestIdentity::RequestHash { request_hash } => {
                    checked_owned_add(&mut total, request_hash.len())?;
                }
                DoctorPendingRequestIdentity::StateSubject {
                    branch,
                    open_instance_id,
                    current_base_commit,
                } => {
                    checked_owned_add(&mut total, branch.len())?;
                    checked_owned_add(&mut total, open_instance_id.len())?;
                    checked_owned_add(&mut total, current_base_commit.len())?;
                }
                DoctorPendingRequestIdentity::LegacyUnbound
                | DoctorPendingRequestIdentity::Withheld { .. } => {}
            }
        }
    }
    Ok(total)
}

#[derive(Default)]
struct JsonLengthWriter {
    bytes: u64,
    overflowed: bool,
}

impl Write for JsonLengthWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| io::Error::other("Doctor JSON size overflow"))?;
        self.bytes = match self.bytes.checked_add(amount) {
            Some(total) => total,
            None => {
                self.overflowed = true;
                return Err(io::Error::other("Doctor JSON size overflow"));
            }
        };
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn compact_json_size<T: serde::Serialize + ?Sized>(value: &T) -> Result<u64, CliError> {
    let mut writer = JsonLengthWriter::default();
    match serde_json::to_writer(&mut writer, value) {
        Ok(()) => Ok(writer.bytes),
        Err(_) if writer.overflowed => Err(CliError::InvalidRepository(
            "Doctor JSON size overflow".to_string(),
        )),
        Err(error) => Err(CliError::from(error)),
    }
}

struct AccountedJsonWriter<'a> {
    body: Vec<u8>,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
    charge_response_bytes: bool,
}

impl Write for AccountedJsonWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| io::Error::other("Doctor JSON output size overflow"))?;
        let phase = if self.charge_response_bytes {
            codefire_core::DecisionWorkPhase::Encoding
        } else {
            codefire_core::DecisionWorkPhase::PublicationPreflight
        };
        let mut reservation = codefire_core::DecisionWorkReservation::new(
            phase,
            codefire_core::DecisionWorkSurface::Atomic,
        )
        .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount)
        .count(
            codefire_core::DecisionWorkCounter::ResponseBytesEncoded,
            amount,
        );
        if self.charge_response_bytes {
            reservation =
                reservation.charge(codefire_core::DecisionWorkDimension::ResponseBytes, amount);
        }
        if let Err(error) = self.ledger.apply(reservation) {
            self.accounting_error = Some(error);
            return Err(io::Error::other("Doctor JSON output budget exceeded"));
        }
        self.body.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_pretty_json_accounted<T: serde::Serialize + ?Sized>(
    value: &T,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<u8>, u64), CliError> {
    let charge_response_bytes = ledger
        .profile()
        .limit(codefire_core::DecisionWorkDimension::ResponseBytes)
        .is_some();
    let (serialized, accounting_error, body) = {
        let mut writer = AccountedJsonWriter {
            body: Vec::new(),
            ledger,
            accounting_error: None,
            charge_response_bytes,
        };
        let serialized = serde_json::to_writer_pretty(&mut writer, value);
        if serialized.is_ok() {
            let _ = writer.write_all(b"\n");
        }
        (serialized, writer.accounting_error.take(), writer.body)
    };
    let body_bytes = u64::try_from(body.len())
        .map_err(|_| CliError::InvalidRepository("Doctor JSON output size overflow".to_string()))?;
    if let Some(error) = accounting_error {
        drop(body);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                body_bytes,
            )
            .map_err(|cleanup| {
                doctor_report_accounting_error(cleanup, "release partial Doctor JSON output")
            })?;
        return Err(doctor_report_accounting_error(
            error,
            "encode Doctor JSON output",
        ));
    }
    if let Err(error) = serialized {
        drop(body);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                body_bytes,
            )
            .map_err(|cleanup| {
                doctor_report_accounting_error(cleanup, "release failed Doctor JSON output")
            })?;
        return Err(CliError::from(error));
    }
    Ok((body, body_bytes))
}

struct AccountedTextWriter<'a> {
    text: String,
    ledger: &'a mut codefire_core::DecisionWorkLedger,
    accounting_error: Option<codefire_core::DecisionWorkAccountingError>,
}

impl fmt::Write for AccountedTextWriter<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let amount = u64::try_from(value.len()).map_err(|_| fmt::Error)?;
        if let Err(error) = self.ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, amount),
        ) {
            self.accounting_error = Some(error);
            return Err(fmt::Error);
        }
        self.text.push_str(value);
        Ok(())
    }
}

fn render_doctor_text_accounted(
    report: &DoctorReport,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(String, u64), CliError> {
    let (rendered, accounting_error, text) = {
        let mut output = AccountedTextWriter {
            text: String::new(),
            ledger,
            accounting_error: None,
        };
        let rendered = write_doctor_text(report, &mut output);
        (rendered, output.accounting_error.take(), output.text)
    };
    let text_bytes = u64::try_from(text.len())
        .map_err(|_| CliError::InvalidRepository("Doctor text size overflow".to_string()))?;
    if let Some(error) = accounting_error {
        drop(text);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                text_bytes,
            )
            .map_err(|cleanup| {
                doctor_report_accounting_error(cleanup, "release partial Doctor text")
            })?;
        return Err(doctor_report_accounting_error(error, "render Doctor text"));
    }
    if rendered.is_err() {
        drop(text);
        ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                text_bytes,
            )
            .map_err(|cleanup| {
                doctor_report_accounting_error(cleanup, "release failed Doctor text")
            })?;
        return Err(CliError::InvalidRepository(
            "Doctor text rendering failed".to_string(),
        ));
    }
    Ok((text, text_bytes))
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use serde::ser::{Error as _, SerializeMap};
    use serde::Serialize;
    use std::path::PathBuf;

    fn ledger(profile: codefire_core::DecisionWorkProfile) -> codefire_core::DecisionWorkLedger {
        let command_kind = if profile
            .applicable_commands()
            .contains(&codefire_core::DecisionWorkCommandKind::HttpRequest)
        {
            codefire_core::DecisionWorkCommandKind::HttpRequest
        } else {
            codefire_core::DecisionWorkCommandKind::LocalGraph
        };
        codefire_core::DecisionWorkLedger::new(profile, command_kind, "doctor report test").unwrap()
    }

    fn sample_report(issue: Option<DoctorIssue>) -> DoctorReport {
        DoctorReport {
            repo_root: PathBuf::from("/repo"),
            ok: issue.is_none(),
            checked_objects: 3,
            checked_branches: 1,
            checked_opened: 0,
            checked_active_files: 2,
            quick: false,
            skipped_checks: Vec::new(),
            issues: issue.into_iter().collect(),
        }
    }

    fn assert_clean(snapshot: &codefire_core::DecisionWorkSnapshot, permits: usize) {
        assert!(snapshot.current.values().all(|amount| *amount == 0));
        assert_eq!(permits, 0);
    }

    #[test]
    fn doctor_json_owner_preserves_existing_envelope_bytes_and_cleans_every_owner() {
        let report = sample_report(Some(DoctorIssue::error(
            "invalid_branch_record",
            "invalid branch",
            Some(PathBuf::from("/repo/.codefire/branches/main.json")),
        )));
        let expected = format!(
            "{}\n",
            serde_json::to_string_pretty(&command_result_envelope(
                "doctor",
                false,
                20,
                Some(&report.repo_root),
                doctor_report_data_json(&report),
                doctor_report_diagnostics_json(&report),
                doctor_report_next_actions(&report),
            ))
            .unwrap()
        );
        let owner = AccountedDoctorReport::new(
            report,
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let mut output = Vec::new();

        let (result, snapshot, permits) = owner.write_json_with_snapshot(&mut output);

        result.unwrap();
        assert_eq!(output, expected.as_bytes());
        assert_clean(&snapshot, permits);
        assert_eq!(
            snapshot.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::Complete)
        );
    }

    #[test]
    fn huge_issue_path_and_action_remain_owned_through_json_handoff() {
        let huge = "x".repeat(256 * 1024);
        let report = sample_report(Some(DoctorIssue::error(
            "legacy_open_base_semantic_profile",
            huge.clone(),
            Some(PathBuf::from(format!("/repo/{huge}"))),
        )));
        let owner = AccountedDoctorReport::new(
            report,
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let mut output = Vec::new();

        let (result, snapshot, permits) = owner.write_json_with_snapshot(&mut output);

        result.unwrap();
        assert!(output.len() > huge.len() * 4);
        assert!(String::from_utf8(output)
            .unwrap()
            .contains("scan_legacy_open_base"));
        assert_clean(&snapshot, permits);
    }

    #[test]
    fn doctor_response_writer_honors_limit_minus_one_exact_and_plus_one() {
        let profile = codefire_core::DecisionWorkProfile::http_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        let boundary = 4096_u64;
        for (amount, succeeds) in [
            (boundary - 1, true),
            (boundary, true),
            (boundary + 1, false),
        ] {
            let mut ledger = ledger(profile.clone());
            let baseline = limit - boundary;
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
            let mut writer = AccountedJsonWriter {
                body: Vec::new(),
                ledger: &mut ledger,
                accounting_error: None,
                charge_response_bytes: true,
            };
            let bytes = vec![b'x'; usize::try_from(amount).unwrap()];
            let result = writer.write_all(&bytes);
            let error = writer.accounting_error.take();
            let body = writer.body;

            assert_eq!(result.is_ok(), succeeds);
            if succeeds {
                assert_eq!(body.len(), usize::try_from(amount).unwrap());
                drop(body);
                ledger
                    .release(codefire_core::DecisionWorkDimension::RetainedBytes, amount)
                    .unwrap();
            } else {
                assert!(body.is_empty());
                let error = error.unwrap();
                let exhaustion = error.budget_exceeded().unwrap();
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, limit + 1);
                assert_eq!(exhaustion.exit_code, 60);
            }
            ledger
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            assert!(ledger
                .snapshot()
                .current
                .values()
                .all(|amount| *amount == 0));
            assert!(!ledger.has_open_permits());
        }
    }

    #[test]
    fn doctor_projection_retained_limit_minus_one_exact_and_plus_one_is_atomic() {
        let profile = codefire_core::DecisionWorkProfile::local_v1();
        let limit = profile
            .limit(codefire_core::DecisionWorkDimension::RetainedBytes)
            .unwrap()
            .limit;
        for peak_delta in [-1_i64, 0, 1] {
            let mut owner =
                AccountedDoctorReport::new(sample_report(None), ledger(profile.clone())).unwrap();
            let expected_projection = command_result_envelope(
                "doctor",
                true,
                0,
                Some(&owner.repo_root),
                doctor_report_data_json(&owner),
                doctor_report_diagnostics_json(&owner),
                doctor_report_next_actions(&owner),
            );
            let projection_bytes = compact_json_size(&expected_projection).unwrap();
            drop(expected_projection);
            let base_peak = owner.report_bytes + projection_bytes;
            let baseline = match peak_delta {
                -1 => limit - base_peak - 1,
                0 => limit - base_peak,
                1 => limit - base_peak + 1,
                _ => unreachable!(),
            };
            owner
                .ledger
                .as_mut()
                .unwrap()
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

            let projected = owner.prepare_json_projection(0);

            if peak_delta <= 0 {
                let (value, actual) = projected.unwrap();
                assert_eq!(actual, projection_bytes);
                let snapshot = owner.ledger.as_ref().unwrap().snapshot();
                assert_eq!(
                    snapshot.current[&codefire_core::DecisionWorkDimension::RetainedBytes],
                    if peak_delta == 0 { limit } else { limit - 1 }
                );
                drop(value);
                owner
                    .release_retained(actual, "release test Doctor projection")
                    .unwrap();
            } else {
                let error = projected.unwrap_err();
                let CliError::ResourceBudget(exhaustion) = error else {
                    panic!("unexpected Doctor projection error: {error:?}");
                };
                assert_eq!(
                    exhaustion.dimension,
                    codefire_core::DecisionWorkDimension::RetainedBytes
                );
                assert_eq!(exhaustion.observed, limit + 1);
                assert_eq!(exhaustion.exit_code, 60);
                assert_eq!(owner.ledger.as_ref().unwrap().open_permit_count(), 0);
                assert_eq!(
                    owner.ledger.as_ref().unwrap().snapshot().current
                        [&codefire_core::DecisionWorkDimension::TemporaryBytes],
                    0
                );
            }
            owner
                .ledger
                .as_mut()
                .unwrap()
                .release(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    baseline,
                )
                .unwrap();
            let (snapshot, permits) = owner.cancel_with_snapshot();
            assert_clean(&snapshot, permits);
        }
    }

    struct FaultingJson;

    impl Serialize for FaultingJson {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("partial", "payload")?;
            Err(S::Error::custom("injected Doctor JSON encoding failure"))
        }
    }

    #[test]
    fn doctor_json_encoding_fault_drops_partial_body_before_release() {
        let mut ledger = ledger(codefire_core::DecisionWorkProfile::http_v1());

        let error = encode_pretty_json_accounted(&FaultingJson, &mut ledger).unwrap_err();

        assert!(matches!(error, CliError::Json(_)));
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        assert!(!ledger.has_open_permits());
    }

    struct BaseWriteError;

    impl Write for BaseWriteError {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("injected base writer failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn doctor_wire_handoff_base_error_cleans_payload_and_ledger() {
        let owner = AccountedDoctorReport::new(
            sample_report(None),
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();

        let (result, snapshot, permits) = owner.write_json_with_snapshot(&mut BaseWriteError);

        assert!(matches!(result, Err(CliError::Io(_))));
        assert_clean(&snapshot, permits);
        assert_eq!(
            snapshot.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[derive(Default)]
    struct PartialWriteError {
        accepted: usize,
    }

    impl Write for PartialWriteError {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.accepted == 0 {
                let accepted = bytes.len().min(7);
                self.accepted = accepted;
                Ok(accepted)
            } else {
                Err(io::Error::other("injected partial writer failure"))
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn doctor_partial_json_and_text_handoffs_are_never_complete() {
        let json_owner = AccountedDoctorReport::new(
            sample_report(None),
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let mut json_writer = PartialWriteError::default();

        let (json_result, json_snapshot, json_permits) =
            json_owner.write_json_with_snapshot(&mut json_writer);

        assert!(matches!(json_result, Err(CliError::Io(_))));
        assert!(json_writer.accepted > 0);
        assert_clean(&json_snapshot, json_permits);
        assert_eq!(
            json_snapshot.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );

        let text_owner = AccountedDoctorReport::new(
            sample_report(None),
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let mut text_writer = PartialWriteError::default();

        let (text_result, text_snapshot, text_permits) =
            text_owner.write_text_with_snapshot(&mut text_writer);

        assert!(matches!(text_result, Err(CliError::Io(_))));
        assert!(text_writer.accepted > 0);
        assert_clean(&text_snapshot, text_permits);
        assert_eq!(
            text_snapshot.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[test]
    fn doctor_owner_api_keeps_scalar_query_and_consuming_operations() {
        let _is_ok: fn(&AccountedDoctorReport) -> bool = AccountedDoctorReport::is_ok;
        let _write_json: fn(AccountedDoctorReport, &mut Vec<u8>) -> Result<(), CliError> =
            AccountedDoctorReport::write_json::<Vec<u8>>;
        let _write_text: fn(AccountedDoctorReport, &mut Vec<u8>) -> Result<(), CliError> =
            AccountedDoctorReport::write_text::<Vec<u8>>;
        let _cancel: fn(AccountedDoctorReport) -> Result<(), CliError> =
            AccountedDoctorReport::cancel;

        let owner = AccountedDoctorReport::new(
            sample_report(None),
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        assert!(owner.is_ok());
        owner.cancel().unwrap();
    }

    #[test]
    fn doctor_report_accepts_exact_preaccounted_transfer_without_double_charge() {
        let report = sample_report(None);
        let total = doctor_report_owned_bytes(&report).unwrap();
        let transferred = REPORT_OWNER_FRAME_BYTES;
        let mut work_ledger = ledger(codefire_core::DecisionWorkProfile::local_v1());
        work_ledger
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    transferred,
                ),
            )
            .unwrap();
        let report_accounting =
            DoctorReportAccountingTransfer::new(work_ledger.identity(), transferred);

        let owner =
            AccountedDoctorReport::new_with_preaccounted(report, work_ledger, report_accounting)
                .unwrap();

        assert_eq!(
            owner.ledger.as_ref().unwrap().snapshot().current
                [&codefire_core::DecisionWorkDimension::RetainedBytes],
            total
        );
        let (snapshot, permits) = owner.cancel_with_snapshot();
        assert_clean(&snapshot, permits);

        let unmatched = ledger(codefire_core::DecisionWorkProfile::local_v1());
        let unmatched_transfer =
            DoctorReportAccountingTransfer::new(unmatched.identity(), transferred);
        let error = match AccountedDoctorReport::new_with_preaccounted(
            sample_report(None),
            unmatched,
            unmatched_transfer,
        ) {
            Ok(owner) => {
                owner.cancel().unwrap();
                panic!("unmatched Doctor preaccounted transfer must fail")
            }
            Err(error) => error,
        };
        assert!(matches!(error, CliError::InvalidRepository(_)));

        let mut transfer_owner = ledger(codefire_core::DecisionWorkProfile::local_v1());
        transfer_owner
            .apply(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(
                    codefire_core::DecisionWorkDimension::RetainedBytes,
                    transferred,
                ),
            )
            .unwrap();
        let foreign = ledger(codefire_core::DecisionWorkProfile::local_v1());
        let transfer = DoctorReportAccountingTransfer::new(transfer_owner.identity(), transferred);
        let owner_before = transfer_owner.snapshot().clone();
        let foreign_before = foreign.snapshot().clone();

        let error = transfer.retained_bytes_for(&foreign).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(transfer_owner.snapshot(), &owner_before);
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(transfer_owner.open_permit_count(), 0);
        assert_eq!(foreign.open_permit_count(), 0);

        let error = match AccountedDoctorReport::new_with_preaccounted(
            sample_report(None),
            foreign,
            transfer,
        ) {
            Ok(owner) => {
                owner.cancel().unwrap();
                panic!("foreign Doctor report transfer must fail")
            }
            Err(error) => error,
        };

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(transfer_owner.snapshot(), &owner_before);
        assert!(foreign_before.current.values().all(|amount| *amount == 0));
        transfer_owner
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                transferred,
            )
            .unwrap();
        assert!(!transfer_owner.has_open_permits());
        assert!(transfer_owner
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
    }

    #[test]
    fn doctor_owner_visibility_audit_keeps_payload_escape_hatches_test_only() {
        let owner_source = include_str!("accounting.rs");
        let report_source = include_str!("../report.rs");
        let test_deref = ["#[cfg(test)]\nimpl Deref", " for AccountedDoctorReport"].concat();
        let payload_debug = ["impl fmt::", "Debug for AccountedDoctorReport"].concat();

        assert!(owner_source.contains(&test_deref));
        assert!(!owner_source.contains(&payload_debug));
        assert!(!report_source.contains("pub(crate) fn doctor_report_data_json"));
        assert!(!report_source.contains("pub(crate) fn doctor_report_diagnostics_json"));
        assert!(!report_source.contains("pub(crate) fn doctor_report_next_actions"));
    }

    #[test]
    fn doctor_text_render_and_explicit_cancel_drop_all_payload_before_release() {
        let report = sample_report(Some(DoctorIssue::warning(
            "empty_object_store",
            "empty",
            Some(PathBuf::from("/repo/.codefire/objects")),
        )));
        let mut expected = String::new();
        write_doctor_text(&report, &mut expected).unwrap();
        let owner = AccountedDoctorReport::new(
            report,
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let mut output = Vec::new();

        let (result, snapshot, permits) = owner.write_text_with_snapshot(&mut output);

        result.unwrap();
        assert_eq!(output, expected.as_bytes());
        assert_clean(&snapshot, permits);

        let owner = AccountedDoctorReport::new(
            sample_report(None),
            ledger(codefire_core::DecisionWorkProfile::local_v1()),
        )
        .unwrap();
        let (cancelled, permits) = owner.cancel_with_snapshot();
        assert_clean(&cancelled, permits);
        assert_eq!(
            cancelled.terminal_reason,
            Some(codefire_core::DecisionWorkTerminalReason::CorruptSubject)
        );
    }

    #[test]
    fn doctor_projection_permit_rejects_foreign_ledger_without_losing_cleanup() {
        let mut owner = ledger(codefire_core::DecisionWorkProfile::local_v1());
        let mut foreign = ledger(codefire_core::DecisionWorkProfile::local_v1());
        let mut permit = owner
            .reserve(
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 8),
            )
            .unwrap();

        let error = foreign
            .consume_reserved(
                &mut permit,
                codefire_core::DecisionWorkReservation::new(
                    codefire_core::DecisionWorkPhase::PublicationPreflight,
                    codefire_core::DecisionWorkSurface::Atomic,
                )
                .acquire(codefire_core::DecisionWorkDimension::TemporaryBytes, 8),
            )
            .unwrap_err();

        assert!(matches!(
            error,
            codefire_core::DecisionWorkAccountingError::InvalidPermit
        ));
        assert_eq!(owner.open_permit_count(), 1);
        assert_eq!(foreign.open_permit_count(), 0);
        assert!(foreign
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
        owner.cancel_reserved(&permit).unwrap();
        assert_eq!(owner.open_permit_count(), 0);
        assert!(owner.snapshot().current.values().all(|amount| *amount == 0));
    }
}
