use super::*;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn markdown_atoms_match_heading_scope_and_python_hash_format() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("spec.md");
    fs::write(
            &path,
            "# Intro\n\n## REQ-AUTH-001: Session expiration\n\nBody\n\n### DES-NESTED-IGNORED: detail\n\nNested\n\n## REQ-AUTH-002 Next\nTail\n",
        )
        .unwrap();

    let atoms = extract_markdown_atoms(&path, "docs/spec/auth.md", "requirement").unwrap();

    assert_eq!(atoms.len(), 3);
    assert_eq!(atoms[0].atom_id, "REQ-AUTH-001");
    assert_eq!(atoms[0].kind, "requirement");
    assert_eq!(atoms[0].selector.selector_type, "markdown_heading");
    assert_eq!(
        atoms[0].content_hash,
        "sha256:c0bf4a8b5b28a4f811bea3dad3ed81c94931b3b93b7b300eed250531f09aab10"
    );
    assert_eq!(atoms[1].atom_id, "DES-NESTED-IGNORED");
    assert_eq!(atoms[1].kind, "design");
    assert_eq!(atoms[2].atom_id, "REQ-AUTH-002");
}

#[test]
fn markdown_atoms_ignore_headings_inside_fenced_code_blocks() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("spec.md");
    fs::write(
            &path,
            "## REQ-REAL-001: Real requirement\n\n```markdown\n## REQ-FAKE-001: sample only\n```\n\n   ~~~text\n### DES-FAKE-001: sample only\n   ~~~\n\n## REQ-REAL-002: Another requirement\n",
        )
        .unwrap();

    let atoms = extract_markdown_atoms(&path, "docs/spec/fenced.md", "requirement").unwrap();
    let ids = atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["REQ-REAL-001", "REQ-REAL-002"]);
}

#[test]
fn markdown_atoms_ignore_process_style_headings_without_explicit_marker() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("process.md");
    fs::write(
            &path,
            "## REQ-PROC-001: Process note\n\nNot a CodeFire atom.\n\n## PROC-REQ-001: Local process id\n\nAlso not an atom.\n\n## REQ-REAL-001: Real requirement\n\nBody\n",
        )
        .unwrap();

    let atoms = extract_markdown_atoms(&path, "docs/process.md", "requirement").unwrap();
    let ids = atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["REQ-REAL-001"]);

    let explicit_path = temp.path().join("explicit.md");
    fs::write(
        &explicit_path,
        "<!-- cf-atom: REQ-PROC-001 -->\n## REQ-PROC-001: Explicit process requirement\n",
    )
    .unwrap();
    let explicit =
        extract_explicit_cf_atoms(&explicit_path, "docs/explicit.md", "requirement").unwrap();
    assert_eq!(explicit[0].atom_id, "REQ-PROC-001");
}

#[test]
fn explicit_cf_atoms_extract_comment_markers_without_symbol_parsing() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("app.py");
    fs::write(
            &path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n# cf-atom: CODE:src/app.py::function:run\n",
        )
        .unwrap();

    let atoms = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

    assert_eq!(atoms.len(), 2);
    assert_eq!(atoms[0].atom_id, "CODE-SessionPolicy");
    assert_eq!(atoms[0].kind, "code");
    assert_eq!(atoms[0].selector.selector_type, "explicit_cf_atom");
    assert_eq!(atoms[1].atom_id, "CODE:src/app.py::function:run");
    assert_eq!(
        atoms[0].content_hash,
        "sha256:01e54e8f2a22283b0e25e51c1571921c2c51af29c5f8fc24517aecdbdaddce1f"
    );
}

#[test]
fn python_derived_atoms_restore_reference_ids_selectors_and_hashes() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("models.py");
    fs::write(
            &path,
            "class ExperimentPlan:\n    hypothesis: Hypothesis\n    benchmark: str\n    baseline: str\n    metric: str\n    estimated_cost: float\n    command: str = \"\"\n    artifact_paths: tuple[str, ...] = field(default_factory=tuple)\n    plan_id: str = \"\"\n\n    @property\n    def uid(self) -> str:\n        return self.plan_id or stable_id(\n            \"plan\",\n            self.hypothesis.uid,\n            self.benchmark,\n            self.baseline,\n            self.metric,\n            self.command,\n        )\n\n\n@dataclass(frozen=True)\nclass ExperimentResult:\n    pass\n",
        )
        .unwrap();

    let atoms = extract_python_atoms(&path, "src/evoagent/models.py", "code").unwrap();
    let class = atoms
        .iter()
        .find(|atom| atom.atom_id == "CODE:src/evoagent/models.py::class:ExperimentPlan")
        .unwrap();
    let method = atoms
        .iter()
        .find(|atom| atom.atom_id == "CODE:src/evoagent/models.py::method:ExperimentPlan.uid")
        .unwrap();

    assert_eq!(class.selector.selector_type, "python_class");
    assert_eq!(class.selector.value, "ExperimentPlan");
    assert_eq!(
        class.content_hash,
        "sha256:5f8966552eba40250bfc050d5823b8a43194c6ce18062771759e7fa3e2a73bd9"
    );
    assert_eq!(method.selector.selector_type, "python_method");
    assert_eq!(method.selector.value, "ExperimentPlan.uid");
}

#[test]
fn python_derived_atoms_distinguish_functions_and_class_owned_methods() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("planners.py");
    fs::write(
            &path,
            "def build_planner():\n    return StaticPlanner()\n\n\nclass StaticPlanner:\n    def propose(self):\n        return []\n\n\nclass CodexPlanner:\n    def propose(self):\n        return []\n",
        )
        .unwrap();

    let atoms = extract_python_atoms(&path, "src/planners.py", "code").unwrap();
    let ids = atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"CODE:src/planners.py::def:build_planner"));
    assert!(ids.contains(&"CODE:src/planners.py::class:StaticPlanner"));
    assert!(ids.contains(&"CODE:src/planners.py::method:StaticPlanner.propose"));
    assert!(ids.contains(&"CODE:src/planners.py::class:CodexPlanner"));
    assert!(ids.contains(&"CODE:src/planners.py::method:CodexPlanner.propose"));
}

#[test]
fn python_test_symbols_use_the_reference_test_prefix() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("test_planner.py");
    fs::write(
        &path,
        "class PlannerTests:\n    def test_plan(self):\n        assert True\n",
    )
    .unwrap();

    let atoms = extract_python_atoms(&path, "tests/test_planner.py", "test").unwrap();
    let ids = atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"TEST:tests/test_planner.py::class:PlannerTests"));
    assert!(ids.contains(&"TEST:tests/test_planner.py::method:PlannerTests.test_plan"));
    assert!(atoms.iter().all(|atom| atom.kind == "test"));
}

#[test]
fn python_explicit_marker_keeps_async_function_body_ownership() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("async_worker.py");
    fs::write(
        &path,
        "# cf-atom: CODE-AsyncWorker\nasync def run_worker():\n    return 1\n",
    )
    .unwrap();
    let before = extract_python_atoms(&path, "src/async_worker.py", "code").unwrap();
    fs::write(
        &path,
        "# cf-atom: CODE-AsyncWorker\nasync def run_worker():\n    return 2\n",
    )
    .unwrap();
    let after = extract_python_atoms(&path, "src/async_worker.py", "code").unwrap();

    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 1);
    assert_eq!(before[0].atom_id, "CODE-AsyncWorker");
    assert_eq!(before[0].selector.selector_type, "python_def");
    assert_eq!(before[0].selector.value, "run_worker");
    assert_ne!(before[0].content_hash, after[0].content_hash);
}

#[test]
fn python_explicit_marker_owns_only_nearest_symbol_once() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("owned.py");
    fs::write(
            &path,
            "# cf-atom: CODE-UnusedMarker\n# cf-atom: CODE-OwnedClass\n@dataclass\nclass Owned:\n    def run(self):\n        return True\n",
        )
        .unwrap();

    let atoms = extract_python_atoms(&path, "src/owned.py", "code").unwrap();
    let owned = atoms
        .iter()
        .find(|atom| atom.atom_id == "CODE-OwnedClass")
        .unwrap();

    assert_eq!(owned.selector.selector_type, "python_class");
    assert_eq!(owned.selector.value, "Owned");
    assert!(atoms.iter().any(|atom| atom.atom_id == "CODE-UnusedMarker"
        && atom.selector.selector_type == "python_comment"));
    assert!(atoms
        .iter()
        .any(|atom| atom.atom_id == "CODE:src/owned.py::method:Owned.run"));
    assert!(!atoms
        .iter()
        .any(|atom| atom.atom_id == "CODE:src/owned.py::class:Owned"));
}

