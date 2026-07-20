use super::*;
use crate::close_worktree::ClosableWorktree;
use crate::secure_worktree::StableFsIdentity;
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{File as CapFile, OpenOptions as CapOpenOptions};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::io::{ErrorKind, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const CLOSE_TRANSACTION_TYPE: &str = "codefire_close_transaction";
const CLOSE_TRANSACTION_VERSION: u64 = 1;
const CLOSE_TRANSACTION_FILE: &str = "close_transaction.json";
pub(crate) const CLOSE_TRANSACTION_BYTE_LIMIT: u64 = 1024 * 1024;
const NEW_MARKER_ATTEMPTS: usize = 128;
static NEW_MARKER_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClosePhase {
    Prepared,
    WorktreeApplied,
    ActiveStateRemoved,
    RegistryRemoved,
    BranchWritten,
}

impl ClosePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::WorktreeApplied => "worktree_applied",
            Self::ActiveStateRemoved => "active_state_removed",
            Self::RegistryRemoved => "registry_removed",
            Self::BranchWritten => "branch_written",
        }
    }

    fn parse(value: &str, path: &Path) -> Result<Self, CliError> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "worktree_applied" => Ok(Self::WorktreeApplied),
            "active_state_removed" => Ok(Self::ActiveStateRemoved),
            "registry_removed" => Ok(Self::RegistryRemoved),
            "branch_written" => Ok(Self::BranchWritten),
            _ => Err(invalid_transaction(path, "unsupported phase")),
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Prepared => Some(Self::WorktreeApplied),
            Self::WorktreeApplied => Some(Self::ActiveStateRemoved),
            Self::ActiveStateRemoved => Some(Self::RegistryRemoved),
            Self::RegistryRemoved => Some(Self::BranchWritten),
            Self::BranchWritten => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemovalArtifact {
    Before,
    Quarantined,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryArtifact {
    Exact,
    Quarantined,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchArtifact {
    Before,
    BeforeWithAfterObject,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CloseEvidence {
    worktree: RemovalArtifact,
    active_state: RemovalArtifact,
    registry: RegistryArtifact,
    branch: BranchArtifact,
}

#[derive(Debug)]
struct CloseTransaction {
    path: PathBuf,
    record: Value,
}

struct PreparedClose {
    repo_root: PathBuf,
    repository_id: String,
    discard: bool,
    keep_dir: bool,
    branch_before: Value,
    branch_reference_identity: StableFsIdentity,
    branch_parent_identity: StableFsIdentity,
    registry: Value,
    marker: Value,
    open_dir: PathBuf,
    active_state_path: PathBuf,
    worktree_identity: StableFsIdentity,
    worktree_parent_identity: StableFsIdentity,
    marker_identity: StableFsIdentity,
    active_state_identity: StableFsIdentity,
    active_state_parent_identity: StableFsIdentity,
    registry_identity: StableFsIdentity,
    registry_parent_identity: StableFsIdentity,
    plan: Value,
}

pub(crate) fn close_transaction_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".codefire").join(CLOSE_TRANSACTION_FILE)
}

pub(crate) fn reject_pending_for_other_mutation(repo_root: &Path) -> Result<(), CliError> {
    if let Some(transaction) = CloseTransaction::load(repo_root)? {
        return Err(CliError::InvalidRepository(format!(
            "close transaction is pending at phase {} for branch {}; rerun the exact close/discard request before another mutation: {}",
            transaction.phase()?.as_str(),
            transaction.branch_name()?,
            transaction.path.display()
        )));
    }
    Ok(())
}

pub(crate) fn doctor_close_transaction_summary(
    repo_root: &Path,
) -> Result<Option<String>, CliError> {
    let Some(transaction) = CloseTransaction::load(repo_root)? else {
        return Ok(None);
    };
    let evidence = inspect_evidence(&transaction)?;
    validate_phase_evidence(transaction.phase()?, evidence, &transaction.path)?;
    Ok(Some(format!(
        "close transaction is pending at phase {} for branch {} worktree {}; evidence worktree={:?} active_state={:?} registry={:?} branch={:?}",
        transaction.phase()?.as_str(),
        transaction.branch_name()?,
        path_field(
            &transaction.record,
            &["worktree", "path"],
            &transaction.path
        )?
        .display(),
        evidence.worktree,
        evidence.active_state,
        evidence.registry,
        evidence.branch,
    )))
}

pub(crate) fn close_branch_from(
    start: &Path,
    options: &CloseOptions,
) -> Result<CloseResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    let repository = read_json(&repo_root.join(".codefire").join("repo.json"))?;
    let repository_id = required_string(&repository, &["repository_id"])?;

    let pending_close = CloseTransaction::load(&repo_root)?;
    let pending_open = open_transaction::doctor_open_transaction_summary(&repo_root)?;
    if pending_close.is_some() && pending_open.is_some() {
        return Err(CliError::InvalidRepository(format!(
            "repository contains simultaneous pending open and close transactions; neither may recover until the conflicting authority is inspected: {}, {}",
            open_transaction::open_transaction_path(&repo_root).display(),
            close_transaction_path(&repo_root).display()
        )));
    }
    if let Some(transaction) = pending_close {
        transaction.verify_request(&repo_root, &repository_id, options)?;
        let evidence = inspect_evidence(&transaction)?;
        validate_phase_evidence(transaction.phase()?, evidence, &transaction.path)?;
        if options.dry_run {
            return Err(CliError::InvalidRepository(format!(
                "close transaction is pending at phase {}; rerun the exact {} request without --dry-run to recover it: {}",
                transaction.phase()?.as_str(),
                string(&transaction.record, &["request", "command"], &transaction.path)?,
                transaction.path.display()
            )));
        }
        return resume_transaction(transaction);
    }
    if let Some(summary) = pending_open {
        return Err(CliError::InvalidRepository(format!(
            "cannot start close/discard while an open transaction is pending: {summary}"
        )));
    }
    if is_primary_branch(&options.branch) {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.branch),
            None,
            format!(
                "refusing to close primary branch {}; use feature branches for close/discard",
                options.branch
            ),
        ));
    }
    if !options.dry_run {
        if let Some(completed) = CloseTransaction::load_completed(&repo_root)? {
            completed.cleanup_pending_authority_from_archive()?;
            let same_request = completed
                .verify_request(&repo_root, &repository_id, options)
                .is_ok();
            let final_evidence = inspect_evidence(&completed)
                .and_then(|evidence| {
                    validate_phase_evidence(ClosePhase::BranchWritten, evidence, &completed.path)
                })
                .is_ok();
            if same_request && final_evidence {
                return completed.result();
            }
            retire_completed_archive(&completed)?;
        }
    }
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    let mut registry = match read_json(&registry_path) {
        Ok(registry) => registry,
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
            return Err(branch_operation_error(
                &repo_root,
                Some(&options.branch),
                None,
                format!("branch is not open: {}", options.branch),
            ));
        }
        Err(error) => return Err(error),
    };
    let open_dir = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let close_worktree = ClosableWorktree::bind(&open_dir).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to bind open directory {} for close/discard: {error}",
            open_dir.display()
        ))
    })?;
    let bound_marker = close_worktree.read_marker().map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to read bounded no-follow open marker at {}: {error}",
            open_dir.display()
        ))
    })?;
    let marker: Value = serde_json::from_slice(bound_marker.contents())?;
    validate_close_open_marker(&repo_root, &open_dir, &options.branch, &registry, &marker)?;
    // Close/discard has its own repository-oriented entry point, but it must
    // consume the same verified open authority as worktree commands before
    // preparing any destructive transaction.
    let verified_context = open_context(&open_dir)?;
    if verified_context.repo_root != repo_root || verified_context.branch != options.branch {
        return Err(CliError::InvalidRepository(format!(
            "verified open context changed while preparing close/discard for {}",
            options.branch
        )));
    }
    drop(verified_context);
    let active_state = validated_active_state_dir(&repo_root, &registry)?;
    reject_pending_active_transactions(&active_state)?;
    let state = if options.dry_run && !options.discard {
        let scan = compute_scan(&open_dir)?;
        scan_branch_state(scan_change_count(&scan.scan), scan.scan.open_fires.len()).to_string()
    } else {
        if !options.discard {
            refresh_open_clean_for_branch_operation(
                &repo_root,
                &options.branch,
                &open_dir,
                lock.as_ref().expect("live close must hold repo lock"),
            )?;
            registry = read_json(&registry_path)?;
        }
        required_string(&registry, &["state", "last_known"])?
    };
    if state != "open-clean" && !options.discard {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.branch),
            Some(&open_dir),
            format!(
                "branch {} is {state}; use --discard to close and discard unsealed open work",
                options.branch
            ),
        ));
    }
    if !matches!(
        state.as_str(),
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported open state for close transaction: {state}"
        )));
    }
    active_state.ensure_current()?;
    let active_state_path = active_state.path().to_path_buf();
    let plan = close_operation_plan(
        options,
        &repo_root,
        &open_dir,
        &registry_path,
        &active_state_path,
        &state,
    );
    if options.dry_run {
        return Ok(CloseResult {
            branch: options.branch.clone(),
            open_dir,
            plan,
        });
    }
    if !options.keep_dir && open_dir == repo_root {
        return Err(CliError::Usage(format!(
            "refusing to remove repository root {}; rerun with --keep-dir",
            open_dir.display()
        )));
    }
    let branch_before = load_branch_record(&repo_root, &options.branch)?;
    if branch_state(&branch_before) != state {
        return Err(CliError::InvalidRepository(format!(
            "branch and opened registry state diverge before close: branch={}, registry={state}",
            branch_state(&branch_before)
        )));
    }
    let (registry_identity, registry_parent_identity) =
        exact_json_file_identities(&registry_path, &registry, &registry_path)?;
    let branch_path = branch_record_path(&repo_root, &options.branch);
    let (branch_reference_identity, branch_parent_identity) =
        exact_json_file_identities(&branch_path, &branch_before, &branch_path)?;
    let transaction = CloseTransaction::new(PreparedClose {
        repo_root: repo_root.clone(),
        repository_id,
        discard: options.discard,
        keep_dir: options.keep_dir,
        branch_before,
        branch_reference_identity,
        branch_parent_identity,
        registry,
        marker,
        open_dir: open_dir.clone(),
        active_state_path,
        worktree_identity: close_worktree.identity()?,
        worktree_parent_identity: close_worktree.parent_identity()?,
        marker_identity: bound_marker.identity()?,
        active_state_identity: active_state.identity()?,
        active_state_parent_identity: active_state.parent_identity()?,
        registry_identity,
        registry_parent_identity,
        plan,
    })?;
    transaction.save_new()?;
    maybe_fail_close_transaction_for_test(&repo_root, "after_prepared")?;
    maybe_swap_close_target_before_delete_for_test(&open_dir)?;
    drop(bound_marker);
    drop(close_worktree);
    drop(active_state);
    resume_transaction(transaction)
}

