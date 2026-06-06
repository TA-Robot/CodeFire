use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const OBJECT_ID_DIGEST_HEX_LENGTH: usize = 12;

const OBJECT_KINDS: &[ObjectKind] = &[
    ObjectKind::new("blob", "CF-BLOB", "blobs"),
    ObjectKind::new("content_manifest", "CF-MANIFEST", "content_manifests"),
    ObjectKind::new("atom_index", "CF-ATOMINDEX", "atom_indexes"),
    ObjectKind::new("trace_graph", "CF-TRACE", "trace_graphs"),
    ObjectKind::new("fire_ledger", "CF-FIRELEDGER", "fire_ledgers"),
    ObjectKind::new("resolution_ledger", "CF-RESOLUTION", "resolution_ledgers"),
    ObjectKind::new("verification", "CF-VERIFY", "verifications"),
    ObjectKind::new("policy", "CF-POLICY", "policies"),
    ObjectKind::new("artifact_ref", "CF-ARTIFACT", "artifact_refs"),
    ObjectKind::new("evidence", "CF-EVIDENCE", "evidence"),
    ObjectKind::new("commit", "CF-COMMIT", "commits"),
    ObjectKind::new("branch", "CF-BRANCH", "branches"),
];

#[derive(Debug, Clone, Copy)]
struct ObjectKind {
    type_tag: &'static str,
    prefix: &'static str,
    subdir: &'static str,
}

