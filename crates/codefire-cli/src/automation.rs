use crate::{scan_branch_state, Status};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) const COMMAND_RESULT_SCHEMA: &str = "codefire.command_result.v1";
pub(crate) const CLI_DISPLAY_NAME: &str = "codefire";
pub(crate) const MAX_NEXT_ACTIONS: usize = 12;

pub(crate) fn command_result_envelope(
    command: &str,
    ok: bool,
    exit_code: i32,
    repo_root: Option<&Path>,
    data: Value,
    diagnostics: Vec<Value>,
    next_actions: Vec<Value>,
) -> Value {
    let next_actions = bounded_next_actions(next_actions);
    json!({
        "schema": COMMAND_RESULT_SCHEMA,
        "command": command,
        "ok": ok,
        "exit_code": exit_code,
        "repo": repo_root.map(|path| path.to_string_lossy().into_owned()),
        "data": data,
        "diagnostics": diagnostics,
        "next_actions": next_actions,
    })
}

pub(crate) fn cli_command(args: impl AsRef<str>) -> String {
    let args = args.as_ref();
    if args.is_empty() {
        CLI_DISPLAY_NAME.to_string()
    } else {
        format!("{CLI_DISPLAY_NAME} {args}")
    }
}

pub(crate) fn status_data_json(status: &Status) -> Value {
    json!({
        "branch": &status.branch,
        "state": &status.state,
        "base": &status.base,
        "open_fires": status.open_fires,
    })
}

pub(crate) fn status_next_actions(status: &Status) -> Vec<Value> {
    let mut actions = Vec::new();
    match status.state.as_str() {
        "open-clean" => actions.push(next_action(
            "verify",
            cli_command("verify --json"),
            "confirm the open branch is consistent before commit",
            json!({"branch": &status.branch}),
        )),
        "open-burning" => {
            actions.push(next_action(
                "scan",
                cli_command("scan --json"),
                "refresh changed atoms and open fire diagnostics",
                json!({"branch": &status.branch}),
            ));
            actions.push(next_action(
                "verify",
                cli_command("verify --details --json"),
                "inspect blocking verification diagnostics",
                json!({"branch": &status.branch}),
            ));
        }
        "open-consistent" => actions.push(next_action(
            "commit",
            cli_command("commit -m <message>"),
            "seal the verified open branch",
            json!({"branch": &status.branch, "base": &status.base}),
        )),
        _ => actions.push(next_action(
            "inspect_status",
            cli_command("status --json"),
            "inspect the current open branch state",
            json!({"branch": &status.branch, "state": &status.state}),
        )),
    }
    if status.open_fires > 0 {
        actions.push(next_action(
            "context_changed",
            cli_command("context --changed --json"),
            "inspect changed atoms related to open fires",
            json!({"open_fires": status.open_fires}),
        ));
    }
    actions
}

pub(crate) fn scan_data_json(scan: &codefire_core::ScanResult) -> Value {
    json!({
        "branch_state": scan_branch_state(scan.changed_atoms.len(), scan.open_fires.len()),
        "base_commit": &scan.base_commit,
        "changed_atoms": &scan.changed_atoms,
        "open_fires": &scan.open_fires,
    })
}

pub(crate) fn scan_diagnostics_json(scan: &codefire_core::ScanResult) -> Vec<Value> {
    scan.open_fires
        .iter()
        .map(|fire| {
            json!({
                "kind": "open_fire",
                "severity": &fire.severity,
                "display_id": &fire.display_id,
                "source_atom": &fire.source.atom_id,
                "target_atom": &fire.target.atom_id,
                "reason": &fire.reason,
            })
        })
        .collect()
}

pub(crate) fn scan_next_actions(scan: &codefire_core::ScanResult) -> Vec<Value> {
    let mut actions = Vec::new();
    if !scan.changed_atoms.is_empty() {
        actions.push(next_action(
            "context_changed",
            cli_command("context --changed --json"),
            "inspect changed atoms and related trace context",
            json!({"changed_atoms": &scan.changed_atoms}),
        ));
        actions.push(next_action(
            "verify",
            cli_command("verify --details --json"),
            "run policy checks against the changed branch",
            json!({"base_commit": &scan.base_commit}),
        ));
    }
    for fire in &scan.open_fires {
        actions.push(next_action(
            "context_fire",
            cli_command(format!("context --fire {} --json", fire.display_id)),
            "inspect the fire source, target, and trace path",
            json!({
                "display_id": &fire.display_id,
                "fire_uid": &fire.fire_uid,
                "source_atom": &fire.source.atom_id,
                "target_atom": &fire.target.atom_id,
            }),
        ));
        actions.push(next_action(
            "extinguish_fire",
            cli_command(format!(
                "extinguish {} --resolution <type> --rationale <text>",
                fire.display_id
            )),
            "record a resolution for the open fire",
            json!({
                "display_id": &fire.display_id,
                "fire_uid": &fire.fire_uid,
            }),
        ));
    }
    if actions.is_empty() {
        actions.push(next_action(
            "verify",
            cli_command("verify --json"),
            "confirm the clean scan satisfies verification policy",
            json!({"base_commit": &scan.base_commit}),
        ));
    }
    actions
}

