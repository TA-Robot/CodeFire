use super::*;
use serde_json::{json, Map};
use tempfile::tempdir;

#[test]
fn init_creates_python_compatible_repo_layout() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");

    let result = init_repo(&repo_root, false).unwrap();
    let branches = list_branches(&repo_root).unwrap();

    assert_eq!(result.repo_root, repo_root);
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert_eq!(branches[0].head, result.main_commit);
    assert_eq!(branches[0].state, "closed");
    codefire_store::validate_sealed_commit(
        &repo_root.join(".codefire").join("objects"),
        &result.main_commit,
    )
    .unwrap();
    assert!(init_repo(&repo_root, false).is_err());
    init_repo(&repo_root, true).unwrap();
}

#[test]
fn open_materializes_manifest_and_updates_registry() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_file_commit(&objects, "docs/readme.md", "hello\n");
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": "main",
        "head": commit_id,
        "state": "closed",
        "created_at": "2026-06-04T00:00:00Z"
    });
    save_branch_record(&repo_root, &branch).unwrap();

    let result = open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
        },
    )
    .unwrap();
    let status = read_status(&open_dir).unwrap();
    let branches = list_branches(&open_dir).unwrap();

    assert_eq!(result.open_dir, open_dir);
    assert_eq!(
        fs::read_to_string(result.open_dir.join("docs").join("readme.md")).unwrap(),
        "hello\n"
    );
    assert_eq!(
        status,
        Status {
            branch: "main".to_string(),
            state: "open-clean".to_string(),
            base: commit_id.clone(),
            open_fires: 0,
        }
    );
    assert_eq!(branches[0].state, "open-clean");
}

#[test]
fn clone_branch_creates_closed_branch_and_rejects_burning_source() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let commit_id = write_file_commit(&objects, "docs/readme.md", "hello\n");
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": commit_id,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    clone_branch(
        &repo_root,
        &CloneOptions {
            source: "main".to_string(),
            new_branch: "feature/session".to_string(),
        },
    )
    .unwrap();
    let cloned = load_branch_record(&repo_root, "feature/session").unwrap();
    assert_eq!(
        required_string(&cloned, &["head"]).unwrap(),
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap()
    );
    assert_eq!(cloned.get("state").and_then(Value::as_str), Some("closed"));

    let mut burning = cloned;
    burning["name"] = Value::String("burning".to_string());
    burning["state"] = Value::String("open-burning".to_string());
    save_branch_record(&repo_root, &burning).unwrap();
    let error = clone_branch(
        &repo_root,
        &CloneOptions {
            source: "burning".to_string(),
            new_branch: "copy".to_string(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("cannot clone from branch 'burning' while it is open-burning"));
}

#[test]
fn show_and_diff_local_branches() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "src/session.py", "def ttl():\n    return 30\n");
    let feature_commit =
        write_file_commit(&objects, "src/session.py", "def ttl():\n    return 15\n");
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": main_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();

    let show = show_commitish(Some(&repo_root), "feature-session").unwrap();
    assert!(show.contains("Object: feature-session@CF-COMMIT-"));
    assert!(show.contains("Message: test commit"));
    assert!(show.contains("Files: 1"));
    assert!(show.contains("Certificate: consistent"));

    let diff = diff_commitish_with_options(
        Some(&repo_root),
        "main",
        "feature-session",
        &DiffOptions::default(),
    )
    .unwrap();
    assert!(diff.contains("--- main@CF-COMMIT-"));
    assert!(diff.contains("+++ feature-session@CF-COMMIT-"));
    assert!(diff.contains("-    return 30"));
    assert!(diff.contains("+    return 15"));
}

#[test]
fn parse_diff_args_accepts_algorithm_forms() {
    let args = vec![
        "--algorithm".to_string(),
        "patience".to_string(),
        "--rename-detection".to_string(),
        "main".to_string(),
        "feature".to_string(),
    ];
    let parsed = parse_diff_args(&args).unwrap();
    assert_eq!(parsed.left, "main");
    assert_eq!(parsed.right, "feature");
    assert_eq!(parsed.diff.algorithm, DiffAlgorithm::Patience);
    assert!(parsed.diff.rename_detection);

    let args = vec![
        "main".to_string(),
        "feature".to_string(),
        "--algorithm=histogram".to_string(),
    ];
    let parsed = parse_diff_args(&args).unwrap();
    assert_eq!(parsed.diff.algorithm, DiffAlgorithm::Histogram);
    assert!(!parsed.diff.rename_detection);

    let error = parse_diff_args(&[
        "--algorithm".to_string(),
        "minimal".to_string(),
        "main".to_string(),
        "feature".to_string(),
    ])
    .unwrap_err()
    .to_string();
    assert!(error.contains("--algorithm must be one of"));
}

