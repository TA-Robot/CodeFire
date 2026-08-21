use crate::secure_worktree::{
    ensure_absolute_dir_identity, ensure_same_directory, open_absolute_dir_nofollow,
    rename_entry_noreplace, RemovalProgress, StableFsIdentity,
};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const TEMP_CREATE_ATTEMPTS: usize = 128;
static ACTIVE_STATE_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct ActiveStateError {
    operation: &'static str,
    path: PathBuf,
    source: io::Error,
}

impl ActiveStateError {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn operation(&self) -> &'static str {
        self.operation
    }

    pub(crate) fn is_not_found(&self) -> bool {
        self.source.kind() == io::ErrorKind::NotFound
    }

    fn new(operation: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self {
            operation,
            path,
            source,
        }
    }
}

impl fmt::Display for ActiveStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "active-state capability {} failed at {}: {}",
            self.operation,
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for ActiveStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// A pinned active-state instance directory.
///
/// The absolute path is retained only for diagnostics and identity rechecks.
/// Every data operation is relative to `root`; a concurrent rename or symlink
/// replacement therefore cannot redirect a read, write, or deletion outside
/// the directory that was validated by `open_context`.
pub(crate) struct ActiveStateDir {
    parent: Dir,
    root: Dir,
    parent_path: PathBuf,
    path: PathBuf,
    name: OsString,
}

enum BoundActiveRemoval {
    Original(ActiveStateDir),
    Quarantined(QuarantinedActiveState),
    Removed,
}

struct QuarantinedActiveState {
    parent: Dir,
    root: Dir,
    parent_path: PathBuf,
    path: PathBuf,
    original_name: OsString,
    quarantine_name: OsString,
}

impl fmt::Debug for ActiveStateDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActiveStateDir")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

fn bind_removal(
    path: &Path,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
    expected_parent: StableFsIdentity,
) -> Result<BoundActiveRemoval, ActiveStateError> {
    validate_entry_name(quarantine_name)
        .map_err(|error| ActiveStateError::new("remove tree", path.to_path_buf(), error))?;
    let parent_path = path.parent().ok_or_else(|| {
        ActiveStateError::new(
            "remove tree",
            path.to_path_buf(),
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "active-state path has no parent",
            ),
        )
    })?;
    let original_name = path.file_name().ok_or_else(|| {
        ActiveStateError::new(
            "remove tree",
            path.to_path_buf(),
            io::Error::new(io::ErrorKind::InvalidInput, "active-state path has no name"),
        )
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)
        .map_err(|error| ActiveStateError::new("remove tree", parent_path.to_path_buf(), error))?;
    expected_parent
        .ensure_directory(&parent, "active-state parent")
        .map_err(|error| ActiveStateError::new("remove tree", parent_path.to_path_buf(), error))?;
    match parent.open_dir_nofollow(original_name) {
        Ok(root) => {
            expected
                .ensure_directory(&root, "active state")
                .map_err(|error| ActiveStateError::new("remove tree", path.to_path_buf(), error))?;
            match parent.symlink_metadata(quarantine_name) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(ActiveStateError::new(
                        "remove tree",
                        parent_path.to_path_buf(),
                        error,
                    ));
                }
                Ok(_) => {
                    return Err(ActiveStateError::new(
                        "remove tree",
                        path.to_path_buf(),
                        io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            "both active state and its transaction quarantine exist",
                        ),
                    ));
                }
            }
            let active = ActiveStateDir {
                parent,
                root,
                parent_path: parent_path.to_path_buf(),
                path: path.to_path_buf(),
                name: original_name.to_os_string(),
            };
            active.ensure_current_for("remove tree")?;
            Ok(BoundActiveRemoval::Original(active))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            match parent.open_dir_nofollow(quarantine_name) {
                Ok(root) => {
                    expected
                        .ensure_directory(&root, "quarantined active state")
                        .map_err(|error| {
                            ActiveStateError::new("remove tree", path.to_path_buf(), error)
                        })?;
                    Ok(BoundActiveRemoval::Quarantined(QuarantinedActiveState {
                        parent,
                        root,
                        parent_path: parent_path.to_path_buf(),
                        path: path.to_path_buf(),
                        original_name: original_name.to_os_string(),
                        quarantine_name: quarantine_name.to_os_string(),
                    }))
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    Ok(BoundActiveRemoval::Removed)
                }
                Err(error) => Err(ActiveStateError::new(
                    "remove tree",
                    parent_path.join(quarantine_name),
                    error,
                )),
            }
        }
        Err(error) => Err(ActiveStateError::new(
            "remove tree",
            path.to_path_buf(),
            error,
        )),
    }
}