pub(crate) fn verification_data_json(verification: &codefire_core::Verification) -> Value {
    json!({
        "result": &verification.result,
        "open_required_fires": verification.open_required_fires,
        "missing_required_links": &verification.missing_required_links,
        "stale_resolutions": &verification.stale_resolutions,
        "missing_evidence_refs": &verification.missing_evidence_refs,
        "duplicate_atom_ids": &verification.duplicate_atom_ids,
        "failed_checks": &verification.failed_checks,
    })
}

pub(crate) fn verification_data_json_with_filter(
    verification: &codefire_core::Verification,
    diagnostic_filter: &str,
) -> Value {
    let mut data = verification_data_json(verification);
    data["diagnostic_filter"] = Value::String(diagnostic_filter.to_string());
    data
}

pub(crate) fn verification_diagnostics_json(
    verification: &codefire_core::Verification,
) -> Vec<Value> {
    let mut diagnostics = Vec::new();
    if verification.open_required_fires > 0 {
        diagnostics.push(json!({
            "kind": "open_required_fires",
            "severity": "error",
            "count": verification.open_required_fires,
        }));
    }
    for item in &verification.missing_required_links {
        diagnostics.push(json!({
            "kind": "missing_required_link",
            "severity": "error",
            "atom_id": &item.atom_id,
            "required_type": &item.required_type,
            "target_kind": &item.target_kind,
            "min": item.min,
            "found": item.found,
        }));
    }
    for item in &verification.stale_resolutions {
        diagnostics.push(json!({
            "kind": "stale_resolution",
            "severity": "error",
            "resolution_uid": &item.resolution_uid,
            "reason": &item.reason,
        }));
    }
    for item in &verification.missing_evidence_refs {
        diagnostics.push(json!({
            "kind": "missing_evidence_ref",
            "severity": "error",
            "resolution_uid": &item.resolution_uid,
            "evidence_id": &item.evidence_id,
        }));
    }
    for atom_id in &verification.duplicate_atom_ids {
        diagnostics.push(json!({
            "kind": "duplicate_atom_id",
            "severity": "error",
            "atom_id": atom_id,
        }));
    }
    for check in &verification.failed_checks {
        diagnostics.push(json!({
            "kind": "failed_check",
            "severity": "error",
            "id": &check.id,
            "command": &check.command,
            "output": &check.output,
        }));
    }
    diagnostics
}

pub(crate) fn verification_next_actions(verification: &codefire_core::Verification) -> Vec<Value> {
    let mut actions = Vec::new();
    if verification.result == "passed" {
        actions.push(next_action(
            "commit",
            cli_command("commit -m <message>"),
            "seal the verified open branch",
            json!({}),
        ));
        return actions;
    }
    if verification.open_required_fires > 0 {
        actions.push(next_action(
            "context_changed",
            cli_command("context --changed --json"),
            "inspect changed atoms and open fire context",
            json!({"open_required_fires": verification.open_required_fires}),
        ));
        actions.push(next_action(
            "scan",
            cli_command("scan --json"),
            "refresh open fire diagnostics before extinguishing",
            json!({}),
        ));
    }
    for item in &verification.missing_required_links {
        actions.push(next_action(
            "context_atom",
            cli_command(format!("context --atom {} --depth 2 --json", item.atom_id)),
            "inspect the atom and nearby trace graph before adding the missing link",
            json!({
                "atom_id": &item.atom_id,
                "required_type": &item.required_type,
                "target_kind": &item.target_kind,
                "min": item.min,
                "found": item.found,
            }),
        ));
    }
    for item in &verification.stale_resolutions {
        actions.push(next_action(
            "refresh_resolution",
            cli_command("extinguish <fire-id> --refresh --resolution <type> --rationale <text>"),
            "refresh the stale resolution against the current atom and trace basis",
            json!({
                "resolution_uid": &item.resolution_uid,
                "reason": &item.reason,
            }),
        ));
    }
    for item in &verification.missing_evidence_refs {
        actions.push(next_action(
            "repair_evidence_ref",
            cli_command("evidence add --artifact <path> --json"),
            "recreate or replace the missing evidence object referenced by the resolution",
            json!({
                "resolution_uid": &item.resolution_uid,
                "evidence_id": &item.evidence_id,
            }),
        ));
    }
    for atom_id in &verification.duplicate_atom_ids {
        actions.push(next_action(
            "context_atom",
            cli_command(format!("context --atom {atom_id} --depth 1 --json")),
            "inspect duplicate Atom ID declarations and rename conflicting atoms",
            json!({"atom_id": atom_id}),
        ));
    }
    for check in &verification.failed_checks {
        actions.push(next_action(
            "rerun_check",
            check.command.clone(),
            "rerun the failed verification command and capture its output",
            json!({
                "id": &check.id,
                "command": &check.command,
            }),
        ));
    }
    actions
}