#[test]
fn python_explicit_marker_ownership_window_is_bounded() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("bounded.py");
    fs::write(
            &path,
            "# cf-atom: CODE-Standalone\n# first gap\n# second gap\n# third gap\nclass FourLinesAway:\n    pass\n",
        )
        .unwrap();

    let atoms = extract_python_atoms(&path, "src/bounded.py", "code").unwrap();

    assert!(atoms
        .iter()
        .any(|atom| atom.atom_id == "CODE-Standalone"
            && atom.selector.selector_type == "python_comment"));
    assert!(atoms
        .iter()
        .any(|atom| atom.atom_id == "CODE:src/bounded.py::class:FourLinesAway"));
}

#[test]
fn python_lexical_view_ignores_declarations_and_markers_inside_strings() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("test_nameaudit.py");
    fs::write(
        &path,
        r##"class AuditTests:
    def test_scan(self):
        inline = "class InlineFixture:"
        source = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
# cf-atom: TEST-FakeMarker
"""
        raw_source = rf'''
async def false_async_fixture():
    return "ignored"
'''
        assert inline and source and raw_source

# cf-atom: TEST-RealFunction
def real_function():
    return True
"##,
    )
    .unwrap();

    let atoms = extract_python_atoms(&path, "tests/test_nameaudit.py", "test").unwrap();
    let ids = atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"TEST:tests/test_nameaudit.py::class:AuditTests"));
    assert!(ids.contains(&"TEST:tests/test_nameaudit.py::method:AuditTests.test_scan"));
    assert!(ids.contains(&"TEST-RealFunction"));
    assert!(!ids.iter().any(|atom_id| {
        atom_id.contains("InlineFixture")
            || atom_id.contains("ExcessivelyLongClassName")
            || atom_id.contains("excessively_long_method")
            || atom_id.contains("false_async_fixture")
            || *atom_id == "TEST-FakeMarker"
    }));
}

#[test]
fn python_derived_and_explicit_atoms_keep_duplicate_detection() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("src/models.py"),
        "class ExperimentPlan:\n    pass\n",
    );
    write_file(
        &temp.path().join("src/duplicate.py"),
        "# cf-atom: CODE:src/models.py::class:ExperimentPlan\n",
    );

    let index = build_atom_index(temp.path()).unwrap();

    assert_eq!(index.hash_schema_version, 4);
    assert_eq!(
        index.duplicate_atom_ids,
        vec!["CODE:src/models.py::class:ExperimentPlan"]
    );
}

#[test]
fn python_symbol_extraction_preserves_invalid_marker_diagnostics() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("invalid.py");
    fs::write(
        &path,
        "# cf-atom: code-invalid\nclass StillNotExtracted:\n    pass\n",
    )
    .unwrap();

    let error = extract_python_atoms(&path, "src/invalid.py", "code").unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid Atom ID in src/invalid.py:1: code-invalid"
    );
}

#[test]
fn explicit_cf_atom_body_changes_update_content_hash_for_python_and_rust() {
    let temp = tempdir().unwrap();
    let python_path = temp.path().join("app.py");
    fs::write(
        &python_path,
        "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    timeout = 30\n",
    )
    .unwrap();
    let first_python =
        extract_explicit_cf_atoms(&python_path, "src/app.py", "code").unwrap()[0].clone();
    fs::write(
        &python_path,
        "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    timeout = 60\n",
    )
    .unwrap();
    let second_python =
        extract_explicit_cf_atoms(&python_path, "src/app.py", "code").unwrap()[0].clone();

    let rust_path = temp.path().join("lib.rs");
    fs::write(
        &rust_path,
        "// cf-atom: CODE-RustPolicy\nfn timeout() -> u32 {\n    30\n}\n",
    )
    .unwrap();
    let first_rust =
        extract_explicit_cf_atoms(&rust_path, "src/lib.rs", "code").unwrap()[0].clone();
    fs::write(
        &rust_path,
        "// cf-atom: CODE-RustPolicy\nfn timeout() -> u32 {\n    60\n}\n",
    )
    .unwrap();
    let second_rust =
        extract_explicit_cf_atoms(&rust_path, "src/lib.rs", "code").unwrap()[0].clone();

    assert_eq!(first_python.atom_id, second_python.atom_id);
    assert_ne!(first_python.content_hash, second_python.content_hash);
    assert_eq!(first_rust.atom_id, second_rust.atom_id);
    assert_ne!(first_rust.content_hash, second_rust.content_hash);
}

#[test]
fn python_explicit_cf_atom_hash_is_stable_when_later_marker_is_added() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("app.py");
    fs::write(
            &path,
            "# cf-atom: CODE-ExistingPolicy\nclass ExistingPolicy:\n    def allowed(self) -> bool:\n        return True\n\n\ndef helper() -> int:\n    return 1\n",
        )
        .unwrap();
    let before = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();
    fs::write(
            &path,
            "# cf-atom: CODE-ExistingPolicy\nclass ExistingPolicy:\n    def allowed(self) -> bool:\n        return True\n\n\n# cf-atom: CODE-Helper\ndef helper() -> int:\n    return 1\n",
        )
        .unwrap();
    let after = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 2);
    assert_eq!(before[0].atom_id, "CODE-ExistingPolicy");
    assert_eq!(after[0].atom_id, "CODE-ExistingPolicy");
    assert_eq!(before[0].content_hash, after[0].content_hash);
    assert_eq!(after[1].atom_id, "CODE-Helper");
}

#[test]
fn python_explicit_cf_atom_hash_tracks_symbol_body_changes() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("test_app.py");
    fs::write(
            &path,
            "# cf-atom: TEST-existing-policy\ndef test_existing_policy():\n    assert True\n\n\ndef unrelated_helper():\n    return 1\n",
        )
        .unwrap();
    let before = extract_explicit_cf_atoms(&path, "tests/test_app.py", "test").unwrap();
    fs::write(
            &path,
            "# cf-atom: TEST-existing-policy\ndef test_existing_policy():\n    assert False\n\n\ndef unrelated_helper():\n    return 1\n",
        )
        .unwrap();
    let after = extract_explicit_cf_atoms(&path, "tests/test_app.py", "test").unwrap();

    assert_eq!(before[0].atom_id, after[0].atom_id);
    assert_ne!(before[0].content_hash, after[0].content_hash);
}

#[test]
fn invalid_explicit_cf_atom_id_is_reported_with_location() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("app.py");
    fs::write(&path, "# cf-atom: code-lowercase\n").unwrap();

    let error = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid Atom ID in src/app.py:1: code-lowercase"
    );
}

#[test]
fn atom_id_validator_is_shared_by_heading_and_explicit_forms() {
    assert!(is_valid_atom_id("REQ-AUTH-001"));
    assert!(is_valid_atom_id("CODE-SessionPolicy"));
    assert!(is_valid_atom_id("CODE:src/app.py::method:Store.find"));
    assert!(!is_valid_atom_id("code-lowercase"));
    assert!(!is_valid_atom_id("REQ-"));
    assert!(!is_valid_atom_id("CODE:src/app.py::method:bad name"));
}

#[test]
fn build_atom_index_uses_defaults_config_and_detects_duplicates() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("docs/spec/auth.md"),
        "## REQ-AUTH-001: Session expiration\n\nRequirement.\n",
    );
    write_file(
        &temp.path().join("docs/design/auth.md"),
        "## DES-AUTH-001: Session design\n\nDesign.\n",
    );
    write_file(
        &temp.path().join("src/app.py"),
        "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n",
    );
    write_file(
        &temp.path().join("tests/test_app.py"),
        "# cf-atom: TEST-session-policy\ndef test_session_policy():\n    pass\n",
    );
    write_file(
        &temp.path().join("src/duplicate.py"),
        "# cf-atom: CODE-SessionPolicy\n",
    );

    let index = build_atom_index(temp.path()).unwrap();
    let ids = index
        .atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(index.type_tag, "atom_index");
    assert_eq!(
        ids,
        vec![
            "CODE-SessionPolicy",
            "CODE-SessionPolicy",
            "DES-AUTH-001",
            "REQ-AUTH-001",
            "TEST-session-policy",
        ]
    );
    assert_eq!(index.duplicate_atom_ids, vec!["CODE-SessionPolicy"]);
}

#[test]
fn codefire_yaml_overrides_artifact_patterns() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("codefire.yaml"),
        "requirements:\n  - path: product/*.md\ncode:\n  - path: lib/*.rs\n",
    );
    write_file(
        &temp.path().join("product/auth.md"),
        "## REQ-CUSTOM-001: Custom requirement\n",
    );
    write_file(
        &temp.path().join("lib/main.rs"),
        "// cf-atom: CODE-RustMain\n",
    );

    let index = build_atom_index(temp.path()).unwrap();
    let ids = index
        .atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["CODE-RustMain", "REQ-CUSTOM-001"]);
}

#[test]
fn trace_graph_parses_links_and_uses_bounded_digest_ids() {
    let temp = tempdir().unwrap();
    write_file(
            &temp.path().join("codefire.links.yaml"),
            "version: 1\nlinks:\n  - from: 'REQ-AUTH-001' # comment\n    to: \"DES-AUTH-001\"\n    type: refined_by\n",
        );

    let graph = current_trace_graph(temp.path()).unwrap();

    assert_eq!(graph.type_tag, "trace_graph");
    assert_eq!(graph.links.len(), 1);
    assert_eq!(graph.links[0].from, "REQ-AUTH-001");
    assert_eq!(graph.links[0].to, "DES-AUTH-001");
    assert_eq!(graph.links[0].link_type, "refined_by");
    assert!(graph.links[0].link_id.starts_with("LINK-sha256-"));
    assert_eq!(graph.links[0].link_id.len(), "LINK-sha256-".len() + 32);
    assert_eq!(
        graph.links[0].link_hash,
        trace_link_hash(
            "REQ-AUTH-001",
            "DES-AUTH-001",
            "refined_by",
            &graph.links[0].link_id,
        )
        .unwrap()
    );
}

#[test]
fn trace_graph_reports_missing_required_link_fields() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("codefire.links.yaml"),
        "links:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n",
    );

    let error = current_trace_graph(temp.path()).unwrap_err();

    assert!(matches!(error, CoreError::Config(message) if message.contains("missing type")));
}

#[test]
fn trace_graph_rejects_unsupported_links_yaml() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("codefire.links.yaml"),
        "version: 1\nmetadata:\n  owner: docs\nlinks:\n  - from: REQ-AUTH-001\n",
    );

    let error = current_trace_graph(temp.path()).unwrap_err();

    assert!(
        matches!(error, CoreError::Config(message) if message.contains("unsupported codefire.links.yaml: root key metadata at line 2"))
    );
}

#[test]
fn trace_graph_is_empty_when_links_file_is_absent() {
    let temp = tempdir().unwrap();

    let graph = current_trace_graph(temp.path()).unwrap();

    assert!(graph.links.is_empty());
}

#[test]
fn trace_graph_endpoint_validator_reports_missing_source() {
    let target = atom("DES-AUTH-001", "design", "sha256:design");
    let link = TraceLinkInput {
        from: "REQ-MISSING-001".to_string(),
        to: Some(target.atom_id.clone()),
        link_type: Some("refined_by".to_string()),
    }
    .into_trace_link()
    .unwrap();
    let link_id = link.link_id.clone();
    let atom_index = atom_index(vec![target]);
    let trace_graph = trace_graph(vec![link]);

    let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

    assert_eq!(
            error.to_string(),
            format!(
                "trace graph endpoint integrity failed: 1 issue(s); link {link_id} from endpoint REQ-MISSING-001 does not resolve to an Atom"
            )
        );
    assert!(matches!(
        error,
        CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
            issues,
            total_issues: 1,
            omitted_issues: 0,
        })
            if issues == vec![TraceGraphEndpointIssue {
                link_index: 0,
                link_id,
                link_id_truncated: false,
                link_id_omitted_bytes: 0,
                endpoint: TraceGraphEndpointRole::From,
                atom_id: "REQ-MISSING-001".to_string(),
                atom_id_truncated: false,
                atom_id_omitted_bytes: 0,
                kind: TraceGraphEndpointIssueKind::MissingAtom,
                atom_occurrences: 0,
            }]
    ));
}

#[test]
fn trace_graph_endpoint_validator_reports_missing_target() {
    let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
    let link = TraceLinkInput {
        from: source.atom_id.clone(),
        to: Some("DES-MISSING-001".to_string()),
        link_type: Some("refined_by".to_string()),
    }
    .into_trace_link()
    .unwrap();
    let link_id = link.link_id.clone();
    let atom_index = atom_index(vec![source]);
    let trace_graph = trace_graph(vec![link]);

    let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

    assert!(matches!(
        error,
        CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
            issues,
            total_issues: 1,
            omitted_issues: 0,
        })
            if issues == vec![TraceGraphEndpointIssue {
                link_index: 0,
                link_id,
                link_id_truncated: false,
                link_id_omitted_bytes: 0,
                endpoint: TraceGraphEndpointRole::To,
                atom_id: "DES-MISSING-001".to_string(),
                atom_id_truncated: false,
                atom_id_omitted_bytes: 0,
                kind: TraceGraphEndpointIssueKind::MissingAtom,
                atom_occurrences: 0,
            }]
    ));
}

#[test]
fn trace_graph_endpoint_validator_reports_duplicate_atom_id_ambiguity() {
    let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
    let duplicate_target = atom("DES-AUTH-001", "design", "sha256:design-a");
    let link = TraceLinkInput {
        from: source.atom_id.clone(),
        to: Some(duplicate_target.atom_id.clone()),
        link_type: Some("refined_by".to_string()),
    }
    .into_trace_link()
    .unwrap();
    let link_id = link.link_id.clone();
    let atom_index = atom_index(vec![
        source,
        duplicate_target,
        atom("DES-AUTH-001", "design", "sha256:design-b"),
    ]);
    let trace_graph = trace_graph(vec![link]);

    let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

    assert!(matches!(
        error,
        CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
            issues,
            total_issues: 1,
            omitted_issues: 0,
        })
            if issues == vec![TraceGraphEndpointIssue {
                link_index: 0,
                link_id,
                link_id_truncated: false,
                link_id_omitted_bytes: 0,
                endpoint: TraceGraphEndpointRole::To,
                atom_id: "DES-AUTH-001".to_string(),
                atom_id_truncated: false,
                atom_id_omitted_bytes: 0,
                kind: TraceGraphEndpointIssueKind::AmbiguousAtomId,
                atom_occurrences: 2,
            }]
    ));
}

#[test]
fn trace_graph_endpoint_validator_accepts_uniquely_resolved_graph() {
    let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
    let target = atom("DES-AUTH-001", "design", "sha256:design");
    let link = TraceLinkInput {
        from: source.atom_id.clone(),
        to: Some(target.atom_id.clone()),
        link_type: Some("refined_by".to_string()),
    }
    .into_trace_link()
    .unwrap();
    let atom_index = atom_index(vec![source, target]);
    let trace_graph = trace_graph(vec![link]);

    validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap();
}

#[test]
fn trace_graph_endpoint_validator_bounds_and_orders_large_diagnostics() {
    let link_count = TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT + 37;
    let links = (0..link_count)
        .rev()
        .map(|index| TraceLink {
            from: format!("REQ-MISSING-{index:04}"),
            to: format!("DES-MISSING-{index:04}"),
            link_type: "refined_by".to_string(),
            link_id: format!("LINK-{index:04}"),
            link_hash: format!("sha256:{index:04}"),
        })
        .collect::<Vec<_>>();

    let error =
        validate_trace_graph_endpoints(&atom_index(Vec::new()), &trace_graph(links)).unwrap_err();
    let CoreError::TraceGraphEndpointIntegrity(error) = error else {
        panic!("expected trace graph endpoint integrity error");
    };

    assert_eq!(error.issues.len(), TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT);
    assert_eq!(error.total_issues, link_count * 2);
    assert_eq!(
        error.omitted_issues,
        link_count * 2 - TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT
    );
    assert!(error
        .issues
        .windows(2)
        .all(|pair| compare_endpoint_issues(&pair[0], &pair[1]).is_lt()));
    assert_eq!(error.issues.first().unwrap().link_id, "LINK-0000");
    assert!(error.to_string().ends_with(&format!(
        "{} additional issue(s) omitted",
        error.omitted_issues
    )));

    // Display is defensive even if an external caller constructs an overfull error value.
    let mut overfull_error = error.clone();
    let repeated_issue = overfull_error.issues[0].clone();
    for _ in 0..7 {
        overfull_error.issues.push(repeated_issue.clone());
    }
    overfull_error.total_issues = overfull_error.issues.len();
    overfull_error.omitted_issues = 0;
    let rendered = overfull_error.to_string();
    assert_eq!(
        rendered.matches("; link ").count(),
        TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT
    );
    assert!(rendered.ends_with("7 additional issue(s) omitted"));
}

#[test]
fn trace_graph_endpoint_validator_byte_bounds_utf8_diagnostic_samples() {
    let long_link_id = format!("LINK-{}{}", "a".repeat(250), "界".repeat(20));
    let long_atom_id = format!("REQ-{}{}", "b".repeat(251), "界".repeat(20));
    let (atom_prefix, _) = bounded_endpoint_sample(&long_atom_id);
    let target = atom("DES-target", "design", "sha256:target");
    let graph = trace_graph(vec![TraceLink {
        from: long_atom_id.clone(),
        to: target.atom_id.clone(),
        link_type: "refined_by".to_string(),
        link_id: long_link_id.clone(),
        link_hash: "sha256:long-diagnostic".to_string(),
    }]);
    // A bounded display prefix must not become the endpoint identity used by validation.
    let index = atom_index(vec![
        atom(&atom_prefix, "requirement", "sha256:prefix-only"),
        target,
    ]);

    let CoreError::TraceGraphEndpointIntegrity(error) =
        validate_trace_graph_endpoints(&index, &graph).unwrap_err()
    else {
        panic!("expected trace graph endpoint integrity error");
    };
    let issue = &error.issues[0];

    assert_eq!(error.total_issues, 1);
    assert!(issue.link_id_truncated);
    assert!(issue.atom_id_truncated);
    assert!(issue.link_id.len() <= TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT);
    assert!(issue.atom_id.len() <= TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT);
    assert_eq!(
        issue.link_id_omitted_bytes,
        long_link_id.len() - issue.link_id.len()
    );
    assert_eq!(
        issue.atom_id_omitted_bytes,
        long_atom_id.len() - issue.atom_id.len()
    );
    assert!(std::str::from_utf8(issue.link_id.as_bytes()).is_ok());
    assert!(std::str::from_utf8(issue.atom_id.as_bytes()).is_ok());
    assert!(error.to_string().len() < 1_024);
    assert!(serde_json::to_vec(&error).unwrap().len() < 2_048);
}

#[test]
fn required_link_missing_uses_default_policy() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("docs/spec/auth.md"),
        "## REQ-AUTH-001: Session expiration\n",
    );
    write_file(
        &temp.path().join("docs/design/auth.md"),
        "## DES-AUTH-001: Session design\n",
    );
    write_file(
        &temp.path().join("src/app.py"),
        "# cf-atom: CODE-SessionPolicy\n",
    );
    write_file(
        &temp.path().join("codefire.links.yaml"),
        "links:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n    type: refined_by\n",
    );

    let missing = current_required_link_missing(temp.path()).unwrap();

    assert_eq!(missing.len(), 2);
    assert!(missing.iter().any(|item| item.atom_id == "REQ-AUTH-001"
        && item.required_type == "verified_by"
        && item.target_kind == "test"));
    assert!(missing.iter().any(|item| item.atom_id == "DES-AUTH-001"
        && item.required_type == "implemented_by"
        && item.target_kind == "code"));
}

#[test]
fn required_link_missing_uses_custom_policy() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("docs/adr/platform.md"),
        "## ADR-PLATFORM-001: Runtime choice\n",
    );
    write_file(
        &temp.path().join("docs/ops/deploy.md"),
        "## OPS-DEPLOY-001: Runtime deployment\n",
    );
    write_file(
        &temp.path().join("codefire.policy.yaml"),
        "required_links:\n  adr:\n    - type: operated_by\n      target_kind: ops\n      min: 1\n",
    );

    let missing = current_required_link_missing(temp.path()).unwrap();

    assert_eq!(
        missing,
        vec![MissingRequiredLink {
            atom_id: "ADR-PLATFORM-001".to_string(),
            required_type: "operated_by".to_string(),
            target_kind: "ops".to_string(),
            min: 1,
            found: 0,
        }]
    );
}

#[test]
fn required_link_missing_counts_large_fixture_with_link_index() {
    let mut atoms = Vec::new();
    let mut links = Vec::new();
    for index in 0..256 {
        let requirement = format!("REQ-LARGE-{index:03}");
        let design = format!("DES-LARGE-{index:03}");
        atoms.push(atom(&requirement, "requirement", "sha256:req"));
        atoms.push(atom(&design, "design", "sha256:des"));
        links.push(
            TraceLinkInput {
                from: requirement,
                to: Some(design),
                link_type: Some("refined_by".to_string()),
            }
            .into_trace_link()
            .unwrap(),
        );
    }
    let atom_index = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms,
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links,
    };
    let policy = TracePolicy {
        required_links: BTreeMap::from([(
            "requirement".to_string(),
            vec![RequiredLinkRule {
                link_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
            }],
        )]),
    };

    let missing = required_link_missing(&atom_index, &trace_graph, &policy);

    assert!(missing.is_empty());
}

#[test]
fn trace_index_required_links_match_reference_on_graph_edge_cases() {
    let atom_index = atom_index(vec![
        atom("REQ-SOURCE", "requirement", "sha256:source"),
        atom("TARGET", "design", "sha256:target-design"),
        // The legacy evaluator's BTreeMap collection makes the last
        // duplicate kind authoritative before corruption is rejected by
        // its owning validation path. The index must not alter that fact.
        atom("TARGET", "test", "sha256:target-test"),
        atom("DES-A", "design", "sha256:a"),
        atom("TEST-B", "test", "sha256:b"),
    ]);
    let make_link = |from: &str, to: &str, link_type: &str| {
        TraceLinkInput {
            from: from.to_string(),
            to: Some(to.to_string()),
            link_type: Some(link_type.to_string()),
        }
        .into_trace_link()
        .unwrap()
    };
    let trace_graph = trace_graph(vec![
        make_link("REQ-SOURCE", "TARGET", "verified_by"),
        make_link("REQ-SOURCE", "TARGET", "verified_by"),
        make_link("REQ-SOURCE", "REQ-SOURCE", "self"),
        make_link("REQ-SOURCE", "DES-A", "refined_by"),
        make_link("DES-A", "TEST-B", "verified_by"),
        make_link("TEST-B", "REQ-SOURCE", "closes_cycle"),
        make_link("REQ-SOURCE", "TEST-B", "verified_by"),
    ]);
    let policy = TracePolicy {
        required_links: BTreeMap::from([(
            "requirement".to_string(),
            vec![
                RequiredLinkRule {
                    link_type: "verified_by".to_string(),
                    target_kind: "test".to_string(),
                    min: 4,
                },
                RequiredLinkRule {
                    link_type: "self".to_string(),
                    target_kind: "requirement".to_string(),
                    min: 1,
                },
            ],
        )]),
    };

    let reference = required_link_missing_reference(&atom_index, &trace_graph, &policy);
    let (indexed, counters) = required_link_missing_with_work(&atom_index, &trace_graph, &policy);

    assert_eq!(indexed, reference);
    assert_eq!(
        required_link_missing(&atom_index, &trace_graph, &policy),
        indexed
    );
    assert_eq!(
        indexed,
        vec![MissingRequiredLink {
            atom_id: "REQ-SOURCE".to_string(),
            required_type: "verified_by".to_string(),
            target_kind: "test".to_string(),
            min: 4,
            found: 3,
        }]
    );
    assert_eq!(counters.atoms_indexed, atom_index.atoms.len());
    assert_eq!(counters.links_indexed, trace_graph.links.len());
    assert_eq!(counters.rules_evaluated, 2);
}

#[test]
fn trace_work_counters_have_linear_1x_3x_10x_slope() {
    for scale in [1_usize, 3, 10] {
        let mut atoms = Vec::with_capacity(scale * 2);
        let mut links = Vec::with_capacity(scale);
        for index in 0..scale {
            let requirement = format!("REQ-SCALE-{index:03}");
            let design = format!("DES-SCALE-{index:03}");
            atoms.push(atom(&requirement, "requirement", "sha256:req"));
            atoms.push(atom(&design, "design", "sha256:des"));
            links.push(
                TraceLinkInput {
                    from: requirement,
                    to: Some(design),
                    link_type: Some("refined_by".to_string()),
                }
                .into_trace_link()
                .unwrap(),
            );
        }
        let atom_index = atom_index(atoms);
        let trace_graph = trace_graph(links);
        let policy = TracePolicy {
            required_links: BTreeMap::from([(
                "requirement".to_string(),
                vec![RequiredLinkRule {
                    link_type: "refined_by".to_string(),
                    target_kind: "design".to_string(),
                    min: 2,
                }],
            )]),
        };
        let mut evaluator = TraceEvaluationIndex::compile(&atom_index, &trace_graph);
        for atom in &atom_index.atoms {
            let _ = evaluator.adjacent_atoms(&atom.atom_id);
        }
        let missing = evaluator.missing_required_links(&policy);

        assert_eq!(missing.len(), scale);
        assert_eq!(
            evaluator.counters(),
            TraceWorkCounters {
                atoms_indexed: 2 * scale,
                rules_evaluated: scale,
                links_indexed: scale,
                index_entries: 5 * scale,
                lookups: 6 * scale,
                adjacency_visits: 2 * scale,
            }
        );
    }
}

#[test]
fn verification_policy_parses_commit_booleans_and_commands() {
    let temp = tempdir().unwrap();
    write_file(
            &temp.path().join("codefire.policy.yaml"),
            "commit_policy:\n  require_no_required_fires: true\n  require_trace_completeness: false\n  reject_duplicate_atom_ids: true\nverification:\n  - id: unit\n    command: cargo test\n    cwd: crates/app\n",
        );

    let policy = parse_verification_policy(temp.path()).unwrap();

    assert!(policy.require_no_required_fires);
    assert!(policy.reject_duplicate_atom_ids);
    assert!(!policy.require_trace_completeness);
    assert_eq!(
        policy.verification,
        vec![VerificationCommand {
            id: "unit".to_string(),
            command: "cargo test".to_string(),
            cwd: "crates/app".to_string(),
            timeout_ms: DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS,
            max_output_bytes: DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES,
            inherit_env: false,
            env: BTreeMap::new(),
            allow_failure: false,
        }]
    );
}

#[test]
fn verification_policy_rejects_safety_floor_relaxation() {
    for field in [
        "require_no_required_fires",
        "require_no_stale_resolutions",
        "require_verification_success",
        "reject_duplicate_atom_ids",
    ] {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            &format!("commit_policy:\n  {field}: false\n"),
        );

        let error = parse_verification_policy(temp.path()).unwrap_err();

        assert!(error
            .to_string()
            .contains(&format!("commit_policy.{field} must be true")));
    }
}

#[test]
fn verification_policy_accepts_legacy_required_wrapper() {
    let temp = tempdir().unwrap();
    write_file(
            &temp.path().join("codefire.policy.yaml"),
            "verification:\n  required:\n    - id: unit\n      command: cargo test\n      cwd: crates/app\n",
        );

    let policy = parse_verification_policy(temp.path()).unwrap();

    assert_eq!(
        policy.verification,
        vec![VerificationCommand {
            id: "unit".to_string(),
            command: "cargo test".to_string(),
            cwd: "crates/app".to_string(),
            timeout_ms: DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS,
            max_output_bytes: DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES,
            inherit_env: false,
            env: BTreeMap::new(),
            allow_failure: false,
        }]
    );
}

#[test]
fn verification_policy_accepts_cli_owned_commit_signature_requirement() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("codefire.policy.yaml"),
        "commit_policy:\n  require_commit_signature: true\n",
    );

    let policy = parse_verification_policy(temp.path()).unwrap();

    assert!(policy.require_commit_signature);
    assert_eq!(
        serde_json::to_value(&policy).unwrap()["require_commit_signature"],
        true
    );
    let default_policy_json = serde_json::to_value(default_verification_policy()).unwrap();
    assert!(default_policy_json
        .get("require_commit_signature")
        .is_none());
    let legacy_policy: VerificationPolicy = serde_json::from_value(default_policy_json).unwrap();
    assert!(!legacy_policy.require_commit_signature);
}

#[test]
fn verification_policy_parses_command_execution_contract() {
    let temp = tempdir().unwrap();
    write_file(
            &temp.path().join("codefire.policy.yaml"),
            "verification:\n  - id: lint\n    command: cargo clippy\n    timeout_ms: 120000\n    max_output_bytes: 4096\n    inherit_env: false\n    env: RUST_BACKTRACE=1,CARGO_TERM_COLOR=never\n    allow_failure: true\n",
        );

    let policy = parse_verification_policy(temp.path()).unwrap();

    assert_eq!(policy.verification[0].timeout_ms, 120_000);
    assert_eq!(policy.verification[0].max_output_bytes, 4096);
    assert!(!policy.verification[0].inherit_env);
    assert_eq!(
        policy.verification[0].env.get("RUST_BACKTRACE"),
        Some(&"1".to_string())
    );
    assert_eq!(
        policy.verification[0].env.get("CARGO_TERM_COLOR"),
        Some(&"never".to_string())
    );
    assert!(policy.verification[0].allow_failure);
}

#[test]
fn verification_policy_rejects_unsupported_yaml_fields() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("codefire.policy.yaml"),
        "commit_policy:\n  require_no_required_fires: true\n  typo_field: true\n",
    );

    let error = parse_verification_policy(temp.path()).unwrap_err();

    assert!(
        matches!(error, CoreError::Config(message) if message.contains("unsupported codefire.policy.yaml: commit_policy field typo_field at line 3"))
    );
}

#[test]
fn stale_resolutions_accept_legacy_trace_link_basis() {
    let policy = default_verification_policy();
    let atom_index = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req1"),
            atom("DES-AUTH-001", "design", "sha256:des1"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };
    let legacy_id = legacy_trace_link_id("REQ-AUTH-001", "DES-AUTH-001", "refined_by");
    let legacy_hash =
        trace_link_hash("REQ-AUTH-001", "DES-AUTH-001", "refined_by", &legacy_id).unwrap();
    let resolution = Resolution {
        type_tag: "resolution".to_string(),
        version: VERSION,
        resolution_uid: "res_legacy".to_string(),
        fire_uid: "fire_legacy".to_string(),
        resolution_type: "addressed".to_string(),
        rationale: "checked".to_string(),
        evidence: String::new(),
        evidence_refs: Vec::new(),
        basis: ResolutionBasis {
            source_atom: ResolutionAtomBasis {
                atom_id: "REQ-AUTH-001".to_string(),
                content_hash: Some("sha256:req1".to_string()),
            },
            target_atom: ResolutionAtomBasis {
                atom_id: "DES-AUTH-001".to_string(),
                content_hash: Some("sha256:des1".to_string()),
            },
            trace_links: vec![ResolutionTraceLinkBasis {
                link_id: legacy_id,
                link_hash: legacy_hash,
            }],
            policy_hash: policy_hash(&policy).unwrap(),
        },
        resolved_at: "2026-06-07T00:00:00Z".to_string(),
        status: "active".to_string(),
    };

    let stale = stale_resolutions(&atom_index, &trace_graph, &policy, &[resolution]).unwrap();

    assert!(stale.is_empty());
}

#[test]
fn committable_decision_enforces_safety_floor_and_trace_warning() {
    let scan = ScanResult {
        type_tag: "scan".to_string(),
        version: VERSION,
        base_commit: "CF-COMMIT-base".to_string(),
        atom_index: AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req"),
                atom("REQ-AUTH-001", "requirement", "sha256:req-copy"),
            ],
            duplicate_atom_ids: vec!["REQ-AUTH-001".to_string()],
        },
        trace_graph: TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: Vec::new(),
        },
        changed_atoms: Vec::new(),
        non_atom_changed_files: Vec::new(),
        change_coverage: None,
        open_fires: vec![Fire {
            type_tag: "fire".to_string(),
            version: VERSION,
            fire_uid: "fire_required".to_string(),
            display_id: "FIRE-001".to_string(),
            status: "open".to_string(),
            severity: "required".to_string(),
            source: FireAtomRef {
                atom_id: "REQ-AUTH-001".to_string(),
                content_hash_at_fire: None,
            },
            target: FireAtomRef {
                atom_id: "REQ-AUTH-001".to_string(),
                content_hash_at_fire: None,
            },
            reason: "characterization".to_string(),
            trace_path: Vec::new(),
            created_by: "test".to_string(),
            created_at: "2026-06-04T00:00:00Z".to_string(),
            key: "required".to_string(),
            obsolete_at: None,
            resolution_uid: None,
        }],
        tool_migration: None,
    };
    let policy = VerificationPolicy {
        require_no_required_fires: false,
        require_no_stale_resolutions: false,
        require_trace_completeness: false,
        require_verification_success: false,
        reject_duplicate_atom_ids: false,
        ..default_verification_policy()
    };

    let verification = build_verification(
        &scan,
        VerificationInputs {
            unresolved_merge_conflicts: Vec::new(),
            missing_required_links: vec![MissingRequiredLink {
                atom_id: "REQ-AUTH-001".to_string(),
                required_type: "verified_by".to_string(),
                target_kind: "test".to_string(),
                min: 1,
                found: 0,
            }],
            failed_checks: vec![FailedCheck {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                output: "failed".to_string(),
            }],
            stale_resolutions: vec![StaleResolution {
                resolution_uid: "resolution_1".to_string(),
                reason: "source changed".to_string(),
                ..Default::default()
            }],
            missing_evidence_refs: vec![MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            ..VerificationInputs::default()
        },
        &policy,
        "2026-06-04T00:00:00Z",
    );

    assert_eq!(verification.result, "failed");
    let decision = committable_decision(&verification);
    assert_eq!(decision.outcome, CommittableOutcome::Failed);
    assert_eq!(
        decision.blocking_reasons,
        vec![
            CommittableReason::OpenRequiredFires,
            CommittableReason::StaleResolutions,
            CommittableReason::MissingEvidenceRefs,
            CommittableReason::DuplicateAtomIds,
            CommittableReason::FailedChecks,
        ]
    );
    assert_eq!(
        decision.warning_reasons,
        vec![CommittableReason::MissingRequiredLinks]
    );
    assert_eq!(
        decision.primary_reason,
        Some(CommittableReason::OpenRequiredFires)
    );
}

#[test]
fn committable_decision_table_covers_every_reason_and_priority() {
    let none = VerificationSignals {
        unresolved_merge_conflicts: 0,
        open_required_fires: 0,
        missing_required_links: 0,
        failed_checks: 0,
        stale_resolutions: 0,
        missing_evidence_refs: 0,
        duplicate_atom_ids: 0,
    };
    assert!(committable_decision_from_signals(true, none).is_committable());

    let cases = [
        (
            CommittableReason::UnresolvedMergeConflicts,
            VerificationSignals {
                unresolved_merge_conflicts: 1,
                ..none
            },
        ),
        (
            CommittableReason::OpenRequiredFires,
            VerificationSignals {
                open_required_fires: 1,
                ..none
            },
        ),
        (
            CommittableReason::MissingRequiredLinks,
            VerificationSignals {
                missing_required_links: 1,
                ..none
            },
        ),
        (
            CommittableReason::StaleResolutions,
            VerificationSignals {
                stale_resolutions: 1,
                ..none
            },
        ),
        (
            CommittableReason::MissingEvidenceRefs,
            VerificationSignals {
                missing_evidence_refs: 1,
                ..none
            },
        ),
        (
            CommittableReason::DuplicateAtomIds,
            VerificationSignals {
                duplicate_atom_ids: 1,
                ..none
            },
        ),
        (
            CommittableReason::FailedChecks,
            VerificationSignals {
                failed_checks: 1,
                ..none
            },
        ),
    ];
    for (reason, signals) in cases {
        let decision = committable_decision_from_signals(true, signals);
        assert_eq!(decision.primary_reason, Some(reason));
        assert_eq!(decision.blocking_reasons, vec![reason]);
        assert!(decision.warning_reasons.is_empty());
    }

    let trace_warning = committable_decision_from_signals(
        false,
        VerificationSignals {
            missing_required_links: 1,
            ..none
        },
    );
    assert!(trace_warning.is_committable());
    assert!(trace_warning.blocking_reasons.is_empty());
    assert_eq!(
        trace_warning.warning_reasons,
        vec![CommittableReason::MissingRequiredLinks]
    );

    let every_reason = committable_decision_from_signals(
        true,
        VerificationSignals {
            unresolved_merge_conflicts: 1,
            open_required_fires: 1,
            missing_required_links: 1,
            failed_checks: 1,
            stale_resolutions: 1,
            missing_evidence_refs: 1,
            duplicate_atom_ids: 1,
        },
    );
    assert_eq!(
        every_reason.blocking_reasons,
        vec![
            CommittableReason::UnresolvedMergeConflicts,
            CommittableReason::OpenRequiredFires,
            CommittableReason::MissingRequiredLinks,
            CommittableReason::StaleResolutions,
            CommittableReason::MissingEvidenceRefs,
            CommittableReason::DuplicateAtomIds,
            CommittableReason::FailedChecks,
        ]
    );
    assert_eq!(
        every_reason.primary_reason,
        Some(CommittableReason::UnresolvedMergeConflicts)
    );
}

#[test]
fn verification_merge_conflicts_are_typed_and_legacy_omission_is_empty() {
    let legacy: Verification = serde_json::from_value(serde_json::json!({
        "type": "verification",
        "version": 1,
        "result": "passed",
        "open_required_fires": 0,
        "failed_checks": [],
        "missing_required_links": [],
        "stale_resolutions": [],
        "missing_evidence_refs": [],
        "duplicate_atom_ids": [],
        "verified_at": "2026-07-12T00:00:00Z"
    }))
    .unwrap();
    assert!(legacy.unresolved_merge_conflicts.is_empty());
    assert!(committable_decision(&legacy).is_committable());

    let mut conflicted = legacy;
    conflicted.unresolved_merge_conflicts = vec![UnresolvedMergeConflict {
        kind: MergeConflictKind::Text,
        subject: "README.md".to_string(),
    }];
    let decision = committable_decision(&conflicted);
    assert_eq!(
        decision.primary_reason,
        Some(CommittableReason::UnresolvedMergeConflicts)
    );
    assert!(!decision.is_committable());
    assert_eq!(
        serde_json::to_value(&conflicted).unwrap()["unresolved_merge_conflicts"][0]["kind"],
        "text"
    );
}

#[test]
fn sealed_snapshot_semantics_rejects_open_fire_recorded_as_passed() {
    let (atom_index, trace_graph, mut fires, mut resolutions, _, mut policy) =
        valid_semantic_snapshot();
    policy.require_no_required_fires = false;
    fires[0].status = "open".to_string();
    fires[0].resolution_uid = None;
    resolutions.clear();
    let scan = ScanResult {
        type_tag: "scan".to_string(),
        version: VERSION,
        base_commit: "CF-COMMIT-base".to_string(),
        atom_index: atom_index.clone(),
        trace_graph: trace_graph.clone(),
        changed_atoms: Vec::new(),
        non_atom_changed_files: Vec::new(),
        change_coverage: None,
        open_fires: fires.clone(),
        tool_migration: None,
    };
    let mut verification = build_verification(
        &scan,
        VerificationInputs::default(),
        &policy,
        "2026-07-12T00:02:00Z",
    );
    assert_eq!(verification.open_required_fires, 1);
    assert_eq!(verification.result, "failed");
    verification.open_required_fires = 0;
    verification.result = "passed".to_string();

    let error = validate_sealed_snapshot_semantics(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
    )
    .unwrap_err();

    let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
        panic!("expected sealed snapshot semantic integrity error");
    };
    assert!(error.issues.iter().any(|issue| matches!(
        issue,
        SealedSnapshotSemanticIssue::OpenRequiredFireCountMismatch {
            computed: 1,
            recorded: 0
        }
    )));
    assert!(error.issues.iter().any(|issue| matches!(
        issue,
        SealedSnapshotSemanticIssue::VerificationResultMismatch { computed, recorded }
            if computed == "failed" && recorded == "passed"
    )));
}

#[test]
fn sealed_snapshot_semantics_reads_clean_historical_false_policy_without_relaxing_facts() {
    let atom_index = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: Vec::new(),
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: Vec::new(),
    };
    let scan = ScanResult {
        type_tag: "scan".to_string(),
        version: VERSION,
        base_commit: "CF-COMMIT-base".to_string(),
        atom_index: atom_index.clone(),
        trace_graph: trace_graph.clone(),
        changed_atoms: Vec::new(),
        non_atom_changed_files: Vec::new(),
        change_coverage: None,
        open_fires: Vec::new(),
        tool_migration: None,
    };
    let policy = VerificationPolicy {
        require_no_required_fires: false,
        require_no_stale_resolutions: false,
        require_verification_success: false,
        reject_duplicate_atom_ids: false,
        required_links: BTreeMap::new(),
        ..default_verification_policy()
    };
    let verification = build_verification(
        &scan,
        VerificationInputs::default(),
        &policy,
        "2026-07-12T00:00:00Z",
    );

    assert!(committable_decision(&verification).is_committable());
    validate_sealed_snapshot_semantics(&atom_index, &trace_graph, &[], &[], &verification, &policy)
        .unwrap();
}

#[test]
fn sealed_snapshot_semantics_recomputes_stale_missing_and_duplicate_facts() {
    let (mut atom_index, trace_graph, fires, resolutions, verification, mut policy) =
        valid_semantic_snapshot();
    atom_index
        .atoms
        .push(atom("REQ-AUTH-001", "requirement", "sha256:req"));
    policy.required_links = BTreeMap::from([(
        "requirement".to_string(),
        vec![RequiredLinkRule {
            link_type: "verified_by".to_string(),
            target_kind: "test".to_string(),
            min: 1,
        }],
    )]);

    let error = validate_sealed_snapshot_semantics(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
    )
    .unwrap_err();

    let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
        panic!("expected sealed snapshot semantic integrity error");
    };
    for expected_kind in [
        "duplicate_atom_ids_mismatch",
        "missing_required_links_mismatch",
        "stale_resolutions_mismatch",
        "verification_result_mismatch",
    ] {
        assert!(
            error
                .issues
                .iter()
                .any(|issue| issue.kind() == expected_kind),
            "missing semantic issue {expected_kind}"
        );
    }
}

#[test]
fn sealed_snapshot_semantics_rejects_ambiguous_active_resolution_relation() {
    let (atom_index, trace_graph, fires, mut resolutions, verification, policy) =
        valid_semantic_snapshot();
    let duplicate = resolutions[0].clone();
    resolutions.push(duplicate);

    let error = validate_sealed_snapshot_semantics(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
    )
    .unwrap_err();

    let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
        panic!("expected sealed snapshot semantic integrity error");
    };
    assert!(error.issues.iter().any(|issue| matches!(
        issue,
        SealedSnapshotSemanticIssue::DuplicateActiveResolutionUid {
            resolution_uid,
            occurrences: 2,
        } if resolution_uid == "res_001"
    )));
    assert!(error.issues.iter().any(|issue| matches!(
        issue,
        SealedSnapshotSemanticIssue::ActiveResolutionPerFireCardinality {
            fire_uid,
            resolution_uids,
        } if fire_uid == "fire_001" && resolution_uids == &["res_001", "res_001"]
    )));
    assert!(error.issues.iter().any(|issue| matches!(
        issue,
        SealedSnapshotSemanticIssue::FireResolutionIdentityMismatch {
            fire_uid,
            declared_resolution_uid: Some(declared),
            active_resolution_uids,
        } if fire_uid == "fire_001"
            && declared == "res_001"
            && active_resolution_uids == &["res_001", "res_001"]
    )));
}

#[test]
fn sealed_snapshot_semantics_accepts_a_consistent_snapshot() {
    let (atom_index, trace_graph, fires, resolutions, verification, policy) =
        valid_semantic_snapshot();

    validate_sealed_snapshot_semantics(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
    )
    .unwrap();
}

#[test]
fn sealed_snapshot_semantics_rejects_trace_requirement_metadata_drift() {
    let (atom_index, trace_graph, fires, resolutions, mut verification, policy) =
        valid_semantic_snapshot();
    verification.trace_completeness_required = false;

    let error = validate_sealed_snapshot_semantics(
        &atom_index,
        &trace_graph,
        &fires,
        &resolutions,
        &verification,
        &policy,
    )
    .unwrap_err();

    let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
        panic!("expected sealed snapshot semantic integrity error");
    };
    assert_eq!(
        error.issues,
        vec![
            SealedSnapshotSemanticIssue::TraceCompletenessRequiredMismatch {
                computed: true,
                recorded: false,
            }
        ]
    );
}

#[test]
fn stale_resolutions_detect_atom_policy_and_trace_changes() {
    let policy = default_verification_policy();
    let atom_index = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req1"),
            atom("DES-AUTH-001", "design", "sha256:des1"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };
    let fire = Fire {
        type_tag: "fire".to_string(),
        version: VERSION,
        fire_uid: "fire_001".to_string(),
        display_id: "FIRE-001".to_string(),
        status: "extinguished".to_string(),
        severity: "required".to_string(),
        source: FireAtomRef {
            atom_id: "REQ-AUTH-001".to_string(),
            content_hash_at_fire: Some("sha256:req1".to_string()),
        },
        target: FireAtomRef {
            atom_id: "DES-AUTH-001".to_string(),
            content_hash_at_fire: Some("sha256:des1".to_string()),
        },
        reason: "atom_changed".to_string(),
        trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
        created_by: "scan".to_string(),
        created_at: "2026-06-04T00:00:00Z".to_string(),
        key: "sha256:key".to_string(),
        obsolete_at: None,
        resolution_uid: Some("res_001".to_string()),
    };
    let resolution = build_resolution(
        &fire,
        &atom_index,
        &trace_graph,
        &policy,
        ResolutionRequest {
            resolution_uid: "res_001".to_string(),
            resolution_type: "addressed".to_string(),
            rationale: "checked".to_string(),
            evidence: String::new(),
            evidence_refs: Vec::new(),
            resolved_at: "2026-06-04T00:00:00Z".to_string(),
        },
    )
    .unwrap();
    assert!(stale_resolutions(
        &atom_index,
        &trace_graph,
        &policy,
        std::slice::from_ref(&resolution),
    )
    .unwrap()
    .is_empty());

    let changed_index = AtomIndex {
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req2"),
            atom("DES-AUTH-001", "design", "sha256:des1"),
        ],
        ..atom_index
    };
    let stale = stale_resolutions(
        &changed_index,
        &trace_graph,
        &policy,
        std::slice::from_ref(&resolution),
    )
    .unwrap();

    assert_eq!(
        stale,
        vec![StaleResolution {
            resolution_uid: "res_001".to_string(),
            reason: "source atom changed".to_string(),
            fire_uid: "fire_001".to_string(),
            fire_display_id: String::new(),
            source_atom_id: "REQ-AUTH-001".to_string(),
            target_atom_id: "DES-AUTH-001".to_string(),
        }]
    );

    assert_eq!(
        fire_display_id_from_uid("fire_sha256_571b5e2c0245bdd96db0c7407e4328bf").as_deref(),
        Some("FIRE-571B5E2C0245")
    );
    assert_eq!(fire_display_id_from_uid("fire_001"), None);
    let encoded = serde_json::to_value(StaleResolution {
        resolution_uid: "res_001".to_string(),
        reason: "policy changed".to_string(),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({"resolution_uid": "res_001", "reason": "policy changed"})
    );
}

#[test]
fn scan_result_detects_changed_atoms_and_creates_trace_fires() {
    let current = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("DES-AUTH-001", "design", "sha256:design2"),
            atom("REQ-AUTH-001", "requirement", "sha256:req2"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let base = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };

    let (scan, fires) = build_scan_result(
        current,
        base,
        trace_graph,
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        "atom_changed",
        "2026-06-04T00:00:00Z",
    )
    .unwrap();

    assert_eq!(scan.changed_atoms, vec!["DES-AUTH-001", "REQ-AUTH-001"]);
    assert_eq!(scan.open_fires.len(), 2);
    assert!(fires[0].fire_uid.starts_with("fire_sha256_"));
    assert_eq!(fires[0].fire_uid.len(), "fire_sha256_".len() + 32);
    assert!(fires[0].display_id.starts_with("FIRE-"));
    assert_eq!(fires[0].display_id.len(), "FIRE-".len() + 12);
    assert_eq!(fires[0].status, "open");
    assert_eq!(fires[0].severity, "required");
    assert_eq!(fires[0].created_by, "scan");
}

#[test]
fn indexed_scan_matches_reference_for_duplicate_self_cycle_and_diamond() {
    let current = atom_index(vec![
        atom("A", "requirement", "sha256:a2"),
        atom("B", "design", "sha256:b2"),
        atom("C", "test", "sha256:c2"),
        atom("D", "code", "sha256:d2"),
    ]);
    let base = atom_index(vec![
        atom("A", "requirement", "sha256:a1"),
        atom("B", "design", "sha256:b1"),
        atom("C", "test", "sha256:c1"),
        atom("D", "code", "sha256:d1"),
    ]);
    let make_link = |from: &str, to: &str| {
        TraceLinkInput {
            from: from.to_string(),
            to: Some(to.to_string()),
            link_type: Some("trace".to_string()),
        }
        .into_trace_link()
        .unwrap()
    };
    let graph = trace_graph(vec![
        make_link("A", "A"),
        make_link("A", "B"),
        make_link("A", "B"),
        make_link("B", "C"),
        make_link("C", "A"),
        make_link("A", "D"),
        make_link("B", "D"),
        make_link("C", "D"),
    ]);
    let options = ScanBuildOptions {
        reason: "atom_changed",
        now: "2026-07-19T00:00:00Z",
        source_unchanged: false,
    };
    let reference_graph = graph.clone();
    let reference = build_scan_result_against_base_with_adjacency(
        current.clone(),
        &base,
        graph.clone(),
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        options,
        |source| adjacent_atoms_reference(source, &reference_graph),
    )
    .unwrap();
    let compatible = build_scan_result_against_base(
        current.clone(),
        &base,
        graph.clone(),
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        options,
    )
    .unwrap();
    let (scan, fires, counters) = build_scan_result_against_base_with_work(
        current,
        &base,
        graph,
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        options,
    )
    .unwrap();

    assert_eq!((scan, fires), reference);
    assert_eq!(compatible, reference);
    assert_eq!(
        counters,
        TraceWorkCounters {
            atoms_indexed: 4,
            rules_evaluated: 0,
            links_indexed: 8,
            index_entries: 26,
            lookups: 12,
            adjacency_visits: 15,
        }
    );
}

#[test]
fn indexed_scan_keeps_direct_only_bidirectional_fire_semantics() {
    let current = atom_index(vec![
        atom("A", "requirement", "sha256:a2"),
        atom("B", "design", "sha256:b"),
        atom("C", "design", "sha256:c"),
        atom("D", "code", "sha256:d"),
    ]);
    let base = atom_index(vec![
        atom("A", "requirement", "sha256:a1"),
        atom("B", "design", "sha256:b"),
        atom("C", "design", "sha256:c"),
        atom("D", "code", "sha256:d"),
    ]);
    let make_link = |from: &str, to: &str| {
        TraceLinkInput {
            from: from.to_string(),
            to: Some(to.to_string()),
            link_type: Some("trace".to_string()),
        }
        .into_trace_link()
        .unwrap()
    };
    let graph = trace_graph(vec![
        make_link("A", "B"),
        make_link("A", "C"),
        make_link("B", "D"),
        make_link("C", "D"),
    ]);

    let (_, fires) = build_scan_result(
        current,
        base,
        graph,
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        "atom_changed",
        "2026-07-19T00:00:00Z",
    )
    .unwrap();
    let targets = fires
        .iter()
        .map(|fire| fire.target.atom_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(targets, vec!["B", "C"]);
}

#[test]
fn scan_result_uses_stable_sha256_fire_identity() {
    let current = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req2"),
            atom("DES-AUTH-001", "design", "sha256:design2"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let base = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };

    let first = build_scan_result(
        current.clone(),
        base.clone(),
        trace_graph.clone(),
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        "atom_changed",
        "2026-06-04T00:00:00Z",
    )
    .unwrap()
    .1;
    let with_obsolete = vec![Fire {
        status: "obsolete".to_string(),
        obsolete_at: Some("2026-06-05T00:00:00Z".to_string()),
        ..first[0].clone()
    }];
    let second = build_scan_result(
        current,
        base,
        trace_graph,
        with_obsolete,
        "CF-COMMIT-base".to_string(),
        "atom_changed",
        "2026-06-06T00:00:00Z",
    )
    .unwrap()
    .1;

    assert_eq!(first[0].fire_uid, second[1].fire_uid);
    assert_eq!(first[0].display_id, second[1].display_id);
    assert!(first[0].fire_uid.starts_with("fire_sha256_"));
}

#[test]
fn scan_result_classifies_source_unchanged_hash_schema_drift_as_tool_migration() {
    let current = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req2"),
            atom("DES-AUTH-001", "design", "sha256:design2"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let base = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: 1,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req1"),
            atom("DES-AUTH-001", "design", "sha256:design1"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };

    let (scan, fires) = build_scan_result_with_options(
        current.clone(),
        base.clone(),
        trace_graph.clone(),
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        ScanBuildOptions {
            reason: "atom_changed",
            now: "2026-06-04T00:00:00Z",
            source_unchanged: true,
        },
    )
    .unwrap();

    assert!(scan.changed_atoms.is_empty());
    assert!(scan.open_fires.is_empty());
    assert!(fires.is_empty());
    let migration = scan.tool_migration.unwrap();
    assert_eq!(migration.kind, "atom_hash_schema_changed");
    assert_eq!(migration.from_hash_schema_version, 1);
    assert_eq!(migration.to_hash_schema_version, ATOM_HASH_SCHEMA_VERSION);
    assert_eq!(
        migration.affected_atoms,
        vec!["DES-AUTH-001", "REQ-AUTH-001"]
    );

    let (scan, fires) = build_scan_result_with_options(
        current,
        base,
        trace_graph,
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        ScanBuildOptions {
            reason: "atom_changed",
            now: "2026-06-04T00:00:00Z",
            source_unchanged: false,
        },
    )
    .unwrap();
    assert_eq!(scan.changed_atoms, vec!["DES-AUTH-001", "REQ-AUTH-001"]);
    assert_eq!(scan.open_fires.len(), 2);
    assert_eq!(fires.len(), 2);
    assert!(scan.tool_migration.is_none());
}

#[test]
fn string_aware_python_symbols_use_schema_four_tool_migration_without_fires() {
    let temp = tempdir().unwrap();
    write_file(
        &temp.path().join("src/evoagent/models.py"),
        "class ExperimentPlan:\n    def uid(self):\n        return \"plan\"\n",
    );
    let current = build_atom_index(temp.path()).unwrap();
    let base = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: 2,
        atoms: Vec::new(),
        duplicate_atom_ids: Vec::new(),
    };

    let (scan, fires) = build_scan_result_with_options(
        current,
        base,
        TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: Vec::new(),
        },
        Vec::new(),
        "CF-COMMIT-base".to_string(),
        ScanBuildOptions {
            reason: "atom_changed",
            now: "2026-07-12T00:00:00Z",
            source_unchanged: true,
        },
    )
    .unwrap();

    assert_eq!(ATOM_HASH_SCHEMA_VERSION, 4);
    assert!(scan.changed_atoms.is_empty());
    assert!(scan.open_fires.is_empty());
    assert!(fires.is_empty());
    let migration = scan.tool_migration.unwrap();
    assert_eq!(migration.from_hash_schema_version, 2);
    assert_eq!(migration.to_hash_schema_version, 4);
    assert_eq!(
        migration.affected_atoms,
        vec![
            "CODE:src/evoagent/models.py::class:ExperimentPlan",
            "CODE:src/evoagent/models.py::method:ExperimentPlan.uid",
        ]
    );
}

#[test]
fn scan_result_obsoletes_scan_fires_when_source_is_no_longer_changed() {
    let current = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
        duplicate_atom_ids: Vec::new(),
    };
    let base = current.clone();
    let existing = Fire {
        type_tag: "fire".to_string(),
        version: VERSION,
        fire_uid: "fire_existing".to_string(),
        display_id: "FIRE-001".to_string(),
        status: "open".to_string(),
        severity: "required".to_string(),
        source: FireAtomRef {
            atom_id: "REQ-AUTH-001".to_string(),
            content_hash_at_fire: Some("sha256:req1".to_string()),
        },
        target: FireAtomRef {
            atom_id: "DES-AUTH-001".to_string(),
            content_hash_at_fire: None,
        },
        reason: "atom_changed".to_string(),
        trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
        created_by: "scan".to_string(),
        created_at: "2026-06-04T00:00:00Z".to_string(),
        key: "sha256:key".to_string(),
        obsolete_at: None,
        resolution_uid: None,
    };

    let (scan, fires) = build_scan_result(
        current,
        base,
        TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: Vec::new(),
        },
        vec![existing],
        "CF-COMMIT-base".to_string(),
        "atom_changed",
        "2026-06-04T00:01:00Z",
    )
    .unwrap();

    assert!(scan.changed_atoms.is_empty());
    assert!(scan.open_fires.is_empty());
    assert_eq!(fires[0].status, "obsolete");
    assert_eq!(
        fires[0].obsolete_at.as_deref(),
        Some("2026-06-04T00:01:00Z")
    );
}

fn valid_semantic_snapshot() -> (
    AtomIndex,
    TraceGraph,
    Vec<Fire>,
    Vec<Resolution>,
    Verification,
    VerificationPolicy,
) {
    let policy = VerificationPolicy {
        required_links: BTreeMap::new(),
        ..default_verification_policy()
    };
    let atom_index = AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms: vec![
            atom("REQ-AUTH-001", "requirement", "sha256:req"),
            atom("DES-AUTH-001", "design", "sha256:des"),
        ],
        duplicate_atom_ids: Vec::new(),
    };
    let trace_graph = TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links: vec![TraceLinkInput {
            from: "REQ-AUTH-001".to_string(),
            to: Some("DES-AUTH-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap()],
    };
    let fire = Fire {
        type_tag: "fire".to_string(),
        version: VERSION,
        fire_uid: "fire_001".to_string(),
        display_id: "FIRE-001".to_string(),
        status: "extinguished".to_string(),
        severity: "required".to_string(),
        source: FireAtomRef {
            atom_id: "REQ-AUTH-001".to_string(),
            content_hash_at_fire: Some("sha256:req".to_string()),
        },
        target: FireAtomRef {
            atom_id: "DES-AUTH-001".to_string(),
            content_hash_at_fire: Some("sha256:des".to_string()),
        },
        reason: "atom_changed".to_string(),
        trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
        created_by: "scan".to_string(),
        created_at: "2026-07-12T00:00:00Z".to_string(),
        key: "sha256:key".to_string(),
        obsolete_at: None,
        resolution_uid: Some("res_001".to_string()),
    };
    let resolution = build_resolution(
        &fire,
        &atom_index,
        &trace_graph,
        &policy,
        ResolutionRequest {
            resolution_uid: "res_001".to_string(),
            resolution_type: "addressed".to_string(),
            rationale: "reviewed".to_string(),
            evidence: "test evidence".to_string(),
            evidence_refs: Vec::new(),
            resolved_at: "2026-07-12T00:01:00Z".to_string(),
        },
    )
    .unwrap();
    let scan = ScanResult {
        type_tag: "scan".to_string(),
        version: VERSION,
        base_commit: "CF-COMMIT-base".to_string(),
        atom_index: atom_index.clone(),
        trace_graph: trace_graph.clone(),
        changed_atoms: Vec::new(),
        non_atom_changed_files: Vec::new(),
        change_coverage: None,
        open_fires: Vec::new(),
        tool_migration: None,
    };
    let verification = build_verification(
        &scan,
        VerificationInputs::default(),
        &policy,
        "2026-07-12T00:02:00Z",
    );
    (
        atom_index,
        trace_graph,
        vec![fire],
        vec![resolution],
        verification,
        policy,
    )
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

fn atom(atom_id: &str, kind: &str, content_hash: &str) -> Atom {
    Atom {
        atom_id: atom_id.to_string(),
        kind: kind.to_string(),
        artifact_path: "artifact".to_string(),
        selector: Selector {
            selector_type: "test".to_string(),
            value: atom_id.to_string(),
        },
        content_hash: content_hash.to_string(),
    }
}

fn atom_index(atoms: Vec<Atom>) -> AtomIndex {
    AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms,
        duplicate_atom_ids: Vec::new(),
    }
}

fn trace_graph(links: Vec<TraceLink>) -> TraceGraph {
    TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links,
    }
}
