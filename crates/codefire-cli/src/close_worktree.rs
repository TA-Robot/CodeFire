use crate::secure_worktree::{
    ensure_same_directory, ensure_stable_identity, open_absolute_dir_nofollow,
    rename_entry_noreplace, RemovalProgress, StableFsIdentity,
};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, OpenOptions};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::Path;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

const OPEN_MARKER_NAME: &str = ".codefire-open";
const OPEN_MARKER_BYTE_LIMIT: u64 = 1024 * 1024;
#[cfg(test)]
const QUARANTINE_NAME_ATTEMPTS: usize = 128;
#[cfg(test)]
static QUARANTINE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// An existing open directory bound to its parent and target identities.
///
/// Close/discard must not delete through an ambient pathname after validating
/// its marker. The target is therefore pinned before the marker is read, and
/// every destructive operation proves that the parent still names that target.
pub(crate) struct ClosableWorktree {
    parent: Dir,
    target: Dir,
    target_name: OsString,
}

pub(crate) struct BoundOpenMarker {
    file: File,
    contents: Vec<u8>,
}

struct QuarantinedWorktree {
    parent: Dir,
    target: Dir,
    original_name: OsString,
    quarantine_name: OsString,
}

enum BoundTreeRemoval {
    Original(ClosableWorktree),
    Quarantined(QuarantinedWorktree),
    Removed,
}

impl BoundOpenMarker {
    pub(crate) fn contents(&self) -> &[u8] {
        &self.contents
    }

    pub(crate) fn identity(&self) -> io::Result<StableFsIdentity> {
        StableFsIdentity::for_file(&self.file)
    }
}

