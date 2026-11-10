//! Budgeted branch listing and its wire encoding.

use crate::branch_refs::{
    BranchRefSnapshotError, BranchRefWork, OpenedRegistrySnapshot, RepositoryMetadataSnapshot,
};
use crate::cli_error::CliError;
use crate::json_files::REPOSITORY_JSON_BYTE_LIMIT;
use crate::metrics::{metrics_wire, write_metrics, CommandMetrics, MetricsWire};
use crate::{automation, find_repo_root, Branch};
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
#[must_use = "branch-list results own an unfinished command ledger and must be completed"]
pub(crate) struct BranchListResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) metadata_snapshot: RepositoryMetadataSnapshot,
    pub(crate) branches: Vec<Branch>,
    pub(crate) opened: Vec<Value>,
    pub(crate) store_work: codefire_store::SealedValidationWork,
    pub(crate) ref_work: BranchRefWork,
    pub(crate) opened_work: BranchRefWork,
    pub(crate) opened_snapshot: Option<OpenedRegistrySnapshot>,
    pub(crate) ledger: codefire_core::DecisionWorkLedger,
    pub(crate) retained_bytes: u64,
}

impl BranchListResult {
    pub(crate) fn decision_work_snapshot(&self) -> &codefire_core::DecisionWorkSnapshot {
        self.ledger.snapshot()
    }

