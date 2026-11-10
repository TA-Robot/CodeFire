//! Text and JSON rendering shared by the command handlers.

use crate::automation::{
    change_coverage_data_json, cli_command, command_result_envelope, status_data_json,
};
use crate::cli_error::CliError;
use crate::http::is_cf_http_url;
use crate::json_files::read_active_json_optional;
use crate::merge_resolution::unresolved_merge_conflicts_from_state;
use crate::open_context::open_context;
use crate::scan_verify::compute_scan_with_cache;
#[cfg(test)]
use crate::Branch;
use crate::{
    automation, find_open_marker, find_repo_root, load_branch_record, opened_registry_path,
    pagination, read_json, required_string, scan_branch_state, scan_change_count, CommitResult,
    Status,
};
use serde_json::{json, Value};
use std::env;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(crate) fn print_status(status: &Status) {
    println!("Branch: {}", status.branch);
    println!("State: {}", status.state);
    println!("Base: {}", status.base);
    println!("Open fires: {}", status.open_fires);
}

/// `status` reports the last recorded state. When the working files have
/// moved past it, say so and name the command that records the change.
pub(crate) fn unscanned_changes_line(status: &Status, data: &Value) -> Option<String> {
    let prediction = data.get("scan_prediction")?;
    let predicted_state = prediction.get("branch_state")?.as_str()?;
    if predicted_state == status.state {
        return None;
    }
    let count = |field: &str| prediction.get(field).and_then(Value::as_u64).unwrap_or(0);
    Some(format!(
        "Unscanned changes: {} changed item(s), {} fire(s) expected; run `codefire scan`",
        count("changed_count"),
        count("open_fire_count")
    ))
}

