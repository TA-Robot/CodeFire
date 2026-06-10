use super::{DoctorIssue, DoctorReport, DoctorSeverity};
use crate::automation::{cli_command, next_action as automation_next_action};
use serde_json::{json, Value};

pub(crate) fn doctor_report_data_json(report: &DoctorReport) -> Value {
    json!({
        "type": "codefire_doctor_report",
        "version": 1,
        "ok": report.ok,
        "mode": if report.quick { "quick" } else { "full" },
        "skipped_checks": &report.skipped_checks,
        "checked": {
            "objects": report.checked_objects,
            "branches": report.checked_branches,
            "opened": report.checked_opened,
            "active_files": report.checked_active_files,
        },
        "issue_counts": {
            "total": report.issues.len(),
            "blocking": report.issues.iter().filter(|issue| issue_blocking(issue)).count(),
        },
        "issues": report.issues.iter().map(issue_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn doctor_report_diagnostics_json(report: &DoctorReport) -> Vec<Value> {
    report.issues.iter().map(issue_json).collect()
}

pub(crate) fn doctor_report_next_actions(report: &DoctorReport) -> Vec<Value> {
    if report.ok {
        return Vec::new();
    }
    let mut actions = Vec::new();
    if report.issues.iter().any(|issue| {
        matches!(
            issue.kind.as_str(),
            "invalid_object_record"
                | "object_integrity_error"
                | "object_type_directory_mismatch"
                | "unknown_object_type"
        )
    }) {
        actions.push(automation_next_action(
            "inspect_object_store_corruption",
            cli_command("doctor --json"),
            "inspect object record integrity errors before opening or uploading branches",
            json!({"affected": count_kinds(report, &["invalid_object_record", "object_integrity_error", "object_type_directory_mismatch", "unknown_object_type"])}),
        ));
    }
    if report.issues.iter().any(|issue| {
        matches!(
            issue.kind.as_str(),
            "invalid_branch_record" | "missing_branch_head" | "invalid_branch_head"
        )
    }) {
        actions.push(automation_next_action(
            "repair_branch_head",
            cli_command("branch list"),
            "identify branch heads that no longer point to valid sealed commits",
            json!({"affected": count_kinds(report, &["invalid_branch_record", "missing_branch_head", "invalid_branch_head"])}),
        ));
    }
    if report.issues.iter().any(|issue| {
        matches!(
            issue.kind.as_str(),
            "invalid_open_registry"
                | "invalid_open_base_commit"
                | "missing_open_base_commit"
                | "missing_active_state_path"
                | "missing_active_state_dir"
                | "invalid_active_state_json"
                | "invalid_active_state_shape"
                | "missing_open_path"
                | "missing_open_path_dir"
                | "missing_open_marker"
                | "invalid_open_marker"
                | "open_marker_repository_mismatch"
                | "open_marker_branch_mismatch"
                | "open_marker_instance_mismatch"
                | "open_marker_path_mismatch"
                | "missing_open_registry_state"
        )
    }) {
        actions.push(automation_next_action(
            "reopen_branch_workspace",
            cli_command("open <branch> <path>"),
            "reopen affected workspaces from a valid sealed commit after preserving local edits",
            json!({"affected": count_kinds(report, &["invalid_open_registry", "invalid_open_base_commit", "missing_open_base_commit", "missing_active_state_path", "missing_active_state_dir", "invalid_active_state_json", "invalid_active_state_shape", "missing_open_path", "missing_open_path_dir", "missing_open_marker", "invalid_open_marker", "open_marker_repository_mismatch", "open_marker_branch_mismatch", "open_marker_instance_mismatch", "open_marker_path_mismatch", "missing_open_registry_state"])}),
        ));
    }
    if actions.is_empty() {
        actions.push(automation_next_action(
            "inspect_doctor_report",
            cli_command("doctor --json"),
            "inspect repository diagnostics",
            json!({"issues": report.issues.len()}),
        ));
    }
    actions
}

pub(crate) fn print_doctor_report(report: &DoctorReport) {
    println!("CodeFire doctor");
    println!("repo: {}", report.repo_root.display());
    println!("ok: {}", report.ok);
    println!("checked objects: {}", report.checked_objects);
    println!("checked branches: {}", report.checked_branches);
    println!("checked opened registries: {}", report.checked_opened);
    println!("checked active files: {}", report.checked_active_files);
    println!("mode: {}", if report.quick { "quick" } else { "full" });
    if !report.skipped_checks.is_empty() {
        println!("skipped checks: {}", report.skipped_checks.join(", "));
    }
    println!("issues: {}", report.issues.len());
    println!(
        "blocking issues: {}",
        report
            .issues
            .iter()
            .filter(|issue| issue_blocking(issue))
            .count()
    );
    if report.issues.is_empty() {
        println!("  none");
    } else {
        for issue in &report.issues {
            match &issue.path {
                Some(path) => println!(
                    "  {} {} {} repairable={} blocking={}: {} ({})",
                    severity_name(issue.severity),
                    issue_category(issue),
                    issue.kind,
                    issue_repairable(issue),
                    issue_blocking(issue),
                    issue.message,
                    path.display()
                ),
                None => println!(
                    "  {} {} {} repairable={} blocking={}: {}",
                    severity_name(issue.severity),
                    issue_category(issue),
                    issue.kind,
                    issue_repairable(issue),
                    issue_blocking(issue),
                    issue.message
                ),
            }
        }
    }
}

fn count_kinds(report: &DoctorReport, kinds: &[&str]) -> usize {
    report
        .issues
        .iter()
        .filter(|issue| kinds.contains(&issue.kind.as_str()))
        .count()
}

fn issue_json(issue: &DoctorIssue) -> Value {
    json!({
        "severity": severity_name(issue.severity),
        "category": issue_category(issue),
        "repairable": issue_repairable(issue),
        "blocking": issue_blocking(issue),
        "kind": &issue.kind,
        "message": &issue.message,
        "path": &issue.path,
    })
}

fn severity_name(severity: DoctorSeverity) -> &'static str {
    match severity {
        DoctorSeverity::Error => "error",
        DoctorSeverity::Warning => "warning",
    }
}

fn issue_category(issue: &DoctorIssue) -> &'static str {
    match issue.kind.as_str() {
        "missing_codefire_dir"
        | "missing_repo_directory"
        | "missing_object_subdirectory"
        | "empty_object_store" => "layout",
        "invalid_object_record"
        | "object_integrity_error"
        | "object_type_directory_mismatch"
        | "unknown_object_type" => "object_store",
        "invalid_branch_record" | "missing_branch_head" | "invalid_branch_head" => "branch",
        "invalid_open_registry"
        | "invalid_open_base_commit"
        | "missing_open_base_commit"
        | "missing_open_branch_name"
        | "missing_open_instance_id"
        | "missing_open_path"
        | "missing_open_path_dir"
        | "missing_active_state_path"
        | "missing_active_state_dir"
        | "missing_open_marker"
        | "invalid_open_marker"
        | "open_marker_repository_mismatch"
        | "open_marker_branch_mismatch"
        | "open_marker_instance_mismatch"
        | "open_marker_path_mismatch"
        | "missing_open_registry_state" => "opened_registry",
        "invalid_active_state_json" | "invalid_active_state_shape" => "active_state",
        _ => "repository",
    }
}

fn issue_repairable(issue: &DoctorIssue) -> bool {
    matches!(
        issue.kind.as_str(),
        "missing_repo_directory"
            | "missing_object_subdirectory"
            | "empty_object_store"
            | "missing_open_path_dir"
            | "missing_active_state_dir"
            | "invalid_active_state_json"
            | "invalid_active_state_shape"
            | "missing_open_marker"
            | "invalid_open_marker"
            | "open_marker_repository_mismatch"
            | "open_marker_branch_mismatch"
            | "open_marker_instance_mismatch"
            | "open_marker_path_mismatch"
            | "missing_open_registry_state"
    )
}

pub(super) fn issue_blocking(issue: &DoctorIssue) -> bool {
    issue.severity == DoctorSeverity::Error
}
