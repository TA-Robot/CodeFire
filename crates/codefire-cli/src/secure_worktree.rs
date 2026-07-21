use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::Mutex;

const STAGING_OWNER_BYTE_LIMIT: u64 = 16 * 1024;
const EXISTING_WORKTREE_FILE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;
const EXISTING_WORKTREE_TOTAL_BYTE_LIMIT: u64 = 1024 * 1024 * 1024;
const EXISTING_WORKTREE_ENTRY_LIMIT: u64 = 100_000;
const ATOMIC_TEMP_CREATE_ATTEMPTS: usize = 128;
static STAGING_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
#[derive(Debug)]
struct ManifestFileSwapInjection {
    display_root: PathBuf,
    relative: PathBuf,
    moved_original: PathBuf,
    replacement: PathBuf,
}

#[cfg(test)]
static MANIFEST_FILE_SWAP: Mutex<Option<ManifestFileSwapInjection>> = Mutex::new(None);

#[cfg(test)]
pub(crate) fn set_manifest_file_swap_before_rebind_for_test(
    display_root: &Path,
    relative: &Path,
    moved_original: &Path,
    replacement: &Path,
) {
    *MANIFEST_FILE_SWAP
        .lock()
        .expect("manifest file swap mutex poisoned") = Some(ManifestFileSwapInjection {
        display_root: display_root.to_path_buf(),
        relative: relative.to_path_buf(),
        moved_original: moved_original.to_path_buf(),
        replacement: replacement.to_path_buf(),
    });
}

#[cfg(test)]
fn maybe_swap_manifest_file_before_rebind_for_test(
    display_root: &Path,
    relative: &Path,
) -> io::Result<()> {
    let injection = {
        let mut guard = MANIFEST_FILE_SWAP
            .lock()
            .expect("manifest file swap mutex poisoned");
        if guard.as_ref().is_some_and(|injection| {
            injection.display_root == display_root && injection.relative == relative
        }) {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    let original = injection.display_root.join(&injection.relative);
    std::fs::rename(&original, &injection.moved_original)?;
    if let Err(error) = std::fs::rename(&injection.replacement, &original) {
        let _ = std::fs::rename(&injection.moved_original, &original);
        return Err(error);
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_manifest_file_before_rebind_for_test(
    _display_root: &Path,
    _relative: &Path,
) -> io::Result<()> {
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StableFsIdentity {
    scheme: &'static str,
    first: u64,
    second: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StableFilePathIdentity {
    parent: StableFsIdentity,
    file: StableFsIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemovalProgress {
    Before,
    Quarantined,
    Removed,
}

impl StableFsIdentity {
    pub(crate) fn for_directory(directory: &Dir) -> io::Result<Self> {
        stable_directory_identity(directory)
    }

    pub(crate) fn for_file(file: &cap_std::fs::File) -> io::Result<Self> {
        stable_file_identity(file)
    }

    pub(crate) fn ensure_directory(self, directory: &Dir, subject: &str) -> io::Result<()> {
        ensure_stable_identity(self, stable_directory_identity(directory)?, subject)
    }

    pub(crate) fn ensure_file(self, file: &cap_std::fs::File, subject: &str) -> io::Result<()> {
        ensure_stable_identity(self, stable_file_identity(file)?, subject)
    }

    pub(crate) fn from_parts(scheme: &str, first: u64, second: u64) -> io::Result<Self> {
        if scheme != platform_identity_scheme()? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("filesystem identity scheme `{scheme}` does not match this platform"),
            ));
        }
        Ok(Self {
            scheme: platform_identity_scheme()?,
            first,
            second,
        })
    }

    pub(crate) fn scheme(self) -> &'static str {
        self.scheme
    }

    pub(crate) fn first(self) -> u64 {
        self.first
    }

    pub(crate) fn second(self) -> u64 {
        self.second
    }
}

/// A directory reached component-by-component from a pinned workspace boundary.
///
/// Every opened directory rejects a symlink in its final component. File writes
/// are relative to the pinned target handle, so a concurrent ambient rename or
/// symlink replacement cannot redirect materialization outside this subject.
pub(crate) struct SecureWorktree {
    boundary: Dir,
    target: Dir,
    relative_target: PathBuf,
    display_path: PathBuf,
}

pub(crate) struct StagedWorktree {
    boundary: Dir,
    parent: Dir,
    stage: Dir,
    relative_target: PathBuf,
    target_name: OsString,
    stage_name: OsString,
    owner_name: OsString,
    existing_target: Option<Dir>,
    display_target: PathBuf,
    published: bool,
}

/// A target resolution pinned to every real directory that existed when the
/// open target was resolved. Components that were absent must be created with
/// create-new semantics; an appearing replacement is never adopted.
pub(crate) struct PinnedTargetResolution {
    boundary: Dir,
    boundary_path: PathBuf,
    target_path: PathBuf,
    relative_target: PathBuf,
    components: Vec<OsString>,
    existing: Vec<Dir>,
}

impl PinnedTargetResolution {
    pub(crate) fn resolve(boundary: &Path, requested: &Path) -> io::Result<Self> {
        let absolute = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            std::env::current_dir()?.join(requested)
        };
        let mut normalized = PathBuf::new();
        for component in absolute.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "path traversal with '..' is not allowed for open targets: {}",
                            requested.display()
                        ),
                    ))
                }
                Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                    normalized.push(component.as_os_str());
                }
            }
        }
        Self::bind(boundary, &normalized)
    }

    pub(crate) fn bind(boundary: &Path, target: &Path) -> io::Result<Self> {
        let relative_target = target.strip_prefix(boundary).map_err(|_| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "target {} is outside workspace boundary {}",
                    target.display(),
                    boundary.display()
                ),
            )
        })?;
        let components = normal_components(relative_target)?;
        if components.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "open target must be below the workspace boundary",
            ));
        }
        let boundary_handle = open_absolute_dir_nofollow(boundary)?;
        let mut current = boundary_handle.try_clone()?;
        let mut existing = vec![boundary_handle.try_clone()?];
        for component in &components {
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                    current = current.open_dir_nofollow(component)?;
                    existing.push(current.try_clone()?);
                }
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "symlink components are not allowed for open targets: {}",
                            component.to_string_lossy()
                        ),
                    ))
                }
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "resolved target component is not a real directory: {}",
                            component.to_string_lossy()
                        ),
                    ))
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => break,
                Err(error) => return Err(error),
            }
        }
        let resolution = Self {
            boundary: boundary_handle,
            boundary_path: boundary.to_path_buf(),
            target_path: target.to_path_buf(),
            relative_target: relative_target.to_path_buf(),
            components,
            existing,
        };
        resolution.ensure_current()?;
        Ok(resolution)
    }

    pub(crate) fn ensure_current(&self) -> io::Result<()> {
        ensure_absolute_dir_identity(&self.boundary, &self.boundary_path)?;
        let mut current = self.boundary.try_clone()?;
        for (index, component) in self
            .components
            .iter()
            .take(self.existing.len().saturating_sub(1))
            .enumerate()
        {
            current = current.open_dir_nofollow(component)?;
            ensure_same_directory(&self.existing[index + 1], &current)?;
        }
        Ok(())
    }

    pub(crate) fn identity_chain(&self) -> io::Result<Vec<(PathBuf, StableFsIdentity)>> {
        self.ensure_current()?;
        let mut relative = PathBuf::new();
        let mut identities = vec![(
            relative.clone(),
            StableFsIdentity::for_directory(&self.existing[0])?,
        )];
        for (index, component) in self
            .components
            .iter()
            .take(self.existing.len().saturating_sub(1))
            .enumerate()
        {
            relative.push(component);
            identities.push((
                relative.clone(),
                StableFsIdentity::for_directory(&self.existing[index + 1])?,
            ));
        }
        Ok(identities)
    }

    pub(crate) fn ensure_relative_identity(
        &self,
        relative: &Path,
        expected: StableFsIdentity,
    ) -> io::Result<()> {
        let directory = open_relative_dir_chain_nofollow(&self.boundary, relative)?;
        expected.ensure_directory(&directory, "recorded open target ancestor")
    }

    pub(crate) fn target_path(&self) -> &Path {
        &self.target_path
    }
}