impl ClosableWorktree {
    pub(crate) fn bind(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "close worktree path must be absolute",
            ));
        }
        let parent_path = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "close worktree has no parent")
        })?;
        let target_name = path.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "close worktree has no name")
        })?;
        let parent = open_absolute_dir_nofollow(parent_path)?;
        let target = parent.open_dir_nofollow(target_name)?;
        Ok(Self {
            parent,
            target,
            target_name: target_name.to_os_string(),
        })
    }

    pub(crate) fn read_marker(&self) -> io::Result<BoundOpenMarker> {
        self.ensure_current()?;
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let mut file = self.target.open_with(OPEN_MARKER_NAME, &options)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > OPEN_MARKER_BYTE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "open marker must be a regular file no larger than {OPEN_MARKER_BYTE_LIMIT} bytes"
                ),
            ));
        }
        let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
        (&mut file)
            .take(OPEN_MARKER_BYTE_LIMIT.saturating_add(1))
            .read_to_end(&mut contents)?;
        if contents.len() as u64 > OPEN_MARKER_BYTE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "open marker grew beyond its byte limit",
            ));
        }
        self.ensure_current()?;
        Ok(BoundOpenMarker { file, contents })
    }

    pub(crate) fn identity(&self) -> io::Result<StableFsIdentity> {
        StableFsIdentity::for_directory(&self.target)
    }

    pub(crate) fn parent_identity(&self) -> io::Result<StableFsIdentity> {
        StableFsIdentity::for_directory(&self.parent)
    }

    pub(crate) fn ensure_parent_identity(&self, expected: StableFsIdentity) -> io::Result<()> {
        expected.ensure_directory(&self.parent, "worktree parent")
    }

    pub(crate) fn ensure_identity(&self, expected: StableFsIdentity) -> io::Result<()> {
        ensure_stable_identity(
            expected,
            StableFsIdentity::for_directory(&self.target)?,
            "worktree",
        )
    }

    pub(crate) fn marker_removal_progress(
        &self,
        expected: StableFsIdentity,
        quarantine_name: &OsStr,
    ) -> io::Result<RemovalProgress> {
        validate_entry_name(quarantine_name, "marker quarantine")?;
        self.ensure_current()?;
        let marker =
            open_optional_regular_file_nofollow(&self.target, OsStr::new(OPEN_MARKER_NAME))?;
        let quarantine = open_optional_regular_file_nofollow(&self.target, quarantine_name)?;
        match (marker, quarantine) {
            (Some(marker), None) => {
                expected.ensure_file(&marker, "open marker")?;
                Ok(RemovalProgress::Before)
            }
            (None, Some(quarantine)) => {
                expected.ensure_file(&quarantine, "quarantined open marker")?;
                Ok(RemovalProgress::Quarantined)
            }
            (None, None) => Ok(RemovalProgress::Removed),
            (Some(_), Some(_)) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "both the open marker and its transaction quarantine exist",
            )),
        }
    }

    pub(crate) fn remove_marker_transactional(
        self,
        expected: StableFsIdentity,
        quarantine_name: &OsStr,
    ) -> io::Result<()> {
        match self.marker_removal_progress(expected, quarantine_name)? {
            RemovalProgress::Before => {
                rename_entry_noreplace(
                    &self.target,
                    OsStr::new(OPEN_MARKER_NAME),
                    quarantine_name,
                )?;
                let current = open_regular_file_nofollow(&self.target, quarantine_name)?;
                expected.ensure_file(&current, "quarantined open marker")?;
                self.target.remove_file(quarantine_name)?;
            }
            RemovalProgress::Quarantined => self.target.remove_file(quarantine_name)?,
            RemovalProgress::Removed => return Ok(()),
        }
        sync_directory(&self.target)?;
        if self.marker_removal_progress(expected, quarantine_name)? != RemovalProgress::Removed {
            return Err(io::Error::other(
                "open marker transaction did not reach the removed state",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn remove_marker(self, marker: BoundOpenMarker) -> io::Result<()> {
        self.ensure_current()?;
        let quarantine = move_to_fresh_quarantine(
            &self.target,
            OsStr::new(OPEN_MARKER_NAME),
            ".codefire-close-marker-",
        )?;
        let current = match open_regular_file_nofollow(&self.target, &quarantine) {
            Ok(current) => current,
            Err(error) => {
                return Err(restore_after_identity_failure(
                    &self.target,
                    &quarantine,
                    OsStr::new(OPEN_MARKER_NAME),
                    error,
                ));
            }
        };
        if let Err(identity_error) = ensure_same_file(&marker.file, &current) {
            return Err(restore_after_identity_failure(
                &self.target,
                &quarantine,
                OsStr::new(OPEN_MARKER_NAME),
                identity_error,
            ));
        }
        if let Err(identity_error) = self.ensure_current() {
            return Err(restore_after_identity_failure(
                &self.target,
                &quarantine,
                OsStr::new(OPEN_MARKER_NAME),
                identity_error,
            ));
        }
        if let Err(error) = self.target.remove_file(&quarantine) {
            return Err(restore_after_identity_failure(
                &self.target,
                &quarantine,
                OsStr::new(OPEN_MARKER_NAME),
                error,
            ));
        }
        sync_directory(&self.target)?;
        self.ensure_current()
    }

    #[cfg(test)]
    pub(crate) fn remove_tree(self) -> io::Result<()> {
        self.remove_tree_with(clear_directory_contents_nofollow)
    }

    #[cfg(test)]
    fn remove_tree_with(
        self,
        delete_contents: impl FnOnce(&Dir) -> io::Result<()>,
    ) -> io::Result<()> {
        self.ensure_current()?;
        reject_nested_repository(&self.target)?;
        let quarantine =
            move_to_fresh_quarantine(&self.parent, &self.target_name, ".codefire-close-worktree-")?;
        let moved = match self.parent.open_dir_nofollow(&quarantine) {
            Ok(moved) => moved,
            Err(error) => {
                return Err(restore_after_identity_failure(
                    &self.parent,
                    &quarantine,
                    &self.target_name,
                    error,
                ));
            }
        };
        if let Err(identity_error) = ensure_same_directory(&self.target, &moved) {
            return Err(restore_after_identity_failure(
                &self.parent,
                &quarantine,
                &self.target_name,
                identity_error,
            ));
        }
        if let Err(sync_error) = sync_directory(&self.parent) {
            return Err(restore_after_identity_failure(
                &self.parent,
                &quarantine,
                &self.target_name,
                sync_error,
            ));
        }

        // From here onward deletion may have changed the tree. Never restore a
        // partially deleted worktree to its authoritative name: repository
        // metadata still points to the original path, and the quarantine name
        // in the error is the explicit recovery handle.
        ensure_entry_missing(&self.parent, &self.target_name)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        delete_contents(&self.target)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        ensure_entry_missing(&self.parent, &self.target_name)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        let current = self
            .parent
            .open_dir_nofollow(&quarantine)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        ensure_same_directory(&self.target, &current)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        self.parent
            .remove_dir(&quarantine)
            .map_err(|error| quarantined_delete_error(error, &quarantine))?;
        sync_directory(&self.parent)?;
        ensure_entry_missing(&self.parent, &self.target_name)
    }

    fn ensure_current(&self) -> io::Result<()> {
        let current = self.parent.open_dir_nofollow(&self.target_name)?;
        ensure_same_directory(&self.target, &current)
    }
}

pub(crate) fn tree_removal_progress(
    path: &Path,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
    expected_parent: StableFsIdentity,
) -> io::Result<RemovalProgress> {
    Ok(
        match bind_tree_removal(path, quarantine_name, expected, expected_parent)? {
            BoundTreeRemoval::Original(_) => RemovalProgress::Before,
            BoundTreeRemoval::Quarantined(_) => RemovalProgress::Quarantined,
            BoundTreeRemoval::Removed => RemovalProgress::Removed,
        },
    )
}

pub(crate) fn remove_tree_transactional(
    path: &Path,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
    expected_parent: StableFsIdentity,
) -> io::Result<()> {
    let quarantined = match bind_tree_removal(path, quarantine_name, expected, expected_parent)? {
        BoundTreeRemoval::Original(worktree) => {
            reject_nested_repository(&worktree.target)?;
            rename_entry_noreplace(&worktree.parent, &worktree.target_name, quarantine_name)?;
            sync_directory(&worktree.parent)?;
            let target = worktree.parent.open_dir_nofollow(quarantine_name)?;
            expected.ensure_directory(&target, "quarantined worktree")?;
            QuarantinedWorktree {
                parent: worktree.parent,
                target,
                original_name: worktree.target_name,
                quarantine_name: quarantine_name.to_os_string(),
            }
        }
        BoundTreeRemoval::Quarantined(worktree) => worktree,
        BoundTreeRemoval::Removed => return Ok(()),
    };
    quarantined.remove()?;
    if tree_removal_progress(path, quarantine_name, expected, expected_parent)?
        != RemovalProgress::Removed
    {
        return Err(io::Error::other(
            "worktree transaction did not reach the removed state",
        ));
    }
    Ok(())
}

fn bind_tree_removal(
    path: &Path,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
    expected_parent: StableFsIdentity,
) -> io::Result<BoundTreeRemoval> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "close worktree path must be absolute",
        ));
    }
    validate_entry_name(quarantine_name, "worktree quarantine")?;
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "close worktree has no parent")
    })?;
    let original_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "close worktree has no name"))?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    expected_parent.ensure_directory(&parent, "worktree parent")?;
    match parent.open_dir_nofollow(original_name) {
        Ok(target) => {
            expected.ensure_directory(&target, "close worktree")?;
            match parent.symlink_metadata(quarantine_name) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "both the original worktree and its transaction quarantine exist",
                    ));
                }
            }
            Ok(BoundTreeRemoval::Original(ClosableWorktree {
                parent,
                target,
                target_name: original_name.to_os_string(),
            }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            match parent.open_dir_nofollow(quarantine_name) {
                Ok(target) => {
                    expected.ensure_directory(&target, "quarantined worktree")?;
                    Ok(BoundTreeRemoval::Quarantined(QuarantinedWorktree {
                        parent,
                        target,
                        original_name: original_name.to_os_string(),
                        quarantine_name: quarantine_name.to_os_string(),
                    }))
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    Ok(BoundTreeRemoval::Removed)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

impl QuarantinedWorktree {
    fn remove(self) -> io::Result<()> {
        ensure_entry_missing(&self.parent, &self.original_name)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        let current = self
            .parent
            .open_dir_nofollow(&self.quarantine_name)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        ensure_same_directory(&self.target, &current)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        clear_directory_contents_nofollow(&self.target)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        ensure_entry_missing(&self.parent, &self.original_name)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        let current = self
            .parent
            .open_dir_nofollow(&self.quarantine_name)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        ensure_same_directory(&self.target, &current)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        self.parent
            .remove_dir(&self.quarantine_name)
            .map_err(|error| quarantined_delete_error(error, &self.quarantine_name))?;
        sync_directory(&self.parent)?;
        ensure_entry_missing(&self.parent, &self.original_name)?;
        ensure_entry_missing(&self.parent, &self.quarantine_name)
    }
}

fn open_regular_file_nofollow(directory: &Dir, name: &OsStr) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let file = directory.open_with(name, &options)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "quarantined open marker is not a regular file",
        ));
    }
    Ok(file)
}