impl ObjectKind {
    const fn new(type_tag: &'static str, prefix: &'static str, subdir: &'static str) -> Self {
        Self {
            type_tag,
            prefix,
            subdir,
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Json(serde_json::Error),
    Io(std::io::Error),
    UnknownObjectType(String),
    IncompleteRecord(String),
    ObjectIdMismatch { expected: String, actual: String },
    ObjectHashMismatch { expected: String, actual: String },
    FilenameMismatch { expected: String, actual: String },
    ObjectNotFound(String),
    InvalidSealedCommit(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Json(error) => write!(f, "json error: {error}"),
            StoreError::Io(error) => write!(f, "io error: {error}"),
            StoreError::UnknownObjectType(type_tag) => write!(f, "unknown object type: {type_tag}"),
            StoreError::IncompleteRecord(context) => {
                write!(f, "object record is incomplete: {context}")
            }
            StoreError::ObjectIdMismatch { expected, actual } => {
                write!(f, "object id mismatch: expected {expected}, got {actual}")
            }
            StoreError::ObjectHashMismatch { expected, actual } => {
                write!(f, "object hash mismatch: expected {expected}, got {actual}")
            }
            StoreError::FilenameMismatch { expected, actual } => {
                write!(
                    f,
                    "object filename/id mismatch: expected {expected}, got {actual}"
                )
            }
            StoreError::ObjectNotFound(object_id) => write!(f, "object not found: {object_id}"),
            StoreError::InvalidSealedCommit(message) => {
                write!(f, "sealed commit validation failed: {message}")
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        StoreError::Json(error)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        StoreError::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRecord {
    pub object_id: String,
    #[serde(rename = "type")]
    pub type_tag: String,
    pub hash: String,
    pub payload: Value,
}

pub fn canonical_json(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(value)
}

pub fn object_digest(type_tag: &str, payload: &Value) -> Result<String, serde_json::Error> {
    let mut hasher = Sha256::new();
    hasher.update(type_tag.as_bytes());
    hasher.update([0]);
    hasher.update(canonical_json(payload)?);
    Ok(hex_lower(&hasher.finalize()))
}

pub fn object_id(type_tag: &str, payload: &Value) -> Result<String, serde_json::Error> {
    let digest = object_digest(type_tag, payload)?;
    Ok(format!(
        "{}-{}",
        object_prefix(type_tag),
        &digest[..OBJECT_ID_DIGEST_HEX_LENGTH]
    ))
}

pub fn object_prefix(type_tag: &str) -> &'static str {
    object_kind_by_type(type_tag)
        .map(|kind| kind.prefix)
        .unwrap_or("CF-OBJECT")
}

pub fn object_subdir(type_tag: &str) -> Option<&'static str> {
    object_kind_by_type(type_tag).map(|kind| kind.subdir)
}

pub fn known_object_subdirs() -> impl Iterator<Item = &'static str> {
    OBJECT_KINDS.iter().map(|kind| kind.subdir)
}

fn object_kind_by_type(type_tag: &str) -> Option<ObjectKind> {
    OBJECT_KINDS
        .iter()
        .copied()
        .find(|kind| kind.type_tag == type_tag)
}

fn object_subdir_by_id(object_id: &str) -> Option<&'static str> {
    OBJECT_KINDS
        .iter()
        .filter(|kind| object_id.starts_with(kind.prefix))
        .max_by_key(|kind| kind.prefix.len())
        .map(|kind| kind.subdir)
}

pub fn object_record(type_tag: &str, payload: Value) -> Result<ObjectRecord, StoreError> {
    Ok(ObjectRecord {
        object_id: object_id(type_tag, &payload)?,
        type_tag: type_tag.to_string(),
        hash: object_digest(type_tag, &payload)?,
        payload,
    })
}

pub fn validate_object_record(
    record: &ObjectRecord,
    expected_object_id: Option<&str>,
    path: Option<&Path>,
) -> Result<(), StoreError> {
    if record.type_tag.is_empty() || record.payload.is_null() {
        return Err(StoreError::IncompleteRecord(
            path.map(|value| value.display().to_string())
                .or_else(|| expected_object_id.map(str::to_string))
                .unwrap_or_else(|| "(unknown)".to_string()),
        ));
    }

    let computed_id = object_id(&record.type_tag, &record.payload)?;
    let computed_hash = object_digest(&record.type_tag, &record.payload)?;
    if record.object_id != computed_id {
        return Err(StoreError::ObjectIdMismatch {
            expected: computed_id,
            actual: record.object_id.clone(),
        });
    }
    if record.hash != computed_hash {
        return Err(StoreError::ObjectHashMismatch {
            expected: computed_hash,
            actual: record.hash.clone(),
        });
    }
    if let Some(expected) = expected_object_id {
        if expected != computed_id {
            return Err(StoreError::ObjectIdMismatch {
                expected: expected.to_string(),
                actual: computed_id,
            });
        }
    }
    if let Some(path) = path {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if stem != computed_id {
            return Err(StoreError::FilenameMismatch {
                expected: computed_id,
                actual: stem.to_string(),
            });
        }
    }
    Ok(())
}

pub fn object_record_path(objects_root: &Path, object_id: &str) -> Option<PathBuf> {
    if let Some(subdir) = object_subdir_by_id(object_id) {
        let candidate = objects_root.join(subdir).join(format!("{object_id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
        return None;
    }
    for entry in fs::read_dir(objects_root).ok()? {
        let entry = entry.ok()?;
        let file_type = entry.file_type().ok()?;
        if !file_type.is_dir() {
            continue;
        }
        let candidate = entry.path().join(format!("{object_id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn store_object(
    objects_root: &Path,
    type_tag: &str,
    payload: Value,
) -> Result<String, StoreError> {
    let subdir = object_subdir(type_tag)
        .ok_or_else(|| StoreError::UnknownObjectType(type_tag.to_string()))?;
    let record = object_record(type_tag, payload)?;
    let path = objects_root
        .join(subdir)
        .join(format!("{}.json", record.object_id));
    fs::create_dir_all(path.parent().expect("object path must have parent"))?;

    if path.exists() {
        let existing = read_object_record_path(&path)?;
        validate_object_record(&existing, Some(&record.object_id), Some(&path))?;
        return Ok(record.object_id);
    }

    let temp_path = unique_temp_path(&path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)?;
    let rendered = serde_json::to_string_pretty(&record)?;
    file.write_all(rendered.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);
    let temp_record = read_object_record_path(&temp_path)?;
    validate_object_record(&temp_record, Some(&record.object_id), None)?;
    match fs::hard_link(&temp_path, &path) {
        Ok(()) => {
            fs::remove_file(&temp_path)?;
            sync_directory_best_effort(path.parent().expect("object path must have parent"))?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            fs::remove_file(&temp_path)?;
            let existing = read_object_record_path(&path)?;
            validate_object_record(&existing, Some(&record.object_id), Some(&path))?;
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            return Err(StoreError::Io(error));
        }
    }
    Ok(record.object_id)
}

pub fn read_object(objects_root: &Path, object_id: &str) -> Result<Value, StoreError> {
    Ok(read_object_record(objects_root, object_id)?.payload)
}

pub fn read_object_record(
    objects_root: &Path,
    object_id: &str,
) -> Result<ObjectRecord, StoreError> {
    let path = object_record_path(objects_root, object_id)
        .ok_or_else(|| StoreError::ObjectNotFound(object_id.to_string()))?;
    let record = read_object_record_path(&path)?;
    validate_object_record(&record, Some(object_id), Some(&path))?;
    Ok(record)
}

fn read_object_record_path(path: &Path) -> Result<ObjectRecord, StoreError> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_temp_path(path: &Path) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("object.json");
    path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        counter
    ))
}

fn sync_directory_best_effort(path: &Path) -> Result<(), StoreError> {
    match File::open(path) {
        Ok(dir) => {
            let _ = dir.sync_all();
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(StoreError::Io(error)),
    }
}

pub fn validate_sealed_commit(objects_root: &Path, commit_id: &str) -> Result<(), StoreError> {
    let mut seen = HashSet::new();
    validate_sealed_commit_inner(objects_root, commit_id, &mut seen)
}

fn validate_sealed_commit_inner(
    objects_root: &Path,
    commit_id: &str,
    seen: &mut HashSet<String>,
) -> Result<(), StoreError> {
    if !seen.insert(commit_id.to_string()) {
        return Ok(());
    }
    let commit = read_object(objects_root, commit_id)?;
    if commit.get("type").and_then(Value::as_str) != Some("commit") {
        return Err(StoreError::InvalidSealedCommit(format!(
            "not a commit object: {commit_id}"
        )));
    }

    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| StoreError::InvalidSealedCommit("parents must be a list".to_string()))?;
    for (index, parent) in parents.iter().enumerate() {
        let parent_id = parent.as_str().ok_or_else(|| {
            StoreError::InvalidSealedCommit(format!("parent {index} must be a commit object id"))
        })?;
        if !parent_id.starts_with("CF-COMMIT-") {
            return Err(StoreError::InvalidSealedCommit(format!(
                "parent {index} must be a commit object id"
            )));
        }
        let parent_payload = read_object(objects_root, parent_id)?;
        if parent_payload.get("type").and_then(Value::as_str) != Some("commit") {
            return Err(StoreError::InvalidSealedCommit(format!(
                "parent {index} is not a commit object"
            )));
        }
        validate_sealed_commit_inner(objects_root, parent_id, seen)?;
    }

    let roots = commit
        .get("roots")
        .and_then(Value::as_object)
        .ok_or_else(|| StoreError::InvalidSealedCommit("roots must be an object".to_string()))?;
    for root in REQUIRED_COMMIT_ROOTS {
        if !roots.contains_key(*root) {
            return Err(StoreError::InvalidSealedCommit(format!(
                "missing root {root}"
            )));
        }
    }
    for (root, object_id) in roots {
        let object_id = object_id.as_str().ok_or_else(|| {
            StoreError::InvalidSealedCommit(format!("root {root} must be an object id"))
        })?;
        if !object_id.starts_with("CF-") {
            return Err(StoreError::InvalidSealedCommit(format!(
                "root {root} must be an object id"
            )));
        }
        if let Some(expected_type) = expected_root_type(root) {
            let payload = read_object(objects_root, object_id)?;
            let actual_type = payload
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("(missing)");
            if actual_type != expected_type {
                return Err(StoreError::InvalidSealedCommit(format!(
                    "root {root} has type {actual_type}, expected {expected_type}"
                )));
            }
        }
    }

    let certificate = commit
        .get("certificate")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            StoreError::InvalidSealedCommit("certificate must be an object".to_string())
        })?;
    if certificate.get("result").and_then(Value::as_str) != Some("consistent") {
        return Err(StoreError::InvalidSealedCommit(
            "certificate is not consistent".to_string(),
        ));
    }
    Ok(())
}

const REQUIRED_COMMIT_ROOTS: &[&str] = &[
    "content_manifest",
    "atom_index",
    "trace_graph",
    "fire_delta",
    "verification",
    "policy",
];

fn expected_root_type(root: &str) -> Option<&'static str> {
    match root {
        "content_manifest" => Some("content_manifest"),
        "atom_index" => Some("atom_index"),
        "trace_graph" => Some("trace_graph"),
        "fire_delta" => Some("fire_ledger"),
        "resolution_ledger" => Some("resolution_ledger"),
        "verification" => Some("verification"),
        "policy" => Some("policy"),
        _ => None,
    }
}