/// An already-open worktree pinned to a no-follow directory capability.
///
/// Merge and patch application must not resolve output paths through ambient
/// filesystem APIs: an existing parent symlink could otherwise redirect a
/// write or delete outside the open directory. All mutations below this handle
/// walk normal relative components and open every parent without following its
/// final component.
pub(crate) struct ExistingWorktree {
    root: Dir,
    path: PathBuf,
}

impl ExistingWorktree {
    pub(crate) fn bind(path: &Path) -> io::Result<Self> {
        Ok(Self {
            root: open_absolute_dir_nofollow(path)?,
            path: path.to_path_buf(),
        })
    }

    /// Reads the source files through the pinned worktree capability.
    ///
    /// Directory and file components are opened without following symlinks,
    /// reads are bounded, and every child name is rebound after use. Callers
    /// must additionally call `ensure_path_identity` after consuming the
    /// snapshot so an ambient replacement of the worktree name fails closed.
    pub(crate) fn read_manifest_files(&self) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
        read_manifest_files_bounded(&self.root, &self.path)
    }

    pub(crate) fn ensure_path_identity(&self) -> io::Result<()> {
        ensure_absolute_dir_identity(&self.root, &self.path)
    }

    pub(crate) fn read_file_optional_bounded(
        &self,
        relative: &Path,
        limit: u64,
    ) -> io::Result<Option<Vec<u8>>> {
        let mut components = normal_components(relative)?;
        let file_name = components.pop().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "file path must not be empty")
        })?;
        let mut parent = self.root.try_clone()?;
        for component in &components {
            parent = match parent.open_dir_nofollow(component) {
                Ok(directory) => directory,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error),
            };
        }
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        options.nonblock(true);
        let file = match parent.open_with(&file_name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "worktree conflict file is not a bounded regular file (size {}, limit {limit})",
                    metadata.len()
                ),
            ));
        }
        let identity = StableFsIdentity::for_file(&file)?;
        let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
        file.take(limit + 1).read_to_end(&mut contents)?;
        if contents.len() as u64 > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("worktree conflict file grew beyond {limit} bytes"),
            ));
        }
        let rebound = parent.open_with(&file_name, &options)?;
        identity.ensure_file(&rebound, "worktree conflict file")?;
        Ok(Some(contents))
    }

    pub(crate) fn entry_exists_nofollow(&self, relative: &Path) -> io::Result<bool> {
        let mut components = normal_components(relative)?;
        let entry_name = components.pop().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "entry path must not be empty")
        })?;
        let mut parent = self.root.try_clone()?;
        for component in &components {
            parent = match parent.open_dir_nofollow(component) {
                Ok(directory) => directory,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(error),
            };
        }
        match parent.symlink_metadata(&entry_name) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn write_file_atomic(&self, relative: &Path, contents: &[u8]) -> io::Result<()> {
        if u64::try_from(contents.len()).unwrap_or(u64::MAX) > EXISTING_WORKTREE_FILE_BYTE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "worktree file exceeds the {} byte write limit",
                    EXISTING_WORKTREE_FILE_BYTE_LIMIT
                ),
            ));
        }

        let mut components = normal_components(relative)?;
        let file_name = components.pop().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "file path must not be empty")
        })?;
        let mut parent = self.root.try_clone()?;
        for component in &components {
            parent = open_or_create_dir_nofollow(&parent, component)?;
        }
        write_relative_file_atomic_replace(&parent, &file_name, contents)
    }

    pub(crate) fn delete_file(&self, relative: &Path) -> io::Result<()> {
        let mut components = normal_components(relative)?;
        let file_name = components.pop().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "file path must not be empty")
        })?;
        let mut parent = self.root.try_clone()?;
        for component in &components {
            parent = match parent.open_dir_nofollow(component) {
                Ok(directory) => directory,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error),
            };
        }

        match parent.symlink_metadata(&file_name) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "worktree delete target is a directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        }
        parent.remove_file(&file_name)?;
        sync_capability_directory_best_effort(&parent)
    }
}

impl SecureWorktree {
    #[cfg(test)]
    pub(crate) fn prepare(boundary: &Path, target: &Path) -> io::Result<Self> {
        let relative_target = target.strip_prefix(boundary).map_err(|_| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "target {} is outside workspace boundary {}",
                    target.display(),
                    boundary.display()
                ),
            )
        })?;
        let components = normal_components(relative_target)?;
        if components.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "open target must be below the workspace boundary",
            ));
        }

        let boundary = open_absolute_dir_nofollow(boundary)?;
        let mut current = boundary.try_clone()?;
        for component in &components {
            current = open_or_create_dir_nofollow(&current, component)?;
        }
        if current.entries()?.next().transpose()?.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "open target must be empty",
            ));
        }
        Ok(Self {
            boundary,
            target: current,
            relative_target: relative_target.to_path_buf(),
            display_path: target.to_path_buf(),
        })
    }

    #[cfg(test)]
    pub(crate) fn write_file(&self, relative: &Path, contents: &[u8]) -> io::Result<()> {
        write_file_below(&self.target, relative, contents)
    }

    pub(crate) fn ensure_path_identity(&self) -> io::Result<()> {
        let current = self.boundary.open_dir_nofollow(&self.relative_target)?;
        ensure_same_directory(&self.target, &current)
    }

    pub(crate) fn read_manifest_files(&self) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
        read_manifest_files_bounded(&self.target, &self.display_path)
    }
}

