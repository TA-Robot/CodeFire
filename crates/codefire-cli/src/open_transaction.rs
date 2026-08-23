use super::*;
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const OPEN_TRANSACTION_TYPE: &str = "codefire_open_transaction";
const OPEN_TRANSACTION_VERSION: u64 = 1;
const OPEN_TRANSACTION_FILE: &str = "open_transaction.json";
pub(crate) const OPEN_TRANSACTION_BYTE_LIMIT: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenPhase {
    Prepared,
    TargetPublished,
    RegistryWritten,
    ActiveStateWritten,
    BranchWritten,
    IdempotencyWritten,
}

impl OpenPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::TargetPublished => "target_published",
            Self::RegistryWritten => "registry_written",
            Self::ActiveStateWritten => "active_state_written",
            Self::BranchWritten => "branch_written",
            Self::IdempotencyWritten => "idempotency_written",
        }
    }

    fn parse(value: &str, path: &Path) -> Result<Self, CliError> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "target_published" => Ok(Self::TargetPublished),
            "registry_written" => Ok(Self::RegistryWritten),
            "active_state_written" => Ok(Self::ActiveStateWritten),
            "branch_written" => Ok(Self::BranchWritten),
            "idempotency_written" => Ok(Self::IdempotencyWritten),
            _ => Err(invalid_transaction(path, "unsupported phase")),
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Prepared => Some(Self::TargetPublished),
            Self::TargetPublished => Some(Self::RegistryWritten),
            Self::RegistryWritten => Some(Self::ActiveStateWritten),
            Self::ActiveStateWritten => Some(Self::BranchWritten),
            Self::BranchWritten => Some(Self::IdempotencyWritten),
            Self::IdempotencyWritten => None,
        }
    }
}

#[derive(Debug)]
struct OpenTransaction {
    path: PathBuf,
    record: Value,
    marker_identity: Option<secure_worktree::StableFsIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonArtifact {
    Missing,
    Exact,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetArtifact {
    Ready,
    Exact,
}

enum PublishedTargetCapability {
    Existing(ExistingWorktree),
    Secure(secure_worktree::SecureWorktree),
}

impl PublishedTargetCapability {
    fn ensure_path_identity(&self) -> std::io::Result<()> {
        match self {
            Self::Existing(target) => target.ensure_path_identity(),
            Self::Secure(target) => target.ensure_path_identity(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchArtifact {
    Before,
    BeforeObjectStored,
    After,
}

#[derive(Debug)]
struct OpenEvidence {
    target: TargetArtifact,
    registry: JsonArtifact,
    active_state: JsonArtifact,
    branch: BranchArtifact,
    idempotency: JsonArtifact,
}

pub(crate) fn open_transaction_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".codefire").join(OPEN_TRANSACTION_FILE)
}

pub(crate) fn open_transaction_removal_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("open_transaction.removing.json")
}

pub(crate) fn open_branch_from(
    start: &Path,
    options: &OpenOptions,
) -> Result<OpenResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    let cf = repo_root.join(".codefire");
    let objects = cf.join("objects");
    let repo_json = read_json(&cf.join("repo.json"))?;
    let repository_id = required_string(&repo_json, &["repository_id"])?;

    close_transaction::reject_pending_for_other_mutation(&repo_root)?;

    // A pending transaction is repository authority. Load it before resolving
    // the requested branch so a conflicting retry cannot be masked by an
    // unrelated missing/corrupt branch record.
    if let Some(transaction) = OpenTransaction::load(&repo_root)? {
        let requested_target = resolve_path_without_symlinks(&options.path).map_err(|error| {
            with_branch_operation_context(
                error,
                &repo_root,
                Some(&options.branch),
                Some(&options.path),
            )
        })?;
        let stored_base = transaction.base_commit()?;
        let payload =
            open_idempotency_payload(options, &repo_root, &requested_target, &stored_base);
        transaction.verify_request(
            &repo_root,
            &repository_id,
            &payload,
            options.idempotency_key.as_deref(),
        )?;
        if options.dry_run {
            return Err(CliError::InvalidRepository(format!(
                "pending open transaction requires an exact live retry before dry-run planning: {}",
                transaction.path.display()
            )));
        }
        let validated_base = codefire_store::validate_sealed_commit_pinned(&objects, &stored_base)?;
        return resume_transaction(transaction, options, None, true, &validated_base);
    }

    let branch = load_branch_record(&repo_root, &options.branch).map_err(|error| {
        with_branch_operation_context(error, &repo_root, Some(&options.branch), None)
    })?;
    let branch_head = required_string(&branch, &["head"])?;
    let validated_base = codefire_store::validate_sealed_commit_pinned(&objects, &branch_head)?;
    let (target, target_resolution) =
        resolve_open_target(&repo_root, &options.path, &options.branch).map_err(|error| {
            with_branch_operation_context(
                error,
                &repo_root,
                Some(&options.branch),
                Some(&options.path),
            )
        })?;
    let payload = open_idempotency_payload(options, &repo_root, &target, &branch_head);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_open_idempotency(&repo_root, key, &payload)? {
                return Ok(result);
            }
        }
    }

    if target.exists() && !is_empty_dir(&target)? {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.branch),
            Some(&target),
            format!(
                "target path must not exist or must be empty: {}",
                target.display()
            ),
        ));
    }
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    if registry_path.exists() {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.branch),
            Some(&target),
            format!("branch is already open: {}", options.branch),
        ));
    }
    let state = branch_state(&branch);
    if state != "closed" {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.branch),
            Some(&target),
            format!(
                "branch {} is {state}; recover or close the existing open branch state before opening it again",
                options.branch
            ),
        ));
    }

    let opened_at = now_iso_utc();
    let open_instance_id = open_instance_id(&options.branch, &target, &opened_at);
    let active_path = cf.join("active").join(&open_instance_id);
    let plan = open_operation_plan(
        options,
        &repo_root,
        &target,
        &branch_head,
        &registry_path,
        &active_path,
    );
    if options.dry_run {
        return Ok(OpenResult {
            branch: options.branch.clone(),
            open_dir: target,
            plan,
        });
    }

    let mut transaction = OpenTransaction::new(
        &repo_root,
        &repository_id,
        branch,
        &target,
        &target_resolution,
        &payload,
        options.idempotency_key.as_deref(),
        opened_at,
        open_instance_id,
        plan,
    )?;
    maybe_change_open_target_before_use_for_test(&options.path)?;
    maybe_swap_open_target_ancestor_before_use_for_test(&target)?;
    let pinned_boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    target_resolution.ensure_current().map_err(|error| {
        open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&target),
            &pinned_boundary,
            "identity_changed_during_use",
            format!("open target ancestor changed before staging: {error}"),
        )
    })?;
    maybe_swap_open_target_ancestor_after_validation_for_test(&target)?;
    ensure_target_unchanged(&transaction, options, true)?;
    let staged = prepare_staged_target(
        &transaction,
        options,
        true,
        &validated_base,
        Some(&target_resolution),
    )?;
    transaction.record_staging_parent(&staged)?;
    maybe_publish_foreign_open_marker_before_save_for_test(&repo_root)?;
    transaction.save_new()?;
    maybe_fail_open_transaction_for_test(&repo_root, "after_prepared")?;
    resume_transaction(transaction, options, Some(staged), false, &validated_base)
}