    pub(crate) fn prepare_opened_registries(mut self) -> Result<Self, CliError> {
        if let Err(error) = self.metadata_snapshot.ensure_current() {
            let error = branch_list_ref_error(error);
            let reason = branch_list_cli_terminal_reason(&error);
            self.abort_after_branch_list_failure(reason);
            return Err(error);
        }
        let opened_root = self.repo_root.join(".codefire").join("opened");
        let snapshot = match OpenedRegistrySnapshot::capture(
            &opened_root,
            REPOSITORY_JSON_BYTE_LIMIT,
            &mut self.ledger,
        ) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let error = opened_registry_ref_error(error);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        if snapshot.parent_identity() != self.metadata_snapshot.identity() {
            let _ = snapshot.release(&mut self.ledger);
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::SubjectChanged,
            );
            return Err(CliError::OpenedRegistrySubjectChanged {
                subject: opened_root,
                message: "repository metadata identity changed between branch and opened capture"
                    .to_string(),
            });
        }
        let projection = project_opened_registries(&snapshot, &mut self.ledger);
        match projection {
            Ok((opened, retained_bytes)) => {
                let Some(total_retained) = self.retained_bytes.checked_add(retained_bytes) else {
                    let _ = self.ledger.release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    );
                    let _ = snapshot.release(&mut self.ledger);
                    self.abort_after_branch_list_failure(
                        codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    );
                    return Err(CliError::InvalidRepository(
                        "opened registry projection size overflow".to_string(),
                    ));
                };
                self.retained_bytes = total_retained;
                self.opened = opened;
                self.opened_snapshot = Some(snapshot);
                Ok(self)
            }
            Err(error) => {
                let _ = snapshot.release(&mut self.ledger);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                Err(error)
            }
        }
    }

    pub(crate) fn revalidate_opened_registries(&mut self) -> Result<(), CliError> {
        let Some(snapshot) = self.opened_snapshot.take() else {
            return Ok(());
        };
        match snapshot.revalidate(REPOSITORY_JSON_BYTE_LIMIT, &mut self.ledger) {
            Ok(work) => {
                self.opened_work = work;
                Ok(())
            }
            Err(error) => {
                let error = opened_registry_ref_error(error);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                Err(error)
            }
        }
    }

    pub(crate) fn output_branch_list(
        mut self,
        json_output: bool,
        metrics: Option<CommandMetrics>,
    ) -> Result<(), CliError> {
        let encoded = if json_output {
            let data = BranchListDataWire {
                type_tag: "codefire_branch_list",
                version: 1,
                branches: BranchesWire(&self.branches),
                metrics: metrics.as_ref().map(metrics_wire),
            };
            let envelope = CommandResultWire {
                schema: automation::COMMAND_RESULT_SCHEMA,
                command: "branch-list",
                ok: true,
                exit_code: 0,
                repo: Some(&self.repo_root),
                data,
                diagnostics: &[],
                next_actions: &[],
            };
            encode_pretty_json_output(&mut self.ledger, &envelope)
        } else {
            encode_text_output(&mut self.ledger, |writer| {
                write_branches(writer, &self.branches)?;
                if let Some(metrics) = metrics.as_ref() {
                    write_metrics(writer, metrics)?;
                }
                Ok(())
            })
        };
        let (bytes, retained_bytes) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => {
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        self.retained_bytes = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| CliError::InvalidRepository("output size overflow".to_string()))?;
        self.publish_and_finish(bytes)
    }

    pub(crate) fn output_local_list(mut self, json_output: bool) -> Result<(), CliError> {
        let encoded = if json_output {
            let data = LocalListDataWire {
                type_tag: "codefire_local_list",
                version: 1,
                branches: BranchesWire(&self.branches),
                opened: &self.opened,
            };
            let envelope = CommandResultWire {
                schema: automation::COMMAND_RESULT_SCHEMA,
                command: "list",
                ok: true,
                exit_code: 0,
                repo: Some(&self.repo_root),
                data,
                diagnostics: &[],
                next_actions: &[],
            };
            encode_pretty_json_output(&mut self.ledger, &envelope)
        } else {
            encode_text_output(&mut self.ledger, |writer| {
                writeln!(writer, "Branches:")?;
                write_branches(writer, &self.branches)?;
                writeln!(writer, "Opened:")?;
                if self.opened.is_empty() {
                    writeln!(writer, "none")?;
                } else {
                    for item in &self.opened {
                        writeln!(
                            writer,
                            "{}\t{}",
                            item.get("branch")
                                .and_then(Value::as_str)
                                .unwrap_or("(unknown)"),
                            item.get("path").and_then(Value::as_str).unwrap_or("")
                        )?;
                    }
                }
                Ok(())
            })
        };
        let (bytes, retained_bytes) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => {
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        self.retained_bytes = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| CliError::InvalidRepository("output size overflow".to_string()))?;
        if let Err(error) = self.revalidate_opened_registries() {
            drop(bytes);
            return Err(error);
        }
        self.publish_and_finish(bytes)
    }

    pub(crate) fn publish_and_finish(mut self, bytes: Vec<u8>) -> Result<(), CliError> {
        if let Err(error) = self.metadata_snapshot.ensure_current() {
            drop(bytes);
            let error = branch_list_ref_error(error);
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::SubjectChanged,
            );
            return Err(error);
        }
        let write_result = std::io::stdout().lock().write_all(&bytes);
        drop(bytes);
        if let Err(error) = write_result {
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            );
            return Err(CliError::Io(error));
        }
        self.finish_complete().map(drop)
    }

    pub(crate) fn abort_after_branch_list_failure(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) {
        if let Some(snapshot) = self.opened_snapshot.take() {
            let _ = snapshot.release(&mut self.ledger);
        }
        if self.retained_bytes > 0 {
            let _ = self.ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            );
            self.retained_bytes = 0;
        }
        if self.ledger.snapshot().terminal_reason.is_none() {
            let _ = self.ledger.finish(reason);
        }
    }

    pub(crate) fn finish_complete(
        mut self,
    ) -> Result<codefire_core::DecisionWorkSnapshot, CliError> {
        self.ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )
            .map_err(branch_list_accounting_error)?;
        self.retained_bytes = 0;
        self.ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .map_err(branch_list_accounting_error)?;
        Ok(self.ledger.snapshot().clone())
    }
}

