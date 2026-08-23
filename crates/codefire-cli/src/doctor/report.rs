mod accounting;

use super::{
    DoctorIssue, DoctorPendingAuthority, DoctorPendingAuthorityKind, DoctorPendingRequestIdentity,
    DoctorReport, DoctorReportAccountingTransfer, DoctorSeverity,
};
use crate::automation::{cli_command, next_action as automation_next_action, path_command};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fmt;

pub(crate) use accounting::AccountedDoctorReport;

pub(super) fn account_doctor_report(
    report: DoctorReport,
    ledger: codefire_core::DecisionWorkLedger,
    report_accounting: DoctorReportAccountingTransfer,
) -> Result<AccountedDoctorReport, crate::CliError> {
    AccountedDoctorReport::new_with_preaccounted(report, ledger, report_accounting)
}

pub(super) fn doctor_report_data_json(report: &DoctorReport) -> Value {
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

pub(super) fn doctor_report_diagnostics_json(report: &DoctorReport) -> Vec<Value> {
    report.issues.iter().map(issue_json).collect()
}

pub(super) fn doctor_report_next_actions(report: &DoctorReport) -> Vec<Value> {
    let has_legacy_semantic_profile = report.issues.iter().any(|issue| {
        matches!(
            issue.kind.as_str(),
            "legacy_branch_head_semantic_profile" | "legacy_open_base_semantic_profile"
        )
    });
    if report.ok && !has_legacy_semantic_profile {
        return Vec::new();
    }
    let mut actions = Vec::new();
    let legacy_open_paths = report
        .issues
        .iter()
        .filter(|issue| issue.kind == "legacy_open_base_semantic_profile")
        .filter_map(|issue| issue.path.clone())
        .collect::<BTreeSet<_>>();
    for path in legacy_open_paths {
        actions.push(automation_next_action(
            "scan_legacy_open_base",
            cli_command(format!(
                "{} --json",
                path_command("scan", Some(path.as_path()))
            )),
            "scan the open workspace to build a current semantic-profile snapshot before sealing a bridge commit",
            json!({
                "path": path,
                "semantic_validation_profile": "legacy-unversioned",
            }),
        ));
    }
    if report
        .issues
        .iter()
        .any(|issue| issue.kind == "legacy_branch_head_semantic_profile")
    {
        actions.push(automation_next_action(
            "inspect_legacy_branch_profile",
            cli_command(format!(
                "{} --json",
                path_command("migrate check", Some(report.repo_root.as_path()))
            )),
            "inspect legacy branch heads and their migration status without rewriting authenticated history",
            json!({
                "affected": count_kinds(report, &["legacy_branch_head_semantic_profile"]),
                "semantic_validation_profile": "legacy-unversioned",
            }),
        ));
    }
    if report
        .issues
        .iter()
        .any(|issue| matches!(issue.kind.as_str(), "pending_open_transaction"))
    {
        actions.push(automation_next_action(
            "resume_pending_open",
            cli_command("open <branch> <path> --json"),
            "rerun the exact open request to reconcile its durable transaction",
            json!({"affected": count_kinds(report, &["pending_open_transaction"])}),
        ));
    }
    if report
        .issues
        .iter()
        .any(|issue| matches!(issue.kind.as_str(), "conflicting_repository_transactions"))
    {
        actions.push(automation_next_action(
            "inspect_conflicting_transactions",
            cli_command("doctor --json"),
            "inspect simultaneous open/close transaction authorities before retrying either mutation",
            json!({"affected": count_kinds(report, &["conflicting_repository_transactions"])}),
        ));
    }
    if report
        .issues
        .iter()
        .any(|issue| matches!(issue.kind.as_str(), "pending_close_transaction"))
    {
        actions.push(automation_next_action(
            "resume_pending_close",
            cli_command("close <branch> --json"),
            "rerun the exact close/discard request to reconcile its durable transaction",
            json!({"affected": count_kinds(report, &["pending_close_transaction"])}),
        ));
    }
    for issue in report
        .issues
        .iter()
        .filter(|issue| issue.pending_authority.is_some())
    {
        let pending = issue
            .pending_authority
            .as_ref()
            .expect("pending authority was filtered above");
        let (kind, reason) = match pending.authority {
            DoctorPendingAuthorityKind::Commit => (
                "inspect_pending_commit_transaction",
                "inspect the pending commit authority read-only and rerun only its exact owning commit request",
            ),
            DoctorPendingAuthorityKind::State => (
                "inspect_pending_state_transaction",
                "inspect the pending state authority read-only and rerun only its exact owning operation",
            ),
            DoctorPendingAuthorityKind::ActiveLedger => (
                "inspect_pending_active_ledger_transaction",
                "inspect the pending active-ledger authority read-only and rerun only its exact owning operation",
            ),
        };
        actions.push(automation_next_action(
            kind,
            cli_command(format!(
                "{} --json",
                path_command("doctor", Some(report.repo_root.as_path()))
            )),
            reason,
            pending_authority_target(report, issue, pending),
        ));
    }
    for divergence in report
        .issues
        .iter()
        .filter_map(|issue| issue.open_snapshot.as_ref())
    {
        actions.push(automation_next_action(
            "inspect_open_snapshot_divergence",
            cli_command(format!(
                "{} --json",
                path_command("doctor", Some(report.repo_root.as_path()))
            )),
            "inspect the divergent open authorities without rewriting branch, registry, marker, or active state",
            json!({
                "repo": &divergence.repo_root,
                "branch": &divergence.branch,
                "phase": &divergence.phase,
                "authority": divergence.authority,
                "expected": &divergence.expected,
                "actual": &divergence.actual,
                "expected_record": &divergence.expected_record,
                "actual_record": &divergence.actual_record,
            }),
        ));
    }
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
                | "invalid_active_state_dir"
                | "missing_active_state_file"
                | "invalid_active_state_json"
                | "invalid_active_state_shape"
                | "active_state_invariant_violation"
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
            json!({"affected": count_kinds(report, &["invalid_open_registry", "invalid_open_base_commit", "missing_open_base_commit", "missing_active_state_path", "missing_active_state_dir", "invalid_active_state_dir", "missing_active_state_file", "invalid_active_state_json", "invalid_active_state_shape", "active_state_invariant_violation", "missing_open_path", "missing_open_path_dir", "missing_open_marker", "invalid_open_marker", "open_marker_repository_mismatch", "open_marker_branch_mismatch", "open_marker_instance_mismatch", "open_marker_path_mismatch", "missing_open_registry_state"])}),
        ));
    }
    if report.issues.iter().any(|issue| {
        matches!(
            issue.kind.as_str(),
            "missing_repairable_directory" | "missing_repairable_object_subdirectory"
        )
    }) {
        actions.push(automation_next_action(
            "review_migration_plan",
            cli_command("migrate dry-run --json"),
            "review the directory creation plan for repairable layout findings",
            json!({"affected": count_kinds(report, &["missing_repairable_directory", "missing_repairable_object_subdirectory"])}),
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

fn write_doctor_text<W: fmt::Write>(report: &DoctorReport, output: &mut W) -> fmt::Result {
    writeln!(output, "CodeFire doctor")?;
    writeln!(output, "repo: {}", report.repo_root.display())?;
    writeln!(output, "ok: {}", report.ok)?;
    writeln!(output, "checked objects: {}", report.checked_objects)?;
    writeln!(output, "checked branches: {}", report.checked_branches)?;
    writeln!(
        output,
        "checked opened registries: {}",
        report.checked_opened
    )?;
    writeln!(
        output,
        "checked active files: {}",
        report.checked_active_files
    )?;
    writeln!(
        output,
        "mode: {}",
        if report.quick { "quick" } else { "full" }
    )?;
    if !report.skipped_checks.is_empty() {
        write!(output, "skipped checks: ")?;
        for (index, skipped) in report.skipped_checks.iter().enumerate() {
            if index != 0 {
                write!(output, ", ")?;
            }
            write!(output, "{skipped}")?;
        }
        writeln!(output)?;
    }
    writeln!(output, "issues: {}", report.issues.len())?;
    writeln!(
        output,
        "blocking issues: {}",
        report
            .issues
            .iter()
            .filter(|issue| issue_blocking(issue))
            .count()
    )?;
    if report.issues.is_empty() {
        writeln!(output, "  none")?;
    } else {
        for issue in &report.issues {
            match &issue.path {
                Some(path) => writeln!(
                    output,
                    "  {} {} {} repairable={} blocking={}: {} ({})",
                    severity_name(issue.severity),
                    issue_category(issue),
                    issue.kind,
                    issue_repairable(issue),
                    issue_blocking(issue),
                    issue.message,
                    path.display()
                )?,
                None => writeln!(
                    output,
                    "  {} {} {} repairable={} blocking={}: {}",
                    severity_name(issue.severity),
                    issue_category(issue),
                    issue.kind,
                    issue_repairable(issue),
                    issue_blocking(issue),
                    issue.message
                )?,
            }
        }
    }
    Ok(())
}

fn count_kinds(report: &DoctorReport, kinds: &[&str]) -> usize {
    report
        .issues
        .iter()
        .filter(|issue| kinds.contains(&issue.kind.as_str()))
        .count()
}

fn pending_authority_target(
    report: &DoctorReport,
    issue: &DoctorIssue,
    pending: &DoctorPendingAuthority,
) -> Value {
    let mut target = json!({
        "type": "codefire_pending_authority_target",
        "version": 1,
        "repo": &report.repo_root,
        "path": &issue.path,
        "authority": pending.authority.as_str(),
        "operation": pending.operation.as_str(),
        "phase": &pending.phase,
        "request_identity": pending_request_identity_json(&pending.request_identity),
        "owner": {
            "command": pending.operation.owner_command(),
            "mode": pending.operation.owner_mode(),
            "retry_contract": "exact_request_only",
        },
        "inspection": {
            "command": "doctor",
            "mode": "read_only",
        },
        "automatic_repair": false,
    });
    if let Some(intended_state) = pending.intended_state.as_ref() {
        target["intended_state"] = Value::String(intended_state.clone());
    }
    target
}

fn pending_request_identity_json(identity: &DoctorPendingRequestIdentity) -> Value {
    match identity {
        DoctorPendingRequestIdentity::TransactionId { transaction_id } => json!({
            "kind": "transaction_id",
            "transaction_id": transaction_id,
        }),
        DoctorPendingRequestIdentity::RequestHash { request_hash } => json!({
            "kind": "request_hash",
            "request_hash": request_hash,
        }),
        DoctorPendingRequestIdentity::StateSubject {
            branch,
            open_instance_id,
            current_base_commit,
        } => json!({
            "kind": "state_subject",
            "branch": branch,
            "open_instance_id": open_instance_id,
            "current_base_commit": current_base_commit,
        }),
        DoctorPendingRequestIdentity::LegacyUnbound => json!({
            "kind": "legacy_unbound",
        }),
        DoctorPendingRequestIdentity::Withheld { reason } => json!({
            "kind": "withheld",
            "reason": reason,
        }),
    }
}

fn issue_json(issue: &DoctorIssue) -> Value {
    let mut value = json!({
        "severity": severity_name(issue.severity),
        "category": issue_category(issue),
        "repairable": issue_repairable(issue),
        "blocking": issue_blocking(issue),
        "kind": &issue.kind,
        "message": &issue.message,
        "path": &issue.path,
    });
    if let Some(divergence) = issue.open_snapshot.as_ref() {
        value["phase"] = Value::String(divergence.phase.to_string());
        value["authority"] = Value::String(divergence.authority.to_string());
        value["expected"] = Value::String(divergence.expected.clone());
        value["actual"] = Value::String(divergence.actual.clone());
        value["expected_record"] = json!(&divergence.expected_record);
        value["actual_record"] = json!(&divergence.actual_record);
        value["repo"] = json!(&divergence.repo_root);
        value["branch"] = Value::String(divergence.branch.clone());
    }
    if let Some(pending) = issue.pending_authority.as_ref() {
        value["authority"] = Value::String(pending.authority.as_str().to_string());
        value["operation"] = Value::String(pending.operation.as_str().to_string());
        value["phase"] = Value::String(pending.phase.clone());
        value["request_identity"] = pending_request_identity_json(&pending.request_identity);
        if let Some(intended_state) = pending.intended_state.as_ref() {
            value["intended_state"] = Value::String(intended_state.clone());
        }
    }
    value
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
        | "missing_repairable_directory"
        | "missing_repairable_object_subdirectory"
        | "empty_object_store" => "layout",
        "invalid_object_record"
        | "object_integrity_error"
        | "object_type_directory_mismatch"
        | "unknown_object_type" => "object_store",
        "invalid_branch_record"
        | "missing_branch_head"
        | "invalid_branch_head"
        | "legacy_branch_head_semantic_profile" => "branch",
        "invalid_open_registry"
        | "invalid_open_base_commit"
        | "legacy_open_base_semantic_profile"
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
        | "missing_open_registry_state"
        | crate::open_snapshot::OPEN_SNAPSHOT_DIVERGENCE_KIND => "opened_registry",
        "invalid_active_state_dir"
        | "invalid_active_state_json"
        | "invalid_active_state_shape"
        | "missing_active_state_file"
        | "active_state_invariant_violation" => "active_state",
        "pending_active_ledger_transaction" | "invalid_active_ledger_transaction" => {
            "active_ledger_transaction"
        }
        "pending_commit_transaction" => "commit_transaction",
        "pending_state_transaction" => "state_transaction",
        "pending_open_transaction" | "invalid_open_transaction" => "open_transaction",
        "pending_close_transaction" | "invalid_close_transaction" => "close_transaction",
        "conflicting_repository_transactions" => "repository_transaction",
        _ => "repository",
    }
}

fn issue_repairable(issue: &DoctorIssue) -> bool {
    matches!(
        issue.kind.as_str(),
        "missing_repo_directory"
            | "missing_object_subdirectory"
            | "missing_repairable_directory"
            | "missing_repairable_object_subdirectory"
            | "empty_object_store"
            | "missing_open_path_dir"
            | "missing_active_state_dir"
            | "invalid_active_state_json"
            | "invalid_active_state_shape"
            | "missing_active_state_file"
            | "active_state_invariant_violation"
            | "missing_open_marker"
            | "invalid_open_marker"
            | "open_marker_repository_mismatch"
            | "open_marker_branch_mismatch"
            | "open_marker_instance_mismatch"
            | "open_marker_path_mismatch"
            | "missing_open_registry_state"
            | "pending_open_transaction"
            | "pending_close_transaction"
            | "legacy_branch_head_semantic_profile"
            | "legacy_open_base_semantic_profile"
    )
}

pub(super) fn issue_blocking(issue: &DoctorIssue) -> bool {
    issue.severity == DoctorSeverity::Error
}