impl StagedWorktree {
    #[cfg(test)]
    pub(crate) fn prepare(
        boundary: &Path,
        target: &Path,
        stage_name: &OsStr,
        owner_contents: &[u8],
    ) -> io::Result<Self> {
        let resolution = PinnedTargetResolution::bind(boundary, target)?;
        Self::prepare_pinned(&resolution, stage_name, owner_contents)
    }

    pub(crate) fn prepare_pinned(
        resolution: &PinnedTargetResolution,
        stage_name: &OsStr,
        owner_contents: &[u8],
    ) -> io::Result<Self> {
        if owner_contents.len() as u64 > STAGING_OWNER_BYTE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "open staging owner metadata exceeds its byte limit",
            ));
        }
        resolution.ensure_current()?;
        let relative_target = &resolution.relative_target;
        let mut components = resolution.components.clone();
        let target_name = components.pop().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "open target must be below the workspace boundary",
            )
        })?;
        let stage_components = normal_components(Path::new(stage_name))?;
        if stage_components.len() != 1 || stage_components[0] == target_name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "open staging name must be one distinct path component",
            ));
        }
        let stage_name = stage_components[0].clone();
        let mut owner_name = stage_name.clone();
        owner_name.push(".owner.json");

        let boundary_handle = resolution.boundary.try_clone()?;
        let existing_count = resolution.existing.len().saturating_sub(1);
        let parent_component_count = components.len();
        let mut parent =
            resolution.existing[existing_count.min(parent_component_count)].try_clone()?;
        if existing_count < parent_component_count {
            for component in components.iter().skip(existing_count) {
                parent.create_dir(component)?;
                parent = parent.open_dir_nofollow(component)?;
            }
        }
        let existing_target = if existing_count == parent_component_count + 1 {
            let expected = resolution.existing[existing_count].try_clone()?;
            let current =
                open_empty_target_if_present(&parent, &target_name)?.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "resolved empty open target disappeared before staging",
                    )
                })?;
            ensure_same_directory(&expected, &current)?;
            Some(expected)
        } else {
            match parent.symlink_metadata(&target_name) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "open target appeared after target resolution",
                    ))
                }
                Err(error) => return Err(error),
            }
        };

        let owner_exists = match parent.symlink_metadata(&owner_name) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "open staging owner is not a regular no-follow file",
                    ));
                }
                let existing =
                    read_relative_file_bounded(&parent, &owner_name, STAGING_OWNER_BYTE_LIMIT)?;
                if existing != owner_contents {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "open staging owner belongs to a different operation",
                    ));
                }
                true
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };
        match parent.symlink_metadata(&stage_name) {
            Ok(_) if owner_exists => remove_directory_tree_nofollow(&parent, &stage_name)?,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "open staging directory exists without matching provenance",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if !owner_exists {
            write_relative_file_atomic(&parent, &owner_name, owner_contents)?;
        }
        parent.create_dir(&stage_name)?;
        let stage = parent.open_dir_nofollow(&stage_name)?;
        sync_capability_directory_best_effort(&parent)?;
        Ok(Self {
            boundary: boundary_handle,
            parent,
            stage,
            relative_target: relative_target.to_path_buf(),
            target_name,
            stage_name,
            owner_name,
            existing_target,
            display_target: resolution.target_path().to_path_buf(),
            published: false,
        })
    }

    pub(crate) fn write_file(&self, relative: &Path, contents: &[u8]) -> io::Result<()> {
        write_file_below(&self.stage, relative, contents)
    }

    pub(crate) fn publication_parent_identity(&self) -> io::Result<StableFsIdentity> {
        StableFsIdentity::for_directory(&self.parent)
    }

    pub(crate) fn relative_publication_parent(&self) -> PathBuf {
        self.relative_target
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf()
    }

    pub(crate) fn publish(mut self) -> io::Result<SecureWorktree> {
        // Prepare every fallible handle clone before the visibility point. Once
        // the rename succeeds, callers must receive the published worktree even
        // if best-effort durability or owner cleanup is unavailable.
        let published_boundary = self.boundary.try_clone()?;
        let published_target = self.stage.try_clone()?;
        let current_parent = open_relative_dir_chain_nofollow(
            &self.boundary,
            self.relative_target
                .parent()
                .unwrap_or_else(|| Path::new("")),
        )?;
        ensure_same_directory(&self.parent, &current_parent)?;
        let current_stage = self.parent.open_dir_nofollow(&self.stage_name)?;
        ensure_same_directory(&self.stage, &current_stage)?;
        match &self.existing_target {
            Some(expected) => {
                let current = open_empty_target_if_present(&self.parent, &self.target_name)?
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "existing open target disappeared before publication",
                        )
                    })?;
                ensure_same_directory(expected, &current)?;
            }
            None => match self.parent.symlink_metadata(&self.target_name) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "open target appeared before staged publication",
                    ))
                }
            },
        }

        // On platforms where rename cannot atomically replace an existing empty
        // directory, fail without removing it. A remove-then-rename fallback
        // would expose an interruption window and a name-based replacement race.
        self.parent
            .rename(&self.stage_name, &self.parent, &self.target_name)?;
        let current_target = self.parent.open_dir_nofollow(&self.target_name)?;
        ensure_same_directory(&self.stage, &current_target)?;
        self.published = true;
        let _ = self.parent.remove_file(&self.owner_name);
        let _ = sync_capability_directory_best_effort(&self.parent);
        Ok(SecureWorktree {
            boundary: published_boundary,
            target: published_target,
            relative_target: self.relative_target.clone(),
            display_path: self.display_target.clone(),
        })
    }
}

impl Drop for StagedWorktree {
    fn drop(&mut self) {
        if self.published {
            return;
        }
        if remove_directory_tree_nofollow(&self.parent, &self.stage_name).is_ok() {
            let _ = self.parent.remove_file(&self.owner_name);
            let _ = sync_capability_directory_best_effort(&self.parent);
        }
    }
}