#[test]
fn file_remote_upload_clone_show_diff_and_merge_request_flow() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let clone_repo = temp.path().join("clone-repo");
    let server = temp.path().join("server");
    init_repo(&repo_root, false).unwrap();
    init_repo(&clone_repo, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let main_commit = write_file_commit(&objects, "docs/readme.md", "hello 30\n");
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/readme.md",
        "hello 15\n",
        vec![main_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": main_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    let project_url = format!("cf://{}/org/app", server.display());
    let main_url = format!("{project_url}/main");
    let feature_url = format!("{project_url}/feature-session");

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "main".to_string(),
            remote_url: main_url.clone(),
        },
    )
    .unwrap();
    let branches = list_remote_branches(&project_url).unwrap();
    assert_eq!(branches[0].name, "main");
    assert_eq!(
        branches[0].head,
        load_branch_record(&repo_root, "main").unwrap()["head"]
    );

    clone_branch(
        &clone_repo,
        &CloneOptions {
            source: main_url.clone(),
            new_branch: "main-from-remote".to_string(),
        },
    )
    .unwrap();
    let cloned = load_branch_record(&clone_repo, "main-from-remote").unwrap();
    assert_eq!(
        required_string(&cloned, &["head"]).unwrap(),
        required_string(&load_branch_record(&repo_root, "main").unwrap(), &["head"]).unwrap()
    );

    upload_branch(
        &repo_root,
        &UploadOptions {
            branch: "feature-session".to_string(),
            remote_url: feature_url.clone(),
        },
    )
    .unwrap();
    let show = show_commitish(None, &feature_url).unwrap();
    assert!(show.contains(&format!("Object: {feature_url}@CF-COMMIT-")));
    assert!(show.contains("Certificate: consistent"));
    let diff = diff_commitish_with_options(None, &main_url, &feature_url, &DiffOptions::default())
        .unwrap();
    assert!(diff.contains(&format!("--- {main_url}@CF-COMMIT-")));
    assert!(diff.contains(&format!("+++ {feature_url}@CF-COMMIT-")));
    assert!(diff.contains("+hello 15"));

    let mr = request_merge(&RequestMergeOptions {
        source_url: feature_url.clone(),
        target_url: main_url.clone(),
    })
    .unwrap();
    assert!(mr.id.starts_with("MR-"));
    let listed = list_merge_requests(&project_url).unwrap();
    assert_eq!(listed[0].status, "open");
    review_merge_request(&RequestReviewOptions {
        project_url: project_url.clone(),
        mr_id: mr.id.clone(),
        reviewer: "alice".to_string(),
        decision: "approve".to_string(),
        comment: "sealed source is ready".to_string(),
    })
    .unwrap();
    assert_eq!(
        list_merge_requests(&project_url).unwrap()[0].status,
        "approved"
    );
    let applied = apply_merge_request(&RequestApplyOptions {
        project_url: project_url.clone(),
        mr_id: mr.id,
    })
    .unwrap();
    assert_eq!(applied.target_branch, "main");
    assert_eq!(
        applied.head,
        required_string(
            &load_remote_branch(&parse_cf_url(&feature_url).unwrap()).unwrap(),
            &["head"]
        )
        .unwrap()
    );
    assert_eq!(
        required_string(
            &load_remote_branch(&parse_cf_url(&main_url).unwrap()).unwrap(),
            &["head"]
        )
        .unwrap(),
        applied.head
    );
}

