use serde_json::Value;
use sha2::{Digest, Sha256};

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
}