fn write_file_below(root: &Dir, relative: &Path, contents: &[u8]) -> io::Result<()> {
    let mut components = normal_components(relative)?;
    let file_name = components.pop().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "file path must not be empty")
    })?;
    let mut current = root.try_clone()?;
    for component in &components {
        current = open_or_create_dir_nofollow(&current, component)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = current.open_with(file_name, &options)?;
    file.write_all(contents)?;
    file.sync_all()
}

fn open_empty_target_if_present(parent: &Dir, name: &OsStr) -> io::Result<Option<Dir>> {
    match parent.symlink_metadata(name) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "open target is not a real directory",
                ));
            }
            let directory = parent.open_dir_nofollow(name)?;
            if directory.entries()?.next().transpose()?.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "open target must be empty",
                ));
            }
            Ok(Some(directory))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn read_relative_file_bounded(parent: &Dir, name: &OsStr, limit: u64) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let file = parent.open_with(name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bounded staging metadata reader rejected the file",
        ));
    }
    ensure_single_link_file(&metadata, Path::new(name))?;
    let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
    file.take(limit.saturating_add(1))
        .read_to_end(&mut contents)?;
    if contents.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "staging metadata grew beyond its byte limit",
        ));
    }
    Ok(contents)
}

#[derive(Default)]
struct WorktreeReadBudget {
    entries: u64,
    bytes: u64,
}

fn read_manifest_files_bounded(
    root: &Dir,
    display_root: &Path,
) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let identity = StableFsIdentity::for_directory(root)?;
    let mut files = Vec::new();
    let mut budget = WorktreeReadBudget::default();
    collect_manifest_files_bounded(root, display_root, Path::new(""), &mut files, &mut budget)?;
    identity.ensure_directory(root, "pinned worktree")?;
    files.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(files)
}

fn collect_manifest_files_bounded(
    directory: &Dir,
    display_root: &Path,
    relative_directory: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
    budget: &mut WorktreeReadBudget,
) -> io::Result<()> {
    let mut names = directory
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();

    for name in names {
        budget.entries = budget.entries.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "worktree entry count overflow")
        })?;
        if budget.entries > EXISTING_WORKTREE_ENTRY_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("worktree exceeds the {EXISTING_WORKTREE_ENTRY_LIMIT} entry read limit"),
            ));
        }

        let display_name = name.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "worktree contains a non-UTF-8 entry name",
            )
        })?;
        let relative = relative_directory.join(display_name);
        let metadata = directory.symlink_metadata(&name)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "worktree manifest traversal rejects symlink entry {}",
                    relative.display()
                ),
            ));
        }
        if metadata.is_dir() {
            if matches!(display_name, ".codefire" | "__pycache__" | ".pytest_cache") {
                continue;
            }
            let child = directory.open_dir_nofollow(&name)?;
            let child_identity = StableFsIdentity::for_directory(&child)?;
            collect_manifest_files_bounded(&child, display_root, &relative, files, budget)?;
            let rebound = directory.open_dir_nofollow(&name)?;
            child_identity.ensure_directory(&rebound, "worktree child directory")?;
        } else if metadata.is_file() {
            let relative_path = relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let import_operational_temp = relative.components().count() == 1
                && ((relative_path.starts_with(".codefire-import.lock.bootstrap-")
                    && (relative_path.ends_with(".tmp") || relative_path.ends_with(".part")))
                    || (relative_path.starts_with(".codefire-import.write-")
                        && relative_path.ends_with(".tmp")));
            if import_operational_temp
                || matches!(
                    relative_path.as_str(),
                    ".codefire-open"
                        | ".codefire-import.lock"
                        | ".codefire-import-transaction.json"
                )
            {
                continue;
            }

            let data =
                read_manifest_file_bounded(directory, display_root, &relative, &name, budget)?;
            files.push((relative, data));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "worktree manifest traversal rejects non-regular entry {}",
                    relative.display()
                ),
            ));
        }
    }
    Ok(())
}

fn read_manifest_file_bounded(
    parent: &Dir,
    display_root: &Path,
    relative: &Path,
    name: &OsStr,
    budget: &mut WorktreeReadBudget,
) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let mut file = parent.open_with(name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > EXISTING_WORKTREE_FILE_BYTE_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "worktree file exceeds the {EXISTING_WORKTREE_FILE_BYTE_LIMIT} byte read limit"
            ),
        ));
    }
    let next_total = budget.bytes.checked_add(metadata.len()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "worktree byte count overflow")
    })?;
    if next_total > EXISTING_WORKTREE_TOTAL_BYTE_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "worktree exceeds the {EXISTING_WORKTREE_TOTAL_BYTE_LIMIT} byte total read limit"
            ),
        ));
    }
    let identity = StableFsIdentity::for_file(&file)?;
    let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
    std::io::Read::by_ref(&mut file)
        .take(EXISTING_WORKTREE_FILE_BYTE_LIMIT.saturating_add(1))
        .read_to_end(&mut contents)?;
    if contents.len() as u64 > EXISTING_WORKTREE_FILE_BYTE_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worktree file grew beyond its bounded read limit",
        ));
    }
    identity.ensure_file(&file, "worktree source file")?;
    maybe_swap_manifest_file_before_rebind_for_test(display_root, relative)?;
    let rebound = open_regular_file_nofollow(parent, name)?;
    identity.ensure_file(&rebound, "worktree source file")?;
    budget.bytes = budget
        .bytes
        .checked_add(contents.len() as u64)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "worktree byte count overflow")
        })?;
    if budget.bytes > EXISTING_WORKTREE_TOTAL_BYTE_LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worktree grew beyond its bounded total read limit",
        ));
    }
    Ok(contents)
}

fn write_relative_file_atomic(parent: &Dir, name: &OsStr, contents: &[u8]) -> io::Result<()> {
    let counter = STAGING_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut temp_name = OsString::from(".");
    temp_name.push(name);
    temp_name.push(format!(".{}.{}.tmp", std::process::id(), counter));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(&temp_name, &options)?;
        file.write_all(contents)?;
        file.sync_all()?;
        // hard_link is an atomic no-replace publication primitive. Unlike
        // rename, it cannot overwrite a foreign owner created after preflight.
        parent.hard_link(&temp_name, parent, name)?;
        drop(file);
        parent.remove_file(&temp_name)?;
        sync_capability_directory_best_effort(parent)
    })();
    if result.is_err() {
        let _ = parent.remove_file(&temp_name);
    }
    result
}

