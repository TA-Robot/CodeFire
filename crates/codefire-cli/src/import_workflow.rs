#[cfg(test)]
use super::idempotency::idempotency_record_path;
use super::idempotency::{
    idempotency_payload_hash, require_idempotency_key, verify_idempotency_record,
};
use super::secure_worktree::{ensure_absolute_dir_identity, open_absolute_dir_nofollow};
use super::*;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions as CapabilityOpenOptions};
use fs2::FileExt;
use serde_json::{json, Value};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const IMPORT_IDEMPOTENCY_RECORD_TYPE: &str = "codefire_import_idempotency_record";
const IMPORT_IDEMPOTENCY_RECORD_VERSION: u64 = 2;
const IMPORT_IDEMPOTENCY_PAYLOAD_VERSION: u64 = 1;
const IMPORT_OPERATION: &str = "import";
const IMPORT_MODE: &str = "adopt_existing_project";
const IMPORT_BOOTSTRAP_LOCK_FILE: &str = ".codefire-import.lock";
const IMPORT_BOOTSTRAP_LOCK_TYPE: &str = "codefire_import_bootstrap_lock";
const IMPORT_BOOTSTRAP_LOCK_VERSION: u64 = 1;
const IMPORT_BOOTSTRAP_TEMP_PREFIX: &str = ".codefire-import.lock.bootstrap-";
const IMPORT_BOOTSTRAP_TEMP_SUFFIX: &str = ".tmp";
const IMPORT_BOOTSTRAP_STAGING_SUFFIX: &str = ".part";
const IMPORT_WRITE_TEMP_PREFIX: &str = ".codefire-import.write-";
const IMPORT_WRITE_TEMP_SUFFIX: &str = ".tmp";
const IMPORT_TRANSACTION_FILE: &str = ".codefire-import-transaction.json";
const IMPORT_TRANSACTION_TYPE: &str = "codefire_import_transaction";
const IMPORT_TRANSACTION_VERSION: u64 = 1;
const IMPORT_CONTROL_FILE_BYTE_LIMIT: u64 = 1024 * 1024;
const IMPORT_BLOB_RECORD_OVERHEAD_RESERVE_BYTES: u64 = 1024 * 1024;
const IMPORT_USER_FILE_BYTE_LIMIT: u64 =
    ((codefire_store::MAX_OBJECT_RECORD_BYTES - IMPORT_BLOB_RECORD_OVERHEAD_RESERVE_BYTES) / 4) * 3;
const IMPORT_REPOSITORY_FILE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;

pub(crate) fn parse_import_args(args: &[String]) -> Result<ImportOptions, CliError> {
    let mut path = None;
    let mut branch = "main".to_string();
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--branch" => {
                branch = take_option_value(args, &mut index, "--branch")?;
            }
            value if value.starts_with("--branch=") => {
                branch = value.trim_start_matches("--branch=").to_string();
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported import option: {value}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected import argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(ImportOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        branch,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}

pub(crate) fn import_existing_project(options: &ImportOptions) -> Result<ImportResult, CliError> {
    let target = normalized_import_target(&absolute_path(&options.path)?)?;
    if let Some(key) = options.idempotency_key.as_deref() {
        // Dry-run and live execution intentionally share key validation.
        require_idempotency_key(key)?;
    }
    let idempotency_payload = import_idempotency_payload(options, &target)?;
    preflight_import_snapshot(&target)?;

    if options.dry_run {
        validate_dry_run_reserved_paths(&target)?;
        validate_import_target(&target)?;
        let initial_files = rel_files(&target)?.len();
        let opened_at = now_iso_utc();
        let active_path = target
            .join(".codefire")
            .join("active")
            .join(import_open_instance_id(
                &options.branch,
                &target,
                &opened_at,
            ));
        let registry_path = opened_registry_path(&target, &options.branch);
        return Ok(ImportResult {
            repo_root: target.clone(),
            open_dir: target.clone(),
            branch: options.branch.clone(),
            base_commit: String::new(),
            initial_files,
            initial_state: "planned".to_string(),
            plan: import_operation_plan(
                options,
                &target,
                &target,
                &registry_path,
                &active_path,
                initial_files,
            ),
        });
    }

    // This lock inode is deliberately never unlinked. Kernel lock release on
    // close/crash plus a stable pathname prevents stale-directory and unlink
    // ABA failures while the separate transaction file can be atomically
    // replaced at phase boundaries.
    let bootstrap_lock = ImportBootstrapLock::acquire(
        &target,
        &options.branch,
        &idempotency_payload,
        &options.lock,
    )?;
    bootstrap_lock.ensure_target_identity()?;
    let subject = ImportSubject::from_bootstrap(&bootstrap_lock)?;
    let transaction_after_lock = bootstrap_lock.load_transaction()?;
    if let Some(key) = options.idempotency_key.as_deref() {
        if let Some(result) =
            load_import_idempotency_marker_subject(&subject, key, &idempotency_payload)?
        {
            let mut transaction = transaction_after_lock.ok_or_else(|| {
                CliError::InvalidRepository(
                    "import idempotency marker has no durable transaction provenance".to_string(),
                )
            })?;
            transaction.verify_request(&target, &options.branch, &idempotency_payload)?;
            if transaction.transaction_id != bootstrap_lock.seed.transaction_id
                || !matches!(
                    transaction.phase,
                    ImportPhase::Scanned | ImportPhase::Completed
                )
            {
                return Err(CliError::InvalidRepository(
                    "import idempotency marker and transaction phase disagree".to_string(),
                ));
            }
            let transaction_result = transaction.result(&target, options)?;
            if !same_import_result_subject(&transaction_result, &result) {
                return Err(CliError::InvalidRepository(
                    "import idempotency marker and transaction result disagree".to_string(),
                ));
            }
            let _repo_lock = ImportRepoLock::acquire(&subject, &options.lock)?;
            _repo_lock.ensure_subject_identity(&subject)?;
            bootstrap_lock.ensure_target_identity()?;
            validate_import_replay_subject(&subject, &idempotency_payload, &transaction, &result)?;
            if transaction.phase == ImportPhase::Scanned {
                transaction.phase = ImportPhase::Completed;
                save_import_transaction_subject(&subject, &transaction)?;
            }
            bootstrap_lock.ensure_target_identity()?;
            return Ok(result);
        }
    }

    let mut transaction = match transaction_after_lock {
        Some(transaction) => {
            transaction.verify_request(&target, &options.branch, &idempotency_payload)?;
            if transaction.transaction_id != bootstrap_lock.seed.transaction_id {
                return Err(CliError::InvalidRepository(format!(
                    "import transaction and bootstrap lock identities disagree: {}",
                    target.display()
                )));
            }
            transaction
        }
        None => {
            validate_import_target(&target)?;
            let transaction = bootstrap_lock.seed.clone();
            bootstrap_lock.ensure_target_identity()?;
            save_import_transaction_subject(&subject, &transaction)?;
            maybe_fail_import_phase(&target, ImportPhase::Planned)?;
            transaction
        }
    };

    if matches!(
        transaction.phase,
        ImportPhase::Scanned | ImportPhase::Completed
    ) {
        let mut result = transaction.result(&target, options)?;
        let _repo_lock = ImportRepoLock::acquire(&subject, &options.lock)?;
        _repo_lock.ensure_subject_identity(&subject)?;
        bootstrap_lock.ensure_target_identity()?;
        validate_import_replay_subject(&subject, &idempotency_payload, &transaction, &result)?;
        if let Some(key) = options.idempotency_key.as_deref() {
            save_import_idempotency_marker_subject(&subject, key, &idempotency_payload, &result)?;
        }
        if transaction.phase != ImportPhase::Completed {
            transaction.phase = ImportPhase::Completed;
            save_import_transaction_subject(&subject, &transaction)?;
        }
        result.plan["replayed"] = Value::Bool(true);
        result.plan["recovered_from_transaction"] = Value::Bool(true);
        bootstrap_lock.ensure_target_identity()?;
        return Ok(result);
    }

    let _repo_lock = ImportRepoLock::acquire(&subject, &options.lock)?;
    _repo_lock.ensure_subject_identity(&subject)?;
    bootstrap_lock.ensure_target_identity()?;
    let base_commit = ensure_import_repository(&subject, &options.branch, &transaction)?;
    transaction.base_commit = Some(base_commit.clone());
    transaction.phase = ImportPhase::Initialized;
    save_import_transaction_subject(&subject, &transaction)?;
    maybe_fail_import_phase(&target, ImportPhase::Initialized)?;

    bootstrap_lock.ensure_target_identity()?;
    ensure_import_branch(&subject, &options.branch, &base_commit)?;
    ensure_import_open_state(&subject, &transaction, &base_commit)?;
    bootstrap_lock.ensure_target_identity()?;
    transaction.phase = ImportPhase::Opened;
    save_import_transaction_subject(&subject, &transaction)?;
    maybe_fail_import_phase(&target, ImportPhase::Opened)?;

    maybe_publish_delayed_bootstrap_temp(&target)?;
    recover_import_owned_state_transaction(&subject, &transaction)?;
    let scan_snapshot = ImportReadSnapshot::build(&subject)?;
    scan_snapshot.prepare_open_context(&transaction)?;
    let scan_execution = compute_scan(&scan_snapshot.path)?;
    bootstrap_lock.ensure_target_identity()?;
    let initial_state = persist_import_scan(&subject, &transaction, &scan_execution)?;
    let result = transaction.live_result(options, &base_commit, &initial_state);
    transaction.result = Some(ImportTransactionResult::from_import_result(&result));
    transaction.phase = ImportPhase::Scanned;
    save_import_transaction_subject(&subject, &transaction)?;
    maybe_fail_import_phase(&target, ImportPhase::Scanned)?;

    if let Some(key) = options.idempotency_key.as_deref() {
        save_import_idempotency_marker_subject(&subject, key, &idempotency_payload, &result)?;
    }
    bootstrap_lock.ensure_target_identity()?;
    transaction.phase = ImportPhase::Completed;
    save_import_transaction_subject(&subject, &transaction)?;
    maybe_fail_import_phase(&target, ImportPhase::Completed)?;
    bootstrap_lock.ensure_target_identity()?;
    Ok(result)
}

fn recover_import_owned_state_transaction(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
) -> Result<(), CliError> {
    let path = PathBuf::from(".codefire")
        .join("active")
        .join(&transaction.open_instance_id)
        .join("state_transaction.json");
    let Some(pending) = subject.read_json(&path)? else {
        return Ok(());
    };
    validate_state_transaction_record(&pending).map_err(|message| {
        CliError::InvalidRepository(format!(
            "{message}: {}",
            subject.target.join(&path).display()
        ))
    })?;
    if pending.get("version").and_then(Value::as_u64) == Some(2) {
        let expected_base = transaction.base_commit.as_deref().ok_or_else(|| {
            CliError::InvalidRepository(
                "pending import state transaction has no import base subject".to_string(),
            )
        })?;
        if required_string(&pending, &["subject", "branch"])? != transaction.branch
            || required_string(&pending, &["subject", "open_instance_id"])?
                != transaction.open_instance_id
            || required_string(&pending, &["subject", "current_base_commit"])? != expected_base
        {
            return Err(CliError::InvalidRepository(format!(
                "pending state transaction does not belong to the import transaction: {}",
                subject.target.join(&path).display()
            )));
        }
    }
    subject.remove_file_if_exists(&path)
}

fn validate_dry_run_reserved_paths(target: &Path) -> Result<(), CliError> {
    for name in [IMPORT_BOOTSTRAP_LOCK_FILE, IMPORT_TRANSACTION_FILE] {
        let path = target.join(name);
        if fs::symlink_metadata(&path).is_ok() {
            return Err(CliError::Usage(format!(
                "import target contains reserved CodeFire path: {}",
                path.display()
            )));
        }
    }
    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name.starts_with(IMPORT_BOOTSTRAP_TEMP_PREFIX)
            && (name.ends_with(IMPORT_BOOTSTRAP_TEMP_SUFFIX)
                || name.ends_with(IMPORT_BOOTSTRAP_STAGING_SUFFIX)))
            || (name.starts_with(IMPORT_WRITE_TEMP_PREFIX)
                && name.ends_with(IMPORT_WRITE_TEMP_SUFFIX))
        {
            return Err(CliError::Usage(format!(
                "import target contains reserved CodeFire bootstrap temp path: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ImportBootstrapLock {
    seed: ImportTransaction,
    target_dir: Dir,
    _file: fs::File,
}

impl ImportBootstrapLock {
    fn acquire(
        target: &Path,
        branch: &str,
        payload: &Value,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let path = target.join(IMPORT_BOOTSTRAP_LOCK_FILE);
        let target_dir = open_absolute_dir_nofollow(target)?;
        let persisted_transaction = load_import_transaction_from_dir(target, &target_dir)?;
        let recovered_seed = discover_bootstrap_seed(target, &target_dir, branch, payload)?;
        if recovered_seed.is_none() {
            reject_foreign_import_write_temps(target, &target_dir)?;
        }
        if persisted_transaction.is_some() && recovered_seed.is_none() {
            return Err(CliError::InvalidRepository(format!(
                "import transaction exists without its bootstrap lock provenance: {}",
                import_transaction_path(target).display()
            )));
        }
        if let (Some(transaction), Some(recovered)) =
            (persisted_transaction.as_ref(), recovered_seed.as_ref())
        {
            if transaction.transaction_id != recovered.transaction_id {
                return Err(CliError::InvalidRepository(format!(
                    "import transaction and bootstrap provenance disagree: {}",
                    target.display()
                )));
            }
        }
        let proposed_seed = persisted_transaction
            .or(recovered_seed)
            .map(Ok)
            .unwrap_or_else(|| {
                validate_import_target(target)?;
                ImportTransaction::new(target, branch, rel_files(target)?.len(), payload)
            })?;
        let seed =
            ensure_bootstrap_lock_published(target, &target_dir, branch, payload, &proposed_seed)?;
        let mut open_options = CapabilityOpenOptions::new();
        open_options.read(true).write(true);
        open_options.follow(FollowSymlinks::No);
        open_options.nonblock(true);
        let capability_file = target_dir.open_with(IMPORT_BOOTSTRAP_LOCK_FILE, &open_options)?;
        ensure_import_file_metadata(
            &capability_file.metadata()?,
            &path,
            IMPORT_CONTROL_FILE_BYTE_LIMIT,
        )?;
        let mut file = capability_file.into_std();
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let started = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => break,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if !options.wait || timeout.is_some_and(|timeout| started.elapsed() >= timeout)
                    {
                        let mut message =
                            format!("CodeFire import target is locked: {}", path.display());
                        if let Some(timeout_ms) = options.timeout_ms {
                            message.push_str(&format!("; timed out after {timeout_ms}ms"));
                        }
                        return Err(CliError::LockContention(message));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
        let contents = read_locked_import_control_file(&mut file, &path)?;
        let locked_seed = parse_bootstrap_lock_bytes(&contents, &path)?;
        locked_seed.verify_request(target, branch, payload)?;
        if locked_seed.transaction_id != seed.transaction_id {
            return Err(CliError::InvalidRepository(format!(
                "import bootstrap lock identity changed while acquiring it: {}",
                path.display()
            )));
        }
        cleanup_bootstrap_temps(target, &target_dir, &contents, &locked_seed)?;
        cleanup_import_write_temps(&target_dir, &locked_seed.transaction_id)?;
        Ok(Self {
            seed: locked_seed,
            target_dir,
            _file: file,
        })
    }

    fn ensure_target_identity(&self) -> Result<(), CliError> {
        ensure_absolute_dir_identity(&self.target_dir, &self.seed.target).map_err(CliError::Io)
    }

    fn load_transaction(&self) -> Result<Option<ImportTransaction>, CliError> {
        load_import_transaction_from_dir(&self.seed.target, &self.target_dir)
    }
}

fn reject_foreign_import_write_temps(target: &Path, target_dir: &Dir) -> Result<(), CliError> {
    for entry in target_dir.entries()? {
        let name = entry?.file_name();
        let text = name.to_string_lossy();
        if text.starts_with(IMPORT_WRITE_TEMP_PREFIX) && text.ends_with(IMPORT_WRITE_TEMP_SUFFIX) {
            return Err(CliError::InvalidRepository(format!(
                "foreign import write temp requires recovery: {}",
                target.join(name).display()
            )));
        }
    }
    Ok(())
}

fn cleanup_import_write_temps(target_dir: &Dir, transaction_id: &str) -> Result<(), CliError> {
    let expected_prefix = format!("{IMPORT_WRITE_TEMP_PREFIX}{transaction_id}-");
    let mut names = target_dir
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    for name in names {
        let text = name.to_string_lossy();
        if text.starts_with(IMPORT_WRITE_TEMP_PREFIX) && text.ends_with(IMPORT_WRITE_TEMP_SUFFIX) {
            if !text.starts_with(&expected_prefix) {
                return Err(CliError::InvalidRepository(format!(
                    "foreign import write temp does not match transaction {transaction_id}: {}",
                    text
                )));
            }
            match target_dir.remove_file(&name) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }
    sync_capability_directory_best_effort(target_dir)
}

impl Drop for ImportBootstrapLock {
    fn drop(&mut self) {
        // Keep the inode/path forever, but release the kernel ownership before
        // other in-process descriptors attempt the next replay.
        let _ = FileExt::unlock(&self._file);
    }
}

struct ImportSubject {
    target: PathBuf,
    root: Dir,
    transaction_id: String,
}

#[derive(Debug)]
struct ImportRepoLock {
    file: fs::File,
    identity: ImportRepoLockIdentity,
}

impl ImportRepoLock {
    fn acquire(subject: &ImportSubject, options: &LockOptions) -> Result<Self, CliError> {
        subject.create_dir_all(Path::new(".codefire/locks"))?;
        let (parent, name) =
            subject.parent_and_name(Path::new(".codefire/locks/repo.lock"), true)?;
        let mut open_options = CapabilityOpenOptions::new();
        open_options.read(true).write(true).create(true);
        open_options.follow(FollowSymlinks::No);
        open_options.nonblock(true);
        let capability_file = parent.open_with(&name, &open_options)?;
        let lock_path = subject.target.join(".codefire/locks/repo.lock");
        let owner_path = repo_lock_owner_path(&lock_path);
        let metadata = capability_file.metadata()?;
        ensure_import_repo_lock_provenance(&metadata, &lock_path)?;
        let identity = import_repo_lock_identity(&metadata)?;
        let file = capability_file.into_std();
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let started = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => break,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if !options.wait || timeout.is_some_and(|timeout| started.elapsed() >= timeout)
                    {
                        return Err(repo_lock_contention_with_owner(
                            &subject.target,
                            &lock_path,
                            Some(&owner_path),
                            options,
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
        let mut validation_options = CapabilityOpenOptions::new();
        validation_options.read(true).write(true);
        validation_options.follow(FollowSymlinks::No);
        validation_options.nonblock(true);
        let candidate = parent.open_with(&name, &validation_options)?;
        let candidate_metadata = candidate.metadata()?;
        ensure_import_repo_lock_provenance(&candidate_metadata, &lock_path)?;
        if import_repo_lock_identity(&candidate_metadata)? != identity {
            return Err(CliError::InvalidRepository(format!(
                "repository lock identity changed while acquiring it: {}",
                lock_path.display()
            )));
        }
        // The stable kernel-lock inode is deliberately content-immutable. In
        // particular, do not truncate/write it: a malicious hard link must
        // never turn lock-owner diagnostics into an outside-file mutation.
        subject.write_json_atomic_with_limit(
            Path::new(".codefire/locks/repo.lock.owner.json"),
            &json!({
                "version": 1,
                "pid": std::process::id(),
                "created_at": now_iso_utc(),
            }),
            REPO_LOCK_OWNER_BYTE_LIMIT,
        )?;
        Ok(Self { file, identity })
    }

    fn ensure_subject_identity(&self, subject: &ImportSubject) -> Result<(), CliError> {
        let (parent, name) =
            subject.parent_and_name(Path::new(".codefire/locks/repo.lock"), false)?;
        let mut open_options = CapabilityOpenOptions::new();
        open_options.read(true).write(true);
        open_options.follow(FollowSymlinks::No);
        open_options.nonblock(true);
        let candidate = parent.open_with(name, &open_options)?;
        let lock_path = subject.target.join(".codefire/locks/repo.lock");
        let metadata = candidate.metadata()?;
        ensure_import_repo_lock_provenance(&metadata, &lock_path)?;
        if import_repo_lock_identity(&metadata)? != self.identity {
            return Err(CliError::InvalidRepository(format!(
                "repository lock identity changed while importing: {}",
                lock_path.display()
            )));
        }
        Ok(())
    }
}

fn ensure_import_repo_lock_provenance(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> Result<(), CliError> {
    if !metadata.is_file() {
        return Err(CliError::InvalidRepository(format!(
            "repository lock is not a regular no-follow file: {}",
            path.display()
        )));
    }
    ensure_import_repo_lock_single_link(metadata, path)
}

#[cfg(unix)]
fn ensure_import_repo_lock_single_link(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> Result<(), CliError> {
    use cap_std::fs::MetadataExt;

    if metadata.nlink() != 1 {
        return Err(CliError::InvalidRepository(format!(
            "repository lock must have exactly one filesystem link: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn ensure_import_repo_lock_single_link(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> Result<(), CliError> {
    use cap_std::fs::MetadataExt;

    if metadata.number_of_links() != Some(1) {
        return Err(CliError::InvalidRepository(format!(
            "repository lock must have exactly one provable filesystem link: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn ensure_import_repo_lock_single_link(
    _metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> Result<(), CliError> {
    Err(CliError::InvalidRepository(format!(
        "repository lock link provenance cannot be proven on this platform: {}",
        path.display()
    )))
}

#[cfg(unix)]
type ImportRepoLockIdentity = (u64, u64);

#[cfg(unix)]
fn import_repo_lock_identity(
    metadata: &cap_std::fs::Metadata,
) -> Result<ImportRepoLockIdentity, CliError> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
type ImportRepoLockIdentity = (u64, u64);

#[cfg(windows)]
fn import_repo_lock_identity(
    metadata: &cap_std::fs::Metadata,
) -> Result<ImportRepoLockIdentity, CliError> {
    use cap_std::fs::MetadataExt;

    match (metadata.volume_serial_number(), metadata.file_index()) {
        (Some(volume), Some(index)) => Ok((u64::from(volume), index)),
        _ => Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "repository lock identity is unavailable",
        ))),
    }
}

#[cfg(not(any(unix, windows)))]
type ImportRepoLockIdentity = ();

#[cfg(not(any(unix, windows)))]
fn import_repo_lock_identity(
    _metadata: &cap_std::fs::Metadata,
) -> Result<ImportRepoLockIdentity, CliError> {
    Err(CliError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "repository lock identity cannot be proven on this platform",
    )))
}

impl Drop for ImportRepoLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

impl ImportSubject {
    fn from_bootstrap(lock: &ImportBootstrapLock) -> Result<Self, CliError> {
        Ok(Self {
            target: lock.seed.target.clone(),
            root: lock.target_dir.try_clone()?,
            transaction_id: lock.seed.transaction_id.clone(),
        })
    }

    fn create_dir_all(&self, relative: &Path) -> Result<(), CliError> {
        let _ = self.open_dir(relative, true)?;
        Ok(())
    }

    fn open_dir(&self, relative: &Path, create: bool) -> Result<Dir, CliError> {
        let mut current = self.root.try_clone()?;
        for component in import_relative_components(relative)? {
            match current.symlink_metadata(&component) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(CliError::InvalidRepository(format!(
                            "import subject component is not a real directory: {}",
                            self.target.join(relative).display()
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => {
                    current.create_dir(&component)?;
                }
                Err(error) => return Err(CliError::Io(error)),
            }
            current = current.open_dir_nofollow(&component)?;
        }
        Ok(current)
    }

    fn parent_and_name(&self, relative: &Path, create: bool) -> Result<(Dir, OsString), CliError> {
        let mut components = import_relative_components(relative)?;
        let name = components.pop().ok_or_else(|| {
            CliError::InvalidRepository("import subject file path must not be empty".to_string())
        })?;
        let mut parent_path = PathBuf::new();
        for component in components {
            parent_path.push(component);
        }
        Ok((self.open_dir(&parent_path, create)?, name))
    }

    fn read_bytes(&self, relative: &Path) -> Result<Option<Vec<u8>>, CliError> {
        let (parent, name) = match self.parent_and_name(relative, false) {
            Ok(value) => value,
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None)
            }
            Err(error) => return Err(error),
        };
        match read_repository_capability_file(&parent, Path::new(&name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CliError::Io(error)),
        }
    }

    fn read_json(&self, relative: &Path) -> Result<Option<Value>, CliError> {
        self.read_json_with_limit(relative, IMPORT_REPOSITORY_FILE_BYTE_LIMIT)
    }

    fn read_control_json(&self, relative: &Path) -> Result<Option<Value>, CliError> {
        self.read_json_with_limit(relative, IMPORT_CONTROL_FILE_BYTE_LIMIT)
    }

    fn read_json_with_limit(&self, relative: &Path, limit: u64) -> Result<Option<Value>, CliError> {
        let (parent, name) = match self.parent_and_name(relative, false) {
            Ok(value) => value,
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None)
            }
            Err(error) => return Err(error),
        };
        match read_capability_file_with_limit(&parent, Path::new(&name), limit) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(CliError::Json),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CliError::Io(error)),
        }
    }

    fn read_required_json(&self, relative: &Path) -> Result<Value, CliError> {
        self.read_json(relative)?.ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "import subject record is missing: {}",
                self.target.join(relative).display()
            ))
        })
    }

    fn exists(&self, relative: &Path) -> Result<bool, CliError> {
        let (parent, name) = match self.parent_and_name(relative, false) {
            Ok(value) => value,
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false)
            }
            Err(error) => return Err(error),
        };
        match parent.symlink_metadata(name) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                Err(CliError::InvalidRepository(format!(
                    "import subject path is a symlink: {}",
                    self.target.join(relative).display()
                )))
            }
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(CliError::Io(error)),
        }
    }

    fn write_json_atomic(&self, relative: &Path, value: &Value) -> Result<(), CliError> {
        self.write_json_atomic_with_limit(relative, value, IMPORT_REPOSITORY_FILE_BYTE_LIMIT)
    }

    fn write_control_json_atomic(&self, relative: &Path, value: &Value) -> Result<(), CliError> {
        self.write_json_atomic_with_limit(relative, value, IMPORT_CONTROL_FILE_BYTE_LIMIT)
    }

    fn write_json_atomic_with_limit(
        &self,
        relative: &Path,
        value: &Value,
        limit: u64,
    ) -> Result<(), CliError> {
        let mut bytes = serde_json::to_vec_pretty(value)?;
        bytes.push(b'\n');
        if bytes.len() as u64 > limit {
            return Err(CliError::InvalidRepository(format!(
                "import JSON exceeds the {limit}-byte limit at {}: {} bytes",
                self.target.join(relative).display(),
                bytes.len()
            )));
        }
        self.write_bytes_atomic(relative, &bytes)
    }

    fn write_bytes_atomic(&self, relative: &Path, bytes: &[u8]) -> Result<(), CliError> {
        let (parent, name) = self.parent_and_name(relative, true)?;
        cleanup_import_write_temps(&parent, &self.transaction_id)?;
        let temp_name = format!(
            "{IMPORT_WRITE_TEMP_PREFIX}{}-{}{IMPORT_WRITE_TEMP_SUFFIX}",
            self.transaction_id,
            random_import_token()?,
        );
        let mut options = CapabilityOpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(&temp_name, &options)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        parent.rename(&temp_name, &parent, name)?;
        sync_capability_directory_best_effort(&parent)
    }

    fn write_bytes_if_absent_or_equal(
        &self,
        relative: &Path,
        bytes: &[u8],
    ) -> Result<(), CliError> {
        if let Some(existing) = self.read_bytes(relative)? {
            if existing != bytes {
                return Err(CliError::InvalidRepository(format!(
                    "partial import file differs from transaction provenance: {}",
                    self.target.join(relative).display()
                )));
            }
            return Ok(());
        }
        self.write_bytes_atomic(relative, bytes)
    }

    fn remove_file_if_exists(&self, relative: &Path) -> Result<(), CliError> {
        let (parent, name) = match self.parent_and_name(relative, false) {
            Ok(value) => value,
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(())
            }
            Err(error) => return Err(error),
        };
        match parent.remove_file(name) {
            Ok(()) => {
                sync_capability_directory_best_effort(&parent)?;
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CliError::Io(error)),
        }
    }

    fn store_object(&self, type_tag: &str, payload: Value) -> Result<String, CliError> {
        let record = codefire_store::object_record(type_tag, payload)?;
        let subdir = codefire_store::object_subdir(type_tag).ok_or_else(|| {
            CliError::InvalidRepository(format!("unsupported import object type: {type_tag}"))
        })?;
        let relative = PathBuf::from(".codefire")
            .join("objects")
            .join(subdir)
            .join(format!("{}.json", record.object_id));
        let value = serde_json::to_value(&record)?;
        let mut bytes = serde_json::to_vec_pretty(&value)?;
        bytes.push(b'\n');
        self.write_bytes_if_absent_or_equal(&relative, &bytes)?;
        Ok(record.object_id)
    }
}

fn import_relative_components(path: &Path) -> Result<Vec<OsString>, CliError> {
    path.components()
        .map(|component| match component {
            std::path::Component::Normal(value) => Ok(value.to_os_string()),
            std::path::Component::CurDir => Ok(OsString::from(".")),
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => Err(CliError::InvalidRepository(format!(
                "import subject path escapes pinned target: {}",
                path.display()
            ))),
        })
        .filter(|component| !matches!(component, Ok(value) if value == OsStr::new(".")))
        .collect()
}

fn random_import_token() -> Result<String, CliError> {
    let mut nonce = [0u8; 16];
    getrandom::getrandom(&mut nonce).map_err(|error| {
        CliError::InvalidRepository(format!("failed to generate import file identity: {error}"))
    })?;
    Ok(hex_bytes(&nonce))
}

fn discover_bootstrap_seed(
    target: &Path,
    target_dir: &Dir,
    branch: &str,
    payload: &Value,
) -> Result<Option<ImportTransaction>, CliError> {
    let canonical_path = target.join(IMPORT_BOOTSTRAP_LOCK_FILE);
    match read_capability_file(target_dir, IMPORT_BOOTSTRAP_LOCK_FILE) {
        Ok(contents) => {
            let seed = parse_bootstrap_lock_bytes(&contents, &canonical_path)?;
            seed.verify_request(target, branch, payload)?;
            return Ok(Some(seed));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(CliError::Io(error)),
    }

    let mut candidates = target_dir
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    candidates.sort();
    let mut recovered = None;
    for name in candidates {
        let name_text = name.to_string_lossy();
        // `.part` is unpublished scratch, never an authority candidate. A
        // stalled contender may still own it, and a crashed one is harmless:
        // preserve it, exclude it from scans, and continue the complete
        // `.tmp` -> canonical publication race.
        if !name_text.starts_with(IMPORT_BOOTSTRAP_TEMP_PREFIX)
            || !name_text.ends_with(IMPORT_BOOTSTRAP_TEMP_SUFFIX)
        {
            continue;
        }
        let path = target.join(&name);
        let contents = match read_capability_file(target_dir, Path::new(&name)) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(CliError::Io(error)),
        };
        let seed = parse_bootstrap_lock_bytes(&contents, &path)?;
        if recovered.is_none() {
            recovered = Some(seed);
        }
    }
    if let Some(seed) = recovered.as_ref() {
        seed.verify_request(target, branch, payload)?;
    }
    Ok(recovered)
}

fn ensure_bootstrap_lock_published(
    target: &Path,
    target_dir: &Dir,
    branch: &str,
    payload: &Value,
    proposed_seed: &ImportTransaction,
) -> Result<ImportTransaction, CliError> {
    let path = target.join(IMPORT_BOOTSTRAP_LOCK_FILE);
    match read_capability_file(target_dir, IMPORT_BOOTSTRAP_LOCK_FILE) {
        Ok(contents) => {
            let seed = parse_bootstrap_lock_bytes(&contents, &path)?;
            seed.verify_request(target, branch, payload)?;
            return Ok(seed);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(CliError::Io(error)),
    }

    let header = bootstrap_lock_header(proposed_seed);
    let mut contents = serde_json::to_vec_pretty(&header)?;
    contents.push(b'\n');
    if contents.len() as u64 > IMPORT_CONTROL_FILE_BYTE_LIMIT {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap lock exceeds the {}-byte limit at {}: {} bytes",
            IMPORT_CONTROL_FILE_BYTE_LIMIT,
            path.display(),
            contents.len()
        )));
    }
    let temp_name = format!(
        "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}{IMPORT_BOOTSTRAP_TEMP_SUFFIX}",
        proposed_seed.transaction_id
    );
    let staging_name = format!(
        "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}-{}{IMPORT_BOOTSTRAP_STAGING_SUFFIX}",
        proposed_seed.transaction_id,
        random_import_token()?,
    );
    let mut open_options = CapabilityOpenOptions::new();
    open_options.write(true).create_new(true);
    open_options.follow(FollowSymlinks::No);
    let mut staging = target_dir.open_with(&staging_name, &open_options)?;
    staging.write_all(&contents)?;
    staging.sync_all()?;
    maybe_pause_bootstrap_staging(target);
    drop(staging);
    match target_dir.hard_link(&staging_name, target_dir, &temp_name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = read_capability_file(target_dir, &temp_name)?;
            if existing != contents {
                return Err(CliError::InvalidRepository(format!(
                    "foreign or incomplete import bootstrap temp file: {}",
                    target.join(&temp_name).display()
                )));
            }
        }
        Err(error) => return Err(CliError::Io(error)),
    }
    match target_dir.remove_file(&staging_name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(CliError::Io(error)),
    }
    sync_capability_directory_best_effort(target_dir)?;
    match target_dir.hard_link(&temp_name, target_dir, IMPORT_BOOTSTRAP_LOCK_FILE) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && target_dir
                    .symlink_metadata(IMPORT_BOOTSTRAP_LOCK_FILE)
                    .is_ok() => {}
        Err(error) => return Err(CliError::Io(error)),
    }
    sync_capability_directory_best_effort(target_dir)?;
    let canonical = read_capability_file(target_dir, IMPORT_BOOTSTRAP_LOCK_FILE)?;
    let _ = target_dir.remove_file(&temp_name);
    let seed = parse_bootstrap_lock_bytes(&canonical, &path)?;
    seed.verify_request(target, branch, payload)?;
    Ok(seed)
}

