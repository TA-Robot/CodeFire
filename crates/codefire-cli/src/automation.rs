use crate::{scan_branch_state, Status};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) const COMMAND_RESULT_SCHEMA: &str = "codefire.command_result.v1";
pub(crate) const CLI_DISPLAY_NAME: &str = "codefire";
pub(crate) const MAX_NEXT_ACTIONS: usize = 12;
pub(crate) const MAX_SCAN_JSON_SAMPLE: usize = 50;

pub(crate) fn command_result_envelope(
    command: &str,
    ok: bool,
    exit_code: i32,
    repo_root: Option<&Path>,
    data: Value,
    diagnostics: Vec<Value>,
    next_actions: Vec<Value>,
) -> Value {
    let next_actions = bounded_next_actions(
        next_actions
            .into_iter()
            .map(normalize_next_action)
            .collect::<Vec<_>>(),
    );
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
        "open-clean" => {}
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

pub(crate) fn scan_data_json_with_full(scan: &codefire_core::ScanResult, full: bool) -> Value {
    let changed_count = scan.changed_atoms.len();
    let open_fire_count = scan.open_fires.len();
    let changed_limit = if full {
        changed_count
    } else {
        MAX_SCAN_JSON_SAMPLE
    };
    let fire_limit = if full {
        open_fire_count
    } else {
        MAX_SCAN_JSON_SAMPLE
    };
    let changed_atom_ids = scan
        .changed_atoms
        .iter()
        .take(changed_limit)
        .cloned()
        .collect::<Vec<_>>();
    let open_fires = scan
        .open_fires
        .iter()
        .take(fire_limit)
        .cloned()
        .collect::<Vec<_>>();
    let atom_by_id = scan
        .atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect::<BTreeMap<_, _>>();
    let changed_atoms_sample = changed_atom_ids
        .iter()
        .map(|atom_id| {
            let atom = atom_by_id.get(atom_id.as_str());
            json!({
                "atom_id": atom_id,
                "kind": atom.map(|atom| atom.kind.as_str()),
                "path": atom.map(|atom| atom.artifact_path.as_str()),
                "selector": atom.map(|atom| &atom.selector),
                "content_hash": atom.map(|atom| atom.content_hash.as_str()),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "branch_state": scan_branch_state(scan.changed_atoms.len(), scan.open_fires.len()),
        "base_commit": &scan.base_commit,
        "tool_migration": &scan.tool_migration,
        "full": full,
        "sample_limit": MAX_SCAN_JSON_SAMPLE,
        "changed_count": changed_count,
        "open_fire_count": open_fire_count,
        "changed_atoms_omitted": changed_count.saturating_sub(changed_atom_ids.len()),
        "open_fires_omitted": open_fire_count.saturating_sub(open_fires.len()),
        "changed_atom_ids": &changed_atom_ids,
        "changed_atoms": &changed_atom_ids,
        "changed_atoms_sample": changed_atoms_sample,
        "open_fires": &open_fires,
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

pub(crate) fn scan_next_actions(
    scan: &codefire_core::ScanResult,
    action_path: Option<&Path>,
) -> Vec<Value> {
    let mut actions = Vec::new();
    if !scan.changed_atoms.is_empty() {
        actions.push(next_action(
            "context_changed",
            cli_command(path_command("context --changed --json", action_path)),
            "inspect changed atoms and related trace context",
            action_target(json!({"changed_atoms": &scan.changed_atoms}), action_path),
        ));
        actions.push(next_action(
            "verify",
            cli_command(path_command("verify --details --json", action_path)),
            "run policy checks against the changed branch",
            action_target(json!({"base_commit": &scan.base_commit}), action_path),
        ));
    }
    if scan.open_fires.len() > 1 {
        actions.push(next_action(
            "extinguish_batch_template",
            cli_command(path_command(
                "extinguish --batch-template --json",
                action_path,
            )),
            "generate a reusable batch template for all current open fires",
            action_target(
                json!({
                    "open_fires": scan.open_fires.len(),
                    "template_format": "json",
                }),
                action_path,
            ),
        ));
    }
    for fire in &scan.open_fires {
        actions.push(next_action(
            "context_fire",
            cli_command(path_command(
                format!("context --fire {} --json", fire.display_id),
                action_path,
            )),
            "inspect the fire source, target, and trace path",
            action_target(
                json!({
                    "display_id": &fire.display_id,
                    "fire_uid": &fire.fire_uid,
                    "source_atom": &fire.source.atom_id,
                    "target_atom": &fire.target.atom_id,
                }),
                action_path,
            ),
        ));
        actions.push(next_action(
            "extinguish_fire",
            cli_command(path_command(
                format!(
                    "extinguish {} --resolution <type> --rationale <text>",
                    fire.display_id
                ),
                action_path,
            )),
            "record a resolution for the open fire",
            action_target(
                json!({
                    "display_id": &fire.display_id,
                    "fire_uid": &fire.fire_uid,
                }),
                action_path,
            ),
        ));
    }
    if actions.is_empty() {
        actions.push(next_action(
            "status",
            cli_command(path_command("status --json", action_path)),
            "scan is clean; inspect current branch state only if another command needs it",
            action_target(
                json!({"base_commit": &scan.base_commit, "pending_changes": false}),
                action_path,
            ),
        ));
    }
    actions
}

pub(crate) fn verification_data_json(verification: &codefire_core::Verification) -> Value {
    json!({
        "result": &verification.result,
        "trace_completeness_required": verification.trace_completeness_required,
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
    let blocking_only = diagnostic_filter == "blocking_only";
    let all_missing_required_links = json!(&verification.missing_required_links);
    let blocking_missing_required_links = if verification.trace_completeness_required {
        json!(&verification.missing_required_links)
    } else {
        json!([])
    };
    data["all_missing_required_links"] = all_missing_required_links;
    data["blocking_missing_required_links"] = blocking_missing_required_links.clone();
    if blocking_only {
        data["missing_required_links"] = blocking_missing_required_links;
    }
    data["diagnostic_filter"] = Value::String(diagnostic_filter.to_string());
    data["diagnostic_summary"] = verification_diagnostic_summary(verification, blocking_only);
    data
}

pub(crate) fn verification_diagnostics_json(
    verification: &codefire_core::Verification,
) -> Vec<Value> {
    verification_diagnostics_json_with_filter(verification, false)
}

pub(crate) fn verification_diagnostics_json_with_filter(
    verification: &codefire_core::Verification,
    blocking_only: bool,
) -> Vec<Value> {
    let mut diagnostics = Vec::new();
    if verification.open_required_fires > 0 {
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "open_required_fires",
            "severity": "error",
            "blocking": true,
            "count": verification.open_required_fires,
            }),
        );
    }
    for item in &verification.missing_required_links {
        let blocking = verification.trace_completeness_required;
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "missing_required_link",
            "severity": if blocking { "error" } else { "warning" },
            "blocking": blocking,
            "atom_id": &item.atom_id,
            "required_type": &item.required_type,
            "target_kind": &item.target_kind,
            "min": item.min,
            "found": item.found,
            }),
        );
    }
    for item in &verification.stale_resolutions {
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "stale_resolution",
            "severity": "error",
            "blocking": true,
            "resolution_uid": &item.resolution_uid,
            "reason": &item.reason,
            }),
        );
    }
    for item in &verification.missing_evidence_refs {
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "missing_evidence_ref",
            "severity": "error",
            "blocking": true,
            "resolution_uid": &item.resolution_uid,
            "evidence_id": &item.evidence_id,
            }),
        );
    }
    for atom_id in &verification.duplicate_atom_ids {
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "duplicate_atom_id",
            "severity": "error",
            "blocking": true,
            "atom_id": atom_id,
            }),
        );
    }
    for check in &verification.failed_checks {
        push_diagnostic(
            &mut diagnostics,
            blocking_only,
            json!({
            "kind": "failed_check",
            "severity": "error",
            "blocking": true,
            "id": &check.id,
            "command": &check.command,
            "output": &check.output,
            }),
        );
    }
    diagnostics
}

