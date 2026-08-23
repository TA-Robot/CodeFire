use crate::{now_iso_utc, secure_worktree, unique_temp_path, CliError, LockOptions};
use cap_std::fs::Dir;
use fs2::FileExt as Fs2FileExt;
use serde_json::{json, Value};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const OWNER_BYTE_LIMIT: u64 = 16 * 1024;
const RETRY_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StableLockMode {
    Shared,
    Exclusive,
}

impl StableLockMode {
    fn label(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Exclusive => "exclusive",
        }
    }
}

/// One monotonic deadline shared by every lock acquired for an operation.
///
/// A multi-project operation must not receive a fresh timeout for each lock:
/// doing so makes the configured bound grow with the number of resources.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LockDeadline {
    wait: bool,
    timeout: Option<Duration>,
    started_at: Instant,
}

impl LockDeadline {
    pub(crate) fn from_options(options: &LockOptions) -> Self {
        Self {
            wait: options.wait,
            timeout: options.timeout_ms.map(Duration::from_millis),
            started_at: Instant::now(),
        }
    }

    fn should_retry(self) -> bool {
        self.wait
            && match self.timeout {
                Some(timeout) => self.started_at.elapsed() < timeout,
                None => true,
            }
    }

    fn timeout_ms(self) -> Option<u64> {
        self.timeout.map(|timeout| timeout.as_millis() as u64)
    }