pub(crate) fn status_data_json_with_prediction(
    status: &Status,
    path: &Path,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Value {
    let mut data = status_data_json(status);
    if let Ok(scan_execution) = compute_scan_with_cache(path, None, validation_cache) {
        let scan = scan_execution.scan;
        data["scan_prediction"] = json!({
            "source": "preview_recomputed",
            "stale_resolution_count": stale_resolution_count(path, &scan),
            "branch_state": scan_branch_state(scan_change_count(&scan), scan.open_fires.len()),
            "changed_count": scan_change_count(&scan),
            "changed_atom_count": scan.changed_atoms.len(),
            "non_atom_changed_file_count": scan.non_atom_changed_files.len(),
            "open_fire_count": scan.open_fires.len(),
            "tool_migration": &scan.tool_migration,
            "change_coverage": change_coverage_data_json(
                scan.change_coverage.as_ref(),
                automation::MAX_SCAN_JSON_SAMPLE,
            ),
            "base_commit": &scan.base_commit,
        });
    }
    data
}

pub(crate) fn print_branch_show(data: &Value) {
    let branch = data.get("branch").unwrap_or(&Value::Null);
    println!(
        "Branch: {}",
        branch.get("name").and_then(Value::as_str).unwrap_or("")
    );
    println!(
        "Head: {}",
        branch.get("head").and_then(Value::as_str).unwrap_or("")
    );
    println!(
        "State: {}",
        branch.get("state").and_then(Value::as_str).unwrap_or("")
    );
    println!(
        "Sealed: {}",
        branch
            .get("sealed_validation")
            .and_then(|value| value.get("ok"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    );
    let opened = branch.get("opened").unwrap_or(&Value::Null);
    if opened.get("present").and_then(Value::as_bool) == Some(true) {
        println!(
            "Open path: {}",
            opened
                .get("open")
                .and_then(|value| value.get("opened_path"))
                .or_else(|| opened.get("open").and_then(|value| value.get("path")))
                .and_then(Value::as_str)
                .unwrap_or("")
        );
    }
}

#[cfg(test)]
pub(crate) fn branch_json(branch: &Branch) -> Value {
    json!({
        "name": &branch.name,
        "head": &branch.head,
        "state": &branch.state,
    })
}

pub(crate) fn is_remote_project_url(value: &str) -> bool {
    value.starts_with("cf://") || is_cf_http_url(value)
}

#[cfg(test)]
pub(crate) fn local_list_data_json(branches: &[Branch], opened: Vec<Value>) -> Value {
    json!({
        "type": "codefire_local_list",
        "version": 1,
        "branches": branches.iter().map(branch_json).collect::<Vec<_>>(),
        "opened": opened,
    })
}

pub(crate) fn remote_branch_list_data_json(project_url: &str, page: &pagination::Page) -> Value {
    json!({
        "type": "codefire_remote_branch_list",
        "version": 1,
        "project_url": project_url,
        "branches": &page.items,
        "page": &page.metadata,
    })
}

pub(crate) fn merge_request_list_data_json(project_url: &str, page: &pagination::Page) -> Value {
    json!({
        "type": "codefire_merge_request_list",
        "version": 1,
        "project_url": project_url,
        "merge_requests": &page.items,
        "page": &page.metadata,
    })
}

pub(crate) fn branch_show_data(
    start: &Path,
    branch: Option<&str>,
) -> Result<(PathBuf, Value), CliError> {
    let repo_root = find_repo_root(start)?;
    let branch_name = match branch {
        Some(branch) => branch.to_string(),
        None => {
            let marker_path =
                find_open_marker(start)?.ok_or_else(|| CliError::NotOpen(start.to_path_buf()))?;
            let marker = read_json(&marker_path)?;
            required_string(&marker, &["branch", "name"])?
        }
    };
    let record = load_branch_record(&repo_root, &branch_name)?;
    let head = required_string(&record, &["head"])?;
    let objects_root = repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects_root, &head)?;
    let state = record
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed");
    let registry_path = opened_registry_path(&repo_root, &branch_name);
    let opened = if registry_path.exists() {
        let registry = read_json(&registry_path)?;
        json!({
            "present": true,
            "registry_path": registry_path,
            "branch": registry.get("branch").cloned().unwrap_or_else(|| json!({})),
            "open": registry.get("open").cloned().unwrap_or_else(|| json!({})),
        })
    } else {
        json!({
            "present": false,
            "registry_path": registry_path,
        })
    };
    Ok((
        repo_root,
        json!({
            "type": "codefire_branch_detail",
            "version": 1,
            "branch": {
                "name": branch_name,
                "head": head,
                "state": state,
                "sealed_validation": {
                    "ok": true,
                    "objects": objects_root,
                },
                "opened": opened,
            },
        }),
    ))
}

pub(crate) fn branch_unsupported_command_envelope(
    command: &str,
    repo_root: Option<&Path>,
    exit_code: i32,
) -> Value {
    let supported = vec!["list", "show"];
    command_result_envelope(
        "branch",
        false,
        exit_code,
        repo_root,
        json!({
            "type": "codefire_branch_error",
            "unsupported_subcommand": command,
            "supported_subcommands": supported,
        }),
        vec![json!({
            "kind": "unsupported_subcommand",
            "severity": "blocking",
            "command": "branch",
            "subcommand": command,
            "supported_subcommands": ["list", "show"],
            "message": format!("unsupported branch command: {command}"),
        })],
        vec![
            json!({
                "kind": "show_help",
                "command": "codefire branch --help",
                "reason": "inspect supported branch subcommands",
                "target": {"command": "branch", "supported_subcommands": ["list", "show"]},
            }),
            json!({
                "kind": "branch_list",
                "command": "codefire branch list --json",
                "reason": "list known branches",
                "target": {"command": "branch list"},
            }),
            json!({
                "kind": "branch_show",
                "command": "codefire branch show --json",
                "reason": "show the current branch detail",
                "target": {"command": "branch show"},
            }),
        ],
    )
}

pub(crate) fn branch_error_repo_root(args: &[String]) -> Option<PathBuf> {
    let mut path = env::current_dir().ok()?;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--path" => {
                index += 1;
                if let Some(value) = args.get(index) {
                    path = PathBuf::from(value);
                }
            }
            value if value.starts_with("--path=") => {
                path = PathBuf::from(value.trim_start_matches("--path="));
            }
            _ => {}
        }
        index += 1;
    }
    find_repo_root(&path).ok()
}

