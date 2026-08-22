use super::*;
use crate::compatibility::{
    support_manifest_json, BRANCH_LIST_SCHEMA, COMMAND_RESULT_SCHEMA, SHOW_RESULT_SCHEMA,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn public_schema(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/public")
        .join(name);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn type_matches(spec: &Value, instance: &Value) -> bool {
    match spec {
        Value::String(name) => match name.as_str() {
            "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
            expected => json_type_name(instance) == expected,
        },
        Value::Array(options) => options.iter().any(|option| type_matches(option, instance)),
        _ => true,
    }
}

fn validate_node(schema: &Value, instance: &Value, path: &str) -> Result<(), String> {
    if let Some(expected) = schema.get("const") {
        if instance != expected {
            return Err(format!("{path}: expected const {expected}, got {instance}"));
        }
    }
    if let Some(types) = schema.get("type") {
        if !type_matches(types, instance) {
            return Err(format!(
                "{path}: type mismatch, got {}",
                json_type_name(instance)
            ));
        }
    }
    let expects_object = schema.get("properties").is_some()
        || schema.get("required").is_some()
        || schema.get("type").and_then(Value::as_str) == Some("object")
        || schema
            .get("type")
            .and_then(Value::as_array)
            .is_some_and(|types| types.iter().any(|item| item.as_str() == Some("object")));
    if expects_object && !instance.is_null() {
        let object = instance
            .as_object()
            .ok_or_else(|| format!("{path}: expected object"))?;
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for field in required {
                let name = field.as_str().expect("required field names are strings");
                if !object.contains_key(name) {
                    return Err(format!("{path}: missing required {name}"));
                }
            }
        }
        let properties = schema.get("properties").and_then(Value::as_object);
        if let Some(properties) = properties {
            for (name, child) in properties {
                if let Some(value) = object.get(name) {
                    validate_node(child, value, &format!("{path}.{name}"))?;
                }
            }
        }
        if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            let allowed = properties
                .map(|props| props.keys().cloned().collect::<BTreeSet<_>>())
                .unwrap_or_default();
            for key in object.keys() {
                if !allowed.contains(key) {
                    return Err(format!("{path}: unexpected {key}"));
                }
            }
        }
    }
    if schema.get("type").and_then(Value::as_str) == Some("array") {
        let items = instance
            .as_array()
            .ok_or_else(|| format!("{path}: expected array"))?;
        if let Some(item_schema) = schema.get("items") {
            for (index, item) in items.iter().enumerate() {
                validate_node(item_schema, item, &format!("{path}[{index}]"))?;
            }
        }
    }
    Ok(())
}

fn assert_schema_accepts(schema: &Value, instance: &Value) {
    validate_node(schema, instance, "$").unwrap();
}

fn assert_schema_rejects(schema: &Value, instance: &Value, needle: &str) {
    let error = validate_node(schema, instance, "$").expect_err("schema should reject");
    assert!(
        error.contains(needle),
        "rejection {error} did not mention {needle}"
    );
}

fn advertised_schema(id: &str) -> Value {
    let schemas = support_manifest_json()["schemas"]
        .as_array()
        .cloned()
        .unwrap();
    schemas
        .into_iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("support.schemas missing {id}"))
}

fn command_entry(name: &str) -> Value {
    capabilities_data_json()["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["name"] == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing command {name}"))
}