pub(crate) fn write_branches<W: Write + ?Sized>(
    writer: &mut W,
    branches: &[Branch],
) -> std::io::Result<()> {
    for branch in branches {
        writeln!(writer, "{}\t{}\t{}", branch.name, branch.head, branch.state)?;
    }
    Ok(())
}

pub(crate) fn list_branches(start: &Path) -> Result<BranchListResult, CliError> {
    let ledger = codefire_core::DecisionWorkLedger::new(
        codefire_core::DecisionWorkProfile::local_v1(),
        codefire_core::DecisionWorkCommandKind::LocalGraph,
        "branch-list",
    )
    .map_err(branch_list_accounting_error)?;
    list_branches_with_ledger_and_hook(start, ledger, |_| Ok(()))
}

pub(crate) fn list_branches_with_ledger_and_hook<F>(
    start: &Path,
    mut ledger: codefire_core::DecisionWorkLedger,
    before_finalize: F,
) -> Result<BranchListResult, CliError>
where
    F: FnOnce(&Path) -> Result<(), CliError>,
{
    let repo_root = match find_repo_root(start) {
        Ok(repo_root) => repo_root,
        Err(error) => {
            if ledger.snapshot().terminal_reason.is_none() {
                let _ = ledger.finish(branch_list_cli_terminal_reason(&error));
            }
            return Err(error);
        }
    };
    let branches_root = repo_root.join(".codefire").join("branches");
    let objects_root = repo_root.join(".codefire").join("objects");
    let metadata_snapshot = match RepositoryMetadataSnapshot::capture(&repo_root.join(".codefire"))
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    let branch_snapshot = match metadata_snapshot.capture_branch_refs(
        &branches_root,
        REPOSITORY_JSON_BYTE_LIMIT,
        &mut ledger,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    let mut validation =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            &objects_root,
            ledger,
        ) {
            Ok(validation) => validation,
            Err(begin_error) => {
                let (error, mut ledger) = begin_error.into_parts();
                branch_snapshot
                    .release(&mut ledger)
                    .map_err(branch_list_ref_error)?;
                finish_branch_list_store_failure(&mut ledger, &error);
                return Err(branch_list_store_error(error));
            }
        };
    let mut validation_error = None;
    for document in branch_snapshot.documents() {
        if let Err(error) = validation.validate_commit(document.head()) {
            validation_error = Some(error);
            break;
        }
    }
    if let Some(error) = validation_error {
        let reason = branch_list_store_terminal_reason(&error);
        let mut ledger = validation
            .abort_and_take_decision_work_ledger(reason)
            .map_err(branch_list_store_error)?
            .expect("branch-list validation always owns a decision-work ledger");
        branch_snapshot
            .release(&mut ledger)
            .map_err(branch_list_ref_error)?;
        return Err(branch_list_store_error(error));
    }
    if let Err(error) = before_finalize(&branches_root) {
        let reason = branch_list_cli_terminal_reason(&error);
        let mut ledger = validation
            .abort_and_take_decision_work_ledger(reason)
            .map_err(branch_list_store_error)?
            .expect("branch-list validation always owns a decision-work ledger");
        branch_snapshot
            .release(&mut ledger)
            .map_err(branch_list_ref_error)?;
        return Err(error);
    }
    let store_work = match validation.finalize() {
        Ok(work) => work,
        Err(error) => {
            let mut ledger = validation
                .take_decision_work_ledger()
                .expect("finalized branch-list validation can return its ledger")
                .expect("branch-list validation always owns a decision-work ledger");
            branch_snapshot
                .release(&mut ledger)
                .map_err(branch_list_ref_error)?;
            return Err(branch_list_store_error(error));
        }
    };
    let mut ledger = validation
        .take_decision_work_ledger()
        .expect("finalized branch-list validation can return its ledger")
        .expect("branch-list validation always owns a decision-work ledger");
    let mut branches = Vec::with_capacity(branch_snapshot.documents().len());
    let mut retained_bytes = 0_u64;
    for document in branch_snapshot.documents() {
        let state = document
            .value()
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("closed");
        let branch_bytes = compact_branch_result_size(document.name(), document.head(), state)?;
        let next_retained = retained_bytes.checked_add(branch_bytes).ok_or_else(|| {
            CliError::InvalidRepository("branch result size overflow".to_string())
        })?;
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                branch_bytes,
            ),
        ) {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            branch_snapshot
                .release(&mut ledger)
                .map_err(branch_list_ref_error)?;
            return Err(branch_list_accounting_error(error));
        }
        retained_bytes = next_retained;
        branches.push(Branch {
            name: document.name().to_string(),
            head: document.head().to_string(),
            state: state.to_string(),
        });
    }
    debug_assert!(branches
        .iter()
        .all(|branch| branch_snapshot.heads().contains(&branch.head)));
    let ref_work = match branch_snapshot.revalidate(REPOSITORY_JSON_BYTE_LIMIT, &mut ledger) {
        Ok(work) => work,
        Err(error) => {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    if let Err(error) = metadata_snapshot.ensure_current() {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        finish_branch_list_failure(&mut ledger, &error);
        return Err(branch_list_ref_error(error));
    }
    Ok(BranchListResult {
        repo_root,
        metadata_snapshot,
        branches,
        opened: Vec::new(),
        store_work,
        ref_work,
        opened_work: BranchRefWork::default(),
        opened_snapshot: None,
        ledger,
        retained_bytes,
    })
}

#[derive(Serialize)]
pub(crate) struct BorrowedBranchResult<'a> {
    pub(crate) name: &'a str,
    pub(crate) head: &'a str,
    pub(crate) state: &'a str,
}