/// A merge whose conflicts are not all resolved blocks verify and commit;
/// say so wherever the user looks first. Best effort, like the stale count.
pub(crate) fn merge_in_progress_line(path: &Path) -> Option<String> {
    let context = open_context(path).ok()?;
    let state =
        read_active_json_optional(&context.active_state, Path::new("state.json")).ok()??;
    let unresolved = unresolved_merge_conflicts_from_state(&state).ok()?;
    (!unresolved.is_empty()).then(|| {
        format!(
            "Merge in progress: {} unresolved item(s); run `codefire merge --continue` to list them",
            unresolved.len()
        )
    })
}

/// Resolutions that a later edit made stale. Best effort: scan output must
/// not fail because the policy cannot be read; verify reports that properly.
pub(crate) fn stale_resolution_count(path: &Path, scan: &codefire_core::ScanResult) -> usize {
    let Ok(context) = open_context(path) else {
        return 0;
    };
    let Ok(parsed_policy) = codefire_core::parse_effective_verification_policy(&context.open_dir)
    else {
        return 0;
    };
    let resolutions: Vec<codefire_core::Resolution> =
        match read_active_json_optional(&context.active_state, Path::new("resolutions.json")) {
            Ok(Some(value)) => serde_json::from_value(value).unwrap_or_default(),
            _ => return 0,
        };
    codefire_core::stale_resolutions(
        &scan.atom_index,
        &scan.trace_graph,
        &parsed_policy.policy,
        &resolutions,
    )
    .map_or(0, |stale| stale.len())
}

pub(crate) fn print_scan(scan: &codefire_core::ScanResult) {
    print!("{}", render_scan(scan));
}

pub(crate) fn render_scan(scan: &codefire_core::ScanResult) -> String {
    let state = scan_branch_state(scan_change_count(scan), scan.open_fires.len());
    let mut output = String::new();
    output.push_str(&format!("Branch state: {state}\n"));
    if scan.changed_atoms.is_empty() {
        output.push_str("Changed atoms: none\n");
    } else {
        output.push_str("Changed atoms:\n");
        for atom_id in &scan.changed_atoms {
            output.push_str(&format!("  {atom_id}\n"));
        }
    }
    if scan.non_atom_changed_files.is_empty() {
        output.push_str("Non-atom changed files: none\n");
    } else {
        output.push_str("Non-atom changed files:\n");
        for file in &scan.non_atom_changed_files {
            output.push_str(&format!("  {} {}\n", file.status, file.path));
        }
    }
    if let Some(migration) = &scan.tool_migration {
        output.push_str(&format!(
            "Tool migration: {} hash schema {} -> {} affected_atoms={}\n",
            migration.kind,
            migration.from_hash_schema_version,
            migration.to_hash_schema_version,
            migration.affected_atoms.len()
        ));
    }
    if let Some(coverage) = &scan.change_coverage {
        output.push_str(&format!(
            "Change coverage: {} changed file(s), {} non-Atom, {} deleted-Atom, {} out-of-Atom, {} Atom file(s) with unclassified spans [{}]\n",
            coverage.summary.changed_files,
            coverage.summary.non_atom_files,
            coverage.summary.deleted_atom_files,
            coverage.summary.out_of_atom_files,
            coverage.summary.atom_files_with_unclassified_spans,
            coverage.span_precision,
        ));
    }
    if scan.open_fires.is_empty() {
        output.push_str("Open fires: none\n");
    } else {
        output.push_str("Open fires:\n");
        for fire in &scan.open_fires {
            output.push_str(&format!(
                "  {} ({})  {} -> {}  {}",
                fire.display_id,
                fire.fire_uid,
                fire.source.atom_id,
                fire.target.atom_id,
                fire.reason
            ));
            output.push('\n');
        }
    }
    output.push_str(scan_next_step(state, scan.open_fires.len()));
    output
}

