//! Content manifest construction and comparison for worktrees.

use crate::cli_error::CliError;
use crate::commit::{
    planned_commit_object, CommitObjectStage, ManifestCommitPlan, PlannedCommitObject,
};
use crate::secure_worktree::ExistingWorktree;
use crate::{encode_base64, required_string, secure_worktree};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) fn manifest_blob_payload(data: &[u8]) -> Value {
    json!({
        "type": "blob",
        "version": 1,
        "encoding": "base64",
        "content": encode_base64(data),
    })
}

pub(crate) fn build_manifest_plan(open_dir: &Path) -> Result<ManifestCommitPlan, CliError> {
    let mut entries = Vec::new();
    let mut blobs = Vec::new();
    for rel in rel_files(open_dir)? {
        let data = fs::read(open_dir.join(&rel))?;
        let path = to_posix(&rel);
        let blob_payload = manifest_blob_payload(&data);
        let blob = PlannedCommitObject {
            role: "manifest_blob",
            path: Some(path.clone()),
            object_type: "blob",
            object_id: codefire_store::object_id("blob", &blob_payload)?,
            payload: None,
            worktree_source: Some(rel),
            stage: CommitObjectStage::ManifestBlob,
        };
        entries.push(json!({
            "path": path,
            "kind": "file",
            "mode": "100644",
            "blob": &blob.object_id,
        }));
        blobs.push(blob);
    }
    let manifest = planned_commit_object(
        "content_manifest",
        None,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": entries}),
        CommitObjectStage::SealedRoot,
    )?;
    Ok(ManifestCommitPlan { blobs, manifest })
}

pub(crate) fn source_manifest_unchanged(
    objects: &Path,
    base_commit: &str,
    open_dir: &Path,
) -> Result<bool, CliError> {
    let validated = codefire_store::validate_sealed_commit_pinned(objects, base_commit)?;
    source_manifest_unchanged_validated(&validated, open_dir)
}

pub(crate) fn source_manifest_unchanged_validated(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &Path,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint(open_dir)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

pub(crate) fn source_manifest_unchanged_existing(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &ExistingWorktree,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint_from_files(open_dir.read_manifest_files()?)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

pub(crate) fn source_manifest_unchanged_secure(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &secure_worktree::SecureWorktree,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint_from_files(open_dir.read_manifest_files()?)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

pub(crate) fn manifest_file_changes_since_base(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &Path,
) -> Result<Vec<codefire_core::ManifestFileChange>, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint(open_dir)?;
    let base_files = manifest_blob_map(&base_manifest)?;
    let current_files = manifest_blob_map(&current_manifest)?;
    let paths = base_files
        .keys()
        .chain(current_files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    for path in paths {
        let base_blob = base_files.get(&path).cloned();
        let current_blob = current_files.get(&path).cloned();
        if base_blob == current_blob {
            continue;
        }
        let status = match (base_blob.as_ref(), current_blob.as_ref()) {
            (None, Some(_)) => "added",
            (Some(_), None) => "deleted",
            (Some(_), Some(_)) => "modified",
            (None, None) => continue,
        };
        changes.push(codefire_core::ManifestFileChange {
            path,
            status: status.to_string(),
            base_blob,
            current_blob,
        });
    }
    Ok(changes)
}

pub(crate) fn manifest_blob_map(manifest: &Value) -> Result<BTreeMap<String, String>, CliError> {
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    let mut files = BTreeMap::new();
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let path = required_string(entry, &["path"])?;
        let blob = required_string(entry, &["blob"])?;
        files.insert(path, blob);
    }
    Ok(files)
}

pub(crate) fn build_manifest_fingerprint(open_dir: &Path) -> Result<Value, CliError> {
    let files = rel_files(open_dir)?
        .into_iter()
        .map(|relative| {
            let data = fs::read(open_dir.join(&relative))?;
            Ok((relative, data))
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    build_manifest_fingerprint_from_files(files)
}

pub(crate) fn build_manifest_fingerprint_from_files(
    files: Vec<(PathBuf, Vec<u8>)>,
) -> Result<Value, CliError> {
    let mut entries = Vec::with_capacity(files.len());
    for (rel, data) in files {
        let blob_payload = json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(&data),
        });
        let blob_id = codefire_store::object_id("blob", &blob_payload)?;
        entries.push(json!({
            "path": to_posix(&rel),
            "kind": "file",
            "mode": "100644",
            "blob": blob_id,
        }));
    }
    Ok(json!({"type": "content_manifest", "version": 1, "entries": entries}))
}

pub(crate) fn rel_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    let mut files = Vec::new();
    collect_rel_files(root, Path::new(""), &mut files)?;
    files.sort_by_key(|path| to_posix(path));
    Ok(files)
}

pub(crate) fn collect_rel_files(
    root: &Path,
    rel_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), CliError> {
    let mut entries = fs::read_dir(root.join(rel_dir))?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rel = rel_dir.join(name.as_ref());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if matches!(name.as_ref(), ".codefire" | "__pycache__" | ".pytest_cache") {
                continue;
            }
            collect_rel_files(root, &rel, files)?;
        } else if file_type.is_file() {
            let rel_path = to_posix(&rel);
            let import_operational_temp = rel.components().count() == 1
                && ((rel_path.starts_with(".codefire-import.lock.bootstrap-")
                    && (rel_path.ends_with(".tmp") || rel_path.ends_with(".part")))
                    || (rel_path.starts_with(".codefire-import.write-")
                        && rel_path.ends_with(".tmp")));
            if !import_operational_temp
                && !matches!(
                    rel_path.as_str(),
                    ".codefire-open"
                        | ".codefire-import.lock"
                        | ".codefire-import-transaction.json"
                )
            {
                files.push(rel);
            }
        }
    }
    Ok(())
}

pub(crate) fn to_posix(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn safe_relative_manifest_path(rel_path: &str) -> Result<PathBuf, CliError> {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        return Err(CliError::InvalidRepository(format!(
            "manifest path must be relative: {rel_path}"
        )));
    }
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => output.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::InvalidRepository(format!(
                    "manifest path escapes target: {rel_path}"
                )));
            }
        }
    }
    Ok(output)
}