#[derive(Serialize)]
pub(crate) struct BorrowedOpenedRegistryResult<'a> {
    pub(crate) branch: &'a str,
    pub(crate) path: &'a str,
    pub(crate) open_instance_id: Option<&'a Value>,
    pub(crate) current_base_commit: Option<&'a Value>,
    pub(crate) opened_from_commit: Option<&'a Value>,
    pub(crate) registry_path: &'a Path,
}

pub(crate) struct BranchesWire<'a>(pub(crate) &'a [Branch]);

impl Serialize for BranchesWire<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for branch in self.0 {
            sequence.serialize_element(&BorrowedBranchResult {
                name: &branch.name,
                head: &branch.head,
                state: &branch.state,
            })?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
pub(crate) struct BranchListDataWire<'a> {
    #[serde(rename = "type")]
    pub(crate) type_tag: &'static str,
    pub(crate) version: u64,
    pub(crate) branches: BranchesWire<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metrics: Option<MetricsWire<'a>>,
}

#[derive(Serialize)]
pub(crate) struct LocalListDataWire<'a> {
    #[serde(rename = "type")]
    pub(crate) type_tag: &'static str,
    pub(crate) version: u64,
    pub(crate) branches: BranchesWire<'a>,
    pub(crate) opened: &'a [Value],
}

#[derive(Serialize)]
pub(crate) struct CommandResultWire<'a, T> {
    pub(crate) schema: &'static str,
    pub(crate) command: &'static str,
    pub(crate) ok: bool,
    pub(crate) exit_code: i32,
    pub(crate) repo: Option<&'a Path>,
    pub(crate) data: T,
    pub(crate) diagnostics: &'a [Value],
    pub(crate) next_actions: &'a [Value],
}

#[derive(Default)]
pub(crate) struct JsonSizeWriter(pub(crate) u64);

impl Write for JsonSizeWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("serialized JSON size does not fit u64"))?;
        self.0 = self
            .0
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("serialized JSON size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn reserve_output_bytes(
    ledger: &mut codefire_core::DecisionWorkLedger,
    bytes: u64,
) -> Result<(), CliError> {
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, bytes)
            .count(
                codefire_core::DecisionWorkCounter::ResponseBytesEncoded,
                bytes,
            ),
        )
        .map_err(branch_list_accounting_error)
}