/// What to do after a scan, for someone who has not learned the loop yet.
pub(crate) fn scan_next_step(state: &str, open_fires: usize) -> &'static str {
    if open_fires > 0 {
        "Next: each fire asks you to confirm that its target still agrees with the changed source.\n\
         \x20 one fire:   codefire extinguish <FIRE-id> --resolution <type> --rationale <text>\n\
         \x20 many fires: codefire extinguish --batch-template --rationale <text> --output <file>, then codefire extinguish --batch <file>\n\
         \x20 then:       codefire verify\n"
    } else if state == "open-clean" {
        ""
    } else {
        "Next: codefire verify, then codefire commit -m <message>\n"
    }
}

pub(crate) fn print_lock_contention_json(command: &str, error: &CliError) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&lock_contention_envelope(command, error))?
    );
    Ok(())
}

pub(crate) fn print_plan_result_json(command: &str, plan: &Value) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&plan_result_envelope(command, plan))?
    );
    Ok(())
}

pub(crate) fn print_data_result_json(command: &str, data: Value) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&command_result_envelope(
            command,
            true,
            0,
            None,
            data,
            Vec::new(),
            Vec::new(),
        ))?
    );
    Ok(())
}

pub(crate) fn print_commit_result_json(result: &CommitResult) -> Result<(), CliError> {
    let next_actions = if result.blocked || result.dry_run {
        result
            .plan
            .get("next_actions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&command_result_envelope(
            "commit",
            !result.blocked,
            result.exit_code.code(),
            Some(&result.repo_root),
            commit_result_data_json(result),
            Vec::new(),
            next_actions,
        ))?
    );
    Ok(())
}

pub(crate) fn commit_result_data_json(result: &CommitResult) -> Value {
    json!({
        "type": "codefire_commit_result",
        "version": 1,
        "dry_run": result.dry_run,
        "blocked": result.blocked,
        "applied": !result.dry_run && !result.blocked,
        "commit_id": if result.commit_id.is_empty() { Value::Null } else { Value::String(result.commit_id.clone()) },
        "branch": &result.branch,
        "open_dir": &result.open_dir,
        "plan": &result.plan,
        "message": &result.plan["message"],
        "changed_atom_count": &result.plan["changed_atom_count"],
        "changed_atoms": &result.plan["changed_atoms"],
        "changed_atoms_omitted": &result.plan["changed_atoms_omitted"],
        "non_atom_changed_file_count": &result.plan["non_atom_changed_file_count"],
        "non_atom_changed_files": &result.plan["non_atom_changed_files"],
        "non_atom_changed_files_omitted": &result.plan["non_atom_changed_files_omitted"],
        "change_coverage": &result.plan["change_coverage"],
        "extinguished_fire_count": &result.plan["extinguished_fire_count"],
        "active_resolution_count": &result.plan["active_resolution_count"],
        "evidence_ref_count": &result.plan["evidence_ref_count"],
        "verification": &result.plan["verification"],
    })
}

pub(crate) fn review_pack_result_data(
    output_path: Option<&PathBuf>,
    payload: &str,
) -> Result<Value, CliError> {
    let value: Value = serde_json::from_str(payload)?;
    Ok(json!({
        "type": "codefire_review_pack_result",
        "version": 1,
        "output_path": output_path,
        "bytes": payload.len(),
        "base": value.get("base").cloned().unwrap_or(Value::Null),
        "source": value.get("source").cloned().unwrap_or(Value::Null),
        "options": value.get("options").cloned().unwrap_or_else(|| json!({})),
        "included_sections": [
            "base",
            "source",
            "options",
            "file_diff",
            "semantic_diff",
            "verification",
            "next_actions"
        ],
        "payload_in_envelope": false,
    }))
}