impl QuarantinedActiveState {
    fn remove_with(
        self,
        delete_contents: impl FnOnce(&Dir) -> io::Result<()>,
    ) -> Result<(), ActiveStateError> {
        ensure_entry_missing(&self.parent, &self.original_name)
            .map_err(|error| self.preserved_error(error))?;
        let current = self
            .parent
            .open_dir_nofollow(&self.quarantine_name)
            .map_err(|error| self.preserved_error(error))?;
        ensure_same_directory(&self.root, &current).map_err(|error| self.preserved_error(error))?;
        delete_contents(&self.root).map_err(|error| self.preserved_error(error))?;
        ensure_entry_missing(&self.parent, &self.original_name)
            .map_err(|error| self.preserved_error(error))?;
        let current = self
            .parent
            .open_dir_nofollow(&self.quarantine_name)
            .map_err(|error| self.preserved_error(error))?;
        ensure_same_directory(&self.root, &current).map_err(|error| self.preserved_error(error))?;
        self.parent
            .remove_dir(&self.quarantine_name)
            .map_err(|error| self.preserved_error(error))?;
        sync_directory(&self.parent).map_err(|error| {
            ActiveStateError::new("remove tree", self.parent_path.clone(), error)
        })?;
        ensure_entry_missing(&self.parent, &self.original_name)
            .map_err(|error| ActiveStateError::new("remove tree", self.path.clone(), error))?;
        ensure_entry_missing(&self.parent, &self.quarantine_name)
            .map_err(|error| ActiveStateError::new("remove tree", self.path.clone(), error))
    }

    fn preserved_error(&self, cause: io::Error) -> ActiveStateError {
        ActiveStateError::new(
            "remove tree",
            self.path.clone(),
            io::Error::new(
                cause.kind(),
                format!(
                    "{cause}; partially removed active state remains in transaction quarantine `{}`",
                    self.quarantine_name.to_string_lossy()
                ),
            ),
        )
    }
}

fn ensure_entry_missing(parent: &Dir, name: &OsStr) -> io::Result<()> {
    match parent.symlink_metadata(name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "active-state removal name changed during transaction",
        )),
        Err(error) => Err(error),
    }
}

fn validate_entry_name(name: &OsStr) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "active-state quarantine must be one normal path component",
        ));
    }
    Ok(())
}

impl ActiveStateDir {
    pub(crate) fn bind(path: &Path) -> Result<Self, ActiveStateError> {
        Self::bind_inner(path, false)
    }

    pub(crate) fn create(path: &Path) -> Result<Self, ActiveStateError> {
        Self::bind_inner(path, true)
    }