fn bootstrap_lock_header(seed: &ImportTransaction) -> Value {
    let seed = seed.planned_seed();
    let mut transaction_seed = seed.to_json();
    transaction_seed["updated_at"] = Value::String(seed.opened_at.clone());
    json!({
        "type": IMPORT_BOOTSTRAP_LOCK_TYPE,
        "version": IMPORT_BOOTSTRAP_LOCK_VERSION,
        "transaction_seed": transaction_seed,
    })
}

fn parse_bootstrap_lock_bytes(bytes: &[u8], path: &Path) -> Result<ImportTransaction, CliError> {
    let record: Value = serde_json::from_slice(bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "invalid import bootstrap lock {}: invalid JSON: {error}",
            path.display()
        ))
    })?;
    if transaction_string(&record, path, &["type"])? != IMPORT_BOOTSTRAP_LOCK_TYPE
        || transaction_u64(&record, path, &["version"])? != IMPORT_BOOTSTRAP_LOCK_VERSION
    {
        return Err(CliError::InvalidRepository(format!(
            "foreign import bootstrap lock: {}",
            path.display()
        )));
    }
    let seed_value = transaction_value(&record, path, &["transaction_seed"])?;
    let seed = parse_import_transaction_record(seed_value, path)?;
    if seed.phase != ImportPhase::Planned || seed.base_commit.is_some() || seed.result.is_some() {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap lock seed must be planned and unpublished: {}",
            path.display()
        )));
    }
    Ok(seed)
}

fn read_capability_file(target_dir: &Dir, name: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    read_capability_file_with_limit(target_dir, name, IMPORT_CONTROL_FILE_BYTE_LIMIT)
}

fn read_repository_capability_file(
    target_dir: &Dir,
    name: impl AsRef<Path>,
) -> std::io::Result<Vec<u8>> {
    read_capability_file_with_limit(target_dir, name, IMPORT_REPOSITORY_FILE_BYTE_LIMIT)
}

fn read_snapshot_capability_file(
    target_dir: &Dir,
    name: impl AsRef<Path>,
    limit: u64,
) -> std::io::Result<Vec<u8>> {
    read_capability_file_with_limit(target_dir, name, limit)
}

fn read_capability_file_with_limit(
    target_dir: &Dir,
    name: impl AsRef<Path>,
    limit: u64,
) -> std::io::Result<Vec<u8>> {
    read_capability_file_with_limit_and_hook(target_dir, name, limit, || Ok(()))
}