pub(crate) fn patch_export_result_data(
    output_path: Option<&PathBuf>,
    payload: &str,
) -> Result<Value, CliError> {
    let value: Value = serde_json::from_str(payload)?;
    Ok(json!({
        "type": "codefire_patch_export_result",
        "version": 1,
        "output_path": output_path,
        "bytes": payload.len(),
        "base": value.get("base").cloned().unwrap_or(Value::Null),
        "source": value.get("source").cloned().unwrap_or(Value::Null),
        "summary": value.get("summary").cloned().unwrap_or_else(|| json!({})),
        "omissions": value.get("omissions").cloned().unwrap_or_else(|| json!([])),
        "entries": value
            .get("entries")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        "next_actions": value.get("next_actions").cloned().unwrap_or_else(|| json!([])),
        "payload_in_envelope": false,
    }))
}

pub(crate) fn args_want_json(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

pub(crate) fn diff_result_envelope(
    repo_root: Option<&Path>,
    output: &str,
) -> Result<Value, CliError> {
    let data: Value = serde_json::from_str(output)?;
    let next_actions = data
        .get("next_actions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(command_result_envelope(
        "diff",
        true,
        0,
        repo_root,
        data,
        Vec::new(),
        next_actions,
    ))
}

pub(crate) fn print_cli_error_json(command: &str, error: &CliError) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&cli_error_envelope(command, error))?
    );
    Ok(())
}

pub(crate) fn cli_error_envelope(command: &str, error: &CliError) -> Value {
    let repo_root = match error {
        CliError::MissingEvidenceRef { repo_root, .. } => Some(repo_root.as_path()),
        CliError::RepositoryLockContention { repo_root, .. } => Some(repo_root.as_path()),
        CliError::BranchOperation { repo_root, .. } => Some(repo_root.as_path()),
        CliError::OpenTargetResolution(error) => Some(error.repo_root.as_path()),
        CliError::OpenSnapshotDivergence(divergence) => Some(divergence.repo_root.as_path()),
        CliError::OpenRecoveryPending(pending) => Some(pending.repo_root.as_path()),
        CliError::CoreContext { repo_root, .. } | CliError::StoreContext { repo_root, .. } => {
            Some(repo_root.as_path())
        }
        _ => None,
    };
    command_result_envelope(
        command,
        false,
        error.exit_code(),
        repo_root,
        json!({ "type": "codefire_command_error" }),
        vec![cli_error_diagnostic(error)],
        cli_error_next_actions(command, error),
    )
}

pub(crate) fn cli_error_diagnostic(error: &CliError) -> Value {
    error.diagnostic()
}

