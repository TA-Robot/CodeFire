use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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
    Ok(format!("{}-{}", object_prefix(type_tag), &digest[..12]))
}

pub fn object_prefix(type_tag: &str) -> &'static str {
    match type_tag {
        "blob" => "CF-BLOB",
        "content_manifest" => "CF-MANIFEST",
        "atom_index" => "CF-ATOMINDEX",
        "trace_graph" => "CF-TRACE",
        "fire_ledger" => "CF-FIRELEDGER",
        "resolution_ledger" => "CF-RESOLUTION",
        "verification" => "CF-VERIFY",
        "policy" => "CF-POLICY",
        "commit" => "CF-COMMIT",
        "branch" => "CF-BRANCH",
        _ => "CF-OBJECT",
    }
}

pub fn object_subdir(type_tag: &str) -> Option<&'static str> {
    match type_tag {
        "blob" => Some("blobs"),
        "content_manifest" => Some("content_manifests"),
        "atom_index" => Some("atom_indexes"),
        "trace_graph" => Some("trace_graphs"),
        "fire_ledger" => Some("fire_ledgers"),
        "resolution_ledger" => Some("resolution_ledgers"),
        "verification" => Some("verifications"),
        "policy" => Some("policies"),
        "commit" => Some("commits"),
        "branch" => Some("branches"),
        _ => None,
    }
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

    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = read_object_record_path(&path)?;
            validate_object_record(&existing, Some(&record.object_id), Some(&path))?;
            return Ok(record.object_id);
        }
        Err(error) => return Err(StoreError::Io(error)),
    };
    let rendered = serde_json::to_string_pretty(&record)?;
    file.write_all(rendered.as_bytes())?;
    file.write_all(b"\n")?;
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
}
