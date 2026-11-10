//! Locating and verifying the open directory a command runs in.

use crate::cli_error::{CliError, OpenRecoveryPendingError};
use crate::commit::validate_commit_transaction_record;
use crate::fire::active_ledger_transaction::validate_active_ledger_descriptor;
use crate::json_files::{read_active_json_optional, read_active_json_required};
use crate::state_transaction::validate_state_transaction_record;
use crate::{
    active_state, branch_record_path, find_open_marker, find_repo_root, open_snapshot,
    opened_registry_path, read_json, ref_file_name, required_string, OpenContext,
    OpenContextAuthority,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) fn optional_repo_root(start: &Path) -> Option<PathBuf> {
    find_repo_root(start).ok()
}

pub(crate) fn validated_active_state_dir(
    repo_root: &Path,
    registry: &Value,
) -> Result<active_state::ActiveStateDir, CliError> {
    let open_instance_id = required_string(registry, &["open", "open_instance_id"])?;
    let recorded = PathBuf::from(required_string(registry, &["open", "active_state_path"])?);
    let expected = repo_root
        .join(".codefire")
        .join("active")
        .join(ref_file_name(&open_instance_id));
    if recorded != expected {
        return Err(CliError::InvalidMarker(format!(
            "active_state_path does not match repository/open instance: expected {}, found {}",
            expected.display(),
            recorded.display()
        )));
    }
    Ok(active_state::ActiveStateDir::bind(&expected)?)
}

pub(crate) fn open_context_for_status(start: &Path) -> Result<OpenContext, CliError> {
    if let Some(context) = legacy_python_status_context(start)? {
        return Ok(context);
    }
    open_context(start)
}

/// The Python fallback historically emitted only marker, opened registry,
/// object store, and active fire data. Keep that exact read-only status shape
/// readable without weakening the current snapshot required by mutations.
pub(crate) fn legacy_python_status_context(start: &Path) -> Result<Option<OpenContext>, CliError> {
    let Some(marker_path) = find_open_marker(start)? else {
        return Ok(None);
    };
    let marker = read_json(&marker_path)?;
    let open_dir = marker_path
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("marker has no parent directory".to_string()))?
        .to_path_buf();
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repo_root = repo_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()))?;
    let branch = required_string(&marker, &["branch", "name"])?;
    let registry_path = opened_registry_path(&repo_root, &branch);
    let registry = read_json(&registry_path)?;
    let active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let is_exact_legacy_shape = !repo_root.join(".codefire").join("repo.json").exists()
        && !branch_record_path(&repo_root, &branch).exists()
        && !active_state_path.join("state.json").exists();
    if !is_exact_legacy_shape {
        return Ok(None);
    }

    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    if registry_open_path != open_dir {
        return Err(CliError::InvalidMarker(
            "opened_path does not match registry".to_string(),
        ));
    }
    let marker_open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    if marker_open_instance_id != registry_open_instance_id {
        return Err(CliError::InvalidMarker(
            "open_instance_id does not match registry".to_string(),
        ));
    }
    let active_state = validated_active_state_dir(&repo_root, &registry)?;
    Ok(Some(OpenContext {
        repo_root,
        open_dir,
        branch,
        registry_path,
        active_state,
        registry,
        authority: OpenContextAuthority::LegacyPythonStatus,
    }))
}

pub(crate) fn open_context(start: &Path) -> Result<OpenContext, CliError> {
    open_context_with_recovery(start, None)
}

pub(crate) fn open_context_for_recovery(
    start: &Path,
    operation: &str,
) -> Result<OpenContext, CliError> {
    open_context_with_recovery(start, Some(operation))
}

pub(crate) fn open_context_with_recovery(
    start: &Path,
    allowed_operation: Option<&str>,
) -> Result<OpenContext, CliError> {
    if legacy_python_status_context(start)?.is_some() {
        return Err(CliError::InvalidRepository(
            "legacy Python-compatible open metadata is read-only; use status or migrate before mutation"
                .to_string(),
        ));
    }
    let context = load_verified_open_context(start)?;
    let pending = match &context.authority {
        OpenContextAuthority::Verified(snapshot) => match &snapshot.disposition {
            open_snapshot::OpenSnapshotDisposition::Steady => None,
            open_snapshot::OpenSnapshotDisposition::PendingCommit { phase } => {
                Some(("commit".to_string(), phase.clone()))
            }
            open_snapshot::OpenSnapshotDisposition::PendingState { operation, phase } => {
                Some((operation.clone(), phase.clone()))
            }
            open_snapshot::OpenSnapshotDisposition::PendingActiveLedger {
                operation,
                phase,
                ..
            } => Some((operation.clone(), phase.clone())),
        },
        OpenContextAuthority::LegacyPythonStatus => None,
    };
    let Some((operation, phase)) = pending else {
        return Ok(context);
    };
    if allowed_operation == Some(operation.as_str()) {
        return Ok(context);
    }
    Err(CliError::OpenRecoveryPending(Box::new(
        OpenRecoveryPendingError {
            operation,
            phase,
            repo_root: context.repo_root,
            branch: context.branch,
            open_dir: context.open_dir,
        },
    )))
}