impl OpenTransaction {
    #[allow(clippy::too_many_arguments)]
    fn new(
        repo_root: &Path,
        repository_id: &str,
        branch_before: Value,
        target: &Path,
        target_resolution: &secure_worktree::PinnedTargetResolution,
        payload: &Value,
        idempotency_key: Option<&str>,
        opened_at: String,
        open_instance_id: String,
        plan: Value,
    ) -> Result<Self, CliError> {
        let path = open_transaction_path(repo_root);
        let cf = repo_root.join(".codefire");
        let branch_name = required_string(&branch_before, &["name"])?;
        let base_commit = required_string(&branch_before, &["head"])?;
        if branch_state(&branch_before) != "closed" {
            return Err(invalid_transaction(
                &path,
                "new transaction branch preimage must be closed",
            ));
        }
        let mut branch_after = branch_before.clone();
        branch_after["state"] = Value::String("open-clean".to_string());
        let branch_object_id = codefire_store::object_id("branch", &branch_after)?;
        let boundary = fs::canonicalize(repo_root.parent().unwrap_or(repo_root))?;
        let registry_path = opened_registry_path(repo_root, &branch_name);
        let active_state_path = cf.join("active").join(&open_instance_id);
        let staging_owner = json!({
            "type": "codefire_open_staging_owner",
            "version": 1,
            "repo": repo_root,
            "repository_id": repository_id,
            "branch": branch_name,
            "target": target,
            "base_commit": base_commit,
        });
        let staging_name = staging_name(&staging_owner)?;
        let resolution_identity_chain = target_resolution
            .identity_chain()?
            .into_iter()
            .map(|(relative, identity)| {
                json!({
                    "relative": relative,
                    "identity": stable_identity_json(identity),
                })
            })
            .collect::<Vec<_>>();
        let open_marker = expected_open_marker(
            &cf,
            repository_id,
            &branch_name,
            &base_commit,
            target,
            &open_instance_id,
            &opened_at,
        );
        let registry = expected_registry(
            &branch_name,
            &base_commit,
            target,
            &open_instance_id,
            &active_state_path,
        );
        let active_state = json!({"state": "open-clean"});
        let result = json!({
            "branch": branch_name,
            "open_dir": target,
            "plan": plan,
        });
        let started_at = now_iso_utc();
        let idempotency = match idempotency_key {
            Some(key) => {
                require_idempotency_key(key)?;
                let record = json!({
                    "version": 1,
                    "command": "open",
                    "key": key,
                    "payload_hash": idempotency_payload_hash(payload)?,
                    "payload": payload,
                    "result": &result,
                    "created_at": &started_at,
                });
                json!({
                    "path": idempotency_record_path(repo_root, "open", key),
                    "record": record,
                })
            }
            None => Value::Null,
        };
        let transaction_id = random_transaction_id()?;
        let record = json!({
            "type": OPEN_TRANSACTION_TYPE,
            "version": OPEN_TRANSACTION_VERSION,
            "status": "pending",
            "transaction_id": transaction_id,
            "phase": OpenPhase::Prepared.as_str(),
            "repository": {"path": repo_root, "repository_id": repository_id},
            "request": {
                "payload": payload,
                "payload_hash": idempotency_payload_hash(payload)?,
                "idempotency_key": idempotency_key,
            },
            "branch": {
                "name": branch_name,
                "base_commit": base_commit,
                "before": branch_before,
                "after": branch_after,
                "object_id": branch_object_id,
            },
            "open": {
                "target": target,
                "workspace_boundary": boundary,
                "opened_at": opened_at,
                "open_instance_id": open_instance_id,
                "staging_name": staging_name,
                "staging_owner": staging_owner,
                "resolution": {
                    "identity_chain": resolution_identity_chain,
                    "staging_parent": Value::Null,
                },
                "marker": open_marker,
            },
            "registry": {"path": registry_path, "record": registry},
            "active_state": {
                "path": active_state_path.join("state.json"),
                "record": active_state,
            },
            "idempotency": idempotency,
            "result": result,
            "started_at": started_at,
            "updated_at": started_at,
        });
        let transaction = Self {
            path,
            record,
            marker_identity: None,
        };
        transaction.validate(repo_root)?;
        Ok(transaction)
    }

    fn load(repo_root: &Path) -> Result<Option<Self>, CliError> {
        let canonical_path = open_transaction_path(repo_root);
        let removal_path = open_transaction_removal_path(repo_root);
        let canonical = read_marker_with_identity(&canonical_path)?;
        let removal = read_marker_with_identity(&removal_path)?;
        let (path, bytes, marker_identity) = match (canonical, removal) {
            (None, None) => return Ok(None),
            (Some(_), Some(_)) => {
                return Err(invalid_transaction(
                    &canonical_path,
                    "both the canonical marker and its removal quarantine exist",
                ))
            }
            (Some((bytes, identity)), None) => (canonical_path, bytes, identity),
            (None, Some((bytes, identity))) => (removal_path, bytes, identity),
        };
        let record: Value = serde_json::from_slice(&bytes)
            .map_err(|error| invalid_transaction(&path, &format!("invalid JSON: {error}")))?;
        let transaction = Self {
            path,
            record,
            marker_identity: Some(marker_identity),
        };
        transaction.validate(repo_root)?;
        if transaction.path == open_transaction_removal_path(repo_root)
            && transaction.phase()? != OpenPhase::IdempotencyWritten
        {
            return Err(invalid_transaction(
                &transaction.path,
                "a quarantined marker must be at the final durable phase",
            ));
        }
        Ok(Some(transaction))
    }