    fn bind_inner(path: &Path, create: bool) -> Result<Self, ActiveStateError> {
        let operation = if create { "create" } else { "bind" };
        let parent_path = path.parent().ok_or_else(|| {
            ActiveStateError::new(
                operation,
                path.to_path_buf(),
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "active-state path has no parent",
                ),
            )
        })?;
        let name = path.file_name().ok_or_else(|| {
            ActiveStateError::new(
                operation,
                path.to_path_buf(),
                io::Error::new(io::ErrorKind::InvalidInput, "active-state path has no name"),
            )
        })?;
        let parent = open_absolute_dir_nofollow(parent_path)
            .map_err(|error| ActiveStateError::new(operation, parent_path.to_path_buf(), error))?;
        if create {
            match parent.symlink_metadata(name) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    return Err(ActiveStateError::new(
                        operation,
                        path.to_path_buf(),
                        io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "active-state instance is not a real directory",
                        ),
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    parent.create_dir(name).map_err(|error| {
                        ActiveStateError::new(operation, path.to_path_buf(), error)
                    })?;
                    sync_directory(&parent).map_err(|error| {
                        ActiveStateError::new(operation, parent_path.to_path_buf(), error)
                    })?;
                }
                Err(error) => {
                    return Err(ActiveStateError::new(operation, path.to_path_buf(), error));
                }
            }
        }
        let root = parent
            .open_dir_nofollow(name)
            .map_err(|error| ActiveStateError::new(operation, path.to_path_buf(), error))?;
        let active = Self {
            parent,
            root,
            parent_path: parent_path.to_path_buf(),
            path: path.to_path_buf(),
            name: name.to_os_string(),
        };
        active.ensure_current_for(operation)?;
        Ok(active)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn display_path(&self, relative: &Path) -> PathBuf {
        self.path.join(relative)
    }

    pub(crate) fn ensure_current(&self) -> Result<(), ActiveStateError> {
        self.ensure_current_for("identity recheck")
    }

    pub(crate) fn identity(&self) -> Result<StableFsIdentity, ActiveStateError> {
        self.ensure_current_for("identity")?;
        StableFsIdentity::for_directory(&self.root)
            .map_err(|error| ActiveStateError::new("identity", self.path.clone(), error))
    }

    pub(crate) fn parent_identity(&self) -> Result<StableFsIdentity, ActiveStateError> {
        self.ensure_current_for("parent identity")?;
        StableFsIdentity::for_directory(&self.parent).map_err(|error| {
            ActiveStateError::new("parent identity", self.parent_path.clone(), error)
        })
    }

    fn ensure_current_for(&self, operation: &'static str) -> Result<(), ActiveStateError> {
        ensure_absolute_dir_identity(&self.parent, &self.parent_path)
            .map_err(|error| ActiveStateError::new(operation, self.parent_path.clone(), error))?;
        let current = self
            .parent
            .open_dir_nofollow(&self.name)
            .map_err(|error| ActiveStateError::new(operation, self.path.clone(), error))?;
        ensure_same_directory(&self.root, &current)
            .map_err(|error| ActiveStateError::new(operation, self.path.clone(), error))?;
        ensure_absolute_dir_identity(&self.root, &self.path)
            .map_err(|error| ActiveStateError::new(operation, self.path.clone(), error))
    }

    pub(crate) fn exists(&self, relative: &Path) -> Result<bool, ActiveStateError> {
        let name = self.file_name(relative, "metadata")?;
        self.ensure_current_for("metadata")?;
        let exists = match self.root.symlink_metadata(&name) {
            Ok(_) => true,
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => {
                return Err(ActiveStateError::new(
                    "metadata",
                    self.display_path(relative),
                    error,
                ));
            }
        };
        self.ensure_current_for("metadata")?;
        Ok(exists)
    }

    pub(crate) fn read_optional(
        &self,
        relative: &Path,
        limit: u64,
    ) -> Result<Option<Vec<u8>>, ActiveStateError> {
        let name = self.file_name(relative, "read")?;
        self.ensure_current_for("read")?;
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let mut file = match self.root.open_with(&name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.ensure_current_for("read")?;
                return Ok(None);
            }
            Err(error) => {
                return Err(ActiveStateError::new(
                    "read",
                    self.display_path(relative),
                    error,
                ));
            }
        };
        let metadata = file
            .metadata()
            .map_err(|error| ActiveStateError::new("read", self.display_path(relative), error))?;
        if !metadata.is_file() || metadata.len() > limit {
            return Err(ActiveStateError::new(
                "read",
                self.display_path(relative),
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("active-state file must be regular and at most {limit} bytes"),
                ),
            ));
        }
        let capacity = usize::try_from(metadata.len().min(limit)).unwrap_or(usize::MAX);
        let mut bytes = Vec::with_capacity(capacity.saturating_add(1));
        (&mut file)
            .take(limit.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| ActiveStateError::new("read", self.display_path(relative), error))?;
        if bytes.len() as u64 > limit {
            return Err(ActiveStateError::new(
                "read",
                self.display_path(relative),
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("active-state file grew beyond its {limit}-byte limit"),
                ),
            ));
        }
        self.ensure_current_for("read")?;
        Ok(Some(bytes))
    }

    pub(crate) fn read_required(
        &self,
        relative: &Path,
        limit: u64,
    ) -> Result<Vec<u8>, ActiveStateError> {
        self.read_optional(relative, limit)?.ok_or_else(|| {
            ActiveStateError::new(
                "read",
                self.display_path(relative),
                io::Error::from(io::ErrorKind::NotFound),
            )
        })
    }

    pub(crate) fn write_atomic(
        &self,
        relative: &Path,
        contents: &[u8],
        limit: u64,
    ) -> Result<(), ActiveStateError> {
        let name = self.file_name(relative, "write")?;
        if contents.len() as u64 > limit {
            return Err(ActiveStateError::new(
                "write",
                self.display_path(relative),
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("active-state file exceeds its {limit}-byte limit"),
                ),
            ));
        }
        self.ensure_current_for("write")?;
        match self.root.symlink_metadata(&name) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                return Err(ActiveStateError::new(
                    "write",
                    self.display_path(relative),
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "active-state write target is a directory",
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(ActiveStateError::new(
                    "write",
                    self.display_path(relative),
                    error,
                ));
            }
        }

        let (mut file, temp_name) = self.create_temp()?;
        let result = (|| {
            file.write_all(contents)?;
            file.sync_all()?;
            drop(file);
            self.ensure_current_for_io()?;
            self.root.rename(&temp_name, &self.root, &name)?;
            sync_directory(&self.root)?;
            self.ensure_current_for_io()
        })();
        if let Err(error) = result {
            let _ = self.root.remove_file(&temp_name);
            return Err(ActiveStateError::new(
                "write",
                self.display_path(relative),
                error,
            ));
        }
        Ok(())
    }

    pub(crate) fn write_new(
        &self,
        relative: &Path,
        contents: &[u8],
        limit: u64,
    ) -> Result<(), ActiveStateError> {
        let name = self.file_name(relative, "write new")?;
        if contents.len() as u64 > limit {
            return Err(ActiveStateError::new(
                "write new",
                self.display_path(relative),
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("active-state file exceeds its {limit}-byte limit"),
                ),
            ));
        }
        self.ensure_current_for("write new")?;
        let (mut file, temp_name) = self.create_temp().map_err(|error| {
            ActiveStateError::new("write new", self.display_path(relative), error.source)
        })?;
        let result = (|| {
            file.write_all(contents)?;
            file.sync_all()?;
            drop(file);
            self.ensure_current_for_io()?;
            rename_entry_noreplace(&self.root, &temp_name, &name)?;
            sync_directory(&self.root)?;
            self.ensure_current_for_io()
        })();
        if let Err(error) = result {
            let _ = self.root.remove_file(&temp_name);
            return Err(ActiveStateError::new(
                "write new",
                self.display_path(relative),
                error,
            ));
        }
        Ok(())
    }

    pub(crate) fn remove_file_if_exists(&self, relative: &Path) -> Result<bool, ActiveStateError> {
        let name = self.file_name(relative, "remove")?;
        self.ensure_current_for("remove")?;
        match self.root.symlink_metadata(&name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.ensure_current_for("remove")?;
                return Ok(false);
            }
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                return Err(ActiveStateError::new(
                    "remove",
                    self.display_path(relative),
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "active-state remove target is a directory",
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => {
                return Err(ActiveStateError::new(
                    "remove",
                    self.display_path(relative),
                    error,
                ));
            }
        }
        self.root
            .remove_file(&name)
            .map_err(|error| ActiveStateError::new("remove", self.display_path(relative), error))?;
        sync_directory(&self.root)
            .map_err(|error| ActiveStateError::new("remove", self.display_path(relative), error))?;
        self.ensure_current_for("remove")?;
        Ok(true)
    }

    pub(crate) fn removal_progress(
        path: &Path,
        quarantine_name: &OsStr,
        expected: StableFsIdentity,
        expected_parent: StableFsIdentity,
    ) -> Result<RemovalProgress, ActiveStateError> {
        match bind_removal(path, quarantine_name, expected, expected_parent)? {
            BoundActiveRemoval::Original(_) => Ok(RemovalProgress::Before),
            BoundActiveRemoval::Quarantined(_) => Ok(RemovalProgress::Quarantined),
            BoundActiveRemoval::Removed => Ok(RemovalProgress::Removed),
        }
    }

    pub(crate) fn remove_tree_transactional(
        path: &Path,
        quarantine_name: &OsStr,
        expected: StableFsIdentity,
        expected_parent: StableFsIdentity,
    ) -> Result<(), ActiveStateError> {
        Self::remove_tree_transactional_with(
            path,
            quarantine_name,
            expected,
            expected_parent,
            clear_directory_contents_nofollow,
        )
    }

    fn remove_tree_transactional_with(
        path: &Path,
        quarantine_name: &OsStr,
        expected: StableFsIdentity,
        expected_parent: StableFsIdentity,
        delete_contents: impl FnOnce(&Dir) -> io::Result<()>,
    ) -> Result<(), ActiveStateError> {
        let quarantined = match bind_removal(path, quarantine_name, expected, expected_parent)? {
            BoundActiveRemoval::Original(active) => {
                rename_entry_noreplace(&active.parent, &active.name, quarantine_name).map_err(
                    |error| ActiveStateError::new("remove tree", active.path.clone(), error),
                )?;
                sync_directory(&active.parent).map_err(|error| {
                    ActiveStateError::new("remove tree", active.parent_path.clone(), error)
                })?;
                let root = active
                    .parent
                    .open_dir_nofollow(quarantine_name)
                    .map_err(|error| {
                        ActiveStateError::new("remove tree", active.path.clone(), error)
                    })?;
                expected
                    .ensure_directory(&root, "quarantined active state")
                    .map_err(|error| {
                        ActiveStateError::new("remove tree", active.path.clone(), error)
                    })?;
                QuarantinedActiveState {
                    parent: active.parent,
                    root,
                    parent_path: active.parent_path,
                    path: active.path,
                    original_name: active.name,
                    quarantine_name: quarantine_name.to_os_string(),
                }
            }
            BoundActiveRemoval::Quarantined(active) => active,
            BoundActiveRemoval::Removed => return Ok(()),
        };
        quarantined.remove_with(delete_contents)?;
        if Self::removal_progress(path, quarantine_name, expected, expected_parent)?
            != RemovalProgress::Removed
        {
            return Err(ActiveStateError::new(
                "remove tree",
                path.to_path_buf(),
                io::Error::other("active-state removal did not reach the removed state"),
            ));
        }
        Ok(())
    }

    fn ensure_current_for_io(&self) -> io::Result<()> {
        ensure_absolute_dir_identity(&self.parent, &self.parent_path)?;
        let current = self.parent.open_dir_nofollow(&self.name)?;
        ensure_same_directory(&self.root, &current)?;
        ensure_absolute_dir_identity(&self.root, &self.path)
    }

    fn file_name(
        &self,
        relative: &Path,
        operation: &'static str,
    ) -> Result<OsString, ActiveStateError> {
        let mut components = relative.components();
        let name = match (components.next(), components.next()) {
            (Some(Component::Normal(name)), None) => name.to_os_string(),
            _ => {
                return Err(ActiveStateError::new(
                    operation,
                    self.display_path(relative),
                    io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "active-state file must be one normal relative component",
                    ),
                ));
            }
        };
        Ok(name)
    }

    fn create_temp(&self) -> Result<(cap_std::fs::File, OsString), ActiveStateError> {
        for _ in 0..TEMP_CREATE_ATTEMPTS {
            let counter = ACTIVE_STATE_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let name = OsString::from(format!(
                ".codefire-active-write.{}.{counter:016x}.tmp",
                std::process::id()
            ));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            options.follow(FollowSymlinks::No);
            match self.root.open_with(&name, &options) {
                Ok(file) => return Ok((file, name)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(ActiveStateError::new("write", self.path.clone(), error));
                }
            }
        }
        Err(ActiveStateError::new(
            "write",
            self.path.clone(),
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "could not allocate an active-state temporary file",
            ),
        ))
    }
}