fn reject_pending_active_transactions(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    if let Some(descriptor) =
        crate::fire::active_ledger_transaction::validate_active_ledger_descriptor(active_state)?
    {
        if descriptor.publication_status
            != crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released
        {
            return Err(CliError::InvalidRepository(format!(
                "pending active-ledger transaction {}/{} at phase {} must be recovered by its exact owning operation before close/discard: {}",
                descriptor.operation,
                descriptor.request_hash,
                descriptor.phase,
                active_state
                    .display_path(Path::new(
                        crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE,
                    ))
                    .display()
            )));
        }
    }
    if active_state.exists(Path::new("commit_transaction.json"))? {
        reject_pending_commit_transaction_for_preview(active_state)?;
    }
    if active_state.exists(Path::new("state_transaction.json"))? {
        validate_pending_state_transaction_for_recovery(active_state)?;
        return Err(CliError::InvalidRepository(format!(
            "pending state transaction must be recovered by rerunning its exact scan/verify operation before close/discard: {}",
            active_state.display_path(Path::new("state_transaction.json")).display()
        )));
    }
    Ok(())
}

fn completed_transaction_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("close_transaction.completed.json")
}

fn retiring_completed_transaction_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codefire")
        .join(".codefire-close-transaction-completed-retiring")
}

impl CloseTransaction {
    fn new(prepared: PreparedClose) -> Result<Self, CliError> {
        let path = close_transaction_path(&prepared.repo_root);
        let transaction_id = random_transaction_id()?;
        let suffix = transaction_id
            .strip_prefix("close_tx_")
            .ok_or_else(|| invalid_transaction(&path, "invalid generated transaction id"))?;
        let branch_name = required_string(&prepared.branch_before, &["name"])?;
        let mut branch_after = prepared.branch_before.clone();
        branch_after["state"] = Value::String("closed".to_string());
        let branch_object_id = codefire_store::object_id("branch", &branch_after)?;
        let worktree_action = if prepared.keep_dir {
            "remove_marker"
        } else {
            "remove_tree"
        };
        let command = if prepared.discard { "discard" } else { "close" };
        let started_at = now_iso_utc();
        let result = json!({
            "branch": branch_name,
            "open_dir": &prepared.open_dir,
            "plan": &prepared.plan,
        });
        let record = json!({
            "type": CLOSE_TRANSACTION_TYPE,
            "version": CLOSE_TRANSACTION_VERSION,
            "status": "pending",
            "transaction_id": transaction_id,
            "phase": ClosePhase::Prepared.as_str(),
            "repository": {
                "path": &prepared.repo_root,
                "repository_id": &prepared.repository_id,
            },
            "request": {
                "command": command,
                "branch": branch_name,
                "discard": prepared.discard,
                "keep_dir": prepared.keep_dir,
            },
            "branch": {
                "name": branch_name,
                "before": prepared.branch_before,
                "after": branch_after,
                "object_id": branch_object_id,
                "reference_identity": identity_json(prepared.branch_reference_identity),
                "parent_identity": identity_json(prepared.branch_parent_identity),
            },
            "worktree": {
                "path": &prepared.open_dir,
                "action": worktree_action,
                "identity": identity_json(prepared.worktree_identity),
                "parent_identity": identity_json(prepared.worktree_parent_identity),
                "marker": prepared.marker,
                "marker_identity": identity_json(prepared.marker_identity),
                "quarantine_name": format!(".codefire-close-worktree-{suffix}"),
                "marker_quarantine_name": format!(".codefire-close-marker-{suffix}"),
            },
            "active_state": {
                "path": &prepared.active_state_path,
                "identity": identity_json(prepared.active_state_identity),
                "parent_identity": identity_json(prepared.active_state_parent_identity),
                "quarantine_name": format!(".codefire-active-remove-{suffix}"),
            },
            "registry": {
                "path": opened_registry_path(&prepared.repo_root, &branch_name),
                "record": prepared.registry,
                "identity": identity_json(prepared.registry_identity),
                "parent_identity": identity_json(prepared.registry_parent_identity),
                "quarantine_name": format!(".codefire-close-registry-{suffix}"),
            },
            "result": result,
            "started_at": &started_at,
            "updated_at": &started_at,
        });
        let transaction = Self { path, record };
        transaction.validate(&prepared.repo_root, "pending")?;
        Ok(transaction)
    }