fn open_optional_regular_file_nofollow(directory: &Dir, name: &OsStr) -> io::Result<Option<File>> {
    match open_regular_file_nofollow(directory, name) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn validate_entry_name(name: &OsStr, subject: &str) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{subject} must be one normal path component"),
        ));
    }
    Ok(())
}

#[cfg(test)]
fn move_to_fresh_quarantine(directory: &Dir, source: &OsStr, prefix: &str) -> io::Result<OsString> {
    for _ in 0..QUARANTINE_NAME_ATTEMPTS {
        let counter = QUARANTINE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = OsString::from(format!("{prefix}{}-{counter:016x}", std::process::id()));
        match rename_entry_noreplace(directory, source, &candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not reserve a close quarantine name",
    ))
}

fn reject_nested_repository(target: &Dir) -> io::Result<()> {
    match target.symlink_metadata(".codefire") {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "refusing to remove an open directory containing .codefire",
        )),
        Err(error) => Err(error),
    }
}

fn ensure_entry_missing(parent: &Dir, name: &OsStr) -> io::Result<()> {
    match parent.symlink_metadata(name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "close target name changed during quarantined deletion",
        )),
        Err(error) => Err(error),
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

#[cfg(test)]
fn restore_after_identity_failure(
    directory: &Dir,
    quarantine: &OsStr,
    original: &OsStr,
    cause: io::Error,
) -> io::Error {
    let recovery = match directory.symlink_metadata(quarantine) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "quarantined entry disappeared before recovery",
        )),
        Err(error) => Err(error),
        Ok(_) => rename_entry_noreplace(directory, quarantine, original)
            .and_then(|()| sync_directory(directory)),
    };
    match recovery {
        Ok(()) => cause,
        Err(recovery_error) => io::Error::new(
            cause.kind(),
            format!("{cause}; close quarantine recovery failed: {recovery_error}"),
        ),
    }
}