    fn validate(&self, repo_root: &Path) -> Result<(), CliError> {
        let record = &self.record;
        if string(record, &["type"], &self.path)? != OPEN_TRANSACTION_TYPE
            || u64_field(record, &["version"], &self.path)? != OPEN_TRANSACTION_VERSION
            || string(record, &["status"], &self.path)? != "pending"
        {
            return Err(invalid_transaction(
                &self.path,
                "invalid type/version/status",
            ));
        }
        let transaction_id = string(record, &["transaction_id"], &self.path)?;
        let suffix = transaction_id
            .strip_prefix("open_tx_")
            .filter(|suffix| {
                suffix.len() == 32 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .ok_or_else(|| invalid_transaction(&self.path, "invalid transaction_id"))?;
        let _ = suffix;
        self.phase()?;
        for field in ["started_at", "updated_at"] {
            if string(record, &[field], &self.path)?.is_empty() {
                return Err(invalid_transaction(&self.path, "empty timestamp"));
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
        let payload = value(record, &["request", "payload"], &self.path)?;
        if string(record, &["request", "payload_hash"], &self.path)?
            != idempotency_payload_hash(payload)?
        {
            return Err(invalid_transaction(
                &self.path,
                "request payload hash mismatch",
            ));
        }
        let branch_name = string(record, &["branch", "name"], &self.path)?;
        let base_commit = string(record, &["branch", "base_commit"], &self.path)?;
        let before = value(record, &["branch", "before"], &self.path)?;
        let after = value(record, &["branch", "after"], &self.path)?;
        if required_string(before, &["name"])? != branch_name
            || required_string(before, &["head"])? != base_commit
            || branch_state(before) != "closed"
        {
            return Err(invalid_transaction(&self.path, "invalid branch preimage"));
        }
        let mut expected_after = before.clone();
        expected_after["state"] = Value::String("open-clean".to_string());
        if after != &expected_after
            || string(record, &["branch", "object_id"], &self.path)?
                != codefire_store::object_id("branch", after)?
        {
            return Err(invalid_transaction(&self.path, "invalid branch postimage"));
        }
        let target = path_field(record, &["open", "target"], &self.path)?;
        let opened_at = string(record, &["open", "opened_at"], &self.path)?;
        let open_instance = string(record, &["open", "open_instance_id"], &self.path)?;
        if open_instance != open_instance_id(&branch_name, &target, &opened_at) {
            return Err(invalid_transaction(
                &self.path,
                "open instance identity mismatch",
            ));
        }
        let expected_boundary = fs::canonicalize(repo_root.parent().unwrap_or(repo_root))?;
        if path_field(record, &["open", "workspace_boundary"], &self.path)? != expected_boundary {
            return Err(invalid_transaction(
                &self.path,
                "workspace boundary mismatch",
            ));
        }
        recorded_resolution_identity_chain(self)?;
        if self.marker_identity.is_some() && recorded_staging_parent_identity(self)?.is_none() {
            return Err(invalid_transaction(
                &self.path,
                "pending marker is missing staged publication-parent identity",
            ));
        }
        let owner = value(record, &["open", "staging_owner"], &self.path)?;
        let expected_owner = json!({
            "type": "codefire_open_staging_owner",
            "version": 1,
            "repo": repo_root,
            "repository_id": string(record, &["repository", "repository_id"], &self.path)?,
            "branch": branch_name,
            "target": target,
            "base_commit": base_commit,
        });
        if owner != &expected_owner
            || string(record, &["open", "staging_name"], &self.path)? != staging_name(owner)?
        {
            return Err(invalid_transaction(&self.path, "staging subject mismatch"));
        }
        let cf = repo_root.join(".codefire");
        if value(record, &["open", "marker"], &self.path)?
            != &expected_open_marker(
                &cf,
                &string(record, &["repository", "repository_id"], &self.path)?,
                &branch_name,
                &base_commit,
                &target,
                &open_instance,
                &opened_at,
            )
        {
            return Err(invalid_transaction(
                &self.path,
                "open marker subject mismatch",
            ));
        }
        let active_path = cf.join("active").join(&open_instance);
        if path_field(record, &["registry", "path"], &self.path)?
            != opened_registry_path(repo_root, &branch_name)
            || value(record, &["registry", "record"], &self.path)?
                != &expected_registry(
                    &branch_name,
                    &base_commit,
                    &target,
                    &open_instance,
                    &active_path,
                )
            || path_field(record, &["active_state", "path"], &self.path)?
                != active_path.join("state.json")
            || value(record, &["active_state", "record"], &self.path)?
                != &json!({"state": "open-clean"})
        {
            return Err(invalid_transaction(
                &self.path,
                "registry or active-state subject mismatch",
            ));
        }
        let result = value(record, &["result"], &self.path)?;
        if string(result, &["branch"], &self.path)? != branch_name
            || path_field(result, &["open_dir"], &self.path)? != target
            || string(result, &["plan", "command"], &self.path)? != "open"
            || value(result, &["plan", "dry_run"], &self.path)?.as_bool() != Some(false)
            || string(result, &["plan", "base_commit"], &self.path)? != base_commit
        {
            return Err(invalid_transaction(&self.path, "result subject mismatch"));
        }
        self.validate_idempotency(repo_root, payload, result)?;
        Ok(())
    }

    fn validate_idempotency(
        &self,
        repo_root: &Path,
        payload: &Value,
        result: &Value,
    ) -> Result<(), CliError> {
        let key = optional_string(&self.record, &["request", "idempotency_key"], &self.path)?;
        let subject = value(&self.record, &["idempotency"], &self.path)?;
        match key {
            None if subject.is_null() => Ok(()),
            Some(key) if subject.is_object() => {
                require_idempotency_key(&key)?;
                let record = value(subject, &["record"], &self.path)?;
                if path_field(subject, &["path"], &self.path)?
                    != idempotency_record_path(repo_root, "open", &key)
                    || u64_field(record, &["version"], &self.path)? != 1
                    || string(record, &["command"], &self.path)? != "open"
                    || string(record, &["key"], &self.path)? != key
                    || string(record, &["payload_hash"], &self.path)?
                        != idempotency_payload_hash(payload)?
                    || value(record, &["payload"], &self.path)? != payload
                    || value(record, &["result"], &self.path)? != result
                    || string(record, &["created_at"], &self.path)?.is_empty()
                {
                    return Err(invalid_transaction(
                        &self.path,
                        "idempotency subject mismatch",
                    ));
                }
                Ok(())
            }
            _ => Err(invalid_transaction(
                &self.path,
                "idempotency key and subject disagree",
            )),
        }
    }

    fn verify_request(
        &self,
        repo_root: &Path,
        repository_id: &str,
        payload: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<(), CliError> {
        let stored_key =
            optional_string(&self.record, &["request", "idempotency_key"], &self.path)?;
        if path_field(&self.record, &["repository", "path"], &self.path)? != repo_root
            || string(&self.record, &["repository", "repository_id"], &self.path)? != repository_id
            || value(&self.record, &["request", "payload"], &self.path)? != payload
            || string(&self.record, &["request", "payload_hash"], &self.path)?
                != idempotency_payload_hash(payload)?
            || stored_key.as_deref() != idempotency_key
        {
            return Err(CliError::IdempotencyConflict(format!(
                "pending open transaction conflicts with the requested branch, target, head, or idempotency key: {}",
                self.path.display()
            )));
        }
        Ok(())
    }

    fn phase(&self) -> Result<OpenPhase, CliError> {
        OpenPhase::parse(&string(&self.record, &["phase"], &self.path)?, &self.path)
    }

    fn record_staging_parent(&mut self, staged: &StagedWorktree) -> Result<(), CliError> {
        if !value(
            &self.record,
            &["open", "resolution", "staging_parent"],
            &self.path,
        )?
        .is_null()
        {
            return Err(invalid_transaction(
                &self.path,
                "staging-parent identity was already recorded",
            ));
        }
        let identity = staged.publication_parent_identity()?;
        self.record["open"]["resolution"]["staging_parent"] = json!({
            "relative": staged.relative_publication_parent(),
            "identity": stable_identity_json(identity),
        });
        let repo_root = path_field(&self.record, &["repository", "path"], &self.path)?;
        self.validate(&repo_root)
    }

    fn save_new(&mut self) -> Result<(), CliError> {
        if recorded_staging_parent_identity(self)?.is_none() {
            return Err(invalid_transaction(
                &self.path,
                "new transaction is missing staged publication-parent identity",
            ));
        }
        let repo_root = path_field(&self.record, &["repository", "path"], &self.path)?;
        if read_marker_with_identity(&open_transaction_removal_path(&repo_root))?.is_some() {
            return Err(invalid_transaction(
                &self.path,
                "a transaction removal quarantine already exists",
            ));
        }
        let bytes = self.render()?;
        secure_worktree::write_file_create_noreplace_nofollow(
            &self.path,
            &unique_temp_path(&self.path),
            &bytes,
        )
        .map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                invalid_transaction(&self.path, "a pending transaction already exists")
            } else {
                CliError::Io(error)
            }
        })?;
        self.bind_written_marker(&bytes)
    }

    fn advance(&mut self, next: OpenPhase) -> Result<(), CliError> {
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
        self.record["phase"] = Value::String(next.as_str().to_string());
        self.record["updated_at"] = Value::String(now_iso_utc());
        self.write()
    }

    fn write(&mut self) -> Result<(), CliError> {
        let bytes = self.render()?;
        secure_worktree::write_file_atomic_nofollow(
            &self.path,
            &unique_temp_path(&self.path),
            &bytes,
        )?;
        self.bind_written_marker(&bytes)
    }

    fn render(&self) -> Result<Vec<u8>, CliError> {
        let mut bytes = serde_json::to_vec_pretty(&self.record)?;
        bytes.push(b'\n');
        if bytes.len() as u64 > OPEN_TRANSACTION_BYTE_LIMIT {
            return Err(invalid_transaction(
                &self.path,
                &format!(
                    "marker exceeds the {}-byte limit",
                    OPEN_TRANSACTION_BYTE_LIMIT
                ),
            ));
        }
        Ok(bytes)
    }

    fn bind_written_marker(&mut self, expected: &[u8]) -> Result<(), CliError> {
        let (actual, identity) = secure_worktree::read_file_nofollow_bounded_with_identity(
            &self.path,
            OPEN_TRANSACTION_BYTE_LIMIT,
        )?;
        if actual != expected {
            return Err(invalid_transaction(
                &self.path,
                "marker identity changed immediately after publication",
            ));
        }
        self.marker_identity = Some(identity);
        Ok(())
    }

    fn result(&self) -> Result<OpenResult, CliError> {
        let result = value(&self.record, &["result"], &self.path)?;
        Ok(OpenResult {
            branch: string(result, &["branch"], &self.path)?,
            open_dir: path_field(result, &["open_dir"], &self.path)?,
            plan: value(result, &["plan"], &self.path)?.clone(),
        })
    }

    fn target(&self) -> Result<PathBuf, CliError> {
        path_field(&self.record, &["open", "target"], &self.path)
    }

    fn branch_name(&self) -> Result<String, CliError> {
        string(&self.record, &["branch", "name"], &self.path)
    }

    fn base_commit(&self) -> Result<String, CliError> {
        string(&self.record, &["branch", "base_commit"], &self.path)
    }

    fn archive_and_remove(&mut self) -> Result<(), CliError> {
        if self.record.get("completed_at").is_none() {
            self.record["completed_at"] = Value::String(now_iso_utc());
            self.write()?;
        }
        let active_dir_path = active_state_dir_path(self)?;
        let active_dir = active_state::ActiveStateDir::bind(&active_dir_path)?;
        let transaction_id = string(&self.record, &["transaction_id"], &self.path)?;
        let suffix = transaction_id
            .strip_prefix("open_tx_")
            .ok_or_else(|| invalid_transaction(&self.path, "invalid transaction id"))?;
        let archive_name = format!("open_transaction.completed.{suffix}.json");
        let mut archive = self.record.clone();
        archive["status"] = Value::String("completed".to_string());
        write_active_json_new_or_exact(
            &active_dir,
            Path::new(&archive_name),
            &archive,
            OPEN_TRANSACTION_BYTE_LIMIT,
            "completed open transaction archive",
        )?;
        maybe_fail_open_transaction_for_test(
            &path_field(&self.record, &["repository", "path"], &self.path)?,
            "after_complete_archive_before_marker_removal",
        )?;
        maybe_swap_open_marker_before_removal_for_test(
            &path_field(&self.record, &["repository", "path"], &self.path)?,
            &self.path,
        )?;
        let marker_identity = self.marker_identity.ok_or_else(|| {
            invalid_transaction(&self.path, "marker removal is missing bound identity")
        })?;
        let repo_root = path_field(&self.record, &["repository", "path"], &self.path)?;
        secure_worktree::remove_file_quarantined_nofollow(
            &self.path,
            &open_transaction_removal_path(&repo_root),
            marker_identity,
        )?;
        maybe_fail_open_transaction_for_test(
            &path_field(&self.record, &["repository", "path"], &self.path)?,
            "after_marker_removed",
        )
    }
}

fn read_marker_with_identity(
    path: &Path,
) -> Result<Option<(Vec<u8>, secure_worktree::StableFsIdentity)>, CliError> {
    match secure_worktree::read_file_nofollow_bounded_with_identity(
        path,
        OPEN_TRANSACTION_BYTE_LIMIT,
    ) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::Io(error)),
    }
}