fn write_relative_file_atomic_replace(
    parent: &Dir,
    name: &OsStr,
    contents: &[u8],
) -> io::Result<()> {
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worktree write target is a directory",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let (mut file, temp_name) = {
        let mut opened = None;
        for _ in 0..ATOMIC_TEMP_CREATE_ATTEMPTS {
            let counter = STAGING_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let temp_name = OsString::from(format!(
                ".codefire-worktree-write.{}.{}.tmp",
                std::process::id(),
                counter
            ));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            options.follow(FollowSymlinks::No);
            match parent.open_with(&temp_name, &options) {
                Ok(file) => {
                    opened = Some((file, temp_name));
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        opened.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "could not allocate a unique atomic worktree temp file",
            )
        })?
    };

    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        parent.rename(&temp_name, parent, name)?;
        sync_capability_directory_best_effort(parent)
    })();
    if result.is_err() {
        let _ = parent.remove_file(&temp_name);
    }
    result
}

fn remove_directory_tree_nofollow(parent: &Dir, name: &OsStr) -> io::Result<()> {
    let directory = parent.open_dir_nofollow(name)?;
    let mut names = directory
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    for child_name in names {
        let metadata = directory.symlink_metadata(&child_name)?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            remove_directory_tree_nofollow(&directory, &child_name)?;
        } else {
            directory.remove_file(&child_name)?;
        }
    }
    let current = parent.open_dir_nofollow(name)?;
    ensure_same_directory(&directory, &current)?;
    parent.remove_dir(name)
}

pub(crate) fn sync_capability_directory_best_effort(directory: &Dir) -> io::Result<()> {
    match directory.open(".")?.sync_all() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LockFileIdentity {
    first: u64,
    second: u64,
}

pub(crate) fn open_lock_file_nofollow(
    path: &Path,
) -> io::Result<(std::fs::File, LockFileIdentity)> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "lock file path has no parent")
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "lock file path has no name"))?;
    let parent = open_absolute_dir_nofollow(parent)?;
    open_lock_file_in_directory_nofollow(&parent, file_name, path)
}

pub(crate) fn open_lock_file_in_directory_nofollow(
    parent: &Dir,
    file_name: &OsStr,
    display_path: &Path,
) -> io::Result<(std::fs::File, LockFileIdentity)> {
    match parent.symlink_metadata(file_name) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "lock path is not a regular no-follow file: {}",
                    display_path.display()
                ),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    options.follow(FollowSymlinks::No);
    let file = parent.open_with(file_name, &options)?;
    let identity = validated_lock_file_identity(&file.metadata()?, display_path)?;
    Ok((file.into_std(), identity))
}

pub(crate) fn ensure_lock_file_identity_nofollow(
    path: &Path,
    expected: LockFileIdentity,
) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "lock file path has no parent")
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "lock file path has no name"))?;
    let parent = open_absolute_dir_nofollow(parent)?;
    ensure_lock_file_identity_in_directory_nofollow(&parent, file_name, path, expected)
}

pub(crate) fn ensure_lock_file_identity_in_directory_nofollow(
    parent: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    expected: LockFileIdentity,
) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    options.follow(FollowSymlinks::No);
    let file = parent.open_with(file_name, &options)?;
    let actual = validated_lock_file_identity(&file.metadata()?, display_path)?;
    if actual != expected {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "repository lock identity changed: {}",
                display_path.display()
            ),
        ));
    }
    Ok(())
}

pub(crate) fn write_file_atomic_nofollow(
    path: &Path,
    temp_path: &Path,
    contents: &[u8],
) -> io::Result<()> {
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic file path has no parent",
        )
    })?;
    if temp_path.parent() != Some(parent_path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic file and temp path must share a parent",
        ));
    }
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "atomic file path has no name")
    })?;
    let temp_name = temp_path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "atomic temp path has no name")
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    write_file_atomic_in_directory_nofollow(&parent, file_name, temp_name, contents)
}

pub(crate) fn write_file_atomic_in_directory_nofollow(
    parent: &Dir,
    file_name: &OsStr,
    temp_name: &OsStr,
    contents: &[u8],
) -> io::Result<()> {
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(temp_name, &options)?;
        file.write_all(contents)?;
        file.sync_all()?;
        parent.rename(temp_name, parent, file_name)?;
        match parent.open(".")?.sync_all() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
            Err(error) => Err(error),
        }
    })();
    if result.is_err() {
        let _ = parent.remove_file(temp_name);
    }
    result
}

pub(crate) fn write_file_create_noreplace_nofollow(
    path: &Path,
    temp_path: &Path,
    contents: &[u8],
) -> io::Result<()> {
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "no-replace file path has no parent",
        )
    })?;
    if temp_path.parent() != Some(parent_path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no-replace file and temp path must share a parent",
        ));
    }
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "no-replace path has no name")
    })?;
    let temp_name = temp_path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "no-replace temp path has no name",
        )
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    write_file_create_noreplace_in_directory_nofollow(&parent, file_name, temp_name, contents)
}

pub(crate) fn write_file_create_noreplace_in_directory_nofollow(
    parent: &Dir,
    file_name: &OsStr,
    temp_name: &OsStr,
    contents: &[u8],
) -> io::Result<()> {
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        let mut file = parent.open_with(temp_name, &options)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        rename_entry_noreplace(parent, temp_name, file_name)?;
        sync_capability_directory_best_effort(parent)
    })();
    if result.is_err() {
        let _ = parent.remove_file(temp_name);
    }
    result
}

pub(crate) fn read_file_nofollow_bounded(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    Ok(read_file_nofollow_bounded_with_identity(path, limit)?.0)
}

pub(crate) fn ensure_file_nofollow_path_identity(
    path: &Path,
    identity: StableFilePathIdentity,
) -> io::Result<()> {
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "file identity path has no parent",
        )
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "file identity path has no name",
        )
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    identity
        .parent
        .ensure_directory(&parent, "bounded no-follow file parent")?;
    ensure_absolute_dir_identity(&parent, parent_path)?;
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let file = parent.open_with(file_name, &options)?;
    identity
        .file
        .ensure_file(&file, "bounded no-follow file path")?;
    identity
        .parent
        .ensure_directory(&parent, "bounded no-follow file parent")?;
    ensure_absolute_dir_identity(&parent, parent_path)
}

pub(crate) fn read_file_in_directory_nofollow_bounded(
    parent: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    limit: u64,
) -> io::Result<Vec<u8>> {
    Ok(read_file_in_directory_nofollow_bounded_with_identity(
        parent,
        file_name,
        display_path,
        limit,
    )?
    .0)
}