fn push_diagnostic(diagnostics: &mut Vec<Value>, blocking_only: bool, diagnostic: Value) {
    if !blocking_only
        || diagnostic
            .get("blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        diagnostics.push(diagnostic);
    }
}

pub(crate) fn verification_next_actions(
    verification: &codefire_core::Verification,
    scan: &codefire_core::ScanResult,
    blocking_only: bool,
    action_path: Option<&Path>,
) -> Vec<Value> {
    let mut actions = Vec::new();
    if verification.result == "passed" {
        if has_pending_scan_changes(scan) {
            actions.push(next_action(
                "commit",
                cli_command(path_command("commit -m <message>", action_path)),
                "seal the verified open branch",
                action_target(pending_changes_target(scan), action_path),
            ));
        } else {
            actions.push(next_action(
                "status",
                cli_command(path_command("status --json", action_path)),
                "branch is verified and has no pending scan changes to commit",
                action_target(pending_changes_target(scan), action_path),
            ));
        }
        return actions;
    }
    if verification.open_required_fires > 0 {
        actions.push(next_action(
            "context_changed",
            cli_command(path_command("context --changed --json", action_path)),
            "inspect changed atoms and open fire context",
            action_target(
                json!({"open_required_fires": verification.open_required_fires}),
                action_path,
            ),
        ));
        actions.push(next_action(
            "scan",
            cli_command(path_command("scan --json", action_path)),
            "refresh open fire diagnostics before extinguishing",
            action_target(json!({}), action_path),
        ));
        if verification.open_required_fires > 1 || scan.open_fires.len() > 1 {
            actions.push(next_action(
                "extinguish_batch_template",
                cli_command(path_command(
                    "extinguish --batch-template --json",
                    action_path,
                )),
                "generate a batch extinguish template for the open required fires",
                action_target(
                    json!({
                        "open_required_fires": verification.open_required_fires,
                        "open_fires": scan.open_fires.len(),
                        "template_format": "json",
                    }),
                    action_path,
                ),
            ));
        }
    }
    if !blocking_only || verification.trace_completeness_required {
        for item in &verification.missing_required_links {
            actions.push(next_action(
                "context_atom",
                cli_command(path_command(
                    format!("context --atom {} --depth 2 --json", item.atom_id),
                    action_path,
                )),
                "inspect the atom and nearby trace graph before adding the missing link",
                action_target(
                    json!({
                        "atom_id": &item.atom_id,
                        "required_type": &item.required_type,
                        "target_kind": &item.target_kind,
                        "min": item.min,
                        "found": item.found,
                        "blocking": verification.trace_completeness_required,
                    }),
                    action_path,
                ),
            ));
        }
    }
    for item in &verification.stale_resolutions {
        actions.push(next_action(
            "refresh_resolution",
            cli_command(path_command(
                "extinguish <fire-id> --refresh --resolution <type> --rationale <text>",
                action_path,
            )),
            "refresh the stale resolution against the current atom and trace basis",
            action_target(
                json!({
                    "resolution_uid": &item.resolution_uid,
                    "reason": &item.reason,
                }),
                action_path,
            ),
        ));
    }
    for item in &verification.missing_evidence_refs {
        actions.push(next_action(
            "repair_evidence_ref",
            cli_command(path_command(
                "evidence add --artifact <path> --json",
                action_path,
            )),
            "recreate or replace the missing evidence object referenced by the resolution",
            action_target(
                json!({
                    "resolution_uid": &item.resolution_uid,
                    "evidence_id": &item.evidence_id,
                }),
                action_path,
            ),
        ));
    }
    for atom_id in &verification.duplicate_atom_ids {
        actions.push(next_action(
            "context_atom",
            cli_command(path_command(
                format!("context --atom {atom_id} --depth 1 --json"),
                action_path,
            )),
            "inspect duplicate Atom ID declarations and rename conflicting atoms",
            action_target(json!({"atom_id": atom_id}), action_path),
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

fn verification_diagnostic_summary(
    verification: &codefire_core::Verification,
    blocking_only: bool,
) -> Value {
    let unfiltered = verification_diagnostics_json_with_filter(verification, false);
    let filtered = verification_diagnostics_json_with_filter(verification, blocking_only);
    let blocking_count = unfiltered
        .iter()
        .filter(|item| {
            item.get("blocking")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let warning_count = unfiltered
        .iter()
        .filter(|item| {
            !item
                .get("blocking")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    json!({
        "filter": if blocking_only { "blocking_only" } else { "all" },
        "filtered_count": filtered.len(),
        "unfiltered_count": unfiltered.len(),
        "blocking_count": blocking_count,
        "warning_count": warning_count,
        "missing_required_links_view": if blocking_only { "blocking" } else { "all" },
    })
}

fn has_pending_scan_changes(scan: &codefire_core::ScanResult) -> bool {
    !scan.changed_atoms.is_empty() || !scan.open_fires.is_empty() || scan.tool_migration.is_some()
}

fn pending_changes_target(scan: &codefire_core::ScanResult) -> Value {
    json!({
        "base_commit": &scan.base_commit,
        "pending_changes": has_pending_scan_changes(scan),
        "changed_atoms": &scan.changed_atoms,
        "open_fires": scan.open_fires.len(),
        "tool_migration": &scan.tool_migration,
    })
}

pub(crate) fn bounded_next_actions(mut actions: Vec<Value>) -> Vec<Value> {
    if actions.len() <= MAX_NEXT_ACTIONS {
        return actions;
    }
    let omitted = actions.len() - (MAX_NEXT_ACTIONS - 1);
    let omitted_actions = actions[(MAX_NEXT_ACTIONS - 1)..].to_vec();
    let omitted_by_kind = omitted_actions_by_kind(&omitted_actions);
    let first_omitted_targets = omitted_actions
        .iter()
        .take(3)
        .map(|action| {
            json!({
                "kind": action.get("kind").or_else(|| action.get("id")).cloned().unwrap_or(Value::Null),
                "target": action.get("target").or_else(|| action.get("context")).cloned().unwrap_or(Value::Null),
                "command": action.get("command").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    let recovery_command = omitted_actions
        .first()
        .and_then(|action| action.get("command"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| cli_command("status --json"));
    actions.truncate(MAX_NEXT_ACTIONS - 1);
    actions.push(next_action(
        "next_actions_omitted",
        recovery_command,
        "additional next actions were omitted; inspect omitted_by_kind and run the recovery command or rerun the source command with a narrower target",
        json!({
            "omitted": omitted,
            "limit": MAX_NEXT_ACTIONS,
            "omitted_by_kind": omitted_by_kind,
            "first_omitted_targets": first_omitted_targets,
        }),
    ));
    actions
}

fn omitted_actions_by_kind(actions: &[Value]) -> Value {
    let mut counts = BTreeMap::new();
    for action in actions {
        let kind = action
            .get("kind")
            .or_else(|| action.get("id"))
            .and_then(Value::as_str)
            .unwrap_or("next_action");
        *counts.entry(kind.to_string()).or_insert(0usize) += 1;
    }
    json!(counts)
}

fn action_target(mut target: Value, action_path: Option<&Path>) -> Value {
    if let Some(path) = action_path {
        if let Some(object) = target.as_object_mut() {
            object
                .entry("path")
                .or_insert_with(|| Value::String(path.to_string_lossy().into_owned()));
        }
    }
    target
}

fn path_command(command: impl AsRef<str>, action_path: Option<&Path>) -> String {
    let command = command.as_ref();
    match action_path {
        Some(path) => format!("{command} --path {}", command_arg(&path.to_string_lossy())),
        None => command.to_string(),
    }
}

fn command_arg(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn normalize_next_action(action: Value) -> Value {
    let Some(object) = action.as_object() else {
        return next_action(
            "invalid_next_action",
            cli_command("status --json"),
            "inspect status because a next_action payload was not an object",
            json!({"raw": action}),
        );
    };
    let kind = object
        .get("kind")
        .or_else(|| object.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("next_action")
        .to_string();
    let command = object
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let reason = object
        .get("reason")
        .or_else(|| object.get("description"))
        .and_then(Value::as_str)
        .unwrap_or("follow the recommended next action")
        .to_string();
    let target = object
        .get("target")
        .or_else(|| object.get("context"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut normalized = action;
    if let Some(map) = normalized.as_object_mut() {
        map.entry("kind")
            .or_insert_with(|| Value::String(kind.clone()));
        map.entry("id").or_insert_with(|| Value::String(kind));
        map.entry("command")
            .or_insert_with(|| Value::String(command));
        map.entry("reason")
            .or_insert_with(|| Value::String(reason.clone()));
        map.entry("description")
            .or_insert_with(|| Value::String(reason));
        map.entry("target").or_insert_with(|| target.clone());
        map.entry("context").or_insert(target);
    }
    normalized
}

pub(crate) fn next_action(
    kind: impl Into<String>,
    command: impl Into<String>,
    reason: impl Into<String>,
    target: Value,
) -> Value {
    let kind = kind.into();
    let command = command.into();
    let reason = reason.into();
    json!({
        "kind": &kind,
        "id": &kind,
        "command": command,
        "reason": &reason,
        "description": &reason,
        "target": &target,
        "context": target,
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
    fn command_result_envelope_normalizes_legacy_next_actions() {
        let envelope = command_result_envelope(
            "doctor",
            false,
            20,
            None,
            json!({}),
            Vec::new(),
            vec![json!({
                "id": "inspect_doctor_report",
                "command": cli_command("doctor --json"),
                "description": "inspect repository diagnostics",
                "context": {"issues": 1},
            })],
        );
        let action = &envelope["next_actions"][0];
        assert_eq!(action["kind"], "inspect_doctor_report");
        assert_eq!(action["id"], "inspect_doctor_report");
        assert_eq!(action["reason"], "inspect repository diagnostics");
        assert_eq!(action["description"], "inspect repository diagnostics");
        assert_eq!(action["target"]["issues"], 1);
        assert_eq!(action["context"]["issues"], 1);
    }

    #[test]
    fn clean_status_has_no_next_action_loop() {
        let status = Status {
            branch: "main".to_string(),
            state: "open-clean".to_string(),
            base: "CF-COMMIT-base".to_string(),
            open_fires: 0,
        };

        assert!(status_next_actions(&status).is_empty());
    }

    #[test]
    fn clean_scan_next_action_is_status_not_verify() {
        let actions = scan_next_actions(&empty_scan(), None);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["kind"], "status");
        assert_eq!(actions[0]["target"]["pending_changes"], false);
    }

    #[test]
    fn scan_next_actions_preserve_explicit_path_context() {
        let mut scan = empty_scan();
        scan.changed_atoms.push("REQ-session".to_string());
        scan.open_fires.push(test_fire("FIRE-001"));

        let actions = scan_next_actions(&scan, Some(Path::new("/tmp/open")));
        assert!(actions
            .iter()
            .all(|action| action["target"]["path"] == "/tmp/open"));
        assert!(actions.iter().all(|action| action["command"]
            .as_str()
            .unwrap()
            .contains("--path /tmp/open")));
    }

    #[test]
    fn bounded_next_actions_report_omitted_kinds_and_recovery_command() {
        let mut scan = empty_scan();
        scan.changed_atoms.push("REQ-session".to_string());
        for index in 0..6 {
            scan.open_fires.push(test_fire(&format!("FIRE-{index:03}")));
        }

        let envelope = command_result_envelope(
            "scan",
            true,
            0,
            None,
            json!({}),
            Vec::new(),
            scan_next_actions(&scan, Some(Path::new("/tmp/open"))),
        );
        let omitted = &envelope["next_actions"][MAX_NEXT_ACTIONS - 1];
        assert_eq!(omitted["kind"], "next_actions_omitted");
        assert_eq!(omitted["target"]["omitted"], 4);
        assert_eq!(omitted["target"]["omitted_by_kind"]["extinguish_fire"], 2);
        assert_eq!(omitted["target"]["omitted_by_kind"]["context_fire"], 2);
        assert!(omitted["target"]["first_omitted_targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| target["command"]
                .as_str()
                .unwrap()
                .contains("--path /tmp/open")));
        assert_ne!(omitted["command"], cli_command("status --json"));
    }

    #[test]
    fn verification_json_diagnostics_report_blockers() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            trace_completeness_required: true,
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
    fn verification_diagnostics_use_policy_aware_missing_link_severity() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "passed".to_string(),
            trace_completeness_required: false,
            open_required_fires: 0,
            failed_checks: Vec::new(),
            missing_required_links: vec![codefire_core::MissingRequiredLink {
                atom_id: "REQ-session".to_string(),
                required_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
                found: 0,
            }],
            stale_resolutions: Vec::new(),
            missing_evidence_refs: Vec::new(),
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        };

        let diagnostics = verification_diagnostics_json(&verification);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["kind"], "missing_required_link");
        assert_eq!(diagnostics[0]["severity"], "warning");
        assert_eq!(diagnostics[0]["blocking"], false);
        assert!(verification_diagnostics_json_with_filter(&verification, true).is_empty());
    }

    #[test]
    fn verification_next_actions_cover_blocker_classes() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            trace_completeness_required: true,
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

        let actions = verification_next_actions(&verification, &empty_scan(), false, None);
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

    #[test]
    fn verification_next_actions_do_not_suggest_commit_for_clean_pass() {
        let verification = passed_verification();
        let actions = verification_next_actions(&verification, &empty_scan(), false, None);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["kind"], "status");
        assert_eq!(actions[0]["target"]["pending_changes"], false);
    }

    #[test]
    fn verification_next_actions_suggest_commit_for_dirty_pass() {
        let verification = passed_verification();
        let mut scan = empty_scan();
        scan.changed_atoms.push("REQ-session".to_string());
        let actions = verification_next_actions(&verification, &scan, false, None);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["kind"], "commit");
        assert_eq!(actions[0]["target"]["pending_changes"], true);
        assert_eq!(actions[0]["target"]["changed_atoms"][0], "REQ-session");
    }

    #[test]
    fn verification_next_actions_preserve_explicit_path_context() {
        let mut verification = passed_verification();
        verification.result = "failed".to_string();
        verification.open_required_fires = 1;
        let actions =
            verification_next_actions(&verification, &empty_scan(), false, Some(Path::new(".")));

        assert!(actions.iter().any(|action| action["kind"] == "scan"
            && action["command"].as_str().unwrap().contains("--path .")
            && action["target"]["path"] == "."));
        assert!(actions
            .iter()
            .any(|action| action["kind"] == "context_changed"
                && action["command"].as_str().unwrap().contains("--path .")
                && action["target"]["path"] == "."));
    }

    #[test]
    fn scan_json_is_bounded_and_reports_changed_atom_details() {
        let mut scan = empty_scan();
        for index in 0..(MAX_SCAN_JSON_SAMPLE + 3) {
            let atom_id = format!("REQ-{index:03}");
            scan.changed_atoms.push(atom_id.clone());
            scan.atom_index.atoms.push(codefire_core::Atom {
                atom_id,
                kind: "requirement".to_string(),
                artifact_path: format!("docs/spec/{index:03}.md"),
                selector: codefire_core::Selector {
                    selector_type: "heading".to_string(),
                    value: format!("REQ-{index:03}"),
                },
                content_hash: format!("sha256:{index:03}"),
            });
        }
        for index in 0..(MAX_SCAN_JSON_SAMPLE + 2) {
            scan.open_fires.push(test_fire(&format!("FIRE-{index:03}")));
        }

        let bounded = scan_data_json_with_full(&scan, false);
        assert_eq!(bounded["full"], false);
        assert_eq!(bounded["changed_count"], MAX_SCAN_JSON_SAMPLE + 3);
        assert_eq!(bounded["open_fire_count"], MAX_SCAN_JSON_SAMPLE + 2);
        assert_eq!(
            bounded["changed_atom_ids"].as_array().unwrap().len(),
            MAX_SCAN_JSON_SAMPLE
        );
        assert_eq!(
            bounded["changed_atoms_sample"].as_array().unwrap().len(),
            MAX_SCAN_JSON_SAMPLE
        );
        assert_eq!(bounded["changed_atoms_omitted"], 3);
        assert_eq!(bounded["open_fires_omitted"], 2);
        assert_eq!(bounded["changed_atoms_sample"][0]["atom_id"], "REQ-000");
        assert_eq!(bounded["changed_atoms_sample"][0]["kind"], "requirement");
        assert_eq!(
            bounded["changed_atoms_sample"][0]["path"],
            "docs/spec/000.md"
        );

        let full = scan_data_json_with_full(&scan, true);
        assert_eq!(
            full["changed_atom_ids"].as_array().unwrap().len(),
            MAX_SCAN_JSON_SAMPLE + 3
        );
        assert_eq!(
            full["open_fires"].as_array().unwrap().len(),
            MAX_SCAN_JSON_SAMPLE + 2
        );
        assert_eq!(full["changed_atoms_omitted"], 0);
        assert_eq!(full["open_fires_omitted"], 0);
    }

    fn test_fire(display_id: &str) -> codefire_core::Fire {
        codefire_core::Fire {
            type_tag: "fire".to_string(),
            version: 1,
            fire_uid: format!("fire_{display_id}"),
            display_id: display_id.to_string(),
            status: "open".to_string(),
            severity: "required".to_string(),
            source: codefire_core::FireAtomRef {
                atom_id: "REQ-session".to_string(),
                content_hash_at_fire: None,
            },
            target: codefire_core::FireAtomRef {
                atom_id: "DES-session".to_string(),
                content_hash_at_fire: None,
            },
            reason: "required trace link missing".to_string(),
            trace_path: Vec::new(),
            created_by: "test".to_string(),
            created_at: "2026-06-10T00:00:00Z".to_string(),
            key: display_id.to_string(),
            obsolete_at: None,
            resolution_uid: None,
        }
    }

    #[test]
    fn verification_blocking_only_json_filters_nonblocking_missing_links() {
        let mut verification = passed_verification();
        verification.trace_completeness_required = false;
        verification.missing_required_links = vec![codefire_core::MissingRequiredLink {
            atom_id: "REQ-session".to_string(),
            required_type: "refined_by".to_string(),
            target_kind: "design".to_string(),
            min: 1,
            found: 0,
        }];

        let data = verification_data_json_with_filter(&verification, "blocking_only");
        assert!(data["missing_required_links"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            data["all_missing_required_links"].as_array().unwrap().len(),
            1
        );
        assert!(data["blocking_missing_required_links"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(data["diagnostic_summary"]["filtered_count"], 0);
        assert_eq!(data["diagnostic_summary"]["unfiltered_count"], 1);
        assert_eq!(data["diagnostic_summary"]["warning_count"], 1);
    }

    #[test]
    fn verification_blocking_only_next_actions_skip_nonblocking_missing_links() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            trace_completeness_required: false,
            open_required_fires: 2,
            failed_checks: Vec::new(),
            missing_required_links: vec![codefire_core::MissingRequiredLink {
                atom_id: "REQ-session".to_string(),
                required_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
                found: 0,
            }],
            stale_resolutions: Vec::new(),
            missing_evidence_refs: Vec::new(),
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        };

        let actions = verification_next_actions(&verification, &empty_scan(), true, None);
        let kinds = actions
            .iter()
            .filter_map(|item| item.get("kind").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"context_changed"));
        assert!(kinds.contains(&"scan"));
        assert!(!kinds.contains(&"context_atom"));
    }

    fn passed_verification() -> codefire_core::Verification {
        codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "passed".to_string(),
            trace_completeness_required: true,
            open_required_fires: 0,
            failed_checks: Vec::new(),
            missing_required_links: Vec::new(),
            stale_resolutions: Vec::new(),
            missing_evidence_refs: Vec::new(),
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        }
    }

    fn empty_scan() -> codefire_core::ScanResult {
        codefire_core::ScanResult {
            type_tag: "scan_result".to_string(),
            version: 1,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: codefire_core::AtomIndex {
                type_tag: "atom_index".to_string(),
                version: 1,
                hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
                atoms: Vec::new(),
                duplicate_atom_ids: Vec::new(),
            },
            trace_graph: codefire_core::TraceGraph {
                type_tag: "trace_graph".to_string(),
                version: 1,
                links: Vec::new(),
            },
            changed_atoms: Vec::new(),
            open_fires: Vec::new(),
            tool_migration: None,
        }
    }
}