fn write_active_json_new_or_exact(
    active_dir: &active_state::ActiveStateDir,
    relative: &Path,
    value: &Value,
    limit: u64,
    context: &str,
) -> Result<(), CliError> {
    let mut expected = serde_json::to_vec_pretty(value)?;
    expected.push(b'\n');
    if expected.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "{context} exceeds the {limit}-byte limit: {}",
            active_dir.display_path(relative).display()
        )));
    }
    match active_dir.write_new(relative, &expected, limit) {
        Ok(()) => Ok(()),
        Err(create_error) => {
            let Some(actual) = active_dir.read_optional(relative, limit)? else {
                return Err(create_error.into());
            };
            if actual == expected {
                Ok(())
            } else {
                Err(CliError::InvalidRepository(format!(
                    "{context} already exists with divergent contents: {}",
                    active_dir.display_path(relative).display()
                )))
            }
        }
    }
}

fn bind_verified_published_target(
    transaction: &OpenTransaction,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<PublishedTargetCapability, CliError> {
    let target_path = transaction.target()?;
    validated_base.ensure_subject_identity()?;
    let target = ExistingWorktree::bind(&target_path).map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("published target cannot be retained no-follow: {error}"),
        )
    })?;
    if !source_manifest_unchanged_existing(validated_base, &target)? {
        return Err(invalid_transaction(
            &transaction.path,
            "published target content differs from the sealed base",
        ));
    }
    target.ensure_path_identity().map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("published target identity changed during verification: {error}"),
        )
    })?;
    validated_base.ensure_subject_identity()?;
    Ok(PublishedTargetCapability::Existing(target))
}

fn ensure_open_publication_subjects(
    transaction: &OpenTransaction,
    validated_base: &codefire_store::ValidatedSealedCommit,
    target: &PublishedTargetCapability,
    boundary: &str,
) -> Result<(), CliError> {
    validated_base.ensure_subject_identity()?;
    target.ensure_path_identity().map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("published target identity changed {boundary}: {error}"),
        )
    })?;
    Ok(())
}

fn resume_transaction(
    mut transaction: OpenTransaction,
    options: &OpenOptions,
    staged: Option<StagedWorktree>,
    recovery: bool,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<OpenResult, CliError> {
    let repo_root = path_field(
        &transaction.record,
        &["repository", "path"],
        &transaction.path,
    )?;
    if validated_base.commit_id() != transaction.base_commit()? {
        return Err(invalid_transaction(
            &transaction.path,
            "validated sealed base does not match transaction base",
        ));
    }
    let mut evidence = inspect_evidence(&transaction, recovery, validated_base)?;
    validate_phase_evidence(transaction.phase()?, &evidence, &transaction.path)?;
    let mut target_capability = if evidence.target == TargetArtifact::Exact {
        Some(bind_verified_published_target(
            &transaction,
            validated_base,
        )?)
    } else {
        None
    };
    let mut target_published_now = false;

    if transaction.phase()? == OpenPhase::Prepared {
        if evidence.target == TargetArtifact::Ready {
            target_capability = Some(PublishedTargetCapability::Secure(publish_target(
                &transaction,
                options,
                staged,
                validated_base,
            )?));
            target_published_now = true;
            maybe_fail_open_transaction_for_test(&repo_root, "after_target_publish_before_phase")?;
            evidence.target = TargetArtifact::Exact;
        }
        transaction.advance(OpenPhase::TargetPublished)?;
        maybe_fail_open_transaction_for_test(&repo_root, "after_target_published")?;
    }

    let target_capability = target_capability.ok_or_else(|| {
        invalid_transaction(
            &transaction.path,
            "published target capability is unavailable for metadata publication",
        )
    })?;

    if transaction.phase()? == OpenPhase::TargetPublished {
        if target_published_now {
            maybe_swap_open_target_after_fingerprint_for_test(&repo_root, &transaction.target()?)?;
        }
        if evidence.registry == JsonArtifact::Missing {
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "before opened-registry publication",
            )?;
            write_json_atomic(
                &path_field(
                    &transaction.record,
                    &["registry", "path"],
                    &transaction.path,
                )?,
                value(
                    &transaction.record,
                    &["registry", "record"],
                    &transaction.path,
                )?,
            )?;
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "after opened-registry publication",
            )?;
            maybe_fail_open_transaction_for_test(&repo_root, "after_registry_write_before_phase")?;
            evidence.registry = JsonArtifact::Exact;
        }
        transaction.advance(OpenPhase::RegistryWritten)?;
        maybe_fail_open_transaction_for_test(&repo_root, "after_registry_written")?;
    }

    if transaction.phase()? == OpenPhase::RegistryWritten {
        if evidence.active_state == JsonArtifact::Missing {
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "before active-state directory publication",
            )?;
            let active_dir =
                active_state::ActiveStateDir::create(&active_state_dir_path(&transaction)?)?;
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "after active-state directory publication",
            )?;
            maybe_fail_open_transaction_for_test(
                &repo_root,
                "after_active_directory_create_before_state",
            )?;
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "before active-state record publication",
            )?;
            write_active_json_atomic(
                &active_dir,
                Path::new("state.json"),
                value(
                    &transaction.record,
                    &["active_state", "record"],
                    &transaction.path,
                )?,
            )?;
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "after active-state record publication",
            )?;
            maybe_fail_open_transaction_for_test(
                &repo_root,
                "after_active_state_write_before_phase",
            )?;
            evidence.active_state = JsonArtifact::Exact;
        }
        transaction.advance(OpenPhase::ActiveStateWritten)?;
        maybe_fail_open_transaction_for_test(&repo_root, "after_active_state_written")?;
    }

    if transaction.phase()? == OpenPhase::ActiveStateWritten {
        if evidence.branch != BranchArtifact::After {
            if evidence.branch == BranchArtifact::Before {
                ensure_open_publication_subjects(
                    &transaction,
                    validated_base,
                    &target_capability,
                    "before branch-object publication",
                )?;
                let object_id = codefire_store::store_object(
                    &repo_root.join(".codefire").join("objects"),
                    "branch",
                    value(&transaction.record, &["branch", "after"], &transaction.path)?.clone(),
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
                ensure_open_publication_subjects(
                    &transaction,
                    validated_base,
                    &target_capability,
                    "after branch-object publication",
                )?;
                maybe_fail_open_transaction_for_test(
                    &repo_root,
                    "after_branch_object_before_reference",
                )?;
                evidence.branch = BranchArtifact::BeforeObjectStored;
            }
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "before branch-reference publication",
            )?;
            write_branch_record_reference(
                &repo_root,
                value(&transaction.record, &["branch", "after"], &transaction.path)?,
            )?;
            ensure_open_publication_subjects(
                &transaction,
                validated_base,
                &target_capability,
                "after branch-reference publication",
            )?;
            maybe_fail_open_transaction_for_test(
                &repo_root,
                "after_branch_reference_before_phase",
            )?;
            evidence.branch = BranchArtifact::After;
        }
        transaction.advance(OpenPhase::BranchWritten)?;
        maybe_fail_open_transaction_for_test(&repo_root, "after_branch_written")?;
    }

    if transaction.phase()? == OpenPhase::BranchWritten {
        if evidence.idempotency == JsonArtifact::Missing {
            let subject = value(&transaction.record, &["idempotency"], &transaction.path)?;
            if !subject.is_null() {
                ensure_open_publication_subjects(
                    &transaction,
                    validated_base,
                    &target_capability,
                    "before idempotency publication",
                )?;
                write_json_atomic(
                    &path_field(subject, &["path"], &transaction.path)?,
                    value(subject, &["record"], &transaction.path)?,
                )?;
                ensure_open_publication_subjects(
                    &transaction,
                    validated_base,
                    &target_capability,
                    "after idempotency publication",
                )?;
                maybe_fail_open_transaction_for_test(
                    &repo_root,
                    "after_idempotency_write_before_phase",
                )?;
            }
        }
        transaction.advance(OpenPhase::IdempotencyWritten)?;
        maybe_fail_open_transaction_for_test(&repo_root, "after_idempotency_written")?;
    }

    ensure_open_publication_subjects(
        &transaction,
        validated_base,
        &target_capability,
        "before completion validation",
    )?;
    validate_complete(&transaction, validated_base)?;
    let result = transaction.result()?;
    transaction.archive_and_remove()?;
    target_capability
        .ensure_path_identity()
        .map_err(CliError::Io)?;
    Ok(result)
}