pub(crate) fn read_file_in_directory_nofollow_bounded_with_identity(
    parent: &Dir,
    file_name: &OsStr,
    display_path: &Path,
    limit: u64,
) -> io::Result<(Vec<u8>, StableFsIdentity)> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let mut file = parent.open_with(file_name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "bounded no-follow reader rejected {} (size {}, limit {limit})",
                display_path.display(),
                metadata.len()
            ),
        ));
    }
    ensure_single_link_file(&metadata, display_path)?;
    let identity = StableFsIdentity::for_file(&file)?;
    let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
    (&mut file).take(limit + 1).read_to_end(&mut contents)?;
    if contents.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "bounded file grew beyond {limit} bytes: {}",
                display_path.display()
            ),
        ));
    }
    identity.ensure_file(&file, "bounded no-follow file")?;
    Ok((contents, identity))
}

pub(crate) fn read_file_nofollow_bounded_with_identity(
    path: &Path,
    limit: u64,
) -> io::Result<(Vec<u8>, StableFsIdentity)> {
    let (contents, identity) = read_file_nofollow_bounded_with_path_identity(path, limit)?;
    Ok((contents, identity.file))
}

pub(crate) fn read_file_nofollow_bounded_with_path_identity(
    path: &Path,
    limit: u64,
) -> io::Result<(Vec<u8>, StableFilePathIdentity)> {
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "bounded file path has no parent",
        )
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "bounded file path has no name")
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    ensure_absolute_dir_identity(&parent, parent_path)?;
    let parent_identity = StableFsIdentity::for_directory(&parent)?;
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let mut file = parent.open_with(file_name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "bounded no-follow reader rejected {} (size {}, limit {limit})",
                path.display(),
                metadata.len()
            ),
        ));
    }
    ensure_single_link_file(&metadata, path)?;
    let identity = StableFsIdentity::for_file(&file)?;
    let mut contents = Vec::with_capacity(metadata.len() as usize + 1);
    (&mut file).take(limit + 1).read_to_end(&mut contents)?;
    if contents.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bounded file grew beyond {limit} bytes: {}", path.display()),
        ));
    }
    identity.ensure_file(&file, "bounded no-follow file")?;
    let final_metadata = file.metadata()?;
    if final_metadata.len() != metadata.len() || contents.len() as u64 != metadata.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "bounded file changed size while reading: {}",
                path.display()
            ),
        ));
    }
    parent_identity.ensure_directory(&parent, "bounded no-follow file parent")?;
    ensure_absolute_dir_identity(&parent, parent_path)?;
    Ok((
        contents,
        StableFilePathIdentity {
            parent: parent_identity,
            file: identity,
        },
    ))
}

pub(crate) fn file_identity_nofollow(path: &Path) -> io::Result<(StableFsIdentity, u64)> {
    let parent_path = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "file path has no parent"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "file path has no name"))?;
    let parent = open_absolute_dir_nofollow(parent_path)?;
    let file = open_regular_file_nofollow(&parent, file_name)?;
    let metadata = file.metadata()?;
    ensure_single_link_file(&metadata, path)?;
    Ok((StableFsIdentity::for_file(&file)?, metadata.len()))
}

pub(crate) fn remove_file_quarantined_nofollow(
    path: &Path,
    quarantine_path: &Path,
    expected: StableFsIdentity,
) -> io::Result<()> {
    let parent_path = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "remove path has no parent"))?;
    if quarantine_path.parent() != Some(parent_path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "remove path and quarantine must share a parent",
        ));
    }
    let source_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "remove path has no name"))?;
    let quarantine_name = quarantine_path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "remove quarantine has no name")
    })?;
    let parent = open_absolute_dir_nofollow(parent_path)?;

    remove_file_quarantined_in_directory_nofollow(&parent, source_name, quarantine_name, expected)
}

pub(crate) fn remove_file_quarantined_in_directory_nofollow(
    parent: &Dir,
    source_name: &OsStr,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
) -> io::Result<()> {
    if source_name != quarantine_name {
        let source = open_regular_file_nofollow(parent, source_name)?;
        expected.ensure_file(&source, "open transaction marker")?;
        rename_entry_noreplace(parent, source_name, quarantine_name)?;
    }
    let quarantined = open_regular_file_nofollow(parent, quarantine_name)?;
    expected.ensure_file(&quarantined, "quarantined open transaction marker")?;
    parent.remove_file(quarantine_name)?;
    sync_capability_directory_best_effort(parent)
}

pub(crate) fn remove_directory_quarantined_in_directory_nofollow(
    parent: &Dir,
    source_name: &OsStr,
    quarantine_name: &OsStr,
    expected: StableFsIdentity,
) -> io::Result<()> {
    if source_name != quarantine_name {
        let source = parent.open_dir_nofollow(source_name)?;
        expected.ensure_directory(&source, "remote transaction directory")?;
        rename_entry_noreplace(parent, source_name, quarantine_name)?;
    }
    let quarantined = parent.open_dir_nofollow(quarantine_name)?;
    expected.ensure_directory(&quarantined, "quarantined remote transaction directory")?;
    if quarantined.entries()?.next().transpose()?.is_some() {
        return Err(io::Error::other(
            "quarantined remote transaction directory is not empty",
        ));
    }
    parent.remove_dir(quarantine_name)?;
    sync_capability_directory_best_effort(parent)
}

fn open_regular_file_nofollow(parent: &Dir, name: &OsStr) -> io::Result<cap_std::fs::File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let file = parent.open_with(name, &options)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "filesystem entry is not a regular no-follow file",
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn ensure_single_link_file(metadata: &cap_std::fs::Metadata, path: &Path) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    if metadata.nlink() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "bounded authority must have exactly one filesystem link: {}",
                path.display()
            ),
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn ensure_single_link_file(metadata: &cap_std::fs::Metadata, path: &Path) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    if metadata.number_of_links() != Some(1) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "bounded authority must have one provable filesystem link: {}",
                path.display()
            ),
        ));
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn ensure_single_link_file(_metadata: &cap_std::fs::Metadata, path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "bounded authority link count cannot be proven: {}",
            path.display()
        ),
    ))
}

#[cfg(unix)]
fn validated_lock_file_identity(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> io::Result<LockFileIdentity> {
    use cap_std::fs::MetadataExt;

    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "repository lock must be a regular file with exactly one link: {}",
                path.display()
            ),
        ));
    }
    Ok(LockFileIdentity {
        first: metadata.dev(),
        second: metadata.ino(),
    })
}

#[cfg(windows)]
fn validated_lock_file_identity(
    metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> io::Result<LockFileIdentity> {
    use cap_std::fs::MetadataExt;

    if !metadata.is_file() || metadata.number_of_links() != Some(1) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "repository lock must be a regular file with one provable link: {}",
                path.display()
            ),
        ));
    }
    match (metadata.volume_serial_number(), metadata.file_index()) {
        (Some(volume), Some(index)) => Ok(LockFileIdentity {
            first: u64::from(volume),
            second: index,
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "repository lock identity is unavailable: {}",
                path.display()
            ),
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn validated_lock_file_identity(
    _metadata: &cap_std::fs::Metadata,
    path: &Path,
) -> io::Result<LockFileIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "repository lock link provenance is unavailable: {}",
            path.display()
        ),
    ))
}