#[test]
fn merge_branch_writes_source_changes_and_marks_target_burning() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 30\n\n## DES-session: Design\nClock policy\n",
    );
    let feature_commit = write_file_commit_with_parents(
        &objects,
        "docs/spec/session.md",
        "## REQ-session: Requirement\nTTL 15\n\n## DES-session: Design\nClock policy\n",
        vec![base_commit.clone()],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": base_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": feature_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
        },
    )
    .unwrap();

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
        },
    )
    .unwrap();

    assert!(result.conflicts.is_empty());
    assert_eq!(
        fs::read_to_string(open_dir.join("docs/spec/session.md")).unwrap(),
        "## REQ-session: Requirement\nTTL 15\n\n## DES-session: Design\nClock policy\n"
    );
    fs::write(
        open_dir.join("codefire.links.yaml"),
        "links:\n  - from: REQ-session\n    to: DES-session\n    type: refined_by\n",
    )
    .unwrap();
    let scan = run_scan(&open_dir).unwrap();
    assert!(scan
        .open_fires
        .iter()
        .any(|fire| fire.reason == "merge_changed"));
    let status = read_status(&open_dir).unwrap();
    assert_eq!(status.state, "open-burning");
    let active_state_path = PathBuf::from(
        required_string(
            &read_json(&opened_registry_path(&repo_root, "main")).unwrap(),
            &["open", "active_state_path"],
        )
        .unwrap(),
    );
    let state = read_json(&active_state_path.join("state.json")).unwrap();
    let expected_parent = required_string(
        &load_branch_record(&repo_root, "feature-session").unwrap(),
        &["head"],
    )
    .unwrap();
    assert_eq!(
        state.get("pending_merge_parent").and_then(Value::as_str),
        Some(expected_parent.as_str())
    );
}

#[test]
fn merge_branch_writes_conflict_markers_for_divergent_changes() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
    let objects = repo_root.join(".codefire").join("objects");
    let base_commit = write_file_commit(&objects, "src/session.py", "def ttl():\n    return 30\n");
    let target_commit = write_file_commit_with_parents(
        &objects,
        "src/session.py",
        "def ttl():\n    return 45\n",
        vec![base_commit.clone()],
    );
    let source_commit = write_file_commit_with_parents(
        &objects,
        "src/session.py",
        "def ttl():\n    return 15\n",
        vec![base_commit],
    );
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "main",
            "head": target_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    save_branch_record(
        &repo_root,
        &json!({
            "type": "branch",
            "version": 1,
            "name": "feature-session",
            "head": source_commit,
            "state": "closed",
            "created_at": "2026-06-04T00:00:00Z"
        }),
    )
    .unwrap();
    open_branch_from(
        &repo_root,
        &OpenOptions {
            branch: "main".to_string(),
            path: open_dir.clone(),
        },
    )
    .unwrap();

    let result = merge_branch(
        &repo_root,
        &MergeOptions {
            source_branch: "feature-session".to_string(),
            target_branch: "main".to_string(),
        },
    )
    .unwrap();

    assert_eq!(result.conflicts, vec!["src/session.py".to_string()]);
    let content = fs::read_to_string(open_dir.join("src").join("session.py")).unwrap();
    assert!(content.contains("<<<<<<< target"));
    assert!(content.contains("return 45"));
    assert!(content.contains("return 15"));
    assert!(content.contains(">>>>>>> source"));
}

#[test]
fn status_reads_python_compatible_open_directory() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();
    let open_dir = repo_root.join("main");
    let cf = repo_root.join(".codefire");
    fs::create_dir_all(&open_dir).unwrap();
    fs::create_dir_all(cf.join("opened")).unwrap();
    fs::create_dir_all(cf.join("active").join("open_123")).unwrap();

    let commit_id = write_valid_commit(&cf.join("objects"));
    fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": "main", "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();
    fs::write(
        repo_root.join(".codefire").join("opened").join("main.json"),
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "branch": {"name": "main"},
            "open": {
                "path": open_dir,
                "open_instance_id": "open_123",
                "opened_from_commit": commit_id,
                "current_base_commit": commit_id,
                "active_state_path": repo_root.join(".codefire").join("active").join("open_123")
            },
            "state": {"last_known": "open-consistent"}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        repo_root
            .join(".codefire")
            .join("active")
            .join("open_123")
            .join("fires.json"),
        serde_json::to_string_pretty(&json!([
            {"display_id": "FIRE-001", "status": "open"},
            {"display_id": "FIRE-002", "status": "extinguished"}
        ]))
        .unwrap(),
    )
    .unwrap();

    let status = read_status(&open_dir).unwrap();

    assert_eq!(
        status,
        Status {
            branch: "main".to_string(),
            state: "open-consistent".to_string(),
            base: commit_id,
            open_fires: 1
        }
    );
}