fn inspect_evidence(
    transaction: &OpenTransaction,
    verify_materialization: bool,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<OpenEvidence, CliError> {
    let target = classify_target(transaction, verify_materialization, validated_base)?;
    let registry = classify_json(
        &path_field(
            &transaction.record,
            &["registry", "path"],
            &transaction.path,
        )?,
        value(
            &transaction.record,
            &["registry", "record"],
            &transaction.path,
        )?,
        "opened registry",
    )?;
    let active_state = classify_active_state(transaction)?;
    let branch = classify_branch(transaction)?;
    let idempotency_subject = value(&transaction.record, &["idempotency"], &transaction.path)?;
    let idempotency = if idempotency_subject.is_null() {
        JsonArtifact::NotApplicable
    } else {
        classify_json(
            &path_field(idempotency_subject, &["path"], &transaction.path)?,
            value(idempotency_subject, &["record"], &transaction.path)?,
            "open idempotency record",
        )?
    };
    Ok(OpenEvidence {
        target,
        registry,
        active_state,
        branch,
        idempotency,
    })
}

fn validate_phase_evidence(
    phase: OpenPhase,
    evidence: &OpenEvidence,
    path: &Path,
) -> Result<(), CliError> {
    let valid = match phase {
        OpenPhase::Prepared => {
            matches!(
                evidence.target,
                TargetArtifact::Ready | TargetArtifact::Exact
            ) && evidence.registry == JsonArtifact::Missing
                && evidence.active_state == JsonArtifact::Missing
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeObjectStored
                )
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Missing | JsonArtifact::NotApplicable
                )
        }
        OpenPhase::TargetPublished => {
            evidence.target == TargetArtifact::Exact
                && matches!(
                    evidence.registry,
                    JsonArtifact::Missing | JsonArtifact::Exact
                )
                && evidence.active_state == JsonArtifact::Missing
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeObjectStored
                )
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Missing | JsonArtifact::NotApplicable
                )
        }
        OpenPhase::RegistryWritten => {
            evidence.target == TargetArtifact::Exact
                && evidence.registry == JsonArtifact::Exact
                && matches!(
                    evidence.active_state,
                    JsonArtifact::Missing | JsonArtifact::Exact
                )
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before | BranchArtifact::BeforeObjectStored
                )
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Missing | JsonArtifact::NotApplicable
                )
        }
        OpenPhase::ActiveStateWritten => {
            evidence.target == TargetArtifact::Exact
                && evidence.registry == JsonArtifact::Exact
                && evidence.active_state == JsonArtifact::Exact
                && matches!(
                    evidence.branch,
                    BranchArtifact::Before
                        | BranchArtifact::BeforeObjectStored
                        | BranchArtifact::After
                )
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Missing | JsonArtifact::NotApplicable
                )
        }
        OpenPhase::BranchWritten => {
            evidence.target == TargetArtifact::Exact
                && evidence.registry == JsonArtifact::Exact
                && evidence.active_state == JsonArtifact::Exact
                && evidence.branch == BranchArtifact::After
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Missing | JsonArtifact::Exact | JsonArtifact::NotApplicable
                )
        }
        OpenPhase::IdempotencyWritten => {
            evidence.target == TargetArtifact::Exact
                && evidence.registry == JsonArtifact::Exact
                && evidence.active_state == JsonArtifact::Exact
                && evidence.branch == BranchArtifact::After
                && matches!(
                    evidence.idempotency,
                    JsonArtifact::Exact | JsonArtifact::NotApplicable
                )
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

fn classify_target(
    transaction: &OpenTransaction,
    verify_materialization: bool,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<TargetArtifact, CliError> {
    let target = transaction.target()?;
    match fs::symlink_metadata(&target) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(TargetArtifact::Ready),
        Err(error) => return Err(CliError::Io(error)),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(invalid_transaction(
                &transaction.path,
                "open target is not a real directory",
            ))
        }
        Ok(_) => {}
    }
    if is_empty_dir(&target)? {
        return Ok(TargetArtifact::Ready);
    }
    let pinned_target = ExistingWorktree::bind(&target).map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("published target cannot be pinned no-follow: {error}"),
        )
    })?;
    let marker_path = target.join(".codefire-open");
    if classify_json(
        &marker_path,
        value(&transaction.record, &["open", "marker"], &transaction.path)?,
        "open target marker",
    )? != JsonArtifact::Exact
    {
        return Err(invalid_transaction(
            &transaction.path,
            "published target is foreign or missing its exact marker",
        ));
    }
    maybe_swap_open_target_before_fingerprint_for_test(
        &path_field(
            &transaction.record,
            &["repository", "path"],
            &transaction.path,
        )?,
        &target,
    )?;
    maybe_insert_open_target_symlink_before_fingerprint_for_test(
        &path_field(
            &transaction.record,
            &["repository", "path"],
            &transaction.path,
        )?,
        &target,
    )?;
    if verify_materialization
        && !source_manifest_unchanged_existing(validated_base, &pinned_target)?
    {
        return Err(invalid_transaction(
            &transaction.path,
            "published target content differs from the sealed base",
        ));
    }
    pinned_target.ensure_path_identity().map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("published target identity changed during verification: {error}"),
        )
    })?;
    Ok(TargetArtifact::Exact)
}

fn classify_branch(transaction: &OpenTransaction) -> Result<BranchArtifact, CliError> {
    let repo_root = path_field(
        &transaction.record,
        &["repository", "path"],
        &transaction.path,
    )?;
    let branch_name = transaction.branch_name()?;
    let actual = read_json(&branch_record_path(&repo_root, &branch_name))?;
    let before = value(
        &transaction.record,
        &["branch", "before"],
        &transaction.path,
    )?;
    let after = value(&transaction.record, &["branch", "after"], &transaction.path)?;
    let is_before = &actual == before;
    let is_after = &actual == after;
    if !is_before && !is_after {
        return Err(invalid_transaction(
            &transaction.path,
            "branch record differs from both transaction preimage and postimage",
        ));
    }
    let object_id = string(
        &transaction.record,
        &["branch", "object_id"],
        &transaction.path,
    )?;
    let objects = repo_root.join(".codefire").join("objects");
    let object_stored = match codefire_store::read_object(&objects, &object_id) {
        Ok(stored) if &stored == after => true,
        Ok(_) => {
            return Err(invalid_transaction(
                &transaction.path,
                "branch object differs from the planned postimage",
            ))
        }
        Err(codefire_store::StoreError::ObjectNotFound(id)) if id == object_id => false,
        Err(error) => return Err(error.into()),
    };
    match (is_before, object_stored) {
        (true, false) => Ok(BranchArtifact::Before),
        (true, true) => Ok(BranchArtifact::BeforeObjectStored),
        (false, true) => Ok(BranchArtifact::After),
        (false, false) => Err(invalid_transaction(
            &transaction.path,
            "published branch reference is missing its immutable branch object",
        )),
    }
}

fn active_state_dir_path(transaction: &OpenTransaction) -> Result<PathBuf, CliError> {
    let state_path = path_field(
        &transaction.record,
        &["active_state", "path"],
        &transaction.path,
    )?;
    state_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| invalid_transaction(&transaction.path, "active state has no parent"))
}

fn classify_active_state(transaction: &OpenTransaction) -> Result<JsonArtifact, CliError> {
    let active_dir_path = active_state_dir_path(transaction)?;
    let active_dir = match active_state::ActiveStateDir::bind(&active_dir_path) {
        Ok(active_dir) => active_dir,
        Err(error) if error.is_not_found() => return Ok(JsonArtifact::Missing),
        Err(error) => return Err(error.into()),
    };
    let Some(bytes) =
        active_dir.read_optional(Path::new("state.json"), REPOSITORY_JSON_BYTE_LIMIT)?
    else {
        return Ok(JsonArtifact::Missing);
    };
    let actual: Value = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "pending open transaction found malformed active state {}: {error}",
            active_dir.display_path(Path::new("state.json")).display()
        ))
    })?;
    let expected = value(
        &transaction.record,
        &["active_state", "record"],
        &transaction.path,
    )?;
    if &actual != expected {
        return Err(CliError::InvalidRepository(format!(
            "pending open transaction found divergent active state: {}",
            active_dir.display_path(Path::new("state.json")).display()
        )));
    }
    Ok(JsonArtifact::Exact)
}

fn classify_json(path: &Path, expected: &Value, subject: &str) -> Result<JsonArtifact, CliError> {
    let bytes = match secure_worktree::read_file_nofollow_bounded(path, REPOSITORY_JSON_BYTE_LIMIT)
    {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(JsonArtifact::Missing),
        Err(error) => return Err(CliError::Io(error)),
    };
    let actual: Value = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "pending open transaction found malformed {subject} {}: {error}",
            path.display()
        ))
    })?;
    if &actual != expected {
        return Err(CliError::InvalidRepository(format!(
            "pending open transaction found divergent {subject}: {}",
            path.display()
        )));
    }
    Ok(JsonArtifact::Exact)
}

fn stable_identity_json(identity: secure_worktree::StableFsIdentity) -> Value {
    json!({
        "scheme": identity.scheme(),
        "first": identity.first(),
        "second": identity.second(),
    })
}