pub(crate) fn open_absolute_dir_nofollow(path: &Path) -> io::Result<Dir> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "secure directory boundary must be absolute",
        ));
    }
    let mut root = PathBuf::new();
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => root.push(component.as_os_str()),
            Component::Normal(value) => components.push(value.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "secure directory boundary contains parent traversal",
                ));
            }
        }
    }
    let mut current = Dir::open_ambient_dir(&root, ambient_authority())?;
    for component in components {
        current = current.open_dir_nofollow(component)?;
    }
    Ok(current)
}

pub(crate) fn ensure_absolute_dir_identity(expected: &Dir, path: &Path) -> io::Result<()> {
    let current = open_absolute_dir_nofollow(path)?;
    ensure_same_directory(expected, &current)
}

fn open_or_create_dir_nofollow(parent: &Dir, name: &OsStr) -> io::Result<Dir> {
    match parent.symlink_metadata(name) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "path component is not a real directory: {}",
                        name.to_string_lossy()
                    ),
                ));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => parent.create_dir(name)?,
        Err(error) => return Err(error),
    }
    parent.open_dir_nofollow(name)
}

fn open_relative_dir_chain_nofollow(root: &Dir, relative: &Path) -> io::Result<Dir> {
    let mut current = root.try_clone()?;
    for component in normal_components(relative)? {
        current = current.open_dir_nofollow(component)?;
    }
    Ok(current)
}

fn normal_components(path: &Path) -> io::Result<Vec<OsString>> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => Ok(value.to_os_string()),
            Component::CurDir => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "current-directory components are not allowed",
            )),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "path escapes secure directory",
                ))
            }
        })
        .collect()
}

pub(crate) fn ensure_stable_identity(
    expected: StableFsIdentity,
    actual: StableFsIdentity,
    subject: &str,
) -> io::Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{subject} identity changed"),
        ))
    }
}

#[cfg(unix)]
fn platform_identity_scheme() -> io::Result<&'static str> {
    Ok("unix_dev_inode")
}

#[cfg(windows)]
fn platform_identity_scheme() -> io::Result<&'static str> {
    Ok("windows_volume_file_index")
}

#[cfg(not(any(unix, windows)))]
fn platform_identity_scheme() -> io::Result<&'static str> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "stable filesystem identity is unavailable on this platform",
    ))
}

#[cfg(unix)]
fn stable_directory_identity(directory: &Dir) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "filesystem capability is not a directory",
        ));
    }
    Ok(StableFsIdentity {
        scheme: "unix_dev_inode",
        first: metadata.dev(),
        second: metadata.ino(),
    })
}

#[cfg(windows)]
fn stable_directory_identity(directory: &Dir) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    match (metadata.volume_serial_number(), metadata.file_index()) {
        (Some(volume), Some(index)) if metadata.is_dir() => Ok(StableFsIdentity {
            scheme: "windows_volume_file_index",
            first: u64::from(volume),
            second: index,
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "directory identity cannot be proven",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn stable_directory_identity(_directory: &Dir) -> io::Result<StableFsIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "directory identity cannot be proven on this platform",
    ))
}

#[cfg(unix)]
fn stable_file_identity(file: &cap_std::fs::File) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "filesystem capability is not a regular file",
        ));
    }
    Ok(StableFsIdentity {
        scheme: "unix_dev_inode",
        first: metadata.dev(),
        second: metadata.ino(),
    })
}

#[cfg(windows)]
fn stable_file_identity(file: &cap_std::fs::File) -> io::Result<StableFsIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    match (metadata.volume_serial_number(), metadata.file_index()) {
        (Some(volume), Some(index)) if metadata.is_file() => Ok(StableFsIdentity {
            scheme: "windows_volume_file_index",
            first: u64::from(volume),
            second: index,
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "file identity cannot be proven",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn stable_file_identity(_file: &cap_std::fs::File) -> io::Result<StableFsIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "file identity cannot be proven on this platform",
    ))
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
))]
pub(crate) fn rename_entry_noreplace(
    directory: &Dir,
    source: &OsStr,
    target: &OsStr,
) -> io::Result<()> {
    Ok(rustix::fs::renameat_with(
        directory,
        source,
        directory,
        target,
        rustix::fs::RenameFlags::NOREPLACE,
    )?)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
)))]
pub(crate) fn rename_entry_noreplace(
    _directory: &Dir,
    _source: &OsStr,
    _target: &OsStr,
) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic no-replace rename is unavailable on this platform",
    ))
}