fn read_capability_file_with_limit_and_hook<F>(
    target_dir: &Dir,
    name: impl AsRef<Path>,
    limit: u64,
    after_metadata_check: F,
) -> std::io::Result<Vec<u8>>
where
    F: FnOnce() -> std::io::Result<()>,
{
    let name = name.as_ref();
    let mut open_options = CapabilityOpenOptions::new();
    open_options.read(true);
    open_options.follow(FollowSymlinks::No);
    open_options.nonblock(true);
    let file = target_dir.open_with(name, &open_options)?;
    let metadata = file.metadata()?;
    ensure_import_file_metadata(&metadata, name, limit)?;
    after_metadata_check()?;
    read_bounded_import_file(file, metadata.len(), name, limit)
}

fn read_absolute_import_control_file(path: &Path) -> std::io::Result<Vec<u8>> {
    read_absolute_import_file_with_limit(path, IMPORT_CONTROL_FILE_BYTE_LIMIT)
}

fn read_absolute_repository_file(path: &Path) -> std::io::Result<Vec<u8>> {
    read_absolute_import_file_with_limit(path, IMPORT_REPOSITORY_FILE_BYTE_LIMIT)
}

fn read_absolute_import_file_with_limit(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let parent_path = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("import file path has no parent: {}", path.display()),
        )
    })?;
    let name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("import file path has no name: {}", path.display()),
        )
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    read_capability_file_with_limit(&parent, name, limit)
}

fn ensure_import_file_metadata(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
    limit: u64,
) -> std::io::Result<()> {
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "import reader requires a regular no-follow file: {}",
                path.display()
            ),
        ));
    }
    ensure_import_file_size(metadata.len(), path, limit)
}

fn ensure_std_import_file_metadata(
    metadata: &fs::Metadata,
    path: &Path,
    limit: u64,
) -> std::io::Result<()> {
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "import reader requires a regular no-follow file: {}",
                path.display()
            ),
        ));
    }
    ensure_import_file_size(metadata.len(), path, limit)
}

fn ensure_import_file_size(size: u64, path: &Path, limit: u64) -> std::io::Result<()> {
    if size > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "import file exceeds the {limit}-byte limit at {}: {size} bytes",
                path.display()
            ),
        ));
    }
    Ok(())
}

fn read_bounded_import_file(
    reader: impl Read,
    reported_size: u64,
    path: &Path,
    limit: u64,
) -> std::io::Result<Vec<u8>> {
    ensure_import_file_size(reported_size, path, limit)?;
    let initial_capacity = usize::try_from(reported_size)
        .unwrap_or(usize::MAX)
        .min(limit as usize);
    let mut contents = Vec::with_capacity(initial_capacity.saturating_add(1));
    reader.take(limit + 1).read_to_end(&mut contents)?;
    if contents.len() as u64 > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "import file grew beyond the {limit}-byte limit while reading {}",
                path.display()
            ),
        ));
    }
    Ok(contents)
}

fn read_locked_import_control_file(file: &mut fs::File, path: &Path) -> std::io::Result<Vec<u8>> {
    let metadata = file.metadata()?;
    ensure_std_import_file_metadata(&metadata, path, IMPORT_CONTROL_FILE_BYTE_LIMIT)?;
    file.seek(SeekFrom::Start(0))?;
    read_bounded_import_file(file, metadata.len(), path, IMPORT_CONTROL_FILE_BYTE_LIMIT)
}

fn cleanup_bootstrap_temps(
    target: &Path,
    target_dir: &Dir,
    canonical_contents: &[u8],
    canonical_seed: &ImportTransaction,
) -> Result<(), CliError> {
    let canonical_staging_prefix = format!(
        "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}-",
        canonical_seed.transaction_id
    );
    let mut names = target_dir
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    for name in names {
        let name_text = name.to_string_lossy();
        if !name_text.starts_with(IMPORT_BOOTSTRAP_TEMP_PREFIX) {
            continue;
        }
        if name_text.ends_with(IMPORT_BOOTSTRAP_STAGING_SUFFIX) {
            if name_text.starts_with(&canonical_staging_prefix) {
                cleanup_canonical_bootstrap_staging(
                    target,
                    target_dir,
                    &name,
                    canonical_contents,
                    canonical_seed,
                )?;
            }
            continue;
        }
        if !name_text.ends_with(IMPORT_BOOTSTRAP_TEMP_SUFFIX) {
            continue;
        }
        let contents = match read_capability_file(target_dir, Path::new(&name)) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(CliError::Io(error)),
        };
        let path = target.join(&name);
        let seed = parse_bootstrap_lock_bytes(&contents, &path)?;
        if seed.transaction_id != canonical_seed.transaction_id {
            // A concurrently prepared contender may still be between its temp
            // publication and canonical hard-link attempt. It owns this temp;
            // do not unlink it from under that process. Scanners reserve this
            // exact fully-formed temp namespace until the contender removes it.
            continue;
        }
        if contents != canonical_contents {
            return Err(CliError::InvalidRepository(format!(
                "import bootstrap hard-link content mismatch: {}",
                path.display()
            )));
        }
        match ensure_same_bootstrap_inode(target_dir, &name) {
            Ok(()) => {}
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
        match target_dir.remove_file(&name) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CliError::Io(error)),
        }
    }
    sync_capability_directory_best_effort(target_dir)?;
    Ok(())
}

fn cleanup_canonical_bootstrap_staging(
    target: &Path,
    target_dir: &Dir,
    name: &OsStr,
    canonical_contents: &[u8],
    canonical_seed: &ImportTransaction,
) -> Result<(), CliError> {
    let path = target.join(name);
    match ensure_same_bootstrap_inode(target_dir, name) {
        Ok(()) => {}
        Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        // A different inode is still unpublished scratch owned by a stalled or
        // crashed contender. It is never parsed as authority or unlinked here.
        Err(CliError::InvalidRepository(_)) => return Ok(()),
        Err(error) => return Err(error),
    }
    let contents = match read_capability_file(target_dir, Path::new(name)) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(CliError::Io(error)),
    };
    if contents != canonical_contents {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap staging content mismatch: {}",
            path.display()
        )));
    }
    let seed = parse_bootstrap_lock_bytes(&contents, &path)?;
    if seed.transaction_id != canonical_seed.transaction_id {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap staging transaction mismatch: {}",
            path.display()
        )));
    }
    ensure_same_bootstrap_inode(target_dir, name)?;
    match target_dir.remove_file(name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CliError::Io(error)),
    }
}

fn sync_capability_directory_best_effort(target_dir: &Dir) -> Result<(), CliError> {
    let directory = target_dir.open(".")?;
    match directory.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(CliError::Io(error)),
    }
}