fn recorded_resolution_identity_chain(
    transaction: &OpenTransaction,
) -> Result<Vec<(PathBuf, secure_worktree::StableFsIdentity)>, CliError> {
    let entries = value(
        &transaction.record,
        &["open", "resolution", "identity_chain"],
        &transaction.path,
    )?
    .as_array()
    .ok_or_else(|| {
        invalid_transaction(
            &transaction.path,
            "open.resolution.identity_chain must be a list",
        )
    })?;
    let target = transaction.target()?;
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let target_relative = target
        .strip_prefix(&boundary)
        .map_err(|_| invalid_transaction(&transaction.path, "target is outside boundary"))?;
    let target_component_count = target_relative.components().count();
    if entries.is_empty() || entries.len() > target_component_count.saturating_add(1) {
        return Err(invalid_transaction(
            &transaction.path,
            "resolution identity chain has an invalid length",
        ));
    }
    let mut chain = Vec::with_capacity(entries.len());
    let mut previous = None::<PathBuf>;
    for entry in entries {
        let relative = path_field(entry, &["relative"], &transaction.path)?;
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
                && !relative.as_os_str().is_empty()
            || !target_relative.starts_with(&relative)
        {
            return Err(invalid_transaction(
                &transaction.path,
                "resolution identity chain contains an unsafe relative path",
            ));
        }
        match &previous {
            None if !relative.as_os_str().is_empty() => {
                return Err(invalid_transaction(
                    &transaction.path,
                    "resolution identity chain must start at the boundary",
                ))
            }
            Some(previous) if relative.parent() != Some(previous.as_path()) => {
                return Err(invalid_transaction(
                    &transaction.path,
                    "resolution identity chain is not an adjacent prefix chain",
                ))
            }
            _ => {}
        }
        let identity_value = value(entry, &["identity"], &transaction.path)?;
        let identity = secure_worktree::StableFsIdentity::from_parts(
            &string(identity_value, &["scheme"], &transaction.path)?,
            u64_field(identity_value, &["first"], &transaction.path)?,
            u64_field(identity_value, &["second"], &transaction.path)?,
        )
        .map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("invalid resolution filesystem identity: {error}"),
            )
        })?;
        previous = Some(relative.clone());
        chain.push((relative, identity));
    }
    Ok(chain)
}

fn recorded_staging_parent_identity(
    transaction: &OpenTransaction,
) -> Result<Option<(PathBuf, secure_worktree::StableFsIdentity)>, CliError> {
    let subject = value(
        &transaction.record,
        &["open", "resolution", "staging_parent"],
        &transaction.path,
    )?;
    if subject.is_null() {
        return Ok(None);
    }
    let relative = path_field(subject, &["relative"], &transaction.path)?;
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let target = transaction.target()?;
    let expected_relative = target
        .strip_prefix(&boundary)
        .map_err(|_| invalid_transaction(&transaction.path, "target is outside boundary"))?
        .parent()
        .unwrap_or_else(|| Path::new(""));
    if relative != expected_relative {
        return Err(invalid_transaction(
            &transaction.path,
            "recorded staging parent is not the target parent",
        ));
    }
    let identity_value = value(subject, &["identity"], &transaction.path)?;
    let identity = secure_worktree::StableFsIdentity::from_parts(
        &string(identity_value, &["scheme"], &transaction.path)?,
        u64_field(identity_value, &["first"], &transaction.path)?,
        u64_field(identity_value, &["second"], &transaction.path)?,
    )
    .map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("invalid staging-parent filesystem identity: {error}"),
        )
    })?;
    Ok(Some((relative, identity)))
}

fn bind_recorded_target_resolution(
    transaction: &OpenTransaction,
) -> Result<secure_worktree::PinnedTargetResolution, CliError> {
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let target = transaction.target()?;
    let resolution =
        secure_worktree::PinnedTargetResolution::bind(&boundary, &target).map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("recorded target resolution cannot be rebound: {error}"),
            )
        })?;
    for (relative, identity) in recorded_resolution_identity_chain(transaction)? {
        resolution
            .ensure_relative_identity(&relative, identity)
            .map_err(|error| {
                invalid_transaction(
                    &transaction.path,
                    &format!("recorded target ancestor identity changed: {error}"),
                )
            })?;
    }
    let (relative, identity) = recorded_staging_parent_identity(transaction)?.ok_or_else(|| {
        invalid_transaction(
            &transaction.path,
            "pending transaction is missing its staged publication-parent identity",
        )
    })?;
    resolution
        .ensure_relative_identity(&relative, identity)
        .map_err(|error| {
            invalid_transaction(
                &transaction.path,
                &format!("recorded staging-parent identity changed: {error}"),
            )
        })?;
    resolution.ensure_current().map_err(|error| {
        invalid_transaction(
            &transaction.path,
            &format!("recorded target resolution changed: {error}"),
        )
    })?;
    Ok(resolution)
}

fn prepare_staged_target(
    transaction: &OpenTransaction,
    options: &OpenOptions,
    inject_race: bool,
    validated_base: &codefire_store::ValidatedSealedCommit,
    target_resolution: Option<&secure_worktree::PinnedTargetResolution>,
) -> Result<StagedWorktree, CliError> {
    let repo_root = path_field(
        &transaction.record,
        &["repository", "path"],
        &transaction.path,
    )?;
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let target = transaction.target()?;
    let staging_name = string(
        &transaction.record,
        &["open", "staging_name"],
        &transaction.path,
    )?;
    let staging_path = target
        .parent()
        .ok_or_else(|| invalid_transaction(&transaction.path, "target has no parent"))?
        .join(&staging_name);
    let owner = value(
        &transaction.record,
        &["open", "staging_owner"],
        &transaction.path,
    )?;
    let mut owner_bytes = serde_json::to_vec(owner)?;
    owner_bytes.push(b'\n');
    let rebound_resolution;
    let resolution = match target_resolution {
        Some(resolution) => resolution,
        None => {
            rebound_resolution = bind_recorded_target_resolution(transaction)?;
            &rebound_resolution
        }
    };
    let staged = StagedWorktree::prepare_pinned(resolution, staging_name.as_ref(), &owner_bytes)
        .map_err(|error| {
            open_target_resolution_error(
                &repo_root,
                &options.branch,
                &options.path,
                Some(&target),
                &boundary,
                "identity_changed_during_use",
                format!("secure open-target preparation failed: {error}"),
            )
        })?;
    if inject_race {
        maybe_insert_nested_open_target_symlink_for_test(&options.path, &target, &staging_path)
            .map_err(|error| {
                open_target_resolution_error(
                    &repo_root,
                    &options.branch,
                    &options.path,
                    Some(&target),
                    &boundary,
                    "identity_changed_during_use",
                    format!("secure open-target race fixture failed: {error}"),
                )
            })?;
    }
    materialize_commit(validated_base, &staged).map_err(|error| match error {
        CliError::Io(error) => {
            let reason = match error.kind() {
                ErrorKind::AlreadyExists
                | ErrorKind::InvalidInput
                | ErrorKind::PermissionDenied => "identity_changed_during_use",
                _ => "materialization_write_failed",
            };
            open_target_resolution_error(
                &repo_root,
                &options.branch,
                &options.path,
                Some(&target),
                &boundary,
                reason,
                format!("secure open-target materialization failed: {error}"),
            )
        }
        error => error,
    })?;
    staged
        .write_file(
            Path::new(".codefire-open"),
            serde_json::to_string_pretty(value(
                &transaction.record,
                &["open", "marker"],
                &transaction.path,
            )?)?
            .as_bytes(),
        )
        .map_err(|error| {
            open_target_resolution_error(
                &repo_root,
                &options.branch,
                &options.path,
                Some(&target),
                &boundary,
                "identity_changed_during_use",
                format!("secure open-target marker write failed: {error}"),
            )
        })?;
    Ok(staged)
}

fn publish_target(
    transaction: &OpenTransaction,
    options: &OpenOptions,
    staged: Option<StagedWorktree>,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<secure_worktree::SecureWorktree, CliError> {
    let repo_root = path_field(
        &transaction.record,
        &["repository", "path"],
        &transaction.path,
    )?;
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let target = transaction.target()?;
    let staged = match staged {
        Some(staged) => staged,
        None => prepare_staged_target(transaction, options, false, validated_base, None)?,
    };
    validated_base.ensure_subject_identity()?;
    let secure_target = staged.publish().map_err(|error| {
        open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&target),
            &boundary,
            "identity_changed_during_use",
            format!("secure open-target staged publication failed: {error}"),
        )
    })?;
    secure_target.ensure_path_identity().map_err(|error| {
        open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&target),
            &boundary,
            "identity_changed_during_use",
            format!("open target identity changed during materialization: {error}"),
        )
    })?;
    maybe_fail_open_transaction_for_test(
        &repo_root,
        "after_target_publish_before_materialization_check",
    )?;
    validated_base.ensure_subject_identity()?;
    ensure_target_unchanged(transaction, options, false)?;
    maybe_swap_open_target_before_fingerprint_for_test(&repo_root, &target)?;
    maybe_insert_open_target_symlink_before_fingerprint_for_test(&repo_root, &target)?;
    if !source_manifest_unchanged_secure(validated_base, &secure_target)? {
        return Err(invalid_transaction(
            &transaction.path,
            "published target content differs from the sealed base",
        ));
    }
    secure_target.ensure_path_identity().map_err(|error| {
        open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&target),
            &boundary,
            "identity_changed_during_use",
            format!("open target identity changed during verification: {error}"),
        )
    })?;
    Ok(secure_target)
}