    fn load(repo_root: &Path) -> Result<Option<Self>, CliError> {
        let path = close_transaction_path(repo_root);
        let parent_path = path
            .parent()
            .ok_or_else(|| invalid_transaction(&path, "transaction marker has no parent"))?;
        let marker_name = path
            .file_name()
            .ok_or_else(|| invalid_transaction(&path, "transaction marker has no name"))?;
        let expected_parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
        secure_worktree::ensure_absolute_dir_identity(&expected_parent, parent_path)?;
        let bytes = match read_bytes_from_capability(
            &expected_parent,
            marker_name,
            CLOSE_TRANSACTION_BYTE_LIMIT,
        ) {
            Ok(bytes) => bytes,
            Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                secure_worktree::ensure_absolute_dir_identity(&expected_parent, parent_path)?;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        secure_worktree::ensure_absolute_dir_identity(&expected_parent, parent_path)?;
        let record: Value = serde_json::from_slice(&bytes).map_err(|error| {
            invalid_transaction(&path, &format!("marker is not valid JSON: {error}"))
        })?;
        let transaction = Self { path, record };
        transaction.validate(repo_root, "pending")?;
        Ok(Some(transaction))
    }

    fn load_completed(repo_root: &Path) -> Result<Option<Self>, CliError> {
        let primary = completed_transaction_path(repo_root);
        let retiring = retiring_completed_transaction_path(repo_root);
        let parent_path = primary
            .parent()
            .ok_or_else(|| invalid_transaction(&primary, "completed archive has no parent"))?
            .to_path_buf();
        let primary_name = primary
            .file_name()
            .ok_or_else(|| invalid_transaction(&primary, "completed archive has no name"))?
            .to_os_string();
        let retiring_name = retiring
            .file_name()
            .ok_or_else(|| invalid_transaction(&retiring, "retiring archive has no name"))?
            .to_os_string();
        let parent = secure_worktree::open_absolute_dir_nofollow(&parent_path)?;
        secure_worktree::ensure_absolute_dir_identity(&parent, &parent_path)?;
        let (path, bytes) = match read_bytes_from_capability(
            &parent,
            &primary_name,
            CLOSE_TRANSACTION_BYTE_LIMIT,
        ) {
            Ok(bytes) => {
                match read_bytes_from_capability(
                    &parent,
                    &retiring_name,
                    CLOSE_TRANSACTION_BYTE_LIMIT,
                ) {
                    Ok(_) => {
                        return Err(invalid_transaction(
                            &primary,
                            "both completed archive and retiring archive exist",
                        ));
                    }
                    Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
                (primary, bytes)
            }
            Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                match read_bytes_from_capability(
                    &parent,
                    &retiring_name,
                    CLOSE_TRANSACTION_BYTE_LIMIT,
                ) {
                    Ok(bytes) => (retiring, bytes),
                    Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                        secure_worktree::ensure_absolute_dir_identity(&parent, &parent_path)?;
                        return Ok(None);
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        };
        secure_worktree::ensure_absolute_dir_identity(&parent, &parent_path)?;
        let record: Value = serde_json::from_slice(&bytes).map_err(|error| {
            invalid_transaction(
                &path,
                &format!("completed archive is not valid JSON: {error}"),
            )
        })?;
        let transaction = Self { path, record };
        transaction.validate(repo_root, "completed")?;
        if transaction.phase()? != ClosePhase::BranchWritten {
            return Err(invalid_transaction(
                &transaction.path,
                "completed archive is not at the terminal phase",
            ));
        }
        string(&transaction.record, &["completed_at"], &transaction.path)?;
        Ok(Some(transaction))
    }

    fn validate(&self, repo_root: &Path, expected_status: &str) -> Result<(), CliError> {
        let record = &self.record;
        if string(record, &["type"], &self.path)? != CLOSE_TRANSACTION_TYPE
            || u64_field(record, &["version"], &self.path)? != CLOSE_TRANSACTION_VERSION
            || string(record, &["status"], &self.path)? != expected_status
        {
            return Err(invalid_transaction(
                &self.path,
                "type, version, or status is unsupported",
            ));
        }
        self.phase()?;
        let transaction_id = string(record, &["transaction_id"], &self.path)?;
        validate_transaction_id(&transaction_id, &self.path)?;
        for field in ["started_at", "updated_at"] {
            if string(record, &[field], &self.path)?.is_empty() {
                return Err(invalid_transaction(
                    &self.path,
                    "empty transaction timestamp",
                ));
            }
        }
        if path_field(record, &["repository", "path"], &self.path)? != repo_root
            || string(record, &["repository", "repository_id"], &self.path)?.is_empty()
        {
            return Err(invalid_transaction(
                &self.path,
                "repository identity does not match marker location",
            ));
        }
        let branch_name = string(record, &["branch", "name"], &self.path)?;
        if string(record, &["request", "branch"], &self.path)? != branch_name {
            return Err(invalid_transaction(
                &self.path,
                "request and branch subjects disagree",
            ));
        }
        let discard = bool_field(record, &["request", "discard"], &self.path)?;
        let keep_dir = bool_field(record, &["request", "keep_dir"], &self.path)?;
        let command = string(record, &["request", "command"], &self.path)?;
        if command != if discard { "discard" } else { "close" } {
            return Err(invalid_transaction(
                &self.path,
                "request command and discard mode disagree",
            ));
        }
        let before = value(record, &["branch", "before"], &self.path)?;
        let after = value(record, &["branch", "after"], &self.path)?;
        if required_string(before, &["name"])? != branch_name
            || !matches!(
                branch_state(before),
                "open-clean" | "open-burning" | "open-consistent"
            )
        {
            return Err(invalid_transaction(&self.path, "invalid branch preimage"));
        }
        let mut expected_after = before.clone();
        expected_after["state"] = Value::String("closed".to_string());
        if after != &expected_after
            || string(record, &["branch", "object_id"], &self.path)?
                != codefire_store::object_id("branch", after)?
        {
            return Err(invalid_transaction(&self.path, "invalid branch postimage"));
        }
        identity_field(record, &["branch", "reference_identity"], &self.path)?;
        identity_field(record, &["branch", "parent_identity"], &self.path)?;
        let open_dir = path_field(record, &["worktree", "path"], &self.path)?;
        let action = string(record, &["worktree", "action"], &self.path)?;
        if action
            != if keep_dir {
                "remove_marker"
            } else {
                "remove_tree"
            }
        {
            return Err(invalid_transaction(
                &self.path,
                "worktree action and keep-dir mode disagree",
            ));
        }
        identity_field(record, &["worktree", "identity"], &self.path)?;
        identity_field(record, &["worktree", "parent_identity"], &self.path)?;
        identity_field(record, &["worktree", "marker_identity"], &self.path)?;
        identity_field(record, &["active_state", "identity"], &self.path)?;
        identity_field(record, &["active_state", "parent_identity"], &self.path)?;
        identity_field(record, &["registry", "identity"], &self.path)?;
        identity_field(record, &["registry", "parent_identity"], &self.path)?;
        let suffix = transaction_id
            .strip_prefix("close_tx_")
            .ok_or_else(|| invalid_transaction(&self.path, "invalid transaction id prefix"))?;
        validate_quarantine_field(
            record,
            &["worktree", "quarantine_name"],
            &format!(".codefire-close-worktree-{suffix}"),
            &self.path,
        )?;
        validate_quarantine_field(
            record,
            &["worktree", "marker_quarantine_name"],
            &format!(".codefire-close-marker-{suffix}"),
            &self.path,
        )?;
        validate_quarantine_field(
            record,
            &["active_state", "quarantine_name"],
            &format!(".codefire-active-remove-{suffix}"),
            &self.path,
        )?;
        validate_quarantine_field(
            record,
            &["registry", "quarantine_name"],
            &format!(".codefire-close-registry-{suffix}"),
            &self.path,
        )?;
        if expected_status == "completed" {
            identity_field(
                record,
                &["completion", "pending_marker_identity"],
                &self.path,
            )?;
            identity_field(
                record,
                &["completion", "pending_parent_identity"],
                &self.path,
            )?;
            validate_quarantine_field(
                record,
                &["completion", "pending_quarantine_name"],
                &format!(".codefire-close-transaction-finish-{suffix}"),
                &self.path,
            )?;
            if string(record, &["completed_at"], &self.path)?.is_empty() {
                return Err(invalid_transaction(
                    &self.path,
                    "empty completion timestamp",
                ));
            }
        }
        let registry = value(record, &["registry", "record"], &self.path)?;
        let open_instance_id = required_string(registry, &["open", "open_instance_id"])?;
        let registry_open_path = required_string(registry, &["open", "path"])?;
        let registry_active_path = required_string(registry, &["open", "active_state_path"])?;
        let expected_active = repo_root
            .join(".codefire")
            .join("active")
            .join(&open_instance_id);
        if required_string(registry, &["branch", "name"])? != branch_name
            || Path::new(&registry_open_path) != open_dir
            || path_field(record, &["registry", "path"], &self.path)?
                != opened_registry_path(repo_root, &branch_name)
            || Path::new(&registry_active_path) != expected_active
            || path_field(record, &["active_state", "path"], &self.path)? != expected_active
        {
            return Err(invalid_transaction(
                &self.path,
                "registry, worktree, or active-state subjects disagree",
            ));
        }
        let marker = value(record, &["worktree", "marker"], &self.path)?;
        let marker_opened_path = required_string(marker, &["open", "opened_path"])?;
        let marker_repository_path = required_string(marker, &["repository", "path"])?;
        let expected_repository_path = repo_root.join(".codefire");
        if required_string(marker, &["branch", "name"])? != branch_name
            || Path::new(&marker_opened_path) != open_dir
            || required_string(marker, &["open", "open_instance_id"])? != open_instance_id
            || Path::new(&marker_repository_path) != expected_repository_path
            || required_string(marker, &["repository", "repository_id"])?
                != string(record, &["repository", "repository_id"], &self.path)?
        {
            return Err(invalid_transaction(
                &self.path,
                "open marker subject mismatch",
            ));
        }
        let result = value(record, &["result"], &self.path)?;
        if string(result, &["branch"], &self.path)? != branch_name
            || path_field(result, &["open_dir"], &self.path)? != open_dir
            || string(result, &["plan", "command"], &self.path)? != command
            || bool_field(result, &["plan", "dry_run"], &self.path)?
        {
            return Err(invalid_transaction(&self.path, "result subject mismatch"));
        }
        Ok(())
    }

    fn verify_request(
        &self,
        repo_root: &Path,
        repository_id: &str,
        options: &CloseOptions,
    ) -> Result<(), CliError> {
        if path_field(&self.record, &["repository", "path"], &self.path)? != repo_root
            || string(&self.record, &["repository", "repository_id"], &self.path)? != repository_id
            || string(&self.record, &["request", "branch"], &self.path)? != options.branch
            || bool_field(&self.record, &["request", "discard"], &self.path)? != options.discard
            || bool_field(&self.record, &["request", "keep_dir"], &self.path)? != options.keep_dir
        {
            return Err(CliError::IdempotencyConflict(format!(
                "pending close transaction conflicts with branch, close/discard mode, or keep-dir mode: {}",
                self.path.display()
            )));
        }
        Ok(())
    }

    fn phase(&self) -> Result<ClosePhase, CliError> {
        ClosePhase::parse(&string(&self.record, &["phase"], &self.path)?, &self.path)
    }

    fn save_new(&self) -> Result<(), CliError> {
        let mut bytes = self.bytes()?;
        write_new_marker_nofollow(&self.path, &mut bytes).map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                invalid_transaction(&self.path, "a pending transaction already exists")
            } else {
                CliError::Io(error)
            }
        })
    }