pub(crate) fn load_verified_open_context(start: &Path) -> Result<OpenContext, CliError> {
    let marker_path =
        find_open_marker(start)?.ok_or_else(|| CliError::NotOpen(start.to_path_buf()))?;
    let marker = read_json(&marker_path)?;
    let open_dir = marker_path
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("marker has no parent directory".to_string()))?
        .to_path_buf();
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repo_root = repo_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()))?;
    let branch = required_string(&marker, &["branch", "name"])?;
    let repository_record_path = repo_root.join(".codefire").join("repo.json");
    let repository = read_json(&repository_record_path)?;
    let branch_record_path = branch_record_path(&repo_root, &branch);
    let branch_record = read_json(&branch_record_path)?;
    let registry_path = opened_registry_path(&repo_root, &branch);
    let registry = read_json(&registry_path)?;
    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let marker_open_path = PathBuf::from(required_string(&marker, &["open", "opened_path"])?);
    let marker_open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    let registry_active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let expected_active_state_path = repo_root
        .join(".codefire")
        .join("active")
        .join(ref_file_name(&marker_open_instance_id));
    let expected_repository_path = repo_root.join(".codefire");
    let marker_repository_path = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repository_id = required_string(&repository, &["repository_id"])?;
    let marker_repository_id = required_string(&marker, &["repository", "repository_id"])?;
    let marker_branch = required_string(&marker, &["branch", "name"])?;
    let registry_branch = required_string(&registry, &["branch", "name"])?;
    let branch_record_name = required_string(&branch_record, &["name"])?;
    let marker_opened_from_commit = required_string(&marker, &["branch", "opened_from_commit"])?;
    let registry_opened_from_commit = required_string(&registry, &["open", "opened_from_commit"])?;
    let branch_head = required_string(&branch_record, &["head"])?;
    let registry_current_base = required_string(&registry, &["open", "current_base_commit"])?;

    let locator = open_snapshot::OpenSnapshotIdentityAuthorities {
        repo_root: &repo_root,
        repository_record_path: &repository_record_path,
        marker_path: &marker_path,
        branch_record_path: &branch_record_path,
        registry_path: &registry_path,
        expected_repository_path: &expected_repository_path,
        marker_repository_path: &marker_repository_path,
        repository_id: &repository_id,
        marker_repository_id: &marker_repository_id,
        branch: &branch,
        marker_branch: &marker_branch,
        registry_branch: &registry_branch,
        branch_record_name: &branch_record_name,
        open_dir: &open_dir,
        marker_open_path: &marker_open_path,
        registry_open_path: &registry_open_path,
        marker_open_instance_id: &marker_open_instance_id,
        registry_open_instance_id: &registry_open_instance_id,
        marker_opened_from_commit: &marker_opened_from_commit,
        registry_opened_from_commit: &registry_opened_from_commit,
        expected_active_state_path: &expected_active_state_path,
        registry_active_state_path: &registry_active_state_path,
        branch_head: &branch_head,
        registry_current_base: &registry_current_base,
        pending_commit_transaction: None,
    };
    open_snapshot::verify_open_snapshot_locator(&locator)
        .map_err(CliError::OpenSnapshotDivergence)?;
    validate_open_snapshot_commit(&repo_root, &branch_head)?;
    if registry_current_base != branch_head {
        validate_open_snapshot_commit(&repo_root, &registry_current_base)?;
    }

    let active_state = validated_active_state_dir(&repo_root, &registry)?;
    let active_state_record_path = active_state.display_path(Path::new("state.json"));
    let active_state_record = read_active_json_required(&active_state, Path::new("state.json"))?;
    let pending_commit_transaction =
        read_active_json_optional(&active_state, Path::new("commit_transaction.json"))?;
    let pending_state_transaction =
        read_active_json_optional(&active_state, Path::new("state_transaction.json"))?;
    let active_ledger_descriptor = validate_active_ledger_descriptor(&active_state)?;
    if let Some(transaction) = pending_commit_transaction.as_ref() {
        validate_commit_transaction_record(transaction).map_err(|message| {
            CliError::InvalidRepository(format!(
                "{message}: {}",
                active_state
                    .display_path(Path::new("commit_transaction.json"))
                    .display()
            ))
        })?;
    }
    if let Some(transaction) = pending_state_transaction.as_ref() {
        validate_state_transaction_record(transaction).map_err(|message| {
            CliError::InvalidRepository(format!(
                "{message}: {}",
                active_state
                    .display_path(Path::new("state_transaction.json"))
                    .display()
            ))
        })?;
    }
    let branch_state = required_string(&branch_record, &["state"])?;
    let registry_state = required_string(&registry, &["state", "last_known"])?;
    let active_state_value = required_string(&active_state_record, &["state"])?;
    let identity = open_snapshot::OpenSnapshotIdentityAuthorities {
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        ..locator
    };
    let state = open_snapshot::OpenSnapshotStateAuthorities {
        active_state_record_path: &active_state_record_path,
        branch_state: &branch_state,
        registry_state: &registry_state,
        active_state: &active_state_value,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        pending_state_transaction: pending_state_transaction.as_ref(),
        active_ledger_descriptor: active_ledger_descriptor.as_ref(),
    };
    let verified_snapshot = open_snapshot::verify_open_snapshot(&identity, &state)
        .map_err(CliError::OpenSnapshotDivergence)?;
    Ok(OpenContext {
        repo_root: verified_snapshot.repo_root.clone(),
        open_dir: verified_snapshot.open_dir.clone(),
        branch: verified_snapshot.branch.clone(),
        registry_path,
        active_state,
        registry,
        authority: OpenContextAuthority::Verified(Box::new(verified_snapshot)),
    })
}

pub(crate) fn validate_open_snapshot_commit(
    repo_root: &Path,
    commit_id: &str,
) -> Result<(), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    let path =
        codefire_store::object_record_path(&objects, commit_id).unwrap_or_else(|| objects.clone());
    codefire_store::validate_sealed_commit(&objects, commit_id).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: repo_root.to_path_buf(),
            path,
        }
    })
}