#[test]
fn capabilities_advertise_branch_and_show_data_schemas_only() {
    let capabilities = capabilities_data_json();
    assert_eq!(capabilities["command_result_schema"], COMMAND_RESULT_SCHEMA);
    let commands = capabilities["commands"].as_array().unwrap();
    let advertised = commands
        .iter()
        .filter(|command| command.get("data_schema").is_some())
        .map(|command| {
            (
                command["name"].as_str().unwrap().to_string(),
                command["data_schema"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        advertised,
        vec![
            ("branch".to_string(), BRANCH_LIST_SCHEMA.to_string()),
            ("show".to_string(), SHOW_RESULT_SCHEMA.to_string()),
        ]
    );
    let branch = command_entry("branch");
    assert_eq!(branch["output_schema"], COMMAND_RESULT_SCHEMA);
    assert_eq!(branch["data_schema"], BRANCH_LIST_SCHEMA);
    let show = command_entry("show");
    assert_eq!(show["output_schema"], COMMAND_RESULT_SCHEMA);
    assert_eq!(show["data_schema"], SHOW_RESULT_SCHEMA);
    let doctor = command_entry("doctor");
    assert_eq!(doctor["output_schema"], COMMAND_RESULT_SCHEMA);
    assert!(doctor.get("data_schema").is_none());
    assert_eq!(
        advertised_schema(BRANCH_LIST_SCHEMA)["path"],
        "schemas/public/branch-list-v1.schema.json"
    );
    assert_eq!(
        advertised_schema(SHOW_RESULT_SCHEMA)["path"],
        "schemas/public/show-result-v1.schema.json"
    );
}

#[test]
fn branch_list_and_show_payloads_match_public_data_schemas() {
    let branch_schema = public_schema("branch-list-v1.schema.json");
    let show_schema = public_schema("show-result-v1.schema.json");
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let listed = list_branches(&repo_root).unwrap();
    let branch_data = serde_json::to_value(BranchListDataWire {
        type_tag: "codefire_branch_list",
        version: 1,
        branches: BranchesWire(&listed.branches),
        metrics: None,
    })
    .unwrap();
    listed.finish_complete().unwrap();
    assert_eq!(branch_data["branches"][0]["name"], "main");
    assert_eq!(branch_data["branches"][0]["state"], "closed");
    assert_schema_accepts(&branch_schema, &branch_data);

    let show_data = show_commitish_data(Some(&repo_root), "main").unwrap();
    assert_eq!(show_data["show"]["certificate"], "consistent");
    assert_eq!(show_data["show"]["claim"]["result"], "consistent");
    assert!(show_data["show"]["claim"].get("profile").is_none());
    assert!(show_data["show"]["claim"].get("phase").is_none());
    assert!(show_data["show"]["claim"].get("subject").is_none());
    assert_schema_accepts(&show_schema, &show_data);

    let mut extra_branch = branch_data.clone();
    extra_branch["branches"][0]["copied"] = json!(true);
    extra_branch["future_note"] = json!("preserve");
    assert_schema_accepts(&branch_schema, &extra_branch);
    extra_branch["version"] = json!(2);
    assert_schema_rejects(&branch_schema, &extra_branch, "const");
    let mut missing_name = branch_data.clone();
    missing_name["branches"][0]
        .as_object_mut()
        .unwrap()
        .remove("name");
    assert_schema_rejects(&branch_schema, &missing_name, "name");

    let mut extra_show = show_data.clone();
    extra_show["show"]["advisory"] = json!("noncritical");
    extra_show["show"]["claim"]["coverage_note"] = json!("unspecified");
    assert_schema_accepts(&show_schema, &extra_show);
    extra_show["version"] = json!(2);
    assert_schema_rejects(&show_schema, &extra_show, "const");
    let mut missing_commit = show_data.clone();
    missing_commit["show"]
        .as_object_mut()
        .unwrap()
        .remove("commit");
    assert_schema_rejects(&show_schema, &missing_commit, "commit");
}

#[test]
fn show_claim_projects_persisted_certificate_fields_without_inventing_proof() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("open");
    init_repo(&repo_root, false).unwrap();
    let initial = show_commitish_data(Some(&repo_root), "main").unwrap();
    assert_eq!(initial["show"]["certificate"], "consistent");
    assert_eq!(initial["show"]["claim"], json!({"result": "consistent"}));
    assert_eq!(
        initial["show"]["certificate"],
        initial["show"]["claim"]["result"]
    );

    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
            dry_run: false,
            json_output: false,
            lock: LockOptions::default(),
            idempotency_key: None,
        },
    )
    .unwrap();
    fs::write(open_dir.join("README.md"), "claim projection\n").unwrap();
    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(verification.result, "passed");
    run_commit(&CommitOptions {
        path: open_dir,
        message: "Seal claim projection".to_string(),
        dry_run: false,
        full_output: false,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    })
    .unwrap();

    let show_data = show_commitish_data(Some(&repo_root), "main").unwrap();
    let envelope = command_result_envelope(
        "show",
        true,
        0,
        Some(&repo_root),
        show_data.clone(),
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(envelope["schema"], COMMAND_RESULT_SCHEMA);
    assert_eq!(show_data["show"]["certificate"], "passed");
    assert_eq!(show_data["show"]["claim"]["result"], "passed");
    assert_eq!(
        show_data["show"]["claim"]["profile"],
        "local_verification_v1"
    );
    assert_eq!(show_data["show"]["claim"]["phase"], "pre_commit");
    assert_eq!(show_data["show"]["claim"]["subject"]["kind"], "live_open");
    assert_eq!(
        show_data["show"]["claim"]["subject"]["trust_kind"],
        "local_unisolated"
    );
    assert_eq!(show_data["show"]["claim"]["subject"]["immutable"], false);
    assert!(show_data["show"]["claim"]["subject"]
        .get("subject_digest")
        .is_none());
    let text = show_commitish(Some(&repo_root), "main").unwrap();
    assert!(text.contains("Certificate: passed"));
    assert!(!text.contains("local_verification_v1"));
    assert_schema_accepts(&public_schema("show-result-v1.schema.json"), &show_data);

    let objects = repo_root.join(".codefire").join("objects");
    let head =
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap();
    let commit = codefire_store::read_object(&objects, &head).unwrap();
    let certificate = commit["certificate"].as_object().unwrap();
    assert_eq!(
        show_data["show"]["claim"]["profile"],
        certificate["profile"]
    );
    assert_eq!(show_data["show"]["claim"]["phase"], certificate["phase"]);
    assert_eq!(
        show_data["show"]["claim"]["subject"]["kind"],
        certificate["subject"]["kind"]
    );
}
