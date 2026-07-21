use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

const LEGACY_CODE_ATOM: &str = "CODE:src/models.py::class:ExperimentPlan";

fn codefire() -> &'static str {
    env!("CARGO_BIN_EXE_codefire-rs")
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(codefire())
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run CodeFire command")
}

fn json_document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout must contain exactly one JSON document: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_success(output: &Output, command: &str) -> Value {
    assert!(
        output.status.success(),
        "{command} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{command} wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = json_document(output);
    assert_eq!(value["schema"], "codefire.command_result.v1");
    assert_eq!(value["command"], command);
    assert_eq!(value["ok"], true);
    assert_eq!(value["exit_code"], 0);
    value
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(ALPHABET[(a >> 2) as usize] as char);
        encoded.push(ALPHABET[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            encoded.push(ALPHABET[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() == 3 {
            encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn store_manifest(objects: &Path, open_dir: &Path, paths: &[&str]) -> String {
    let mut paths = paths.to_vec();
    paths.sort_unstable();
    let entries = paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(open_dir.join(path)).unwrap();
            let blob = codefire_store::store_object(
                objects,
                "blob",
                json!({
                    "type": "blob",
                    "version": 1,
                    "encoding": "base64",
                    "content": encode_base64(&bytes),
                }),
            )
            .unwrap();
            json!({"path": path, "kind": "file", "mode": "100644", "blob": blob})
        })
        .collect::<Vec<_>>();
    codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": entries}),
    )
    .unwrap()
}

fn store_root(objects: &Path, kind: &str, payload: Value) -> String {
    codefire_store::store_object(objects, kind, payload).unwrap()
}

fn consistent_certificate() -> Map<String, Value> {
    [
        ("result", json!("consistent")),
        ("open_required_fires", json!(0)),
        ("failed_checks", json!(0)),
        ("missing_required_links", json!(0)),
        ("stale_resolutions", json!(0)),
        ("missing_evidence_refs", json!(0)),
        ("duplicate_atom_ids", json!(0)),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

fn legacy_schema2_commit(objects: &Path, open_dir: &Path, parent: String) -> (String, Value) {
    let current_index = codefire_core::build_atom_index(open_dir).unwrap();
    let derived_atom = current_index
        .atoms
        .iter()
        .find(|atom| atom.atom_id == LEGACY_CODE_ATOM)
        .expect("the unannotated Python class must restore its historical derived Atom ID");
    assert_eq!(derived_atom.selector.selector_type, "python_class");
    assert_eq!(derived_atom.selector.value, "ExperimentPlan");
    let mut legacy_index = serde_json::to_value(current_index).unwrap();
    legacy_index["hash_schema_version"] = json!(2);
    legacy_index["atoms"] = Value::Array(
        legacy_index["atoms"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|atom| atom["atom_id"] != LEGACY_CODE_ATOM)
            .cloned()
            .collect(),
    );

    let trace_graph = codefire_core::current_trace_graph(open_dir).unwrap();
    assert!(trace_graph
        .links
        .iter()
        .any(|link| link.to == LEGACY_CODE_ATOM));
    let policy = codefire_core::parse_verification_policy(open_dir).unwrap();
    assert_eq!(
        policy.required_links,
        BTreeMap::from([("code".to_string(), Vec::new())])
    );

    let manifest = store_manifest(
        objects,
        open_dir,
        &[
            "codefire.links.yaml",
            "codefire.policy.yaml",
            "codefire.yaml",
            "docs/design/plan.md",
            "src/models.py",
        ],
    );
    let atom_index = store_root(objects, "atom_index", legacy_index);
    let trace_graph = store_root(
        objects,
        "trace_graph",
        serde_json::to_value(trace_graph).unwrap(),
    );
    let fire_delta = store_root(
        objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": []}),
    );
    let resolution_ledger = store_root(
        objects,
        "resolution_ledger",
        json!({"type": "resolution_ledger", "version": 1, "resolutions": []}),
    );
    let verification = store_root(
        objects,
        "verification",
        json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "trace_completeness_required": policy.require_trace_completeness,
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "missing_evidence_refs": [],
            "duplicate_atom_ids": [],
            "verified_at": "2026-07-12T00:00:00Z",
        }),
    );
    let policy = store_root(
        objects,
        "policy",
        json!({"type": "policy", "version": 1, "policy": policy}),
    );
    let roots = Map::from_iter([
        ("content_manifest".to_string(), Value::String(manifest)),
        ("atom_index".to_string(), Value::String(atom_index)),
        ("trace_graph".to_string(), Value::String(trace_graph)),
        ("fire_delta".to_string(), Value::String(fire_delta)),
        (
            "resolution_ledger".to_string(),
            Value::String(resolution_ledger),
        ),
        ("verification".to_string(), Value::String(verification)),
        ("policy".to_string(), Value::String(policy)),
    ]);
    let mut commit = codefire_store::commit_payload(vec![parent], roots, consistent_certificate());
    commit["message"] = json!("Legacy Python-to-Rust extractor transition");
    commit
        .as_object_mut()
        .unwrap()
        .remove("semantic_validation_profile");
    let commit_id = store_root(objects, "commit", commit.clone());
    (commit_id, commit)
}

#[test]
fn legacy_profile_bridge_rebaselines_without_fire_storm_and_restores_strict_closure() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    let repo_arg = repo.to_str().unwrap();
    let open_arg = open_dir.to_str().unwrap();

    let initialized = run(temp.path(), &["init", repo_arg]);
    assert!(
        initialized.status.success(),
        "init failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&initialized.stdout),
        String::from_utf8_lossy(&initialized.stderr)
    );
    assert_success(&run(&repo, &["open", "main", open_arg, "--json"]), "open");

    fs::create_dir_all(open_dir.join("docs/design")).unwrap();
    fs::create_dir_all(open_dir.join("src")).unwrap();
    fs::write(
        open_dir.join("docs/design/plan.md"),
        "## DES-plan: Design\nExperiment planning contract.\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("src/models.py"),
        "from dataclasses import dataclass\n\n@dataclass\nclass ExperimentPlan:\n    name: str\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.links.yaml"),
        format!(
            "version: 1\nlinks:\n  - from: DES-plan\n    to: {LEGACY_CODE_ATOM}\n    type: implemented_by\n"
        ),
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "version: 1\nrequired_links:\n  code:\n",
    )
    .unwrap();
    fs::write(
        open_dir.join("codefire.yaml"),
        "designs:\n  - path: docs/design/**/*.md\ncode:\n  - path: src/**/*.py\n",
    )
    .unwrap();

    let branch_path = repo.join(".codefire/branches/main.json");
    let opened_path = repo.join(".codefire/opened/main.json");
    let initial_head = read_json(&branch_path)["head"]
        .as_str()
        .unwrap()
        .to_string();
    let objects = repo.join(".codefire/objects");
    let (legacy_id, legacy_payload) = legacy_schema2_commit(&objects, &open_dir, initial_head);
    assert_eq!(legacy_payload["root_schema_version"], 2);
    assert!(legacy_payload.get("semantic_validation_profile").is_none());
    codefire_store::validate_sealed_commit(&objects, &legacy_id).unwrap();

    let mut strict_dangling = legacy_payload;
    strict_dangling["semantic_validation_profile"] =
        json!(codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE);
    strict_dangling["message"] = json!("Current profile must reject dangling Trace endpoints");
    let strict_dangling_id = store_root(&objects, "commit", strict_dangling);
    assert!(matches!(
        codefire_store::validate_sealed_commit(&objects, &strict_dangling_id),
        Err(codefire_store::StoreError::InvalidSealedCommit(message))
            if message.contains("is missing from atom_index")
    ));
    let strict_show = run(&repo, &["show", &strict_dangling_id, "--json"]);
    assert_eq!(strict_show.status.code(), Some(22));
    assert!(strict_show.stderr.is_empty());
    let strict_show = json_document(&strict_show);
    assert_eq!(strict_show["ok"], false);
    assert_eq!(
        strict_show["diagnostics"][0]["kind"],
        "sealed_commit_invalid"
    );

    let mut branch = read_json(&branch_path);
    branch["head"] = json!(legacy_id);
    branch["state"] = json!("open-clean");
    write_json(&branch_path, &branch);
    let mut opened = read_json(&opened_path);
    opened["open"]["current_base_commit"] = json!(legacy_id);
    opened["state"]["last_known"] = json!("open-clean");
    write_json(&opened_path, &opened);

    let status = assert_success(
        &run(&repo, &["status", "--path", open_arg, "--json"]),
        "status",
    );
    assert_eq!(status["data"]["base"], legacy_id);
    assert_eq!(status["data"]["open_fires"], 0);
    assert_eq!(
        status["data"]["scan_prediction"]["tool_migration"]["kind"],
        "atom_hash_schema_changed"
    );

    let scan = assert_success(&run(&repo, &["scan", "--path", open_arg, "--json"]), "scan");
    assert_eq!(scan["data"]["changed_atom_count"], 0);
    assert_eq!(scan["data"]["non_atom_changed_file_count"], 0);
    assert_eq!(scan["data"]["open_fire_count"], 0);
    assert_eq!(scan["data"]["changed_atoms"], json!([]));
    assert_eq!(scan["data"]["open_fires"], json!([]));
    assert_eq!(
        scan["data"]["tool_migration"],
        json!({
            "kind": "atom_hash_schema_changed",
            "from_hash_schema_version": 2,
            "to_hash_schema_version": codefire_core::ATOM_HASH_SCHEMA_VERSION,
            "source_unchanged": true,
            "affected_atoms": [LEGACY_CODE_ATOM],
        })
    );

    let verify = assert_success(
        &run(&repo, &["verify", "--path", open_arg, "--json"]),
        "verify",
    );
    assert_eq!(verify["data"]["result"], "passed");
    assert_eq!(verify["data"]["open_required_fires"], 0);
    assert_eq!(verify["data"]["missing_required_links"], json!([]));

    let dry_run = assert_success(
        &run(
            &repo,
            &[
                "commit",
                "--path",
                open_arg,
                "--dry-run",
                "--json",
                "-m",
                "Rebaseline legacy extractor profile",
            ],
        ),
        "commit",
    );
    assert_eq!(dry_run["data"]["dry_run"], true);
    assert_eq!(dry_run["data"]["blocked"], false);
    assert_eq!(dry_run["data"]["plan"]["parents"], json!([legacy_id]));
    assert_eq!(read_json(&branch_path)["head"], legacy_id);

    let committed = assert_success(
        &run(
            &repo,
            &[
                "commit",
                "--path",
                open_arg,
                "--json",
                "-m",
                "Rebaseline legacy extractor profile",
            ],
        ),
        "commit",
    );
    assert_eq!(committed["data"]["applied"], true);
    let current_id = committed["data"]["commit_id"].as_str().unwrap().to_string();
    let current = codefire_store::read_object(&objects, &current_id).unwrap();
    assert_eq!(
        current["semantic_validation_profile"],
        codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
    );
    assert_eq!(current["parents"], json!([legacy_id]));
    codefire_store::validate_sealed_commit(&objects, &current_id).unwrap();

    let atom_index_id = current["roots"]["atom_index"].as_str().unwrap();
    let trace_graph_id = current["roots"]["trace_graph"].as_str().unwrap();
    let atom_index: codefire_core::AtomIndex =
        serde_json::from_value(codefire_store::read_object(&objects, atom_index_id).unwrap())
            .unwrap();
    let trace_graph: codefire_core::TraceGraph =
        serde_json::from_value(codefire_store::read_object(&objects, trace_graph_id).unwrap())
            .unwrap();
    assert!(atom_index
        .atoms
        .iter()
        .any(|atom| atom.atom_id == LEGACY_CODE_ATOM));
    codefire_core::validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap();
    assert_eq!(read_json(&branch_path)["head"], current_id);
}