    fn retry_delay(self) -> Duration {
        match self.timeout {
            Some(timeout) => RETRY_INTERVAL.min(timeout.saturating_sub(self.started_at.elapsed())),
            None => RETRY_INTERVAL,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StableFileLock {
    path: PathBuf,
    directory: Dir,
    file_name: OsString,
    identity: secure_worktree::LockFileIdentity,
    file: File,
}

pub(crate) struct PreparedStableLockLabel {
    path: PathBuf,
}

impl StableFileLock {
    pub(crate) fn acquire(
        path: &Path,
        mode: StableLockMode,
        deadline: &LockDeadline,
    ) -> Result<Self, CliError> {
        let parent_path = path.parent().ok_or_else(|| {
            CliError::InvalidRepository("stable lock path has no parent".to_string())
        })?;
        let file_name = path.file_name().ok_or_else(|| {
            CliError::InvalidRepository("stable lock path has no filename".to_string())
        })?;
        let directory = secure_worktree::open_absolute_dir_nofollow(parent_path)?;
        Self::acquire_in_directory(&directory, file_name, path, mode, deadline)
    }

    pub(crate) fn acquire_in_directory(
        directory: &Dir,
        file_name: &OsStr,
        display_path: &Path,
        mode: StableLockMode,
        deadline: &LockDeadline,
    ) -> Result<Self, CliError> {
        let owner_name = owner_name(file_name);
        let owner_path = display_path.with_file_name(&owner_name);
        let (file, identity) = secure_worktree::open_lock_file_in_directory_nofollow(
            directory,
            file_name,
            display_path,
        )?;
        loop {
            let result = match mode {
                StableLockMode::Shared => Fs2FileExt::try_lock_shared(&file),
                StableLockMode::Exclusive => Fs2FileExt::try_lock_exclusive(&file),
            };
            match result {
                Ok(()) => {
                    secure_worktree::ensure_lock_file_identity_in_directory_nofollow(
                        directory,
                        file_name,
                        display_path,
                        identity,
                    )?;
                    let _ = publish_owner_in_directory(directory, &owner_name, &owner_path, mode);
                    return Ok(Self {
                        path: display_path.to_path_buf(),
                        directory: directory.try_clone()?,
                        file_name: file_name.to_os_string(),
                        identity,
                        file,
                    });
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    if !deadline.should_retry() {
                        return Err(contention_in_directory(
                            directory,
                            &owner_name,
                            display_path,
                            &owner_path,
                            *deadline,
                        ));
                    }
                    std::thread::sleep(deadline.retry_delay());
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }

    pub(crate) fn ensure_identity(&self) -> Result<(), CliError> {
        secure_worktree::ensure_lock_file_identity_in_directory_nofollow(
            &self.directory,
            &self.file_name,
            &self.path,
            self.identity,
        )?;
        Ok(())
    }

    pub(crate) fn ensure_subject(
        &self,
        expected_directory: &Dir,
        expected_file_name: &OsStr,
        expected_display_path: &Path,
    ) -> Result<(), CliError> {
        if self.file_name != expected_file_name {
            return Err(CliError::InvalidRepository(format!(
                "stable lock subject changed: expected {}, found {}",
                expected_display_path.display(),
                self.path.display()
            )));
        }
        let directory_identity = secure_worktree::StableFsIdentity::for_directory(&self.directory)?;
        directory_identity.ensure_directory(expected_directory, "stable lock directory")?;
        self.ensure_identity()?;
        secure_worktree::ensure_lock_file_identity_in_directory_nofollow(
            expected_directory,
            expected_file_name,
            expected_display_path,
            self.identity,
        )?;
        Ok(())
    }

    pub(crate) fn prepare_display_path_label(
        &self,
        path: &Path,
    ) -> Result<PreparedStableLockLabel, CliError> {
        if path.file_name() != Some(self.file_name.as_os_str()) {
            return Err(CliError::InvalidRepository(format!(
                "stable lock final label does not name the locked file: {}",
                path.display()
            )));
        }
        self.ensure_identity()?;
        Ok(PreparedStableLockLabel {
            path: path.to_path_buf(),
        })
    }

    pub(crate) fn adopt_prepared_display_path_label(&mut self, label: PreparedStableLockLabel) {
        self.path = label.path;
    }
}

impl Drop for StableFileLock {
    fn drop(&mut self) {
        let _ = Fs2FileExt::unlock(&self.file);
    }
}

fn owner_name(lock_name: &OsStr) -> OsString {
    let name = lock_name.to_str().unwrap_or("resource.lock");
    OsString::from(format!("{name}.owner.json"))
}

fn publish_owner_in_directory(
    directory: &Dir,
    owner_name: &OsStr,
    display_path: &Path,
    mode: StableLockMode,
) -> Result<(), CliError> {
    let mut owner = serde_json::to_vec_pretty(&json!({
        "version": 1,
        "pid": std::process::id(),
        "mode": mode.label(),
        "created_at": now_iso_utc(),
    }))?;
    owner.push(b'\n');
    let temp_path = unique_temp_path(display_path);
    let temp_name = temp_path.file_name().ok_or_else(|| {
        CliError::InvalidRepository("stable lock owner temp name is missing".to_string())
    })?;
    secure_worktree::write_file_atomic_in_directory_nofollow(
        directory, owner_name, temp_name, &owner,
    )?;
    Ok(())
}

fn contention_in_directory(
    directory: &Dir,
    owner_name: &OsStr,
    path: &Path,
    owner_path: &Path,
    deadline: LockDeadline,
) -> CliError {
    let owner = secure_worktree::read_file_in_directory_nofollow_bounded(
        directory,
        owner_name,
        owner_path,
        OWNER_BYTE_LIMIT,
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let mut message = format!("CodeFire resource is locked: {}", path.display());
    if let Some(timeout_ms) = deadline.timeout_ms() {
        message.push_str(&format!("; timed out after {timeout_ms}ms"));
    }
    if let Some(owner) = owner {
        if let Some(pid) = owner.get("pid").and_then(Value::as_u64) {
            message.push_str(&format!("; owner pid {pid}"));
        }
        if let Some(mode) = owner.get("mode").and_then(Value::as_str) {
            message.push_str(&format!("; owner mode {mode}"));
        }
        if let Some(created_at) = owner.get("created_at").and_then(Value::as_str) {
            message.push_str(&format!("; locked at {created_at}"));
        }
    }
    CliError::LockContention(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "codefire-stable-lock-{label}-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn persistent_lock_file_is_not_removed_on_drop() {
        let root = temp_dir("persistent");
        let path = root.join("project.lock");
        {
            let deadline = LockDeadline::from_options(&LockOptions::default());
            let lock =
                StableFileLock::acquire(&path, StableLockMode::Exclusive, &deadline).unwrap();
            lock.ensure_identity().unwrap();
        }
        assert!(path.is_file());
        let second_deadline = LockDeadline::from_options(&LockOptions::default());
        StableFileLock::acquire(&path, StableLockMode::Exclusive, &second_deadline).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stable_lock_subject_binds_directory_name_and_inode() {
        let root = temp_dir("subject");
        let expected = root.join("expected");
        let other = root.join("other");
        fs::create_dir(&expected).unwrap();
        fs::create_dir(&other).unwrap();
        let expected_dir = secure_worktree::open_absolute_dir_nofollow(&expected).unwrap();
        let other_dir = secure_worktree::open_absolute_dir_nofollow(&other).unwrap();
        let path = expected.join("project-gate.lock");
        let lock = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap();
        lock.ensure_subject(&expected_dir, OsStr::new("project-gate.lock"), &path)
            .unwrap();
        assert!(lock
            .ensure_subject(&other_dir, OsStr::new("project-gate.lock"), &path)
            .is_err());
        assert!(lock
            .ensure_subject(&expected_dir, OsStr::new("other.lock"), &path)
            .is_err());
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stable_lock_adopts_only_a_prevalidated_final_label() {
        let root = temp_dir("prepared-label");
        let path = root.join("project-gate.lock");
        let mut lock = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap();
        let final_path = root.join("published/locks/project-gate.lock");
        let prepared = lock.prepare_display_path_label(&final_path).unwrap();

        lock.adopt_prepared_display_path_label(prepared);

        assert_eq!(lock.path, final_path);
        lock.ensure_identity().unwrap();
        assert!(lock
            .prepare_display_path_label(&root.join("published/locks/other.lock"))
            .is_err());
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_holders_block_exclusive_holder() {
        let root = temp_dir("shared");
        let path = root.join("project.lock");
        let deadline = LockDeadline::from_options(&LockOptions::default());
        let shared = StableFileLock::acquire(&path, StableLockMode::Shared, &deadline).unwrap();
        let error = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap_err();
        assert!(matches!(error, CliError::LockContention(_)));
        drop(shared);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn hostile_owner_symlink_is_non_authoritative_and_lock_hardlinks_fail_closed() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("hostile-owner");
        let outside = root.join("outside.json");
        fs::write(&outside, b"preserve\n").unwrap();
        let path = root.join("project.lock");
        symlink(&outside, root.join("project.lock.owner.json")).unwrap();
        StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap();
        assert_eq!(fs::read(&outside).unwrap(), b"preserve\n");

        let linked = root.join("linked.lock");
        fs::write(&linked, b"").unwrap();
        fs::hard_link(&linked, root.join("linked-copy.lock")).unwrap();
        assert!(StableFileLock::acquire(
            &linked,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn short_wait_uses_one_bounded_deadline() {
        let root = temp_dir("deadline");
        let path = root.join("project.lock");
        let held = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap();
        let started = Instant::now();
        let error = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions {
                wait: true,
                timeout_ms: Some(5),
            }),
        )
        .unwrap_err();
        assert!(matches!(error, CliError::LockContention(_)));
        assert!(started.elapsed() < Duration::from_millis(100));
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rust_lock_interoperates_with_python_flock_across_process_exit() {
        use std::process::Command;

        let root = temp_dir("cross-runtime");
        let path = root.join("project.lock");
        let held = StableFileLock::acquire(
            &path,
            StableLockMode::Exclusive,
            &LockDeadline::from_options(&LockOptions::default()),
        )
        .unwrap();
        let probe = r#"import fcntl, os, sys
fd = os.open(sys.argv[1], os.O_RDWR | os.O_NOFOLLOW)
try:
    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
except BlockingIOError:
    raise SystemExit(23)
"#;
        let blocked = Command::new("python3")
            .arg("-c")
            .arg(probe)
            .arg(&path)
            .status()
            .unwrap();
        assert_eq!(blocked.code(), Some(23));
        drop(held);
        let recovered = Command::new("python3")
            .arg("-c")
            .arg(probe)
            .arg(&path)
            .status()
            .unwrap();
        assert!(recovered.success());
        assert!(path.is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