    fn advance(&mut self, next: ClosePhase) -> Result<(), CliError> {
        let current = self.phase()?;
        if current.next() != Some(next) {
            return Err(invalid_transaction(
                &self.path,
                &format!(
                    "non-adjacent phase transition {} -> {}",
                    current.as_str(),
                    next.as_str()
                ),
            ));
        }
        let previous_record = self.record.clone();
        exact_json_file_identities(&self.path, &previous_record, &self.path)?;
        self.record["phase"] = Value::String(next.as_str().to_string());
        self.record["updated_at"] = Value::String(now_iso_utc());
        let bytes = self.bytes()?;
        let parent_path = self
            .path
            .parent()
            .ok_or_else(|| invalid_transaction(&self.path, "transaction marker has no parent"))?;
        let expected_parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
        secure_worktree::ensure_absolute_dir_identity(&expected_parent, parent_path)?;
        secure_worktree::write_file_atomic_nofollow(
            &self.path,
            &unique_temp_path(&self.path),
            &bytes,
        )?;
        secure_worktree::ensure_absolute_dir_identity(&expected_parent, parent_path)?;
        Ok(())
    }

    fn bytes(&self) -> Result<Vec<u8>, CliError> {
        let mut bytes = serde_json::to_vec_pretty(&self.record)?;
        bytes.push(b'\n');
        if bytes.len() as u64 > CLOSE_TRANSACTION_BYTE_LIMIT {
            return Err(invalid_transaction(
                &self.path,
                &format!(
                    "marker exceeds the {}-byte limit",
                    CLOSE_TRANSACTION_BYTE_LIMIT
                ),
            ));
        }
        Ok(bytes)
    }

    fn result(&self) -> Result<CloseResult, CliError> {
        let result = value(&self.record, &["result"], &self.path)?;
        Ok(CloseResult {
            branch: string(result, &["branch"], &self.path)?,
            open_dir: path_field(result, &["open_dir"], &self.path)?,
            plan: value(result, &["plan"], &self.path)?.clone(),
        })
    }

    fn repo_root(&self) -> Result<PathBuf, CliError> {
        path_field(&self.record, &["repository", "path"], &self.path)
    }

    fn branch_name(&self) -> Result<String, CliError> {
        string(&self.record, &["branch", "name"], &self.path)
    }

    fn archive_and_remove(&self) -> Result<(), CliError> {
        let repo_root = self.repo_root()?;
        let transaction_id = string(&self.record, &["transaction_id"], &self.path)?;
        let suffix = transaction_id
            .strip_prefix("close_tx_")
            .ok_or_else(|| invalid_transaction(&self.path, "invalid transaction id"))?;
        let quarantine_name = format!(".codefire-close-transaction-finish-{suffix}");
        let archive_path = completed_transaction_path(&repo_root);
        let archive = match Self::load_completed(&repo_root)? {
            Some(archive) => {
                if archive.pending_record_from_archive()? != self.record {
                    return Err(invalid_transaction(
                        &archive.path,
                        "completed archive does not derive from the pending marker",
                    ));
                }
                archive.record
            }
            None => {
                let (marker_identity, parent_identity) =
                    exact_json_file_identities(&self.path, &self.record, &self.path)?;
                let mut archive = self.record.clone();
                archive["status"] = Value::String("completed".to_string());
                archive["completed_at"] = Value::String(now_iso_utc());
                archive["completion"] = json!({
                    "pending_marker_identity": identity_json(marker_identity),
                    "pending_parent_identity": identity_json(parent_identity),
                    "pending_quarantine_name": quarantine_name,
                });
                write_json_new_or_exact(&archive_path, &archive, CLOSE_TRANSACTION_BYTE_LIMIT)?;
                archive
            }
        };
        maybe_fail_close_transaction_for_test(
            &repo_root,
            "after_complete_archive_before_marker_removal",
        )?;
        remove_json_file_transactional(
            &self.path,
            &quarantine_name,
            identity_field(
                &archive,
                &["completion", "pending_marker_identity"],
                &archive_path,
            )?,
            identity_field(
                &archive,
                &["completion", "pending_parent_identity"],
                &archive_path,
            )?,
            &self.record,
            &archive_path,
        )?;
        maybe_fail_close_transaction_for_test(&repo_root, "after_pending_marker_removed")
    }

    fn cleanup_pending_authority_from_archive(&self) -> Result<(), CliError> {
        if string(&self.record, &["status"], &self.path)? != "completed" {
            return Err(invalid_transaction(
                &self.path,
                "pending-authority cleanup requires a completed archive",
            ));
        }
        let pending = self.pending_record_from_archive()?;
        remove_json_file_transactional(
            &close_transaction_path(&self.repo_root()?),
            &string(
                &self.record,
                &["completion", "pending_quarantine_name"],
                &self.path,
            )?,
            identity_field(
                &self.record,
                &["completion", "pending_marker_identity"],
                &self.path,
            )?,
            identity_field(
                &self.record,
                &["completion", "pending_parent_identity"],
                &self.path,
            )?,
            &pending,
            &self.path,
        )
    }

    fn pending_record_from_archive(&self) -> Result<Value, CliError> {
        let mut pending = self.record.clone();
        pending["status"] = Value::String("pending".to_string());
        let object = pending
            .as_object_mut()
            .ok_or_else(|| invalid_transaction(&self.path, "archive must be an object"))?;
        object.remove("completed_at");
        object.remove("completion");
        Ok(pending)
    }
}

fn retire_completed_archive(transaction: &CloseTransaction) -> Result<(), CliError> {
    let repo_root = transaction.repo_root()?;
    let retiring = retiring_completed_transaction_path(&repo_root);
    if transaction.path == retiring {
        let (identity, parent_identity) =
            exact_json_file_identities(&retiring, &transaction.record, &transaction.path)?;
        let parent_path = retiring.parent().ok_or_else(|| {
            invalid_transaction(&transaction.path, "retiring archive has no parent")
        })?;
        let name = retiring.file_name().ok_or_else(|| {
            invalid_transaction(&transaction.path, "retiring archive has no name")
        })?;
        let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
        parent_identity.ensure_directory(&parent, "completed-archive parent")?;
        let file = open_optional_regular(&parent, name)?.ok_or_else(|| {
            invalid_transaction(&transaction.path, "retiring archive disappeared")
        })?;
        identity.ensure_file(&file, "retiring completed archive")?;
        parent.remove_file(name)?;
        sync_cap_directory(&parent)?;
        return Ok(());
    }
    let (identity, parent_identity) =
        exact_json_file_identities(&transaction.path, &transaction.record, &transaction.path)?;
    remove_json_file_transactional(
        &transaction.path,
        retiring
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| invalid_transaction(&transaction.path, "invalid retiring name"))?,
        identity,
        parent_identity,
        &transaction.record,
        &transaction.path,
    )
}

fn exact_json_file_identities(
    path: &Path,
    expected: &Value,
    transaction_path: &Path,
) -> Result<(StableFsIdentity, StableFsIdentity), CliError> {
    let parent_path = path
        .parent()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no name"))?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    let parent_identity = StableFsIdentity::for_directory(&parent)?;
    let file = open_optional_regular(&parent, name)?
        .ok_or_else(|| invalid_transaction(transaction_path, "expected JSON subject is missing"))?;
    let identity = StableFsIdentity::for_file(&file)?;
    validate_json_file(
        file,
        identity,
        expected,
        "transaction JSON subject",
        transaction_path,
    )?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    Ok((identity, parent_identity))
}

fn write_json_new_or_exact(path: &Path, value: &Value, limit: u64) -> Result<(), CliError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > limit {
        return Err(invalid_transaction(
            path,
            "completed archive exceeds its byte limit",
        ));
    }
    let parent_path = path
        .parent()
        .ok_or_else(|| invalid_transaction(path, "completed archive has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid_transaction(path, "completed archive has no name"))?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    let result = match read_bytes_from_capability(&parent, name, limit) {
        Ok(existing) => {
            let existing: Value = serde_json::from_slice(&existing).map_err(|error| {
                invalid_transaction(path, &format!("completed archive is malformed: {error}"))
            })?;
            if &existing != value {
                return Err(invalid_transaction(path, "completed archive is divergent"));
            }
            Ok(())
        }
        Err(CliError::Io(error)) if error.kind() == ErrorKind::NotFound => {
            write_new_marker_to_capability(&parent, name, &mut bytes).map_err(CliError::Io)
        }
        Err(error) => Err(error),
    };
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    result
}