#[cfg(unix)]
pub(crate) fn ensure_same_directory(expected: &Dir, actual: &Dir) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    let expected = expected.dir_metadata()?;
    let actual = actual.dir_metadata()?;
    if expected.dev() != actual.dev() || expected.ino() != actual.ino() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "open target identity changed during materialization",
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn ensure_same_directory(expected: &Dir, actual: &Dir) -> io::Result<()> {
    use cap_std::fs::MetadataExt;

    let expected = expected.dir_metadata()?;
    let actual = actual.dir_metadata()?;
    let expected_identity = (expected.volume_serial_number(), expected.file_index());
    let actual_identity = (actual.volume_serial_number(), actual.file_index());
    if expected_identity.0.is_none()
        || expected_identity.1.is_none()
        || expected_identity != actual_identity
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "open target identity changed or could not be proven during materialization",
        ));
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn ensure_same_directory(_expected: &Dir, _actual: &Dir) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "open target identity cannot be proven on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const STAGE_NAME: &str = ".codefire-open-stage-test";
    const OWNER_CONTENTS: &[u8] = b"{\"type\":\"test-open-stage\",\"version\":1}\n";

    fn owner_path(boundary: &Path) -> PathBuf {
        boundary.join(format!("{STAGE_NAME}.owner.json"))
    }

    #[test]
    fn failed_staging_is_removed_without_creating_the_target() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        std::fs::create_dir_all(&boundary).unwrap();

        {
            let staged =
                StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                    .unwrap();
            staged
                .write_file(Path::new("docs/readme.md"), b"partial\n")
                .unwrap();
        }

        assert!(!target.exists());
        assert!(!boundary.join(STAGE_NAME).exists());
        assert!(!owner_path(&boundary).exists());
    }

    #[test]
    fn matching_owner_recovers_an_abandoned_stage_before_retry() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        std::fs::create_dir_all(&boundary).unwrap();

        let abandoned =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .unwrap();
        abandoned
            .write_file(Path::new("docs/readme.md"), b"stale\n")
            .unwrap();
        std::mem::forget(abandoned);

        let retry =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .unwrap();
        retry
            .write_file(Path::new("docs/readme.md"), b"retry\n")
            .unwrap();
        let published = retry.publish().unwrap();

        assert_eq!(
            std::fs::read_to_string(target.join("docs/readme.md")).unwrap(),
            "retry\n"
        );
        assert!(!owner_path(&boundary).exists());
        published.ensure_path_identity().unwrap();
    }

    #[test]
    fn foreign_stage_without_owner_is_left_untouched() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        let foreign = boundary.join(STAGE_NAME);
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(foreign.join("foreign.txt"), b"owned elsewhere\n").unwrap();

        let error =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .err()
                .unwrap();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            std::fs::read_to_string(foreign.join("foreign.txt")).unwrap(),
            "owned elsewhere\n"
        );
        assert!(!owner_path(&boundary).exists());
    }

    #[test]
    fn publication_rejects_a_target_that_appears_and_preserves_it() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        std::fs::create_dir_all(&boundary).unwrap();
        let staged =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .unwrap();
        staged
            .write_file(Path::new("docs/readme.md"), b"staged\n")
            .unwrap();
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("foreign.txt"), b"foreign\n").unwrap();

        let error = staged.publish().err().unwrap();

        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read_to_string(target.join("foreign.txt")).unwrap(),
            "foreign\n"
        );
        assert!(!boundary.join(STAGE_NAME).exists());
        assert!(!owner_path(&boundary).exists());
    }

    #[test]
    fn publication_moves_a_complete_stage_into_an_absent_target() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        std::fs::create_dir_all(&boundary).unwrap();
        let staged =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .unwrap();
        staged
            .write_file(Path::new("docs/readme.md"), b"complete\n")
            .unwrap();

        let published = staged.publish().unwrap();

        assert_eq!(
            std::fs::read_to_string(target.join("docs/readme.md")).unwrap(),
            "complete\n"
        );
        assert!(!boundary.join(STAGE_NAME).exists());
        assert!(!owner_path(&boundary).exists());
        published.ensure_path_identity().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn publication_atomically_replaces_the_same_empty_target() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        std::fs::create_dir_all(&target).unwrap();
        let staged =
            StagedWorktree::prepare(&boundary, &target, OsStr::new(STAGE_NAME), OWNER_CONTENTS)
                .unwrap();
        staged
            .write_file(Path::new("docs/readme.md"), b"complete\n")
            .unwrap();

        let published = staged.publish().unwrap();

        assert_eq!(
            std::fs::read_to_string(target.join("docs/readme.md")).unwrap(),
            "complete\n"
        );
        published.ensure_path_identity().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn pinned_target_cannot_be_redirected_by_symlink_swap() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        let moved = boundary.join("moved-target");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&boundary).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let secure = SecureWorktree::prepare(&boundary, &target).unwrap();

        std::fs::rename(&target, &moved).unwrap();
        std::os::unix::fs::symlink(&outside, &target).unwrap();
        secure
            .write_file(Path::new("docs/readme.md"), b"safe\n")
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(moved.join("docs/readme.md")).unwrap(),
            "safe\n"
        );
        assert!(!outside.join("docs/readme.md").exists());
        assert!(secure.ensure_path_identity().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn nested_symlink_swap_is_rejected_before_file_write() {
        let temp = tempdir().unwrap();
        let boundary = temp.path().join("workspace");
        let target = boundary.join("target");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&boundary).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let secure = SecureWorktree::prepare(&boundary, &target).unwrap();
        std::os::unix::fs::symlink(&outside, target.join("docs")).unwrap();

        assert!(secure
            .write_file(Path::new("docs/readme.md"), b"blocked\n")
            .is_err());
        assert!(!outside.join("readme.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn existing_worktree_mutations_stay_on_the_pinned_root_after_path_swap() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let moved = temp.path().join("moved-open");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(target.join("delete.txt"), b"inside\n").unwrap();
        std::fs::write(target.join("conflict.txt"), b"inside conflict\n").unwrap();
        std::fs::write(target.join("recorded-side"), b"inside side\n").unwrap();
        std::fs::write(outside.join("delete.txt"), b"outside\n").unwrap();
        std::fs::write(outside.join("conflict.txt"), b"outside conflict\n").unwrap();
        std::fs::write(outside.join("outside-only"), b"outside only\n").unwrap();
        let bound = ExistingWorktree::bind(&target).unwrap();

        std::fs::rename(&target, &moved).unwrap();
        std::os::unix::fs::symlink(&outside, &target).unwrap();

        assert_eq!(
            bound
                .read_file_optional_bounded(Path::new("conflict.txt"), 1024)
                .unwrap(),
            Some(b"inside conflict\n".to_vec())
        );
        assert!(bound
            .entry_exists_nofollow(Path::new("recorded-side"))
            .unwrap());
        assert!(!bound
            .entry_exists_nofollow(Path::new("outside-only"))
            .unwrap());

        bound
            .write_file_atomic(Path::new("docs/readme.md"), b"pinned\n")
            .unwrap();
        bound.delete_file(Path::new("delete.txt")).unwrap();

        assert_eq!(
            std::fs::read_to_string(moved.join("docs/readme.md")).unwrap(),
            "pinned\n"
        );
        assert!(!moved.join("delete.txt").exists());
        assert!(!outside.join("docs/readme.md").exists());
        assert_eq!(
            std::fs::read_to_string(outside.join("delete.txt")).unwrap(),
            "outside\n"
        );
        assert!(bound.ensure_path_identity().is_err());
        assert!(ExistingWorktree::bind(&target).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn existing_worktree_conflict_helpers_do_not_follow_final_symlinks() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("open");
        let outside = temp.path().join("outside.txt");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(&outside, b"outside\n").unwrap();
        std::os::unix::fs::symlink(&outside, target.join("conflict-link")).unwrap();
        std::os::unix::fs::symlink(&outside, target.join("side-link")).unwrap();
        let bound = ExistingWorktree::bind(&target).unwrap();

        assert!(bound
            .read_file_optional_bounded(Path::new("conflict-link"), 1024)
            .is_err());
        assert!(bound.entry_exists_nofollow(Path::new("side-link")).unwrap());
        assert_eq!(std::fs::read(&outside).unwrap(), b"outside\n");
    }
}