fn ensure_target_unchanged(
    transaction: &OpenTransaction,
    options: &OpenOptions,
    require_empty: bool,
) -> Result<(), CliError> {
    let repo_root = path_field(
        &transaction.record,
        &["repository", "path"],
        &transaction.path,
    )?;
    let target = transaction.target()?;
    let boundary = path_field(
        &transaction.record,
        &["open", "workspace_boundary"],
        &transaction.path,
    )?;
    let (rechecked, _rechecked_resolution) =
        resolve_open_target(&repo_root, &options.path, &options.branch).map_err(|error| {
            with_branch_operation_context(
                error,
                &repo_root,
                Some(&options.branch),
                Some(&options.path),
            )
        })?;
    if rechecked != target {
        return Err(open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&rechecked),
            &boundary,
            "target_changed_before_use",
            format!(
                "open target resolution changed: expected {}, found {}",
                target.display(),
                rechecked.display()
            ),
        ));
    }
    if require_empty && target.exists() && !is_empty_dir(&target)? {
        return Err(open_target_resolution_error(
            &repo_root,
            &options.branch,
            &options.path,
            Some(&target),
            &boundary,
            "target_changed_before_use",
            format!(
                "target path changed before materialization and is no longer empty: {}",
                target.display()
            ),
        ));
    }
    Ok(())
}

fn validate_complete(
    transaction: &OpenTransaction,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<(), CliError> {
    let evidence = inspect_evidence(transaction, true, validated_base)?;
    validate_phase_evidence(OpenPhase::IdempotencyWritten, &evidence, &transaction.path)
}

fn expected_open_marker(
    cf: &Path,
    repository_id: &str,
    branch: &str,
    base_commit: &str,
    target: &Path,
    open_instance_id: &str,
    opened_at: &str,
) -> Value {
    json!({
        "version": 1,
        "repository": {"path": cf, "repository_id": repository_id},
        "branch": {"name": branch, "opened_from_commit": base_commit},
        "open": {
            "open_instance_id": open_instance_id,
            "opened_path": target,
            "opened_at": opened_at,
        },
    })
}

fn expected_registry(
    branch: &str,
    base_commit: &str,
    target: &Path,
    open_instance_id: &str,
    active_state_path: &Path,
) -> Value {
    json!({
        "version": 1,
        "branch": {"name": branch},
        "open": {
            "open_instance_id": open_instance_id,
            "path": target,
            "opened_from_commit": base_commit,
            "current_base_commit": base_commit,
            "active_state_path": active_state_path,
        },
        "state": {"last_known": "open-clean"},
    })
}

fn staging_name(owner: &Value) -> Result<String, CliError> {
    let mut bytes = serde_json::to_vec(owner)?;
    bytes.push(b'\n');
    Ok(format!(
        ".codefire-open-stage-{:012x}",
        stable_hash_48(&bytes)
    ))
}

fn open_instance_id(branch: &str, target: &Path, opened_at: &str) -> String {
    format!(
        "open_{:012x}",
        stable_hash_48(format!("{branch}:{}:{opened_at}", target.display()).as_bytes())
    )
}

fn random_transaction_id() -> Result<String, CliError> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate open transaction identity: {error}"
        ))
    })?;
    Ok(format!(
        "open_tx_{}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
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

fn optional_string(
    record: &Value,
    fields: &[&str],
    path: &Path,
) -> Result<Option<String>, CliError> {
    match value(record, fields, path)? {
        Value::Null => Ok(None),
        Value::String(value) if !value.is_empty() => Ok(Some(value.clone())),
        _ => Err(invalid_transaction(
            path,
            &format!("{} must be null or a non-empty string", fields.join(".")),
        )),
    }
}

fn u64_field(record: &Value, fields: &[&str], path: &Path) -> Result<u64, CliError> {
    value(record, fields, path)?.as_u64().ok_or_else(|| {
        invalid_transaction(path, &format!("{} must be an integer", fields.join(".")))
    })
}

fn path_field(record: &Value, fields: &[&str], path: &Path) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(string(record, fields, path)?))
}

fn invalid_transaction(path: &Path, detail: &str) -> CliError {
    CliError::InvalidRepository(format!(
        "invalid pending open transaction {}: {detail}",
        path.display()
    ))
}

pub(crate) fn doctor_open_transaction_summary(
    repo_root: &Path,
) -> Result<Option<String>, CliError> {
    let Some(transaction) = OpenTransaction::load(repo_root)? else {
        return Ok(None);
    };
    doctor_open_transaction_summary_inner(repo_root, transaction).map(Some)
}

pub(crate) fn doctor_open_transaction_summary_from_record(
    repo_root: &Path,
    path: PathBuf,
    record: Value,
) -> Result<String, CliError> {
    let transaction = OpenTransaction {
        path,
        record,
        marker_identity: None,
    };
    transaction.validate(repo_root)?;
    if recorded_staging_parent_identity(&transaction)?.is_none() {
        return Err(invalid_transaction(
            &transaction.path,
            "pending marker is missing staged publication-parent identity",
        ));
    }
    if transaction.path == open_transaction_removal_path(repo_root)
        && transaction.phase()? != OpenPhase::IdempotencyWritten
    {
        return Err(invalid_transaction(
            &transaction.path,
            "a quarantined marker must be at the final durable phase",
        ));
    }
    doctor_open_transaction_summary_inner(repo_root, transaction)
}

fn doctor_open_transaction_summary_inner(
    repo_root: &Path,
    transaction: OpenTransaction,
) -> Result<String, CliError> {
    let validated_base = codefire_store::validate_sealed_commit_pinned(
        &repo_root.join(".codefire").join("objects"),
        &transaction.base_commit()?,
    )?;
    let evidence = inspect_evidence(&transaction, true, &validated_base)?;
    validate_phase_evidence(transaction.phase()?, &evidence, &transaction.path)?;
    Ok(format!(
        "open transaction is pending at phase {} for branch {} target {} open instance {}; evidence target={:?} registry={:?} active_state={:?} branch={:?} idempotency={:?}",
        transaction.phase()?.as_str(),
        transaction.branch_name()?,
        transaction.target()?.display(),
        string(&transaction.record, &["open", "open_instance_id"], &transaction.path)?,
        evidence.target,
        evidence.registry,
        evidence.active_state,
        evidence.branch,
        evidence.idempotency,
    ))
}

#[cfg(test)]
#[derive(Debug)]
struct OpenFailureInjection {
    repo_root: PathBuf,
    point: String,
}

#[cfg(test)]
static OPEN_TRANSACTION_FAILURES: Mutex<Vec<OpenFailureInjection>> = Mutex::new(Vec::new());

#[cfg(test)]
#[derive(Debug)]
struct ForeignOpenMarkerInjection {
    repo_root: PathBuf,
    contents: Vec<u8>,
}

#[cfg(test)]
static FOREIGN_OPEN_MARKER_BEFORE_SAVE: Mutex<Option<ForeignOpenMarkerInjection>> =
    Mutex::new(None);

#[cfg(test)]
#[derive(Debug)]
struct OpenMarkerRemovalSwapInjection {
    repo_root: PathBuf,
    moved_original: PathBuf,
}

#[cfg(test)]
static OPEN_MARKER_REMOVAL_SWAP: Mutex<Option<OpenMarkerRemovalSwapInjection>> = Mutex::new(None);

#[cfg(test)]
#[derive(Debug)]
struct OpenTargetAncestorSwapInjection {
    target: PathBuf,
    ancestor: PathBuf,
    moved_original: PathBuf,
    replacement: PathBuf,
}

#[cfg(test)]
static OPEN_TARGET_ANCESTOR_SWAP: Mutex<Option<OpenTargetAncestorSwapInjection>> = Mutex::new(None);
#[cfg(test)]
static OPEN_TARGET_ANCESTOR_POST_VALIDATION_SWAP: Mutex<Option<OpenTargetAncestorSwapInjection>> =
    Mutex::new(None);

#[cfg(test)]
pub(crate) fn set_open_transaction_failure_for_test(repo_root: &Path, point: &str) {
    OPEN_TRANSACTION_FAILURES
        .lock()
        .expect("open failure mutex")
        .push(OpenFailureInjection {
            repo_root: repo_root.to_path_buf(),
            point: point.to_string(),
        });
}

#[cfg(test)]
pub(crate) fn set_foreign_open_marker_before_save_for_test(repo_root: &Path, contents: &[u8]) {
    *FOREIGN_OPEN_MARKER_BEFORE_SAVE
        .lock()
        .expect("foreign open marker mutex") = Some(ForeignOpenMarkerInjection {
        repo_root: repo_root.to_path_buf(),
        contents: contents.to_vec(),
    });
}

