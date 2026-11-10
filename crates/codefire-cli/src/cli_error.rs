//! The CLI error type, its exit codes, and its diagnostics.

use crate::cli_output::{
    batch_file_diagnostic_kind, batch_schema_command, unknown_target_parts, usage_diagnostic_kind,
};
use crate::exit_code::{core_error_exit_code, store_error_exit_code, usage_exit_code, ExitCode};
use crate::{active_state, open_snapshot};
use serde_json::{json, Value};
use std::fmt;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct OpenTargetResolutionError {
    pub(crate) message: String,
    pub(crate) reason: &'static str,
    pub(crate) repo_root: PathBuf,
    pub(crate) branch: String,
    pub(crate) requested_path: PathBuf,
    pub(crate) resolved_path: Option<PathBuf>,
    pub(crate) workspace_boundary: PathBuf,
}

#[derive(Debug)]
pub(crate) struct OpenRecoveryPendingError {
    pub(crate) operation: String,
    pub(crate) phase: String,
    pub(crate) repo_root: PathBuf,
    pub(crate) branch: String,
    pub(crate) open_dir: PathBuf,
}

#[derive(Debug)]
pub(crate) enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Core(codefire_core::CoreError),
    CoreContext {
        error: codefire_core::CoreError,
        repo_root: PathBuf,
        path: PathBuf,
    },
    Store(Box<codefire_store::StoreError>),
    StoreContext {
        error: Box<codefire_store::StoreError>,
        repo_root: PathBuf,
        path: PathBuf,
    },
    Usage(String),
    AuthenticationOrSignature(String),
    IdempotencyConflict(String),
    ResourceBudget(Box<codefire_core::DecisionWorkBudgetExceeded>),
    RequestDeadlineExceeded {
        phase: &'static str,
        limit_ms: u64,
    },
    RemoteDiagnostic {
        status: u16,
        exit_code: ExitCode,
        kind: String,
        message: String,
        resource_budget: Option<Box<codefire_core::DecisionWorkBudgetExceeded>>,
    },
    MigrationIncompatibility(String),
    VerificationFailed(ExitCode),
    CommandFailed(ExitCode),
    NotOpen(PathBuf),
    InvalidMarker(String),
    InvalidRepository(String),
    BranchReferenceSubjectChanged {
        subject: PathBuf,
        message: String,
    },
    OpenedRegistrySubjectChanged {
        subject: PathBuf,
        message: String,
    },
    OpenSnapshotDivergence(Box<open_snapshot::OpenSnapshotDivergence>),
    OpenRecoveryPending(Box<OpenRecoveryPendingError>),
    ActiveStateCapability(Box<active_state::ActiveStateError>),
    LockContention(String),
    RepositoryLockContention {
        message: String,
        repo_root: PathBuf,
        lock_path: PathBuf,
    },
    BranchOperation {
        message: String,
        repo_root: PathBuf,
        branch: Option<String>,
        path: Option<PathBuf>,
    },
    OpenTargetResolution(Box<OpenTargetResolutionError>),
    BatchFileRead {
        operation: &'static str,
        path: PathBuf,
        kind: ErrorKind,
        message: String,
    },
    EvidenceCommandFailed {
        exit_code: Option<i32>,
        timed_out: bool,
        cwd: String,
        message: String,
    },
    MissingEvidenceRef {
        evidence_id: String,
        repo_root: PathBuf,
    },
}

