//! Bounded, atomic JSON file reads and writes.

use crate::active_state;
use crate::cli_error::CliError;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) const REPOSITORY_JSON_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

pub(crate) fn read_active_json_optional(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
) -> Result<Option<Value>, CliError> {
    active_state
        .read_optional(relative, REPOSITORY_JSON_BYTE_LIMIT)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(CliError::from))
        .transpose()
}

pub(crate) fn read_active_json_required(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(relative, REPOSITORY_JSON_BYTE_LIMIT)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn write_active_json_atomic(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
    value: &Value,
) -> Result<(), CliError> {
    write_active_json_atomic_bounded(
        active_state,
        relative,
        value,
        REPOSITORY_JSON_BYTE_LIMIT,
        "active-state JSON file",
    )
}

pub(crate) fn write_active_json_atomic_bounded(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
    value: &Value,
    limit: u64,
    context: &str,
) -> Result<(), CliError> {
    let mut rendered = serde_json::to_vec_pretty(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "{context} exceeds the {limit}-byte limit at {}: {} bytes",
            active_state.display_path(relative).display(),
            rendered.len()
        )));
    }
    active_state.write_atomic(relative, &rendered, limit)?;
    Ok(())
}

pub(crate) fn write_json_atomic(path: &Path, value: &Value) -> Result<(), CliError> {
    write_json_atomic_bounded(
        path,
        value,
        REPOSITORY_JSON_BYTE_LIMIT,
        "repository JSON file",
    )
}

pub(crate) fn write_json_atomic_bounded(
    path: &Path,
    value: &Value,
    limit: u64,
    context: &str,
) -> Result<(), CliError> {
    let mut rendered = serde_json::to_vec_pretty(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "{context} exceeds the {limit}-byte limit at {}: {} bytes",
            path.display(),
            rendered.len()
        )));
    }
    write_bytes_atomic(path, &rendered)
}

pub(crate) fn write_bytes_atomic(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::Usage(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let temp_path = unique_temp_path(path);
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
    }
    fs::rename(temp_path, path)?;
    sync_directory_best_effort(parent)?;
    Ok(())
}

pub(crate) fn unique_temp_path(path: &Path) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!("{extension}."))
        .unwrap_or_default();
    path.with_extension(format!("{extension}{}.{}.tmp", std::process::id(), counter))
}

pub(crate) fn sync_directory_best_effort(path: &Path) -> Result<(), CliError> {
    match File::open(path) {
        Ok(dir) => {
            let _ = dir.sync_all();
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(CliError::Io(error)),
    }
}
