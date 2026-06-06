use super::{DoctorIssue, DoctorReport, DoctorSeverity};
use crate::automation::cli_command;
use serde_json::{json, Value};

pub(crate) fn doctor_report_data_json(report: &DoctorReport) -> Value {
    json!({
        "type": "codefire_doctor_report",
        "version": 1,
        "ok": report.ok,
        "checked": {
            "objects": report.checked_objects,
            "branches": report.checked_branches,
            "opened": report.checked_opened,
            "active_files": report.checked_active_files,
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
        actions.push(next_action(
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
        actions.push(next_action(
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
        )
    }) {
        actions.push(next_action(
            "reopen_branch_workspace",
            cli_command("open <branch> <path>"),
            "reopen affected workspaces from a valid sealed commit after preserving local edits",
            json!({"affected": count_kinds(report, &["invalid_open_registry", "invalid_open_base_commit", "missing_open_base_commit", "missing_active_state_path", "missing_active_state_dir", "invalid_active_state_json"])}),
        ));
    }
    if actions.is_empty() {
        actions.push(next_action(
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
    println!("issues: {}", report.issues.len());
    if report.issues.is_empty() {
        println!("  none");
    } else {
        for issue in &report.issues {
            match &issue.path {
                Some(path) => println!(
                    "  {} {}: {} ({})",
                    severity_name(issue.severity),
                    issue.kind,
                    issue.message,
                    path.display()
                ),
                None => println!(
                    "  {} {}: {}",
                    severity_name(issue.severity),
                    issue.kind,
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
        "kind": &issue.kind,
        "message": &issue.message,
        "path": &issue.path,
    })
}

fn next_action(id: &str, command: String, description: &str, context: Value) -> Value {
    json!({
        "id": id,
        "command": command,
        "description": description,
        "context": context,
    })
}

fn severity_name(severity: DoctorSeverity) -> &'static str {
    match severity {
        DoctorSeverity::Error => "error",
        DoctorSeverity::Warning => "warning",
    }
}