#[cfg(test)]
fn maybe_publish_foreign_open_marker_before_save_for_test(
    repo_root: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = FOREIGN_OPEN_MARKER_BEFORE_SAVE
            .lock()
            .expect("foreign open marker mutex");
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
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(open_transaction_path(repo_root))?;
    use std::io::Write as _;
    file.write_all(&injection.contents)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(not(test))]
fn maybe_publish_foreign_open_marker_before_save_for_test(
    _repo_root: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_open_marker_swap_before_removal_for_test(
    repo_root: &Path,
    moved_original: &Path,
) {
    *OPEN_MARKER_REMOVAL_SWAP
        .lock()
        .expect("open marker removal swap mutex") = Some(OpenMarkerRemovalSwapInjection {
        repo_root: repo_root.to_path_buf(),
        moved_original: moved_original.to_path_buf(),
    });
}

#[cfg(test)]
fn maybe_swap_open_marker_before_removal_for_test(
    repo_root: &Path,
    marker_path: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = OPEN_MARKER_REMOVAL_SWAP
            .lock()
            .expect("open marker removal swap mutex");
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
    let bytes = fs::read(marker_path)?;
    fs::rename(marker_path, &injection.moved_original)?;
    if let Err(error) = fs::write(marker_path, bytes) {
        let _ = fs::rename(&injection.moved_original, marker_path);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_open_marker_before_removal_for_test(
    _repo_root: &Path,
    _marker_path: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_open_target_ancestor_swap_before_use_for_test(
    target: &Path,
    ancestor: &Path,
    moved_original: &Path,
    replacement: &Path,
) {
    *OPEN_TARGET_ANCESTOR_SWAP
        .lock()
        .expect("open target ancestor swap mutex") = Some(OpenTargetAncestorSwapInjection {
        target: target.to_path_buf(),
        ancestor: ancestor.to_path_buf(),
        moved_original: moved_original.to_path_buf(),
        replacement: replacement.to_path_buf(),
    });
}

#[cfg(test)]
pub(crate) fn set_open_target_ancestor_swap_after_validation_for_test(
    target: &Path,
    ancestor: &Path,
    moved_original: &Path,
    replacement: &Path,
) {
    *OPEN_TARGET_ANCESTOR_POST_VALIDATION_SWAP
        .lock()
        .expect("post-validation open target ancestor swap mutex") =
        Some(OpenTargetAncestorSwapInjection {
            target: target.to_path_buf(),
            ancestor: ancestor.to_path_buf(),
            moved_original: moved_original.to_path_buf(),
            replacement: replacement.to_path_buf(),
        });
}

#[cfg(test)]
fn maybe_swap_open_target_ancestor_before_use_for_test(target: &Path) -> Result<(), CliError> {
    let injection = {
        let mut guard = OPEN_TARGET_ANCESTOR_SWAP
            .lock()
            .expect("open target ancestor swap mutex");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.target == target)
        {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    fs::rename(&injection.ancestor, &injection.moved_original)?;
    if let Err(error) = fs::rename(&injection.replacement, &injection.ancestor) {
        let _ = fs::rename(&injection.moved_original, &injection.ancestor);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(test)]
fn maybe_swap_open_target_ancestor_after_validation_for_test(
    target: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = OPEN_TARGET_ANCESTOR_POST_VALIDATION_SWAP
            .lock()
            .expect("post-validation open target ancestor swap mutex");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.target == target)
        {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    fs::rename(&injection.ancestor, &injection.moved_original)?;
    if let Err(error) = fs::rename(&injection.replacement, &injection.ancestor) {
        let _ = fs::rename(&injection.moved_original, &injection.ancestor);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_open_target_ancestor_before_use_for_test(_target: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_open_target_ancestor_after_validation_for_test(
    _target: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
#[derive(Debug)]
struct OpenTargetSwapInjection {
    repo_root: PathBuf,
    target: PathBuf,
    moved: PathBuf,
}

#[cfg(all(test, unix))]
static OPEN_TARGET_SWAPS: Mutex<Vec<OpenTargetSwapInjection>> = Mutex::new(Vec::new());
#[cfg(all(test, unix))]
static OPEN_TARGET_POST_FINGERPRINT_SWAPS: Mutex<Vec<OpenTargetSwapInjection>> =
    Mutex::new(Vec::new());

#[cfg(all(test, unix))]
#[derive(Debug)]
struct OpenTargetSymlinkInjection {
    repo_root: PathBuf,
    target: PathBuf,
    entry_name: String,
    outside: PathBuf,
}

#[cfg(all(test, unix))]
static OPEN_TARGET_SYMLINKS: Mutex<Vec<OpenTargetSymlinkInjection>> = Mutex::new(Vec::new());

#[cfg(all(test, unix))]
pub(crate) fn set_open_target_swap_before_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
    moved: &Path,
) {
    OPEN_TARGET_SWAPS
        .lock()
        .expect("open target swap mutex")
        .push(OpenTargetSwapInjection {
            repo_root: repo_root.to_path_buf(),
            target: target.to_path_buf(),
            moved: moved.to_path_buf(),
        });
}

#[cfg(all(test, unix))]
pub(crate) fn set_open_target_swap_after_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
    moved: &Path,
) {
    OPEN_TARGET_POST_FINGERPRINT_SWAPS
        .lock()
        .expect("post-fingerprint open target swap mutex")
        .push(OpenTargetSwapInjection {
            repo_root: repo_root.to_path_buf(),
            target: target.to_path_buf(),
            moved: moved.to_path_buf(),
        });
}

#[cfg(all(test, unix))]
pub(crate) fn set_open_target_symlink_before_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
    entry_name: &str,
    outside: &Path,
) {
    OPEN_TARGET_SYMLINKS
        .lock()
        .expect("open target symlink mutex")
        .push(OpenTargetSymlinkInjection {
            repo_root: repo_root.to_path_buf(),
            target: target.to_path_buf(),
            entry_name: entry_name.to_string(),
            outside: outside.to_path_buf(),
        });
}

#[cfg(all(test, unix))]
fn maybe_swap_open_target_before_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
) -> Result<(), CliError> {
    use std::os::unix::fs::symlink;

    let injection = {
        let mut swaps = OPEN_TARGET_SWAPS.lock().expect("open target swap mutex");
        swaps
            .iter()
            .position(|swap| swap.repo_root == repo_root && swap.target == target)
            .map(|index| swaps.remove(index))
    };
    let Some(injection) = injection else {
        return Ok(());
    };

    fs::rename(&injection.target, &injection.moved)?;
    if let Err(error) = symlink(&injection.moved, &injection.target) {
        let _ = fs::rename(&injection.moved, &injection.target);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(all(test, unix))]
fn maybe_swap_open_target_after_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
) -> Result<(), CliError> {
    use std::os::unix::fs::symlink;

    let injection = {
        let mut swaps = OPEN_TARGET_POST_FINGERPRINT_SWAPS
            .lock()
            .expect("post-fingerprint open target swap mutex");
        swaps
            .iter()
            .position(|swap| swap.repo_root == repo_root && swap.target == target)
            .map(|index| swaps.remove(index))
    };
    let Some(injection) = injection else {
        return Ok(());
    };

    fs::rename(&injection.target, &injection.moved)?;
    if let Err(error) = symlink(&injection.moved, &injection.target) {
        let _ = fs::rename(&injection.moved, &injection.target);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(all(test, unix))]
fn maybe_insert_open_target_symlink_before_fingerprint_for_test(
    repo_root: &Path,
    target: &Path,
) -> Result<(), CliError> {
    use std::os::unix::fs::symlink;

    let injection = {
        let mut injections = OPEN_TARGET_SYMLINKS
            .lock()
            .expect("open target symlink mutex");
        injections
            .iter()
            .position(|injection| injection.repo_root == repo_root && injection.target == target)
            .map(|index| injections.remove(index))
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    symlink(
        &injection.outside,
        injection.target.join(&injection.entry_name),
    )?;
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_swap_open_target_before_fingerprint_for_test(
    _repo_root: &Path,
    _target: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_swap_open_target_after_fingerprint_for_test(
    _repo_root: &Path,
    _target: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_insert_open_target_symlink_before_fingerprint_for_test(
    _repo_root: &Path,
    _target: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn maybe_fail_open_transaction_for_test(repo_root: &Path, point: &str) -> Result<(), CliError> {
    let mut failures = OPEN_TRANSACTION_FAILURES
        .lock()
        .expect("open failure mutex");
    if let Some(index) = failures
        .iter()
        .position(|failure| failure.repo_root == repo_root && failure.point == point)
    {
        failures.remove(index);
        return Err(CliError::InvalidRepository(format!(
            "injected open transaction failure at {point}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_open_transaction_for_test(_repo_root: &Path, _point: &str) -> Result<(), CliError> {
    Ok(())
}