fn quarantined_delete_error(cause: io::Error, quarantine: &OsStr) -> io::Error {
    io::Error::new(
        cause.kind(),
        format!(
            "{cause}; partially deleted close worktree remains in transaction quarantine `{}`; preserve it and rerun the exact close/discard request after resolving any entry at the original name",
            quarantine.to_string_lossy()
        ),
    )
}

fn sync_directory(directory: &Dir) -> io::Result<()> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(all(test, unix))]
fn ensure_same_file(expected: &File, actual: &File) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    let expected = expected.metadata()?;
    let actual = actual.metadata()?;
    if expected.dev() != actual.dev() || expected.ino() != actual.ino() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "open marker identity changed before deletion",
        ));
    }
    Ok(())
}

#[cfg(all(test, windows))]
fn ensure_same_file(expected: &File, actual: &File) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    let expected = expected.metadata()?;
    let actual = actual.metadata()?;
    let expected_identity = (expected.volume_serial_number(), expected.file_index());
    let actual_identity = (actual.volume_serial_number(), actual.file_index());
    if expected_identity.0.is_none()
        || expected_identity.1.is_none()
        || expected_identity != actual_identity
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "open marker identity changed or could not be proven before deletion",
        ));
    }
    Ok(())
}

#[cfg(all(test, not(any(unix, windows))))]
fn ensure_same_file(_expected: &File, _actual: &File) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "open marker identity cannot be proven on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_vendor = "apple",
        target_os = "redox"
    ))]
    #[test]
    fn no_replace_rename_has_exactly_one_winner_under_contention() {
        use std::sync::{Arc, Barrier};

        for round in 0..32 {
            let temp = tempdir().unwrap();
            let first = format!("first-{round}");
            let second = format!("second-{round}");
            std::fs::write(temp.path().join(&first), b"first").unwrap();
            std::fs::write(temp.path().join(&second), b"second").unwrap();
            let barrier = Arc::new(Barrier::new(3));
            let mut workers = Vec::new();
            for source in [first.clone(), second.clone()] {
                let path = temp.path().to_path_buf();
                let barrier = Arc::clone(&barrier);
                workers.push(std::thread::spawn(move || {
                    let directory = open_absolute_dir_nofollow(&path).unwrap();
                    barrier.wait();
                    rename_entry_noreplace(
                        &directory,
                        OsStr::new(&source),
                        OsStr::new("quarantine"),
                    )
                }));
            }
            barrier.wait();
            let results = workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>();

            assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
            assert_eq!(
                results
                    .iter()
                    .filter(|result| matches!(result, Err(error) if error.kind() == io::ErrorKind::AlreadyExists))
                    .count(),
                1
            );
            let published = std::fs::read(temp.path().join("quarantine")).unwrap();
            match published.as_slice() {
                b"first" => {
                    assert!(!temp.path().join(&first).exists());
                    assert_eq!(std::fs::read(temp.path().join(&second)).unwrap(), b"second");
                }
                b"second" => {
                    assert!(!temp.path().join(&second).exists());
                    assert_eq!(std::fs::read(temp.path().join(&first)).unwrap(), b"first");
                }
                other => panic!("unexpected quarantine contents: {other:?}"),
            }
        }
    }

    #[test]
    fn keep_directory_removes_only_the_bound_marker() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(target.join("user.txt"), b"preserved\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        let marker = worktree.read_marker().unwrap();

        worktree.remove_marker(marker).unwrap();

        assert!(!target.join(OPEN_MARKER_NAME).exists());
        assert_eq!(
            std::fs::read_to_string(target.join("user.txt")).unwrap(),
            "preserved\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn recursive_delete_removes_symlinks_without_following_them() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let outside = temp.path().join("outside");
        std::fs::create_dir(&target).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(outside.join("victim.txt"), b"outside\n").unwrap();
        std::os::unix::fs::symlink(&outside, target.join("escape")).unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();

        worktree.remove_tree().unwrap();

        assert!(!target.exists());
        assert_eq!(
            std::fs::read_to_string(outside.join("victim.txt")).unwrap(),
            "outside\n"
        );
    }

    #[test]
    fn partial_delete_failure_stays_quarantined_and_can_be_recovered() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(target.join("deleted-before-fault.txt"), b"delete me\n").unwrap();
        std::fs::write(target.join("preserved-after-fault.txt"), b"preserve me\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();

        let error = worktree
            .remove_tree_with(|directory| {
                directory.remove_file("deleted-before-fault.txt")?;
                Err(io::Error::other("injected recursive delete failure"))
            })
            .unwrap_err();

        assert!(!target.exists());
        let quarantines = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".codefire-close-worktree-")
            })
            .collect::<Vec<_>>();
        assert_eq!(quarantines.len(), 1);
        let quarantine_name = quarantines[0].file_name();
        let quarantine = quarantines[0].path();
        assert!(error.to_string().contains("partially deleted"));
        assert!(error
            .to_string()
            .contains(&quarantine_name.to_string_lossy().to_string()));
        assert!(!quarantine.join("deleted-before-fault.txt").exists());
        assert_eq!(
            std::fs::read_to_string(quarantine.join("preserved-after-fault.txt")).unwrap(),
            "preserve me\n"
        );
        assert!(quarantine.join(OPEN_MARKER_NAME).exists());

        std::fs::rename(&quarantine, &target).unwrap();
        ClosableWorktree::bind(&target)
            .unwrap()
            .remove_tree()
            .unwrap();
        assert!(!target.exists());
    }

    #[test]
    fn deterministic_tree_quarantine_is_resumed_after_a_crash() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let quarantine_name = OsStr::new(".codefire-close-worktree-transaction");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(target.join("user.txt"), b"discard\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        let identity = worktree.identity().unwrap();
        let parent_identity = worktree.parent_identity().unwrap();
        drop(worktree);
        std::fs::rename(&target, temp.path().join(quarantine_name)).unwrap();

        assert_eq!(
            tree_removal_progress(&target, quarantine_name, identity, parent_identity).unwrap(),
            RemovalProgress::Quarantined
        );
        remove_tree_transactional(&target, quarantine_name, identity, parent_identity).unwrap();
        assert_eq!(
            tree_removal_progress(&target, quarantine_name, identity, parent_identity).unwrap(),
            RemovalProgress::Removed
        );
        assert!(!temp.path().join(quarantine_name).exists());
    }

    #[test]
    fn transactional_tree_removal_rejects_parent_replacement() {
        let temp = tempdir().unwrap();
        let parent = temp.path().join("workspace");
        let moved_parent = temp.path().join("moved-workspace");
        let target = parent.join("open");
        let quarantine_name = OsStr::new(".codefire-close-worktree-transaction");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(target.join("user.txt"), b"preserved\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        let identity = worktree.identity().unwrap();
        let parent_identity = worktree.parent_identity().unwrap();
        drop(worktree);
        std::fs::rename(&parent, &moved_parent).unwrap();
        std::fs::create_dir(&parent).unwrap();

        assert!(
            tree_removal_progress(&target, quarantine_name, identity, parent_identity).is_err()
        );
        assert_eq!(
            std::fs::read_to_string(moved_parent.join("open/user.txt")).unwrap(),
            "preserved\n"
        );
    }

    #[test]
    fn deterministic_marker_quarantine_is_resumed_after_a_crash() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let quarantine_name = OsStr::new(".codefire-close-marker-transaction");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        let marker = worktree.read_marker().unwrap();
        let marker_identity = marker.identity().unwrap();
        drop(marker);
        drop(worktree);
        std::fs::rename(target.join(OPEN_MARKER_NAME), target.join(quarantine_name)).unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();

        assert_eq!(
            worktree
                .marker_removal_progress(marker_identity, quarantine_name)
                .unwrap(),
            RemovalProgress::Quarantined
        );
        worktree
            .remove_marker_transactional(marker_identity, quarantine_name)
            .unwrap();
        assert!(!target.join(OPEN_MARKER_NAME).exists());
        assert!(!target.join(quarantine_name).exists());
    }

    #[test]
    fn target_swap_is_restored_without_deleting_the_replacement() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let moved = temp.path().join("moved-open");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"{}\n").unwrap();
        std::fs::write(target.join("original.txt"), b"original\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        std::fs::rename(&target, &moved).unwrap();
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("victim.txt"), b"victim\n").unwrap();

        assert!(worktree.remove_tree().is_err());

        assert_eq!(
            std::fs::read_to_string(target.join("victim.txt")).unwrap(),
            "victim\n"
        );
        assert_eq!(
            std::fs::read_to_string(moved.join("original.txt")).unwrap(),
            "original\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn marker_symlink_is_rejected_without_following_it() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let outside = temp.path().join("outside-marker");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(&outside, b"{}\n").unwrap();
        std::os::unix::fs::symlink(&outside, target.join(OPEN_MARKER_NAME)).unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();

        assert!(worktree.read_marker().is_err());
        assert_eq!(std::fs::read_to_string(outside).unwrap(), "{}\n");
    }

    #[test]
    fn marker_swap_is_rejected_and_the_replacement_is_restored() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"original\n").unwrap();
        let worktree = ClosableWorktree::bind(&target).unwrap();
        let marker = worktree.read_marker().unwrap();
        std::fs::rename(
            target.join(OPEN_MARKER_NAME),
            target.join("original-marker"),
        )
        .unwrap();
        std::fs::write(target.join(OPEN_MARKER_NAME), b"replacement\n").unwrap();

        assert!(worktree.remove_marker(marker).is_err());

        assert_eq!(
            std::fs::read_to_string(target.join(OPEN_MARKER_NAME)).unwrap(),
            "replacement\n"
        );
        assert_eq!(
            std::fs::read_to_string(target.join("original-marker")).unwrap(),
            "original\n"
        );
    }

    #[test]
    fn oversized_marker_is_rejected_before_allocation_or_parse() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        std::fs::create_dir(&target).unwrap();
        let marker = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target.join(OPEN_MARKER_NAME))
            .unwrap();
        marker.set_len(OPEN_MARKER_BYTE_LIMIT + 1).unwrap();
        drop(marker);
        let worktree = ClosableWorktree::bind(&target).unwrap();

        let error = worktree.read_marker().err().unwrap();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[cfg(unix)]
    #[test]
    fn marker_fifo_is_rejected_without_blocking() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        std::fs::create_dir(&target).unwrap();
        let status = std::process::Command::new("mkfifo")
            .arg(target.join(OPEN_MARKER_NAME))
            .status()
            .unwrap();
        assert!(status.success());
        let worktree = ClosableWorktree::bind(&target).unwrap();

        let error = worktree.read_marker().err().unwrap();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