fn inspect_evidence(transaction: &CloseTransaction) -> Result<CloseEvidence, CliError> {
    let worktree_path = path_field(
        &transaction.record,
        &["worktree", "path"],
        &transaction.path,
    )?;
    let worktree_identity = identity_field(
        &transaction.record,
        &["worktree", "identity"],
        &transaction.path,
    )?;
    let action = string(
        &transaction.record,
        &["worktree", "action"],
        &transaction.path,
    )?;
    let worktree = if action == "remove_marker" {
        let bound = ClosableWorktree::bind(&worktree_path).map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("kept worktree cannot be rebound: {error}"),
            )
        })?;
        bound.ensure_identity(worktree_identity).map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("kept worktree identity diverged: {error}"),
            )
        })?;
        bound
            .ensure_parent_identity(identity_field(
                &transaction.record,
                &["worktree", "parent_identity"],
                &transaction.path,
            )?)
            .map_err(|error| {
                invalid_transaction(
                    &transaction.path,
                    &format!("kept worktree parent identity diverged: {error}"),
                )
            })?;
        removal_artifact(
            bound.marker_removal_progress(
                identity_field(
                    &transaction.record,
                    &["worktree", "marker_identity"],
                    &transaction.path,
                )?,
                OsString::from(string(
                    &transaction.record,
                    &["worktree", "marker_quarantine_name"],
                    &transaction.path,
                )?)
                .as_os_str(),
            )?,
        )
    } else if action == "remove_tree" {
        removal_artifact(close_worktree::tree_removal_progress(
            &worktree_path,
            OsString::from(string(
                &transaction.record,
                &["worktree", "quarantine_name"],
                &transaction.path,
            )?)
            .as_os_str(),
            worktree_identity,
            identity_field(
                &transaction.record,
                &["worktree", "parent_identity"],
                &transaction.path,
            )?,
        )?)
    } else {
        return Err(invalid_transaction(
            &transaction.path,
            "unsupported worktree action",
        ));
    };

    let active_path = path_field(
        &transaction.record,
        &["active_state", "path"],
        &transaction.path,
    )?;
    let active_state = removal_artifact(active_state::ActiveStateDir::removal_progress(
        &active_path,
        OsString::from(string(
            &transaction.record,
            &["active_state", "quarantine_name"],
            &transaction.path,
        )?)
        .as_os_str(),
        identity_field(
            &transaction.record,
            &["active_state", "identity"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["active_state", "parent_identity"],
            &transaction.path,
        )?,
    )?);

    let registry_path = path_field(
        &transaction.record,
        &["registry", "path"],
        &transaction.path,
    )?;
    let registry = match classify_json_file_removal(
        &registry_path,
        &string(
            &transaction.record,
            &["registry", "quarantine_name"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["registry", "identity"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["registry", "parent_identity"],
            &transaction.path,
        )?,
        value(
            &transaction.record,
            &["registry", "record"],
            &transaction.path,
        )?,
        &transaction.path,
    )? {
        RemovalArtifact::Before => RegistryArtifact::Exact,
        RemovalArtifact::Quarantined => RegistryArtifact::Quarantined,
        RemovalArtifact::Removed => RegistryArtifact::Missing,
    };
    let branch = classify_branch(transaction)?;
    Ok(CloseEvidence {
        worktree,
        active_state,
        registry,
        branch,
    })
}

fn classify_branch(transaction: &CloseTransaction) -> Result<BranchArtifact, CliError> {
    let repo_root = transaction.repo_root()?;
    let branch_name = transaction.branch_name()?;
    let branch_path = branch_record_path(&repo_root, &branch_name);
    let branch_parent_identity = identity_field(
        &transaction.record,
        &["branch", "parent_identity"],
        &transaction.path,
    )?;
    let actual = read_json_with_parent_identity(&branch_path, branch_parent_identity)?;
    let before = value(
        &transaction.record,
        &["branch", "before"],
        &transaction.path,
    )?;
    let after = value(&transaction.record, &["branch", "after"], &transaction.path)?;
    let object_id = string(
        &transaction.record,
        &["branch", "object_id"],
        &transaction.path,
    )?;
    let object =
        match codefire_store::read_object(&repo_root.join(".codefire").join("objects"), &object_id)
        {
            Ok(object) if &object == after => Some(object),
            Ok(_) => {
                return Err(invalid_transaction(
                    &transaction.path,
                    "stored branch object differs from the close postimage",
                ));
            }
            Err(codefire_store::StoreError::ObjectNotFound(_)) => None,
            Err(error) => return Err(CliError::Store(Box::new(error))),
        };
    if &actual == before {
        let (actual_identity, actual_parent_identity) =
            exact_json_file_identities(&branch_path, before, &transaction.path)?;
        if actual_identity
            != identity_field(
                &transaction.record,
                &["branch", "reference_identity"],
                &transaction.path,
            )?
            || actual_parent_identity != branch_parent_identity
        {
            return Err(invalid_transaction(
                &transaction.path,
                "branch preimage identity changed",
            ));
        }
        return Ok(if object.is_some() {
            BranchArtifact::BeforeWithAfterObject
        } else {
            BranchArtifact::Before
        });
    }
    if &actual == after {
        if object.is_none() {
            return Err(invalid_transaction(
                &transaction.path,
                "branch reference reached the close postimage before its immutable object",
            ));
        }
        return Ok(BranchArtifact::After);
    }
    Err(invalid_transaction(
        &transaction.path,
        "branch record differs from both close preimage and postimage",
    ))
}

fn read_json_with_parent_identity(
    path: &Path,
    expected_parent: StableFsIdentity,
) -> Result<Value, CliError> {
    let parent_path = path.parent().ok_or_else(|| {
        CliError::InvalidRepository(format!("JSON path has no parent: {}", path.display()))
    })?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    expected_parent.ensure_directory(&parent, "branch-record parent")?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    let name = path.file_name().ok_or_else(|| {
        CliError::InvalidRepository(format!("JSON path has no name: {}", path.display()))
    })?;
    let value = read_json_from_capability(&parent, name, REPOSITORY_JSON_BYTE_LIMIT)?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    Ok(value)
}

fn read_json_from_capability(
    parent: &cap_std::fs::Dir,
    name: &std::ffi::OsStr,
    limit: u64,
) -> Result<Value, CliError> {
    let bytes = read_bytes_from_capability(parent, name, limit)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn read_bytes_from_capability(
    parent: &cap_std::fs::Dir,
    name: &std::ffi::OsStr,
    limit: u64,
) -> Result<Vec<u8>, CliError> {
    let mut file = open_optional_regular(parent, name)?
        .ok_or_else(|| CliError::Io(std::io::Error::from(ErrorKind::NotFound)))?;
    let metadata = file.metadata()?;
    if metadata.len() > limit {
        return Err(CliError::InvalidRepository(format!(
            "JSON file exceeds its {limit}-byte limit"
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
    (&mut file)
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "JSON file grew beyond its {limit}-byte limit"
        )));
    }
    Ok(bytes)
}

fn write_json_to_capability(
    parent: &cap_std::fs::Dir,
    name: &std::ffi::OsStr,
    value: &Value,
) -> Result<(), CliError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "branch reference exceeds its {}-byte limit",
            REPOSITORY_JSON_BYTE_LIMIT
        )));
    }
    for _ in 0..NEW_MARKER_ATTEMPTS {
        let counter = NEW_MARKER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_name = OsString::from(format!(
            ".codefire-branch-close.{}.{counter:016x}.tmp",
            std::process::id()
        ));
        let mut options = CapOpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut temp = match parent.open_with(&temp_name, &options) {
            Ok(temp) => temp,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(CliError::Io(error)),
        };
        let result = (|| -> std::io::Result<()> {
            temp.write_all(&bytes)?;
            temp.sync_all()?;
            drop(temp);
            parent.rename(&temp_name, parent, name)?;
            sync_cap_directory(parent)
        })();
        if result.is_err() {
            let _ = parent.remove_file(&temp_name);
        }
        return result.map_err(CliError::Io);
    }
    Err(CliError::Io(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not allocate branch-close temporary file",
    )))
}

fn removal_artifact(progress: secure_worktree::RemovalProgress) -> RemovalArtifact {
    match progress {
        secure_worktree::RemovalProgress::Before => RemovalArtifact::Before,
        secure_worktree::RemovalProgress::Quarantined => RemovalArtifact::Quarantined,
        secure_worktree::RemovalProgress::Removed => RemovalArtifact::Removed,
    }
}