fn clear_directory_contents_nofollow(directory: &Dir) -> io::Result<()> {
    let mut names = directory
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    for name in names {
        let metadata = directory.symlink_metadata(&name)?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let child = directory.open_dir_nofollow(&name)?;
            clear_directory_contents_nofollow(&child)?;
            let current = directory.open_dir_nofollow(&name)?;
            ensure_same_directory(&child, &current)?;
            directory.remove_dir(&name)?;
        } else {
            directory.remove_file(&name)?;
        }
    }
    sync_directory(directory)
}

fn sync_directory(directory: &Dir) -> io::Result<()> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn relative_io_is_bounded_to_the_pinned_instance() {
        let temp = tempdir().unwrap();
        let parent = temp.path().join("active");
        let path = parent.join("open_1");
        std::fs::create_dir_all(&path).unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();

        active
            .write_atomic(
                Path::new("state.json"),
                b"{\"state\":\"open-clean\"}\n",
                1024,
            )
            .unwrap();

        assert_eq!(
            active.read_required(Path::new("state.json"), 1024).unwrap(),
            b"{\"state\":\"open-clean\"}\n"
        );
        assert!(active.exists(Path::new("state.json")).unwrap());
        assert!(active
            .remove_file_if_exists(Path::new("state.json"))
            .unwrap());
        assert!(!active.exists(Path::new("state.json")).unwrap());
    }

    #[test]
    fn write_new_never_replaces_an_existing_archive() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("active/open_1");
        std::fs::create_dir_all(&path).unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();

        active
            .write_new(Path::new("archive.json"), b"first\n", 1024)
            .unwrap();
        let error = active
            .write_new(Path::new("archive.json"), b"second\n", 1024)
            .unwrap_err();

        assert_eq!(error.operation(), "write new");
        assert_eq!(error.path(), path.join("archive.json"));
        assert_eq!(
            std::fs::read(path.join("archive.json")).unwrap(),
            b"first\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rename_and_outside_symlink_never_redirects_io() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let parent = temp.path().join("active");
        let path = parent.join("open_1");
        let moved = parent.join("moved-open_1");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("state.json"), b"outside\n").unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();

        std::fs::rename(&path, &moved).unwrap();
        symlink(&outside, &path).unwrap();

        let write_error = active
            .write_atomic(Path::new("state.json"), b"inside\n", 1024)
            .unwrap_err();
        assert_eq!(write_error.operation(), "write");
        assert_eq!(write_error.path(), path);
        let read_error = active
            .read_optional(Path::new("state.json"), 1024)
            .unwrap_err();
        assert_eq!(read_error.operation(), "read");
        assert_eq!(read_error.path(), path);
        assert_eq!(
            std::fs::read(outside.join("state.json")).unwrap(),
            b"outside\n"
        );
        assert!(!moved.join("state.json").exists());
    }

    #[test]
    fn transactional_remove_refuses_a_quarantine_collision() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("active/open_1");
        let quarantine_name = OsStr::new(".codefire-active-remove-collision");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("state.json"), b"inside\n").unwrap();
        std::fs::create_dir(temp.path().join("active").join(quarantine_name)).unwrap();
        std::fs::write(
            temp.path()
                .join("active")
                .join(quarantine_name)
                .join("foreign.txt"),
            b"foreign\n",
        )
        .unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();
        let identity = active.identity().unwrap();
        let parent_identity = active.parent_identity().unwrap();
        drop(active);

        let error = ActiveStateDir::remove_tree_transactional(
            &path,
            quarantine_name,
            identity,
            parent_identity,
        )
        .unwrap_err();

        assert_eq!(error.operation(), "remove tree");
        assert_eq!(std::fs::read(path.join("state.json")).unwrap(), b"inside\n");
        assert_eq!(
            std::fs::read(
                temp.path()
                    .join("active")
                    .join(quarantine_name)
                    .join("foreign.txt")
            )
            .unwrap(),
            b"foreign\n"
        );
    }

    #[test]
    fn partial_remove_is_resumed_from_the_deterministic_quarantine() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("active/open_1");
        let quarantine_name = OsStr::new(".codefire-active-remove-transaction");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("first.json"), b"first\n").unwrap();
        std::fs::write(path.join("second.json"), b"second\n").unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();
        let identity = active.identity().unwrap();
        let parent_identity = active.parent_identity().unwrap();
        drop(active);

        let error = ActiveStateDir::remove_tree_transactional_with(
            &path,
            quarantine_name,
            identity,
            parent_identity,
            |directory| {
                directory.remove_file("first.json")?;
                Err(io::Error::other("injected active-state deletion failure"))
            },
        )
        .unwrap_err();

        assert_eq!(error.operation(), "remove tree");
        assert!(error.to_string().contains("transaction quarantine"));
        assert_eq!(
            ActiveStateDir::removal_progress(&path, quarantine_name, identity, parent_identity,)
                .unwrap(),
            RemovalProgress::Quarantined
        );
        let quarantine = temp.path().join("active").join(quarantine_name);
        assert!(!path.exists());
        assert!(!quarantine.join("first.json").exists());
        assert_eq!(
            std::fs::read(quarantine.join("second.json")).unwrap(),
            b"second\n"
        );

        ActiveStateDir::remove_tree_transactional(
            &path,
            quarantine_name,
            identity,
            parent_identity,
        )
        .unwrap();
        assert_eq!(
            ActiveStateDir::removal_progress(&path, quarantine_name, identity, parent_identity,)
                .unwrap(),
            RemovalProgress::Removed
        );
        assert!(!quarantine.exists());
    }

    #[test]
    fn removed_state_still_requires_the_recorded_parent_identity() {
        let temp = tempdir().unwrap();
        let parent = temp.path().join("active");
        let moved_parent = temp.path().join("moved-active");
        let path = parent.join("open_1");
        let quarantine_name = OsStr::new(".codefire-active-remove-parent-swap");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("state.json"), b"inside\n").unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();
        let identity = active.identity().unwrap();
        let parent_identity = active.parent_identity().unwrap();
        drop(active);

        ActiveStateDir::remove_tree_transactional(
            &path,
            quarantine_name,
            identity,
            parent_identity,
        )
        .unwrap();
        std::fs::rename(&parent, &moved_parent).unwrap();
        std::fs::create_dir(&parent).unwrap();
        std::fs::write(parent.join("outside-sentinel.txt"), b"outside\n").unwrap();

        let error =
            ActiveStateDir::removal_progress(&path, quarantine_name, identity, parent_identity)
                .unwrap_err();

        assert_eq!(error.operation(), "remove tree");
        assert_eq!(error.path(), parent);
        assert_eq!(
            std::fs::read(temp.path().join("active/outside-sentinel.txt")).unwrap(),
            b"outside\n"
        );
        assert!(moved_parent.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn final_file_symlink_is_not_followed() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let path = temp.path().join("active/open_1");
        let outside = temp.path().join("outside.json");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(&outside, b"outside\n").unwrap();
        symlink(&outside, path.join("state.json")).unwrap();
        let active = ActiveStateDir::bind(&path).unwrap();

        assert!(active.read_optional(Path::new("state.json"), 1024).is_err());
        active
            .write_atomic(Path::new("state.json"), b"inside\n", 1024)
            .unwrap();
        assert_eq!(std::fs::read(&outside).unwrap(), b"outside\n");
        assert_eq!(std::fs::read(path.join("state.json")).unwrap(), b"inside\n");
    }
}