#[cfg(unix)]
fn ensure_same_bootstrap_inode(
    target_dir: &Dir,
    temp_name: &std::ffi::OsStr,
) -> Result<(), CliError> {
    use cap_std::fs::MetadataExt;

    let canonical = target_dir.metadata(IMPORT_BOOTSTRAP_LOCK_FILE)?;
    let temp = target_dir.metadata(temp_name)?;
    if canonical.dev() != temp.dev() || canonical.ino() != temp.ino() {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap temp is not linked to the canonical lock: {}",
            temp_name.to_string_lossy()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn ensure_same_bootstrap_inode(
    target_dir: &Dir,
    temp_name: &std::ffi::OsStr,
) -> Result<(), CliError> {
    use cap_std::fs::MetadataExt;

    let canonical = target_dir.metadata(IMPORT_BOOTSTRAP_LOCK_FILE)?;
    let temp = target_dir.metadata(temp_name)?;
    let canonical_identity = (canonical.volume_serial_number(), canonical.file_index());
    let temp_identity = (temp.volume_serial_number(), temp.file_index());
    if canonical_identity.0.is_none()
        || canonical_identity.1.is_none()
        || canonical_identity != temp_identity
    {
        return Err(CliError::InvalidRepository(format!(
            "import bootstrap temp is not linked to the canonical lock: {}",
            temp_name.to_string_lossy()
        )));
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn ensure_same_bootstrap_inode(
    _target_dir: &Dir,
    _temp_name: &std::ffi::OsStr,
) -> Result<(), CliError> {
    Err(CliError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "bootstrap file identity cannot be proven on this platform",
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportPhase {
    Planned,
    Initialized,
    Opened,
    Scanned,
    Completed,
}

impl ImportPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Initialized => "initialized",
            Self::Opened => "opened",
            Self::Scanned => "scanned",
            Self::Completed => "completed",
        }
    }

    fn parse(value: &str, path: &Path) -> Result<Self, CliError> {
        match value {
            "planned" => Ok(Self::Planned),
            "initialized" => Ok(Self::Initialized),
            "opened" => Ok(Self::Opened),
            "scanned" => Ok(Self::Scanned),
            "completed" => Ok(Self::Completed),
            _ => Err(invalid_import_transaction(
                path,
                &format!("unsupported phase: {value}"),
            )),
        }
    }
}

#[cfg(test)]
#[derive(Debug)]
struct ImportFailureInjection {
    target: PathBuf,
    phase: ImportPhase,
}

#[cfg(test)]
static IMPORT_FAILURE_PHASE: std::sync::Mutex<Vec<ImportFailureInjection>> =
    std::sync::Mutex::new(Vec::new());
#[cfg(test)]
static IMPORT_INIT_LAYOUT_FAILURE: std::sync::Mutex<Vec<PathBuf>> =
    std::sync::Mutex::new(Vec::new());
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportInitPublicationPoint {
    BeforeBranch,
    AfterBranch,
}
#[cfg(test)]
#[derive(Debug)]
struct ImportInitPublicationFailure {
    target: PathBuf,
    point: ImportInitPublicationPoint,
}
#[cfg(test)]
static IMPORT_INIT_PUBLICATION_FAILURE: std::sync::Mutex<Vec<ImportInitPublicationFailure>> =
    std::sync::Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
struct ImportDelayedBootstrapTemp {
    target: PathBuf,
    name: String,
    contents: Vec<u8>,
}
#[cfg(test)]
static IMPORT_DELAYED_BOOTSTRAP_TEMP: std::sync::Mutex<Vec<ImportDelayedBootstrapTemp>> =
    std::sync::Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
struct ImportBootstrapStagingPause {
    target: PathBuf,
    reached: std::sync::Arc<std::sync::Barrier>,
    release: std::sync::Arc<std::sync::Barrier>,
}
#[cfg(test)]
static IMPORT_BOOTSTRAP_STAGING_PAUSE: std::sync::Mutex<Vec<ImportBootstrapStagingPause>> =
    std::sync::Mutex::new(Vec::new());
#[cfg(all(test, unix))]
#[derive(Debug)]
struct ImportTargetSwap {
    target: PathBuf,
    moved: PathBuf,
}
#[cfg(all(test, unix))]
static IMPORT_TARGET_SWAP: std::sync::Mutex<Vec<ImportTargetSwap>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(test)]
fn maybe_fail_import_phase(target: &Path, phase: ImportPhase) -> Result<(), CliError> {
    let mut injection = IMPORT_FAILURE_PHASE
        .lock()
        .expect("import failure injection mutex poisoned");
    if let Some(index) = injection
        .iter()
        .position(|injection| injection.target == target && injection.phase == phase)
    {
        injection.remove(index);
        return Err(CliError::Io(std::io::Error::other(format!(
            "injected import failure after {}",
            phase.as_str()
        ))));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_import_phase(_target: &Path, _phase: ImportPhase) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn inject_import_failure(target: &Path, phase: ImportPhase) {
    IMPORT_FAILURE_PHASE
        .lock()
        .expect("import failure injection mutex poisoned")
        .push(ImportFailureInjection {
            target: target.to_path_buf(),
            phase,
        });
}

#[cfg(test)]
fn maybe_fail_import_init_layout(target: &Path) -> Result<(), CliError> {
    let mut injection = IMPORT_INIT_LAYOUT_FAILURE
        .lock()
        .expect("import init-layout failure mutex poisoned");
    if let Some(index) = injection.iter().position(|expected| expected == target) {
        injection.remove(index);
        return Err(CliError::Io(std::io::Error::other(
            "injected import failure after layout creation",
        )));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_import_init_layout(_target: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn inject_import_init_layout_failure(target: &Path) {
    IMPORT_INIT_LAYOUT_FAILURE
        .lock()
        .expect("import init-layout failure mutex poisoned")
        .push(target.to_path_buf());
}

#[cfg(test)]
fn maybe_fail_import_init_publication(
    target: &Path,
    point: ImportInitPublicationPoint,
) -> Result<(), CliError> {
    let mut injection = IMPORT_INIT_PUBLICATION_FAILURE
        .lock()
        .expect("import init-publication failure mutex poisoned");
    if let Some(index) = injection
        .iter()
        .position(|injection| injection.target == target && injection.point == point)
    {
        injection.remove(index);
        return Err(CliError::Io(std::io::Error::other(format!(
            "injected import init publication failure at {point:?}"
        ))));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_import_init_publication(
    _target: &Path,
    _point: ImportInitPublicationPoint,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn inject_import_init_publication_failure(target: &Path, point: ImportInitPublicationPoint) {
    IMPORT_INIT_PUBLICATION_FAILURE
        .lock()
        .expect("import init-publication failure mutex poisoned")
        .push(ImportInitPublicationFailure {
            target: target.to_path_buf(),
            point,
        });
}

#[cfg(test)]
fn maybe_publish_delayed_bootstrap_temp(target: &Path) -> Result<(), CliError> {
    let mut injection = IMPORT_DELAYED_BOOTSTRAP_TEMP
        .lock()
        .expect("delayed bootstrap temp mutex poisoned");
    let Some(index) = injection
        .iter()
        .position(|injection| injection.target == target)
    else {
        return Ok(());
    };
    let injection = injection.remove(index);
    let path = target.join(&injection.name);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path)?;
    file.write_all(&injection.contents)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(not(test))]
fn maybe_publish_delayed_bootstrap_temp(_target: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn inject_delayed_bootstrap_temp(target: &Path, name: String, contents: Vec<u8>) {
    IMPORT_DELAYED_BOOTSTRAP_TEMP
        .lock()
        .expect("delayed bootstrap temp mutex poisoned")
        .push(ImportDelayedBootstrapTemp {
            target: target.to_path_buf(),
            name,
            contents,
        });
}

#[cfg(test)]
fn maybe_pause_bootstrap_staging(target: &Path) {
    let pause = {
        let mut pauses = IMPORT_BOOTSTRAP_STAGING_PAUSE
            .lock()
            .expect("bootstrap staging pause mutex poisoned");
        pauses
            .iter()
            .position(|pause| pause.target == target)
            .map(|index| pauses.remove(index))
    };
    if let Some(pause) = pause {
        pause.reached.wait();
        pause.release.wait();
    }
}

#[cfg(not(test))]
fn maybe_pause_bootstrap_staging(_target: &Path) {}

#[cfg(test)]
fn inject_bootstrap_staging_pause(
    target: &Path,
    reached: std::sync::Arc<std::sync::Barrier>,
    release: std::sync::Arc<std::sync::Barrier>,
) {
    IMPORT_BOOTSTRAP_STAGING_PAUSE
        .lock()
        .expect("bootstrap staging pause mutex poisoned")
        .push(ImportBootstrapStagingPause {
            target: target.to_path_buf(),
            reached,
            release,
        });
}

#[cfg(all(test, unix))]
fn maybe_swap_import_target_during_init(target: &Path) -> Result<(), CliError> {
    let mut injection = IMPORT_TARGET_SWAP
        .lock()
        .expect("import target swap mutex poisoned");
    let Some(index) = injection
        .iter()
        .position(|injection| injection.target == target)
    else {
        return Ok(());
    };
    let injection = injection.remove(index);
    fs::rename(target, injection.moved)?;
    fs::create_dir(target)?;
    fs::write(
        target.join("replacement.txt"),
        b"replacement must stay untouched\n",
    )?;
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_swap_import_target_during_init(_target: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
fn inject_import_target_swap(target: &Path, moved: &Path) {
    IMPORT_TARGET_SWAP
        .lock()
        .expect("import target swap mutex poisoned")
        .push(ImportTargetSwap {
            target: target.to_path_buf(),
            moved: moved.to_path_buf(),
        });
}

#[derive(Debug, Clone)]
struct ImportTransactionResult {
    repo_root: PathBuf,
    open_dir: PathBuf,
    branch: String,
    base_commit: String,
    initial_files: usize,
    initial_state: String,
}

impl ImportTransactionResult {
    fn from_import_result(result: &ImportResult) -> Self {
        Self {
            repo_root: result.repo_root.clone(),
            open_dir: result.open_dir.clone(),
            branch: result.branch.clone(),
            base_commit: result.base_commit.clone(),
            initial_files: result.initial_files,
            initial_state: result.initial_state.clone(),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "repo_root": &self.repo_root,
            "open_dir": &self.open_dir,
            "branch": &self.branch,
            "base_commit": &self.base_commit,
            "initial_files": self.initial_files,
            "initial_state": &self.initial_state,
        })
    }
}

#[derive(Debug, Clone)]
struct ImportTransaction {
    transaction_id: String,
    target: PathBuf,
    branch: String,
    scope: Value,
    payload_version: u64,
    payload_hash: String,
    phase: ImportPhase,
    initial_files: usize,
    opened_at: String,
    open_instance_id: String,
    base_commit: Option<String>,
    result: Option<ImportTransactionResult>,
}

impl ImportTransaction {
    fn new(
        target: &Path,
        branch: &str,
        initial_files: usize,
        payload: &Value,
    ) -> Result<Self, CliError> {
        let opened_at = now_iso_utc();
        let payload_hash = idempotency_payload_hash(payload)?;
        let mut nonce = [0u8; 16];
        getrandom::getrandom(&mut nonce).map_err(|error| {
            CliError::InvalidRepository(format!(
                "failed to generate import transaction identity: {error}"
            ))
        })?;
        let transaction_id = format!("import_tx_{}", hex_bytes(&nonce));
        Ok(Self {
            transaction_id,
            target: target.to_path_buf(),
            branch: branch.to_string(),
            scope: import_payload_value(payload, &["scope"])?.clone(),
            payload_version: import_payload_u64(payload, &["version"])?,
            payload_hash,
            phase: ImportPhase::Planned,
            initial_files,
            opened_at: opened_at.clone(),
            open_instance_id: import_open_instance_id(branch, target, &opened_at),
            base_commit: None,
            result: None,
        })
    }

    fn active_path(&self) -> PathBuf {
        self.target
            .join(".codefire")
            .join("active")
            .join(&self.open_instance_id)
    }

    fn planned_seed(&self) -> Self {
        let mut seed = self.clone();
        seed.phase = ImportPhase::Planned;
        seed.base_commit = None;
        seed.result = None;
        seed
    }

    fn repository_initialization(&self) -> Value {
        json!({
            "type": "codefire_import_initialization",
            "version": 1,
            "operation": IMPORT_OPERATION,
            "transaction_id": &self.transaction_id,
            "payload_hash": &self.payload_hash,
            "branch": &self.branch,
            "target": &self.target,
            "opened_at": &self.opened_at,
            "open_instance_id": &self.open_instance_id,
        })
    }

    fn verify_request(&self, target: &Path, branch: &str, payload: &Value) -> Result<(), CliError> {
        let expected_hash = idempotency_payload_hash(payload)?;
        let expected_scope = import_payload_value(payload, &["scope"])?;
        let expected_version = import_payload_u64(payload, &["version"])?;
        if self.target != target
            || self.branch != branch
            || self.scope != *expected_scope
            || self.payload_version != expected_version
            || self.payload_hash != expected_hash
        {
            return Err(CliError::IdempotencyConflict(format!(
                "incomplete import transaction conflicts with the requested target or branch: {}",
                target.display()
            )));
        }
        Ok(())
    }

    fn live_result(
        &self,
        options: &ImportOptions,
        base_commit: &str,
        initial_state: &str,
    ) -> ImportResult {
        ImportResult {
            repo_root: self.target.clone(),
            open_dir: self.target.clone(),
            branch: self.branch.clone(),
            base_commit: base_commit.to_string(),
            initial_files: self.initial_files,
            initial_state: initial_state.to_string(),
            plan: import_operation_plan(
                options,
                &self.target,
                &self.target,
                &opened_registry_path(&self.target, &self.branch),
                &self.active_path(),
                self.initial_files,
            ),
        }
    }

    fn result(&self, target: &Path, options: &ImportOptions) -> Result<ImportResult, CliError> {
        let result = self.result.as_ref().ok_or_else(|| {
            invalid_import_transaction(
                &import_transaction_path(target),
                "scanned/completed transaction is missing result",
            )
        })?;
        Ok(ImportResult {
            repo_root: result.repo_root.clone(),
            open_dir: result.open_dir.clone(),
            branch: result.branch.clone(),
            base_commit: result.base_commit.clone(),
            initial_files: result.initial_files,
            initial_state: result.initial_state.clone(),
            plan: import_operation_plan(
                options,
                &result.repo_root,
                &result.open_dir,
                &opened_registry_path(&result.repo_root, &result.branch),
                &self.active_path(),
                result.initial_files,
            ),
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "type": IMPORT_TRANSACTION_TYPE,
            "version": IMPORT_TRANSACTION_VERSION,
            "operation": IMPORT_OPERATION,
            "transaction_id": &self.transaction_id,
            "scope": &self.scope,
            "payload_version": self.payload_version,
            "payload_hash": &self.payload_hash,
            "payload_mode": "hash_only",
            "phase": self.phase.as_str(),
            "target": &self.target,
            "branch": &self.branch,
            "initial_files": self.initial_files,
            "opened_at": &self.opened_at,
            "open_instance_id": &self.open_instance_id,
            "base_commit": self.base_commit,
            "result": self.result.as_ref().map(ImportTransactionResult::to_json),
            "updated_at": now_iso_utc(),
        })
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn import_transaction_path(target: &Path) -> PathBuf {
    target.join(IMPORT_TRANSACTION_FILE)
}

fn save_import_transaction_subject(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
) -> Result<(), CliError> {
    subject.write_control_json_atomic(Path::new(IMPORT_TRANSACTION_FILE), &transaction.to_json())
}

#[cfg(test)]
fn save_import_transaction(target: &Path, transaction: &ImportTransaction) -> Result<(), CliError> {
    write_json_atomic(&import_transaction_path(target), &transaction.to_json())
}

#[cfg(test)]
fn load_import_transaction(target: &Path) -> Result<Option<ImportTransaction>, CliError> {
    let path = import_transaction_path(target);
    let bytes = match read_absolute_import_control_file(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(invalid_import_transaction(&path, &error.to_string()))
        }
        Err(error) => return Err(CliError::Io(error)),
    };
    let record: Value = serde_json::from_slice(&bytes)
        .map_err(|error| invalid_import_transaction(&path, &format!("invalid JSON: {error}")))?;
    Ok(Some(parse_import_transaction_record(&record, &path)?))
}

fn load_import_transaction_from_dir(
    target: &Path,
    target_dir: &Dir,
) -> Result<Option<ImportTransaction>, CliError> {
    let path = import_transaction_path(target);
    let bytes = match read_capability_file(target_dir, IMPORT_TRANSACTION_FILE) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(invalid_import_transaction(&path, &error.to_string()))
        }
        Err(error) => return Err(CliError::Io(error)),
    };
    let record: Value = serde_json::from_slice(&bytes)
        .map_err(|error| invalid_import_transaction(&path, &format!("invalid JSON: {error}")))?;
    Ok(Some(parse_import_transaction_record(&record, &path)?))
}

fn parse_import_transaction_record(
    record: &Value,
    path: &Path,
) -> Result<ImportTransaction, CliError> {
    if transaction_string(record, path, &["type"])? != IMPORT_TRANSACTION_TYPE {
        return Err(invalid_import_transaction(path, "unexpected record type"));
    }
    if transaction_u64(record, path, &["version"])? != IMPORT_TRANSACTION_VERSION {
        return Err(invalid_import_transaction(
            path,
            "unsupported transaction version",
        ));
    }
    if transaction_string(record, path, &["operation"])? != IMPORT_OPERATION {
        return Err(invalid_import_transaction(path, "unexpected operation"));
    }
    if transaction_string(record, path, &["payload_mode"])? != "hash_only" {
        return Err(invalid_import_transaction(
            path,
            "payload_mode must be hash_only",
        ));
    }
    let transaction_id = transaction_string(record, path, &["transaction_id"])?;
    let target_path = PathBuf::from(transaction_string(record, path, &["target"])?);
    let branch = transaction_string(record, path, &["branch"])?;
    let scope = transaction_value(record, path, &["scope"])?.clone();
    let payload_version = transaction_u64(record, path, &["payload_version"])?;
    let payload_hash = transaction_string(record, path, &["payload_hash"])?;
    let phase = ImportPhase::parse(&transaction_string(record, path, &["phase"])?, path)?;
    let initial_files = transaction_usize(record, path, &["initial_files"])?;
    let opened_at = transaction_string(record, path, &["opened_at"])?;
    let open_instance_id = transaction_string(record, path, &["open_instance_id"])?;
    let base_commit = match transaction_value(record, path, &["base_commit"])? {
        Value::Null => None,
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        _ => {
            return Err(invalid_import_transaction(
                path,
                "base_commit must be null or a non-empty string",
            ))
        }
    };
    let result = match transaction_value(record, path, &["result"])? {
        Value::Null => None,
        Value::Object(_) => Some(ImportTransactionResult {
            repo_root: PathBuf::from(transaction_string(record, path, &["result", "repo_root"])?),
            open_dir: PathBuf::from(transaction_string(record, path, &["result", "open_dir"])?),
            branch: transaction_string(record, path, &["result", "branch"])?,
            base_commit: transaction_string(record, path, &["result", "base_commit"])?,
            initial_files: transaction_usize(record, path, &["result", "initial_files"])?,
            initial_state: transaction_string(record, path, &["result", "initial_state"])?,
        }),
        _ => {
            return Err(invalid_import_transaction(
                path,
                "result must be null or an object",
            ))
        }
    };
    let expected_open_instance = import_open_instance_id(&branch, &target_path, &opened_at);
    if transaction_id.is_empty()
        || payload_hash.len() != 64
        || open_instance_id != expected_open_instance
    {
        return Err(invalid_import_transaction(
            path,
            "transaction identity fields are inconsistent",
        ));
    }
    if phase != ImportPhase::Planned && base_commit.is_none() {
        return Err(invalid_import_transaction(
            path,
            "initialized transaction is missing base_commit",
        ));
    }
    if matches!(phase, ImportPhase::Scanned | ImportPhase::Completed) && result.is_none() {
        return Err(invalid_import_transaction(
            path,
            "scanned/completed transaction is missing result",
        ));
    }
    if let (Some(base), Some(result)) = (&base_commit, &result) {
        if result.repo_root != target_path
            || result.open_dir != target_path
            || result.branch != branch
            || result.base_commit != *base
            || result.initial_files != initial_files
            || result.initial_state.is_empty()
        {
            return Err(invalid_import_transaction(
                path,
                "transaction result does not match its subject",
            ));
        }
    }
    Ok(ImportTransaction {
        transaction_id,
        target: target_path,
        branch,
        scope,
        payload_version,
        payload_hash,
        phase,
        initial_files,
        opened_at,
        open_instance_id,
        base_commit,
        result,
    })
}

fn invalid_import_transaction(path: &Path, detail: &str) -> CliError {
    CliError::InvalidRepository(format!(
        "invalid import transaction {}: {detail}",
        path.display()
    ))
}

fn transaction_value<'a>(
    record: &'a Value,
    path: &Path,
    fields: &[&str],
) -> Result<&'a Value, CliError> {
    let mut current = record;
    for field in fields {
        current = current.get(*field).ok_or_else(|| {
            invalid_import_transaction(path, &format!("missing {}", fields.join(".")))
        })?;
    }
    Ok(current)
}

fn transaction_string(record: &Value, path: &Path, fields: &[&str]) -> Result<String, CliError> {
    transaction_value(record, path, fields)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            invalid_import_transaction(path, &format!("{} must be a string", fields.join(".")))
        })
}

fn transaction_u64(record: &Value, path: &Path, fields: &[&str]) -> Result<u64, CliError> {
    transaction_value(record, path, fields)?
        .as_u64()
        .ok_or_else(|| {
            invalid_import_transaction(
                path,
                &format!("{} must be an unsigned integer", fields.join(".")),
            )
        })
}

fn transaction_usize(record: &Value, path: &Path, fields: &[&str]) -> Result<usize, CliError> {
    usize::try_from(transaction_u64(record, path, fields)?).map_err(|_| {
        invalid_import_transaction(path, &format!("{} is too large", fields.join(".")))
    })
}

struct ImportInitStage {
    path: PathBuf,
    base_commit: String,
}

impl ImportInitStage {
    fn build(subject: &ImportSubject, transaction: &ImportTransaction) -> Result<Self, CliError> {
        let path = std::env::temp_dir().join(format!(
            "codefire-import-init-{}-{}",
            std::process::id(),
            random_import_token()?
        ));
        fs::create_dir(&path)?;
        let initialization = transaction.repository_initialization();
        let init = match init_repo_with_metadata(&path, false, Some(&initialization)) {
            Ok(init) => init,
            Err(error) => {
                let _ = fs::remove_dir_all(&path);
                return Err(error);
            }
        };
        codefire_store::validate_sealed_commit(
            &path.join(".codefire").join("objects"),
            &init.main_commit,
        )?;
        let repo_path = path.join(".codefire").join("repo.json");
        let repo_bytes = read_absolute_import_control_file(&repo_path)?;
        let mut repo: Value = serde_json::from_slice(&repo_bytes)?;
        repo["repository_id"] = Value::String(stable_repo_id(&subject.target));
        repo["created_at"] = Value::String(transaction.opened_at.clone());
        write_json_atomic(&repo_path, &repo)?;
        Ok(Self {
            path,
            base_commit: init.main_commit,
        })
    }

    fn publish(&self, subject: &ImportSubject) -> Result<(), CliError> {
        subject.create_dir_all(Path::new(".codefire"))?;
        maybe_fail_import_init_layout(&subject.target)?;
        let stage_cf = self.path.join(".codefire");
        let repo_bytes = read_absolute_import_control_file(&stage_cf.join("repo.json"))?;
        let repo_relative = Path::new(".codefire/repo.json");
        match subject.read_bytes(repo_relative)? {
            Some(existing) => {
                let existing: Value = serde_json::from_slice(&existing)?;
                let staged: Value = serde_json::from_slice(&repo_bytes)?;
                if existing != staged {
                    return Err(CliError::InvalidRepository(
                        "partial import repo.json differs from the transaction subject".to_string(),
                    ));
                }
            }
            None => subject.write_bytes_atomic(repo_relative, &repo_bytes)?,
        }
        maybe_swap_import_target_during_init(&subject.target)?;
        publish_staged_directory(subject, &stage_cf, Path::new(""), true)?;
        maybe_fail_import_init_publication(
            &subject.target,
            ImportInitPublicationPoint::BeforeBranch,
        )?;
        publish_staged_directory(subject, &stage_cf, Path::new("branches"), false)?;
        maybe_fail_import_init_publication(
            &subject.target,
            ImportInitPublicationPoint::AfterBranch,
        )?;
        Ok(())
    }
}

impl Drop for ImportInitStage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct ImportReadSnapshot {
    path: PathBuf,
}

impl ImportReadSnapshot {
    fn build(subject: &ImportSubject) -> Result<Self, CliError> {
        let path = std::env::temp_dir().join(format!(
            "codefire-import-read-{}-{}",
            std::process::id(),
            random_import_token()?
        ));
        fs::create_dir(&path)?;
        if let Err(error) = copy_capability_directory(&subject.root, &path, Path::new("")) {
            let _ = fs::remove_dir_all(&path);
            return Err(error);
        }
        Ok(Self { path })
    }

    fn prepare_open_context(&self, transaction: &ImportTransaction) -> Result<(), CliError> {
        let marker_path = self.path.join(".codefire-open");
        let marker_bytes = read_absolute_import_control_file(&marker_path)?;
        let mut marker: Value = serde_json::from_slice(&marker_bytes)?;
        marker["repository"]["path"] =
            Value::String(self.path.join(".codefire").to_string_lossy().into_owned());
        marker["open"]["opened_path"] = Value::String(self.path.to_string_lossy().into_owned());
        write_json_atomic(&marker_path, &marker)?;

        let registry_path = opened_registry_path(&self.path, &transaction.branch);
        let registry_bytes = read_absolute_import_control_file(&registry_path)?;
        let mut registry: Value = serde_json::from_slice(&registry_bytes)?;
        registry["open"]["path"] = Value::String(self.path.to_string_lossy().into_owned());
        registry["open"]["active_state_path"] = Value::String(
            self.path
                .join(".codefire")
                .join("active")
                .join(&transaction.open_instance_id)
                .to_string_lossy()
                .into_owned(),
        );
        write_json_atomic(&registry_path, &registry)
    }

    fn objects_root(&self) -> PathBuf {
        self.path.join(".codefire").join("objects")
    }
}

impl Drop for ImportReadSnapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn preflight_import_snapshot(target: &Path) -> Result<(), CliError> {
    let root = open_absolute_dir_nofollow(target)?;
    preflight_import_snapshot_directory(target, &root, Path::new(""))
}

fn preflight_import_snapshot_directory(
    target: &Path,
    source: &Dir,
    relative: &Path,
) -> Result<(), CliError> {
    let mut entries = source
        .entries()?
        .map(|entry| entry.map(|entry| (entry.file_name(), entry)))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, entry) in entries {
        let child_relative = relative.join(&name);
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(CliError::Io(error)),
        };
        if file_type.is_dir() {
            let child = match source.open_dir_nofollow(&name) {
                Ok(child) => child,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(CliError::Io(error)),
            };
            preflight_import_snapshot_directory(target, &child, &child_relative)?;
        } else if file_type.is_file() {
            let limit = import_snapshot_file_limit(&child_relative);
            let mut options = CapabilityOpenOptions::new();
            options.read(true);
            options.follow(FollowSymlinks::No);
            options.nonblock(true);
            let file = match source.open_with(&name, &options) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(CliError::InvalidRepository(format!(
                        "cannot safely open import snapshot file {}: {error}",
                        target.join(&child_relative).display()
                    )))
                }
            };
            ensure_import_file_metadata(&file.metadata()?, &target.join(&child_relative), limit)
                .map_err(|error| CliError::InvalidRepository(error.to_string()))?;
        } else {
            return Err(CliError::InvalidRepository(format!(
                "import snapshot entry is not a regular file or directory: {}",
                target.join(child_relative).display()
            )));
        }
    }
    Ok(())
}

fn import_snapshot_file_limit(relative: &Path) -> u64 {
    if relative
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == OsStr::new(".codefire"))
    {
        return IMPORT_REPOSITORY_FILE_BYTE_LIMIT;
    }
    if is_root_import_control_path(relative) {
        return IMPORT_CONTROL_FILE_BYTE_LIMIT;
    }
    IMPORT_USER_FILE_BYTE_LIMIT
}

fn is_root_import_control_path(relative: &Path) -> bool {
    if relative.components().count() != 1 {
        return false;
    }
    let Some(name) = relative.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    matches!(
        name,
        ".codefire-open" | IMPORT_BOOTSTRAP_LOCK_FILE | IMPORT_TRANSACTION_FILE
    ) || (name.starts_with(IMPORT_BOOTSTRAP_TEMP_PREFIX)
        && (name.ends_with(IMPORT_BOOTSTRAP_TEMP_SUFFIX)
            || name.ends_with(IMPORT_BOOTSTRAP_STAGING_SUFFIX)))
        || (name.starts_with(IMPORT_WRITE_TEMP_PREFIX) && name.ends_with(IMPORT_WRITE_TEMP_SUFFIX))
}

fn copy_capability_directory(
    source: &Dir,
    destination: &Path,
    relative: &Path,
) -> Result<(), CliError> {
    let mut entries = source
        .entries()?
        .map(|entry| entry.map(|entry| (entry.file_name(), entry)))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, entry) in entries {
        let file_type = entry.file_type()?;
        let output = destination.join(&name);
        let child_relative = relative.join(&name);
        if file_type.is_dir() {
            let child = source.open_dir_nofollow(&name)?;
            fs::create_dir(&output)?;
            copy_capability_directory(&child, &output, &child_relative)?;
        } else if file_type.is_file() {
            fs::write(
                &output,
                read_snapshot_capability_file(
                    source,
                    Path::new(&name),
                    import_snapshot_file_limit(&child_relative),
                )?,
            )?;
        } else {
            return Err(CliError::InvalidRepository(format!(
                "import snapshot contains unsupported entry: {}",
                child_relative.display()
            )));
        }
    }
    Ok(())
}