fn resume_transaction(mut transaction: CloseTransaction) -> Result<CloseResult, CliError> {
    let repo_root = transaction.repo_root()?;
    let mut evidence = inspect_evidence(&transaction)?;
    validate_phase_evidence(transaction.phase()?, evidence, &transaction.path)?;

    if transaction.phase()? == ClosePhase::Prepared {
        if evidence.worktree != RemovalArtifact::Removed {
            apply_worktree_action(&transaction)?;
            maybe_fail_close_transaction_for_test(
                &repo_root,
                "after_worktree_action_before_phase",
            )?;
            evidence = inspect_evidence(&transaction)?;
            if evidence.worktree != RemovalArtifact::Removed {
                return Err(invalid_transaction(
                    &transaction.path,
                    "worktree action did not reach its postimage",
                ));
            }
        }
        transaction.advance(ClosePhase::WorktreeApplied)?;
        maybe_fail_close_transaction_for_test(&repo_root, "after_worktree_applied")?;
    }

    if transaction.phase()? == ClosePhase::WorktreeApplied {
        if evidence.active_state != RemovalArtifact::Removed {
            apply_active_state_removal(&transaction)?;
            maybe_fail_close_transaction_for_test(
                &repo_root,
                "after_active_state_remove_before_phase",
            )?;
            evidence = inspect_evidence(&transaction)?;
            if evidence.active_state != RemovalArtifact::Removed {
                return Err(invalid_transaction(
                    &transaction.path,
                    "active-state action did not reach its postimage",
                ));
            }
        }
        transaction.advance(ClosePhase::ActiveStateRemoved)?;
        maybe_fail_close_transaction_for_test(&repo_root, "after_active_state_removed")?;
    }

    if transaction.phase()? == ClosePhase::ActiveStateRemoved {
        if evidence.registry != RegistryArtifact::Missing {
            apply_registry_removal(&transaction)?;
            maybe_fail_close_transaction_for_test(
                &repo_root,
                "after_registry_remove_before_phase",
            )?;
            evidence = inspect_evidence(&transaction)?;
            if evidence.registry != RegistryArtifact::Missing {
                return Err(invalid_transaction(
                    &transaction.path,
                    "opened-registry action did not reach its postimage",
                ));
            }
        }
        transaction.advance(ClosePhase::RegistryRemoved)?;
        maybe_fail_close_transaction_for_test(&repo_root, "after_registry_removed")?;
    }

    if transaction.phase()? == ClosePhase::RegistryRemoved {
        if evidence.branch != BranchArtifact::After {
            apply_branch_publication(&transaction, evidence.branch)?;
            evidence = inspect_evidence(&transaction)?;
            if evidence.branch != BranchArtifact::After {
                return Err(invalid_transaction(
                    &transaction.path,
                    "branch publication did not reach its postimage",
                ));
            }
        }
        transaction.advance(ClosePhase::BranchWritten)?;
        maybe_fail_close_transaction_for_test(&repo_root, "after_branch_written")?;
    }

    evidence = inspect_evidence(&transaction)?;
    validate_phase_evidence(ClosePhase::BranchWritten, evidence, &transaction.path)?;
    let result = transaction.result()?;
    transaction.archive_and_remove()?;
    Ok(result)
}

fn apply_worktree_action(transaction: &CloseTransaction) -> Result<(), CliError> {
    let path = path_field(
        &transaction.record,
        &["worktree", "path"],
        &transaction.path,
    )?;
    let action = string(
        &transaction.record,
        &["worktree", "action"],
        &transaction.path,
    )?;
    if action == "remove_marker" {
        let worktree = ClosableWorktree::bind(&path).map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("kept worktree cannot be rebound: {error}"),
            )
        })?;
        worktree.ensure_identity(identity_field(
            &transaction.record,
            &["worktree", "identity"],
            &transaction.path,
        )?)?;
        worktree.ensure_parent_identity(identity_field(
            &transaction.record,
            &["worktree", "parent_identity"],
            &transaction.path,
        )?)?;
        worktree.remove_marker_transactional(
            identity_field(
                &transaction.record,
                &["worktree", "marker_identity"],
                &transaction.path,
            )?,
            OsString::from(string(
                &transaction.record,
                &["worktree", "marker_quarantine_name"],
                &transaction.path,
            )?)
            .as_os_str(),
        )?;
    } else if action == "remove_tree" {
        close_worktree::remove_tree_transactional(
            &path,
            OsString::from(string(
                &transaction.record,
                &["worktree", "quarantine_name"],
                &transaction.path,
            )?)
            .as_os_str(),
            identity_field(
                &transaction.record,
                &["worktree", "identity"],
                &transaction.path,
            )?,
            identity_field(
                &transaction.record,
                &["worktree", "parent_identity"],
                &transaction.path,
            )?,
        )?;
    } else {
        return Err(invalid_transaction(
            &transaction.path,
            "unsupported worktree action",
        ));
    }
    Ok(())
}

fn apply_active_state_removal(transaction: &CloseTransaction) -> Result<(), CliError> {
    active_state::ActiveStateDir::remove_tree_transactional(
        &path_field(
            &transaction.record,
            &["active_state", "path"],
            &transaction.path,
        )?,
        OsString::from(string(
            &transaction.record,
            &["active_state", "quarantine_name"],
            &transaction.path,
        )?)
        .as_os_str(),
        identity_field(
            &transaction.record,
            &["active_state", "identity"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["active_state", "parent_identity"],
            &transaction.path,
        )?,
    )?;
    Ok(())
}

fn apply_registry_removal(transaction: &CloseTransaction) -> Result<(), CliError> {
    remove_json_file_transactional(
        &path_field(
            &transaction.record,
            &["registry", "path"],
            &transaction.path,
        )?,
        &string(
            &transaction.record,
            &["registry", "quarantine_name"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["registry", "identity"],
            &transaction.path,
        )?,
        identity_field(
            &transaction.record,
            &["registry", "parent_identity"],
            &transaction.path,
        )?,
        value(
            &transaction.record,
            &["registry", "record"],
            &transaction.path,
        )?,
        &transaction.path,
    )
}

fn apply_branch_publication(
    transaction: &CloseTransaction,
    evidence: BranchArtifact,
) -> Result<(), CliError> {
    let repo_root = transaction.repo_root()?;
    let branch_name = transaction.branch_name()?;
    let branch_path = branch_record_path(&repo_root, &branch_name);
    let branch_parent_path = branch_path
        .parent()
        .ok_or_else(|| invalid_transaction(&transaction.path, "branch reference has no parent"))?;
    let branch_parent = secure_worktree::open_absolute_dir_nofollow(branch_parent_path)?;
    let expected_parent = identity_field(
        &transaction.record,
        &["branch", "parent_identity"],
        &transaction.path,
    )?;
    expected_parent.ensure_directory(&branch_parent, "branch-record parent")?;
    secure_worktree::ensure_absolute_dir_identity(&branch_parent, branch_parent_path)?;
    let after = value(&transaction.record, &["branch", "after"], &transaction.path)?;
    if evidence == BranchArtifact::Before {
        let object_id = codefire_store::store_object(
            &repo_root.join(".codefire").join("objects"),
            "branch",
            after.clone(),
        )?;
        if object_id
            != string(
                &transaction.record,
                &["branch", "object_id"],
                &transaction.path,
            )?
        {
            return Err(invalid_transaction(
                &transaction.path,
                "stored branch object identity changed",
            ));
        }
        maybe_fail_close_transaction_for_test(&repo_root, "after_branch_object_before_reference")?;
    }
    let branch_name_component = branch_path
        .file_name()
        .ok_or_else(|| invalid_transaction(&transaction.path, "branch reference has no name"))?;
    let current =
        open_optional_regular(&branch_parent, branch_name_component)?.ok_or_else(|| {
            invalid_transaction(
                &transaction.path,
                "branch reference disappeared before close publication",
            )
        })?;
    validate_json_file(
        current,
        identity_field(
            &transaction.record,
            &["branch", "reference_identity"],
            &transaction.path,
        )?,
        value(
            &transaction.record,
            &["branch", "before"],
            &transaction.path,
        )?,
        "branch reference before close publication",
        &transaction.path,
    )?;
    maybe_swap_close_branch_parent_before_write_for_test(&repo_root, branch_parent_path)?;
    secure_worktree::ensure_absolute_dir_identity(&branch_parent, branch_parent_path)?;
    write_json_to_capability(&branch_parent, branch_name_component, after)?;
    secure_worktree::ensure_absolute_dir_identity(&branch_parent, branch_parent_path)?;
    maybe_fail_close_transaction_for_test(&repo_root, "after_branch_reference_before_phase")
}

fn validate_phase_evidence(
    phase: ClosePhase,
    evidence: CloseEvidence,
    path: &Path,
) -> Result<(), CliError> {
    let worktree_pending = matches!(
        evidence.worktree,
        RemovalArtifact::Before | RemovalArtifact::Quarantined
    );
    let active_pending = matches!(
        evidence.active_state,
        RemovalArtifact::Before | RemovalArtifact::Quarantined
    );
    let valid = match phase {
        ClosePhase::Prepared => {
            matches!(
                evidence.worktree,
                RemovalArtifact::Before | RemovalArtifact::Quarantined | RemovalArtifact::Removed
            ) && evidence.active_state == RemovalArtifact::Before
                && evidence.registry == RegistryArtifact::Exact
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeWithAfterObject
                )
        }
        ClosePhase::WorktreeApplied => {
            !worktree_pending
                && matches!(
                    evidence.active_state,
                    RemovalArtifact::Before
                        | RemovalArtifact::Quarantined
                        | RemovalArtifact::Removed
                )
                && evidence.registry == RegistryArtifact::Exact
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeWithAfterObject
                )
        }
        ClosePhase::ActiveStateRemoved => {
            evidence.worktree == RemovalArtifact::Removed
                && !active_pending
                && matches!(
                    evidence.registry,
                    RegistryArtifact::Exact
                        | RegistryArtifact::Quarantined
                        | RegistryArtifact::Missing
                )
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeWithAfterObject
                )
        }
        ClosePhase::RegistryRemoved => {
            evidence.worktree == RemovalArtifact::Removed
                && evidence.active_state == RemovalArtifact::Removed
                && evidence.registry == RegistryArtifact::Missing
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before
                        | BranchArtifact::BeforeWithAfterObject
                        | BranchArtifact::After
                )
        }
        ClosePhase::BranchWritten => {
            evidence.worktree == RemovalArtifact::Removed
                && evidence.active_state == RemovalArtifact::Removed
                && evidence.registry == RegistryArtifact::Missing
                && evidence.branch == BranchArtifact::After
        }
    };
    if valid {
        Ok(())
    } else {
        Err(invalid_transaction(
            path,
            &format!("artifacts diverge from phase {}", phase.as_str()),
        ))
    }
}

