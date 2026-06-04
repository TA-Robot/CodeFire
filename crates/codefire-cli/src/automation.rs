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
}