fn publish_staged_directory(
    subject: &ImportSubject,
    stage_root: &Path,
    relative: &Path,
    defer_root_branches: bool,
) -> Result<(), CliError> {
    let mut entries = fs::read_dir(stage_root.join(relative))?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let child = relative.join(&name);
        if defer_root_branches && relative.as_os_str().is_empty() && child == Path::new("branches")
        {
            continue;
        }
        let target_relative = PathBuf::from(".codefire").join(&child);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            subject.create_dir_all(&target_relative)?;
            publish_staged_directory(subject, stage_root, &child, false)?;
        } else if file_type.is_file() {
            if child == Path::new("repo.json") {
                continue;
            }
            subject.write_bytes_if_absent_or_equal(
                &target_relative,
                &read_absolute_repository_file(&entry.path())?,
            )?;
        } else {
            return Err(CliError::InvalidRepository(format!(
                "staged import repository contains unsupported entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn ensure_import_repository(
    subject: &ImportSubject,
    branch: &str,
    transaction: &ImportTransaction,
) -> Result<String, CliError> {
    let target = &subject.target;
    let cf = target.join(".codefire");
    if !subject.exists(Path::new(".codefire"))? {
        if transaction.base_commit.is_some() {
            return Err(invalid_import_transaction(
                &import_transaction_path(target),
                "repository disappeared after initialization",
            ));
        }
        let stage = ImportInitStage::build(subject, transaction)?;
        stage.publish(subject)?;
        return Ok(stage.base_commit.clone());
    }

    if !subject.exists(Path::new(".codefire/repo.json"))? {
        if transaction.base_commit.is_some() {
            return Err(invalid_import_transaction(
                &import_transaction_path(target),
                "repository disappeared after initialization",
            ));
        }
        validate_preinit_repo_lock_layout(subject, &cf)?;
        let stage = ImportInitStage::build(subject, transaction)?;
        stage.publish(subject)?;
        return Ok(stage.base_commit.clone());
    }

    validate_import_repository_identity(subject)?;
    validate_import_repository_provenance(subject, transaction)?;
    let requested_relative = PathBuf::from(".codefire")
        .join("branches")
        .join(format!("{}.json", ref_file_name(branch)));
    let main_relative =
        PathBuf::from(".codefire/branches").join(format!("{}.json", ref_file_name("main")));
    let record = if subject.exists(&requested_relative)? {
        subject.read_required_json(&requested_relative)?
    } else if branch != "main" && subject.exists(&main_relative)? {
        subject.read_required_json(&main_relative)?
    } else {
        if transaction.base_commit.is_some() {
            return Err(invalid_import_transaction(
                &import_transaction_path(target),
                "initialized repository is missing its base branch",
            ));
        }
        // Repository metadata is the first durable init write and binds this
        // directory to the import transaction. If init crashed before publishing
        // its initial branch, replaying the deterministic initialization is safe.
        let stage = ImportInitStage::build(subject, transaction)?;
        stage.publish(subject)?;
        return Ok(stage.base_commit.clone());
    };
    let base_commit = required_string(&record, &["head"])?;
    if let Some(expected) = transaction.base_commit.as_deref() {
        if base_commit != expected {
            return Err(CliError::InvalidRepository(format!(
                "incomplete import base changed: expected {expected}, found {base_commit}"
            )));
        }
    }
    validate_subject_sealed_commit(subject, &base_commit)?;
    Ok(base_commit)
}

fn validate_preinit_repo_lock_layout(subject: &ImportSubject, cf: &Path) -> Result<(), CliError> {
    let directory = subject.open_dir(Path::new(".codefire"), false)?;
    let mut entries = directory
        .entries()?
        .map(|entry| entry.map(|entry| (entry.file_name(), entry.file_type())))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.len() != 1
        || entries[0].0 != OsStr::new("locks")
        || !entries[0]
            .1
            .as_ref()
            .is_ok_and(|file_type| file_type.is_dir())
    {
        return Err(CliError::InvalidRepository(format!(
            "partial import layout has no provenance beyond its repository lock: {}",
            cf.display()
        )));
    }
    let locks = directory.open_dir_nofollow("locks")?;
    let mut lock_entries = locks
        .entries()?
        .map(|entry| entry.map(|entry| (entry.file_name(), entry.file_type())))
        .collect::<Result<Vec<_>, _>>()?;
    lock_entries.sort_by(|left, right| left.0.cmp(&right.0));
    let lock_layout_is_valid = lock_entries.len() == 2
        && ["repo.lock", "repo.lock.owner.json"]
            .iter()
            .all(|expected| {
                lock_entries.iter().any(|(name, file_type)| {
                    name == OsStr::new(expected)
                        && file_type
                            .as_ref()
                            .is_ok_and(|file_type| file_type.is_file())
                })
            });
    if !lock_layout_is_valid {
        return Err(CliError::InvalidRepository(format!(
            "partial import lock layout is invalid: {}",
            cf.join("locks").display()
        )));
    }
    let owner_path = cf.join("locks").join("repo.lock.owner.json");
    let owner_bytes = read_capability_file_with_limit(
        &locks,
        Path::new("repo.lock.owner.json"),
        REPO_LOCK_OWNER_BYTE_LIMIT,
    )?;
    let owner: Value = serde_json::from_slice(&owner_bytes).map_err(|error| {
        CliError::InvalidRepository(format!(
            "invalid repository lock owner metadata {}: {error}",
            owner_path.display()
        ))
    })?;
    let owner_object = owner.as_object().ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "repository lock owner metadata must be an object: {}",
            owner_path.display()
        ))
    })?;
    if owner_object.len() != 3
        || owner.get("version").and_then(Value::as_u64) != Some(1)
        || owner.get("pid").and_then(Value::as_u64).is_none()
        || !owner
            .get("created_at")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
    {
        return Err(CliError::InvalidRepository(format!(
            "repository lock owner metadata has an invalid schema: {}",
            owner_path.display()
        )));
    }
    Ok(())
}

fn validate_import_repository_provenance(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
) -> Result<(), CliError> {
    let path = subject.target.join(".codefire").join("repo.json");
    let repo = subject.read_required_json(Path::new(".codefire/repo.json"))?;
    let expected = json!({
        "version": 1,
        "repository_id": stable_repo_id(&subject.target),
        "created_at": &transaction.opened_at,
        "initialization": transaction.repository_initialization(),
    });
    if repo != expected {
        return Err(CliError::InvalidRepository(format!(
            "repository metadata does not exactly match the pending import transaction: {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_import_repository_identity(subject: &ImportSubject) -> Result<String, CliError> {
    let target = &subject.target;
    let repo_path = target.join(".codefire").join("repo.json");
    let repo = subject.read_required_json(Path::new(".codefire/repo.json"))?;
    let repository_id = required_string(&repo, &["repository_id"])?;
    let expected = stable_repo_id(target);
    if repository_id != expected {
        return Err(CliError::InvalidRepository(format!(
            "import repository identity mismatch at {}: expected {expected}, found {repository_id}",
            repo_path.display()
        )));
    }
    Ok(repository_id)
}

fn validate_subject_sealed_commit(
    subject: &ImportSubject,
    commit_id: &str,
) -> Result<(), CliError> {
    let snapshot = ImportReadSnapshot::build(subject)?;
    codefire_store::validate_sealed_commit(&snapshot.objects_root(), commit_id)?;
    Ok(())
}

fn save_import_branch_subject(subject: &ImportSubject, branch: &Value) -> Result<(), CliError> {
    let name = required_string(branch, &["name"])?;
    let relative = PathBuf::from(".codefire")
        .join("branches")
        .join(format!("{}.json", ref_file_name(&name)));
    subject.store_object("branch", branch.clone())?;
    subject.write_json_atomic(&relative, branch)?;
    Ok(())
}

fn ensure_import_branch(
    subject: &ImportSubject,
    branch: &str,
    base_commit: &str,
) -> Result<(), CliError> {
    let main_relative =
        PathBuf::from(".codefire/branches").join(format!("{}.json", ref_file_name("main")));
    let requested_relative =
        PathBuf::from(".codefire/branches").join(format!("{}.json", ref_file_name(branch)));
    if !subject.exists(&requested_relative)? {
        if branch == "main" || !subject.exists(&main_relative)? {
            return Err(CliError::InvalidRepository(format!(
                "incomplete import branch is missing: {branch}"
            )));
        }
        let main = subject.read_required_json(&main_relative)?;
        require_import_branch_subject(&main, "main", base_commit)?;
        let requested = json!({
            "type": "branch",
            "version": 1,
            "name": branch,
            "head": base_commit,
            "state": "closed",
            "created_from": "import",
            "created_at": now_iso_utc(),
        });
        save_import_branch_subject(subject, &requested)?;
    }

    let requested = subject.read_required_json(&requested_relative)?;
    require_import_branch_subject(&requested, branch, base_commit)?;
    if branch != "main" && subject.exists(&main_relative)? {
        let main = subject.read_required_json(&main_relative)?;
        require_import_branch_subject(&main, "main", base_commit)?;
        // `rename_initial_branch` publishes the requested record before removing
        // main. A crash in that narrow window is resumed by this exact cleanup.
        subject.remove_file_if_exists(&main_relative)?;
    }
    Ok(())
}

fn require_import_branch_subject(
    record: &Value,
    expected_name: &str,
    expected_head: &str,
) -> Result<(), CliError> {
    let name = required_string(record, &["name"])?;
    let head = required_string(record, &["head"])?;
    if name != expected_name || head != expected_head {
        return Err(CliError::InvalidRepository(format!(
            "incomplete import branch subject mismatch: expected {expected_name}@{expected_head}, found {name}@{head}"
        )));
    }
    Ok(())
}

fn ensure_import_open_state(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
    base_commit: &str,
) -> Result<(), CliError> {
    let target = subject.target.as_path();
    let cf = target.join(".codefire");
    let repository_id = validate_import_repository_identity(subject)?;
    let active_path = transaction.active_path();
    let marker_path = target.join(".codefire-open");
    let marker = json!({
        "version": 1,
        "repository": {"path": &cf, "repository_id": &repository_id},
        "branch": {"name": &transaction.branch, "opened_from_commit": base_commit},
        "open": {
            "open_instance_id": &transaction.open_instance_id,
            "opened_path": target,
            "opened_at": &transaction.opened_at,
        },
        "import": {"mode": IMPORT_MODE, "initial_files": transaction.initial_files},
    });
    write_or_validate_exact_json(
        subject,
        Path::new(".codefire-open"),
        &marker_path,
        &marker,
        "import open marker",
    )?;

    let registry = json!({
        "version": 1,
        "branch": {"name": &transaction.branch},
        "open": {
            "open_instance_id": &transaction.open_instance_id,
            "path": target,
            "opened_from_commit": base_commit,
            "current_base_commit": base_commit,
            "active_state_path": &active_path,
        },
        "state": {"last_known": "open-burning"},
        "import": {"mode": IMPORT_MODE, "initial_files": transaction.initial_files},
    });
    let registry_relative = PathBuf::from(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    if subject.exists(&registry_relative)? {
        validate_import_registry_subject(
            &subject.read_required_json(&registry_relative)?,
            target,
            &transaction.branch,
            base_commit,
            &transaction.open_instance_id,
            &active_path,
            transaction.initial_files,
        )?;
    } else {
        subject.write_json_atomic(&registry_relative, &registry)?;
    }

    subject.create_dir_all(
        &PathBuf::from(".codefire")
            .join("active")
            .join(&transaction.open_instance_id),
    )?;
    let state = json!({
        "state": "open-burning",
        "import": {
            "mode": IMPORT_MODE,
            "initial_files": transaction.initial_files,
            "base_commit": base_commit,
        },
    });
    let state_relative = PathBuf::from(".codefire")
        .join("active")
        .join(&transaction.open_instance_id)
        .join("state.json");
    if subject.exists(&state_relative)? {
        validate_import_active_state(
            &subject.read_required_json(&state_relative)?,
            base_commit,
            transaction.initial_files,
        )?;
    } else {
        subject.write_json_atomic(&state_relative, &state)?;
    }

    let branch_relative = PathBuf::from(".codefire/branches")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    let mut branch = subject.read_required_json(&branch_relative)?;
    let branch_state = required_string(&branch, &["state"])?;
    if branch_state == "closed" {
        branch["state"] = Value::String("open-burning".to_string());
        save_import_branch_subject(subject, &branch)?;
    } else if !is_open_branch_state(&branch_state) {
        return Err(CliError::InvalidRepository(format!(
            "incomplete import branch has invalid state: {branch_state}"
        )));
    }
    Ok(())
}

fn write_or_validate_exact_json(
    subject: &ImportSubject,
    relative: &Path,
    path: &Path,
    expected: &Value,
    label: &str,
) -> Result<(), CliError> {
    if let Some(actual) = subject.read_json(relative)? {
        if actual != *expected {
            return Err(CliError::InvalidRepository(format!(
                "{label} differs from the pending import transaction: {}",
                path.display()
            )));
        }
        Ok(())
    } else {
        subject.write_json_atomic(relative, expected)
    }
}

fn is_open_branch_state(state: &str) -> bool {
    matches!(state, "open-burning" | "open-clean" | "open-consistent")
}

fn validate_import_registry_subject(
    registry: &Value,
    target: &Path,
    branch: &str,
    opened_from_commit: &str,
    open_instance_id: &str,
    active_path: &Path,
    initial_files: usize,
) -> Result<(), CliError> {
    let matches_subject = required_string(registry, &["branch", "name"])? == branch
        && PathBuf::from(required_string(registry, &["open", "path"])?) == target
        && required_string(registry, &["open", "opened_from_commit"])? == opened_from_commit
        && required_string(registry, &["open", "current_base_commit"])? == opened_from_commit
        && required_string(registry, &["open", "open_instance_id"])? == open_instance_id
        && PathBuf::from(required_string(registry, &["open", "active_state_path"])?) == active_path
        && required_string(registry, &["import", "mode"])? == IMPORT_MODE
        && import_subject_usize(registry, &["import", "initial_files"])? == initial_files;
    let state = required_string(registry, &["state", "last_known"])?;
    if !matches_subject || !is_open_branch_state(&state) {
        return Err(CliError::InvalidRepository(
            "opened registry differs from the pending import transaction".to_string(),
        ));
    }
    Ok(())
}

fn validate_import_active_state(
    state: &Value,
    base_commit: &str,
    initial_files: usize,
) -> Result<(), CliError> {
    let current = required_string(state, &["state"])?;
    if !is_open_branch_state(&current)
        || required_string(state, &["import", "mode"])? != IMPORT_MODE
        || required_string(state, &["import", "base_commit"])? != base_commit
        || import_subject_usize(state, &["import", "initial_files"])? != initial_files
    {
        return Err(CliError::InvalidRepository(
            "active import state differs from the pending import transaction".to_string(),
        ));
    }
    Ok(())
}

fn persist_import_scan(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
    execution: &ScanExecution,
) -> Result<String, CliError> {
    let state = scan_branch_state(
        scan_change_count(&execution.scan),
        execution.scan.open_fires.len(),
    )
    .to_string();
    let active = PathBuf::from(".codefire")
        .join("active")
        .join(&transaction.open_instance_id);
    let state_transaction_path = active.join("state_transaction.json");
    let registry_relative = PathBuf::from(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    let state_relative = active.join("state.json");
    let branch_relative = PathBuf::from(".codefire/branches")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    let registry_before = subject.read_required_json(&registry_relative)?;
    let active_before = subject.read_required_json(&state_relative)?;
    let branch_before = subject.read_required_json(&branch_relative)?;
    let pre_branch = required_string(&branch_before, &["state"])?;
    let pre_registry = required_string(&registry_before, &["state", "last_known"])?;
    let pre_active = required_string(&active_before, &["state"])?;
    if pre_branch != pre_registry || pre_branch != pre_active {
        return Err(CliError::InvalidRepository(
            "import scan cannot begin from divergent open-state authorities".to_string(),
        ));
    }
    let current_base_commit = transaction.base_commit.as_deref().ok_or_else(|| {
        CliError::InvalidRepository(
            "import scan state transaction is missing its base commit subject".to_string(),
        )
    })?;
    let now = now_iso_utc();
    let mut state_transaction = json!({
        "version": 2,
        "status": "pending",
        "operation": "scan",
        "phase": "started",
        "intended_state": &state,
        "subject": {
            "branch": &transaction.branch,
            "open_instance_id": &transaction.open_instance_id,
            "current_base_commit": current_base_commit,
        },
        "pre_state": {
            "branch": &pre_branch,
            "registry": &pre_registry,
            "active": &pre_active,
        },
        "started_at": &now,
        "updated_at": &now,
    });
    if let Some(existing) = subject.read_json(&state_transaction_path)? {
        validate_state_transaction_record(&existing).map_err(|message| {
            CliError::InvalidRepository(format!(
                "{message}: {}",
                subject.target.join(&state_transaction_path).display()
            ))
        })?;
    }
    subject.write_json_atomic(&state_transaction_path, &state_transaction)?;

    subject.remove_file_if_exists(&active.join("verification.json"))?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "verification_invalidated",
    )?;

    subject.write_json_atomic(
        &active.join("fires.json"),
        &serde_json::to_value(&execution.fires)?,
    )?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "fires_written",
    )?;
    subject.write_json_atomic(
        &active.join("scan.json"),
        &serde_json::to_value(&execution.scan)?,
    )?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "scan_written",
    )?;

    let mut registry = subject.read_required_json(&registry_relative)?;
    registry["state"]["last_known"] = Value::String(state.clone());
    subject.write_json_atomic(&registry_relative, &registry)?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "registry_written",
    )?;

    let mut active_state = subject
        .read_json(&state_relative)?
        .unwrap_or_else(|| json!({}));
    active_state["state"] = Value::String(state.clone());
    subject.write_json_atomic(&state_relative, &active_state)?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "active_state_written",
    )?;

    let mut branch = subject.read_required_json(&branch_relative)?;
    branch["state"] = Value::String(state.clone());
    save_import_branch_subject(subject, &branch)?;
    update_import_state_transaction(
        subject,
        &state_transaction_path,
        &mut state_transaction,
        "branch_written",
    )?;
    complete_import_state_transaction(subject, transaction, &state_transaction_path, &state)?;
    Ok(state)
}

fn update_import_state_transaction(
    subject: &ImportSubject,
    path: &Path,
    transaction: &mut Value,
    phase: &str,
) -> Result<(), CliError> {
    validate_state_transaction_record(transaction).map_err(|message| {
        CliError::InvalidRepository(format!(
            "{message}: {}",
            subject.target.join(path).display()
        ))
    })?;
    let operation = required_string(transaction, &["operation"])?;
    let current_phase = required_string(transaction, &["phase"])?;
    if !state_transaction_transition_is_supported(&operation, &current_phase, phase) {
        return Err(CliError::InvalidRepository(format!(
            "state transaction operation {operation} cannot advance from {current_phase} to {phase}: {}",
            subject.target.join(path).display()
        )));
    }
    transaction["phase"] = Value::String(phase.to_string());
    transaction["updated_at"] = Value::String(now_iso_utc());
    subject.write_json_atomic(path, transaction)
}

fn complete_import_state_transaction(
    subject: &ImportSubject,
    transaction: &ImportTransaction,
    path: &Path,
    intended_state: &str,
) -> Result<(), CliError> {
    let registry_relative = PathBuf::from(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    let active = PathBuf::from(".codefire")
        .join("active")
        .join(&transaction.open_instance_id);
    let branch_relative = PathBuf::from(".codefire/branches")
        .join(format!("{}.json", ref_file_name(&transaction.branch)));
    let registry = subject.read_required_json(&registry_relative)?;
    let active_state = subject.read_required_json(&active.join("state.json"))?;
    let branch = subject.read_required_json(&branch_relative)?;
    for (label, actual) in [
        (
            "open registry",
            required_string(&registry, &["state", "last_known"])?,
        ),
        ("active state", required_string(&active_state, &["state"])?),
        ("branch", required_string(&branch, &["state"])?),
    ] {
        if actual != intended_state {
            return Err(CliError::InvalidRepository(format!(
                "import state transaction expected {intended_state}, but {label} contains {actual}"
            )));
        }
    }
    subject.remove_file_if_exists(path)
}

fn import_subject_usize(value: &Value, fields: &[&str]) -> Result<usize, CliError> {
    let mut current = value;
    for field in fields {
        current = current.get(*field).ok_or_else(|| {
            CliError::InvalidRepository(format!("import subject is missing {}", fields.join(".")))
        })?;
    }
    let value = current.as_u64().ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "import subject {} must be an unsigned integer",
            fields.join(".")
        ))
    })?;
    usize::try_from(value).map_err(|_| {
        CliError::InvalidRepository(format!(
            "import subject {} exceeds platform limits",
            fields.join(".")
        ))
    })
}