impl CliError {
    pub(crate) fn exit_status(&self) -> ExitCode {
        match self {
            CliError::Io(_) => ExitCode::GenericFailure,
            CliError::Json(_) => ExitCode::RepositoryCorruption,
            CliError::Core(error) => core_error_exit_code(error),
            CliError::CoreContext { error, .. } => core_error_exit_code(error),
            CliError::Store(error) => store_error_exit_code(error),
            CliError::StoreContext { error, .. } => store_error_exit_code(error),
            CliError::Usage(message) => usage_exit_code(message),
            CliError::AuthenticationOrSignature(_) => ExitCode::AuthenticationOrSignatureFailure,
            CliError::IdempotencyConflict(_) => ExitCode::IdempotencyConflict,
            CliError::ResourceBudget(_) => ExitCode::ResourceBudgetExceeded,
            CliError::RequestDeadlineExceeded { .. } => ExitCode::ResourceBudgetExceeded,
            CliError::RemoteDiagnostic { exit_code, .. } => *exit_code,
            CliError::MigrationIncompatibility(_) => ExitCode::MigrationIncompatibility,
            CliError::VerificationFailed(exit_code) => *exit_code,
            CliError::CommandFailed(exit_code) => *exit_code,
            CliError::NotOpen(_) => ExitCode::InvalidUsageOrConfig,
            CliError::InvalidMarker(_) | CliError::InvalidRepository(_) => {
                ExitCode::RepositoryCorruption
            }
            CliError::BranchReferenceSubjectChanged { .. }
            | CliError::OpenedRegistrySubjectChanged { .. } => ExitCode::RepositoryCorruption,
            CliError::OpenSnapshotDivergence(_) => ExitCode::RepositoryCorruption,
            CliError::OpenRecoveryPending(_) => ExitCode::RepositoryCorruption,
            CliError::ActiveStateCapability(_) => ExitCode::RepositoryCorruption,
            CliError::LockContention(_) | CliError::RepositoryLockContention { .. } => {
                ExitCode::LockContention
            }
            CliError::BranchOperation { message, .. } => usage_exit_code(message),
            CliError::OpenTargetResolution(error) => usage_exit_code(&error.message),
            CliError::BatchFileRead { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::EvidenceCommandFailed { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::MissingEvidenceRef { .. } => ExitCode::ObjectReferenceInvalid,
        }
    }

    pub(crate) fn exit_code(&self) -> i32 {
        self.exit_status().code()
    }

    pub(crate) fn diagnostic(&self) -> Value {
        match self {
            CliError::MissingEvidenceRef {
                evidence_id,
                repo_root,
            } => json!({
                "kind": "missing_evidence_ref",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "evidence_id": evidence_id,
                "repo": repo_root,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::Usage(message) if unknown_target_parts(message).is_some() => {
                let (kind, id) = unknown_target_parts(message).expect("checked above");
                json!({
                    "kind": "unknown_target",
                    "severity": "blocking",
                    "blocking": true,
                    "repairable": true,
                    "target": {
                        "kind": kind,
                        "id": id,
                    },
                    "exit_code": self.exit_code(),
                    "message": self.to_string(),
                })
            }
            CliError::Usage(message) if batch_schema_command(message).is_some() => json!({
                "kind": "batch_schema_error",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "command": batch_schema_command(message).expect("checked above"),
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::Usage(message) => json!({
                "kind": usage_diagnostic_kind(message),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::BatchFileRead {
                operation,
                path,
                kind,
                ..
            } => json!({
                "kind": batch_file_diagnostic_kind(*kind),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "operation": operation,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::EvidenceCommandFailed {
                exit_code,
                timed_out,
                cwd,
                ..
            } => json!({
                "kind": "evidence_command_failed",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "exit_code": self.exit_code(),
                "process_exit_code": exit_code,
                "timed_out": timed_out,
                "cwd": cwd,
                "message": self.to_string(),
            }),
            CliError::Io(_) => self.simple_diagnostic("io_error", true),
            CliError::Json(_) => self.simple_diagnostic("json_error", false),
            CliError::Core(error) => core_error_diagnostic(error, None, None, self.exit_code()),
            CliError::CoreContext {
                error,
                repo_root,
                path,
            } => core_error_diagnostic(error, Some(repo_root), Some(path), self.exit_code()),
            CliError::Store(error) => store_error_diagnostic(error, None, None, self.exit_code()),
            CliError::StoreContext {
                error,
                repo_root,
                path,
            } => store_error_diagnostic(error, Some(repo_root), Some(path), self.exit_code()),
            CliError::AuthenticationOrSignature(_) => {
                self.simple_diagnostic("authentication_or_signature_error", true)
            }
            CliError::IdempotencyConflict(_) => {
                self.simple_diagnostic("idempotency_conflict", true)
            }
            CliError::ResourceBudget(error) => {
                resource_budget_diagnostic(error, None, &self.to_string())
            }
            CliError::RequestDeadlineExceeded { phase, limit_ms } => json!({
                "kind": "request_deadline_exceeded",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "phase": phase,
                "limit_ms": limit_ms,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::RemoteDiagnostic {
                status,
                exit_code,
                kind,
                resource_budget,
                ..
            } => resource_budget.as_ref().map_or_else(
                || {
                    json!({
                        "kind": kind,
                        "severity": "blocking",
                        "blocking": true,
                        "repairable": true,
                        "http_status": status,
                        "exit_code": exit_code.code(),
                        "message": self.to_string(),
                    })
                },
                |budget| resource_budget_diagnostic(budget, Some(*status), &self.to_string()),
            ),
            CliError::MigrationIncompatibility(_) => {
                self.simple_diagnostic("migration_incompatibility", true)
            }
            CliError::VerificationFailed(_) => self.simple_diagnostic("verification_failed", true),
            CliError::CommandFailed(_) => self.simple_diagnostic("command_failed", true),
            CliError::NotOpen(path) => json!({
                "kind": "not_open",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::InvalidMarker(_) => self.simple_diagnostic("invalid_marker", true),
            CliError::InvalidRepository(_) => self.simple_diagnostic("invalid_repository", true),
            CliError::BranchReferenceSubjectChanged { subject, .. } => json!({
                "kind": "branch_reference_subject_changed",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "subject": subject,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenedRegistrySubjectChanged { subject, .. } => json!({
                "kind": "opened_registry_subject_changed",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "subject": subject,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenSnapshotDivergence(divergence) => json!({
                "kind": open_snapshot::OPEN_SNAPSHOT_DIVERGENCE_KIND,
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "phase": &divergence.phase,
                "authority": divergence.authority,
                "expected": &divergence.expected,
                "actual": &divergence.actual,
                "expected_record": &divergence.expected_record,
                "actual_record": &divergence.actual_record,
                "repo": &divergence.repo_root,
                "branch": &divergence.branch,
                "exit_code": self.exit_code(),
                "message": divergence.to_string(),
            }),
            CliError::OpenRecoveryPending(pending) => json!({
                "kind": "open_recovery_pending",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "operation": &pending.operation,
                "phase": &pending.phase,
                "repo": &pending.repo_root,
                "branch": &pending.branch,
                "open_dir": &pending.open_dir,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::ActiveStateCapability(error) => json!({
                "kind": "active_state_capability_invalid",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "operation": error.operation(),
                "path": error.path(),
                "exit_code": self.exit_code(),
                "message": error.to_string(),
            }),
            CliError::LockContention(_) => self.simple_diagnostic("lock_contention", true),
            CliError::RepositoryLockContention {
                repo_root,
                lock_path,
                ..
            } => json!({
                "kind": "lock_contention",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "repo": repo_root,
                "lock_path": lock_path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::BranchOperation {
                message,
                repo_root,
                branch,
                path,
            } => json!({
                "kind": usage_diagnostic_kind(message),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "repo": repo_root,
                "branch": branch,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenTargetResolution(error) => json!({
                "kind": "open_target_resolution_failed",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "reason": error.reason,
                "repo": error.repo_root,
                "branch": error.branch,
                "requested_path": error.requested_path,
                "resolved_path": error.resolved_path,
                "workspace_boundary": error.workspace_boundary,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
        }
    }

    pub(crate) fn simple_diagnostic(&self, kind: &str, repairable: bool) -> Value {
        json!({
            "kind": kind,
            "severity": "blocking",
            "blocking": true,
            "repairable": repairable,
            "exit_code": self.exit_code(),
            "message": self.to_string(),
        })
    }
}

pub(crate) fn resource_budget_diagnostic(
    error: &codefire_core::DecisionWorkBudgetExceeded,
    http_status: Option<u16>,
    message: &str,
) -> Value {
    let mut diagnostic = serde_json::to_value(error).unwrap_or_else(|_| {
        json!({
            "kind": "resource_budget_exceeded",
            "exit_code": ExitCode::ResourceBudgetExceeded.code(),
        })
    });
    if let Some(fields) = diagnostic.as_object_mut() {
        fields.insert("severity".to_string(), json!("blocking"));
        fields.insert("blocking".to_string(), json!(true));
        fields.insert("repairable".to_string(), json!(true));
        fields.insert("message".to_string(), json!(message));
        if let Some(status) = http_status {
            fields.insert("http_status".to_string(), json!(status));
        }
    }
    diagnostic
}

pub(crate) fn core_error_diagnostic(
    error: &codefire_core::CoreError,
    repo_root: Option<&Path>,
    path: Option<&Path>,
    exit_code: i32,
) -> Value {
    match error {
        codefire_core::CoreError::TraceGraphEndpointIntegrity(error) => json!({
            "kind": "trace_graph_endpoint_integrity",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "total_issues": error.total_issues,
            "issues_omitted": error.omitted_issues,
            "issues": &error.issues,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_core::CoreError::SealedSnapshotSemanticIntegrity(error) => json!({
            "kind": "sealed_snapshot_semantic_integrity",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "total_issues": error.issues.len(),
            "issues": &error.issues,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        _ => json!({
            "kind": "core_error",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
    }
}

pub(crate) fn store_error_diagnostic(
    error: &codefire_store::StoreError,
    repo_root: Option<&Path>,
    path: Option<&Path>,
    exit_code: i32,
) -> Value {
    match error {
        codefire_store::StoreError::ObjectCollision {
            object_id,
            requested_type,
            existing_type,
            requested_hash,
            existing_hash,
            canonical_payload_matches,
        } => json!({
            "kind": "object_id_collision",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "object_id": object_id,
            "requested_type": requested_type,
            "existing_type": existing_type,
            "requested_hash": requested_hash,
            "existing_hash": existing_hash,
            "canonical_payload_matches": canonical_payload_matches,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_store::StoreError::InvalidSealedCommit(_) => json!({
            "kind": "sealed_commit_invalid",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_store::StoreError::ObjectNotFound(object_id) => json!({
            "kind": "object_not_found",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "object_id": object_id,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        _ => json!({
            "kind": "store_error",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Json(error) => write!(f, "{error}"),
            CliError::Core(error) => write!(f, "{error}"),
            CliError::CoreContext { error, .. } => write!(f, "{error}"),
            CliError::Store(error) => write!(f, "{error}"),
            CliError::StoreContext { error, .. } => write!(f, "{error}"),
            CliError::Usage(message) => write!(f, "{message}"),
            CliError::AuthenticationOrSignature(message) => write!(f, "{message}"),
            CliError::IdempotencyConflict(message) => write!(f, "{message}"),
            CliError::ResourceBudget(error) => write!(f, "{error}"),
            CliError::RequestDeadlineExceeded { phase, limit_ms } => write!(
                f,
                "HTTP request deadline of {limit_ms} ms exceeded during {phase}"
            ),
            CliError::RemoteDiagnostic { message, .. } => write!(f, "{message}"),
            CliError::MigrationIncompatibility(message) => write!(f, "{message}"),
            CliError::VerificationFailed(_) => write!(f, "verification failed"),
            CliError::CommandFailed(_) => write!(f, "command failed"),
            CliError::NotOpen(path) => write!(
                f,
                "not inside an open CodeFire branch directory: {}",
                path.display()
            ),
            CliError::InvalidMarker(message) => {
                write!(f, "invalid open directory marker: {message}")
            }
            CliError::InvalidRepository(message) => {
                write!(f, "invalid CodeFire repository: {message}")
            }
            CliError::BranchReferenceSubjectChanged { subject, message } => write!(
                f,
                "branch reference subject changed at {}: {message}",
                subject.display()
            ),
            CliError::OpenedRegistrySubjectChanged { subject, message } => write!(
                f,
                "opened registry subject changed at {}: {message}",
                subject.display()
            ),
            CliError::OpenSnapshotDivergence(divergence) => write!(f, "{divergence}"),
            CliError::OpenRecoveryPending(pending) => write!(
                f,
                "open recovery is pending for {} at phase {} on branch {}",
                pending.operation, pending.phase, pending.branch
            ),
            CliError::ActiveStateCapability(error) => write!(f, "{error}"),
            CliError::LockContention(message) => write!(f, "{message}"),
            CliError::RepositoryLockContention { message, .. } => write!(f, "{message}"),
            CliError::BranchOperation { message, .. } => write!(f, "{message}"),
            CliError::OpenTargetResolution(error) => write!(f, "{}", error.message),
            CliError::BatchFileRead {
                operation,
                path,
                message,
                ..
            } => write!(
                f,
                "{operation} file read failed: {}: {message}",
                path.display()
            ),
            CliError::EvidenceCommandFailed { message, .. } => write!(f, "{message}"),
            CliError::MissingEvidenceRef { evidence_id, .. } => {
                write!(f, "missing evidence object: {evidence_id}")
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::Json(error)
    }
}

impl From<codefire_core::CoreError> for CliError {
    fn from(error: codefire_core::CoreError) -> Self {
        CliError::Core(error)
    }
}

impl From<codefire_core::DecisionWorkBudgetExceeded> for CliError {
    fn from(error: codefire_core::DecisionWorkBudgetExceeded) -> Self {
        CliError::ResourceBudget(Box::new(error))
    }
}

impl From<codefire_store::StoreError> for CliError {
    fn from(error: codefire_store::StoreError) -> Self {
        CliError::Store(Box::new(error))
    }
}

impl From<active_state::ActiveStateError> for CliError {
    fn from(error: active_state::ActiveStateError) -> Self {
        CliError::ActiveStateCapability(Box::new(error))
    }
}
