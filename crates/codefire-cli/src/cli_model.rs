use crate::exit_code::ExitCode;
use crate::view::{DiffOptions, PatchExportOptions, ReviewPackOptions};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Status {
    pub(crate) branch: String,
    pub(crate) state: String,
    pub(crate) base: String,
    pub(crate) open_fires: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Branch {
    pub(crate) name: String,
    pub(crate) head: String,
    pub(crate) state: String,
}

#[derive(Debug)]
pub(crate) struct InitOptions {
    pub(crate) path: PathBuf,
    pub(crate) force: bool,
}

#[derive(Debug)]
pub(crate) struct InitResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) main_commit: String,
}

#[derive(Debug)]
pub(crate) struct OpenOptions {
    pub(crate) branch: String,
    pub(crate) path: PathBuf,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug)]
pub(crate) struct OpenResult {
    pub(crate) branch: String,
    pub(crate) open_dir: PathBuf,
    pub(crate) plan: Value,
}

#[derive(Debug)]
pub(crate) struct PathJsonOptions {
    pub(crate) path: PathBuf,
    pub(crate) json_output: bool,
    pub(crate) metrics: bool,
    pub(crate) full: bool,
}

#[derive(Debug)]
pub(crate) struct ExtinguishOptions {
    pub(crate) path: PathBuf,
    pub(crate) fire_id: String,
    pub(crate) resolution: String,
    pub(crate) rationale: String,
    pub(crate) evidence: String,
    pub(crate) evidence_refs: Vec<String>,
    pub(crate) refresh: bool,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) edit_rationale: bool,
}

#[derive(Debug)]
pub(crate) struct ExtinguishResult {
    pub(crate) display_id: String,
    pub(crate) fire_uid: String,
    pub(crate) source_atom: String,
    pub(crate) target_atom: String,
    pub(crate) reason: String,
    pub(crate) remaining_open_fire_count: usize,
    pub(crate) plan: Value,
}

#[derive(Debug)]
pub(crate) struct CommitOptions {
    pub(crate) path: PathBuf,
    pub(crate) message: String,
    pub(crate) dry_run: bool,
    pub(crate) full_output: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) signer: Option<String>,
    pub(crate) key_id: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CommitResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) open_dir: PathBuf,
    pub(crate) commit_id: String,
    pub(crate) branch: String,
    pub(crate) dry_run: bool,
    pub(crate) blocked: bool,
    pub(crate) exit_code: ExitCode,
    pub(crate) plan: Value,
}

#[derive(Debug)]
pub(crate) struct CloneOptions {
    pub(crate) source: String,
    pub(crate) new_branch: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CloneResult {
    pub(crate) plan: Value,
}

#[derive(Debug)]
pub(crate) struct DiffArgs {
    pub(crate) left: String,
    pub(crate) right: String,
    pub(crate) diff: DiffOptions,
}

#[derive(Debug)]
pub(crate) struct ReviewPackArgs {
    pub(crate) review: ReviewPackOptions,
    pub(crate) output: Option<PathBuf>,
    pub(crate) json_output: bool,
}

#[derive(Debug)]
pub(crate) struct PatchExportArgs {
    pub(crate) patch: PatchExportOptions,
    pub(crate) output: Option<PathBuf>,
    pub(crate) json_output: bool,
}

#[derive(Debug)]
pub(crate) struct PatchImportOptions {
    pub(crate) path: PathBuf,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ServeOptions {
    pub(crate) storage_root: PathBuf,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) tls_cert: Option<PathBuf>,
    pub(crate) tls_key: Option<PathBuf>,
    pub(crate) tls_client_ca: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct MergeOptions {
    pub(crate) source_branch: String,
    pub(crate) target_branch: String,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct LockOptions {
    pub(crate) wait: bool,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Debug)]
pub(crate) struct MergeResult {
    pub(crate) source_branch: String,
    pub(crate) target_branch: String,
    pub(crate) source_head: String,
    pub(crate) target_head: String,
    pub(crate) base: String,
    pub(crate) target_dir: PathBuf,
    pub(crate) file_actions: Vec<MergeFileAction>,
    pub(crate) conflicts: Vec<String>,
    pub(crate) binary_conflicts: Vec<BinaryMergeConflict>,
    pub(crate) semantic_conflicts: Vec<SemanticConflictCandidate>,
    pub(crate) dry_run: bool,
    pub(crate) applied: bool,
}

#[derive(Debug)]
pub(crate) struct MergeFileAction {
    pub(crate) path: String,
    pub(crate) action: MergeAction,
    pub(crate) content: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinaryMergeConflict {
    pub(crate) path: String,
    pub(crate) target_path: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) target_bytes: Option<usize>,
    pub(crate) source_bytes: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeAction {
    WriteSource,
    DeleteTarget,
    WriteConflictMarkers,
    WriteConflictSide,
}

impl MergeAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::WriteSource => "write_source",
            Self::DeleteTarget => "delete_target",
            Self::WriteConflictMarkers => "write_conflict_markers",
            Self::WriteConflictSide => "write_conflict_side",
        }
    }
}

#[derive(Debug)]
pub(crate) struct SemanticConflictCandidate {
    pub(crate) atom_id: String,
    pub(crate) base_hash: Option<String>,
    pub(crate) source_hash: Option<String>,
    pub(crate) target_hash: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) target_path: Option<String>,
}

pub(crate) struct OpenContext {
    pub(crate) repo_root: PathBuf,
    pub(crate) open_dir: PathBuf,
    pub(crate) branch: String,
    pub(crate) registry_path: PathBuf,
    pub(crate) registry: Value,
}

pub(crate) struct ScanExecution {
    pub(crate) context: OpenContext,
    pub(crate) active_state_path: PathBuf,
    pub(crate) scan: codefire_core::ScanResult,
    pub(crate) fires: Vec<codefire_core::Fire>,
}

pub(crate) struct VerifyExecution {
    pub(crate) scan: codefire_core::ScanResult,
    pub(crate) verification: codefire_core::Verification,
}