pub(crate) fn cli_error_next_actions(command: &str, error: &CliError) -> Vec<Value> {
    match error {
        CliError::Usage(message) if message.starts_with("unsupported command:") => vec![json!({
            "kind": "show_help",
            "command": cli_command("--help"),
            "reason": "inspect supported commands before retrying",
            "target": {"error": message},
        })],
        CliError::Usage(message) if message.starts_with("unknown branch:") => vec![json!({
            "kind": "list_branches",
            "command": cli_command("branch list --json"),
            "reason": "inspect available branch names before retrying the command",
            "target": {"error": message},
        })],
        CliError::BranchOperation {
            message,
            repo_root,
            branch,
            path,
        } => branch_error_next_actions(message, repo_root, branch.as_deref(), path.as_deref()),
        CliError::OpenTargetResolution(error) => vec![json!({
            "kind": "choose_safe_open_target",
            "command": cli_command(format!(
                "open {} <path-under-{}> --dry-run --json",
                command_arg(&error.branch),
                command_arg(&error.workspace_boundary.to_string_lossy())
            )),
            "reason": "choose a non-overlapping target below the resolved workspace boundary and preview it before applying",
            "target": {
                "repo": error.repo_root,
                "branch": error.branch,
                "requested_path": error.requested_path,
                "workspace_boundary": error.workspace_boundary,
            },
        })],
        CliError::Usage(message) if unknown_target_parts(message).is_some() => vec![json!({
            "kind": "refresh_scan",
            "command": cli_command("scan --json"),
            "reason": "refresh changed atoms and open fires before retrying the target-specific command",
            "target": {
                "error": message,
            },
        })],
        CliError::Usage(message) if unsupported_option_command(message).is_some() => {
            let command = unsupported_option_command(message).expect("checked above");
            vec![json!({
                "kind": "show_help",
                "command": cli_command(format!("{command} --help")),
                "reason": "inspect supported options before retrying the command",
                "target": {
                    "command": command,
                    "error": message,
                },
            })]
        }
        CliError::Usage(message) if batch_schema_command(message).is_some() => {
            let command = batch_schema_command(message).expect("checked above");
            vec![json!({
                "kind": "show_batch_schema",
                "command": cli_command(format!("{command} --help")),
                "reason": "inspect the required batch wrapper schema before retrying",
                "target": {
                    "command": command,
                    "error": message,
                },
            })]
        }
        CliError::BatchFileRead {
            operation, path, ..
        } => vec![
            json!({
                "kind": "check_batch_path",
                "command": cli_command(format!(
                    "{} --batch {} --json",
                    batch_operation_help_command(operation),
                    command_arg(&path.to_string_lossy())
                )),
                "reason": "verify that the batch file path exists and is readable",
                "target": {
                    "operation": operation,
                    "path": path,
                },
            }),
            json!({
                "kind": "show_batch_schema",
                "command": cli_command(format!("{} --help", batch_operation_help_command(operation))),
                "reason": "generate a batch file that matches the command schema before retrying",
                "target": {
                    "operation": operation,
                },
            }),
        ],
        CliError::EvidenceCommandFailed { cwd, .. } => vec![json!({
            "kind": "allow_failed_command",
            "command": cli_command("evidence add ... --allow-failed-command --json"),
            "reason": "retry only if a failed command log is intentionally being captured as evidence",
            "target": {
                "cwd": cwd,
                "error": error.to_string(),
            },
        })],
        CliError::NotOpen(path) => vec![json!({
            "kind": "inspect_repository",
            "command": cli_command(format!("doctor --path {} --json", command_arg(&path.to_string_lossy()))),
            "reason": "inspect repository/open-directory discovery before retrying",
            "target": {"path": path},
        })],
        CliError::OpenSnapshotDivergence(divergence) => vec![json!({
            "kind": "inspect_open_snapshot_divergence",
            "command": cli_command(format!(
                "doctor --path {} --json",
                command_arg(&divergence.repo_root.to_string_lossy())
            )),
            "reason": "inspect divergent open authorities without rewriting branch, registry, marker, or active state",
            "target": {
                "repo": &divergence.repo_root,
                "branch": &divergence.branch,
                "phase": &divergence.phase,
                "authority": divergence.authority,
                "expected": &divergence.expected,
                "actual": &divergence.actual,
            },
        })],
        CliError::OpenRecoveryPending(pending) => vec![json!({
            "kind": "inspect_pending_open_recovery",
            "command": cli_command(format!(
                "doctor --path {} --json",
                command_arg(&pending.repo_root.to_string_lossy())
            )),
            "reason": "inspect the pending transaction and rerun only its exact owning operation",
            "target": {
                "repo": &pending.repo_root,
                "branch": &pending.branch,
                "open_dir": &pending.open_dir,
                "operation": &pending.operation,
                "phase": &pending.phase,
            },
        })],
        CliError::InvalidMarker(_) | CliError::InvalidRepository(_) => vec![
            json!({
                "kind": "doctor",
                "command": cli_command("doctor --json"),
                "reason": "inspect repository health and corruption diagnostics",
                "target": {"error": error.to_string()},
            }),
            json!({
                "kind": "migrate_check",
                "command": cli_command("migrate check --json"),
                "reason": "inspect repository layout compatibility and planned repairs",
                "target": {"error": error.to_string()},
            }),
        ],
        CliError::MissingEvidenceRef { evidence_id, .. } => vec![
            json!({
                "kind": "create_evidence",
                "command": cli_command("evidence add --path <open-dir> --from-command <cmd> --json"),
                "reason": "create an evidence object before referencing it from a resolution",
                "target": {"missing_evidence_id": evidence_id},
            }),
            json!({
                "kind": "remove_evidence_ref",
                "command": cli_command("extinguish <fire> --resolution <type> --rationale <text> --json"),
                "reason": "retry without the missing evidence reference if the resolution should not cite it",
                "target": {"missing_evidence_id": evidence_id},
            }),
        ],
        CliError::RepositoryLockContention {
            repo_root,
            lock_path,
            ..
        } => vec![json!({
            "kind": "retry_with_wait_lock",
            "command": cli_command(format!("{command} ... --wait-lock --lock-timeout 2s")),
            "reason": "retry after the repository lock is released or wait up to a bounded timeout",
            "target": {
                "repo": repo_root,
                "lock_path": lock_path,
            },
        })],
        CliError::LockContention(_) => vec![json!({
            "kind": "retry_with_wait_lock",
            "command": cli_command(format!("{command} ... --wait-lock --lock-timeout 2s")),
            "reason": "retry after the resource lock is released or wait up to a bounded timeout",
            "target": {"error": error.to_string()},
        })],
        CliError::CoreContext {
            repo_root, path, ..
        }
        | CliError::StoreContext {
            repo_root, path, ..
        } => {
            let doctor_command = cli_command(format!(
                "doctor --path {} --json",
                command_arg(&repo_root.to_string_lossy())
            ));
            let mut actions = vec![json!({
                "kind": "doctor",
                "command": doctor_command,
                "reason": "inspect repository integrity before retrying the command",
                "target": {
                    "repo": repo_root,
                    "path": path,
                    "error": error.to_string(),
                },
            })];
            if command == "commit" {
                actions.push(json!({
                    "kind": "inspect_pending_commit_transaction",
                    "command": cli_command(format!(
                        "doctor --path {} --json",
                        command_arg(&repo_root.to_string_lossy())
                    )),
                    "reason": "inspect the pending commit transaction before recovery or retry",
                    "target": {
                        "repo": repo_root,
                        "candidate_object_path": path,
                    },
                }));
            }
            actions
        }
        _ => Vec::new(),
    }
}