pub(crate) fn encode_pretty_json_output(
    ledger: &mut codefire_core::DecisionWorkLedger,
    value: &impl Serialize,
) -> Result<(Vec<u8>, u64), CliError> {
    let mut counter = JsonSizeWriter::default();
    serde_json::to_writer_pretty(&mut counter, value)?;
    let retained_bytes = counter
        .0
        .checked_add(1)
        .ok_or_else(|| CliError::InvalidRepository("JSON output size overflow".to_string()))?;
    reserve_output_bytes(ledger, retained_bytes)?;
    let capacity = usize::try_from(retained_bytes)
        .map_err(|_| CliError::InvalidRepository("JSON output is too large".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity);
    if let Err(error) = serde_json::to_writer_pretty(&mut bytes, value) {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        return Err(error.into());
    }
    bytes.push(b'\n');
    debug_assert_eq!(bytes.len(), capacity);
    Ok((bytes, retained_bytes))
}

pub(crate) fn encode_text_output<F>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    render: F,
) -> Result<(Vec<u8>, u64), CliError>
where
    F: Fn(&mut dyn Write) -> std::io::Result<()>,
{
    let mut counter = JsonSizeWriter::default();
    render(&mut counter)?;
    let retained_bytes = counter.0;
    reserve_output_bytes(ledger, retained_bytes)?;
    let capacity = usize::try_from(retained_bytes)
        .map_err(|_| CliError::InvalidRepository("text output is too large".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity);
    if let Err(error) = render(&mut bytes) {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        return Err(error.into());
    }
    debug_assert_eq!(bytes.len(), capacity);
    Ok((bytes, retained_bytes))
}

pub(crate) fn compact_branch_result_size(
    name: &str,
    head: &str,
    state: &str,
) -> Result<u64, CliError> {
    let mut writer = JsonSizeWriter::default();
    serde_json::to_writer(&mut writer, &BorrowedBranchResult { name, head, state })?;
    Ok(writer.0)
}

pub(crate) fn project_opened_registries(
    snapshot: &OpenedRegistrySnapshot,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<Value>, u64), CliError> {
    let mut retained_bytes = 0_u64;
    let result = (|| {
        let mut opened = Vec::with_capacity(snapshot.documents().len());
        for document in snapshot.documents() {
            let registry = document.value();
            let branch = registry
                .get("branch")
                .and_then(Value::as_str)
                .or_else(|| {
                    registry
                        .get("branch")
                        .and_then(|branch| branch.get("name"))
                        .and_then(Value::as_str)
                })
                .or_else(|| registry.get("name").and_then(Value::as_str))
                .ok_or_else(|| {
                    CliError::InvalidRepository(format!(
                        "opened registry at {} has no branch name",
                        document.display_path().display()
                    ))
                })?;
            let open = registry.get("open").and_then(Value::as_object);
            let open_path = open
                .and_then(|open| open.get("path").or_else(|| open.get("opened_path")))
                .and_then(Value::as_str)
                .unwrap_or("");
            let borrowed = BorrowedOpenedRegistryResult {
                branch,
                path: open_path,
                open_instance_id: open.and_then(|open| open.get("open_instance_id")),
                current_base_commit: open.and_then(|open| open.get("current_base_commit")),
                opened_from_commit: open.and_then(|open| open.get("opened_from_commit")),
                registry_path: document.display_path(),
            };
            let mut counter = JsonSizeWriter::default();
            serde_json::to_writer(&mut counter, &borrowed)?;
            let next_retained = retained_bytes.checked_add(counter.0).ok_or_else(|| {
                CliError::InvalidRepository("opened registry projection size overflow".to_string())
            })?;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        counter.0,
                    ),
                )
                .map_err(branch_list_accounting_error)?;
            retained_bytes = next_retained;
            opened.push(json!({
                "branch": branch,
                "path": open_path,
                "open_instance_id": borrowed.open_instance_id.cloned().unwrap_or(Value::Null),
                "current_base_commit": borrowed.current_base_commit.cloned().unwrap_or(Value::Null),
                "opened_from_commit": borrowed.opened_from_commit.cloned().unwrap_or(Value::Null),
                "registry_path": document.display_path(),
            }));
        }
        Ok(opened)
    })();
    match result {
        Ok(opened) => Ok((opened, retained_bytes)),
        Err(error) => {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            Err(error)
        }
    }
}