fn validate_import_replay_subject(
    subject: &ImportSubject,
    payload: &Value,
    transaction: &ImportTransaction,
    result: &ImportResult,
) -> Result<(), CliError> {
    let target = subject.target.as_path();
    transaction.verify_request(target, &result.branch, payload)?;
    let recorded = transaction.result.as_ref().ok_or_else(|| {
        CliError::InvalidRepository(
            "import replay transaction is missing its result subject".to_string(),
        )
    })?;
    if recorded.repo_root != result.repo_root
        || recorded.open_dir != result.open_dir
        || recorded.branch != result.branch
        || recorded.base_commit != result.base_commit
        || recorded.initial_files != result.initial_files
        || recorded.initial_state != result.initial_state
    {
        return Err(CliError::InvalidRepository(
            "import replay transaction result does not match the idempotency result".to_string(),
        ));
    }
    let expected_target = PathBuf::from(import_payload_string(payload, &["target"])?);
    let expected_branch = import_payload_string(payload, &["branch"])?;
    if expected_target != target
        || result.repo_root != target
        || result.open_dir != target
        || result.branch != expected_branch
        || result.base_commit.is_empty()
    {
        return Err(CliError::InvalidRepository(
            "import replay result does not identify the bound target, branch, and base".to_string(),
        ));
    }

    let cf = target.join(".codefire");
    let repository_id = validate_import_repository_identity(subject)?;
    validate_import_repository_provenance(subject, transaction)?;
    let snapshot = ImportReadSnapshot::build(subject)?;
    let objects = snapshot.objects_root();
    codefire_store::validate_sealed_commit(&objects, &result.base_commit)?;

    let branch_relative = PathBuf::from(".codefire")
        .join("branches")
        .join(format!("{}.json", ref_file_name(&result.branch)));
    let branch = subject.read_required_json(&branch_relative)?;
    if required_string(&branch, &["name"])? != result.branch {
        return Err(CliError::InvalidRepository(
            "import replay branch record name mismatch".to_string(),
        ));
    }
    let branch_head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &branch_head)?;

    let marker_path = target.join(".codefire-open");
    let marker = subject.read_required_json(Path::new(".codefire-open"))?;
    let marker_repository_path = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let marker_open_path = PathBuf::from(required_string(&marker, &["open", "opened_path"])?);
    let open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    if marker_repository_path != cf
        || required_string(&marker, &["repository", "repository_id"])? != repository_id
        || marker_open_path != target
        || required_string(&marker, &["branch", "name"])? != result.branch
        || required_string(&marker, &["branch", "opened_from_commit"])? != result.base_commit
        || open_instance_id != transaction.open_instance_id
        || required_string(&marker, &["open", "opened_at"])? != transaction.opened_at
        || required_string(&marker, &["import", "mode"])? != IMPORT_MODE
        || import_subject_usize(&marker, &["import", "initial_files"])? != result.initial_files
    {
        return Err(CliError::InvalidRepository(format!(
            "import replay open marker does not match its recorded subject: {}",
            marker_path.display()
        )));
    }

    let registry_path = opened_registry_path(target, &result.branch);
    let registry_relative = PathBuf::from(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(&result.branch)));
    let registry = subject.read_required_json(&registry_relative)?;
    let current_base = required_string(&registry, &["open", "current_base_commit"])?;
    let active_path = PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let expected_active_path = cf.join("active").join(&open_instance_id);
    let registry_state = required_string(&registry, &["state", "last_known"])?;
    if required_string(&registry, &["branch", "name"])? != result.branch
        || PathBuf::from(required_string(&registry, &["open", "path"])?) != target
        || required_string(&registry, &["open", "open_instance_id"])? != open_instance_id
        || required_string(&registry, &["open", "opened_from_commit"])? != result.base_commit
        || active_path != expected_active_path
        || required_string(&registry, &["import", "mode"])? != IMPORT_MODE
        || import_subject_usize(&registry, &["import", "initial_files"])? != result.initial_files
        || !is_open_branch_state(&registry_state)
    {
        return Err(CliError::InvalidRepository(format!(
            "import replay opened registry does not match its recorded subject: {}",
            registry_path.display()
        )));
    }
    codefire_store::validate_sealed_commit(&objects, &current_base)?;
    if current_base != branch_head {
        return Err(CliError::InvalidRepository(format!(
            "import replay branch head and opened base disagree: {branch_head} != {current_base}"
        )));
    }

    let active_state_path = active_path.join("state.json");
    let active_state = subject.read_required_json(
        &PathBuf::from(".codefire")
            .join("active")
            .join(&open_instance_id)
            .join("state.json"),
    )?;
    let active_state_name = required_string(&active_state, &["state"])?;
    let branch_state = required_string(&branch, &["state"])?;
    if active_state_name != registry_state
        || branch_state != registry_state
        || !is_open_branch_state(&active_state_name)
    {
        return Err(CliError::InvalidRepository(format!(
            "import replay active, registry, and branch state disagree: {}",
            active_state_path.display()
        )));
    }
    Ok(())
}

fn same_import_result_subject(left: &ImportResult, right: &ImportResult) -> bool {
    left.repo_root == right.repo_root
        && left.open_dir == right.open_dir
        && left.branch == right.branch
        && left.base_commit == right.base_commit
        && left.initial_files == right.initial_files
        && left.initial_state == right.initial_state
}

pub(crate) fn import_result_data_json(result: &ImportResult) -> Value {
    json!({
        "type": "codefire_import_result",
        "version": 1,
        "repo_root": &result.repo_root,
        "open_dir": &result.open_dir,
        "branch": &result.branch,
        "base_commit": empty_string_as_null(&result.base_commit),
        "initial_files": result.initial_files,
        "initial_state": &result.initial_state,
        "plan": &result.plan,
    })
}

fn validate_import_target(target: &Path) -> Result<(), CliError> {
    if !target.exists() {
        return Err(CliError::Usage(format!(
            "import target does not exist: {}",
            target.display()
        )));
    }
    if !target.is_dir() {
        return Err(CliError::Usage(format!(
            "import target must be a directory: {}",
            target.display()
        )));
    }
    if target.join(".codefire").exists() {
        return Err(CliError::Usage(format!(
            "import target is already a CodeFire repository: {}",
            target.display()
        )));
    }
    if target.join(".codefire-open").exists() {
        return Err(CliError::Usage(format!(
            "import target is already a CodeFire open directory: {}",
            target.display()
        )));
    }
    Ok(())
}

fn import_open_instance_id(branch: &str, target: &Path, opened_at: &str) -> String {
    format!(
        "import_{:012x}",
        stable_hash_48(format!("{branch}:{}:{opened_at}", target.display()).as_bytes())
    )
}

fn import_operation_plan(
    options: &ImportOptions,
    repo_root: &Path,
    target: &Path,
    registry_path: &Path,
    active_path: &Path,
    initial_files: usize,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "import",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "target": target,
        "branch": &options.branch,
        "initial_files": initial_files,
        "operations": [
            {"kind": "initialize_repository", "path": repo_root},
            {"kind": "write_open_marker", "path": target.join(".codefire-open")},
            {"kind": "write_open_registry", "path": registry_path},
            {"kind": "create_active_state", "path": active_path},
            {"kind": "scan_imported_tree", "path": target},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "open-burning_or_scan_result"},
        ],
        "next_actions": [
            {"kind": "scan", "command": "codefire scan --json", "target": {"path": target}},
            {"kind": "verify", "command": "codefire verify --json", "target": {"path": target}},
            {"kind": "commit", "command": "codefire commit -m \"Initial import\"", "target": {"path": target}},
        ],
    })
}

fn empty_string_as_null(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

fn import_idempotency_payload(options: &ImportOptions, target: &Path) -> Result<Value, CliError> {
    let normalized_target = normalized_import_target(target)?;
    Ok(json!({
        "type": "codefire_import_idempotency_payload",
        "version": IMPORT_IDEMPOTENCY_PAYLOAD_VERSION,
        "operation": IMPORT_OPERATION,
        "scope": {
            "kind": "import_target",
            "target": &normalized_target,
        },
        "target": &normalized_target,
        "branch": &options.branch,
        "semantic_options": {
            "mode": IMPORT_MODE,
        },
    }))
}

fn normalized_import_target(target: &Path) -> Result<PathBuf, CliError> {
    if target.exists() {
        Ok(target.canonicalize()?)
    } else {
        Ok(target.to_path_buf())
    }
}

fn import_idempotency_record(
    key: &str,
    payload: &Value,
    result: &ImportResult,
) -> Result<Value, CliError> {
    require_idempotency_key(key)?;
    let scope = import_payload_value(payload, &["scope"])?;
    let payload_version = import_payload_u64(payload, &["version"])?;
    Ok(json!({
        "type": IMPORT_IDEMPOTENCY_RECORD_TYPE,
        "version": IMPORT_IDEMPOTENCY_RECORD_VERSION,
        "operation": IMPORT_OPERATION,
        "scope": scope,
        "key": key,
        "payload_version": payload_version,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload_mode": "hash_only",
        "result": {
            "repo_root": &result.repo_root,
            "open_dir": &result.open_dir,
            "branch": &result.branch,
            "base_commit": &result.base_commit,
            "initial_files": result.initial_files,
            "initial_state": &result.initial_state,
        },
        "created_at": now_iso_utc(),
    }))
}

fn save_import_idempotency_marker_subject(
    subject: &ImportSubject,
    key: &str,
    payload: &Value,
    result: &ImportResult,
) -> Result<(), CliError> {
    let relative = PathBuf::from(".codefire")
        .join("idempotency")
        .join(IMPORT_OPERATION)
        .join(format!("{}.json", ref_file_name(key)));
    subject.write_control_json_atomic(&relative, &import_idempotency_record(key, payload, result)?)
}

#[cfg(test)]
fn save_import_idempotency_marker(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &ImportResult,
) -> Result<(), CliError> {
    let path = idempotency_record_path(repo_root, IMPORT_OPERATION, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_atomic(&path, &import_idempotency_record(key, payload, result)?)
}

#[cfg(test)]
fn load_import_idempotency_marker(
    target: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<ImportResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(target, IMPORT_OPERATION, key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    parse_import_idempotency_result(&record, &path, key, payload).map(Some)
}

fn load_import_idempotency_marker_subject(
    subject: &ImportSubject,
    key: &str,
    payload: &Value,
) -> Result<Option<ImportResult>, CliError> {
    require_idempotency_key(key)?;
    let relative = PathBuf::from(".codefire")
        .join("idempotency")
        .join(IMPORT_OPERATION)
        .join(format!("{}.json", ref_file_name(key)));
    let Some(record) = subject.read_control_json(&relative)? else {
        return Ok(None);
    };
    let path = subject.target.join(relative);
    parse_import_idempotency_result(&record, &path, key, payload).map(Some)
}

fn parse_import_idempotency_result(
    record: &Value,
    path: &Path,
    key: &str,
    payload: &Value,
) -> Result<ImportResult, CliError> {
    verify_import_idempotency_marker(record, path, key, payload)?;
    let repo_root = PathBuf::from(import_marker_string(
        record,
        path,
        &["result", "repo_root"],
    )?);
    let open_dir = PathBuf::from(import_marker_string(record, path, &["result", "open_dir"])?);
    let expected_target = PathBuf::from(import_payload_string(payload, &["target"])?);
    if repo_root != expected_target || open_dir != expected_target {
        return Err(invalid_import_marker(
            path,
            "result repository paths do not match the bound target",
        ));
    }
    let branch = import_marker_string(record, path, &["result", "branch"])?;
    let expected_branch = import_payload_string(payload, &["branch"])?;
    if branch != expected_branch {
        return Err(invalid_import_marker(
            path,
            "result.branch does not match the bound request",
        ));
    }
    let base_commit = import_marker_string(record, path, &["result", "base_commit"])?;
    if base_commit.is_empty() {
        return Err(invalid_import_marker(
            path,
            "result.base_commit must not be empty",
        ));
    }
    let initial_files = import_marker_usize(record, path, &["result", "initial_files"])?;
    let initial_state = import_marker_string(record, path, &["result", "initial_state"])?;
    if initial_state.is_empty() {
        return Err(invalid_import_marker(
            path,
            "result.initial_state must not be empty",
        ));
    }
    Ok(ImportResult {
        repo_root: repo_root.clone(),
        open_dir: open_dir.clone(),
        branch: branch.clone(),
        base_commit,
        initial_files,
        initial_state,
        plan: json!({
            "type": "codefire_operation_plan",
            "version": 1,
            "command": "import",
            "replayed": true,
            "repo": repo_root,
            "target": open_dir,
            "branch": branch,
        }),
    })
}

fn verify_import_idempotency_marker(
    record: &Value,
    path: &Path,
    key: &str,
    payload: &Value,
) -> Result<(), CliError> {
    let version = import_marker_u64(record, path, &["version"])?;
    if version == 1 {
        // Version 1 markers stored only a result. They cannot prove that a replay
        // request has the same branch or semantic options, so fail closed.
        return Err(CliError::IdempotencyConflict(format!(
            "legacy import idempotency marker is not payload-bound for key {key}; use a new key"
        )));
    }
    if version != IMPORT_IDEMPOTENCY_RECORD_VERSION {
        return Err(invalid_import_marker(
            path,
            &format!("unsupported version: {version}"),
        ));
    }
    if import_marker_string(record, path, &["type"])? != IMPORT_IDEMPOTENCY_RECORD_TYPE {
        return Err(invalid_import_marker(path, "unexpected record type"));
    }
    if import_marker_string(record, path, &["payload_mode"])? != "hash_only" {
        return Err(invalid_import_marker(
            path,
            "payload_mode must be hash_only",
        ));
    }
    let payload_version = import_payload_u64(payload, &["version"])?;
    if import_marker_u64(record, path, &["payload_version"])? != payload_version {
        return Err(import_idempotency_conflict(key));
    }
    if import_marker_string(record, path, &["operation"])? != IMPORT_OPERATION {
        return Err(import_idempotency_conflict(key));
    }
    if import_marker_value(record, path, &["scope"])? != import_payload_value(payload, &["scope"])?
    {
        return Err(import_idempotency_conflict(key));
    }
    import_marker_string(record, path, &["key"])?;
    import_marker_string(record, path, &["payload_hash"])?;
    verify_idempotency_record(record, path, IMPORT_OPERATION, key, payload)
}

fn import_idempotency_conflict(key: &str) -> CliError {
    CliError::IdempotencyConflict(format!(
        "idempotency key conflict for {IMPORT_OPERATION}: {key}"
    ))
}

fn invalid_import_marker(path: &Path, detail: &str) -> CliError {
    CliError::InvalidRepository(format!(
        "invalid import idempotency marker {}: {detail}",
        path.display()
    ))
}

fn import_marker_value<'a>(
    record: &'a Value,
    marker_path: &Path,
    field_path: &[&str],
) -> Result<&'a Value, CliError> {
    let mut current = record;
    for field in field_path {
        current = current.get(*field).ok_or_else(|| {
            invalid_import_marker(marker_path, &format!("missing {}", field_path.join(".")))
        })?;
    }
    Ok(current)
}

fn import_marker_string(
    record: &Value,
    marker_path: &Path,
    field_path: &[&str],
) -> Result<String, CliError> {
    import_marker_value(record, marker_path, field_path)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            invalid_import_marker(
                marker_path,
                &format!("{} must be a string", field_path.join(".")),
            )
        })
}

fn import_marker_u64(
    record: &Value,
    marker_path: &Path,
    field_path: &[&str],
) -> Result<u64, CliError> {
    import_marker_value(record, marker_path, field_path)?
        .as_u64()
        .ok_or_else(|| {
            invalid_import_marker(
                marker_path,
                &format!("{} must be an unsigned integer", field_path.join(".")),
            )
        })
}

fn import_marker_usize(
    record: &Value,
    marker_path: &Path,
    field_path: &[&str],
) -> Result<usize, CliError> {
    let value = import_marker_u64(record, marker_path, field_path)?;
    usize::try_from(value).map_err(|_| {
        invalid_import_marker(
            marker_path,
            &format!("{} exceeds platform limits", field_path.join(".")),
        )
    })
}

fn import_payload_value<'a>(
    payload: &'a Value,
    field_path: &[&str],
) -> Result<&'a Value, CliError> {
    let mut current = payload;
    for field in field_path {
        current = current.get(*field).ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "canonical import idempotency payload missing {}",
                field_path.join(".")
            ))
        })?;
    }
    Ok(current)
}

fn import_payload_string(payload: &Value, field_path: &[&str]) -> Result<String, CliError> {
    import_payload_value(payload, field_path)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "canonical import idempotency payload {} must be a string",
                field_path.join(".")
            ))
        })
}

