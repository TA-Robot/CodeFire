use super::*;
use crate::remote::{ensure_remote_layout, run_remote_doctor};

fn project_url(server: &Path) -> String {
    format!("cf://{}/org/app", server.display())
}

#[test]
fn file_remote_doctor_routes_urls_and_reports_missing_without_local_discovery() {
    let temp = tempdir().unwrap();
    let url = project_url(temp.path());

    let report = run_remote_doctor(&url, false).unwrap().unwrap();

    assert!(!report.ok);
    assert!(!report.project_exists);
    assert_eq!(report.transport, "file");
    assert_eq!(report.issues[0].kind, "remote_project_missing");
}

#[test]
fn file_remote_doctor_accepts_empty_healthy_current_layout() {
    let temp = tempdir().unwrap();
    let project = temp.path().join(".codefire-server/projects/org/app");
    ensure_remote_layout(&project).unwrap();

    let report = run_remote_doctor(&project_url(temp.path()), false)
        .unwrap()
        .unwrap();

    assert!(report.ok, "{:#?}", report.issues);
    assert!(report.project_exists);
    assert_eq!(report.checked_objects, 0);
    assert_eq!(report.branch_count, 0);
}

#[test]
fn file_remote_doctor_rejects_future_branch_record_before_semantic_use() {
    let temp = tempdir().unwrap();
    let project = temp.path().join(".codefire-server/projects/org/app");
    ensure_remote_layout(&project).unwrap();
    fs::write(
        project.join("branches/main.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 2,
            "name": "main",
            "head": "CF-COMMIT-future"
        }))
        .unwrap(),
    )
    .unwrap();

    let report = run_remote_doctor(&project_url(temp.path()), false)
        .unwrap()
        .unwrap();

    assert!(!report.ok);
    assert_eq!(report.invalid_record_versions, 1);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.kind == "remote_record_version_unsupported"));
}

#[test]
fn file_remote_doctor_reports_corrupt_object_as_bounded_typed_issue() {
    let temp = tempdir().unwrap();
    let project = temp.path().join(".codefire-server/projects/org/app");
    ensure_remote_layout(&project).unwrap();
    fs::write(
        project.join("objects/policies/CF-POLICY-corrupt.json"),
        b"{\"object_id\":",
    )
    .unwrap();

    let report = run_remote_doctor(&project_url(temp.path()), false)
        .unwrap()
        .unwrap();

    assert!(!report.ok);
    assert_eq!(report.object_hash_errors, 1);
    assert!(report.issues.len() <= 128);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.kind == "remote_object_record_invalid"));
}

#[test]
fn local_doctor_target_is_not_claimed_by_remote_router() {
    assert!(run_remote_doctor("/tmp/local-repository", false)
        .unwrap()
        .is_none());
}