fn identity_json(identity: StableFsIdentity) -> Value {
    json!({
        "scheme": identity.scheme(),
        "first": identity.first(),
        "second": identity.second(),
    })
}

fn classify_json_file_removal(
    path: &Path,
    quarantine_name: &str,
    expected_identity: StableFsIdentity,
    expected_parent_identity: StableFsIdentity,
    expected: &Value,
    transaction_path: &Path,
) -> Result<RemovalArtifact, CliError> {
    let parent_path = path
        .parent()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no parent"))?;
    let original_name = path
        .file_name()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no name"))?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    expected_parent_identity
        .ensure_directory(&parent, "transaction JSON parent")
        .map_err(CliError::Io)?;
    let original = open_optional_regular(&parent, original_name)?;
    let quarantine = open_optional_regular(&parent, quarantine_name.as_ref())?;
    match (original, quarantine) {
        (Some(file), None) => {
            validate_json_file(
                file,
                expected_identity,
                expected,
                "transaction JSON subject",
                transaction_path,
            )?;
            Ok(RemovalArtifact::Before)
        }
        (None, Some(file)) => {
            validate_json_file(
                file,
                expected_identity,
                expected,
                "quarantined transaction JSON subject",
                transaction_path,
            )?;
            Ok(RemovalArtifact::Quarantined)
        }
        (None, None) => Ok(RemovalArtifact::Removed),
        (Some(_), Some(_)) => Err(invalid_transaction(
            transaction_path,
            "both transaction JSON subject and its quarantine exist",
        )),
    }
}

fn remove_json_file_transactional(
    path: &Path,
    quarantine_name: &str,
    expected_identity: StableFsIdentity,
    expected_parent_identity: StableFsIdentity,
    expected: &Value,
    transaction_path: &Path,
) -> Result<(), CliError> {
    let progress = classify_json_file_removal(
        path,
        quarantine_name,
        expected_identity,
        expected_parent_identity,
        expected,
        transaction_path,
    )?;
    if progress == RemovalArtifact::Removed {
        return Ok(());
    }
    let parent_path = path
        .parent()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no parent"))?;
    let original_name = path
        .file_name()
        .ok_or_else(|| invalid_transaction(transaction_path, "JSON subject has no name"))?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    expected_parent_identity
        .ensure_directory(&parent, "transaction JSON parent")
        .map_err(CliError::Io)?;
    if progress == RemovalArtifact::Before {
        secure_worktree::rename_entry_noreplace(&parent, original_name, quarantine_name.as_ref())?;
        sync_cap_directory(&parent)?;
    }
    let quarantined =
        open_optional_regular(&parent, quarantine_name.as_ref())?.ok_or_else(|| {
            invalid_transaction(
                transaction_path,
                "transaction JSON quarantine disappeared during removal",
            )
        })?;
    validate_json_file(
        quarantined,
        expected_identity,
        expected,
        "quarantined transaction JSON subject",
        transaction_path,
    )?;
    parent.remove_file(quarantine_name)?;
    sync_cap_directory(&parent)?;
    if classify_json_file_removal(
        path,
        quarantine_name,
        expected_identity,
        expected_parent_identity,
        expected,
        transaction_path,
    )? != RemovalArtifact::Removed
    {
        return Err(invalid_transaction(
            transaction_path,
            "transaction JSON removal did not reach the removed state",
        ));
    }
    Ok(())
}

fn open_optional_regular(
    directory: &cap_std::fs::Dir,
    name: &std::ffi::OsStr,
) -> std::io::Result<Option<CapFile>> {
    let mut options = CapOpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    match directory.open_with(name, &options) {
        Ok(file) if file.metadata()?.is_file() => Ok(Some(file)),
        Ok(_) => Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "close transaction file subject is not a regular file",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn validate_json_file(
    mut file: CapFile,
    expected_identity: StableFsIdentity,
    expected: &Value,
    subject: &str,
    transaction_path: &Path,
) -> Result<(), CliError> {
    expected_identity
        .ensure_file(&file, subject)
        .map_err(CliError::Io)?;
    let metadata = file.metadata()?;
    if metadata.len() > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(invalid_transaction(
            transaction_path,
            &format!("{subject} exceeds its byte limit"),
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
    (&mut file)
        .take(REPOSITORY_JSON_BYTE_LIMIT.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > REPOSITORY_JSON_BYTE_LIMIT {
        return Err(invalid_transaction(
            transaction_path,
            &format!("{subject} grew beyond its byte limit"),
        ));
    }
    let actual: Value = serde_json::from_slice(&bytes).map_err(|error| {
        invalid_transaction(
            transaction_path,
            &format!("{subject} is malformed JSON: {error}"),
        )
    })?;
    if &actual != expected {
        return Err(invalid_transaction(
            transaction_path,
            &format!("{subject} differs from the transaction preimage"),
        ));
    }
    Ok(())
}

fn identity_field(
    record: &Value,
    fields: &[&str],
    path: &Path,
) -> Result<StableFsIdentity, CliError> {
    let identity = value(record, fields, path)?;
    let scheme = string(identity, &["scheme"], path)?;
    let first = u64_field(identity, &["first"], path)?;
    let second = u64_field(identity, &["second"], path)?;
    StableFsIdentity::from_parts(&scheme, first, second).map_err(|error| {
        invalid_transaction(path, &format!("invalid filesystem identity: {error}"))
    })
}

fn validate_quarantine_field(
    record: &Value,
    fields: &[&str],
    expected: &str,
    path: &Path,
) -> Result<(), CliError> {
    let actual = string(record, fields, path)?;
    let mut components = Path::new(&actual).components();
    if actual != expected
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(invalid_transaction(
            path,
            "transaction quarantine name is not the derived single component",
        ));
    }
    Ok(())
}

fn validate_transaction_id(transaction_id: &str, path: &Path) -> Result<(), CliError> {
    let Some(suffix) = transaction_id.strip_prefix("close_tx_") else {
        return Err(invalid_transaction(path, "invalid transaction id prefix"));
    };
    if suffix.len() != 32 || !suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid_transaction(path, "invalid transaction id"));
    }
    Ok(())
}

fn random_transaction_id() -> Result<String, CliError> {
    let mut bytes = [0_u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate close transaction identity: {error}"
        ))
    })?;
    Ok(format!(
        "close_tx_{}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
}

fn write_new_marker_nofollow(path: &Path, bytes: &mut [u8]) -> std::io::Result<()> {
    let parent_path = path.parent().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidInput,
            "close transaction marker has no parent",
        )
    })?;
    let final_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidInput,
            "close transaction marker has no name",
        )
    })?;
    let parent = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
    let result = write_new_marker_to_capability(&parent, final_name, bytes);
    secure_worktree::ensure_absolute_dir_identity(&parent, parent_path)?;
    result
}