pub(crate) fn finish_branch_list_failure(
    ledger: &mut codefire_core::DecisionWorkLedger,
    error: &BranchRefSnapshotError,
) {
    if ledger.snapshot().terminal_reason.is_none() {
        let reason = match error {
            BranchRefSnapshotError::SubjectChanged { .. } => {
                codefire_core::DecisionWorkTerminalReason::SubjectChanged
            }
            _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
        };
        let _ = ledger.finish(reason);
    }
}

pub(crate) fn finish_branch_list_store_failure(
    ledger: &mut codefire_core::DecisionWorkLedger,
    error: &codefire_store::StoreError,
) {
    if ledger.snapshot().terminal_reason.is_none() {
        let _ = ledger.finish(branch_list_store_terminal_reason(error));
    }
}

pub(crate) fn branch_list_store_terminal_reason(
    error: &codefire_store::StoreError,
) -> codefire_core::DecisionWorkTerminalReason {
    if matches!(
        error,
        codefire_store::StoreError::SealedValidationSubjectChanged { .. }
    ) {
        codefire_core::DecisionWorkTerminalReason::SubjectChanged
    } else {
        codefire_core::DecisionWorkTerminalReason::CorruptSubject
    }
}

pub(crate) fn branch_list_cli_terminal_reason(
    error: &CliError,
) -> codefire_core::DecisionWorkTerminalReason {
    if matches!(
        error,
        CliError::BranchReferenceSubjectChanged { .. }
            | CliError::OpenedRegistrySubjectChanged { .. }
    ) {
        codefire_core::DecisionWorkTerminalReason::SubjectChanged
    } else {
        codefire_core::DecisionWorkTerminalReason::CorruptSubject
    }
}

pub(crate) fn branch_list_ref_error(error: BranchRefSnapshotError) -> CliError {
    match error {
        BranchRefSnapshotError::Io(error) => CliError::Io(error),
        BranchRefSnapshotError::Json(error) => CliError::Json(error),
        BranchRefSnapshotError::Accounting(error) => branch_list_accounting_error(error),
        BranchRefSnapshotError::SubjectChanged { subject, reason } => {
            CliError::BranchReferenceSubjectChanged {
                subject,
                message: reason,
            }
        }
        BranchRefSnapshotError::InvalidRecord { path, reason } => CliError::InvalidRepository(
            format!("invalid branch reference at {}: {reason}", path.display()),
        ),
    }
}

pub(crate) fn opened_registry_ref_error(error: BranchRefSnapshotError) -> CliError {
    match error {
        BranchRefSnapshotError::Io(error) => CliError::Io(error),
        BranchRefSnapshotError::Json(error) => CliError::Json(error),
        BranchRefSnapshotError::Accounting(error) => branch_list_accounting_error(error),
        BranchRefSnapshotError::SubjectChanged { subject, reason } => {
            CliError::OpenedRegistrySubjectChanged {
                subject,
                message: reason,
            }
        }
        BranchRefSnapshotError::InvalidRecord { path, reason } => CliError::InvalidRepository(
            format!("invalid opened registry at {}: {reason}", path.display()),
        ),
    }
}

pub(crate) fn branch_list_accounting_error(
    error: codefire_core::DecisionWorkAccountingError,
) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while listing branches: {error}"
        )),
    }
}

pub(crate) fn branch_list_store_error(error: codefire_store::StoreError) -> CliError {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
            branch_list_accounting_error(*accounting)
        }
        error => CliError::from(error),
    }
}