pub(crate) fn branch_error_next_actions(
    message: &str,
    repo_root: &Path,
    branch: Option<&str>,
    path: Option<&Path>,
) -> Vec<Value> {
    let mut actions = vec![json!({
        "kind": "list_branches",
        "command": cli_command(format!(
            "branch list --path {} --json",
            command_arg(&repo_root.to_string_lossy())
        )),
        "reason": "inspect branch state in the repository before retrying the branch operation",
        "target": {
            "repo": repo_root,
            "branch": branch,
            "error": message,
        },
    })];
    if let Some(path) = path {
        actions.push(json!({
            "kind": "status",
            "command": cli_command(format!(
                "status --path {} --json",
                command_arg(&path.to_string_lossy())
            )),
            "reason": "refresh the open directory state before retrying the branch operation",
            "target": {
                "repo": repo_root,
                "path": path,
                "branch": branch,
            },
        }));
    }
    actions
}

pub(crate) fn batch_file_diagnostic_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "batch_file_missing",
        ErrorKind::PermissionDenied => "batch_file_permission_denied",
        _ => "batch_file_read_failed",
    }
}

pub(crate) fn usage_diagnostic_kind(message: &str) -> &'static str {
    if message.starts_with("unsupported command:") {
        "unsupported_command"
    } else if (message.contains("unsupported") && message.contains("option"))
        || message.contains("does not support --")
    {
        "unsupported_option"
    } else if message.starts_with("usage:") {
        "usage_error"
    } else if message.starts_with("unknown branch:") {
        "unknown_branch"
    } else {
        "invalid_usage"
    }
}

