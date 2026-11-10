//! The repository lock and its owner record.

use crate::cli_error::CliError;
use crate::json_files::unique_temp_path;
use crate::open_context::{open_context, open_context_for_recovery};
use crate::test_hooks::maybe_mutate_locked_open_context_for_test;
use crate::{find_repo_root, now_iso_utc, secure_worktree, stable_lock, LockOptions, OpenContext};
use fs2::FileExt as Fs2FileExt;
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct RepoLock {
    pub(crate) repo_root: PathBuf,
    pub(crate) path: PathBuf,
    pub(crate) identity: secure_worktree::LockFileIdentity,
    pub(crate) file: File,
}

pub(crate) const REPO_LOCK_OWNER_BYTE_LIMIT: u64 = 16 * 1024;

pub(crate) fn acquire_locked_open_context(
    start: &Path,
    options: &LockOptions,
) -> Result<(RepoLock, OpenContext), CliError> {
    acquire_locked_open_context_with_recovery(start, options, None)
}

pub(crate) fn acquire_locked_open_context_for_recovery(
    start: &Path,
    options: &LockOptions,
    operation: &str,
) -> Result<(RepoLock, OpenContext), CliError> {
    acquire_locked_open_context_with_recovery(start, options, Some(operation))
}

pub(crate) fn acquire_locked_open_context_with_recovery(
    start: &Path,
    options: &LockOptions,
    recovery_operation: Option<&str>,
) -> Result<(RepoLock, OpenContext), CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    maybe_mutate_locked_open_context_for_test(start)?;
    let context = match recovery_operation {
        Some(operation) => open_context_for_recovery(start, operation)?,
        None => open_context(start)?,
    };
    lock.ensure_repo_root(&context.repo_root)?;
    Ok((lock, context))
}

impl RepoLock {
    pub(crate) fn acquire_with_options(
        repo_root: &Path,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let repo_root = fs::canonicalize(repo_root)?;
        let path = repo_root.join(".codefire").join("locks").join("repo.lock");
        let owner_path = repo_lock_owner_path(&path);
        let (file, identity) = secure_worktree::open_lock_file_nofollow(&path)?;
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let start = Instant::now();
        loop {
            match Fs2FileExt::try_lock_exclusive(&file) {
                Ok(()) => {
                    secure_worktree::ensure_lock_file_identity_nofollow(&path, identity)?;
                    publish_repo_lock_owner(&owner_path)?;
                    return Ok(Self {
                        repo_root: repo_root.clone(),
                        path,
                        identity,
                        file,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if !options.wait {
                        return Err(repo_lock_contention_with_owner(
                            &repo_root,
                            &path,
                            Some(&owner_path),
                            options,
                        ));
                    }
                    if timeout.is_some_and(|timeout| start.elapsed() >= timeout) {
                        return Err(repo_lock_contention_with_owner(
                            &repo_root,
                            &path,
                            Some(&owner_path),
                            options,
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }

    pub(crate) fn ensure_repo_root(&self, repo_root: &Path) -> Result<(), CliError> {
        let resolved = fs::canonicalize(repo_root)?;
        if resolved != self.repo_root {
            return Err(CliError::InvalidMarker(format!(
                "open marker repository changed while acquiring the repository lock: locked {}, resolved {}",
                self.repo_root.display(),
                resolved.display()
            )));
        }
        secure_worktree::ensure_lock_file_identity_nofollow(&self.path, self.identity)?;
        Ok(())
    }
}

pub(crate) fn repo_lock_contention_with_owner(
    repo_root: &Path,
    path: &Path,
    owner_path: Option<&Path>,
    options: &LockOptions,
) -> CliError {
    let owner = owner_path.and_then(read_repo_lock_owner);
    let mut message = format!("CodeFire repository is locked: {}", path.display());
    if let Some(timeout_ms) = options.timeout_ms {
        message.push_str(&format!("; timed out after {timeout_ms}ms"));
    }
    if let Some(owner) = owner {
        if let Some(pid) = owner.get("pid").and_then(Value::as_u64) {
            message.push_str(&format!("; owner pid {pid}"));
        }
        if let Some(created_at) = owner.get("created_at").and_then(Value::as_str) {
            message.push_str(&format!("; locked at {created_at}"));
        }
    }
    CliError::RepositoryLockContention {
        message,
        repo_root: repo_root.to_path_buf(),
        lock_path: path.to_path_buf(),
    }
}

pub(crate) fn repo_lock_owner_path(lock_path: &Path) -> PathBuf {
    lock_path.with_file_name("repo.lock.owner.json")
}

pub(crate) fn publish_repo_lock_owner(path: &Path) -> Result<(), CliError> {
    let mut owner = serde_json::to_vec_pretty(&json!({
        "version": 1,
        "pid": std::process::id(),
        "created_at": now_iso_utc(),
    }))?;
    owner.push(b'\n');
    let temp_path = unique_temp_path(path);
    secure_worktree::write_file_atomic_nofollow(path, &temp_path, &owner)?;
    Ok(())
}

pub(crate) fn read_repo_lock_owner(path: &Path) -> Option<Value> {
    let bytes =
        secure_worktree::read_file_nofollow_bounded(path, REPO_LOCK_OWNER_BYTE_LIMIT).ok()?;
    serde_json::from_slice(&bytes).ok()
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = Fs2FileExt::unlock(&self.file);
    }
}

pub(crate) struct FileLock {
    pub(crate) _lock: stable_lock::StableFileLock,
}

impl FileLock {
    pub(crate) fn acquire_with_options(
        path: PathBuf,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let deadline = stable_lock::LockDeadline::from_options(options);
        stable_lock::StableFileLock::acquire(
            &path,
            stable_lock::StableLockMode::Exclusive,
            &deadline,
        )
        .map(|lock| Self { _lock: lock })
    }
}