fn import_payload_u64(payload: &Value, field_path: &[&str]) -> Result<u64, CliError> {
    import_payload_value(payload, field_path)?
        .as_u64()
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "canonical import idempotency payload {} must be an unsigned integer",
                field_path.join(".")
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use tempfile::tempdir;

    fn import_options(path: &Path, branch: &str) -> ImportOptions {
        ImportOptions {
            path: path.to_path_buf(),
            branch: branch.to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: Some("import-once".to_string()),
        }
    }

    fn import_result(path: &Path, branch: &str) -> ImportResult {
        ImportResult {
            repo_root: path.to_path_buf(),
            open_dir: path.to_path_buf(),
            branch: branch.to_string(),
            base_commit: "CF-COMMIT-test".to_string(),
            initial_files: 3,
            initial_state: "open-burning".to_string(),
            plan: json!({"command": "import"}),
        }
    }

    fn wait_import_options(path: &Path, branch: &str, key: &str) -> ImportOptions {
        ImportOptions {
            path: path.to_path_buf(),
            branch: branch.to_string(),
            dry_run: false,
            json_output: false,
            lock: LockOptions {
                wait: true,
                timeout_ms: Some(5_000),
            },
            idempotency_key: Some(key.to_string()),
        }
    }

    fn import_subject(target: &Path) -> ImportSubject {
        ImportSubject {
            target: target.to_path_buf(),
            root: open_absolute_dir_nofollow(target).unwrap(),
            transaction_id: "import_tx_test".to_string(),
        }
    }

    #[cfg(unix)]
    fn make_fifo(path: &Path) {
        let status = std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn import_file_limits_preserve_blob_record_headroom() {
        let encoded_size = IMPORT_USER_FILE_BYTE_LIMIT.div_ceil(3) * 4;

        assert!(
            encoded_size + IMPORT_BLOB_RECORD_OVERHEAD_RESERVE_BYTES
                <= codefire_store::MAX_OBJECT_RECORD_BYTES
        );
        assert_eq!(IMPORT_CONTROL_FILE_BYTE_LIMIT, 1024 * 1024);
        assert_eq!(
            IMPORT_REPOSITORY_FILE_BYTE_LIMIT,
            codefire_store::MAX_OBJECT_RECORD_BYTES
        );
    }

    #[test]
    fn repository_state_and_control_json_use_distinct_symmetric_limits() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let subject = import_subject(&target);
        let value = json!({"payload": "x".repeat(IMPORT_CONTROL_FILE_BYTE_LIMIT as usize)});
        let state_path = Path::new("large-state.json");

        subject.write_json_atomic(state_path, &value).unwrap();
        assert_eq!(subject.read_json(state_path).unwrap(), Some(value.clone()));
        let read_error = subject.read_control_json(state_path).unwrap_err();
        assert!(matches!(
            read_error,
            CliError::Io(error) if error.kind() == std::io::ErrorKind::InvalidData
        ));

        let control_path = Path::new("oversize-control.json");
        let write_error = subject
            .write_control_json_atomic(control_path, &value)
            .unwrap_err();
        assert!(matches!(write_error, CliError::InvalidRepository(_)));
        assert!(!target.join(control_path).exists());
    }

    #[test]
    fn first_import_keeps_lock_inode_empty_and_publishes_owner_sidecar() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), b"import me\n").unwrap();

        import_existing_project(&import_options(&target, "main")).unwrap();

        let lock_path = target.join(".codefire/locks/repo.lock");
        let owner_path = repo_lock_owner_path(&lock_path);
        assert_eq!(fs::read(&lock_path).unwrap(), b"");
        let owner = read_json(&owner_path).unwrap();
        assert_eq!(owner["version"], 1);
        assert_eq!(owner["pid"], std::process::id());
        assert!(owner["created_at"]
            .as_str()
            .is_some_and(|value| !value.is_empty()));
    }

    #[cfg(unix)]
    #[test]
    fn import_repo_lock_rejects_outside_hardlink_without_mutation() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        let locks = target.join(".codefire/locks");
        fs::create_dir_all(&locks).unwrap();
        let outside = temp.path().join("outside.txt");
        fs::write(&outside, b"outside sentinel\n").unwrap();
        fs::hard_link(&outside, locks.join("repo.lock")).unwrap();
        let subject = import_subject(&target);

        let error = ImportRepoLock::acquire(&subject, &LockOptions::default()).unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("exactly one filesystem link"));
        assert_eq!(fs::read(&outside).unwrap(), b"outside sentinel\n");
        assert!(!locks.join("repo.lock.owner.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn import_repo_lock_rechecks_link_count_at_use_boundary() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let subject = import_subject(&target);
        let lock = ImportRepoLock::acquire(&subject, &LockOptions::default()).unwrap();
        let lock_path = target.join(".codefire/locks/repo.lock");
        fs::hard_link(&lock_path, temp.path().join("outside-lock-link")).unwrap();

        let error = lock.ensure_subject_identity(&subject).unwrap_err();

        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("exactly one filesystem link"));
        assert_eq!(fs::read(lock_path).unwrap(), b"");
    }

    #[cfg(unix)]
    #[test]
    fn owner_sidecar_replaces_hardlink_without_touching_outside_file() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        let locks = target.join(".codefire/locks");
        fs::create_dir_all(&locks).unwrap();
        let outside = temp.path().join("owner-outside.json");
        fs::write(&outside, b"outside owner sentinel\n").unwrap();
        fs::hard_link(&outside, locks.join("repo.lock.owner.json")).unwrap();
        let subject = import_subject(&target);

        let lock = ImportRepoLock::acquire(&subject, &LockOptions::default()).unwrap();

        assert_eq!(fs::read(&outside).unwrap(), b"outside owner sentinel\n");
        let owner = subject
            .read_required_json(Path::new(".codefire/locks/repo.lock.owner.json"))
            .unwrap();
        assert_eq!(owner["version"], 1);
        assert_eq!(owner["pid"], std::process::id());
        drop(lock);
    }

    #[cfg(unix)]
    #[test]
    fn import_contention_diagnostic_does_not_block_on_owner_fifo() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let subject = import_subject(&target);
        let first = ImportRepoLock::acquire(&subject, &LockOptions::default()).unwrap();
        let owner_path = target.join(".codefire/locks/repo.lock.owner.json");
        fs::remove_file(&owner_path).unwrap();
        make_fifo(&owner_path);

        let started = Instant::now();
        let error = ImportRepoLock::acquire(&subject, &LockOptions::default()).unwrap_err();

        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(matches!(error, CliError::RepositoryLockContention { .. }));
        drop(first);
    }

    #[cfg(unix)]
    #[test]
    fn control_reader_rejects_fifo_directory_and_device_without_blocking() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let root = open_absolute_dir_nofollow(&target).unwrap();
        let fifo_name = "control.fifo";
        make_fifo(&target.join(fifo_name));
        fs::create_dir(target.join("control-dir")).unwrap();

        for name in [fifo_name, "control-dir"] {
            let started = Instant::now();
            let error = read_capability_file(&root, name).unwrap_err();
            assert!(started.elapsed() < Duration::from_secs(1));
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        }
        if Path::new("/dev/null").exists() {
            let dev = open_absolute_dir_nofollow(Path::new("/dev")).unwrap();
            let error = read_capability_file(&dev, "null").unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn control_reader_rejects_growth_after_metadata_check() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let path = target.join("control.json");
        fs::write(&path, b"{}\n").unwrap();
        let initial_size = fs::metadata(&path).unwrap().len();
        let root = open_absolute_dir_nofollow(&target).unwrap();

        let error =
            read_capability_file_with_limit_and_hook(&root, "control.json", initial_size, || {
                fs::OpenOptions::new()
                    .write(true)
                    .open(&path)?
                    .set_len(initial_size + 1)
            })
            .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("grew beyond"));
    }

    #[cfg(unix)]
    #[test]
    fn import_rejects_prelock_fifo_without_waiting_for_timeout() {
        for name in [IMPORT_BOOTSTRAP_LOCK_FILE, IMPORT_TRANSACTION_FILE] {
            let temp = tempdir().unwrap();
            let target = temp.path().join("project");
            fs::create_dir_all(&target).unwrap();
            make_fifo(&target.join(name));
            let mut options = wait_import_options(&target, "main", "prelock-fifo");
            options.lock.timeout_ms = Some(10_000);

            let started = Instant::now();
            let error = import_existing_project(&options).unwrap_err();

            assert!(started.elapsed() < Duration::from_secs(1));
            assert!(matches!(error, CliError::InvalidRepository(_)));
            assert!(!target.join(".codefire").exists());
        }
    }

    #[test]
    fn import_preflights_control_and_user_size_caps_before_mutation() {
        for (name, limit) in [
            (IMPORT_TRANSACTION_FILE, IMPORT_CONTROL_FILE_BYTE_LIMIT),
            ("large-user.bin", IMPORT_USER_FILE_BYTE_LIMIT),
        ] {
            let temp = tempdir().unwrap();
            let target = temp.path().join("project");
            fs::create_dir_all(&target).unwrap();
            let path = target.join(name);
            let file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            file.set_len(limit + 1).unwrap();
            drop(file);

            let error = import_existing_project(&import_options(&target, "main")).unwrap_err();

            assert!(matches!(error, CliError::InvalidRepository(_)));
            assert!(error.to_string().contains(&limit.to_string()));
            assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
            assert!(!target.join(".codefire").exists());
        }
    }

    #[test]
    fn import_idempotency_exact_replay_is_payload_bound_and_hash_only() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let result = import_result(&target, "main");

        save_import_idempotency_marker(&target, "import-once", &payload, &result).unwrap();
        let replay = load_import_idempotency_marker(&target, "import-once", &payload)
            .unwrap()
            .unwrap();

        assert_eq!(replay.repo_root, result.repo_root);
        assert_eq!(replay.open_dir, result.open_dir);
        assert_eq!(replay.branch, result.branch);
        assert_eq!(replay.base_commit, result.base_commit);
        assert_eq!(replay.initial_files, result.initial_files);
        assert_eq!(replay.initial_state, result.initial_state);
        assert_eq!(replay.plan["replayed"], true);

        let path = idempotency_record_path(&target, IMPORT_OPERATION, "import-once");
        let record = read_json(&path).unwrap();
        assert_eq!(record["type"], IMPORT_IDEMPOTENCY_RECORD_TYPE);
        assert_eq!(record["version"], IMPORT_IDEMPOTENCY_RECORD_VERSION);
        assert_eq!(record["operation"], IMPORT_OPERATION);
        assert_eq!(record["scope"], payload["scope"]);
        assert_eq!(
            record["payload_version"],
            IMPORT_IDEMPOTENCY_PAYLOAD_VERSION
        );
        assert_eq!(
            record["payload_hash"],
            idempotency_payload_hash(&payload).unwrap()
        );
        assert_eq!(record["payload_mode"], "hash_only");
        assert!(record.get("payload").is_none());
    }

    #[test]
    fn import_idempotency_rejects_branch_mismatch() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let original =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        save_import_idempotency_marker(
            &target,
            "import-once",
            &original,
            &import_result(&target, "main"),
        )
        .unwrap();
        let changed =
            import_idempotency_payload(&import_options(&target, "feature"), &target).unwrap();

        let error = load_import_idempotency_marker(&target, "import-once", &changed).unwrap_err();
        assert!(matches!(&error, CliError::IdempotencyConflict(_)));
    }

    #[test]
    fn import_idempotency_rejects_scope_mismatch() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        let other_target = temp.path().join("other-project");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&other_target).unwrap();
        let original =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        save_import_idempotency_marker(
            &target,
            "import-once",
            &original,
            &import_result(&target, "main"),
        )
        .unwrap();
        let changed =
            import_idempotency_payload(&import_options(&other_target, "main"), &other_target)
                .unwrap();

        let error = load_import_idempotency_marker(&target, "import-once", &changed).unwrap_err();
        assert!(matches!(&error, CliError::IdempotencyConflict(_)));
    }

    #[test]
    fn import_idempotency_rejects_semantic_options_mismatch() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let original =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        save_import_idempotency_marker(
            &target,
            "import-once",
            &original,
            &import_result(&target, "main"),
        )
        .unwrap();
        let mut changed = original.clone();
        changed["semantic_options"]["mode"] = Value::String("copy_project".to_string());

        let error = load_import_idempotency_marker(&target, "import-once", &changed).unwrap_err();
        assert!(matches!(&error, CliError::IdempotencyConflict(_)));
    }

    #[test]
    fn import_idempotency_rejects_malformed_marker() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let payload =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        let path = idempotency_record_path(&target, IMPORT_OPERATION, "import-once");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_json_atomic(
            &path,
            &json!({
                "type": IMPORT_IDEMPOTENCY_RECORD_TYPE,
                "version": IMPORT_IDEMPOTENCY_RECORD_VERSION,
                "operation": IMPORT_OPERATION,
                "scope": payload["scope"],
                "key": "import-once",
                "payload_version": IMPORT_IDEMPOTENCY_PAYLOAD_VERSION,
                "payload_mode": "hash_only",
                "result": {},
            }),
        )
        .unwrap();

        let error = load_import_idempotency_marker(&target, "import-once", &payload).unwrap_err();
        assert!(matches!(&error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("missing payload_hash"));
    }

    #[test]
    fn import_idempotency_rejects_result_for_a_different_target() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let payload =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        save_import_idempotency_marker(
            &target,
            "import-once",
            &payload,
            &import_result(&target, "main"),
        )
        .unwrap();
        let path = idempotency_record_path(&target, IMPORT_OPERATION, "import-once");
        let mut record = read_json(&path).unwrap();
        record["result"]["open_dir"] = Value::String("/different/target".to_string());
        write_json_atomic(&path, &record).unwrap();

        let error = load_import_idempotency_marker(&target, "import-once", &payload).unwrap_err();
        assert!(matches!(&error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("do not match the bound target"));
    }

    #[test]
    fn import_idempotency_rejects_legacy_result_only_marker() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let payload =
            import_idempotency_payload(&import_options(&target, "main"), &target).unwrap();
        let path = idempotency_record_path(&target, IMPORT_OPERATION, "import-once");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_json_atomic(
            &path,
            &json!({
                "version": 1,
                "operation": IMPORT_OPERATION,
                "key": "import-once",
                "repo_root": &target,
                "open_dir": &target,
                "branch": "main",
                "base_commit": "CF-COMMIT-legacy",
                "initial_files": 1,
                "initial_state": "open-burning",
            }),
        )
        .unwrap();

        let error = load_import_idempotency_marker(&target, "import-once", &payload).unwrap_err();
        assert!(matches!(&error, CliError::IdempotencyConflict(_)));
        assert!(error.to_string().contains("not payload-bound"));
    }

    #[test]
    fn concurrent_exact_import_with_wait_replays_the_winner() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let barrier = Arc::new(Barrier::new(3));

        let workers = (0..2)
            .map(|_| {
                let target = target.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    import_existing_project(&wait_import_options(
                        &target,
                        "main",
                        "concurrent-import",
                    ))
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();

        let results = workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results[0].base_commit, results[1].base_commit);
        assert_eq!(results[0].branch, results[1].branch);
        assert_eq!(
            results
                .iter()
                .filter(|result| result.plan["replayed"] == true)
                .count(),
            1
        );
        assert!(target.join(IMPORT_BOOTSTRAP_LOCK_FILE).is_file());
        assert!(target.join(IMPORT_TRANSACTION_FILE).is_file());
    }

    #[test]
    fn stalled_bootstrap_staging_contender_converges_to_the_winner() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let reached = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        inject_bootstrap_staging_pause(&target, Arc::clone(&reached), Arc::clone(&release));

        let paused_target = target.clone();
        let paused = std::thread::spawn(move || {
            import_existing_project(&wait_import_options(
                &paused_target,
                "main",
                "stalled-import",
            ))
        });
        reached.wait();
        std::thread::sleep(Duration::from_millis(150));

        let winner =
            import_existing_project(&wait_import_options(&target, "main", "stalled-import"))
                .unwrap();
        release.wait();
        let resumed = paused.join().unwrap().unwrap();

        assert_eq!(winner.base_commit, resumed.base_commit);
        assert_eq!(winner.initial_files, 1);
        assert_eq!(resumed.initial_files, 1);
        assert_ne!(winner.plan["replayed"], true);
        assert_eq!(resumed.plan["replayed"], true);
        assert!(fs::read_dir(&target).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(IMPORT_BOOTSTRAP_STAGING_SUFFIX)
        }));
    }

    #[test]
    fn dry_run_rejects_an_empty_key_without_writes() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let mut options = import_options(&target, "main");
        options.dry_run = true;
        options.idempotency_key = Some(String::new());

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::Usage(_)));
        assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        assert!(!target.join(".codefire").exists());
    }

    #[test]
    fn preexisting_reserved_lock_is_not_silently_adopted() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let lock_path = target.join(IMPORT_BOOTSTRAP_LOCK_FILE);
        fs::write(&lock_path, b"user-owned content\n").unwrap();

        let error = import_existing_project(&import_options(&target, "main")).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(fs::read(&lock_path).unwrap(), b"user-owned content\n");
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        assert!(!target.join(".codefire").exists());
    }

    #[cfg(unix)]
    #[test]
    fn bootstrap_rejects_a_target_replaced_by_a_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        let moved = temp.path().join("moved-project");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        fs::rename(&target, moved).unwrap();
        symlink(&outside, &target).unwrap();

        let error =
            ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
                .unwrap_err();
        assert!(matches!(error, CliError::Io(_)));
        assert!(!outside.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
        assert!(!outside.join(IMPORT_TRANSACTION_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn target_swap_during_init_never_writes_to_the_replacement() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        let moved = temp.path().join("moved-project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        inject_import_target_swap(&target, &moved);

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::Io(_)));
        assert_eq!(
            fs::read(target.join("replacement.txt")).unwrap(),
            b"replacement must stay untouched\n"
        );
        assert!(!target.join(".codefire").exists());
        assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        assert!(moved.join(".codefire/repo.json").is_file());
        assert!(moved.join(IMPORT_BOOTSTRAP_LOCK_FILE).is_file());
        assert!(moved.join(IMPORT_TRANSACTION_FILE).is_file());
    }

    #[test]
    fn published_lock_seed_resumes_before_transaction_creation() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();

        let lock = ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
            .unwrap();
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        drop(lock);

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert_eq!(
            load_import_transaction(&target).unwrap().unwrap().phase,
            ImportPhase::Completed
        );
    }

    #[test]
    fn complete_prepublication_temp_is_recovered_without_becoming_content() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let seed = ImportTransaction::new(&target, "main", 1, &payload).unwrap();
        let temp_name = format!(
            "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}{IMPORT_BOOTSTRAP_TEMP_SUFFIX}",
            seed.transaction_id
        );
        let mut header = serde_json::to_vec_pretty(&bootstrap_lock_header(&seed)).unwrap();
        header.push(b'\n');
        fs::write(target.join(&temp_name), header).unwrap();

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert!(!target.join(temp_name).exists());
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        assert!(scan
            .non_atom_changed_files
            .iter()
            .all(|change| !change.path.starts_with(IMPORT_BOOTSTRAP_TEMP_PREFIX)));
    }

    #[test]
    fn completed_bootstrap_staging_hard_link_is_recovered_and_removed() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let seed = ImportTransaction::new(&target, "main", 1, &payload).unwrap();
        let staging_name = format!(
            "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}-crash{IMPORT_BOOTSTRAP_STAGING_SUFFIX}",
            seed.transaction_id
        );
        let temp_name = format!(
            "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}{IMPORT_BOOTSTRAP_TEMP_SUFFIX}",
            seed.transaction_id
        );
        let mut header = serde_json::to_vec_pretty(&bootstrap_lock_header(&seed)).unwrap();
        header.push(b'\n');
        fs::write(target.join(&staging_name), &header).unwrap();
        fs::hard_link(target.join(&staging_name), target.join(&temp_name)).unwrap();

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert!(!target.join(staging_name).exists());
        assert!(!target.join(temp_name).exists());
    }

    #[test]
    fn delayed_loser_bootstrap_temp_never_becomes_imported_content() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let contender = ImportTransaction::new(&target, "main", 1, &payload).unwrap();
        let temp_name = format!(
            "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}{IMPORT_BOOTSTRAP_TEMP_SUFFIX}",
            contender.transaction_id
        );
        let mut contents = serde_json::to_vec_pretty(&bootstrap_lock_header(&contender)).unwrap();
        contents.push(b'\n');
        inject_delayed_bootstrap_temp(&target, temp_name.clone(), contents.clone());

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert_eq!(fs::read(target.join(&temp_name)).unwrap(), contents);
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        assert!(scan
            .non_atom_changed_files
            .iter()
            .all(|change| change.path != temp_name));

        let replay = import_existing_project(&options).unwrap();
        assert_eq!(replay.base_commit, result.base_commit);
        assert_eq!(fs::read(target.join(temp_name)).unwrap(), contents);
    }

    #[test]
    fn delayed_bootstrap_staging_file_never_becomes_imported_content() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let temp_name =
            format!("{IMPORT_BOOTSTRAP_TEMP_PREFIX}delayed-loser{IMPORT_BOOTSTRAP_STAGING_SUFFIX}");
        let contents = b"not yet published\n".to_vec();
        inject_delayed_bootstrap_temp(&target, temp_name.clone(), contents.clone());

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert_eq!(fs::read(target.join(&temp_name)).unwrap(), contents);
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        assert!(scan
            .non_atom_changed_files
            .iter()
            .all(|change| change.path != temp_name));
        import_existing_project(&options).unwrap();
        assert_eq!(fs::read(target.join(temp_name)).unwrap(), contents);
    }

    #[test]
    fn delayed_transaction_write_temp_never_becomes_imported_content() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let lock = ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
            .unwrap();
        let temp_name = format!(
            "{IMPORT_WRITE_TEMP_PREFIX}{}-delayed{IMPORT_WRITE_TEMP_SUFFIX}",
            lock.seed.transaction_id
        );
        drop(lock);
        inject_delayed_bootstrap_temp(
            &target,
            temp_name.clone(),
            b"interrupted transaction write\n".to_vec(),
        );

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        assert!(scan
            .non_atom_changed_files
            .iter()
            .all(|change| change.path != temp_name));
        assert!(!target.join(temp_name).exists());
    }

    #[test]
    fn partial_prepublication_temp_fails_closed_without_deletion() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let temp_name =
            format!("{IMPORT_BOOTSTRAP_TEMP_PREFIX}foreign{IMPORT_BOOTSTRAP_TEMP_SUFFIX}");
        fs::write(target.join(&temp_name), b"{\"type\":").unwrap();

        let error = import_existing_project(&import_options(&target, "main")).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(fs::read(target.join(&temp_name)).unwrap(), b"{\"type\":");
        assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        assert!(!target.join(".codefire").exists());
    }

    #[test]
    fn unpublished_bootstrap_staging_file_is_preserved_and_excluded() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let temp_name =
            format!("{IMPORT_BOOTSTRAP_TEMP_PREFIX}foreign{IMPORT_BOOTSTRAP_STAGING_SUFFIX}");
        fs::write(target.join(&temp_name), b"partial staging\n").unwrap();

        let result = import_existing_project(&import_options(&target, "main")).unwrap();
        assert_eq!(result.initial_files, 1);
        assert_eq!(
            fs::read(target.join(&temp_name)).unwrap(),
            b"partial staging\n"
        );
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        assert!(scan
            .non_atom_changed_files
            .iter()
            .all(|change| change.path != temp_name));
    }

    #[test]
    fn unlinked_same_transaction_staging_file_is_preserved_and_ignored() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let lock = ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
            .unwrap();
        let staging_name = format!(
            "{IMPORT_BOOTSTRAP_TEMP_PREFIX}{}-partial{IMPORT_BOOTSTRAP_STAGING_SUFFIX}",
            lock.seed.transaction_id
        );
        drop(lock);
        fs::write(target.join(&staging_name), b"partial same transaction\n").unwrap();

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert_eq!(
            fs::read(target.join(&staging_name)).unwrap(),
            b"partial same transaction\n"
        );
    }

    #[test]
    fn foreign_atomic_write_temp_is_preserved_and_fails_closed() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let lock = ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
            .unwrap();
        drop(lock);
        let temp_name =
            format!("{IMPORT_WRITE_TEMP_PREFIX}foreign-deadbeef{IMPORT_WRITE_TEMP_SUFFIX}");
        fs::write(target.join(&temp_name), b"foreign atomic write\n").unwrap();

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert_eq!(
            fs::read(target.join(&temp_name)).unwrap(),
            b"foreign atomic write\n"
        );
        assert!(!target.join(IMPORT_TRANSACTION_FILE).exists());
        assert!(!target.join(".codefire").exists());
    }

    #[test]
    fn transaction_bound_atomic_write_temp_is_recovered() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        let lock = ImportBootstrapLock::acquire(&target, "main", &payload, &LockOptions::default())
            .unwrap();
        let temp_name = format!(
            "{IMPORT_WRITE_TEMP_PREFIX}{}-deadbeef{IMPORT_WRITE_TEMP_SUFFIX}",
            lock.seed.transaction_id
        );
        drop(lock);
        fs::write(target.join(&temp_name), b"interrupted owned write\n").unwrap();

        let result = import_existing_project(&options).unwrap();
        assert_eq!(result.initial_files, 1);
        assert!(!target.join(temp_name).exists());
    }

    #[test]
    fn empty_or_truncated_transaction_fails_closed() {
        for contents in [b"".as_slice(), b"{\"type\":".as_slice()] {
            let temp = tempdir().unwrap();
            let target = temp.path().join("project");
            fs::create_dir_all(&target).unwrap();
            fs::write(target.join(IMPORT_TRANSACTION_FILE), contents).unwrap();

            let error = import_existing_project(&import_options(&target, "main")).unwrap_err();
            assert!(matches!(error, CliError::InvalidRepository(_)));
            assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
            assert!(!target.join(".codefire").exists());
        }
    }

    #[test]
    fn crash_after_preinit_lock_layout_retries_under_the_same_transaction() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        inject_import_init_layout_failure(&target);

        let error = import_existing_project(&options).unwrap_err();
        assert!(error
            .to_string()
            .contains("injected import failure after layout creation"));
        assert!(target.join(".codefire/locks/repo.lock").is_file());
        assert!(!target.join(".codefire/repo.json").exists());
        assert_eq!(
            load_import_transaction(&target).unwrap().unwrap().phase,
            ImportPhase::Planned
        );

        let recovered = import_existing_project(&options).unwrap();
        assert_eq!(recovered.initial_files, 1);
        assert_eq!(
            load_import_transaction(&target).unwrap().unwrap().phase,
            ImportPhase::Completed
        );
    }

    #[test]
    fn crash_before_or_after_initial_branch_publication_is_retryable() {
        for point in [
            ImportInitPublicationPoint::BeforeBranch,
            ImportInitPublicationPoint::AfterBranch,
        ] {
            let temp = tempdir().unwrap();
            let target = temp.path().join(format!("project-{point:?}"));
            fs::create_dir_all(&target).unwrap();
            fs::write(target.join("README.md"), "import me\n").unwrap();
            let options = import_options(&target, "main");
            inject_import_init_publication_failure(&target, point);

            let error = import_existing_project(&options).unwrap_err();
            assert!(error
                .to_string()
                .contains("injected import init publication failure"));
            assert!(target.join(".codefire/repo.json").is_file());
            let branch_path = target.join(".codefire/branches/main.json");
            assert_eq!(
                branch_path.exists(),
                point == ImportInitPublicationPoint::AfterBranch
            );
            if branch_path.exists() {
                let branch = read_json(&branch_path).unwrap();
                let head = required_string(&branch, &["head"]).unwrap();
                codefire_store::validate_sealed_commit(&target.join(".codefire/objects"), &head)
                    .unwrap();
            }

            let recovered = import_existing_project(&options).unwrap();
            assert_eq!(recovered.initial_files, 1);
            assert_eq!(
                load_import_transaction(&target).unwrap().unwrap().phase,
                ImportPhase::Completed
            );
        }
    }

    #[test]
    fn every_durable_phase_resumes_to_the_same_result() {
        for phase in [
            ImportPhase::Planned,
            ImportPhase::Initialized,
            ImportPhase::Opened,
            ImportPhase::Scanned,
            ImportPhase::Completed,
        ] {
            let temp = tempdir().unwrap();
            let target = temp.path().join(format!("project-{}", phase.as_str()));
            fs::create_dir_all(&target).unwrap();
            fs::write(target.join("README.md"), "import me\n").unwrap();
            let options = wait_import_options(&target, "main", "import-once");
            inject_import_failure(&target, phase);

            let error = import_existing_project(&options).unwrap_err();
            assert!(error.to_string().contains("injected import failure"));
            let recovered = import_existing_project(&options).unwrap();
            let replay = import_existing_project(&options).unwrap();

            assert_eq!(recovered.base_commit, replay.base_commit);
            assert_eq!(recovered.branch, "main");
            assert_eq!(recovered.initial_files, 1);
            assert_eq!(replay.plan["replayed"], true);
            assert_eq!(
                load_import_transaction(&target).unwrap().unwrap().phase,
                ImportPhase::Completed
            );
            let transaction = load_import_transaction(&target).unwrap().unwrap();
            let subject = ImportSubject {
                target: target.clone(),
                root: open_absolute_dir_nofollow(&target).unwrap(),
                transaction_id: transaction.transaction_id.clone(),
            };
            validate_import_replay_subject(
                &subject,
                &import_idempotency_payload(&options, &target).unwrap(),
                &transaction,
                &replay,
            )
            .unwrap();
        }
    }

    #[test]
    fn retry_preserves_and_rejects_an_invalid_state_transaction() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        inject_import_failure(&target, ImportPhase::Opened);
        import_existing_project(&options).unwrap_err();
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let path = transaction.active_path().join("state_transaction.json");
        let invalid = json!({
            "version": 1,
            "status": "pending",
            "operation": "scan",
            "phase": "verification_written",
            "intended_state": "open-burning",
            "started_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        });
        write_json_atomic(&path, &invalid).unwrap();

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("unsupported for operation scan"));
        assert_eq!(read_json(&path).unwrap(), invalid);
    }

    #[test]
    fn retry_takes_over_a_valid_pending_state_transaction() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        inject_import_failure(&target, ImportPhase::Opened);
        import_existing_project(&options).unwrap_err();
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let active = transaction.active_path();
        let path = active.join("state_transaction.json");
        write_json_atomic(
            &path,
            &json!({
                "version": 1,
                "status": "pending",
                "operation": "verify",
                "phase": "verification_written",
                "intended_state": "open-consistent",
                "started_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            }),
        )
        .unwrap();
        write_json_atomic(&active.join("verification.json"), &json!({"stale": true})).unwrap();

        let recovered = import_existing_project(&options).unwrap();
        assert_eq!(recovered.initial_files, 1);
        assert!(!path.exists());
        assert!(!active.join("verification.json").exists());
    }

    #[test]
    fn reserved_artifacts_are_ignored_but_similar_user_files_are_tracked() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(target.join("nested")).unwrap();
        for path in [
            "README.md",
            ".codefire-import.lock.user",
            ".codefire-import-transaction.json.backup",
            ".codefire-import.lock.bootstrap-user.txt",
            ".codefire-import.write-user.txt",
            ".codefire-import.write-user.part",
            "nested/.codefire-import.lock",
            "nested/.codefire-import.lock.bootstrap-user.tmp",
            "nested/.codefire-import.write-user.tmp",
        ] {
            fs::write(target.join(path), format!("{path}\n")).unwrap();
        }

        let result = import_existing_project(&import_options(&target, "main")).unwrap();
        assert_eq!(result.initial_files, 9);
        let transaction = load_import_transaction(&target).unwrap().unwrap();
        let scan: codefire_core::ScanResult = serde_json::from_value(
            read_json(&transaction.active_path().join("scan.json")).unwrap(),
        )
        .unwrap();
        let changed = scan
            .non_atom_changed_files
            .iter()
            .map(|change| change.path.as_str())
            .collect::<Vec<_>>();
        assert!(changed.contains(&".codefire-import.lock.user"));
        assert!(changed.contains(&".codefire-import-transaction.json.backup"));
        assert!(changed.contains(&".codefire-import.lock.bootstrap-user.txt"));
        assert!(changed.contains(&".codefire-import.write-user.txt"));
        assert!(changed.contains(&".codefire-import.write-user.part"));
        assert!(changed.contains(&"nested/.codefire-import.lock"));
        assert!(changed.contains(&"nested/.codefire-import.lock.bootstrap-user.tmp"));
        assert!(changed.contains(&"nested/.codefire-import.write-user.tmp"));
        assert!(!changed.contains(&IMPORT_BOOTSTRAP_LOCK_FILE));
        assert!(!changed.contains(&IMPORT_TRANSACTION_FILE));
    }

    #[test]
    fn crafted_idempotency_marker_without_repository_subject_is_rejected() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        let options = import_options(&target, "main");
        let payload = import_idempotency_payload(&options, &target).unwrap();
        save_import_idempotency_marker(
            &target,
            "import-once",
            &payload,
            &import_result(&target, "main"),
        )
        .unwrap();

        let error = import_existing_project(&options).unwrap_err();
        assert!(!matches!(error, CliError::IdempotencyConflict(_)));
        assert!(!target.join(IMPORT_BOOTSTRAP_LOCK_FILE).exists());
    }

    #[test]
    fn replay_rejects_corrupted_open_transaction_identity() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        import_existing_project(&options).unwrap();
        let marker_path = target.join(".codefire-open");
        let mut marker = read_json(&marker_path).unwrap();
        marker["open"]["opened_at"] = Value::String("corrupted-opened-at".to_string());
        write_json_atomic(&marker_path, &marker).unwrap();

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("open marker"));
    }

    #[test]
    fn replay_rejects_corrupted_repository_initialization_identity() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        import_existing_project(&options).unwrap();
        let repo_path = target.join(".codefire/repo.json");
        let mut repo = read_json(&repo_path).unwrap();
        repo["initialization"]["transaction_id"] = Value::String("import_tx_corrupted".to_string());
        write_json_atomic(&repo_path, &repo).unwrap();

        let error = import_existing_project(&options).unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        assert!(error.to_string().contains("repository metadata"));
    }

    #[test]
    fn concurrent_conflicting_import_rejects_one_request() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let workers = ["main", "feature"]
            .into_iter()
            .map(|branch| {
                let target = target.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    import_existing_project(&wait_import_options(
                        &target,
                        branch,
                        "same-import-key",
                    ))
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let results = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            results.iter().filter(|result| result.is_ok()).count(),
            1,
            "{results:#?}"
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(CliError::IdempotencyConflict(_))))
                .count(),
            1
        );
    }

    #[test]
    fn replay_accepts_a_coherent_advanced_sealed_head() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let original = import_existing_project(&options).unwrap();
        let objects = target.join(".codefire").join("objects");
        let mut next_commit = codefire_store::read_object(&objects, &original.base_commit).unwrap();
        next_commit["parents"] = json!([&original.base_commit]);
        next_commit["message"] = Value::String("advanced after import".to_string());
        next_commit["created_at"] = Value::String(now_iso_utc());
        let next_commit = codefire_store::store_object(&objects, "commit", next_commit).unwrap();
        codefire_store::validate_sealed_commit(&objects, &next_commit).unwrap();

        let mut branch = load_branch_record(&target, "main").unwrap();
        branch["head"] = Value::String(next_commit.clone());
        branch["state"] = Value::String("open-clean".to_string());
        save_branch_record(&target, &branch).unwrap();
        let registry_path = opened_registry_path(&target, "main");
        let mut registry = read_json(&registry_path).unwrap();
        registry["open"]["current_base_commit"] = Value::String(next_commit);
        registry["state"]["last_known"] = Value::String("open-clean".to_string());
        let active_path =
            PathBuf::from(required_string(&registry, &["open", "active_state_path"]).unwrap());
        write_json_atomic(&registry_path, &registry).unwrap();
        write_json_atomic(
            &active_path.join("state.json"),
            &json!({"state": "open-clean"}),
        )
        .unwrap();

        let replay = import_existing_project(&options).unwrap();
        assert_eq!(replay.base_commit, original.base_commit);
        assert_eq!(replay.plan["replayed"], true);
    }

    #[test]
    fn replay_heals_marker_saved_before_transaction_completion() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        let original = import_existing_project(&options).unwrap();
        let mut transaction = load_import_transaction(&target).unwrap().unwrap();
        transaction.phase = ImportPhase::Scanned;
        save_import_transaction(&target, &transaction).unwrap();

        let replay = import_existing_project(&options).unwrap();
        assert_eq!(replay.base_commit, original.base_commit);
        assert_eq!(
            load_import_transaction(&target).unwrap().unwrap().phase,
            ImportPhase::Completed
        );
    }

    #[cfg(unix)]
    #[test]
    fn replay_keeps_the_same_bootstrap_lock_inode() {
        use std::os::unix::fs::MetadataExt;

        let temp = tempdir().unwrap();
        let target = temp.path().join("project");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("README.md"), "import me\n").unwrap();
        let options = import_options(&target, "main");
        import_existing_project(&options).unwrap();
        let lock_path = target.join(IMPORT_BOOTSTRAP_LOCK_FILE);
        let before = fs::metadata(&lock_path).unwrap();

        import_existing_project(&options).unwrap();
        let after = fs::metadata(&lock_path).unwrap();
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    }
}
