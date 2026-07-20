use super::*;

fn open_empty_repository() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let open_dir = temp.path().join("main-open");
    init_repo(&repo_root, false).unwrap();
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
    (temp, repo_root, open_dir)
}

fn commit_options(path: &Path, message: &str, dry_run: bool) -> CommitOptions {
    CommitOptions {
        path: path.to_path_buf(),
        message: message.to_string(),
        dry_run,
        full_output: true,
        json_output: true,
        lock: LockOptions::default(),
        idempotency_key: None,
        signer: None,
        key_id: None,
    }
}

fn only_relation(scan: &codefire_core::ScanResult) -> codefire_core::ChangeCoverageRelation {
    let coverage = scan.change_coverage.as_ref().expect("coverage contract");
    assert_eq!(coverage.entries.len(), 1);
    coverage.entries[0].relation
}

#[test]
fn effective_policy_provenance_projects_to_verification_json_and_text() {
    let (_temp, _repo_root, open_dir) = open_empty_repository();
    let default = run_verify(&open_dir).unwrap();
    let default_policy = default.effective_policy.as_ref().unwrap();
    let default_rule = default_policy
        .rules
        .iter()
        .find(|rule| rule.id == "commit.require_trace_completeness")
        .unwrap();
    assert_eq!(
        default_rule.source,
        codefire_core::PolicyRuleSource::BuiltInDefault
    );

    fs::write(
        open_dir.join("codefire.policy.yaml"),
        "version: 1\ncommit_policy:\n  require_trace_completeness: false\n",
    )
    .unwrap();
    let verification = run_verify(&open_dir).unwrap();
    let policy = verification.effective_policy.as_ref().unwrap();
    let rule = policy
        .rules
        .iter()
        .find(|rule| rule.id == "commit.require_trace_completeness")
        .unwrap();
    assert_eq!(
        rule.source,
        codefire_core::PolicyRuleSource::RepositoryConfig
    );
    assert_eq!(rule.effective_value, Value::Bool(false));
    assert_ne!(default_policy.digest, policy.digest);

    let json = automation::verification_data_json(&verification);
    assert_eq!(
        json["effective_policy"]["digest"],
        Value::String(policy.digest.clone())
    );
    assert!(json["effective_policy"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["id"] == "commit.require_trace_completeness"
                && item["source"] == "repository_config"
                && item["effective_value"] == false
        }));
    assert_eq!(
        json["decision"]["effective_policy_digest"],
        Value::String(policy.digest.clone())
    );
    let text = verification::render_verification(&verification, false, false);
    assert!(text.contains(&format!("Effective policy: {}", policy.digest)));
    let explanation = explain::run_explain(&explain::ExplainOptions {
        path: open_dir,
        target: explain::ExplainTarget::VerifyFailure,
        json_output: true,
        depth: 1,
        large_threshold_bytes: 1_048_576,
    })
    .unwrap();
    assert_eq!(
        explanation.data["verification"]["effective_policy"]["digest"],
        Value::String(policy.digest.clone())
    );
    assert!(explanation.diagnostics.is_empty());
    let mut blocked = verification.clone();
    blocked.open_required_fires = 1;
    let diagnostics = automation::verification_diagnostics_json(&blocked);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0]["decision_contract"]["effective_policy_digest"],
        policy.digest
    );
}

#[test]
fn file_relation_decision_table_projects_through_scan_verify_and_commit_plan() {
    let (_temp, _repo_root, open_dir) = open_empty_repository();
    fs::create_dir_all(open_dir.join("src")).unwrap();
    let source = open_dir.join("src/model.py");
    fs::write(&source, "def answer():\n    return 1\n").unwrap();
    let committed = run_commit(&commit_options(&open_dir, "base atom", false)).unwrap();
    assert!(!committed.blocked);

    fs::write(
        &source,
        "module_note = 'outside'\ndef answer():\n    return 1\n",
    )
    .unwrap();
    let outside = run_scan(&open_dir).unwrap();
    assert_eq!(
        only_relation(&outside),
        codefire_core::ChangeCoverageRelation::OutOfAtomChange
    );
    assert!(outside.changed_atoms.is_empty());
    assert_eq!(
        scan_data_json_with_full(&outside, true)["change_coverage"]["summary"]["out_of_atom_files"],
        1
    );

    fs::write(
        &source,
        "module_note = 'mixed'\ndef answer():\n    return 2\n",
    )
    .unwrap();
    let mixed = run_scan(&open_dir).unwrap();
    assert_eq!(
        only_relation(&mixed),
        codefire_core::ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans
    );
    assert_eq!(mixed.changed_atoms.len(), 1);
    assert!(render_scan(&mixed).contains("Atom file(s) with unclassified spans"));
    let verification = run_verify(&open_dir).unwrap();
    assert_eq!(
        verification
            .change_coverage
            .as_ref()
            .unwrap()
            .summary
            .atom_files_with_unclassified_spans,
        1
    );
    let verification_json = automation::verification_data_json(&verification);
    assert_eq!(
        verification_json["change_coverage"]["summary"]["atom_files_with_unclassified_spans"],
        1
    );
    let preview = run_commit(&commit_options(&open_dir, "mixed preview", true)).unwrap();
    assert_eq!(
        preview.plan["change_coverage"]["summary"]["atom_files_with_unclassified_spans"],
        1
    );

    fs::remove_file(&source).unwrap();
    let deleted = run_scan(&open_dir).unwrap();
    assert_eq!(
        only_relation(&deleted),
        codefire_core::ChangeCoverageRelation::DeletedAtom
    );
    assert_eq!(
        deleted
            .change_coverage
            .as_ref()
            .unwrap()
            .summary
            .deleted_atom_files,
        1
    );
}

#[test]
fn change_coverage_json_is_bounded_without_losing_full_summary_or_digest() {
    let (_temp, _repo_root, open_dir) = open_empty_repository();
    for index in 0..60 {
        fs::write(
            open_dir.join(format!("plain-{index:02}.txt")),
            format!("value {index}\n"),
        )
        .unwrap();
    }
    let scan = run_scan(&open_dir).unwrap();
    let bounded = scan_data_json_with_full(&scan, false);
    assert_eq!(bounded["change_coverage"]["summary"]["changed_files"], 60);
    assert_eq!(
        bounded["change_coverage"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        50
    );
    assert_eq!(bounded["change_coverage"]["entries_omitted"], 10);
    assert_eq!(bounded["change_coverage"]["entries_truncated"], true);
    assert_eq!(
        bounded["change_coverage"]["digest"],
        scan.change_coverage.as_ref().unwrap().digest
    );

    let full = scan_data_json_with_full(&scan, true);
    assert_eq!(
        full["change_coverage"]["entries"].as_array().unwrap().len(),
        60
    );
    assert_eq!(full["change_coverage"]["entries_omitted"], 0);
}