pub(crate) fn bounded_next_actions(mut actions: Vec<Value>) -> Vec<Value> {
    if actions.len() <= MAX_NEXT_ACTIONS {
        return actions;
    }
    let omitted = actions.len() - (MAX_NEXT_ACTIONS - 1);
    actions.truncate(MAX_NEXT_ACTIONS - 1);
    actions.push(next_action(
        "next_actions_omitted",
        cli_command("status --json"),
        "additional next actions were omitted to keep the command result bounded",
        json!({
            "omitted": omitted,
            "limit": MAX_NEXT_ACTIONS,
        }),
    ));
    actions
}

fn next_action(
    kind: impl Into<String>,
    command: impl Into<String>,
    reason: impl Into<String>,
    target: Value,
) -> Value {
    json!({
        "kind": kind.into(),
        "command": command.into(),
        "reason": reason.into(),
        "target": target,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn command_result_envelope_has_stable_shape() {
        let envelope = command_result_envelope(
            "status",
            true,
            0,
            Some(Path::new("/repo")),
            status_data_json(&Status {
                branch: "main".to_string(),
                state: "open-clean".to_string(),
                base: "CF-COMMIT-base".to_string(),
                open_fires: 0,
            }),
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(envelope["schema"], COMMAND_RESULT_SCHEMA);
        assert_eq!(envelope["command"], "status");
        assert_eq!(envelope["ok"], true);
        assert_eq!(envelope["exit_code"], 0);
        assert_eq!(envelope["repo"], "/repo");
        assert_eq!(envelope["data"]["branch"], "main");
        assert!(envelope["diagnostics"].as_array().unwrap().is_empty());
        assert!(envelope["next_actions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn command_result_envelope_bounds_next_actions_with_omission_summary() {
        let actions = (0..20)
            .map(|index| {
                json!({
                    "kind": format!("action_{index}"),
                    "command": cli_command("status --json"),
                    "target": {"index": index},
                })
            })
            .collect::<Vec<_>>();

        let envelope =
            command_result_envelope("scan", true, 0, None, json!({}), Vec::new(), actions);
        let bounded = envelope["next_actions"].as_array().unwrap();
        assert_eq!(bounded.len(), MAX_NEXT_ACTIONS);
        assert_eq!(
            bounded[MAX_NEXT_ACTIONS - 1]["kind"],
            "next_actions_omitted"
        );
        assert_eq!(bounded[MAX_NEXT_ACTIONS - 1]["target"]["omitted"], 9);
        assert_eq!(
            bounded[MAX_NEXT_ACTIONS - 1]["target"]["limit"],
            MAX_NEXT_ACTIONS
        );
    }

    #[test]
    fn verification_json_diagnostics_report_blockers() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            open_required_fires: 1,
            failed_checks: vec![codefire_core::FailedCheck {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                output: "failed".to_string(),
            }],
            missing_required_links: vec![codefire_core::MissingRequiredLink {
                atom_id: "REQ-session".to_string(),
                required_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
                found: 0,
            }],
            stale_resolutions: vec![codefire_core::StaleResolution {
                resolution_uid: "resolution_1".to_string(),
                reason: "source changed".to_string(),
            }],
            missing_evidence_refs: vec![codefire_core::MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            duplicate_atom_ids: vec!["REQ-session".to_string()],
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        };

        let diagnostics = verification_diagnostics_json(&verification);
        let kinds = diagnostics
            .iter()
            .filter_map(|item| item.get("kind").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"open_required_fires"));
        assert!(kinds.contains(&"missing_required_link"));
        assert!(kinds.contains(&"stale_resolution"));
        assert!(kinds.contains(&"missing_evidence_ref"));
        assert!(kinds.contains(&"duplicate_atom_id"));
        assert!(kinds.contains(&"failed_check"));
    }

    #[test]
    fn verification_next_actions_cover_blocker_classes() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            open_required_fires: 1,
            failed_checks: vec![codefire_core::FailedCheck {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                output: "failed".to_string(),
            }],
            missing_required_links: vec![codefire_core::MissingRequiredLink {
                atom_id: "REQ-session".to_string(),
                required_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
                found: 0,
            }],
            stale_resolutions: vec![codefire_core::StaleResolution {
                resolution_uid: "resolution_1".to_string(),
                reason: "source changed".to_string(),
            }],
            missing_evidence_refs: vec![codefire_core::MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            duplicate_atom_ids: vec!["REQ-session".to_string()],
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        };

        let actions = verification_next_actions(&verification);
        let kinds = actions
            .iter()
            .filter_map(|item| item.get("kind").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"context_changed"));
        assert!(kinds.contains(&"context_atom"));
        assert!(kinds.contains(&"refresh_resolution"));
        assert!(kinds.contains(&"repair_evidence_ref"));
        assert!(kinds.contains(&"rerun_check"));
        assert!(actions.iter().all(|item| item.get("command").is_some()));
        assert!(actions.iter().all(|item| item.get("target").is_some()));
    }
}
