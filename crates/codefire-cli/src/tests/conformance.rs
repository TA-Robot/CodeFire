use super::*;
use crate::compatibility::validate_record_version;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/conformance")
}

#[test]
fn checked_conformance_manifest_matches_rust_support_decisions() {
    let root = corpus_root();
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("manifest-v1.json")).unwrap()).unwrap();
    assert_eq!(manifest["support_profile"], "codefire.support.v1");
    for vector in manifest["vectors"].as_array().unwrap() {
        let path = root.join(vector["path"].as_str().unwrap());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(
            codefire_util::sha256_hex(&bytes),
            vector["sha256"].as_str().unwrap(),
            "checksum mismatch for {}",
            path.display()
        );
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        if let Some(expected) = vector.get("canonical_sha256").and_then(Value::as_str) {
            assert_eq!(
                codefire_util::sha256_hex(&codefire_store::canonical_json(&value).unwrap()),
                expected
            );
        }
        let decision = match vector["boundary"].as_str().unwrap() {
            "canonical_json" => "accept",
            "remote_branch" => {
                if validate_record_version(&value, "remote_branch").is_ok() {
                    "accept"
                } else {
                    "deny"
                }
            }
            "object_record" => {
                if codefire_store::object_record_document_from_value(value).is_ok() {
                    "accept"
                } else {
                    "deny"
                }
            }
            boundary => panic!("unknown conformance boundary: {boundary}"),
        };
        assert_eq!(decision, vector["expected"].as_str().unwrap());
    }
}

#[test]
fn independent_minimal_consumer_accepts_the_checked_manifest() {
    let root = corpus_root();
    let output = Command::new("python3")
        .arg(root.join("minimal_consumer.py"))
        .arg(root.join("manifest-v1.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "minimal consumer failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["type"], "codefire_conformance_result");
    assert_eq!(result["consumer"], "independent-python-stdlib-v1");
    assert_eq!(result["ok"], true);
}
