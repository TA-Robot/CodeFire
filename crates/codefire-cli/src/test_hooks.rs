//! Failure-injection hooks used by tests; no-ops in normal builds.

use crate::cli_error::CliError;
#[cfg(test)]
use crate::find_repo_root;
#[cfg(test)]
use crate::json_files::write_json_atomic;
#[cfg(test)]
use crate::load_branch_record;
#[cfg(test)]
use crate::read_json;
#[cfg(test)]
use crate::required_string;
#[cfg(test)]
use crate::save_branch_record;
use crate::OpenContext;
use serde_json::Value;
#[cfg(test)]
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
pub(crate) fn maybe_mutate_commit_worktree_for_test(
    open_dir: &Path,
    point: &str,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = COMMIT_WORKTREE_MUTATION
            .lock()
            .expect("commit worktree mutation lock poisoned");
        guard
            .iter()
            .position(|injection| injection.open_dir == open_dir && injection.point == point)
            .map(|index| guard.swap_remove(index))
    };
    if let Some(injection) = injection {
        fs::write(open_dir.join(injection.relative_path), injection.contents)?;
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_mutate_commit_worktree_for_test(
    _open_dir: &Path,
    _point: &str,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_commit_worktree_mutation_for_test(
    open_dir: &Path,
    point: &str,
    relative_path: &Path,
    contents: &[u8],
) {
    let mut guard = COMMIT_WORKTREE_MUTATION
        .lock()
        .expect("commit worktree mutation lock poisoned");
    guard.retain(|injection| injection.open_dir != open_dir || injection.point != point);
    guard.push(CommitWorktreeMutationInjection {
        open_dir: open_dir.to_path_buf(),
        point: point.to_string(),
        relative_path: relative_path.to_path_buf(),
        contents: contents.to_vec(),
    });
}

#[cfg(test)]
pub(crate) fn maybe_fail_commit_transaction_for_test(
    active_state_path: &Path,
    phase: &str,
) -> Result<(), CliError> {
    let mut guard = COMMIT_TRANSACTION_FAILURE_PHASE
        .lock()
        .expect("commit transaction failure phase lock poisoned");
    if let Some(index) = guard.iter().position(|injection| {
        injection.phase == phase && injection.active_state_path == active_state_path
    }) {
        guard.swap_remove(index);
        return Err(CliError::Usage(format!(
            "injected commit transaction failure at {phase}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_fail_commit_transaction_for_test(
    _active_state_path: &Path,
    _phase: &str,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_commit_transaction_failure_for_test(active_state_path: &Path, phase: &str) {
    let mut guard = COMMIT_TRANSACTION_FAILURE_PHASE
        .lock()
        .expect("commit transaction failure phase lock poisoned");
    guard.retain(|injection| {
        injection.active_state_path != active_state_path || injection.phase != phase
    });
    guard.push(CommitTransactionFailureInjection {
        active_state_path: active_state_path.to_path_buf(),
        phase: phase.to_string(),
    });
}

#[cfg(test)]
pub(crate) fn maybe_corrupt_commit_candidate_for_test(
    active_state_path: &Path,
    mut commit: Value,
) -> Value {
    let mut guard = COMMIT_CANDIDATE_CORRUPTION
        .lock()
        .expect("commit candidate corruption lock poisoned");
    let injection = guard
        .as_ref()
        .is_some_and(|injection| injection.active_state_path == active_state_path)
        .then(|| guard.take().expect("matching candidate corruption exists"));
    if let Some(injection) = injection {
        match injection.kind {
            CommitCandidateCorruptionKind::InvalidCertificate => {
                commit["certificate"]["result"] = Value::String("failed".to_string());
            }
            CommitCandidateCorruptionKind::MissingRoot => {
                commit["roots"]
                    .as_object_mut()
                    .expect("commit roots must be an object")
                    .remove("policy");
            }
            CommitCandidateCorruptionKind::WrongRootType => {
                let atom_index = commit["roots"]["atom_index"].clone();
                commit["roots"]["policy"] = atom_index;
            }
            CommitCandidateCorruptionKind::InvalidParent => {
                commit["parents"][0] = Value::String("CF-BLOB-not-a-parent".to_string());
            }
            CommitCandidateCorruptionKind::InvalidSignature => {
                commit["signature"]["signature"] = Value::String("0".repeat(64));
            }
        }
    }
    commit
}

#[cfg(not(test))]
pub(crate) fn maybe_corrupt_commit_candidate_for_test(
    _active_state_path: &Path,
    commit: Value,
) -> Value {
    commit
}

#[cfg(test)]
pub(crate) fn set_commit_candidate_corruption_for_test(
    active_state_path: &Path,
    kind: CommitCandidateCorruptionKind,
) {
    *COMMIT_CANDIDATE_CORRUPTION
        .lock()
        .expect("commit candidate corruption lock poisoned") =
        Some(CommitCandidateCorruptionInjection {
            active_state_path: active_state_path.to_path_buf(),
            kind,
        });
}

#[cfg(test)]
pub(crate) fn maybe_mutate_locked_open_context_for_test(start: &Path) -> Result<(), CliError> {
    let injection = {
        let mut guard = LOCKED_OPEN_CONTEXT_MUTATION
            .lock()
            .expect("locked open context mutation lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.start == start)
        {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        let mut registry = read_json(&injection.registry_path)?;
        let branch_name = required_string(&registry, &["branch", "name"])?;
        registry["open"]["current_base_commit"] = Value::String(injection.base_commit.clone());
        write_json_atomic(&injection.registry_path, &registry)?;
        let repo_root = find_repo_root(start)?;
        let mut branch = load_branch_record(&repo_root, &branch_name)?;
        branch["head"] = Value::String(injection.base_commit);
        save_branch_record(&repo_root, &branch)?;
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_mutate_locked_open_context_for_test(_start: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_locked_open_context_mutation_for_test(
    start: &Path,
    registry_path: &Path,
    base_commit: &str,
) {
    *LOCKED_OPEN_CONTEXT_MUTATION
        .lock()
        .expect("locked open context mutation lock poisoned") = Some(LockedOpenContextMutation {
        start: start.to_path_buf(),
        registry_path: registry_path.to_path_buf(),
        base_commit: base_commit.to_string(),
    });
}

#[cfg(test)]
pub(crate) fn maybe_change_open_target_before_use_for_test(
    requested_path: &Path,
) -> Result<(), CliError> {
    let should_change = {
        let mut guard = OPEN_TARGET_PRE_USE_MUTATION
            .lock()
            .expect("open target pre-use mutation lock poisoned");
        if guard.as_ref().is_some_and(|path| path == requested_path) {
            *guard = None;
            true
        } else {
            false
        }
    };
    if should_change {
        fs::create_dir_all(requested_path)?;
        fs::write(requested_path.join("concurrent-owner.txt"), b"occupied\n")?;
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_change_open_target_before_use_for_test(
    _requested_path: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_open_target_pre_use_mutation_for_test(requested_path: &Path) {
    *OPEN_TARGET_PRE_USE_MUTATION
        .lock()
        .expect("open target pre-use mutation lock poisoned") = Some(requested_path.to_path_buf());
}

#[cfg(all(test, unix))]
pub(crate) fn maybe_insert_nested_open_target_symlink_for_test(
    requested_path: &Path,
    target_path: &Path,
    materialization_path: &Path,
) -> std::io::Result<()> {
    let injection = {
        let mut guard = OPEN_TARGET_NESTED_SYMLINK_INJECTION
            .lock()
            .expect("open target nested symlink injection lock poisoned");
        if guard.as_ref().is_some_and(|injection| {
            injection.requested_path == requested_path && injection.target_path == target_path
        }) {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        fs::create_dir_all(&injection.outside_path)?;
        std::os::unix::fs::symlink(
            &injection.outside_path,
            materialization_path.join(injection.nested_path),
        )?;
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
pub(crate) fn maybe_insert_nested_open_target_symlink_for_test(
    _requested_path: &Path,
    _target_path: &Path,
    _materialization_path: &Path,
) -> std::io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
pub(crate) fn set_open_target_nested_symlink_injection_for_test(
    requested_path: &Path,
    target_path: &Path,
    nested_path: &Path,
    outside_path: &Path,
) {
    *OPEN_TARGET_NESTED_SYMLINK_INJECTION
        .lock()
        .expect("open target nested symlink injection lock poisoned") =
        Some(OpenTargetNestedSymlinkInjection {
            requested_path: requested_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            nested_path: nested_path.to_path_buf(),
            outside_path: outside_path.to_path_buf(),
        });
}

#[cfg(test)]
pub(crate) fn maybe_swap_close_target_before_delete_for_test(
    target: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = CLOSE_TARGET_SWAP_INJECTION
            .lock()
            .expect("close target swap injection lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.target == target)
        {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        fs::rename(&injection.target, &injection.moved_original)?;
        fs::create_dir(&injection.target)?;
        fs::write(injection.target.join("victim.txt"), b"victim\n")?;
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_swap_close_target_before_delete_for_test(
    _target: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_close_target_swap_injection_for_test(target: &Path, moved_original: &Path) {
    *CLOSE_TARGET_SWAP_INJECTION
        .lock()
        .expect("close target swap injection lock poisoned") = Some(CloseTargetSwapInjection {
        target: target.to_path_buf(),
        moved_original: moved_original.to_path_buf(),
    });
}

#[cfg(test)]
pub(crate) fn maybe_fail_state_transaction_for_test(
    active_state_path: &Path,
    operation: &str,
    phase: &str,
) -> Result<(), CliError> {
    let mut guard = STATE_TRANSACTION_FAILURE
        .lock()
        .expect("state transaction failure lock poisoned");
    if let Some(index) = guard.iter().position(|injection| {
        injection.active_state_path == active_state_path
            && injection.operation == operation
            && injection.phase == phase
    }) {
        guard.swap_remove(index);
        return Err(CliError::Usage(format!(
            "injected {operation} state transaction failure at {phase}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
pub(crate) fn maybe_fail_state_transaction_for_test(
    _active_state_path: &Path,
    _operation: &str,
    _phase: &str,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_state_transaction_failure_for_test(
    active_state_path: &Path,
    operation: &str,
    phase: &str,
) {
    let mut guard = STATE_TRANSACTION_FAILURE
        .lock()
        .expect("state transaction failure lock poisoned");
    guard.retain(|injection| {
        injection.active_state_path != active_state_path
            || injection.operation != operation
            || injection.phase != phase
    });
    guard.push(StateTransactionFailureInjection {
        active_state_path: active_state_path.to_path_buf(),
        operation: operation.to_string(),
        phase: phase.to_string(),
    });
}

#[cfg(all(test, unix))]
pub(crate) fn maybe_swap_active_state_after_validation_for_test(
    context: &OpenContext,
) -> Result<(), CliError> {
    use std::os::unix::fs::symlink;

    let injection = {
        let mut guard = ACTIVE_STATE_SWAP_AFTER_VALIDATION
            .lock()
            .expect("active state swap injection lock poisoned");
        if guard.as_ref().is_some_and(|injection| {
            injection.open_dir == context.open_dir
                && injection.active_path == context.active_state.path()
        }) {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };

    fs::rename(&injection.active_path, &injection.moved_path)?;
    if let Err(error) = symlink(&injection.outside_path, &injection.active_path) {
        let _ = fs::rename(&injection.moved_path, &injection.active_path);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
pub(crate) fn maybe_swap_active_state_after_validation_for_test(
    _context: &OpenContext,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
pub(crate) fn set_active_state_swap_after_validation_for_test(
    open_dir: &Path,
    active_path: &Path,
    moved_path: &Path,
    outside_path: &Path,
) {
    *ACTIVE_STATE_SWAP_AFTER_VALIDATION
        .lock()
        .expect("active state swap injection lock poisoned") = Some(ActiveStateSwapInjection {
        open_dir: open_dir.to_path_buf(),
        active_path: active_path.to_path_buf(),
        moved_path: moved_path.to_path_buf(),
        outside_path: outside_path.to_path_buf(),
    });
}

#[cfg(all(test, unix))]
pub(crate) fn maybe_swap_validated_store_before_verify_persist_for_test(
    repo_root: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST
            .lock()
            .expect("validated store swap injection lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.repo_root == repo_root)
        {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    fs::rename(&injection.objects_path, &injection.moved_path)?;
    if let Err(error) = fs::rename(&injection.replacement_path, &injection.objects_path) {
        let _ = fs::rename(&injection.moved_path, &injection.objects_path);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
pub(crate) fn maybe_swap_validated_store_before_verify_persist_for_test(
    _repo_root: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
pub(crate) fn set_validated_store_swap_before_verify_persist_for_test(
    repo_root: &Path,
    objects_path: &Path,
    moved_path: &Path,
    replacement_path: &Path,
) {
    *VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST
        .lock()
        .expect("validated store swap injection lock poisoned") =
        Some(ValidatedStoreSwapInjection {
            repo_root: repo_root.to_path_buf(),
            objects_path: objects_path.to_path_buf(),
            moved_path: moved_path.to_path_buf(),
            replacement_path: replacement_path.to_path_buf(),
        });
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct CommitWorktreeMutationInjection {
    pub(crate) open_dir: PathBuf,
    pub(crate) point: String,
    pub(crate) relative_path: PathBuf,
    pub(crate) contents: Vec<u8>,
}
#[cfg(test)]
pub(crate) static COMMIT_WORKTREE_MUTATION: Mutex<Vec<CommitWorktreeMutationInjection>> =
    Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct CommitTransactionFailureInjection {
    pub(crate) active_state_path: PathBuf,
    pub(crate) phase: String,
}
#[cfg(test)]
pub(crate) static COMMIT_TRANSACTION_FAILURE_PHASE: Mutex<Vec<CommitTransactionFailureInjection>> =
    Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct CommitCandidateCorruptionInjection {
    pub(crate) active_state_path: PathBuf,
    pub(crate) kind: CommitCandidateCorruptionKind,
}
#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum CommitCandidateCorruptionKind {
    InvalidCertificate,
    MissingRoot,
    WrongRootType,
    InvalidParent,
    InvalidSignature,
}
#[cfg(test)]
pub(crate) static COMMIT_CANDIDATE_CORRUPTION: Mutex<Option<CommitCandidateCorruptionInjection>> =
    Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct LockedOpenContextMutation {
    pub(crate) start: PathBuf,
    pub(crate) registry_path: PathBuf,
    pub(crate) base_commit: String,
}
#[cfg(test)]
pub(crate) static LOCKED_OPEN_CONTEXT_MUTATION: Mutex<Option<LockedOpenContextMutation>> =
    Mutex::new(None);
#[cfg(test)]
pub(crate) static OPEN_TARGET_PRE_USE_MUTATION: Mutex<Option<PathBuf>> = Mutex::new(None);
#[cfg(all(test, unix))]
#[derive(Debug)]
pub(crate) struct OpenTargetNestedSymlinkInjection {
    pub(crate) requested_path: PathBuf,
    pub(crate) target_path: PathBuf,
    pub(crate) nested_path: PathBuf,
    pub(crate) outside_path: PathBuf,
}
#[cfg(all(test, unix))]
pub(crate) static OPEN_TARGET_NESTED_SYMLINK_INJECTION: Mutex<
    Option<OpenTargetNestedSymlinkInjection>,
> = Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct CloseTargetSwapInjection {
    pub(crate) target: PathBuf,
    pub(crate) moved_original: PathBuf,
}
#[cfg(test)]
pub(crate) static CLOSE_TARGET_SWAP_INJECTION: Mutex<Option<CloseTargetSwapInjection>> =
    Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct StateTransactionFailureInjection {
    pub(crate) active_state_path: PathBuf,
    pub(crate) operation: String,
    pub(crate) phase: String,
}
#[cfg(test)]
pub(crate) static STATE_TRANSACTION_FAILURE: Mutex<Vec<StateTransactionFailureInjection>> =
    Mutex::new(Vec::new());
#[cfg(all(test, unix))]
#[derive(Debug)]
pub(crate) struct ActiveStateSwapInjection {
    pub(crate) open_dir: PathBuf,
    pub(crate) active_path: PathBuf,
    pub(crate) moved_path: PathBuf,
    pub(crate) outside_path: PathBuf,
}
#[cfg(all(test, unix))]
pub(crate) static ACTIVE_STATE_SWAP_AFTER_VALIDATION: Mutex<Option<ActiveStateSwapInjection>> =
    Mutex::new(None);
#[cfg(all(test, unix))]
#[derive(Debug)]
pub(crate) struct ValidatedStoreSwapInjection {
    pub(crate) repo_root: PathBuf,
    pub(crate) objects_path: PathBuf,
    pub(crate) moved_path: PathBuf,
    pub(crate) replacement_path: PathBuf,
}
#[cfg(all(test, unix))]
pub(crate) static VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST: Mutex<
    Option<ValidatedStoreSwapInjection>,
> = Mutex::new(None);