#[test]
fn branch_list_reads_repo_from_open_marker_and_validates_heads() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("worktree");
    let cf = repo_root.join(".codefire");
    fs::create_dir_all(cf.join("branches")).unwrap();
    fs::create_dir_all(&open_dir).unwrap();

    let commit_id = write_valid_commit(&cf.join("objects"));
    let branch_name = "feature/a b%雪";
    fs::write(
        cf.join("branches")
            .join(format!("{}.json", ref_file_name(branch_name))),
        serde_json::to_string_pretty(&json!({
            "type": "branch",
            "version": 1,
            "name": branch_name,
            "head": commit_id,
            "state": "open-clean",
            "created_at": "2026-06-04T00:00:00Z"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
            open_dir.join(".codefire-open"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "repository": {"path": cf, "repository_id": "repo_123"},
                "branch": {"name": branch_name, "opened_from_commit": commit_id},
                "open": {"open_instance_id": "open_123", "opened_at": "2026-06-04T00:00:00Z", "opened_path": open_dir}
            }))
            .unwrap(),
        )
        .unwrap();

    let branches = list_branches(&open_dir).unwrap();

    assert_eq!(
        branches,
        vec![Branch {
            name: branch_name.to_string(),
            head: commit_id,
            state: "open-clean".to_string()
        }]
    );
}

#[test]
fn ref_file_name_matches_python_quote_safe_empty() {
    assert_eq!(
        ref_file_name("feature/a b%雪"),
        "feature%2Fa%20b%25%E9%9B%AA"
    );
}

#[test]
fn unix_timestamp_format_uses_utc_iso_seconds() {
    assert_eq!(format_unix_seconds_utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(format_unix_seconds_utc(951_782_400), "2000-02-29T00:00:00Z");
}

#[test]
fn base64_decoder_handles_padding_and_rejects_invalid_input() {
    assert_eq!(decode_base64("aGVsbG8K").unwrap(), b"hello\n");
    assert_eq!(decode_base64("Zg==").unwrap(), b"f");
    assert!(decode_base64("Z===").is_err());
}

fn write_valid_commit(objects: &Path) -> String {
    let roots = write_required_roots(objects);
    let certificate = consistent_certificate();
    let commit = codefire_store::commit_payload(vec![], roots, certificate);
    codefire_store::store_object(objects, "commit", commit).unwrap()
}

fn write_file_commit(objects: &Path, path: &str, contents: &str) -> String {
    write_file_commit_with_parents(objects, path, contents, vec![])
}

fn write_file_commit_with_parents(
    objects: &Path,
    path: &str,
    contents: &str,
    parents: Vec<String>,
) -> String {
    let blob = codefire_store::store_object(
        objects,
        "blob",
        json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(contents.as_bytes()),
        }),
    )
    .unwrap();
    let manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({
            "type": "content_manifest",
            "version": 1,
            "entries": [{"path": path, "kind": "file", "mode": "100644", "blob": blob}],
        }),
    )
    .unwrap();
    let mut roots = write_required_roots(objects);
    roots.insert("content_manifest".to_string(), Value::String(manifest));
    codefire_store::store_object(
        objects,
        "commit",
        codefire_store::commit_payload(parents, roots, consistent_certificate()),
    )
    .unwrap()
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

fn write_required_roots(objects: &Path) -> Map<String, Value> {
    let content_manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": []}),
    )
    .unwrap();
    let atom_index = codefire_store::store_object(
        objects,
        "atom_index",
        json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
    )
    .unwrap();
    let trace_graph = codefire_store::store_object(
        objects,
        "trace_graph",
        json!({"type": "trace_graph", "version": 1, "links": []}),
    )
    .unwrap();
    let fire_delta = codefire_store::store_object(
        objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": []}),
    )
    .unwrap();
    let verification = codefire_store::store_object(
        objects,
        "verification",
        json!({"type": "verification", "version": 1, "checks": []}),
    )
    .unwrap();
    let policy = codefire_store::store_object(
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