fn write_new_marker_to_capability(
    parent: &cap_std::fs::Dir,
    final_name: &std::ffi::OsStr,
    bytes: &mut [u8],
) -> std::io::Result<()> {
    for _ in 0..NEW_MARKER_ATTEMPTS {
        let counter = NEW_MARKER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_name = OsString::from(format!(
            ".close-transaction.{}.{counter:016x}.tmp",
            std::process::id()
        ));
        let mut options = CapOpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut temp = match parent.open_with(&temp_name, &options) {
            Ok(temp) => temp,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            temp.write_all(bytes)?;
            temp.sync_all()?;
            parent.hard_link(&temp_name, parent, final_name)?;
            parent.remove_file(&temp_name)?;
            sync_cap_directory(parent)
        })();
        if result.is_err() {
            let _ = parent.remove_file(&temp_name);
        }
        return result;
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not allocate close transaction marker temporary file",
    ))
}

fn sync_cap_directory(directory: &cap_std::fs::Dir) -> std::io::Result<()> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

fn value<'a>(record: &'a Value, fields: &[&str], path: &Path) -> Result<&'a Value, CliError> {
    let mut current = record;
    for field in fields {
        current = current
            .get(*field)
            .ok_or_else(|| invalid_transaction(path, &format!("missing {}", fields.join("."))))?;
    }
    Ok(current)
}

fn string(record: &Value, fields: &[&str], path: &Path) -> Result<String, CliError> {
    value(record, fields, path)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| invalid_transaction(path, &format!("{} must be a string", fields.join("."))))
}

fn u64_field(record: &Value, fields: &[&str], path: &Path) -> Result<u64, CliError> {
    value(record, fields, path)?.as_u64().ok_or_else(|| {
        invalid_transaction(path, &format!("{} must be an integer", fields.join(".")))
    })
}

fn bool_field(record: &Value, fields: &[&str], path: &Path) -> Result<bool, CliError> {
    value(record, fields, path)?.as_bool().ok_or_else(|| {
        invalid_transaction(path, &format!("{} must be a boolean", fields.join(".")))
    })
}

fn path_field(record: &Value, fields: &[&str], path: &Path) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(string(record, fields, path)?))
}

fn invalid_transaction(path: &Path, detail: &str) -> CliError {
    CliError::InvalidRepository(format!(
        "invalid pending close transaction {}: {detail}",
        path.display()
    ))
}

#[cfg(test)]
#[derive(Debug)]
struct CloseFailureInjection {
    repo_root: PathBuf,
    point: String,
}

#[cfg(test)]
#[derive(Debug)]
struct CloseBranchParentSwap {
    repo_root: PathBuf,
    parent: PathBuf,
    moved: PathBuf,
}

#[cfg(test)]
static CLOSE_TRANSACTION_FAILURES: std::sync::Mutex<Vec<CloseFailureInjection>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(test)]
static CLOSE_BRANCH_PARENT_SWAPS: std::sync::Mutex<Vec<CloseBranchParentSwap>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(test)]
pub(crate) fn set_close_transaction_failure_for_test(repo_root: &Path, point: &str) {
    CLOSE_TRANSACTION_FAILURES
        .lock()
        .expect("close failure mutex")
        .push(CloseFailureInjection {
            repo_root: repo_root.to_path_buf(),
            point: point.to_string(),
        });
}

#[cfg(test)]
pub(crate) fn set_close_branch_parent_swap_for_test(repo_root: &Path, parent: &Path, moved: &Path) {
    CLOSE_BRANCH_PARENT_SWAPS
        .lock()
        .expect("close branch parent swap mutex")
        .push(CloseBranchParentSwap {
            repo_root: repo_root.to_path_buf(),
            parent: parent.to_path_buf(),
            moved: moved.to_path_buf(),
        });
}

#[cfg(test)]
fn maybe_swap_close_branch_parent_before_write_for_test(
    repo_root: &Path,
    parent: &Path,
) -> Result<(), CliError> {
    let mut swaps = CLOSE_BRANCH_PARENT_SWAPS
        .lock()
        .expect("close branch parent swap mutex");
    let Some(index) = swaps
        .iter()
        .position(|swap| swap.repo_root == repo_root && swap.parent == parent)
    else {
        return Ok(());
    };
    let swap = swaps.swap_remove(index);
    fs::rename(&swap.parent, &swap.moved)?;
    fs::create_dir(&swap.parent)?;
    fs::write(swap.parent.join("outside-sentinel.txt"), b"outside\n")?;
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_close_branch_parent_before_write_for_test(
    _repo_root: &Path,
    _parent: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn maybe_fail_close_transaction_for_test(repo_root: &Path, point: &str) -> Result<(), CliError> {
    let mut failures = CLOSE_TRANSACTION_FAILURES
        .lock()
        .expect("close failure mutex");
    if let Some(index) = failures
        .iter()
        .position(|failure| failure.repo_root == repo_root && failure.point == point)
    {
        failures.remove(index);
        return Err(CliError::InvalidRepository(format!(
            "injected close transaction failure at {point}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_close_transaction_for_test(_repo_root: &Path, _point: &str) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(
        worktree: RemovalArtifact,
        active_state: RemovalArtifact,
        registry: RegistryArtifact,
        branch: BranchArtifact,
    ) -> CloseEvidence {
        CloseEvidence {
            worktree,
            active_state,
            registry,
            branch,
        }
    }

    #[test]
    fn phase_machine_accepts_only_current_or_immediate_gap_evidence() {
        let path = Path::new("/repo/.codefire/close_transaction.json");
        let valid = [
            (
                ClosePhase::Prepared,
                evidence(
                    RemovalArtifact::Before,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::Prepared,
                evidence(
                    RemovalArtifact::Quarantined,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::Prepared,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::WorktreeApplied,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Quarantined,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::ActiveStateRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::RegistryRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
            (
                ClosePhase::BranchWritten,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::After,
                ),
            ),
        ];
        for (phase, evidence) in valid {
            validate_phase_evidence(phase, evidence, path).unwrap();
        }

        let invalid = [
            (
                ClosePhase::Prepared,
                evidence(
                    RemovalArtifact::Before,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::WorktreeApplied,
                evidence(
                    RemovalArtifact::Before,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::ActiveStateRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::After,
                ),
            ),
            (
                ClosePhase::RegistryRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Exact,
                    BranchArtifact::Before,
                ),
            ),
            (
                ClosePhase::BranchWritten,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
        ];
        for (phase, evidence) in invalid {
            assert!(validate_phase_evidence(phase, evidence, path).is_err());
        }
    }

    #[test]
    fn immutable_branch_postimage_may_preexist_every_prepublication_phase() {
        let path = Path::new("/repo/.codefire/close_transaction.json");
        let cases = [
            (
                ClosePhase::Prepared,
                evidence(
                    RemovalArtifact::Before,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
            (
                ClosePhase::WorktreeApplied,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Before,
                    RegistryArtifact::Exact,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
            (
                ClosePhase::ActiveStateRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Exact,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
            (
                ClosePhase::RegistryRemoved,
                evidence(
                    RemovalArtifact::Removed,
                    RemovalArtifact::Removed,
                    RegistryArtifact::Missing,
                    BranchArtifact::BeforeWithAfterObject,
                ),
            ),
        ];

        for (phase, evidence) in cases {
            validate_phase_evidence(phase, evidence, path).unwrap();
        }
    }

    #[test]
    fn phase_advance_is_strictly_adjacent() {
        assert_eq!(
            ClosePhase::Prepared.next(),
            Some(ClosePhase::WorktreeApplied)
        );
        assert_eq!(
            ClosePhase::WorktreeApplied.next(),
            Some(ClosePhase::ActiveStateRemoved)
        );
        assert_eq!(
            ClosePhase::ActiveStateRemoved.next(),
            Some(ClosePhase::RegistryRemoved)
        );
        assert_eq!(
            ClosePhase::RegistryRemoved.next(),
            Some(ClosePhase::BranchWritten)
        );
        assert_eq!(ClosePhase::BranchWritten.next(), None);
    }

    #[test]
    fn transaction_id_and_quarantine_names_reject_path_injection() {
        let path = Path::new("/repo/.codefire/close_transaction.json");
        assert!(validate_transaction_id("close_tx_0123456789abcdef0123456789abcdef", path).is_ok());
        assert!(validate_transaction_id("close_tx_../../escape", path).is_err());
        let record = json!({"name": "../escape"});
        assert!(validate_quarantine_field(&record, &["name"], "../escape", path).is_err());
    }
}
