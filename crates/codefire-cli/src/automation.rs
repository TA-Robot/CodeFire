use crate::Status;
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn command_result_envelope(
    command: &str,
    ok: bool,
    exit_code: i32,
    repo_root: Option<&Path>,
    data: Value,
    diagnostics: Vec<Value>,
    next_actions: Vec<Value>,
) -> Value {
    json!({
        "schema": "codefire.command_result.v1",
        "command": command,
        "ok": ok,
        "exit_code": exit_code,
        "repo": repo_root.map(|path| path.to_string_lossy().into_owned()),
        "data": data,
        "diagnostics": diagnostics,
        "next_actions": next_actions,
    })
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
            "codefire-rs verify --json",
            "confirm the open branch is consistent before commit",
            json!({"branch": &status.branch}),
        )),
        "open-burning" => {
            actions.push(next_action(
                "scan",
                "codefire-rs scan --json",
                "refresh changed atoms and open fire diagnostics",
                json!({"branch": &status.branch}),
            ));
            actions.push(next_action(
                "verify",
                "codefire-rs verify --details --json",
                "inspect blocking verification diagnostics",
                json!({"branch": &status.branch}),
            ));
        }
        "open-consistent" => actions.push(next_action(
            "commit",
            "codefire-rs commit -m <message>",
            "seal the verified open branch",
            json!({"branch": &status.branch, "base": &status.base}),
        )),
        _ => actions.push(next_action(
            "inspect_status",
            "codefire-rs status --json",
            "inspect the current open branch state",
            json!({"branch": &status.branch, "state": &status.state}),
        )),
    }
    if status.open_fires > 0 {
        actions.push(next_action(
            "context_changed",
            "codefire-rs context --changed --json",
            "inspect changed atoms related to open fires",
            json!({"open_fires": status.open_fires}),
        ));
    }
    actions
}

pub(crate) fn scan_data_json(scan: &codefire_core::ScanResult) -> Value {
    json!({
        "branch_state": if scan.changed_atoms.is_empty() && scan.open_fires.is_empty() {
            "open-clean"
        } else {
            "open-burning"
        },
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
            "codefire-rs context --changed --json",
            "inspect changed atoms and related trace context",
            json!({"changed_atoms": &scan.changed_atoms}),
        ));
        actions.push(next_action(
            "verify",
            "codefire-rs verify --details --json",
            "run policy checks against the changed branch",
            json!({"base_commit": &scan.base_commit}),
        ));
    }
    for fire in &scan.open_fires {
        actions.push(next_action(
            "context_fire",
            format!("codefire-rs context --fire {} --json", fire.display_id),
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
            format!(
                "codefire-rs extinguish {} --resolution <type> --rationale <text>",
                fire.display_id
            ),
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
            "codefire-rs verify --json",
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
        "duplicate_atom_ids": &verification.duplicate_atom_ids,
        "failed_checks": &verification.failed_checks,
    })
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
            "codefire-rs commit -m <message>",
            "seal the verified open branch",
            json!({}),
        ));
        return actions;
    }
    if verification.open_required_fires > 0 {
        actions.push(next_action(
            "context_changed",
            "codefire-rs context --changed --json",
            "inspect changed atoms and open fire context",
            json!({"open_required_fires": verification.open_required_fires}),
        ));
        actions.push(next_action(
            "scan",
            "codefire-rs scan --json",
            "refresh open fire diagnostics before extinguishing",
            json!({}),
        ));
    }
    for item in &verification.missing_required_links {
        actions.push(next_action(
            "context_atom",
            format!(
                "codefire-rs context --atom {} --depth 2 --json",
                item.atom_id
            ),
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
            "codefire-rs extinguish <fire-id> --refresh --resolution <type> --rationale <text>",
            "refresh the stale resolution against the current atom and trace basis",
            json!({
                "resolution_uid": &item.resolution_uid,
                "reason": &item.reason,
            }),
        ));
    }
    for atom_id in &verification.duplicate_atom_ids {
        actions.push(next_action(
            "context_atom",
            format!("codefire-rs context --atom {atom_id} --depth 1 --json"),
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

        assert_eq!(envelope["schema"], "codefire.command_result.v1");
        assert_eq!(envelope["command"], "status");
        assert_eq!(envelope["ok"], true);
        assert_eq!(envelope["exit_code"], 0);
        assert_eq!(envelope["repo"], "/repo");
        assert_eq!(envelope["data"]["branch"], "main");
        assert!(envelope["diagnostics"].as_array().unwrap().is_empty());
        assert!(envelope["next_actions"].as_array().unwrap().is_empty());
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
        assert!(kinds.contains(&"rerun_check"));
        assert!(actions.iter().all(|item| item.get("command").is_some()));
        assert!(actions.iter().all(|item| item.get("target").is_some()));
    }
}