pub(crate) fn batch_schema_command(message: &str) -> Option<&'static str> {
    if message.contains("batch extinguish JSON")
        || message.contains("batch extinguish YAML")
        || message.contains("invalid batch YAML")
        || message.contains("batch fire entry")
        || message.contains("batch defaults")
    {
        Some("extinguish")
    } else if message.contains("fire batch JSON")
        || message.contains("fire batch YAML")
        || message.contains("invalid fire batch YAML")
        || message.contains("fire batch item")
    {
        Some("fire")
    } else if message.contains("evidence batch JSON")
        || message.contains("evidence batch YAML")
        || message.contains("invalid evidence batch YAML")
        || message.contains("evidence batch item")
        || message.contains("evidence batch defaults")
    {
        Some("evidence add")
    } else if message.contains("link batch JSON")
        || message.contains("link batch YAML")
        || message.contains("invalid link batch YAML")
        || message.contains("link batch item")
        || message.contains("link batch defaults")
    {
        Some("link")
    } else {
        None
    }
}

pub(crate) fn batch_operation_help_command(operation: &str) -> &'static str {
    match operation {
        "evidence batch" => "evidence add",
        "fire batch" => "fire",
        "link batch" => "link",
        "extinguish batch" => "extinguish",
        _ => "help",
    }
}

pub(crate) fn unsupported_option_command(message: &str) -> Option<&str> {
    if !message.contains(" option:") {
        return None;
    }
    let remainder = message.strip_prefix("unsupported ")?;
    let command = remainder.split(" option:").next()?;
    Some(match command {
        "storage report" => "storage",
        "batch extinguish" => "extinguish",
        other => other,
    })
}

pub(crate) fn command_arg(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub(crate) fn unknown_target_parts(message: &str) -> Option<(&'static str, &str)> {
    message
        .strip_prefix("unknown atom: ")
        .map(|id| ("atom", id))
        .or_else(|| {
            message
                .strip_prefix("unknown fire: ")
                .map(|id| ("fire", id))
        })
}

pub(crate) fn plan_result_envelope(command: &str, plan: &Value) -> Value {
    let repo_root = plan_repo_root(plan);
    let next_actions = plan
        .get("next_actions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    command_result_envelope(
        command,
        true,
        0,
        repo_root.as_deref(),
        json!({
            "type": "codefire_plan_result",
            "plan": plan,
        }),
        Vec::new(),
        next_actions,
    )
}

pub(crate) fn plan_repo_root(plan: &Value) -> Option<PathBuf> {
    for key in ["repo_root", "repo"] {
        if let Some(path) = plan.get(key).and_then(Value::as_str) {
            return Some(PathBuf::from(path));
        }
    }
    plan.get("open_dir")
        .and_then(Value::as_str)
        .and_then(|path| find_repo_root(Path::new(path)).ok())
}

pub(crate) fn lock_contention_envelope(command: &str, error: &CliError) -> Value {
    command_result_envelope(
        command,
        false,
        error.exit_code(),
        None,
        json!({}),
        vec![json!({
            "kind": "lock_contention",
            "severity": "blocking",
            "message": error.to_string(),
        })],
        vec![json!({
            "kind": "retry_with_wait_lock",
            "command": cli_command(format!("{command} ... --wait-lock --lock-timeout 2s")),
            "description": "retry after the remote resource lock is released or wait up to a bounded timeout",
        })],
    )
}