pub fn commit_payload(
    parents: Vec<String>,
    roots: Map<String, Value>,
    certificate: Map<String, Value>,
) -> Value {
    let mut payload = Map::new();
    payload.insert("type".to_string(), Value::String("commit".to_string()));
    payload.insert("version".to_string(), Value::Number(1.into()));
    payload.insert(
        "parents".to_string(),
        Value::Array(parents.into_iter().map(Value::String).collect()),
    );
    payload.insert(
        "message".to_string(),
        Value::String("test commit".to_string()),
    );
    payload.insert("roots".to_string(), Value::Object(roots));
    payload.insert("certificate".to_string(), Value::Object(certificate));
    Value::Object(payload)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn canonical_json_sorts_keys_and_uses_compact_utf8() {
        let payload = json!({
            "z": 1,
            "a": "火",
            "nested": {
                "b": true,
                "a": null
            }
        });

        let rendered = String::from_utf8(canonical_json(&payload).unwrap()).unwrap();

        assert_eq!(rendered, r#"{"a":"火","nested":{"a":null,"b":true},"z":1}"#);
    }

    #[test]
    fn object_digest_matches_python_golden_payloads() {
        let blob = json!({
            "path": "docs/spec/auth.md",
            "content_hash": "sha256:aaa111",
            "bytes": "hello"
        });
        let atom_index = json!({
            "type": "atom_index",
            "version": 1,
            "duplicate_atom_ids": [],
            "atoms": [
                {
                    "atom_id": "REQ-AUTH-001",
                    "kind": "requirement",
                    "artifact_path": "docs/spec/auth.md",
                    "selector": {"type": "markdown_heading", "value": "REQ-AUTH-001"},
                    "content_hash": "sha256:aaa111"
                }
            ]
        });
        let commit = json!({
            "type": "commit",
            "version": 1,
            "parents": ["CF-COMMIT-parent"],
            "message": "golden",
            "roots": {
                "content_manifest": "CF-MANIFEST-abc",
                "atom_index": "CF-ATOMINDEX-def",
                "trace_graph": "CF-TRACE-ghi",
                "verification": "CF-VERIFY-jkl",
                "policy": "CF-POLICY-mno"
            },
            "certificate": {
                "result": "consistent",
                "open_required_fires": 0,
                "failed_checks": 0,
                "missing_required_links": 0,
                "stale_resolutions": 0,
                "duplicate_atom_ids": 0
            }
        });

        assert_eq!(
            object_digest("blob", &blob).unwrap(),
            "3b9124b51836f77522a243c6caa86d1a8773dc443246ce9860a1c0f630343514"
        );
        assert_eq!(object_id("blob", &blob).unwrap(), "CF-BLOB-3b9124b51836");
        assert_eq!(
            object_digest("atom_index", &atom_index).unwrap(),
            "6f78247e9b25327f61e2b86d57bca8f63a0dc781ae661ae50361d997b23ac0ea"
        );
        assert_eq!(
            object_id("atom_index", &atom_index).unwrap(),
            "CF-ATOMINDEX-6f78247e9b25"
        );
        assert_eq!(
            object_digest("commit", &commit).unwrap(),
            "f2c6dacf47ce757c611162a7e545cb6ea643aa27e877996f19e9268b1746c8c4"
        );
        assert_eq!(
            object_id("commit", &commit).unwrap(),
            "CF-COMMIT-f2c6dacf47ce"
        );
    }

    #[test]
    fn unknown_object_type_uses_generic_prefix() {
        let payload = json!({"type": "future", "version": 1});

        assert!(object_id("future", &payload)
            .unwrap()
            .starts_with("CF-OBJECT-"));
    }

    #[test]
    fn known_object_subdirs_are_single_source_of_truth() {
        let subdirs = known_object_subdirs().collect::<Vec<_>>();
        assert!(subdirs.contains(&"blobs"));
        assert!(subdirs.contains(&"commits"));
        assert!(subdirs.contains(&"evidence"));
        assert_eq!(object_subdir("commit"), Some("commits"));
        assert_eq!(object_prefix("commit"), "CF-COMMIT");
        assert_eq!(OBJECT_ID_DIGEST_HEX_LENGTH, 12);
    }

    #[test]
    fn object_record_path_routes_known_prefix_to_subdir() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commits = objects.join("commits");
        fs::create_dir_all(&commits).unwrap();
        let commit_id = "CF-COMMIT-123456789abc";
        let path = commits.join(format!("{commit_id}.json"));
        fs::write(&path, "{}\n").unwrap();

        assert_eq!(object_record_path(&objects, commit_id), Some(path));
        assert_eq!(object_record_path(&objects, "CF-COMMIT-missing"), None);
    }

    #[test]
    fn store_object_writes_and_reuses_immutable_record() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload =
            json!({"type": "policy", "version": 1, "policy": {"require_commit_signature": false}});

        let first_id = store_object(&objects, "policy", payload.clone()).unwrap();
        let second_id = store_object(&objects, "policy", payload.clone()).unwrap();
        let loaded = read_object(&objects, &first_id).unwrap();

        assert_eq!(first_id, second_id);
        assert_eq!(loaded, payload);
        assert!(objects
            .join("policies")
            .join(format!("{first_id}.json"))
            .exists());
        let leftovers = fs::read_dir(objects.join("policies"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[test]
    fn store_object_supports_artifact_ref_and_evidence_records() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let artifact = json!({
            "type": "artifact_ref",
            "version": 1,
            "uri": "/tmp/model.bin",
            "hash_algorithm": "sha256",
            "content_hash": "sha256:abc",
            "size_bytes": 3
        });
        let artifact_id = store_object(&objects, "artifact_ref", artifact.clone()).unwrap();
        let evidence = json!({
            "type": "evidence",
            "version": 1,
            "artifact_ref": artifact_id,
            "created_at": "2026-06-05T00:00:00Z"
        });
        let evidence_id = store_object(&objects, "evidence", evidence).unwrap();

        assert!(artifact_id.starts_with("CF-ARTIFACT-"));
        assert!(evidence_id.starts_with("CF-EVIDENCE-"));
        assert!(objects
            .join("artifact_refs")
            .join(format!("{artifact_id}.json"))
            .exists());
        assert!(objects
            .join("evidence")
            .join(format!("{evidence_id}.json"))
            .exists());
    }

    #[test]
    fn read_object_rejects_tampered_record() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let payload = json!({"type": "verification", "version": 1, "checks": []});
        let object_id = store_object(&objects, "verification", payload).unwrap();
        let path = objects
            .join("verifications")
            .join(format!("{object_id}.json"));
        let mut record: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        record["payload"]["version"] = json!(2);
        fs::write(&path, serde_json::to_string_pretty(&record).unwrap()).unwrap();

        let error = read_object(&objects, &object_id).unwrap_err();

        assert!(matches!(error, StoreError::ObjectIdMismatch { .. }));
    }

    #[test]
    fn store_object_rejects_unknown_type_for_writes() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");

        let error = store_object(&objects, "future", json!({"type": "future"})).unwrap_err();

        assert!(matches!(error, StoreError::UnknownObjectType(type_tag) if type_tag == "future"));
    }

    #[test]
    fn validate_sealed_commit_accepts_valid_commit_graph() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let parent_id = write_valid_commit(&objects, vec![]);
        let child_id = write_valid_commit(&objects, vec![parent_id]);

        validate_sealed_commit(&objects, &child_id).unwrap();
    }

    #[test]
    fn validate_sealed_commit_rejects_missing_required_root() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        roots.remove("fire_delta");
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("missing root fire_delta"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_wrong_root_type() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let mut roots = write_required_roots(&objects);
        let wrong_id = store_object(
            &objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {}}),
        )
        .unwrap();
        roots.insert("verification".to_string(), Value::String(wrong_id));
        let commit = commit_payload(vec![], roots, consistent_certificate());
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("root verification has type policy"))
        );
    }

    #[test]
    fn validate_sealed_commit_rejects_invalid_parent_id() {
        let temp = tempdir().unwrap();
        let objects = temp.path().join("objects");
        let commit = commit_payload(
            vec!["CF-BLOB-not-a-parent".to_string()],
            write_required_roots(&objects),
            consistent_certificate(),
        );
        let commit_id = store_object(&objects, "commit", commit).unwrap();

        let error = validate_sealed_commit(&objects, &commit_id).unwrap_err();

        assert!(
            matches!(error, StoreError::InvalidSealedCommit(message) if message.contains("parent 0 must be a commit object id"))
        );
    }

    fn write_valid_commit(objects: &Path, parents: Vec<String>) -> String {
        store_object(
            objects,
            "commit",
            commit_payload(
                parents,
                write_required_roots(objects),
                consistent_certificate(),
            ),
        )
        .unwrap()
    }

    fn write_required_roots(objects: &Path) -> Map<String, Value> {
        let content_manifest = store_object(
            objects,
            "content_manifest",
            json!({"type": "content_manifest", "version": 1, "entries": []}),
        )
        .unwrap();
        let atom_index = store_object(
            objects,
            "atom_index",
            json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
        )
        .unwrap();
        let trace_graph = store_object(
            objects,
            "trace_graph",
            json!({"type": "trace_graph", "version": 1, "links": []}),
        )
        .unwrap();
        let fire_delta = store_object(
            objects,
            "fire_ledger",
            json!({"type": "fire_ledger", "version": 1, "fires": []}),
        )
        .unwrap();
        let verification = store_object(
            objects,
            "verification",
            json!({"type": "verification", "version": 1, "checks": []}),
        )
        .unwrap();
        let policy = store_object(
            objects,
            "policy",
            json!({"type": "policy", "version": 1, "policy": {}}),
        )
        .unwrap();

        let mut roots = Map::new();
        roots.insert(
            "content_manifest".to_string(),
            Value::String(content_manifest),
        );
        roots.insert("atom_index".to_string(), Value::String(atom_index));
        roots.insert("trace_graph".to_string(), Value::String(trace_graph));
        roots.insert("fire_delta".to_string(), Value::String(fire_delta));
        roots.insert("verification".to_string(), Value::String(verification));
        roots.insert("policy".to_string(), Value::String(policy));
        roots
    }

    fn consistent_certificate() -> Map<String, Value> {
        let mut certificate = Map::new();
        certificate.insert(
            "result".to_string(),
            Value::String("consistent".to_string()),
        );
        certificate.insert("open_required_fires".to_string(), Value::Number(0.into()));
        certificate.insert("failed_checks".to_string(), Value::Number(0.into()));
        certificate.insert(
            "missing_required_links".to_string(),
            Value::Number(0.into()),
        );
        certificate.insert("stale_resolutions".to_string(), Value::Number(0.into()));
        certificate.insert("duplicate_atom_ids".to_string(), Value::Number(0.into()));
        certificate
    }
}
