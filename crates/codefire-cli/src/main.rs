use fs2::FileExt as Fs2FileExt;
use serde::ser::{SerializeSeq, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::Mutex;
use std::time::{Duration, Instant};

mod active_state;
mod assessment_subject;
mod automation;
mod batch;
mod branch_refs;
mod cli_model;
mod close_transaction;
mod close_worktree;
mod command_execution;
mod command_registry;
mod commit_active_reset;
mod compatibility;
mod completion;
mod context;
mod doctor;
mod evidence;
mod exit_code;
mod explain;
mod extinguish_ux;
mod fire;
mod http;
mod http_tls;
mod idempotency;
mod import_workflow;
mod limited_yaml;
mod link_batch;
mod merge_patch_idempotency;
mod merge_resolution;
mod metrics;
mod migration;
mod open_clone_idempotency;
mod open_snapshot;
mod open_transaction;
mod pagination;
mod remote;
mod repo_layout;
mod secret_redaction;
mod secure_worktree;
mod signatures;
mod stable_lock;
mod storage;
mod verification;
mod view;
use automation::{
    change_coverage_data_json, cli_command, command_result_envelope, ready_verify_details_action,
    scan_data_json_with_full, scan_diagnostics_json, scan_next_actions, status_data_json,
    status_next_actions_with_prediction, verification_data_json_with_filter,
    verification_diagnostics_json_with_filter, verification_next_actions,
};
use batch::{
    batch_extinguish_data_json, batch_template_data_json, has_batch_extinguish_arg,
    has_batch_template_arg, parse_batch_template_args, parse_extinguish_batch_args,
    print_batch_template_result, run_batch_template, run_extinguish_batch,
};
#[cfg(test)]
use branch_refs::{BranchRefSnapshot, DIRECTORY_ITERATION_SCRATCH_BYTES};
use branch_refs::{
    BranchRefSnapshotError, BranchRefWork, OpenedRegistrySnapshot, RepositoryMetadataSnapshot,
};
pub(crate) use cli_model::*;
use command_registry::{capabilities_data_json, supports_metrics};
#[cfg(test)]
use commit_active_reset::commit_active_reset_state_postimage_bytes;
use commit_active_reset::{
    build_commit_active_reset_authority, commit_active_reset_complete,
    commit_active_reset_unstarted, preflight_commit_active_reset_authority,
    reset_commit_active_state, transition_commit_transaction_to_active_reset,
    validate_commit_active_reset_authority,
};
use completion::{completion_script, help_text};
use context::{build_context_pack, parse_context_args, print_context_summary};
#[cfg(test)]
use doctor::{doctor_report_data_json, doctor_report_diagnostics_json, doctor_report_next_actions};
use doctor::{parse_doctor_args, run_doctor};
use evidence::{
    evidence_add_data_json, evidence_batch_data_json, parse_evidence_add_args,
    print_evidence_add_result, run_evidence_add, run_evidence_batch,
};
use exit_code::{
    core_error_exit_code, store_error_exit_code, usage_exit_code, verification_exit_code, ExitCode,
};
use explain::{parse_explain_args, print_explain_result, run_explain};
use extinguish_ux::{
    has_all_matching_extinguish_arg, has_interactive_extinguish_arg,
    parse_all_matching_extinguish_args, parse_interactive_extinguish_args,
    prepare_extinguish_options, print_all_matching_extinguish_result,
    print_interactive_extinguish_result, run_all_matching_extinguish, run_interactive_extinguish,
};
use fire::active_ledger_transaction::{
    active_ledger_hash_is_valid, active_ledger_request_hash, apply_active_ledger_transition,
    ensure_completed_active_ledger_transition, recover_active_ledger_transition,
    reject_pending_active_ledger_transition, release_completed_active_ledger_transition,
    validate_active_ledger_descriptor, ActiveLedgerPublicationStatus, ActiveLedgerWrite,
};
use fire::{
    fire_batch_data_json, fire_data_json, parse_fire_args, parse_fire_batch_args,
    print_fire_result, run_fire, run_fire_batch,
};
use http::{http_json, http_remote_path, is_cf_http_url, parse_cf_http_url, serve_http};
use idempotency::{
    idempotency_payload_hash, idempotency_record_path, idempotency_result_plan,
    require_idempotency_key, verify_idempotency_record,
};
use import_workflow::{import_existing_project, import_result_data_json, parse_import_args};
use link_batch::{link_batch_data_json, parse_link_batch_args, run_link_batch};
use merge_patch_idempotency::{
    load_merge_idempotency, load_patch_import_idempotency, merge_idempotency_payload,
    patch_import_idempotency_payload, save_merge_idempotency, save_patch_import_idempotency,
};
use merge_resolution::{
    merge_continue_data_json, merge_continue_diagnostics_json, merge_continue_next_actions,
    parse_merge_continue_args, render_merge_continue, run_merge_continue,
    unresolved_merge_conflicts_from_state,
};
use metrics::{
    attach_metrics, branch_list_metrics, metrics_wire, print_metrics, scan_metrics, status_metrics,
    verification_metrics, write_metrics, CommandMetrics, MetricsWire,
};
use migration::{
    migration_report_data_json, migration_report_diagnostics_json, migration_report_next_actions,
    parse_migrate_args, print_migration_report, run_migrate,
};
use open_clone_idempotency::{
    clone_idempotency_payload, load_clone_idempotency, load_open_idempotency,
    open_idempotency_payload, save_clone_idempotency,
};
use pagination::{paginate, parse_page_limit, PageRequest};
use remote::{
    apply_merge_request, copy_object_graph, list_merge_requests, list_remote_branches,
    load_remote_branch, parse_cf_url, print_remote_doctor_report, read_optional_json, remote_dirs,
    remote_doctor_data_json, remote_doctor_diagnostics_json, request_merge, review_merge_request,
    run_remote_doctor, upload_branch, write_object_records, RemoteProjectOptions,
    RequestApplyOptions, RequestMergeOptions, RequestReviewOptions, UploadOptions,
};
use secure_worktree::{ExistingWorktree, StagedWorktree};
use storage::{
    parse_storage_report_args, print_storage_report, run_storage_report, storage_report_data_json,
    storage_report_diagnostics_json, storage_report_next_actions,
};
use verification::{parse_verify_args, print_verification};
use view::{
    diff_commitish_with_options, manifest_contents, patch_export_with_options,
    review_pack_with_options, show_commitish, show_commitish_data, DiffAlgorithm, DiffOptions,
    PatchExportOptions, ReviewPackOptions, DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES,
    DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES,
};

pub(crate) fn scan_branch_state(changed_atoms: usize, open_fires: usize) -> &'static str {
    if changed_atoms == 0 && open_fires == 0 {
        "open-clean"
    } else {
        "open-burning"
    }
}

pub(crate) fn scan_change_count(scan: &codefire_core::ScanResult) -> usize {
    scan.changed_atoms.len() + scan.non_atom_changed_files.len()
}

fn status_state(raw_state: &str, open_fires: usize) -> String {
    if open_fires > 0 {
        "open-burning".to_string()
    } else {
        raw_state.to_string()
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if let Err(error) = run(args.clone()) {
        if !matches!(
            error,
            CliError::VerificationFailed(_) | CliError::CommandFailed(_)
        ) {
            if args_want_json(&args) {
                let command = command_name_for_args(&args);
                if let Err(render_error) = print_cli_error_json(command, &error) {
                    eprintln!("error: {render_error}");
                }
            } else {
                eprintln!("error: {error}");
            }
        }
        std::process::exit(error.exit_code());
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    if let Some(help) = command_help_for_args(&args) {
        print!("{help}");
        return Ok(());
    }
    if let Some(command) = args.first().map(String::as_str) {
        if let Some(handler) = local_workflow_handler(command) {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help(command));
                return Ok(());
            }
            return handler(&args[1..]);
        }
    }
    match args.first().map(String::as_str) {
        Some("-h") | Some("--help") | Some("help") => {
            print!("{}", help_text());
            Ok(())
        }
        Some("completion") => {
            let shell = args.get(1).ok_or_else(|| {
                CliError::Usage("usage: codefire completion <bash|zsh>".to_string())
            })?;
            let script = completion_script(shell).ok_or_else(|| {
                CliError::Usage("usage: codefire completion <bash|zsh>".to_string())
            })?;
            print!("{script}");
            Ok(())
        }
        Some("capabilities") => {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help("capabilities"));
                return Ok(());
            }
            let json_output = args_want_json(&args[1..]);
            for arg in &args[1..] {
                if arg != "--json" {
                    return Err(CliError::Usage(format!(
                        "unsupported capabilities option: {arg}"
                    )));
                }
            }
            let data = capabilities_data_json();
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "capabilities",
                        true,
                        0,
                        None,
                        data,
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            Ok(())
        }
        Some("atom-index") => {
            let options = parse_read_only_debug_args(&args[1..], "atom-index")?;
            let index = codefire_core::build_atom_index(&options.path)?;
            let repo_root = open_context(&options.path)
                .ok()
                .map(|context| context.repo_root);
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "atom-index",
                        true,
                        0,
                        repo_root.as_deref(),
                        json!({
                            "type": "codefire_atom_index",
                            "version": 1,
                            "atom_index": index,
                        }),
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(&index)?);
            }
            Ok(())
        }
        Some("trace-graph") => {
            let options = parse_read_only_debug_args(&args[1..], "trace-graph")?;
            let atom_index = codefire_core::build_atom_index(&options.path)?;
            let trace_graph = codefire_core::current_trace_graph(&options.path)?;
            let open = open_context(&options.path).ok();
            codefire_core::validate_trace_graph_endpoints(&atom_index, &trace_graph).map_err(
                |error| match open.as_ref() {
                    Some(context) => CliError::CoreContext {
                        error,
                        repo_root: context.repo_root.clone(),
                        path: context.open_dir.join("codefire.links.yaml"),
                    },
                    None => CliError::Core(error),
                },
            )?;
            let repo_root = open.map(|context| context.repo_root);
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "trace-graph",
                        true,
                        0,
                        repo_root.as_deref(),
                        json!({
                            "type": "codefire_trace_graph",
                            "version": 1,
                            "trace_graph": trace_graph,
                        }),
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(&trace_graph)?);
            }
            Ok(())
        }
        Some("missing-links") => {
            let options = parse_read_only_debug_args(&args[1..], "missing-links")?;
            let open = open_context(&options.path).ok();
            let missing = codefire_core::current_required_link_missing(&options.path).map_err(
                |error| match open.as_ref() {
                    Some(context) => CliError::CoreContext {
                        error,
                        repo_root: context.repo_root.clone(),
                        path: context.open_dir.join("codefire.links.yaml"),
                    },
                    None => CliError::Core(error),
                },
            )?;
            let repo_root = open.map(|context| context.repo_root);
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "missing-links",
                        true,
                        0,
                        repo_root.as_deref(),
                        json!({
                            "type": "codefire_missing_required_links",
                            "version": 1,
                            "missing_required_links": missing,
                        }),
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(&missing)?);
            }
            Ok(())
        }
        Some("context") => {
            let json_requested = args_want_json(&args[1..]);
            let options = match parse_context_args(&args[1..]) {
                Ok(options) => options,
                Err(error) if json_requested => {
                    print_cli_error_json("context", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let pack = match build_context_pack(&options) {
                Ok(pack) => pack,
                Err(error) if options.json_output => {
                    print_cli_error_json("context", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "context",
                        true,
                        0,
                        Some(&pack.repo_root),
                        pack.data,
                        Vec::new(),
                        pack.next_actions,
                    ))?
                );
            } else {
                print_context_summary(&pack.data);
            }
            Ok(())
        }
        Some("clone") => {
            let options = parse_clone_args(&args[1..])?;
            let result = clone_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                print_plan_result_json("clone", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "clone dry-run: {} would create {}",
                    options.source, options.new_branch
                );
            } else {
                println!("cloned {} -> {}", options.source, options.new_branch);
            }
            Ok(())
        }
        Some("close") => {
            let options = parse_close_args(&args[1..], false)?;
            let result = close_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                print_plan_result_json("close", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "close dry-run: {} would close {}",
                    result.branch,
                    result.open_dir.display()
                );
            } else {
                println!("closed {}: {}", result.branch, result.open_dir.display());
            }
            Ok(())
        }
        Some("discard") => {
            let options = parse_close_args(&args[1..], true)?;
            let result = close_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                print_plan_result_json("discard", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "discard dry-run: {} would close {}",
                    result.branch,
                    result.open_dir.display()
                );
            } else {
                println!("discarded {}: {}", result.branch, result.open_dir.display());
            }
            Ok(())
        }
        Some("upload") => {
            let options = parse_upload_args(&args[1..])?;
            let result = match upload_branch(&env::current_dir()?, &options) {
                Ok(result) => result,
                Err(error) if options.json_output && matches!(error, CliError::LockContention(_)) => {
                    print_lock_contention_json("upload", &error)?;
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                print_plan_result_json("upload", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "upload dry-run: {}@{} would update {}",
                    options.branch, result.head, options.remote_url
                );
            } else {
                println!(
                    "uploaded {}@{} -> {}",
                    options.branch, result.head, options.remote_url
                );
            }
            Ok(())
        }
        Some("list") => {
            let options = parse_list_args(&args[1..])?;
            match options.target {
                ListTarget::Local(path) => {
                    if options.page != PageRequest::default() {
                        return Err(CliError::Usage(
                            "--limit/--cursor currently require a remote list target".to_string(),
                        ));
                    }
                    list_branches(&path)?
                        .prepare_opened_registries()?
                        .output_local_list(options.json_output)?;
                }
                ListTarget::Remote(project_url) => {
                    let branches = match list_remote_branches(&project_url) {
                        Ok(branches) => branches,
                        Err(error) if options.json_output => {
                            print_cli_error_json("list", &error)?;
                            return Err(CliError::CommandFailed(error.exit_status()));
                        }
                        Err(error) => return Err(error),
                    };
                    let page = paginate(
                        &format!("remote-branches:{project_url}"),
                        branches
                            .iter()
                            .map(|branch| json!({"name": branch.name, "head": branch.head}))
                            .collect(),
                        &options.page,
                    )?;
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "list",
                                true,
                                0,
                                None,
                                remote_branch_list_data_json(&project_url, &page),
                                Vec::new(),
                                Vec::new(),
                            ))?
                        );
                    } else {
                        for branch in page.items {
                            println!(
                                "{}\t{}",
                                branch["name"].as_str().unwrap_or_default(),
                                branch["head"].as_str().unwrap_or_default()
                            );
                        }
                    }
                }
            }
            Ok(())
        }
        Some("request-merge") => {
            let options = parse_request_merge_args(&args[1..])?;
            let result = match request_merge(&options) {
                Ok(result) => result,
                Err(error) if options.json_output && matches!(error, CliError::LockContention(_)) => {
                    print_lock_contention_json("request-merge", &error)?;
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                print_plan_result_json("request-merge", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "request-merge dry-run: {} would request {}",
                    options.source_url, options.target_url
                );
            } else {
                println!("created {}", result.id);
                println!("source: {}", result.source_head);
                println!("target: {}", result.target_head);
            }
            Ok(())
        }
        Some("request-list") => {
            let options = parse_remote_project_args(&args[1..], "request-list")?;
            let requests = match list_merge_requests(&options.project_url) {
                Ok(requests) => requests,
                Err(error) if options.json_output => {
                    print_cli_error_json("request-list", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let page = paginate(
                &format!("merge-requests:{}", options.project_url),
                requests
                    .iter()
                    .map(|request| {
                        json!({
                            "id": request.id,
                            "status": request.status,
                            "source_url": request.source_url,
                            "target_url": request.target_url,
                        })
                    })
                    .collect(),
                &options.page,
            )?;
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "request-list",
                        true,
                        0,
                        None,
                        merge_request_list_data_json(&options.project_url, &page),
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                for request in page.items {
                    println!(
                        "{}\t{}\t{}\t{}",
                        request["id"].as_str().unwrap_or_default(),
                        request["status"].as_str().unwrap_or_default(),
                        request["source_url"].as_str().unwrap_or_default(),
                        request["target_url"].as_str().unwrap_or_default(),
                    );
                }
            }
            Ok(())
        }
        Some("request-review") => {
            let options = parse_request_review_args(&args[1..])?;
            let result = match review_merge_request(&options) {
                Ok(result) => result,
                Err(error) if options.json_output && matches!(error, CliError::LockContention(_)) => {
                    print_lock_contention_json("request-review", &error)?;
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                print_plan_result_json("request-review", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "request-review dry-run: {} would be {}d by {}",
                    options.mr_id, result.decision, result.reviewer
                );
            } else {
                println!(
                    "{}d {} by {}",
                    result.decision, options.mr_id, result.reviewer
                );
            }
            Ok(())
        }
        Some("request-apply") => {
            let options = parse_request_apply_args(&args[1..])?;
            let result = match apply_merge_request(&options) {
                Ok(result) => result,
                Err(error) if options.json_output && matches!(error, CliError::LockContention(_)) => {
                    print_lock_contention_json("request-apply", &error)?;
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                print_plan_result_json("request-apply", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "request-apply dry-run: {} would update {}@{}",
                    options.mr_id, result.target_branch, result.head
                );
            } else {
                println!("applied {}", options.mr_id);
                println!("target: {}@{}", result.target_branch, result.head);
            }
            Ok(())
        }
        Some("serve") => {
            let options = parse_serve_args(&args[1..])?;
            serve_http(&options)
        }
        Some("show") => {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help("show"));
                return Ok(());
            }
            let options = parse_show_args(&args[1..])?;
            let repo_root = optional_repo_root(&env::current_dir()?);
            if options.json_output {
                match show_commitish_data(repo_root.as_deref(), &options.target) {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string_pretty(&command_result_envelope(
                            "show",
                            true,
                            0,
                            repo_root.as_deref(),
                            data,
                            Vec::new(),
                            Vec::new(),
                        ))?
                    ),
                    Err(error) => {
                        print_cli_error_json("show", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                }
            } else {
                let output = show_commitish(repo_root.as_deref(), &options.target)?;
                print!("{output}");
            }
            Ok(())
        }
        Some("diff") => {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help("diff"));
                return Ok(());
            }
            let options = parse_diff_args(&args[1..])?;
            let repo_root = optional_repo_root(&env::current_dir()?);
            let output = match diff_commitish_with_options(
                repo_root.as_deref(),
                &options.left,
                &options.right,
                &options.diff,
            ) {
                Ok(output) => output,
                Err(error) if options.diff.json_output => {
                    print_cli_error_json("diff", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            if options.diff.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&diff_result_envelope(
                        repo_root.as_deref(),
                        &output,
                    )?)?
                );
            } else {
                print!("{output}");
            }
            Ok(())
        }
        Some("review-pack") => {
            let json_requested = args_want_json(&args[1..]);
            let options = match parse_review_pack_args(&args[1..]) {
                Ok(options) => options,
                Err(error) if json_requested => {
                    print_cli_error_json("review-pack", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let repo_root = optional_repo_root(&env::current_dir()?);
            let output = review_pack_with_options(repo_root.as_deref(), &options.review)?;
            if let Some(path) = options.output.as_ref() {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, &output)?;
            }
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "review-pack",
                        true,
                        0,
                        repo_root.as_deref(),
                        review_pack_result_data(options.output.as_ref(), &output)?,
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                print!("{output}");
            }
            Ok(())
        }
        Some("patch") => match args.get(1).map(String::as_str) {
            Some("export") => {
                let json_requested = args_want_json(&args[2..]);
                let options = match parse_patch_export_args(&args[2..]) {
                    Ok(options) => options,
                    Err(error) if json_requested => {
                        print_cli_error_json("patch-export", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                let repo_root = optional_repo_root(&env::current_dir()?);
                let output = patch_export_with_options(repo_root.as_deref(), &options.patch)?;
                if let Some(path) = options.output.as_ref() {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(path, &output)?;
                }
                if options.json_output {
                    let data = patch_export_result_data(options.output.as_ref(), &output)?;
                    let next_actions = data
                        .get("next_actions")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&command_result_envelope(
                            "patch-export",
                            true,
                            0,
                            repo_root.as_deref(),
                            data,
                            Vec::new(),
                            next_actions,
                        ))?
                    );
                } else {
                    print!("{output}");
                }
                Ok(())
            }
            Some("import") => {
                let options = parse_patch_import_args(&args[2..])?;
                let result = import_patch(&env::current_dir()?, &options)?;
                if options.json_output {
                    print_data_result_json("patch-import", result)?;
                } else if options.dry_run {
                    println!(
                        "patch dry-run: {} entries would apply to {}",
                        result["entries"].as_u64().unwrap_or(0),
                        result["branch"].as_str().unwrap_or("(unknown)")
                    );
                } else {
                    println!(
                        "applied patch: {} entries to {}",
                        result["entries"].as_u64().unwrap_or(0),
                        result["branch"].as_str().unwrap_or("(unknown)")
                    );
                }
                Ok(())
            }
            _ => Err(CliError::Usage(
                "usage: codefire patch export <source> [--base <base>] [--output <path>] | codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>]".to_string(),
            )),
        },
        Some("merge") => {
            if args.get(1).map(String::as_str) == Some("--continue") {
                let options = parse_merge_continue_args(&args[1..])?;
                let result = run_merge_continue(&options)?;
                let exit_code = result.exit_code();
                if options.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&command_result_envelope(
                            "merge",
                            !result.blocked,
                            exit_code.code(),
                            Some(&result.repo_root),
                            merge_continue_data_json(&result),
                            merge_continue_diagnostics_json(&result),
                            merge_continue_next_actions(&result),
                        ))?
                    );
                } else {
                    print!("{}", render_merge_continue(&result));
                }
                return if result.blocked {
                    Err(CliError::CommandFailed(exit_code))
                } else {
                    Ok(())
                };
            }
            let options = parse_merge_args(&args[1..])?;
            let result = merge_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                let plan = merge_result_json_value(&result);
                print_plan_result_json("merge", &plan)?;
            } else if options.dry_run {
                print!("{}", render_merge_dry_run(&result));
            } else if result.conflicts.is_empty() {
                println!(
                    "merged {} into {}; target is open-burning",
                    options.source_branch, options.target_branch
                );
            } else {
                println!(
                    "merged {} into {} with conflicts; target is open-burning",
                    options.source_branch, options.target_branch
                );
                for conflict in result.conflicts {
                    println!("  conflict: {conflict}");
                }
            }
            Ok(())
        }
        Some("init") => {
            let options = parse_init_args(&args[1..])?;
            let result = init_repo(&options.path, options.force)?;
            println!(
                "initialized CodeFire repository: {}",
                result.repo_root.display()
            );
            println!("main: {}", result.main_commit);
            Ok(())
        }
        Some("import") => {
            let options = parse_import_args(&args[1..])?;
            let result = import_existing_project(&options)?;
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "import",
                        true,
                        0,
                        Some(&result.repo_root),
                        import_result_data_json(&result),
                        Vec::new(),
                        result
                            .plan
                            .get("next_actions")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default(),
                    ))?
                );
            } else if options.dry_run {
                println!(
                    "import dry-run: {} files would be adopted at {} on branch {}",
                    result.initial_files,
                    result.open_dir.display(),
                    result.branch
                );
            } else {
                println!(
                    "imported {} as branch {} ({})",
                    result.open_dir.display(),
                    result.branch,
                    result.initial_state
                );
            }
            Ok(())
        }
        Some("open") => {
            let options = parse_open_args(&args[1..])?;
            let result = open_branch(&options)?;
            if options.json_output {
                print_plan_result_json("open", &result.plan)?;
            } else if options.dry_run {
                println!(
                    "open dry-run: {} would open at {}",
                    result.branch,
                    result.open_dir.display()
                );
            } else {
                println!("opened {}: {}", result.branch, result.open_dir.display());
            }
            Ok(())
        }
        Some("branch") => match args.get(1).map(String::as_str) {
            Some("-h") | Some("--help") => {
                print!("{}", subcommand_help("branch"));
                Ok(())
            }
            Some("list") => {
                if wants_help(&args[2..]) {
                    print!("{}", subcommand_help("branch list"));
                    return Ok(());
                }
                let options = parse_path_json_args(&args[2..], "branch list")?;
                let start = options.path;
                let started = Instant::now();
                let branch_list = list_branches(&start)?;
                let metrics = options.metrics.then(|| {
                    branch_list_metrics(
                        started.elapsed(),
                        &branch_list.branches,
                        branch_list.store_work,
                        branch_list.ref_work,
                        branch_list.decision_work_snapshot(),
                    )
                });
                branch_list.output_branch_list(options.json_output, metrics)?;
                Ok(())
            }
            Some("show") => {
                if wants_help(&args[2..]) {
                    print!("{}", subcommand_help("branch show"));
                    return Ok(());
                }
                let options = parse_branch_show_args(&args[2..])?;
                let (repo_root, data) = branch_show_data(&options.path, options.branch.as_deref())?;
                if options.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&command_result_envelope(
                            "branch-show",
                            true,
                            0,
                            Some(&repo_root),
                            data,
                            Vec::new(),
                            Vec::new(),
                        ))?
                    );
                } else {
                    print_branch_show(&data);
                }
                Ok(())
            }
            Some(command) => {
                let error = CliError::Usage(format!("unsupported branch command: {command}"));
                if args[1..].iter().any(|arg| arg == "--json") {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&branch_unsupported_command_envelope(
                            command,
                            branch_error_repo_root(&args[2..]).as_deref(),
                            error.exit_code(),
                        ))?
                    );
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error)
            }
            None => Err(CliError::Usage(
                "missing branch command; expected: list or show".to_string(),
            )),
        },
        Some("status") => {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help("status"));
                return Ok(());
            }
            let options = parse_path_json_args(&args[1..], "status")?;
            let started = Instant::now();
            let context = open_context_for_status(&options.path)?;
            let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
            let status = read_status_with_cache(&options.path, &mut validation_cache)?;
            let metrics = options
                .metrics
                .then(|| status_metrics(started.elapsed(), &status));
            if options.json_output {
                let data = attach_metrics(
                    status_data_json_with_prediction(
                        &status,
                        &options.path,
                        &mut validation_cache,
                    ),
                    metrics.as_ref(),
                );
                let next_actions =
                    status_next_actions_with_prediction(&status, &data, Some(&options.path));
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "status",
                        true,
                        0,
                        Some(&context.repo_root),
                        data,
                        Vec::new(),
                        next_actions,
                    ))?
                );
            } else {
                let data = status_data_json_with_prediction(
                    &status,
                    &options.path,
                    &mut validation_cache,
                );
                print_status(&status);
                if let Some(line) = unscanned_changes_line(&status, &data) {
                    println!("{line}");
                }
                if let Some(metrics) = metrics.as_ref() {
                    print_metrics(metrics);
                }
            }
            Ok(())
        }
        Some("storage") => match args.get(1).map(String::as_str) {
            Some("-h") | Some("--help") => {
                print!("{}", subcommand_help("storage"));
                Ok(())
            }
            Some("report") => {
                if wants_help(&args[2..]) {
                    print!("{}", subcommand_help("storage"));
                    return Ok(());
                }
                let options = match parse_storage_report_args(&args[2..]) {
                    Ok(options) => options,
                    Err(error) if args_want_json(&args[2..]) => {
                        print_cli_error_json("storage", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                run_storage_report_command(options)
            }
            Some(value) if value.starts_with("--") => {
                let options = match parse_storage_report_args(&args[1..]) {
                    Ok(options) => options,
                    Err(error) if args_want_json(&args[1..]) => {
                        print_cli_error_json("storage", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                run_storage_report_command(options)
            }
            Some(_) => {
                let options = match parse_storage_report_args(&args[1..]) {
                    Ok(options) => options,
                    Err(error) if args_want_json(&args[1..]) => {
                        print_cli_error_json("storage", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                run_storage_report_command(options)
            }
            None => {
                let options = parse_storage_report_args(&[])?;
                run_storage_report_command(options)
            }
        },
        Some("doctor") => {
            if wants_help(&args[1..]) {
                print!("{}", subcommand_help("doctor"));
                Ok(())
            } else {
                let options = match parse_doctor_args(&args[1..]) {
                    Ok(options) => options,
                    Err(error) if args_want_json(&args[1..]) => {
                        print_cli_error_json("doctor", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                let target = options.start.to_string_lossy();
                if let Some(report) = run_remote_doctor(&target, options.quick)? {
                    let exit_code = if report.ok {
                        ExitCode::Success
                    } else {
                        ExitCode::RepositoryCorruption
                    };
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "doctor",
                                report.ok,
                                exit_code.code(),
                                None,
                                remote_doctor_data_json(&report),
                                remote_doctor_diagnostics_json(&report),
                                Vec::new(),
                            ))?
                        );
                    } else {
                        print_remote_doctor_report(&report);
                    }
                    return if report.ok {
                        Ok(())
                    } else {
                        Err(CliError::CommandFailed(exit_code))
                    };
                }
                let report = run_doctor(&options)?;
                let exit_code = if report.is_ok() {
                    ExitCode::Success
                } else {
                    ExitCode::RepositoryCorruption
                };
                if options.json_output {
                    let ok = report.is_ok();
                    report.write_json(&mut std::io::stdout().lock())?;
                    if ok {
                        return Ok(());
                    }
                } else {
                    let ok = report.is_ok();
                    report.write_text(&mut std::io::stdout().lock())?;
                    if ok {
                        return Ok(());
                    }
                }
                if options.json_output {
                    // The detailed doctor envelope above owns JSON rendering. Return only the
                    // process-status sentinel so `main` does not append a second generic error
                    // envelope to stdout.
                    Err(CliError::CommandFailed(exit_code))
                } else {
                    Err(CliError::InvalidRepository(
                        "doctor found repository problems".to_string(),
                    ))
                }
            }
        }
        Some("link") => {
            let options = match parse_link_batch_args(&args[1..]) {
                Ok(options) => options,
                Err(error) if args_want_json(&args[1..]) => {
                    print_cli_error_json("link-batch", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let result = match run_link_batch(&options) {
                Ok(result) => result,
                Err(error) if options.json_output => {
                    print_cli_error_json("link-batch", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let valid_result = result.valid;
            let exit_code = if valid_result {
                ExitCode::Success
            } else {
                ExitCode::InvalidUsageOrConfig
            };
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "link-batch",
                        valid_result,
                        exit_code.code(),
                        Some(&result.repo_root),
                        link_batch_data_json(&result),
                        result.diagnostics.clone(),
                        Vec::new(),
                    ))?
                );
            } else if options.dry_run {
                if valid_result {
                    println!("link batch dry-run: {} links validated", result.item_count);
                } else {
                    println!(
                        "link batch dry-run validation failed: {} issue(s)",
                        result.diagnostics.len()
                    );
                }
            } else {
                println!(
                    "recorded link batch: {} links in {}",
                    result.item_count,
                    result.links_file.display()
                );
            }
            if valid_result {
                Ok(())
            } else {
                Err(CliError::CommandFailed(exit_code))
            }
        }
        Some("evidence") => match args.get(1).map(String::as_str) {
            Some("-h") | Some("--help") => {
                print!("{}", subcommand_help("evidence"));
                Ok(())
            }
            Some("add") => {
                if wants_help(&args[2..]) {
                    print!("{}", subcommand_help("evidence add"));
                    return Ok(());
                }
                let options = match parse_evidence_add_args(&args[2..]) {
                    Ok(options) => options,
                    Err(error) if args_want_json(&args[2..]) => {
                        print_cli_error_json("evidence-add", &error)?;
                        return Err(CliError::CommandFailed(error.exit_status()));
                    }
                    Err(error) => return Err(error),
                };
                if options.batch_path.is_some() {
                    let result = match run_evidence_batch(&options) {
                        Ok(result) => result,
                        Err(error) if options.json_output => {
                            print_cli_error_json("evidence-add-batch", &error)?;
                            return Err(CliError::CommandFailed(error.exit_status()));
                        }
                        Err(error) => return Err(error),
                    };
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "evidence-add-batch",
                                true,
                                0,
                                Some(&result.repo_root),
                                evidence_batch_data_json(&result),
                                Vec::new(),
                                Vec::new(),
                            ))?
                        );
                    } else if options.dry_run {
                        println!(
                            "evidence batch dry-run: {} items validated",
                            result.item_count
                        );
                    } else {
                        println!("recorded evidence batch: {} items", result.item_count);
                    }
                } else {
                    let result = match run_evidence_add(&options) {
                        Ok(result) => result,
                        Err(error) if options.json_output => {
                            print_cli_error_json("evidence-add", &error)?;
                            return Err(CliError::CommandFailed(error.exit_status()));
                        }
                        Err(error) => return Err(error),
                    };
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "evidence-add",
                                true,
                                0,
                                Some(&result.repo_root),
                                evidence_add_data_json(&result),
                                result.diagnostics.clone(),
                                Vec::new(),
                            ))?
                        );
                    } else {
                        print_evidence_add_result(&result);
                    }
                }
                Ok(())
            }
            Some(command) => Err(CliError::Usage(format!(
                "unsupported evidence command: {command}"
            ))),
            None => Err(CliError::Usage(
                "usage: codefire evidence add [--path <repo-or-open>] (--artifact <path>|--from-command <command>|--from-argv <program> [--argv <arg>...]|--batch <file>) [--label <text>] [--cwd <dir>] [--timeout <duration>] [--max-output-bytes <bytes>] [--dry-run] [--json]".to_string(),
            )),
        },
        Some("explain") => {
            let json_requested = args_want_json(&args[1..]);
            let options = match parse_explain_args(&args[1..]) {
                Ok(options) => options,
                Err(error) if json_requested => {
                    print_cli_error_json("explain", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let result = match run_explain(&options) {
                Ok(result) => result,
                Err(error) if options.json_output => {
                    print_cli_error_json("explain", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "explain",
                        true,
                        0,
                        Some(&result.repo_root),
                        result.data,
                        result.diagnostics,
                        result.next_actions,
                    ))?
                );
            } else {
                print_explain_result(&result);
            }
            Ok(())
        }
        Some("migrate") => {
            let options = match parse_migrate_args(&args[1..]) {
                Ok(options) => options,
                Err(error) if args_want_json(&args[1..]) => {
                    print_cli_error_json("migrate", &error)?;
                    return Err(CliError::CommandFailed(error.exit_status()));
                }
                Err(error) => return Err(error),
            };
            let report = run_migrate(&options)?;
            let exit_code = if report.compatible {
                ExitCode::Success
            } else {
                ExitCode::MigrationIncompatibility
            };
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "migrate",
                        report.compatible,
                        exit_code.code(),
                        Some(&report.repo_root),
                        migration_report_data_json(&report),
                        migration_report_diagnostics_json(&report),
                        migration_report_next_actions(&report),
                    ))?
                );
            } else {
                print_migration_report(&report);
            }
            if report.compatible {
                Ok(())
            } else if options.json_output {
                // The detailed migration envelope above owns JSON rendering. Return only the
                // process-status sentinel so `main` does not append a second generic error
                // envelope to stdout.
                Err(CliError::CommandFailed(exit_code))
            } else {
                Err(CliError::MigrationIncompatibility(
                    "migration compatibility check failed".to_string(),
                ))
            }
        }
        Some("--version") | Some("version") => {
            println!("codefire foundation {}", codefire_core::VERSION);
            Ok(())
        }
        Some(command) => Err(CliError::Usage(format!("unsupported command: {command}"))),
        None => {
            print!("{}", help_text());
            Ok(())
        }
    }
}

fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}

fn command_name_for_args(args: &[String]) -> &str {
    match args.first().map(String::as_str) {
        Some("branch") => "branch",
        Some("patch") => "patch",
        Some("evidence") => "evidence",
        Some(command) => command,
        None => "help",
    }
}

fn command_help_for_args(args: &[String]) -> Option<&'static str> {
    let command = args.first()?.as_str();
    if !wants_help(&args[1..]) {
        return None;
    }
    match command {
        "status" | "scan" | "verify" | "fire" | "extinguish" | "commit" | "init" | "import"
        | "open" | "clone" | "close" | "discard" | "merge" | "upload" | "list"
        | "request-merge" | "request-list" | "request-review" | "request-apply" | "review-pack"
        | "storage" | "doctor" | "show" | "diff" | "context" | "explain" | "atom-index"
        | "trace-graph" | "missing-links" | "link" | "capabilities" => {
            Some(subcommand_help(command))
        }
        "migrate" => match args.get(1).map(String::as_str) {
            Some("check") | Some("dry-run") | Some("apply") => {
                Some(subcommand_help("migrate check"))
            }
            _ => Some(subcommand_help("migrate")),
        },
        "branch" => match args.get(1).map(String::as_str) {
            Some("list") => Some(subcommand_help("branch list")),
            Some("show") => Some(subcommand_help("branch show")),
            _ => Some(subcommand_help("branch")),
        },
        "evidence" => match args.get(1).map(String::as_str) {
            Some("add") => Some(subcommand_help("evidence add")),
            _ => Some(subcommand_help("evidence")),
        },
        "patch" => match args.get(1).map(String::as_str) {
            Some("export") => Some(subcommand_help("patch export")),
            Some("import") => Some(subcommand_help("patch import")),
            _ => Some(subcommand_help("patch")),
        },
        _ => None,
    }
}

fn subcommand_help(command: &str) -> &'static str {
    match command {
        "init" => {
            "usage: codefire init [path] [--force]\n\nCreate a CodeFire repository without opening a branch.\n"
        }
        "import" => {
            "usage: codefire import [path] [--branch <name>] [--dry-run] [--json] [--idempotency-key <key>]\n\nAdopt an existing non-empty directory as a CodeFire open directory.\n"
        }
        "open" => {
            "usage: codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>]\n\nOpen a sealed branch into a working directory.\n"
        }
        "clone" => {
            "usage: codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a new sealed branch from an existing branch.\n"
        }
        "close" => {
            "usage: codefire close <branch> [--discard] [--keep-dir] [--dry-run] [--json]\n\nClose an open-clean branch and remove its open directory. Use --discard to explicitly discard unsealed open work.\n"
        }
        "discard" => {
            "usage: codefire discard <branch> [--keep-dir] [--dry-run] [--json]\n\nDiscard an open branch worktree and close its open registry entry.\n"
        }
        "merge" => {
            "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>]\n       codefire merge --continue [path|--path <open-dir>] [--accept-semantic-conflicts] [--dry-run] [--json]\n\nMerge a sealed source branch head into an open-clean target branch, then explicitly continue after resolving conflicts.\n"
        }
        "upload" => {
            "usage: codefire upload <branch> <remote-url> [--dry-run] [--json] [--idempotency-key <key>] [--request-key-id <key>]\n\nUpload a sealed branch to a remote project.\n"
        }
        "list" => {
            "usage: codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]\n\nList local repository branches/open worktrees or remote project branches. Pagination options require a remote target and bind the cursor to one immutable result snapshot.\n"
        }
        "request-merge" => {
            "usage: codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a remote merge request.\n"
        }
        "request-list" => {
            "usage: codefire request-list <remote-project-url> [--json] [--limit N] [--cursor CURSOR]\n\nList remote merge requests with a snapshot-bound cursor.\n"
        }
        "request-review" => {
            "usage: codefire request-review <remote-project-url> <mr-id> --decision approve|reject [--reviewer <name>] [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "request-apply" => {
            "usage: codefire request-apply <remote-project-url> <mr-id> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "status" => {
            "usage: codefire status [path|--path <path>] [--json] [--metrics]\n\nShow the current open branch state.\n"
        }
        "scan" => {
            "usage: codefire scan [path|--path <path>] [--json] [--metrics] [--full] [--wait-lock] [--lock-timeout <duration>]\n\nDetect changed atoms and open fires for an open directory.\n"
        }
        "verify" => {
            "usage: codefire verify [path|--path <path>] [--details] [--blocking-only] [--json] [--metrics] [--wait-lock] [--lock-timeout <duration>]\n\nRun CodeFire verification checks for an open directory.\n"
        }
        "fire" => {
            "usage: codefire fire <source-atom> --to <target-atom> --reason <text> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"from\":\"REQ-id\",\"to\":\"DES-id\",\"reason\":\"manual review\"}]}\n"
        }
        "extinguish" => {
            "usage: codefire extinguish <fire-id> [--path <open-dir>] [--refresh] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>) [--dry-run] [--json]\n       codefire extinguish --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire extinguish --batch-template [--path <open-dir>] [--resolution <type>] [--rationale <text>|--evidence <text>|--evidence-ref <id>] [--output <file>] [--json]\n\n--resolution is an extensible string, not a closed enum; omitting it defaults to addressed. Common examples: addressed, changed, no-change-required. Supply at least one of --rationale, --evidence, or --evidence-ref.\n--refresh re-records the resolution of an already extinguished fire against the current atoms, trace links, and policy; use it for each fire that verify reports as a stale resolution.\nJSON next_actions may include optional execution metadata (schema codefire.action_execution.v1): ready=true actions expose a complete argv for exec without a shell; ready=false actions require named required_inputs before they are executable. The command string is a compatibility projection of that argv or argv_template.\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"id\":\"FIRE-1\",\"resolution\":\"addressed\",\"rationale\":\"fixed\"}]}\n"
        }
        "commit" => {
            "usage: codefire commit [path|--path <open-dir>] -m <message> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "branch" => {
            "usage: codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]\n       codefire branch show [branch] [path|--path <repo-or-open>] [--json]\n\nInspect local branches.\n"
        }
        "branch list" => {
            "usage: codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]\n\nList local branches.\n"
        }
        "branch show" => {
            "usage: codefire branch show [branch] [path|--path <repo-or-open>] [--json]\n\nShow a local branch detail record.\n"
        }
        "context" => {
            "usage: codefire context (--changed|--atom <atom-id>|--fire <fire-id>|--branch <branch>) [--path <open-dir>] [--depth <n>] [--limit <n>] [--json]\n\nBuild a bounded context pack for an open directory.\n"
        }
        "explain" => {
            "usage: codefire explain (fire <fire-id>|atom <atom-id>|verify-failure|storage-warning) [--path <open-dir>] [--json]\n\nExplain CodeFire state, diagnostics, and next actions.\n"
        }
        "migrate" | "migrate check" => {
            "usage: codefire migrate check [path] [--json] [--target-format current|v0.6] [--quick|--full]\n       codefire migrate dry-run [path] [--json] [--target-format current|v0.6] [--quick|--full]\n       codefire migrate apply [path] [--json] [--target-format current|v0.6] [--full] [--wait-lock] [--lock-timeout <duration>]\n\nCheck, plan, or apply registered layout-only repository repairs. Target formats select a compatibility profile; migrate does not rewrite repository format metadata.\n"
        }
        "atom-index" => {
            "usage: codefire atom-index [path]\n\nPrint the current Atom index as JSON for a repository or open directory.\n"
        }
        "trace-graph" => {
            "usage: codefire trace-graph [path]\n\nPrint the current trace graph as JSON for a repository or open directory.\n"
        }
        "missing-links" => {
            "usage: codefire missing-links [path]\n\nPrint required trace links that are currently missing.\n"
        }
        "link" => {
            "usage: codefire link --batch <file> [--path <open-dir>] [--dry-run] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"links\":[{\"from\":\"REQ-id\",\"to\":\"DES-id\",\"type\":\"refined_by\"}]}\n"
        }
        "storage" => {
            "usage: codefire storage [path] [--quick|--full] [--json] [--large-threshold <bytes|KB|MB|GB>] [--remote <cf://server/org/app>]\n       codefire storage report [path] [--quick|--full] [--json] [--large-threshold <bytes|KB|MB|GB>] [--remote <cf://server/org/app>]\n"
        }
        "doctor" => {
            "usage: codefire doctor [path] [--quick|--full] [--json]\n\nInspect repository health.\n"
        }
        "evidence" | "evidence add" => {
            "usage: codefire evidence add [--path <repo-or-open>] (--artifact <path>|--from-command <command>|--from-argv <program> [--argv <arg>...]|--batch <file>) [--label <text>] [--cwd <dir>] [--timeout <duration>] [--max-output-bytes <bytes>] [--allow-failed-command] [--dry-run] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"items\":[{\"from_command\":\"cargo test --workspace\",\"label\":\"tests\"}]}\n"
        }
        "show" => {
            "usage: codefire show <branch-or-commit> [--json]\n\nShow a sealed commitish.\n"
        }
        "diff" => {
            "usage: codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <left> <right>\n"
        }
        "review-pack" => {
            "usage: codefire review-pack <source> [--base <base>] [--output <path>] [--algorithm myers|patience|histogram] [--no-rename-detection] [--json]\n"
        }
        "patch" => {
            "usage: codefire patch export <source> [--base <base>] [--output <path>]\n       codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "patch export" => {
            "usage: codefire patch export <source> [--base <base>] [--output <path>] [--max-file-bytes <bytes>] [--max-payload-bytes <bytes>] [--include-large-files] [--json]\n"
        }
        "patch import" => {
            "usage: codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>]\n"
        }
        "capabilities" => {
            "usage: codefire capabilities [--json]\n\nList CLI command automation capabilities.\n"
        }
        _ => "usage: codefire <command> [options]\n",
    }
}

fn run_storage_report_command(options: storage::StorageReportOptions) -> Result<(), CliError> {
    let report = run_storage_report(&options)?;
    if options.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&command_result_envelope(
                "storage-report",
                true,
                0,
                Some(&report.repo_root),
                storage_report_data_json(&report),
                storage_report_diagnostics_json(&report),
                storage_report_next_actions(&report),
            ))?
        );
    } else {
        print_storage_report(&report);
    }
    Ok(())
}

#[derive(Debug)]
struct OpenTargetResolutionError {
    message: String,
    reason: &'static str,
    repo_root: PathBuf,
    branch: String,
    requested_path: PathBuf,
    resolved_path: Option<PathBuf>,
    workspace_boundary: PathBuf,
}

#[derive(Debug)]
struct OpenRecoveryPendingError {
    operation: String,
    phase: String,
    repo_root: PathBuf,
    branch: String,
    open_dir: PathBuf,
}

#[derive(Debug)]
enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Core(codefire_core::CoreError),
    CoreContext {
        error: codefire_core::CoreError,
        repo_root: PathBuf,
        path: PathBuf,
    },
    Store(Box<codefire_store::StoreError>),
    StoreContext {
        error: Box<codefire_store::StoreError>,
        repo_root: PathBuf,
        path: PathBuf,
    },
    Usage(String),
    AuthenticationOrSignature(String),
    IdempotencyConflict(String),
    ResourceBudget(Box<codefire_core::DecisionWorkBudgetExceeded>),
    RequestDeadlineExceeded {
        phase: &'static str,
        limit_ms: u64,
    },
    RemoteDiagnostic {
        status: u16,
        exit_code: ExitCode,
        kind: String,
        message: String,
        resource_budget: Option<Box<codefire_core::DecisionWorkBudgetExceeded>>,
    },
    MigrationIncompatibility(String),
    VerificationFailed(ExitCode),
    CommandFailed(ExitCode),
    NotOpen(PathBuf),
    InvalidMarker(String),
    InvalidRepository(String),
    BranchReferenceSubjectChanged {
        subject: PathBuf,
        message: String,
    },
    OpenedRegistrySubjectChanged {
        subject: PathBuf,
        message: String,
    },
    OpenSnapshotDivergence(Box<open_snapshot::OpenSnapshotDivergence>),
    OpenRecoveryPending(Box<OpenRecoveryPendingError>),
    ActiveStateCapability(Box<active_state::ActiveStateError>),
    LockContention(String),
    RepositoryLockContention {
        message: String,
        repo_root: PathBuf,
        lock_path: PathBuf,
    },
    BranchOperation {
        message: String,
        repo_root: PathBuf,
        branch: Option<String>,
        path: Option<PathBuf>,
    },
    OpenTargetResolution(Box<OpenTargetResolutionError>),
    BatchFileRead {
        operation: &'static str,
        path: PathBuf,
        kind: ErrorKind,
        message: String,
    },
    EvidenceCommandFailed {
        exit_code: Option<i32>,
        timed_out: bool,
        cwd: String,
        message: String,
    },
    MissingEvidenceRef {
        evidence_id: String,
        repo_root: PathBuf,
    },
}

impl CliError {
    fn exit_status(&self) -> ExitCode {
        match self {
            CliError::Io(_) => ExitCode::GenericFailure,
            CliError::Json(_) => ExitCode::RepositoryCorruption,
            CliError::Core(error) => core_error_exit_code(error),
            CliError::CoreContext { error, .. } => core_error_exit_code(error),
            CliError::Store(error) => store_error_exit_code(error),
            CliError::StoreContext { error, .. } => store_error_exit_code(error),
            CliError::Usage(message) => usage_exit_code(message),
            CliError::AuthenticationOrSignature(_) => ExitCode::AuthenticationOrSignatureFailure,
            CliError::IdempotencyConflict(_) => ExitCode::IdempotencyConflict,
            CliError::ResourceBudget(_) => ExitCode::ResourceBudgetExceeded,
            CliError::RequestDeadlineExceeded { .. } => ExitCode::ResourceBudgetExceeded,
            CliError::RemoteDiagnostic { exit_code, .. } => *exit_code,
            CliError::MigrationIncompatibility(_) => ExitCode::MigrationIncompatibility,
            CliError::VerificationFailed(exit_code) => *exit_code,
            CliError::CommandFailed(exit_code) => *exit_code,
            CliError::NotOpen(_) => ExitCode::InvalidUsageOrConfig,
            CliError::InvalidMarker(_) | CliError::InvalidRepository(_) => {
                ExitCode::RepositoryCorruption
            }
            CliError::BranchReferenceSubjectChanged { .. }
            | CliError::OpenedRegistrySubjectChanged { .. } => ExitCode::RepositoryCorruption,
            CliError::OpenSnapshotDivergence(_) => ExitCode::RepositoryCorruption,
            CliError::OpenRecoveryPending(_) => ExitCode::RepositoryCorruption,
            CliError::ActiveStateCapability(_) => ExitCode::RepositoryCorruption,
            CliError::LockContention(_) | CliError::RepositoryLockContention { .. } => {
                ExitCode::LockContention
            }
            CliError::BranchOperation { message, .. } => usage_exit_code(message),
            CliError::OpenTargetResolution(error) => usage_exit_code(&error.message),
            CliError::BatchFileRead { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::EvidenceCommandFailed { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::MissingEvidenceRef { .. } => ExitCode::ObjectReferenceInvalid,
        }
    }

    fn exit_code(&self) -> i32 {
        self.exit_status().code()
    }

    fn diagnostic(&self) -> Value {
        match self {
            CliError::MissingEvidenceRef {
                evidence_id,
                repo_root,
            } => json!({
                "kind": "missing_evidence_ref",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "evidence_id": evidence_id,
                "repo": repo_root,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::Usage(message) if unknown_target_parts(message).is_some() => {
                let (kind, id) = unknown_target_parts(message).expect("checked above");
                json!({
                    "kind": "unknown_target",
                    "severity": "blocking",
                    "blocking": true,
                    "repairable": true,
                    "target": {
                        "kind": kind,
                        "id": id,
                    },
                    "exit_code": self.exit_code(),
                    "message": self.to_string(),
                })
            }
            CliError::Usage(message) if batch_schema_command(message).is_some() => json!({
                "kind": "batch_schema_error",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "command": batch_schema_command(message).expect("checked above"),
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::Usage(message) => json!({
                "kind": usage_diagnostic_kind(message),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::BatchFileRead {
                operation,
                path,
                kind,
                ..
            } => json!({
                "kind": batch_file_diagnostic_kind(*kind),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "operation": operation,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::EvidenceCommandFailed {
                exit_code,
                timed_out,
                cwd,
                ..
            } => json!({
                "kind": "evidence_command_failed",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "exit_code": self.exit_code(),
                "process_exit_code": exit_code,
                "timed_out": timed_out,
                "cwd": cwd,
                "message": self.to_string(),
            }),
            CliError::Io(_) => self.simple_diagnostic("io_error", true),
            CliError::Json(_) => self.simple_diagnostic("json_error", false),
            CliError::Core(error) => core_error_diagnostic(error, None, None, self.exit_code()),
            CliError::CoreContext {
                error,
                repo_root,
                path,
            } => core_error_diagnostic(error, Some(repo_root), Some(path), self.exit_code()),
            CliError::Store(error) => store_error_diagnostic(error, None, None, self.exit_code()),
            CliError::StoreContext {
                error,
                repo_root,
                path,
            } => store_error_diagnostic(error, Some(repo_root), Some(path), self.exit_code()),
            CliError::AuthenticationOrSignature(_) => {
                self.simple_diagnostic("authentication_or_signature_error", true)
            }
            CliError::IdempotencyConflict(_) => {
                self.simple_diagnostic("idempotency_conflict", true)
            }
            CliError::ResourceBudget(error) => {
                resource_budget_diagnostic(error, None, &self.to_string())
            }
            CliError::RequestDeadlineExceeded { phase, limit_ms } => json!({
                "kind": "request_deadline_exceeded",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "phase": phase,
                "limit_ms": limit_ms,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::RemoteDiagnostic {
                status,
                exit_code,
                kind,
                resource_budget,
                ..
            } => resource_budget.as_ref().map_or_else(
                || {
                    json!({
                        "kind": kind,
                        "severity": "blocking",
                        "blocking": true,
                        "repairable": true,
                        "http_status": status,
                        "exit_code": exit_code.code(),
                        "message": self.to_string(),
                    })
                },
                |budget| resource_budget_diagnostic(budget, Some(*status), &self.to_string()),
            ),
            CliError::MigrationIncompatibility(_) => {
                self.simple_diagnostic("migration_incompatibility", true)
            }
            CliError::VerificationFailed(_) => self.simple_diagnostic("verification_failed", true),
            CliError::CommandFailed(_) => self.simple_diagnostic("command_failed", true),
            CliError::NotOpen(path) => json!({
                "kind": "not_open",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::InvalidMarker(_) => self.simple_diagnostic("invalid_marker", true),
            CliError::InvalidRepository(_) => self.simple_diagnostic("invalid_repository", true),
            CliError::BranchReferenceSubjectChanged { subject, .. } => json!({
                "kind": "branch_reference_subject_changed",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "subject": subject,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenedRegistrySubjectChanged { subject, .. } => json!({
                "kind": "opened_registry_subject_changed",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "subject": subject,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenSnapshotDivergence(divergence) => json!({
                "kind": open_snapshot::OPEN_SNAPSHOT_DIVERGENCE_KIND,
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "phase": &divergence.phase,
                "authority": divergence.authority,
                "expected": &divergence.expected,
                "actual": &divergence.actual,
                "expected_record": &divergence.expected_record,
                "actual_record": &divergence.actual_record,
                "repo": &divergence.repo_root,
                "branch": &divergence.branch,
                "exit_code": self.exit_code(),
                "message": divergence.to_string(),
            }),
            CliError::OpenRecoveryPending(pending) => json!({
                "kind": "open_recovery_pending",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "operation": &pending.operation,
                "phase": &pending.phase,
                "repo": &pending.repo_root,
                "branch": &pending.branch,
                "open_dir": &pending.open_dir,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::ActiveStateCapability(error) => json!({
                "kind": "active_state_capability_invalid",
                "severity": "blocking",
                "blocking": true,
                "repairable": false,
                "operation": error.operation(),
                "path": error.path(),
                "exit_code": self.exit_code(),
                "message": error.to_string(),
            }),
            CliError::LockContention(_) => self.simple_diagnostic("lock_contention", true),
            CliError::RepositoryLockContention {
                repo_root,
                lock_path,
                ..
            } => json!({
                "kind": "lock_contention",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "repo": repo_root,
                "lock_path": lock_path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::BranchOperation {
                message,
                repo_root,
                branch,
                path,
            } => json!({
                "kind": usage_diagnostic_kind(message),
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "repo": repo_root,
                "branch": branch,
                "path": path,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
            CliError::OpenTargetResolution(error) => json!({
                "kind": "open_target_resolution_failed",
                "severity": "blocking",
                "blocking": true,
                "repairable": true,
                "reason": error.reason,
                "repo": error.repo_root,
                "branch": error.branch,
                "requested_path": error.requested_path,
                "resolved_path": error.resolved_path,
                "workspace_boundary": error.workspace_boundary,
                "exit_code": self.exit_code(),
                "message": self.to_string(),
            }),
        }
    }

    fn simple_diagnostic(&self, kind: &str, repairable: bool) -> Value {
        json!({
            "kind": kind,
            "severity": "blocking",
            "blocking": true,
            "repairable": repairable,
            "exit_code": self.exit_code(),
            "message": self.to_string(),
        })
    }
}

fn resource_budget_diagnostic(
    error: &codefire_core::DecisionWorkBudgetExceeded,
    http_status: Option<u16>,
    message: &str,
) -> Value {
    let mut diagnostic = serde_json::to_value(error).unwrap_or_else(|_| {
        json!({
            "kind": "resource_budget_exceeded",
            "exit_code": ExitCode::ResourceBudgetExceeded.code(),
        })
    });
    if let Some(fields) = diagnostic.as_object_mut() {
        fields.insert("severity".to_string(), json!("blocking"));
        fields.insert("blocking".to_string(), json!(true));
        fields.insert("repairable".to_string(), json!(true));
        fields.insert("message".to_string(), json!(message));
        if let Some(status) = http_status {
            fields.insert("http_status".to_string(), json!(status));
        }
    }
    diagnostic
}

fn core_error_diagnostic(
    error: &codefire_core::CoreError,
    repo_root: Option<&Path>,
    path: Option<&Path>,
    exit_code: i32,
) -> Value {
    match error {
        codefire_core::CoreError::TraceGraphEndpointIntegrity(error) => json!({
            "kind": "trace_graph_endpoint_integrity",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "total_issues": error.total_issues,
            "issues_omitted": error.omitted_issues,
            "issues": &error.issues,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_core::CoreError::SealedSnapshotSemanticIntegrity(error) => json!({
            "kind": "sealed_snapshot_semantic_integrity",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "total_issues": error.issues.len(),
            "issues": &error.issues,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        _ => json!({
            "kind": "core_error",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
    }
}

fn store_error_diagnostic(
    error: &codefire_store::StoreError,
    repo_root: Option<&Path>,
    path: Option<&Path>,
    exit_code: i32,
) -> Value {
    match error {
        codefire_store::StoreError::ObjectCollision {
            object_id,
            requested_type,
            existing_type,
            requested_hash,
            existing_hash,
            canonical_payload_matches,
        } => json!({
            "kind": "object_id_collision",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "object_id": object_id,
            "requested_type": requested_type,
            "existing_type": existing_type,
            "requested_hash": requested_hash,
            "existing_hash": existing_hash,
            "canonical_payload_matches": canonical_payload_matches,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_store::StoreError::InvalidSealedCommit(_) => json!({
            "kind": "sealed_commit_invalid",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        codefire_store::StoreError::ObjectNotFound(object_id) => json!({
            "kind": "object_not_found",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "object_id": object_id,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
        _ => json!({
            "kind": "store_error",
            "severity": "blocking",
            "blocking": true,
            "repairable": false,
            "repo": repo_root,
            "path": path,
            "exit_code": exit_code,
            "message": error.to_string(),
        }),
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Json(error) => write!(f, "{error}"),
            CliError::Core(error) => write!(f, "{error}"),
            CliError::CoreContext { error, .. } => write!(f, "{error}"),
            CliError::Store(error) => write!(f, "{error}"),
            CliError::StoreContext { error, .. } => write!(f, "{error}"),
            CliError::Usage(message) => write!(f, "{message}"),
            CliError::AuthenticationOrSignature(message) => write!(f, "{message}"),
            CliError::IdempotencyConflict(message) => write!(f, "{message}"),
            CliError::ResourceBudget(error) => write!(f, "{error}"),
            CliError::RequestDeadlineExceeded { phase, limit_ms } => write!(
                f,
                "HTTP request deadline of {limit_ms} ms exceeded during {phase}"
            ),
            CliError::RemoteDiagnostic { message, .. } => write!(f, "{message}"),
            CliError::MigrationIncompatibility(message) => write!(f, "{message}"),
            CliError::VerificationFailed(_) => write!(f, "verification failed"),
            CliError::CommandFailed(_) => write!(f, "command failed"),
            CliError::NotOpen(path) => write!(
                f,
                "not inside an open CodeFire branch directory: {}",
                path.display()
            ),
            CliError::InvalidMarker(message) => {
                write!(f, "invalid open directory marker: {message}")
            }
            CliError::InvalidRepository(message) => {
                write!(f, "invalid CodeFire repository: {message}")
            }
            CliError::BranchReferenceSubjectChanged { subject, message } => write!(
                f,
                "branch reference subject changed at {}: {message}",
                subject.display()
            ),
            CliError::OpenedRegistrySubjectChanged { subject, message } => write!(
                f,
                "opened registry subject changed at {}: {message}",
                subject.display()
            ),
            CliError::OpenSnapshotDivergence(divergence) => write!(f, "{divergence}"),
            CliError::OpenRecoveryPending(pending) => write!(
                f,
                "open recovery is pending for {} at phase {} on branch {}",
                pending.operation, pending.phase, pending.branch
            ),
            CliError::ActiveStateCapability(error) => write!(f, "{error}"),
            CliError::LockContention(message) => write!(f, "{message}"),
            CliError::RepositoryLockContention { message, .. } => write!(f, "{message}"),
            CliError::BranchOperation { message, .. } => write!(f, "{message}"),
            CliError::OpenTargetResolution(error) => write!(f, "{}", error.message),
            CliError::BatchFileRead {
                operation,
                path,
                message,
                ..
            } => write!(
                f,
                "{operation} file read failed: {}: {message}",
                path.display()
            ),
            CliError::EvidenceCommandFailed { message, .. } => write!(f, "{message}"),
            CliError::MissingEvidenceRef { evidence_id, .. } => {
                write!(f, "missing evidence object: {evidence_id}")
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::Json(error)
    }
}

impl From<codefire_core::CoreError> for CliError {
    fn from(error: codefire_core::CoreError) -> Self {
        CliError::Core(error)
    }
}

impl From<codefire_core::DecisionWorkBudgetExceeded> for CliError {
    fn from(error: codefire_core::DecisionWorkBudgetExceeded) -> Self {
        CliError::ResourceBudget(Box::new(error))
    }
}

impl From<codefire_store::StoreError> for CliError {
    fn from(error: codefire_store::StoreError) -> Self {
        CliError::Store(Box::new(error))
    }
}

impl From<active_state::ActiveStateError> for CliError {
    fn from(error: active_state::ActiveStateError) -> Self {
        CliError::ActiveStateCapability(Box::new(error))
    }
}

pub(crate) fn read_batch_file(path: &Path, operation: &'static str) -> Result<String, CliError> {
    fs::read_to_string(path).map_err(|error| CliError::BatchFileRead {
        operation,
        path: path.to_path_buf(),
        kind: error.kind(),
        message: error.to_string(),
    })
}

type CommandHandler = fn(&[String]) -> Result<(), CliError>;

fn local_workflow_handler(command: &str) -> Option<CommandHandler> {
    match command {
        "scan" => Some(run_scan_command),
        "verify" => Some(run_verify_command),
        "fire" => Some(run_fire_command),
        "extinguish" => Some(run_extinguish_command),
        "commit" => Some(run_commit_command),
        _ => None,
    }
}

fn run_scan_command(args: &[String]) -> Result<(), CliError> {
    let options = parse_path_json_args(args, "scan")?;
    let started = Instant::now();
    let scan = run_scan_with_lock_options(&options.path, &options.lock)?;
    let metrics = options
        .metrics
        .then(|| scan_metrics(started.elapsed(), &scan));
    if options.json_output {
        let repo_root = open_context(&options.path)
            .ok()
            .map(|context| context.repo_root);
        println!(
            "{}",
            serde_json::to_string_pretty(&command_result_envelope(
                "scan",
                true,
                0,
                repo_root.as_deref(),
                attach_metrics(
                    scan_data_json_with_full(&scan, options.full),
                    metrics.as_ref(),
                ),
                scan_diagnostics_json(&scan),
                scan_next_actions(&scan, Some(&options.path)),
            ))?
        );
    } else {
        print_scan(&scan);
        if let Some(metrics) = metrics.as_ref() {
            print_metrics(metrics);
        }
    }
    Ok(())
}

fn run_verify_command(args: &[String]) -> Result<(), CliError> {
    let options = parse_verify_args(args)?;
    let started = Instant::now();
    let execution = run_verify_with_lock_options(&options.path, &options.lock)?;
    let verification = execution.verification;
    let scan = execution.scan;
    let metrics = options
        .metrics
        .then(|| verification_metrics(started.elapsed(), &verification));
    let decision = codefire_core::committable_decision(&verification);
    let exit_code = verification_exit_code(&verification);
    if options.json_output {
        let repo_root = open_context(&options.path)
            .ok()
            .map(|context| context.repo_root);
        println!(
            "{}",
            serde_json::to_string_pretty(&command_result_envelope(
                "verify",
                decision.is_committable(),
                exit_code.code(),
                repo_root.as_deref(),
                attach_metrics(
                    verification_data_json_with_filter(&verification, options.diagnostic_filter()),
                    metrics.as_ref(),
                ),
                verification_diagnostics_json_with_filter(&verification, options.blocking_only),
                verification_next_actions(
                    &verification,
                    &scan,
                    options.blocking_only,
                    Some(&options.path)
                ),
            ))?
        );
    } else {
        print_verification(&verification, options.details, options.blocking_only);
        if let Some(metrics) = metrics.as_ref() {
            print_metrics(metrics);
        }
    }
    if decision.is_committable() {
        Ok(())
    } else {
        Err(CliError::VerificationFailed(exit_code))
    }
}

fn run_fire_command(args: &[String]) -> Result<(), CliError> {
    if args
        .iter()
        .any(|arg| arg == "--batch" || arg.starts_with("--batch="))
    {
        let options = match parse_fire_batch_args(args) {
            Ok(options) => options,
            Err(error) if args_want_json(args) => {
                print_cli_error_json("fire-batch", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        let result = match run_fire_batch(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("fire-batch", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&command_result_envelope(
                    "fire-batch",
                    true,
                    0,
                    Some(&result.repo_root),
                    fire_batch_data_json(&result),
                    Vec::new(),
                    Vec::new(),
                ))?
            );
        } else {
            print_fire_result(&result);
        }
    } else {
        let options = match parse_fire_args(args) {
            Ok(options) => options,
            Err(error) if args_want_json(args) => {
                print_cli_error_json("fire", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        let result = match run_fire(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("fire", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&command_result_envelope(
                    "fire",
                    true,
                    0,
                    Some(&result.repo_root),
                    fire_data_json(&result),
                    Vec::new(),
                    Vec::new(),
                ))?
            );
        } else {
            print_fire_result(&result);
        }
    }
    Ok(())
}

fn run_extinguish_command(args: &[String]) -> Result<(), CliError> {
    if has_interactive_extinguish_arg(args) {
        let options = parse_interactive_extinguish_args(args)?;
        let result = match run_interactive_extinguish(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("extinguish-interactive", &error)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            print_plan_result_json("extinguish-interactive", &result.plan)?;
        } else {
            print_interactive_extinguish_result(&result);
        }
    } else if has_all_matching_extinguish_arg(args) {
        let options = parse_all_matching_extinguish_args(args)?;
        let result = match run_all_matching_extinguish(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("extinguish-all", &error)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            print_plan_result_json("extinguish-all", &result.plan)?;
        } else {
            print_all_matching_extinguish_result(&options, &result);
        }
    } else if has_batch_template_arg(args) {
        let options = match parse_batch_template_args(args) {
            Ok(options) => options,
            Err(error) if args_want_json(args) => {
                print_cli_error_json("extinguish-batch-template", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        let result = match run_batch_template(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("extinguish-batch-template", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&command_result_envelope(
                    "extinguish-batch-template",
                    true,
                    0,
                    Some(&result.repo_root),
                    batch_template_data_json(&result),
                    Vec::new(),
                    result
                        .plan
                        .get("next_actions")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                ))?
            );
        } else {
            print_batch_template_result(&result);
        }
    } else if has_batch_extinguish_arg(args) {
        let options = match parse_extinguish_batch_args(args) {
            Ok(options) => options,
            Err(error) if args_want_json(args) => {
                print_cli_error_json("extinguish-batch", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        let result = match run_extinguish_batch(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("extinguish-batch", &error)?;
                return Err(CliError::CommandFailed(error.exit_status()));
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&command_result_envelope(
                    "extinguish-batch",
                    true,
                    0,
                    Some(&result.repo_root),
                    batch_extinguish_data_json(&result),
                    Vec::new(),
                    result
                        .plan
                        .get("next_actions")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                ))?
            );
        } else if options.dry_run {
            println!(
                "extinguish batch dry-run: {} fires would be processed",
                result.item_count
            );
            print_extinguish_batch_item_summary(&result.plan);
        } else {
            println!(
                "extinguished {} fires; remaining open fires: {}",
                result.item_count, result.remaining_open_fire_count
            );
            print_extinguish_batch_item_summary(&result.plan);
        }
    } else {
        let options = prepare_extinguish_options(parse_extinguish_args(args)?)?;
        let result = match run_extinguish(&options) {
            Ok(result) => result,
            Err(error) if options.json_output => {
                print_cli_error_json("extinguish", &error)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        if options.json_output {
            print_plan_result_json("extinguish", &result.plan)?;
        } else if options.dry_run {
            println!(
                "extinguish dry-run: {} ({}) {} -> {} would be {}",
                result.display_id,
                result.fire_uid,
                result.source_atom,
                result.target_atom,
                if options.refresh {
                    "refreshed"
                } else {
                    "extinguished"
                }
            );
        } else {
            println!(
                "{} {} ({}) {} -> {}; remaining open fires: {}",
                if options.refresh {
                    "refreshed"
                } else {
                    "extinguished"
                },
                result.display_id,
                result.fire_uid,
                result.source_atom,
                result.target_atom,
                result.remaining_open_fire_count
            );
        }
    }
    Ok(())
}

fn print_extinguish_batch_item_summary(plan: &Value) {
    let Some(items) = plan.get("items").and_then(Value::as_array) else {
        return;
    };
    for item in items.iter().take(20) {
        let display_id = item
            .get("display_id")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)");
        let source = item
            .get("source_atom")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)");
        let target = item
            .get("target_atom")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)");
        println!("- {display_id}: {source} -> {target}");
    }
    if let Some(omitted) = plan.get("items_omitted").and_then(Value::as_u64) {
        if omitted > 0 {
            println!("- ... {omitted} more omitted; rerun with --full for all items");
        }
    }
}

fn run_commit_command(args: &[String]) -> Result<(), CliError> {
    let options = match parse_commit_args(args) {
        Ok(options) => options,
        Err(error) if args_want_json(args) => {
            print_cli_error_json("commit", &error)?;
            return Err(CliError::CommandFailed(error.exit_status()));
        }
        Err(error) => return Err(error),
    };
    let result = match run_commit(&options) {
        Ok(result) => result,
        Err(error) if options.json_output => {
            print_cli_error_json("commit", &error)?;
            return Err(CliError::CommandFailed(error.exit_status()));
        }
        Err(error) => return Err(error),
    };
    if result.blocked {
        if options.json_output {
            print_commit_result_json(&result)?;
        } else {
            println!(
                "commit blocked: verification failed with exit code {}",
                result.exit_code.code()
            );
        }
        return Err(CliError::CommandFailed(result.exit_code));
    }
    if options.json_output {
        print_commit_result_json(&result)?;
    } else if options.dry_run {
        println!("commit dry-run: branch {} would be sealed", result.branch);
        println!(
            "Changed atoms: {}",
            result.plan["changed_atom_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Non-atom changed files: {}",
            result.plan["non_atom_changed_file_count"]
                .as_u64()
                .unwrap_or(0)
        );
        println!(
            "Extinguished fires: {}",
            result.plan["extinguished_fire_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Active resolutions: {}",
            result.plan["active_resolution_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Evidence refs: {}",
            result.plan["evidence_ref_count"].as_u64().unwrap_or(0)
        );
    } else {
        println!("Sealed commit created.");
        println!("Commit: {}", result.commit_id);
        println!("Branch: {}", result.branch);
        println!("Message: {}", result.plan["message"].as_str().unwrap_or(""));
        println!(
            "Changed atoms: {}",
            result.plan["changed_atom_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Non-atom changed files: {}",
            result.plan["non_atom_changed_file_count"]
                .as_u64()
                .unwrap_or(0)
        );
        println!(
            "Extinguished fires: {}",
            result.plan["extinguished_fire_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Active resolutions: {}",
            result.plan["active_resolution_count"].as_u64().unwrap_or(0)
        );
        println!(
            "Evidence refs: {}",
            result.plan["evidence_ref_count"].as_u64().unwrap_or(0)
        );
        println!("State: open-clean");
    }
    Ok(())
}

fn print_status(status: &Status) {
    println!("Branch: {}", status.branch);
    println!("State: {}", status.state);
    println!("Base: {}", status.base);
    println!("Open fires: {}", status.open_fires);
}

/// `status` reports the last recorded state. When the working files have
/// moved past it, say so and name the command that records the change.
fn unscanned_changes_line(status: &Status, data: &Value) -> Option<String> {
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

fn status_data_json_with_prediction(
    status: &Status,
    path: &Path,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Value {
    let mut data = status_data_json(status);
    if let Ok(scan_execution) = compute_scan_with_cache(path, None, validation_cache) {
        let scan = scan_execution.scan;
        data["scan_prediction"] = json!({
            "source": "preview_recomputed",
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

fn print_branch_show(data: &Value) {
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
fn branch_json(branch: &Branch) -> Value {
    json!({
        "name": &branch.name,
        "head": &branch.head,
        "state": &branch.state,
    })
}

fn is_remote_project_url(value: &str) -> bool {
    value.starts_with("cf://") || is_cf_http_url(value)
}

#[cfg(test)]
fn local_list_data_json(branches: &[Branch], opened: Vec<Value>) -> Value {
    json!({
        "type": "codefire_local_list",
        "version": 1,
        "branches": branches.iter().map(branch_json).collect::<Vec<_>>(),
        "opened": opened,
    })
}

fn remote_branch_list_data_json(project_url: &str, page: &pagination::Page) -> Value {
    json!({
        "type": "codefire_remote_branch_list",
        "version": 1,
        "project_url": project_url,
        "branches": &page.items,
        "page": &page.metadata,
    })
}

fn merge_request_list_data_json(project_url: &str, page: &pagination::Page) -> Value {
    json!({
        "type": "codefire_merge_request_list",
        "version": 1,
        "project_url": project_url,
        "merge_requests": &page.items,
        "page": &page.metadata,
    })
}

fn branch_show_data(start: &Path, branch: Option<&str>) -> Result<(PathBuf, Value), CliError> {
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

fn branch_unsupported_command_envelope(
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

fn branch_error_repo_root(args: &[String]) -> Option<PathBuf> {
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

fn print_scan(scan: &codefire_core::ScanResult) {
    print!("{}", render_scan(scan));
}

fn render_scan(scan: &codefire_core::ScanResult) -> String {
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
    output
}

fn print_lock_contention_json(command: &str, error: &CliError) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&lock_contention_envelope(command, error))?
    );
    Ok(())
}

fn print_plan_result_json(command: &str, plan: &Value) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&plan_result_envelope(command, plan))?
    );
    Ok(())
}

fn print_data_result_json(command: &str, data: Value) -> Result<(), CliError> {
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

fn print_commit_result_json(result: &CommitResult) -> Result<(), CliError> {
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

fn commit_result_data_json(result: &CommitResult) -> Value {
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

fn review_pack_result_data(
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

fn patch_export_result_data(
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

fn args_want_json(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn diff_result_envelope(repo_root: Option<&Path>, output: &str) -> Result<Value, CliError> {
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

fn print_cli_error_json(command: &str, error: &CliError) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&cli_error_envelope(command, error))?
    );
    Ok(())
}

fn cli_error_envelope(command: &str, error: &CliError) -> Value {
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

fn cli_error_diagnostic(error: &CliError) -> Value {
    error.diagnostic()
}

fn cli_error_next_actions(command: &str, error: &CliError) -> Vec<Value> {
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

fn branch_error_next_actions(
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

fn batch_file_diagnostic_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "batch_file_missing",
        ErrorKind::PermissionDenied => "batch_file_permission_denied",
        _ => "batch_file_read_failed",
    }
}

fn usage_diagnostic_kind(message: &str) -> &'static str {
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

fn batch_schema_command(message: &str) -> Option<&'static str> {
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

fn batch_operation_help_command(operation: &str) -> &'static str {
    match operation {
        "evidence batch" => "evidence add",
        "fire batch" => "fire",
        "link batch" => "link",
        "extinguish batch" => "extinguish",
        _ => "help",
    }
}

fn unsupported_option_command(message: &str) -> Option<&str> {
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

fn unknown_target_parts(message: &str) -> Option<(&'static str, &str)> {
    message
        .strip_prefix("unknown atom: ")
        .map(|id| ("atom", id))
        .or_else(|| {
            message
                .strip_prefix("unknown fire: ")
                .map(|id| ("fire", id))
        })
}

fn plan_result_envelope(command: &str, plan: &Value) -> Value {
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

fn plan_repo_root(plan: &Value) -> Option<PathBuf> {
    for key in ["repo_root", "repo"] {
        if let Some(path) = plan.get(key).and_then(Value::as_str) {
            return Some(PathBuf::from(path));
        }
    }
    plan.get("open_dir")
        .and_then(Value::as_str)
        .and_then(|path| find_repo_root(Path::new(path)).ok())
}

fn lock_contention_envelope(command: &str, error: &CliError) -> Value {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadOnlyDebugOptions {
    path: PathBuf,
    json_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShowOptions {
    target: String,
    json_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BranchShowOptions {
    path: PathBuf,
    branch: Option<String>,
    json_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListTarget {
    Local(PathBuf),
    Remote(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListOptions {
    target: ListTarget,
    json_output: bool,
    page: PageRequest,
}

fn parse_show_args(args: &[String]) -> Result<ShowOptions, CliError> {
    let mut target = None;
    let mut json_output = false;
    for value in args {
        match value.as_str() {
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported show option: {option}"
                )));
            }
            value if target.is_none() => target = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected show argument: {value}"
                )));
            }
        }
    }
    Ok(ShowOptions {
        target: target.ok_or_else(|| {
            CliError::Usage("usage: codefire show <branch-or-commit> [--json]".to_string())
        })?,
        json_output,
    })
}

fn parse_list_args(args: &[String]) -> Result<ListOptions, CliError> {
    let mut positional = Vec::new();
    let mut json_output = false;
    let mut page = PageRequest::default();
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--limit" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--limit requires a value".to_string()))?;
                page.limit = Some(parse_page_limit(raw)?);
            }
            value if value.starts_with("--limit=") => {
                page.limit = Some(parse_page_limit(value.trim_start_matches("--limit="))?);
            }
            "--cursor" => {
                index += 1;
                page.cursor = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--cursor requires a value".to_string()))?
                        .clone(),
                );
            }
            value if value.starts_with("--cursor=") => {
                page.cursor = Some(value.trim_start_matches("--cursor=").to_string());
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported list option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    let target = match positional.as_slice() {
        [] => ListTarget::Local(env::current_dir()?),
        [value] if is_remote_project_url(value) => ListTarget::Remote(value.clone()),
        [value] => ListTarget::Local(PathBuf::from(value)),
        _ => {
            return Err(CliError::Usage(
                "usage: codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]".to_string(),
            ));
        }
    };
    Ok(ListOptions {
        target,
        json_output,
        page,
    })
}

fn parse_branch_show_args(args: &[String]) -> Result<BranchShowOptions, CliError> {
    let mut path = None;
    let mut branch = None;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--path" => {
                index += 1;
                let path_value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                set_single_path(&mut path, path_value)?;
            }
            "--metrics" => {
                return Err(CliError::Usage(
                    "branch show does not support --metrics".to_string(),
                ));
            }
            value if value.starts_with("--path=") => {
                set_single_path(&mut path, value.trim_start_matches("--path="))?;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported branch show option: {option}"
                )));
            }
            value if branch.is_none() => branch = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected branch show argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(BranchShowOptions {
        path: path.unwrap_or(env::current_dir()?),
        branch,
        json_output,
    })
}

fn parse_read_only_debug_args(
    args: &[String],
    command: &str,
) -> Result<ReadOnlyDebugOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--path" => {
                index += 1;
                let path_value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                set_single_path(&mut path, path_value)?;
            }
            "--metrics" => {
                return Err(CliError::Usage(format!(
                    "{command} does not support --metrics"
                )));
            }
            value if value.starts_with("--path=") => {
                set_single_path(&mut path, value.trim_start_matches("--path="))?;
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {value}"
                )));
            }
            value => set_single_path(&mut path, value)?,
        }
        index += 1;
    }
    Ok(ReadOnlyDebugOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
    })
}

fn set_single_path(path: &mut Option<PathBuf>, value: &str) -> Result<(), CliError> {
    if path.is_some() {
        return Err(CliError::Usage(
            "path may only be specified once".to_string(),
        ));
    }
    *path = Some(PathBuf::from(value));
    Ok(())
}

fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
    let mut path = None;
    let mut force = false;
    for arg in args {
        if arg == "--force" {
            force = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err(CliError::Usage(format!("unexpected init argument: {arg}")));
        }
    }
    Ok(InitOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        force,
    })
}

fn parse_open_args(args: &[String]) -> Result<OpenOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported open option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [branch, path] => Ok(OpenOptions {
            branch: branch.to_string(),
            path: PathBuf::from(path),
            dry_run,
            json_output,
            lock,
            idempotency_key,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

fn parse_path_json_args(args: &[String], command: &str) -> Result<PathJsonOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut metrics = false;
    let mut full = false;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if command == "scan" && parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        if parse_path_json_metrics_option(arg, &mut json_output, &mut metrics) {
            if arg == "--metrics" && !supports_metrics(command) {
                return Err(CliError::Usage(format!(
                    "{command} does not support --metrics"
                )));
            }
            index += 1;
            continue;
        }
        match arg.as_str() {
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--full" if command == "scan" => {
                full = true;
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value if path.is_none() => {
                path = Some(PathBuf::from(value));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected {command} argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(PathJsonOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
        metrics,
        full,
        lock,
    })
}

fn parse_extinguish_args(args: &[String]) -> Result<ExtinguishOptions, CliError> {
    let mut path = None;
    let mut fire_id = None;
    let mut resolution = "addressed".to_string();
    let mut rationale = String::new();
    let mut evidence = String::new();
    let mut evidence_refs = Vec::new();
    let mut refresh = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut edit_rationale = false;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--resolution" => {
                index += 1;
                resolution = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--resolution requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--resolution=") => {
                resolution = value.trim_start_matches("--resolution=").to_string();
            }
            "--rationale" => {
                index += 1;
                rationale = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--rationale requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--rationale=") => {
                rationale = value.trim_start_matches("--rationale=").to_string();
            }
            "--evidence" => {
                index += 1;
                evidence = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--evidence requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--evidence=") => {
                evidence = value.trim_start_matches("--evidence=").to_string();
            }
            "--evidence-ref" => {
                index += 1;
                evidence_refs.push(
                    args.get(index)
                        .ok_or_else(|| {
                            CliError::Usage("--evidence-ref requires a value".to_string())
                        })?
                        .to_string(),
                );
            }
            "--refresh" => refresh = true,
            "--edit-rationale" => edit_rationale = true,
            value if value.starts_with("--evidence-ref=") => {
                evidence_refs.push(value.trim_start_matches("--evidence-ref=").to_string());
            }
            value if fire_id.is_none() => fire_id = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected extinguish argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(ExtinguishOptions {
        path: path.unwrap_or(env::current_dir()?),
        fire_id: fire_id.ok_or_else(|| {
            CliError::Usage("usage: codefire extinguish <fire-id> [--path <open-dir>] [--refresh] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>)".to_string())
        })?,
        resolution,
        rationale,
        evidence,
        evidence_refs,
        refresh,
        dry_run,
        json_output,
        lock,
        idempotency_key,
        edit_rationale,
    })
}

fn parse_commit_args(args: &[String]) -> Result<CommitOptions, CliError> {
    let mut path = None;
    let mut message = None;
    let mut dry_run = false;
    let mut full_output = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut signer = None;
    let mut key_id = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--path" => {
                path = Some(PathBuf::from(take_option_value(
                    args, &mut index, "--path",
                )?));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            "--full" => {
                full_output = true;
            }
            "-m" | "--message" => {
                message = Some(take_option_value(args, &mut index, "-m")?);
            }
            "--signer" => {
                signer = Some(take_option_value(args, &mut index, "--signer")?);
            }
            value if value.starts_with("--signer=") => {
                signer = Some(value.trim_start_matches("--signer=").to_string());
            }
            "--key-id" => {
                key_id = Some(take_option_value(args, &mut index, "--key-id")?);
            }
            value if value.starts_with("--key-id=") => {
                key_id = Some(value.trim_start_matches("--key-id=").to_string());
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected commit argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(CommitOptions {
        path: path.unwrap_or(env::current_dir()?),
        message: message.unwrap_or_else(|| "CodeFire commit".to_string()),
        dry_run,
        full_output,
        json_output,
        lock,
        idempotency_key,
        signer,
        key_id,
    })
}

fn parse_clone_args(args: &[String]) -> Result<CloneOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported clone option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [source, new_branch] => Ok(CloneOptions {
            source: source.to_string(),
            new_branch: new_branch.to_string(),
            dry_run,
            json_output,
            lock,
            idempotency_key,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

fn parse_close_args(args: &[String], discard_command: bool) -> Result<CloseOptions, CliError> {
    let mut branch = None;
    let mut discard = discard_command;
    let mut keep_dir = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--discard" if !discard_command => discard = true,
            "--discard" => {
                return Err(CliError::Usage(
                    "discard command does not accept --discard".to_string(),
                ));
            }
            "--keep-dir" => keep_dir = true,
            option if option.starts_with("--") => {
                let command = if discard_command { "discard" } else { "close" };
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value if branch.is_none() => branch = Some(value.to_string()),
            value => {
                let command = if discard_command { "discard" } else { "close" };
                return Err(CliError::Usage(format!(
                    "unexpected {command} argument: {value}"
                )));
            }
        }
        index += 1;
    }
    if idempotency_key.is_some() {
        let command = if discard_command { "discard" } else { "close" };
        return Err(CliError::Usage(format!(
            "{command} does not support --idempotency-key"
        )));
    }
    let usage = if discard_command {
        "usage: codefire discard <branch> [--keep-dir] [--dry-run] [--json]"
    } else {
        "usage: codefire close <branch> [--discard] [--keep-dir] [--dry-run] [--json]"
    };
    Ok(CloseOptions {
        branch: branch.ok_or_else(|| CliError::Usage(usage.to_string()))?,
        discard,
        keep_dir,
        dry_run,
        json_output,
        lock,
    })
}

fn parse_diff_args(args: &[String]) -> Result<DiffArgs, CliError> {
    let mut positional = Vec::new();
    let mut algorithm = DiffAlgorithm::Myers;
    let mut context_lines = DiffOptions::default().context_lines;
    let mut rename_detection = false;
    let mut atom_diff = false;
    let mut trace_diff = false;
    let mut impact_diff = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--algorithm" {
            index += 1;
            let name = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--algorithm requires a value".to_string()))?;
            algorithm = parse_diff_algorithm(name)?;
        } else if let Some(name) = value.strip_prefix("--algorithm=") {
            algorithm = parse_diff_algorithm(name)?;
        } else if value == "--context" {
            index += 1;
            let raw = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--context requires a value".to_string()))?;
            context_lines = parse_diff_context(raw)?;
        } else if let Some(raw) = value.strip_prefix("--context=") {
            context_lines = parse_diff_context(raw)?;
        } else if value == "--rename-detection" {
            rename_detection = true;
        } else if value == "--atoms" {
            atom_diff = true;
        } else if value == "--trace" {
            trace_diff = true;
        } else if value == "--impact" {
            impact_diff = true;
        } else if value == "--json" {
            json_output = true;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!("unsupported diff option: {value}")));
        } else {
            positional.push(value.clone());
        }
        index += 1;
    }
    match positional.as_slice() {
        [left, right] => Ok(DiffArgs {
            left: left.clone(),
            right: right.clone(),
            diff: DiffOptions {
                algorithm,
                context_lines,
                rename_detection,
                atom_diff,
                trace_diff,
                impact_diff,
                json_output,
            },
        }),
        _ => Err(CliError::Usage(
            "usage: codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <left> <right>".to_string(),
        )),
    }
}

fn parse_diff_context(value: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::Usage("--context requires a non-negative integer".to_string()))
}

fn parse_diff_algorithm(value: &str) -> Result<DiffAlgorithm, CliError> {
    DiffAlgorithm::parse(value).ok_or_else(|| {
        CliError::Usage("--algorithm must be one of: myers, patience, histogram".to_string())
    })
}

fn parse_review_pack_args(args: &[String]) -> Result<ReviewPackArgs, CliError> {
    let mut source = None;
    let mut base = None;
    let mut output = None;
    let mut json_output = false;
    let mut algorithm = DiffAlgorithm::Myers;
    let mut rename_detection = true;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--base" {
            index += 1;
            base = Some(
                args.get(index)
                    .ok_or_else(|| CliError::Usage("--base requires a value".to_string()))?
                    .to_string(),
            );
        } else if value == "--output" {
            index += 1;
            output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                CliError::Usage("--output requires a value".to_string())
            })?));
        } else if value == "--json" {
            json_output = true;
        } else if value == "--algorithm" {
            index += 1;
            let name = args
                .get(index)
                .ok_or_else(|| CliError::Usage("--algorithm requires a value".to_string()))?;
            algorithm = parse_diff_algorithm(name)?;
        } else if let Some(name) = value.strip_prefix("--algorithm=") {
            algorithm = parse_diff_algorithm(name)?;
        } else if value == "--rename-detection" {
            rename_detection = true;
        } else if value == "--no-rename-detection" {
            rename_detection = false;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!(
                "unsupported review-pack option: {value}"
            )));
        } else if source.is_none() {
            source = Some(value.clone());
        } else {
            return Err(CliError::Usage(format!(
                "unexpected review-pack argument: {value}"
            )));
        }
        index += 1;
    }
    Ok(ReviewPackArgs {
        review: ReviewPackOptions {
            source: source.ok_or_else(|| {
                CliError::Usage(
                    "usage: codefire review-pack <source> [--base <base>] [--output <path>] [--algorithm myers|patience|histogram] [--no-rename-detection]"
                        .to_string(),
                )
            })?,
            base,
            algorithm,
            rename_detection,
        },
        output,
        json_output,
    })
}

fn parse_patch_export_args(args: &[String]) -> Result<PatchExportArgs, CliError> {
    let mut source = None;
    let mut base = None;
    let mut output = None;
    let mut json_output = false;
    let mut max_file_bytes = DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES;
    let mut max_payload_bytes = DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES;
    let mut include_large_files = false;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value == "--base" {
            index += 1;
            base = Some(
                args.get(index)
                    .ok_or_else(|| CliError::Usage("--base requires a value".to_string()))?
                    .to_string(),
            );
        } else if value == "--output" {
            index += 1;
            output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                CliError::Usage("--output requires a value".to_string())
            })?));
        } else if value == "--json" {
            json_output = true;
        } else if value == "--include-large-files" {
            include_large_files = true;
        } else if value == "--max-file-bytes" {
            index += 1;
            max_file_bytes = parse_usize_arg(
                args.get(index).ok_or_else(|| {
                    CliError::Usage("--max-file-bytes requires a value".to_string())
                })?,
                "--max-file-bytes",
            )?;
        } else if let Some(bytes) = value.strip_prefix("--max-file-bytes=") {
            max_file_bytes = parse_usize_arg(bytes, "--max-file-bytes")?;
        } else if value == "--max-payload-bytes" {
            index += 1;
            max_payload_bytes = parse_usize_arg(
                args.get(index).ok_or_else(|| {
                    CliError::Usage("--max-payload-bytes requires a value".to_string())
                })?,
                "--max-payload-bytes",
            )?;
        } else if let Some(bytes) = value.strip_prefix("--max-payload-bytes=") {
            max_payload_bytes = parse_usize_arg(bytes, "--max-payload-bytes")?;
        } else if value.starts_with("--") {
            return Err(CliError::Usage(format!(
                "unsupported patch export option: {value}"
            )));
        } else if source.is_none() {
            source = Some(value.clone());
        } else {
            return Err(CliError::Usage(format!(
                "unexpected patch export argument: {value}"
            )));
        }
        index += 1;
    }
    Ok(PatchExportArgs {
        patch: PatchExportOptions {
            source: source.ok_or_else(|| {
                CliError::Usage(
                    "usage: codefire patch export <source> [--base <base>] [--output <path>]"
                        .to_string(),
                )
            })?,
            base,
            max_file_bytes,
            max_payload_bytes,
            include_large_files,
        },
        output,
        json_output,
    })
}

fn parse_usize_arg(value: &str, name: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|_| CliError::Usage(format!("{name} must be a non-negative integer")))
}

fn parse_patch_import_args(args: &[String]) -> Result<PatchImportOptions, CliError> {
    let mut path = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match value.as_str() {
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported patch import option: {value}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected patch import argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(PatchImportOptions {
        path: path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire patch import <patch-file> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}

fn parse_upload_args(args: &[String]) -> Result<UploadOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [branch, remote_url] => Ok(UploadOptions {
            branch: branch.clone(),
            remote_url: remote_url.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire upload <branch> <cf-url> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

fn parse_remote_project_args(
    args: &[String],
    command: &str,
) -> Result<RemoteProjectOptions, CliError> {
    let mut positional = Vec::new();
    let mut json_output = false;
    let mut page = PageRequest::default();
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        match value.as_str() {
            "--json" => json_output = true,
            "--limit" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--limit requires a value".to_string()))?;
                page.limit = Some(parse_page_limit(raw)?);
            }
            value if value.starts_with("--limit=") => {
                page.limit = Some(parse_page_limit(value.trim_start_matches("--limit="))?);
            }
            "--cursor" => {
                index += 1;
                page.cursor = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--cursor requires a value".to_string()))?
                        .clone(),
                );
            }
            value if value.starts_with("--cursor=") => {
                page.cursor = Some(value.trim_start_matches("--cursor=").to_string());
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [project_url] => Ok(RemoteProjectOptions {
            project_url: project_url.clone(),
            json_output,
            page,
        }),
        _ => Err(CliError::Usage(format!(
            "usage: codefire {command} <cf-project-url> [--json] [--limit N] [--cursor CURSOR]"
        ))),
    }
}

fn parse_request_merge_args(args: &[String]) -> Result<RequestMergeOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [source_url, target_url] => Ok(RequestMergeOptions {
            source_url: source_url.clone(),
            target_url: target_url.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

fn parse_request_review_args(args: &[String]) -> Result<RequestReviewOptions, CliError> {
    let mut positional = Vec::new();
    let mut reviewer = None;
    let mut decision = "approve".to_string();
    let mut comment = String::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            "--reviewer" => {
                index += 1;
                reviewer = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("--reviewer requires a value".to_string()))?
                        .to_string(),
                );
            }
            "--decision" => {
                index += 1;
                decision = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--decision requires a value".to_string()))?
                    .to_string();
            }
            "--comment" => {
                index += 1;
                comment = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--comment requires a value".to_string()))?
                    .to_string();
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    if decision != "approve" && decision != "reject" {
        return Err(CliError::Usage(
            "--decision must be approve or reject".to_string(),
        ));
    }
    match positional.as_slice() {
        [project_url, mr_id] => Ok(RequestReviewOptions {
            project_url: project_url.clone(),
            mr_id: mr_id.clone(),
            reviewer: reviewer.unwrap_or_else(|| "reviewer".to_string()),
            decision,
            comment,
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-review <project-url> <mr-id> [--reviewer <name>] [--decision approve|reject] [--comment <text>] [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]".to_string(),
        )),
    }
}

fn parse_request_apply_args(args: &[String]) -> Result<RequestApplyOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut idempotency_key = None;
    let mut request_key_id = None;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--request-key-id" => {
                request_key_id = Some(take_option_value(args, &mut index, "--request-key-id")?);
            }
            value if value.starts_with("--request-key-id=") => {
                request_key_id = Some(value.trim_start_matches("--request-key-id=").to_string());
            }
            value if value.starts_with("--") => skip_ignored_option(args, &mut index)?,
            value => positional.push(value.to_string()),
        }
        index += 1;
    }
    match positional.as_slice() {
        [project_url, mr_id] => Ok(RequestApplyOptions {
            project_url: project_url.clone(),
            mr_id: mr_id.clone(),
            dry_run,
            json_output,
            idempotency_key,
            request_key_id,
            lock,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire request-apply <project-url> <mr-id> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                .to_string(),
        )),
    }
}

fn parse_serve_args(args: &[String]) -> Result<ServeOptions, CliError> {
    let mut storage_root = None;
    let mut host = "127.0.0.1".to_string();
    let mut port = 8080u16;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut tls_client_ca = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                index += 1;
                host = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--host requires a value".to_string()))?
                    .to_string();
            }
            "--port" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--port requires a value".to_string()))?;
                port = value.parse::<u16>().map_err(|_| {
                    CliError::Usage("--port must be an integer from 0 to 65535".to_string())
                })?;
            }
            "--tls-cert" => {
                index += 1;
                tls_cert = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-cert requires a value".to_string())
                })?));
            }
            "--tls-key" => {
                index += 1;
                tls_key = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-key requires a value".to_string())
                })?));
            }
            "--tls-client-ca" => {
                index += 1;
                tls_client_ca = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--tls-client-ca requires a value".to_string())
                })?));
            }
            value if storage_root.is_none() => storage_root = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected serve argument: {value}"
                )))
            }
        }
        index += 1;
    }
    if tls_cert.is_some() != tls_key.is_some() {
        return Err(CliError::Usage(
            "--tls-cert and --tls-key must be provided together".to_string(),
        ));
    }
    if tls_client_ca.is_some() && tls_cert.is_none() {
        return Err(CliError::Usage(
            "--tls-client-ca requires --tls-cert and --tls-key".to_string(),
        ));
    }
    Ok(ServeOptions {
        storage_root: storage_root.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>] [--tls-client-ca <ca-pem>]"
                    .to_string(),
            )
        })?,
        host,
        port,
        tls_cert,
        tls_key,
        tls_client_ca,
    })
}

fn skip_ignored_option(args: &[String], index: &mut usize) -> Result<(), CliError> {
    match args[*index].as_str() {
        "--actor" | "--token" | "--request-key-id" | "--request-key" => {
            *index += 1;
            if args.get(*index).is_none() {
                return Err(CliError::Usage(format!(
                    "{} requires a value",
                    args[*index - 1]
                )));
            }
            Ok(())
        }
        "--dry-run" => Ok(()),
        option => Err(CliError::Usage(format!("unsupported option: {option}"))),
    }
}

fn parse_common_mutation_option(
    args: &[String],
    index: &mut usize,
    dry_run: &mut bool,
    json_output: &mut bool,
    lock: &mut LockOptions,
    idempotency_key: &mut Option<String>,
) -> Result<bool, CliError> {
    if parse_lock_option(args, index, lock)? {
        return Ok(true);
    }
    match args[*index].as_str() {
        "--dry-run" => {
            *dry_run = true;
            Ok(true)
        }
        "--json" => {
            *json_output = true;
            Ok(true)
        }
        "--idempotency-key" => {
            *idempotency_key = Some(take_option_value(args, index, "--idempotency-key")?);
            Ok(true)
        }
        value if value.starts_with("--idempotency-key=") => {
            *idempotency_key = Some(value.trim_start_matches("--idempotency-key=").to_string());
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn parse_path_json_metrics_option(arg: &str, json_output: &mut bool, metrics: &mut bool) -> bool {
    match arg {
        "--json" => {
            *json_output = true;
            true
        }
        "--metrics" => {
            *metrics = true;
            true
        }
        _ => false,
    }
}

fn take_option_value(args: &[String], index: &mut usize, option: &str) -> Result<String, CliError> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn parse_lock_option(
    args: &[String],
    index: &mut usize,
    lock: &mut LockOptions,
) -> Result<bool, CliError> {
    match args[*index].as_str() {
        "--wait-lock" => {
            lock.wait = true;
            Ok(true)
        }
        "--lock-timeout" => {
            *index += 1;
            let value = args
                .get(*index)
                .ok_or_else(|| CliError::Usage("--lock-timeout requires a value".to_string()))?;
            lock.timeout_ms = Some(parse_lock_timeout_ms(value)?);
            lock.wait = true;
            Ok(true)
        }
        value if value.starts_with("--lock-timeout=") => {
            lock.timeout_ms = Some(parse_lock_timeout_ms(
                value.trim_start_matches("--lock-timeout="),
            )?);
            lock.wait = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn parse_lock_timeout_ms(value: &str) -> Result<u64, CliError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CliError::Usage(
            "--lock-timeout requires a duration".to_string(),
        ));
    }
    if let Some(ms) = value.strip_suffix("ms") {
        return parse_u64_duration(ms, value);
    }
    if let Some(seconds) = value.strip_suffix('s') {
        return parse_u64_duration(seconds, value).map(|seconds| seconds.saturating_mul(1000));
    }
    parse_u64_duration(value, value).map(|seconds| seconds.saturating_mul(1000))
}

fn parse_u64_duration(raw: &str, display: &str) -> Result<u64, CliError> {
    raw.parse::<u64>().map_err(|_| {
        CliError::Usage(format!(
            "invalid --lock-timeout duration: {display}; use <seconds>, <seconds>s, or <milliseconds>ms"
        ))
    })
}

fn parse_merge_args(args: &[String]) -> Result<MergeOptions, CliError> {
    let mut source_branch = None;
    let mut target_branch = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut idempotency_key = None;
    let mut index = 0usize;
    while index < args.len() {
        if parse_common_mutation_option(
            args,
            &mut index,
            &mut dry_run,
            &mut json_output,
            &mut lock,
            &mut idempotency_key,
        )? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--into" => {
                target_branch = Some(take_option_value(args, &mut index, "--into")?);
            }
            value if source_branch.is_none() => source_branch = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected merge argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(MergeOptions {
        source_branch: source_branch.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                    .to_string(),
            )
        })?,
        target_branch: target_branch.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire merge <source-branch> --into <target-branch> [--dry-run] [--json] [--idempotency-key <key>] [--wait-lock] [--lock-timeout <duration>]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}

fn init_repo(path: &Path, force: bool) -> Result<InitResult, CliError> {
    init_repo_with_metadata(path, force, None)
}

fn init_repo_with_metadata(
    path: &Path,
    force: bool,
    initialization: Option<&Value>,
) -> Result<InitResult, CliError> {
    let repo_root = absolute_path(path)?;
    let cf = repo_root.join(".codefire");
    if cf.exists() && !force {
        return Err(CliError::Usage(".codefire already exists".to_string()));
    }

    ensure_repo_layout(&repo_root)?;
    let now = now_iso_utc();
    let mut repo = json!({
        "version": 1,
        "repository_id": stable_repo_id(&repo_root),
        "created_at": now,
    });
    if let Some(initialization) = initialization {
        repo["initialization"] = initialization.clone();
    }
    write_json_atomic(&cf.join("repo.json"), &repo)?;

    let objects = cf.join("objects");
    let (initial_effective_policy, initial_change_coverage) = initial_decision_contracts()?;
    let roots = initial_roots(
        &objects,
        &now,
        &initial_effective_policy,
        &initial_change_coverage,
    )?;
    let commit_payload = json!({
        "type": "commit",
        "version": 1,
        "root_schema_version": 2,
        "semantic_validation_profile": codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE,
        "parents": [],
        "message": "Initial empty CodeFire repository",
        "roots": roots,
        "certificate": {
            "result": "consistent",
            "open_required_fires": 0,
            "failed_checks": 0,
            "missing_required_links": 0,
            "stale_resolutions": 0,
            "missing_evidence_refs": 0,
            "duplicate_atom_ids": 0,
            "effective_policy_digest": &initial_effective_policy.digest,
            "change_coverage_digest": &initial_change_coverage.digest,
        },
        "changed_atoms": [],
        "extinguished_fires": [],
        "created_at": now,
    });
    let commit_id = codefire_store::store_object(&objects, "commit", commit_payload)?;
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": "main",
        "head": commit_id,
        "state": "closed",
        "created_at": now,
    });
    write_json_atomic(
        &cf.join("branches")
            .join(format!("{}.json", ref_file_name("main"))),
        &branch,
    )?;
    codefire_store::store_object(&objects, "branch", branch)?;

    Ok(InitResult {
        repo_root,
        main_commit: commit_id,
    })
}

fn merge_branch(start: &Path, options: &MergeOptions) -> Result<MergeResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    if options.source_branch == options.target_branch {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.source_branch),
            None,
            format!(
                "cannot merge branch '{}' into itself",
                options.source_branch
            ),
        ));
    }
    let idempotency_payload = merge_idempotency_payload(&repo_root, options)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_merge_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    let mut result = build_merge_result(&repo_root, options, lock.as_ref())?;
    if options.dry_run {
        return Ok(result);
    }
    apply_merge_result(
        &repo_root,
        &mut result,
        lock.as_ref().expect("live merge must hold repo lock"),
    )?;
    if let Some(key) = options.idempotency_key.as_deref() {
        save_merge_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

fn build_merge_result(
    repo_root: &Path,
    options: &MergeOptions,
    lock: Option<&RepoLock>,
) -> Result<MergeResult, CliError> {
    if options.source_branch == options.target_branch {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.source_branch),
            None,
            format!(
                "cannot merge branch '{}' into itself",
                options.source_branch
            ),
        ));
    }
    let objects = repo_root.join(".codefire").join("objects");
    let source = load_branch_record(repo_root, &options.source_branch).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(&options.source_branch), None)
    })?;
    let source_head = required_string(&source, &["head"])?;
    ensure_sealed_source_state("merge", &options.source_branch, branch_state(&source)).map_err(
        |error| with_branch_operation_context(error, repo_root, Some(&options.source_branch), None),
    )?;
    let target = load_branch_record(repo_root, &options.target_branch).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(&options.target_branch), None)
    })?;
    let target_head = required_string(&target, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &source_head)?;
    codefire_store::validate_sealed_commit(&objects, &target_head)?;

    let registry_path = opened_registry_path(repo_root, &options.target_branch);
    if !registry_path.exists() {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.target_branch),
            None,
            format!("target branch must be open: {}", options.target_branch),
        ));
    }
    let registry = read_json(&registry_path)?;
    let target_dir = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let target_state = if let Some(lock) = lock {
        refresh_open_clean_for_branch_operation(
            repo_root,
            &options.target_branch,
            &target_dir,
            lock,
        )?;
        required_string(&read_json(&registry_path)?, &["state", "last_known"])?
    } else {
        let scan = compute_scan(&target_dir)?;
        scan_branch_state(scan_change_count(&scan.scan), scan.scan.open_fires.len()).to_string()
    };
    if target_state != "open-clean" {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.target_branch),
            Some(&target_dir),
            format!(
                "target branch has unsealed open work ({target_state}); it must be open-clean before merge"
            ),
        ));
    }
    let base = common_ancestor(&objects, &source_head, &target_head)?
        .ok_or_else(|| CliError::Usage("no common ancestor found".to_string()))?;
    let base_files = manifest_contents(&objects, &base)?;
    let source_files = manifest_contents(&objects, &source_head)?;
    let target_files = manifest_contents(&objects, &target_head)?;

    let paths = base_files
        .keys()
        .chain(source_files.keys())
        .chain(target_files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut conflicts = Vec::new();
    let mut binary_conflicts = Vec::new();
    let mut file_actions = Vec::new();
    for path in paths {
        let base_data = base_files.get(&path);
        let source_data = source_files.get(&path);
        let target_data = target_files.get(&path);
        if source_data == base_data {
            continue;
        }
        if target_data == base_data {
            file_actions.push(MergeFileAction {
                path,
                action: if source_data.is_some() {
                    MergeAction::WriteSource
                } else {
                    MergeAction::DeleteTarget
                },
                content: source_data.cloned(),
            });
            continue;
        }
        if source_data == target_data {
            continue;
        }
        conflicts.push(path.clone());
        let target_bytes = target_data.map(Vec::as_slice);
        let source_bytes = source_data.map(Vec::as_slice);
        if conflict_markers_are_safe(target_bytes, source_bytes) {
            let content = conflict_content(target_bytes, source_bytes);
            file_actions.push(MergeFileAction {
                path,
                action: MergeAction::WriteConflictMarkers,
                content: Some(content),
            });
        } else {
            let conflict = binary_merge_conflict(&path, target_bytes, source_bytes);
            if let (Some(side_path), Some(bytes)) = (&conflict.target_path, target_bytes) {
                file_actions.push(MergeFileAction {
                    path: side_path.clone(),
                    action: MergeAction::WriteConflictSide,
                    content: Some(bytes.to_vec()),
                });
            }
            if let (Some(side_path), Some(bytes)) = (&conflict.source_path, source_bytes) {
                file_actions.push(MergeFileAction {
                    path: side_path.clone(),
                    action: MergeAction::WriteConflictSide,
                    content: Some(bytes.to_vec()),
                });
            }
            binary_conflicts.push(conflict);
        }
    }

    let semantic_conflicts =
        semantic_conflict_candidates(&objects, &base, &source_head, &target_head)?;
    Ok(MergeResult {
        repo_root: repo_root.to_path_buf(),
        source_branch: options.source_branch.clone(),
        target_branch: options.target_branch.clone(),
        source_head,
        target_head,
        base,
        target_dir,
        file_actions,
        conflicts,
        binary_conflicts,
        semantic_conflicts,
        dry_run: options.dry_run,
        applied: false,
    })
}

fn apply_merge_result(
    repo_root: &Path,
    result: &mut MergeResult,
    lock: &RepoLock,
) -> Result<(), CliError> {
    let target = ExistingWorktree::bind(&result.target_dir)?;
    for action in &result.file_actions {
        write_merged_file(&target, &action.path, action.content.as_deref())?;
    }
    let registry_path = opened_registry_path(repo_root, &result.target_branch);
    let registry = read_json(&registry_path)?;
    let active_state = validated_active_state_dir(repo_root, &registry)?;
    let mut state = read_active_json_optional(&active_state, Path::new("state.json"))?
        .unwrap_or_else(|| json!({}));
    state["pending_merge_parent"] = Value::String(result.source_head.clone());
    state["pending_merge_base"] = Value::String(result.base.clone());
    state["merge_conflicts"] = Value::Array(
        result
            .conflicts
            .iter()
            .cloned()
            .map(Value::String)
            .collect(),
    );
    state["binary_merge_conflicts"] = Value::Array(
        result
            .binary_conflicts
            .iter()
            .map(binary_merge_conflict_json)
            .collect(),
    );
    state["semantic_conflict_candidates"] = Value::Array(
        result
            .semantic_conflicts
            .iter()
            .map(semantic_conflict_json)
            .collect(),
    );
    write_active_json_atomic(&active_state, Path::new("state.json"), &state)?;
    compute_scan_locked(&result.target_dir, lock)?;
    let context = open_context(&result.target_dir)?;
    set_open_state(&context, "open-burning")?;
    result.applied = true;
    Ok(())
}

fn semantic_conflict_candidates(
    objects: &Path,
    base_commit: &str,
    source_commit: &str,
    target_commit: &str,
) -> Result<Vec<SemanticConflictCandidate>, CliError> {
    let base_index = load_base_atom_index(objects, base_commit)?;
    let source_index = load_base_atom_index(objects, source_commit)?;
    let target_index = load_base_atom_index(objects, target_commit)?;
    let source_changed = codefire_core::changed_atoms(&source_index, &base_index)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let target_changed = codefire_core::changed_atoms(&target_index, &base_index)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let base_atoms = atom_by_id(&base_index);
    let source_atoms = atom_by_id(&source_index);
    let target_atoms = atom_by_id(&target_index);
    let mut candidates = Vec::new();
    for atom_id in source_changed.intersection(&target_changed) {
        let source_atom = source_atoms.get(atom_id.as_str()).copied();
        let target_atom = target_atoms.get(atom_id.as_str()).copied();
        let source_hash = source_atom.map(|atom| atom.content_hash.clone());
        let target_hash = target_atom.map(|atom| atom.content_hash.clone());
        if source_hash == target_hash {
            continue;
        }
        candidates.push(SemanticConflictCandidate {
            atom_id: atom_id.clone(),
            base_hash: base_atoms
                .get(atom_id.as_str())
                .map(|atom| atom.content_hash.clone()),
            source_hash,
            target_hash,
            source_path: source_atom.map(|atom| atom.artifact_path.clone()),
            target_path: target_atom.map(|atom| atom.artifact_path.clone()),
        });
    }
    candidates.sort_by(|left, right| left.atom_id.cmp(&right.atom_id));
    Ok(candidates)
}

fn atom_by_id(index: &codefire_core::AtomIndex) -> BTreeMap<&str, &codefire_core::Atom> {
    index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect()
}

fn ensure_sealed_source_state(command: &str, branch: &str, state: &str) -> Result<(), CliError> {
    match state {
        "closed" | "open-clean" => Ok(()),
        "open-burning" => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is open-burning"
        ))),
        "open-consistent" => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is open-consistent; commit or close the source branch first"
        ))),
        other => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is in unsupported state '{other}'"
        ))),
    }
}

fn branch_operation_error(
    repo_root: &Path,
    branch: Option<&str>,
    path: Option<&Path>,
    message: impl Into<String>,
) -> CliError {
    CliError::BranchOperation {
        message: message.into(),
        repo_root: repo_root.to_path_buf(),
        branch: branch.map(str::to_string),
        path: path.map(Path::to_path_buf),
    }
}

fn with_branch_operation_context(
    error: CliError,
    repo_root: &Path,
    branch: Option<&str>,
    path: Option<&Path>,
) -> CliError {
    match error {
        CliError::Usage(message) => branch_operation_error(repo_root, branch, path, message),
        other => other,
    }
}

fn branch_state(value: &Value) -> &str {
    value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed")
}

fn render_merge_dry_run(result: &MergeResult) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "merge dry-run {} into {}; target unchanged\n",
        result.source_branch, result.target_branch
    ));
    output.push_str(&format!("  base: {}\n", result.base));
    output.push_str(&format!("  source: {}\n", result.source_head));
    output.push_str(&format!("  target: {}\n", result.target_head));
    output.push_str(&format!("  target dir: {}\n", result.target_dir.display()));
    output.push_str(&format!(
        "  file actions: {} conflicts: {} binary conflicts: {} semantic candidates: {}\n",
        result.file_actions.len(),
        result.conflicts.len(),
        result.binary_conflicts.len(),
        result.semantic_conflicts.len()
    ));
    if result.file_actions.is_empty() {
        output.push_str("  no file changes predicted\n");
    } else {
        for action in &result.file_actions {
            output.push_str(&format!("  {}: {}\n", action.action.as_str(), action.path));
        }
    }
    if !result.binary_conflicts.is_empty() {
        output.push_str("Binary conflicts:\n");
        for conflict in &result.binary_conflicts {
            output.push_str(&format!(
                "  {} target={} source={}\n",
                conflict.path,
                conflict.target_path.as_deref().unwrap_or("(deleted)"),
                conflict.source_path.as_deref().unwrap_or("(deleted)")
            ));
        }
    }
    if !result.semantic_conflicts.is_empty() {
        output.push_str("Semantic conflict candidates:\n");
        for candidate in &result.semantic_conflicts {
            output.push_str(&format!(
                "  {} source={} target={}\n",
                candidate.atom_id,
                candidate.source_hash.as_deref().unwrap_or("(missing)"),
                candidate.target_hash.as_deref().unwrap_or("(missing)")
            ));
        }
    }
    output
}

#[cfg(test)]
fn render_merge_result_json(result: &MergeResult) -> Result<String, CliError> {
    Ok(serde_json::to_string_pretty(&merge_result_json_value(
        result,
    ))?)
}

fn merge_result_json_value(result: &MergeResult) -> Value {
    json!({
        "type": "codefire_merge_plan",
        "version": 1,
        "repo": &result.repo_root,
        "dry_run": result.dry_run,
        "applied": result.applied,
        "source": {
            "branch": &result.source_branch,
            "commit": &result.source_head,
        },
        "target": {
            "branch": &result.target_branch,
            "commit": &result.target_head,
            "open_dir": result.target_dir,
        },
        "base": {"commit": &result.base},
        "file_actions": result.file_actions.iter().map(merge_file_action_json).collect::<Vec<_>>(),
        "conflicts": &result.conflicts,
        "binary_conflicts": result.binary_conflicts.iter().map(binary_merge_conflict_json).collect::<Vec<_>>(),
        "semantic_conflicts": result.semantic_conflicts.iter().map(semantic_conflict_json).collect::<Vec<_>>(),
        "next_actions": merge_next_actions(result),
    })
}

fn merge_file_action_json(action: &MergeFileAction) -> Value {
    json!({
        "path": &action.path,
        "action": action.action.as_str(),
        "bytes": action.content.as_ref().map(Vec::len),
    })
}

fn binary_merge_conflict_json(conflict: &BinaryMergeConflict) -> Value {
    json!({
        "path": &conflict.path,
        "target_path": &conflict.target_path,
        "source_path": &conflict.source_path,
        "target_bytes": conflict.target_bytes,
        "source_bytes": conflict.source_bytes,
    })
}

fn semantic_conflict_json(candidate: &SemanticConflictCandidate) -> Value {
    json!({
        "atom_id": &candidate.atom_id,
        "base_hash": &candidate.base_hash,
        "source_hash": &candidate.source_hash,
        "target_hash": &candidate.target_hash,
        "source_path": &candidate.source_path,
        "target_path": &candidate.target_path,
    })
}

fn merge_next_actions(result: &MergeResult) -> Vec<Value> {
    let mut actions = Vec::new();
    if result.dry_run && result.conflicts.is_empty() && result.semantic_conflicts.is_empty() {
        actions.push(json!({
            "kind": "apply_merge",
            "command": format!(
                "codefire merge {} --into {}",
                command_arg(&result.source_branch),
                command_arg(&result.target_branch)
            ),
            "reason": "apply the predicted merge to the open-clean target branch",
            "source_branch": &result.source_branch,
            "target_branch": &result.target_branch,
            "target": {
                "source_branch": &result.source_branch,
                "target_branch": &result.target_branch,
                "source_commit": &result.source_head,
                "target_commit": &result.target_head,
                "target_open_dir": &result.target_dir,
            },
            "hint": format!(
                "run codefire merge {} --into {}",
                command_arg(&result.source_branch),
                command_arg(&result.target_branch)
            ),
        }));
    }
    for conflict in &result.conflicts {
        let binary = result
            .binary_conflicts
            .iter()
            .find(|item| item.path == *conflict);
        actions.push(json!({
            "kind": "resolve_file_conflict",
            "path": conflict,
            "binary": binary.is_some(),
            "target_path": binary.and_then(|item| item.target_path.as_ref()),
            "source_path": binary.and_then(|item| item.source_path.as_ref()),
            "hint": if binary.is_some() {
                format!("merge saved binary conflict sides for {conflict}; choose or reconstruct the target file before commit")
            } else {
                format!("merge writes conflict markers for {conflict}; resolve them before commit")
            },
        }));
    }
    for candidate in &result.semantic_conflicts {
        actions.push(json!({
            "kind": "review_semantic_conflict",
            "atom_id": &candidate.atom_id,
            "source_hash": &candidate.source_hash,
            "target_hash": &candidate.target_hash,
            "hint": format!("review concurrent semantic changes to {}", candidate.atom_id),
        }));
    }
    actions
}

fn import_patch(start: &Path, options: &PatchImportOptions) -> Result<Value, CliError> {
    let _lock;
    let context;
    if options.dry_run {
        _lock = None;
        context = open_context(start)?;
    } else {
        let (lock, locked_context) = acquire_locked_open_context(start, &options.lock)?;
        _lock = Some(lock);
        context = locked_context;
    }
    let patch = read_json(&options.path)?;
    if patch.get("type").and_then(Value::as_str) != Some("codefire_patch") {
        return Err(CliError::Usage(
            "patch file type must be codefire_patch".to_string(),
        ));
    }
    if patch.get("version").and_then(Value::as_u64) != Some(1) {
        return Err(CliError::Usage(
            "unsupported patch version; expected version 1".to_string(),
        ));
    }
    let patch_base = required_string(&patch, &["base", "commit"])?;
    let current_base = required_string(&context.registry, &["open", "current_base_commit"])?;
    if patch_base != current_base {
        return Err(CliError::Usage(format!(
            "patch base {patch_base} does not match open directory base {current_base}"
        )));
    }
    let entries = patch
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("patch entries must be a list".to_string()))?;
    let mut paths = Vec::with_capacity(entries.len());
    for entry in entries {
        let path = required_string(entry, &["path"])?;
        safe_relative_manifest_path(&path)?;
        paths.push(path);
    }

    let idempotency_payload =
        patch_import_idempotency_payload(options, &context, &patch, &patch_base, &paths)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_patch_import_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }

    if !options.dry_run {
        let open_worktree = ExistingWorktree::bind(&context.open_dir)?;
        for entry in entries {
            apply_patch_entry(&open_worktree, entry)?;
        }
        let mut state = read_active_json_optional(&context.active_state, Path::new("state.json"))?
            .unwrap_or_else(|| json!({}));
        state["pending_patch_base"] = Value::String(patch_base.clone());
        state["pending_patch_source"] = patch
            .get("source")
            .and_then(|source| source.get("commit"))
            .cloned()
            .unwrap_or(Value::Null);
        state["patch_paths"] = Value::Array(paths.iter().cloned().map(Value::String).collect());
        write_active_json_atomic(&context.active_state, Path::new("state.json"), &state)?;
        set_open_state(&context, "open-burning")?;
    }

    let result = json!({
        "type": "codefire_patch_import",
        "version": 1,
        "dry_run": options.dry_run,
        "applied": !options.dry_run,
        "branch": &context.branch,
        "base": patch_base,
        "entries": entries.len(),
        "paths": paths,
    });
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            save_patch_import_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
        }
    }
    Ok(result)
}

fn apply_patch_entry(open_worktree: &ExistingWorktree, entry: &Value) -> Result<(), CliError> {
    let path = required_string(entry, &["path"])?;
    match required_string(entry, &["action"])?.as_str() {
        "write" => {
            let encoding = required_string(entry, &["encoding"])?;
            if encoding != "base64" {
                return Err(CliError::Usage(format!(
                    "unsupported patch entry encoding: {encoding}"
                )));
            }
            let content = required_string(entry, &["content"])?;
            let bytes = decode_base64(&content)?;
            write_merged_file(open_worktree, &path, Some(&bytes))
        }
        "delete" => write_merged_file(open_worktree, &path, None),
        action => Err(CliError::Usage(format!(
            "unsupported patch entry action: {action}"
        ))),
    }
}

fn clone_branch(start: &Path, options: &CloneOptions) -> Result<CloneResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    let idempotency_payload = clone_idempotency_payload(options, &repo_root);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_clone_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    if branch_record_path(&repo_root, &options.new_branch).exists() {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.new_branch),
            None,
            format!("branch already exists: {}", options.new_branch),
        ));
    }
    if is_cf_http_url(&options.source) {
        let remote = parse_cf_http_url(&options.source)?;
        let bundle = http_json(
            "GET",
            &http_remote_path(
                &remote.endpoint,
                &remote.org,
                &remote.app,
                &["branches", &remote.branch, "bundle"],
            ),
            None,
        )?;
        let branch_record = bundle
            .get("branch")
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?
            .clone();
        let records = bundle
            .get("objects")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?;
        let source_head = required_string(&branch_record, &["head"])?;
        ensure_sealed_source_state("clone", &remote.branch, branch_state(&branch_record)).map_err(
            |error| with_branch_operation_context(error, &repo_root, Some(&remote.branch), None),
        )?;
        validate_http_bundle_commit(records, &source_head)?;
        let plan =
            clone_operation_plan(options, &repo_root, &source_head, remote.endpoint_scheme());
        if options.dry_run {
            return Ok(CloneResult { plan });
        }
        write_object_records(
            &repo_root.join(".codefire").join("objects"),
            records.iter().cloned(),
        )?;
        codefire_store::validate_sealed_commit(
            &repo_root.join(".codefire").join("objects"),
            &source_head,
        )?;
        let branch = json!({
            "type": "branch",
            "version": 1,
            "name": options.new_branch,
            "head": source_head,
            "state": "closed",
            "created_from": options.source,
            "created_at": now_iso_utc(),
        });
        save_branch_record(&repo_root, &branch)?;
        let result = CloneResult { plan };
        if let Some(key) = options.idempotency_key.as_deref() {
            save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    if options.source.starts_with("cf://") {
        let remote = parse_cf_url(&options.source)?;
        let source = load_remote_branch(&remote)?;
        let source_head = required_string(&source, &["head"])?;
        ensure_sealed_source_state("clone", &remote.branch, branch_state(&source)).map_err(
            |error| with_branch_operation_context(error, &repo_root, Some(&remote.branch), None),
        )?;
        let remote_objects = remote_dirs(&remote.project_root).objects;
        codefire_store::validate_sealed_commit(&remote_objects, &source_head)?;
        let plan = clone_operation_plan(options, &repo_root, &source_head, "cf");
        if options.dry_run {
            return Ok(CloneResult { plan });
        }
        copy_object_graph(
            &remote_objects,
            &repo_root.join(".codefire").join("objects"),
            &source_head,
        )?;
        let branch = json!({
            "type": "branch",
            "version": 1,
            "name": options.new_branch,
            "head": source_head,
            "state": "closed",
            "created_from": options.source,
            "created_at": now_iso_utc(),
        });
        save_branch_record(&repo_root, &branch)?;
        let result = CloneResult { plan };
        if let Some(key) = options.idempotency_key.as_deref() {
            save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    let source = load_branch_record(&repo_root, &options.source).map_err(|error| {
        with_branch_operation_context(error, &repo_root, Some(&options.source), None)
    })?;
    let source_head = required_string(&source, &["head"])?;
    ensure_sealed_source_state("clone", &options.source, branch_state(&source)).map_err(
        |error| with_branch_operation_context(error, &repo_root, Some(&options.source), None),
    )?;
    codefire_store::validate_sealed_commit(
        &repo_root.join(".codefire").join("objects"),
        &source_head,
    )?;
    let plan = clone_operation_plan(options, &repo_root, &source_head, "local");
    if options.dry_run {
        return Ok(CloneResult { plan });
    }
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": options.new_branch,
        "head": source_head,
        "state": "closed",
        "created_from": options.source,
        "created_at": now_iso_utc(),
    });
    save_branch_record(&repo_root, &branch)?;
    let result = CloneResult { plan };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_clone_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

fn close_branch(start: &Path, options: &CloseOptions) -> Result<CloseResult, CliError> {
    close_transaction::close_branch_from(start, options)
}

fn validate_close_open_marker(
    repo_root: &Path,
    open_dir: &Path,
    branch: &str,
    registry: &Value,
    marker: &Value,
) -> Result<(), CliError> {
    let marker_branch = required_string(marker, &["branch", "name"])?;
    if marker_branch != branch {
        return Err(CliError::InvalidMarker(format!(
            "open marker branch mismatch: expected {branch}, found {marker_branch}"
        )));
    }
    let marker_repo_path = PathBuf::from(required_string(marker, &["repository", "path"])?);
    if marker_repo_path != repo_root.join(".codefire") {
        return Err(CliError::InvalidMarker(format!(
            "open marker repository mismatch for {}",
            open_dir.display()
        )));
    }
    let repository = read_json(&repo_root.join(".codefire").join("repo.json"))?;
    let expected_repository_id = required_string(&repository, &["repository_id"])?;
    let marker_repository_id = required_string(marker, &["repository", "repository_id"])?;
    if marker_repository_id != expected_repository_id {
        return Err(CliError::InvalidMarker(format!(
            "open marker repository identity mismatch for {}",
            open_dir.display()
        )));
    }
    let marker_open_path = PathBuf::from(required_string(marker, &["open", "opened_path"])?);
    if marker_open_path != open_dir {
        return Err(CliError::InvalidMarker(format!(
            "open marker path mismatch: expected {}, found {}",
            open_dir.display(),
            marker_open_path.display()
        )));
    }
    let marker_instance = required_string(marker, &["open", "open_instance_id"])?;
    let registry_instance = required_string(registry, &["open", "open_instance_id"])?;
    if marker_instance != registry_instance {
        return Err(CliError::InvalidMarker(format!(
            "open marker instance mismatch: expected {registry_instance}, found {marker_instance}"
        )));
    }
    Ok(())
}

fn close_operation_plan(
    options: &CloseOptions,
    repo_root: &Path,
    open_dir: &Path,
    registry_path: &Path,
    active_state_path: &Path,
    state: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": if options.discard { "discard" } else { "close" },
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "branch": &options.branch,
        "open_dir": open_dir,
        "discard": options.discard,
        "keep_dir": options.keep_dir,
        "initial_state": state,
        "operations": [
            {"kind": if options.keep_dir { "remove_open_marker" } else { "remove_open_directory" }, "path": open_dir},
            {"kind": "remove_active_state", "path": active_state_path},
            {"kind": "remove_open_registry", "path": registry_path},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "closed"},
        ],
        "next_actions": [
            {
                "kind": "branch_show",
                "command": format!("codefire branch show {} --json", command_arg(&options.branch)),
                "reason": "confirm the branch is closed",
                "target": {"branch": &options.branch, "path": repo_root},
            },
        ],
    })
}

fn open_branch(options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_branch_from(&env::current_dir()?, options)
}

fn open_branch_from(start: &Path, options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_transaction::open_branch_from(start, options)
}

fn open_operation_plan(
    options: &OpenOptions,
    repo_root: &Path,
    target: &Path,
    branch_head: &str,
    registry_path: &Path,
    active_path: &Path,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "open",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &options.branch,
        "repo": repo_root,
        "target": target,
        "base_commit": branch_head,
        "operations": [
            {"kind": "materialize_commit", "commit": branch_head, "target": target, "publication": "sibling_stage"},
            {"kind": "write_open_marker", "path": target.join(".codefire-open"), "publication": "sibling_stage"},
            {"kind": "publish_open_target", "path": target, "visibility": "single_sibling_rename"},
            {"kind": "write_open_registry", "path": registry_path},
            {"kind": "create_active_state", "path": active_path},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "open-clean"},
        ],
        "next_actions": [
            {
                "kind": "status",
                "command": format!("codefire status --path {} --json", command_arg(&target.to_string_lossy())),
                "reason": "inspect the newly opened branch state",
                "target": {"path": target, "branch": &options.branch},
            },
        ],
    })
}

fn clone_operation_plan(
    options: &CloneOptions,
    repo_root: &Path,
    source_head: &str,
    source_kind: &str,
) -> Value {
    let suggested_open_path = suggested_branch_open_path(repo_root, &options.new_branch);
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "clone",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "source": {
            "value": &options.source,
            "kind": source_kind,
            "head": source_head,
        },
        "new_branch": &options.new_branch,
        "operations": [
            {"kind": "copy_object_graph_if_remote", "commit": source_head},
            {"kind": "write_branch_record", "branch": &options.new_branch, "head": source_head, "state": "closed"},
        ],
        "next_actions": [
            {
                "kind": "open",
                "command": format!(
                    "codefire open {} {}",
                    command_arg(&options.new_branch),
                    command_arg(&suggested_open_path.to_string_lossy())
                ),
                "reason": "open the newly cloned branch into a working directory",
                "target": {
                    "branch": &options.new_branch,
                    "repo": repo_root,
                    "recommended_path": suggested_open_path,
                    "path_policy": "use a sibling of the CodeFire repo root and never nest inside an existing open directory",
                },
            },
        ],
    })
}

fn suggested_branch_open_path(repo_root: &Path, branch: &str) -> PathBuf {
    let parent = repo_root.parent().unwrap_or(repo_root);
    let repo_name = repo_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("codefire-worktree");
    parent.join(format!(
        "{}-{}",
        repo_name,
        filesystem_safe_branch_suffix(branch)
    ))
}

fn filesystem_safe_branch_suffix(branch: &str) -> String {
    let suffix = branch
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if suffix.is_empty() {
        "branch".to_string()
    } else {
        suffix
    }
}

fn is_primary_branch(branch: &str) -> bool {
    branch == "main"
}

fn resolve_open_target(
    repo_root: &Path,
    requested: &Path,
    requested_branch: &str,
) -> Result<(PathBuf, secure_worktree::PinnedTargetResolution), CliError> {
    let boundary = fs::canonicalize(repo_root.parent().unwrap_or(repo_root))?;
    let requested_absolute = absolute_path(requested)?;
    let contains_parent = requested_absolute
        .components()
        .any(|component| component == Component::ParentDir);
    let mut lexical_target = PathBuf::new();
    for component in requested_absolute.components() {
        if component != Component::CurDir {
            lexical_target.push(component.as_os_str());
        }
    }
    if !contains_parent && !lexical_target.starts_with(&boundary) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&lexical_target),
            &boundary,
            "outside_workspace_boundary",
            format!(
                "open target must stay under resolved workspace boundary {}: requested {}, resolved {}",
                boundary.display(),
                requested.display(),
                lexical_target.display()
            ),
        ));
    }
    let target_resolution = secure_worktree::PinnedTargetResolution::resolve(&boundary, requested)
        .map_err(|error| {
            open_target_resolution_error(
                repo_root,
                requested_branch,
                requested,
                None,
                &boundary,
                "unsafe_path_component",
                error.to_string(),
            )
        })?;
    let target = target_resolution.target_path().to_path_buf();
    if !target.starts_with(&boundary) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "outside_workspace_boundary",
            format!(
                "open target must stay under resolved workspace boundary {}: requested {}, resolved {}",
                boundary.display(),
                requested.display(),
                target.display()
            ),
        ));
    }
    if target == repo_root || target.starts_with(repo_root) || repo_root.starts_with(&target) {
        return Err(open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "repository_overlap",
            format!(
                "open target must not overlap repository root {}: requested {}, resolved {}; choose a directory outside the repository root, for example {}",
                repo_root.display(),
                requested.display(),
                target.display(),
                suggested_open_dir(repo_root, &boundary, requested_branch).display()
            ),
        ));
    }
    let opened_dir = repo_root.join(".codefire").join("opened");
    if opened_dir.is_dir() {
        for entry in fs::read_dir(opened_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let registry = read_json(&entry.path())?;
                let registered_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
                let open_path = resolve_path_without_symlinks(&registered_path)?;
                let registered_branch = required_string(&registry, &["branch", "name"])?;
                if target == open_path && registered_branch == requested_branch {
                    continue;
                }
                if target.starts_with(&open_path) || open_path.starts_with(&target) {
                    return Err(open_target_resolution_error(
                        repo_root,
                        requested_branch,
                        requested,
                        Some(&target),
                        &boundary,
                        "open_directory_overlap",
                        format!(
                            "open target must not be nested inside existing open directory or overlap it {}: requested {}, resolved {}",
                            open_path.display(),
                            requested.display(),
                            target.display()
                        ),
                    ));
                }
            }
        }
    }
    target_resolution.ensure_current().map_err(|error| {
        open_target_resolution_error(
            repo_root,
            requested_branch,
            requested,
            Some(&target),
            &boundary,
            "identity_changed_during_resolution",
            format!("open target ancestor changed during resolution: {error}"),
        )
    })?;
    Ok((target, target_resolution))
}

fn open_target_resolution_error(
    repo_root: &Path,
    branch: &str,
    requested_path: &Path,
    resolved_path: Option<&Path>,
    workspace_boundary: &Path,
    reason: &'static str,
    message: String,
) -> CliError {
    CliError::OpenTargetResolution(Box::new(OpenTargetResolutionError {
        message,
        reason,
        repo_root: repo_root.to_path_buf(),
        branch: branch.to_string(),
        requested_path: requested_path.to_path_buf(),
        resolved_path: resolved_path.map(Path::to_path_buf),
        workspace_boundary: workspace_boundary.to_path_buf(),
    }))
}

/// A directory beside the repository root to suggest after a rejected target.
fn suggested_open_dir(repo_root: &Path, boundary: &Path, branch: &str) -> PathBuf {
    let leaf = branch.replace('/', "-");
    let candidate = boundary.join(&leaf);
    if candidate.starts_with(repo_root) || repo_root.starts_with(&candidate) {
        boundary.join(format!("{leaf}-open"))
    } else {
        candidate
    }
}

fn resolve_path_without_symlinks(path: &Path) -> Result<PathBuf, CliError> {
    let absolute = absolute_path(path)?;
    if absolute
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(CliError::Usage(format!(
            "path traversal with '..' is not allowed for open targets: {}; use an absolute path without '..' (open directories live beside the repository root, not inside it)",
            path.display()
        )));
    }

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => unreachable!("parent components were rejected above"),
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    let mut existing_ancestor = normalized.as_path();
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(existing_ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(CliError::Usage(format!(
                        "symlink components are not allowed for open targets: {}",
                        existing_ancestor.display()
                    )));
                }
                if !suffix.is_empty() && !metadata.is_dir() {
                    return Err(CliError::Usage(format!(
                        "open target ancestor is not a directory: {}",
                        existing_ancestor.display()
                    )));
                }
                break;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let name = existing_ancestor.file_name().ok_or_else(|| {
                    CliError::Usage(format!(
                        "open target has no existing filesystem ancestor: {}",
                        path.display()
                    ))
                })?;
                suffix.push(name.to_os_string());
                existing_ancestor = existing_ancestor.parent().ok_or_else(|| {
                    CliError::Usage(format!(
                        "open target has no existing filesystem ancestor: {}",
                        path.display()
                    ))
                })?;
            }
            Err(error) => return Err(CliError::Io(error)),
        }
    }

    let mut prefix = PathBuf::new();
    for component in existing_ancestor.components() {
        prefix.push(component.as_os_str());
        if fs::symlink_metadata(&prefix)?.file_type().is_symlink() {
            return Err(CliError::Usage(format!(
                "symlink components are not allowed for open targets: {}",
                prefix.display()
            )));
        }
    }

    let mut resolved = fs::canonicalize(existing_ancestor)?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn refresh_open_clean_for_branch_operation(
    repo_root: &Path,
    branch: &str,
    open_dir: &Path,
    lock: &RepoLock,
) -> Result<(), CliError> {
    let registry_path = opened_registry_path(repo_root, branch);
    let registry = read_json(&registry_path)?;
    let base_commit = required_string(&registry, &["open", "current_base_commit"])?;
    let objects = repo_root.join(".codefire").join("objects");
    let base_path = codefire_store::object_record_path(&objects, &base_commit)
        .unwrap_or_else(|| objects.clone());
    codefire_store::validate_sealed_commit(&objects, &base_commit).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: repo_root.to_path_buf(),
            path: base_path,
        }
    })?;
    if source_manifest_unchanged(&objects, &base_commit, open_dir)? {
        let state = required_string(&registry, &["state", "last_known"])?;
        if state == "open-clean" {
            return Ok(());
        }
        return Err(branch_operation_error(
            repo_root,
            Some(branch),
            Some(open_dir),
            format!("branch {branch} is {state}; commit, verify, or discard before continuing"),
        ));
    }
    let scan = compute_scan_locked(open_dir, lock).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(branch), Some(open_dir))
    })?;
    let state = scan_branch_state(scan_change_count(&scan.scan), scan.scan.open_fires.len());
    if state == "open-clean" {
        Ok(())
    } else {
        Err(branch_operation_error(
            repo_root,
            Some(branch),
            Some(open_dir),
            format!(
                "branch {branch} has unsealed open work after refresh ({state}); commit, verify, or discard before continuing"
            ),
        ))
    }
}

fn validate_http_bundle_commit(records: &[Value], source_head: &str) -> Result<(), CliError> {
    let temp_objects = env::temp_dir().join(format!(
        "codefire-http-bundle-{}-{:012x}",
        std::process::id(),
        stable_hash_48(format!("{}:{}", source_head, now_iso_utc()).as_bytes())
    ));
    if temp_objects.exists() {
        fs::remove_dir_all(&temp_objects)?;
    }
    let result = (|| {
        write_object_records(&temp_objects, records.iter().cloned())?;
        codefire_store::validate_sealed_commit(&temp_objects, source_head)?;
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&temp_objects);
    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) if error.kind() != ErrorKind::NotFound => Err(CliError::Io(error)),
        (Ok(()), _) => Ok(()),
    }
}

#[cfg(test)]
fn run_scan(start: &Path) -> Result<codefire_core::ScanResult, CliError> {
    run_scan_with_lock_options(start, &LockOptions::default())
}

fn run_scan_with_lock_options(
    start: &Path,
    options: &LockOptions,
) -> Result<codefire_core::ScanResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    Ok(compute_scan_with_cache_for_recovery(
        start,
        Some(&lock),
        &mut validation_cache,
        Some("scan"),
    )?
    .scan)
}

fn compute_scan(start: &Path) -> Result<ScanExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_scan_with_cache(start, None, &mut validation_cache)
}

fn compute_scan_locked(start: &Path, lock: &RepoLock) -> Result<ScanExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_scan_with_cache(start, Some(lock), &mut validation_cache)
}

fn compute_scan_with_cache(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Result<ScanExecution, CliError> {
    compute_scan_with_cache_for_recovery(start, lock, validation_cache, None)
}

fn compute_scan_with_cache_for_recovery(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
    recovery_operation: Option<&str>,
) -> Result<ScanExecution, CliError> {
    let context = match recovery_operation {
        Some(operation) => open_context_for_recovery(start, operation)?,
        None => open_context(start)?,
    };
    let recovering_verify_state = recovery_operation == Some("verify")
        && matches!(
            &context.authority,
            OpenContextAuthority::Verified(snapshot)
                if matches!(
                    &snapshot.disposition,
                    open_snapshot::OpenSnapshotDisposition::PendingState { operation, .. }
                        if operation == "verify"
                )
        );
    if let Some(lock) = lock {
        lock.ensure_repo_root(&context.repo_root)?;
    }
    maybe_swap_active_state_after_validation_for_test(&context)?;
    let objects = context.repo_root.join(".codefire").join("objects");
    let base_commit = required_string(&context.registry, &["open", "current_base_commit"])?;
    let base_commit_path = codefire_store::object_record_path(&objects, &base_commit)
        .unwrap_or_else(|| objects.clone());
    let validated_base = codefire_store::validate_sealed_commit_pinned_with_cache(
        &objects,
        &base_commit,
        validation_cache,
    )
    .map_err(|error| CliError::StoreContext {
        error: Box::new(error),
        repo_root: context.repo_root.clone(),
        path: base_commit_path,
    })?;
    let active_state = &context.active_state;
    let current = codefire_core::build_atom_index(&context.open_dir)?;
    let base_index = load_validated_base_atom_index(&validated_base)?;
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    codefire_core::validate_trace_graph_endpoints(&current, &trace_graph).map_err(|error| {
        CliError::CoreContext {
            error,
            repo_root: context.repo_root.clone(),
            path: context.open_dir.join("codefire.links.yaml"),
        }
    })?;
    let fires = read_active_json_optional(active_state, Path::new("fires.json"))?
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let state_data = read_active_json_optional(active_state, Path::new("state.json"))?
        .unwrap_or_else(|| json!({"state": "open-clean"}));
    let reason = if state_data.get("pending_merge_parent").is_some() {
        "merge_changed"
    } else {
        "atom_changed"
    };
    let now = now_iso_utc();
    let source_unchanged = source_manifest_unchanged_validated(&validated_base, &context.open_dir)?;
    let manifest_changes = manifest_file_changes_since_base(&validated_base, &context.open_dir)?;
    let (mut scan, fires) = codefire_core::build_scan_result_against_base(
        current,
        &base_index,
        trace_graph,
        fires,
        base_commit,
        codefire_core::ScanBuildOptions {
            reason,
            now: &now,
            source_unchanged,
        },
    )?;
    let coverage = codefire_core::build_change_coverage(
        manifest_changes,
        &scan.atom_index,
        &base_index,
        &scan.changed_atoms,
        scan.tool_migration.clone(),
    )?;
    scan.non_atom_changed_files = coverage
        .entries
        .iter()
        .filter(|entry| {
            entry.relation != codefire_core::ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans
        })
        .map(|entry| codefire_core::NonAtomFileChange {
            path: entry.path.clone(),
            status: entry.status.clone(),
            base_blob: entry.base_blob.clone(),
            current_blob: entry.current_blob.clone(),
        })
        .collect();
    scan.change_coverage = Some(coverage);
    if lock.is_some() && !recovering_verify_state {
        ensure_validated_store_subject(&context, &validated_base)?;
        let state = scan_branch_state(scan_change_count(&scan), scan.open_fires.len());
        begin_state_transaction(&context, "scan", state)?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "started")?;
        invalidate_active_verification(active_state)?;
        update_state_transaction_phase(active_state, "verification_invalidated")?;
        maybe_fail_state_transaction_for_test(
            active_state.path(),
            "scan",
            "verification_invalidated",
        )?;
        write_active_json_atomic(
            active_state,
            Path::new("fires.json"),
            &serde_json::to_value(&fires)?,
        )?;
        update_state_transaction_phase(active_state, "fires_written")?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "fires_written")?;
        write_active_json_atomic(
            active_state,
            Path::new("scan.json"),
            &serde_json::to_value(&scan)?,
        )?;
        update_state_transaction_phase(active_state, "scan_written")?;
        maybe_fail_state_transaction_for_test(active_state.path(), "scan", "scan_written")?;
        persist_transactional_open_state(
            &context,
            "scan",
            state,
            &validated_base,
            StateTransactionAuthority::Scan {
                scan: &scan,
                fires: &fires,
            },
        )?;
    }
    Ok(ScanExecution {
        context,
        scan,
        fires,
        validated_base,
    })
}

#[cfg(test)]
fn run_verify(start: &Path) -> Result<codefire_core::Verification, CliError> {
    Ok(run_verify_with_lock(start)?.verification)
}

#[cfg(test)]
fn run_verify_with_lock(start: &Path) -> Result<VerifyExecution, CliError> {
    run_verify_with_lock_options(start, &LockOptions::default())
}

fn run_verify_with_lock_options(
    start: &Path,
    options: &LockOptions,
) -> Result<VerifyExecution, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    compute_verify_locked(start, &lock)
}

fn compute_verify(start: &Path) -> Result<VerifyExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_verify_with_cache(start, None, &mut validation_cache)
}

fn compute_verify_locked(start: &Path, lock: &RepoLock) -> Result<VerifyExecution, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    compute_verify_with_cache(start, Some(lock), &mut validation_cache)
}

fn compute_verify_with_cache(
    start: &Path,
    lock: Option<&RepoLock>,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Result<VerifyExecution, CliError> {
    // Verification must reject an invalid/unsafe policy before its scan phase publishes any
    // active-state mutation. Parse again from the post-scan context below so the decision uses the
    // same final path resolution as the scan authority.
    let policy_preflight = if lock.is_some() {
        open_context_for_recovery(start, "verify")?
    } else {
        open_context(start)?
    };
    if let Some(lock) = lock {
        lock.ensure_repo_root(&policy_preflight.repo_root)?;
    }
    let preflight_policy =
        codefire_core::parse_effective_verification_policy(&policy_preflight.open_dir)?;
    validate_verification_command_plans(&policy_preflight.open_dir, &preflight_policy.policy)?;
    let merge_state_preflight =
        read_active_json_required(&policy_preflight.active_state, Path::new("state.json"))?;
    unresolved_merge_conflicts_from_state(&merge_state_preflight)?;
    let ScanExecution {
        context,
        scan,
        fires,
        validated_base,
    } = compute_scan_with_cache_for_recovery(
        start,
        lock,
        validation_cache,
        lock.map(|_| "verify"),
    )?;
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    validate_verification_command_plans(&context.open_dir, policy)?;
    let trace_policy = codefire_core::TracePolicy {
        required_links: policy.required_links.clone(),
    };
    let missing_required_links =
        codefire_core::required_link_missing(&scan.atom_index, &scan.trace_graph, &trace_policy);
    let subject = assessment_subject::live_open_subject(&context, &scan)?;
    let command_results = run_verification_commands(&context.open_dir, policy, &subject)?;
    let resolutions: Vec<codefire_core::Resolution> =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let stale_resolutions = codefire_core::stale_resolutions(
        &scan.atom_index,
        &scan.trace_graph,
        policy,
        &resolutions,
    )?;
    let missing_evidence_refs = collect_missing_evidence_refs(&context.repo_root, &resolutions);
    let state = read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let unresolved_merge_conflicts = unresolved_merge_conflicts_from_state(&state)?;
    let mut verification = codefire_core::build_verification(
        &scan,
        codefire_core::VerificationInputs {
            unresolved_merge_conflicts,
            missing_required_links,
            failed_checks: command_results.failures,
            stale_resolutions,
            missing_evidence_refs,
            subject: Some(subject),
            execution: command_results.records,
        },
        policy,
        &now_iso_utc(),
    );
    verification.effective_policy = Some(parsed_policy.effective);
    if lock.is_some() {
        maybe_swap_validated_store_before_verify_persist_for_test(&context.repo_root)?;
        persist_verification_result_state(&context, &scan, &fires, &verification, &validated_base)?;
    }
    Ok(VerifyExecution { scan, verification })
}

fn persist_verification_result_state(
    context: &OpenContext,
    scan: &codefire_core::ScanResult,
    fires: &[codefire_core::Fire],
    verification: &codefire_core::Verification,
    validated_base: &codefire_store::ValidatedSealedCommit,
) -> Result<(), CliError> {
    ensure_validated_store_subject(context, validated_base)?;
    let state = verification_open_state(verification, scan);
    begin_state_transaction(context, "verify", state)?;
    maybe_fail_state_transaction_for_test(context.active_state.path(), "verify", "started")?;
    write_active_json_atomic(
        &context.active_state,
        Path::new("verification.json"),
        &serde_json::to_value(verification)?,
    )?;
    update_state_transaction_phase(&context.active_state, "verification_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        "verify",
        "verification_written",
    )?;
    persist_transactional_open_state(
        context,
        "verify",
        state,
        validated_base,
        StateTransactionAuthority::Verify {
            scan,
            fires,
            verification,
        },
    )
}

fn verification_open_state(
    verification: &codefire_core::Verification,
    scan: &codefire_core::ScanResult,
) -> &'static str {
    if !codefire_core::committable_decision(verification).is_committable() {
        return "open-burning";
    }
    if scan_change_count(scan) == 0 && scan.open_fires.is_empty() {
        "open-clean"
    } else {
        "open-consistent"
    }
}

fn run_extinguish(options: &ExtinguishOptions) -> Result<ExtinguishResult, CliError> {
    let lock;
    let context;
    if options.dry_run {
        lock = None;
        context = open_context(&options.path)?;
    } else {
        let (acquired_lock, locked_context) =
            acquire_locked_open_context_for_recovery(&options.path, &options.lock, "extinguish")?;
        lock = Some(acquired_lock);
        context = locked_context;
    }
    let idempotency_payload = extinguish_idempotency_payload(options, &context);
    let ledger_request = (!options.dry_run)
        .then(|| extinguish_ledger_transition_request(&context, &idempotency_payload))
        .transpose()?;
    if let Some(request) = ledger_request.as_ref() {
        ensure_active_ledger_state_transaction_request(&context, "extinguish", request)?;
    }
    if options.dry_run {
        reject_pending_active_ledger_transition(&context.active_state)?;
    } else if let Some(durable) = recover_active_ledger_transition(
        &context.active_state,
        "extinguish",
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
    )? {
        let (result, target_state) = extinguish_result_from_ledger_transition(durable)?;
        set_open_state_transactional(
            &context,
            "extinguish",
            &target_state,
            ledger_request
                .as_ref()
                .expect("live extinguish must have a ledger request"),
        )?;
        if let Some(key) = options.idempotency_key.as_deref() {
            save_extinguish_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
        }
        return Ok(result);
    }
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_extinguish_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let active_state_path = context.active_state.path().to_path_buf();
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    if options.resolution == "no-change-required"
        && policy.no_change_required_requires_rationale
        && options.rationale.is_empty()
    {
        return Err(CliError::Usage(
            "no-change-required requires --rationale".to_string(),
        ));
    }
    if options.rationale.is_empty()
        && options.evidence.is_empty()
        && options.evidence_refs.is_empty()
    {
        return Err(CliError::Usage(
            "extinguish requires --rationale, --evidence, or --evidence-ref".to_string(),
        ));
    }
    validate_evidence_refs(&context.repo_root, &options.evidence_refs)?;

    let scan_execution = if options.dry_run {
        compute_scan(&context.open_dir)?
    } else {
        compute_scan_locked(
            &context.open_dir,
            lock.as_ref().expect("live extinguish must hold repo lock"),
        )?
    };
    let scan = scan_execution.scan;
    let mut fires = scan_execution.fires;
    let fire_index = fires
        .iter()
        .position(|fire| fire.display_id == options.fire_id || fire.fire_uid == options.fire_id)
        .ok_or_else(|| CliError::Usage(format!("unknown fire: {}", options.fire_id)))?;
    if fires[fire_index].status != "open" && !options.refresh {
        return Err(CliError::Usage(format!(
            "fire is not open: {}",
            options.fire_id
        )));
    }

    let resolved_at = now_iso_utc();
    let resolution_uid = format!(
        "res_{:012x}",
        stable_hash_48(format!("{}:{resolved_at}", options.fire_id).as_bytes())
    );
    let resolution = codefire_core::build_resolution(
        &fires[fire_index],
        &scan.atom_index,
        &scan.trace_graph,
        policy,
        codefire_core::ResolutionRequest {
            resolution_uid: resolution_uid.clone(),
            resolution_type: options.resolution.clone(),
            rationale: options.rationale.clone(),
            evidence: options.evidence.clone(),
            evidence_refs: options.evidence_refs.clone(),
            resolved_at,
        },
    )?;
    let display_id = fires[fire_index].display_id.clone();
    let fire_uid = fires[fire_index].fire_uid.clone();
    let source_atom = fires[fire_index].source.atom_id.clone();
    let target_atom = fires[fire_index].target.atom_id.clone();
    let reason = fires[fire_index].reason.clone();
    let plan = extinguish_operation_plan(
        options,
        &context,
        &active_state_path,
        &fires[fire_index],
        &resolution_uid,
    );
    if options.dry_run {
        return Ok(ExtinguishResult {
            display_id,
            fire_uid,
            source_atom,
            target_atom,
            reason,
            remaining_open_fire_count: scan.open_fires.len(),
            plan,
        });
    }
    fires[fire_index].status = "extinguished".to_string();
    fires[fire_index].resolution_uid = Some(resolution_uid);

    let fires_before = read_active_json_optional(&context.active_state, Path::new("fires.json"))?;
    let resolutions_before =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?;
    let mut resolutions: Vec<codefire_core::Resolution> = resolutions_before
        .clone()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    if options.refresh {
        for existing in &mut resolutions {
            if existing.fire_uid == fires[fire_index].fire_uid && existing.status == "active" {
                existing.status = "superseded".to_string();
            }
        }
    }
    resolutions.push(resolution);
    let remaining_open_fires = fires.iter().filter(|fire| fire.status == "open").count();
    let target_state =
        scan_branch_state(scan.changed_atoms.len(), remaining_open_fires).to_string();
    let result = ExtinguishResult {
        display_id,
        fire_uid,
        source_atom,
        target_atom,
        reason,
        remaining_open_fire_count: remaining_open_fires,
        plan,
    };
    let durable_result = extinguish_ledger_transition_result(&result, &target_state);
    let durable_result = apply_active_ledger_transition(
        &context.active_state,
        "extinguish",
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
        vec![
            ActiveLedgerWrite::new("fires.json", fires_before, serde_json::to_value(&fires)?),
            ActiveLedgerWrite::new(
                "resolutions.json",
                resolutions_before,
                serde_json::to_value(&resolutions)?,
            ),
        ],
        &durable_result,
    )?;
    let (result, target_state) = extinguish_result_from_ledger_transition(durable_result)?;
    set_open_state_transactional(
        &context,
        "extinguish",
        &target_state,
        ledger_request
            .as_ref()
            .expect("live extinguish must have a ledger request"),
    )?;
    if let Some(key) = options.idempotency_key.as_deref() {
        save_extinguish_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

fn extinguish_ledger_transition_request(
    context: &OpenContext,
    idempotency_payload: &Value,
) -> Result<Value, CliError> {
    Ok(json!({
        "type": "codefire_extinguish_ledger_transition_request",
        "version": 1,
        "branch": &context.branch,
        "open_instance_id": required_string(&context.registry, &["open", "open_instance_id"] )?,
        "current_base_commit": required_string(&context.registry, &["open", "current_base_commit"] )?,
        "request": idempotency_payload,
    }))
}

fn extinguish_ledger_transition_result(result: &ExtinguishResult, target_state: &str) -> Value {
    json!({
        "type": "codefire_extinguish_ledger_transition_result",
        "version": 1,
        "display_id": &result.display_id,
        "fire_uid": &result.fire_uid,
        "source_atom": &result.source_atom,
        "target_atom": &result.target_atom,
        "reason": &result.reason,
        "remaining_open_fire_count": result.remaining_open_fire_count,
        "target_state": target_state,
        "plan": &result.plan,
    })
}

fn extinguish_result_from_ledger_transition(
    value: Value,
) -> Result<(ExtinguishResult, String), CliError> {
    if value.get("type").and_then(Value::as_str)
        != Some("codefire_extinguish_ledger_transition_result")
        || value.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err(CliError::InvalidRepository(
            "active ledger extinguish transaction contains an invalid durable result".to_string(),
        ));
    }
    let required = |field: &str| -> Result<String, CliError> {
        value
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "active ledger extinguish durable result has invalid {field}"
                ))
            })
    };
    let remaining_open_fire_count = value
        .get("remaining_open_fire_count")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| {
            CliError::InvalidRepository(
                "active ledger extinguish durable result has invalid remaining_open_fire_count"
                    .to_string(),
            )
        })?;
    let target_state = required("target_state")?;
    if !matches!(
        target_state.as_str(),
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(CliError::InvalidRepository(format!(
            "active ledger extinguish durable result has unsupported target_state {target_state}"
        )));
    }
    let result = ExtinguishResult {
        display_id: required("display_id")?,
        fire_uid: required("fire_uid")?,
        source_atom: required("source_atom")?,
        target_atom: required("target_atom")?,
        reason: required("reason")?,
        remaining_open_fire_count,
        plan: value.get("plan").cloned().ok_or_else(|| {
            CliError::InvalidRepository(
                "active ledger extinguish durable result is missing plan".to_string(),
            )
        })?,
    };
    Ok((result, target_state))
}

fn extinguish_idempotency_payload(options: &ExtinguishOptions, context: &OpenContext) -> Value {
    json!({
        "command": "extinguish",
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "fire_id": &options.fire_id,
        "resolution": &options.resolution,
        "rationale": &options.rationale,
        "evidence": &options.evidence,
        "evidence_refs": &options.evidence_refs,
        "refresh": options.refresh,
    })
}

fn validate_evidence_refs(repo_root: &Path, refs: &[String]) -> Result<(), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    for evidence_id in refs {
        if evidence_id.trim().is_empty() {
            return Err(CliError::Usage(
                "--evidence-ref requires a non-empty evidence id".to_string(),
            ));
        }
        let payload = codefire_store::read_object(&objects, evidence_id).map_err(|error| {
            if matches!(error, codefire_store::StoreError::ObjectNotFound(_)) {
                CliError::MissingEvidenceRef {
                    evidence_id: evidence_id.clone(),
                    repo_root: repo_root.to_path_buf(),
                }
            } else {
                CliError::Store(Box::new(error))
            }
        })?;
        if payload.get("type").and_then(Value::as_str) != Some("evidence") {
            return Err(CliError::Usage(format!(
                "--evidence-ref must point to an evidence object: {evidence_id}"
            )));
        }
    }
    Ok(())
}

fn collect_missing_evidence_refs(
    repo_root: &Path,
    resolutions: &[codefire_core::Resolution],
) -> Vec<codefire_core::MissingEvidenceRef> {
    let objects = repo_root.join(".codefire").join("objects");
    let mut missing = Vec::new();
    for resolution in resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
    {
        for evidence_id in &resolution.evidence_refs {
            let valid = codefire_store::read_object(&objects, evidence_id)
                .map(|payload| payload.get("type").and_then(Value::as_str) == Some("evidence"))
                .unwrap_or(false);
            if !valid {
                missing.push(codefire_core::MissingEvidenceRef {
                    resolution_uid: resolution.resolution_uid.clone(),
                    evidence_id: evidence_id.clone(),
                });
            }
        }
    }
    missing
}

fn load_extinguish_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<ExtinguishResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "extinguish", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "extinguish", key, payload)?;
    let plan = idempotency_result_plan(&record, &path)?;
    let fire_uid = record
        .pointer("/result/fire_uid")
        .and_then(Value::as_str)
        .or_else(|| plan.pointer("/fire/fire_uid").and_then(Value::as_str))
        .unwrap_or("(unknown)");
    Ok(Some(ExtinguishResult {
        display_id: required_string(&record, &["result", "display_id"])?,
        fire_uid: fire_uid.to_string(),
        source_atom: record
            .pointer("/result/source_atom")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/source_atom").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        target_atom: record
            .pointer("/result/target_atom")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/target_atom").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        reason: record
            .pointer("/result/reason")
            .and_then(Value::as_str)
            .or_else(|| plan.pointer("/fire/reason").and_then(Value::as_str))
            .unwrap_or("(unknown)")
            .to_string(),
        remaining_open_fire_count: record
            .pointer("/result/remaining_open_fire_count")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or_default(),
        plan,
    }))
}

fn save_extinguish_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &ExtinguishResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "extinguish", key);
    let record = json!({
        "version": 1,
        "command": "extinguish",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "display_id": &result.display_id,
            "fire_uid": &result.fire_uid,
            "source_atom": &result.source_atom,
            "target_atom": &result.target_atom,
            "reason": &result.reason,
            "remaining_open_fire_count": result.remaining_open_fire_count,
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

fn run_commit(options: &CommitOptions) -> Result<CommitResult, CliError> {
    let lock;
    let context;
    if options.dry_run {
        lock = None;
        context = open_context(&options.path)?;
    } else {
        let (acquired_lock, locked_context) =
            acquire_locked_open_context_for_recovery(&options.path, &options.lock, "commit")?;
        lock = Some(acquired_lock);
        context = locked_context;
    }
    reject_pending_active_ledger_transition(&context.active_state)?;
    let idempotency_payload = commit_idempotency_payload(options, &context);
    let active_state_path = context.active_state.path().to_path_buf();
    if options.dry_run {
        reject_pending_commit_transaction_for_preview(&context.active_state)?;
    } else if let Some(result) = recover_commit_transaction(
        &context,
        lock.as_ref()
            .expect("live commit must hold repository lock"),
        &context.active_state,
        options,
        &idempotency_payload,
    )? {
        return Ok(result);
    }
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_commit_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    let verify_execution =
        compute_verify_with_cache(&context.open_dir, lock.as_ref(), &mut validation_cache)?;
    let verification = verify_execution.verification;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "after_verify")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "after verification",
    )?;

    let objects = context.repo_root.join(".codefire").join("objects");
    let scan = verify_execution.scan;
    let fires: Vec<codefire_core::Fire> =
        read_active_json_optional(&context.active_state, Path::new("fires.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let extinguished_fires = fires
        .iter()
        .filter(|fire| fire.status == "extinguished")
        .map(|fire| fire.fire_uid.clone())
        .collect::<Vec<_>>();
    let resolutions: Vec<codefire_core::Resolution> =
        read_active_json_optional(&context.active_state, Path::new("resolutions.json"))?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
    let extinguished_fire_count = extinguished_fires.len();
    let active_resolution_count = resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
        .count();
    let evidence_ref_count = resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
        .flat_map(|resolution| resolution.evidence_refs.iter())
        .count();
    let summary = CommitOperationSummary {
        extinguished_fire_count,
        active_resolution_count,
        evidence_ref_count,
    };
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    let policy = &parsed_policy.policy;
    let parents = commit_parents_for_open(&context, &context.active_state)?;
    let plan = commit_operation_plan(
        options,
        &context,
        &active_state_path,
        &scan,
        &verification,
        &parents,
        &summary,
    );
    let decision = codefire_core::committable_decision(&verification);
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "before commit decision publication",
    )?;
    if !decision.is_committable() {
        return Ok(CommitResult {
            repo_root: context.repo_root,
            open_dir: context.open_dir,
            commit_id: String::new(),
            branch: context.branch,
            dry_run: options.dry_run,
            blocked: true,
            exit_code: verification_exit_code(&verification),
            plan,
        });
    }
    if options.dry_run {
        return Ok(CommitResult {
            repo_root: context.repo_root,
            open_dir: context.open_dir,
            commit_id: String::new(),
            branch: context.branch,
            dry_run: true,
            blocked: false,
            exit_code: ExitCode::Success,
            plan,
        });
    }

    let signature_required = policy.require_commit_signature;
    let transaction_id = begin_commit_transaction(
        &context,
        &context.active_state,
        options,
        &parents,
        &idempotency_payload,
        &plan,
    )?;
    let signing_key = signatures::commit_signing_key(signature_required)?;
    let manifest_plan = build_manifest_plan(&context.open_dir)?;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "after_manifest_plan")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "after manifest planning",
    )?;
    let manifest_id = manifest_plan.manifest.object_id.clone();
    let atom_plan = planned_commit_object(
        "atom_index",
        None,
        "atom_index",
        serde_json::to_value(&scan.atom_index)?,
        CommitObjectStage::SealedRoot,
    )?;
    let atom_id = atom_plan.object_id.clone();
    let trace_plan = planned_commit_object(
        "trace_graph",
        None,
        "trace_graph",
        serde_json::to_value(&scan.trace_graph)?,
        CommitObjectStage::SealedRoot,
    )?;
    let trace_id = trace_plan.object_id.clone();
    let fire_plan = planned_commit_object(
        "fire_ledger",
        None,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": fires}),
        CommitObjectStage::SealedRoot,
    )?;
    let fire_id = fire_plan.object_id.clone();
    let resolution_plan = planned_commit_object(
        "resolution_ledger",
        None,
        "resolution_ledger",
        json!({"type": "resolution_ledger", "version": 1, "resolutions": resolutions}),
        CommitObjectStage::SealedRoot,
    )?;
    let resolution_id = resolution_plan.object_id.clone();
    let verification_plan = planned_commit_object(
        "verification",
        None,
        "verification",
        serde_json::to_value(&verification)?,
        CommitObjectStage::SealedRoot,
    )?;
    let verification_id = verification_plan.object_id.clone();
    let policy_plan = planned_commit_object(
        "policy",
        None,
        "policy",
        json!({
            "type": "policy",
            "version": 1,
            "policy": policy,
            "effective_policy": parsed_policy.effective,
        }),
        CommitObjectStage::SealedRoot,
    )?;
    let policy_id = policy_plan.object_id.clone();

    let commit = json!({
        "type": "commit",
        "version": 1,
        "root_schema_version": 2,
        "semantic_validation_profile": codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE,
        "parents": parents,
        "message": options.message,
        "roots": {
            "content_manifest": manifest_id,
            "atom_index": atom_id,
            "trace_graph": trace_id,
            "fire_delta": fire_id,
            "resolution_ledger": resolution_id,
            "verification": verification_id,
            "policy": policy_id,
        },
        "certificate": {
            "result": decision.outcome.as_str(),
            "subject": &verification.subject,
            "profile": "local_verification_v1",
            "phase": "pre_commit",
            "open_required_fires": verification.open_required_fires,
            "failed_checks": verification.failed_checks.len(),
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "missing_evidence_refs": verification.missing_evidence_refs.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
            "effective_policy_digest": verification.effective_policy.as_ref().map(|policy| &policy.digest),
            "change_coverage_digest": verification.change_coverage.as_ref().map(|coverage| &coverage.digest),
        },
        "changed_atoms": scan.changed_atoms,
        "extinguished_fires": extinguished_fires,
        "created_at": now_iso_utc(),
    });
    let commit = signatures::maybe_sign_commit(
        commit,
        options.signer.as_deref(),
        options.key_id.as_deref(),
        signing_key.as_deref(),
    )?;
    let commit = maybe_corrupt_commit_candidate_for_test(&active_state_path, commit);
    let candidate_plan = planned_commit_object(
        "candidate_commit",
        None,
        "commit",
        commit,
        CommitObjectStage::CandidateCommit,
    )?;
    let commit_id = candidate_plan.object_id.clone();
    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["head"] = Value::String(commit_id.clone());
    branch["state"] = Value::String("open-clean".to_string());
    let branch_snapshot_plan = planned_commit_object(
        "branch_snapshot",
        None,
        "branch",
        branch.clone(),
        CommitObjectStage::BranchSnapshot,
    )?;

    // Every immutable role is declared before the first store. Identical file
    // contents may intentionally produce duplicate object IDs; each path keeps
    // its own ordered role entry and therefore its own planned -> stored state.
    let mut planned_objects = manifest_plan.blobs;
    planned_objects.push(manifest_plan.manifest);
    planned_objects.push(atom_plan);
    planned_objects.push(trace_plan);
    planned_objects.push(fire_plan);
    planned_objects.push(resolution_plan);
    planned_objects.push(verification_plan);
    planned_objects.push(policy_plan);
    planned_objects.push(candidate_plan);
    planned_objects.push(branch_snapshot_plan);
    plan_commit_transaction_objects(&context.active_state, &transaction_id, &planned_objects)?;
    let candidate_index = planned_objects
        .iter()
        .position(|planned| planned.role == "candidate_commit")
        .expect("commit plan must contain a candidate");
    for (index, planned) in planned_objects
        .iter_mut()
        .enumerate()
        .take(candidate_index + 1)
    {
        store_planned_commit_object(
            &objects,
            &context.open_dir,
            &context.active_state,
            &transaction_id,
            index,
            planned,
        )?;
    }
    maybe_fail_commit_transaction_for_test(&active_state_path, "before_candidate_validation")?;
    let candidate_commit_path =
        codefire_store::object_record_path(&objects, &commit_id).unwrap_or_else(|| objects.clone());
    let validated_candidate = codefire_store::validate_sealed_commit_pinned_with_cache(
        &objects,
        &commit_id,
        &mut validation_cache,
    )
    .map_err(|error| CliError::StoreContext {
        error: Box::new(error),
        repo_root: context.repo_root.clone(),
        path: candidate_commit_path,
    })?;
    let authenticated_candidate = validated_candidate.read_commit()?;
    signatures::verify_local_commit_signature(
        &authenticated_candidate,
        signing_key.as_deref(),
        signature_required,
    )?;
    for (index, planned) in planned_objects
        .iter_mut()
        .enumerate()
        .skip(candidate_index + 1)
    {
        store_planned_commit_object(
            &objects,
            &context.open_dir,
            &context.active_state,
            &transaction_id,
            index,
            planned,
        )?;
    }
    lock.as_ref()
        .expect("live commit must hold repo lock")
        .ensure_repo_root(&context.repo_root)?;
    update_commit_transaction(&context.active_state, &transaction_id, "publishing_branch")?;
    validated_candidate
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: objects.clone(),
        })?;
    maybe_mutate_commit_worktree_for_test(&context.open_dir, "before_branch_publish")?;
    ensure_verification_subject_matches_worktree(
        &context.open_dir,
        &verification,
        "before branch publication",
    )?;
    write_branch_record_reference(&context.repo_root, &branch)?;
    validated_candidate
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: objects.clone(),
        })?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_branch_publish_before_registry",
    )?;
    let mut registry = context.registry.clone();
    registry["open"]["current_base_commit"] = Value::String(commit_id.clone());
    registry["state"]["last_known"] = Value::String("open-clean".to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_registry_publish_before_phase",
    )?;
    transition_commit_transaction_to_active_reset(&context.active_state, &transaction_id)?;
    reset_commit_active_state(&context.active_state, &transaction_id)?;
    maybe_fail_commit_transaction_for_test(
        &active_state_path,
        "after_active_reset_before_marker_clear",
    )?;
    let result = CommitResult {
        repo_root: context.repo_root.clone(),
        open_dir: context.open_dir.clone(),
        commit_id,
        branch: context.branch,
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        update_commit_transaction(
            &context.active_state,
            &transaction_id,
            "publishing_idempotency",
        )?;
        ensure_commit_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
        maybe_fail_commit_transaction_for_test(
            &active_state_path,
            "after_idempotency_store_before_marker_update",
        )?;
    }
    update_commit_transaction(
        &context.active_state,
        &transaction_id,
        "idempotency_published",
    )?;
    complete_commit_transaction(&context.active_state)?;
    Ok(result)
}

fn commit_idempotency_payload(options: &CommitOptions, context: &OpenContext) -> Value {
    json!({
        "command": "commit",
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "message": &options.message,
        "signer": &options.signer,
        "key_id": &options.key_id,
    })
}

fn load_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
) -> Result<Option<CommitResult>, CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "commit", key);
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    verify_idempotency_record(&record, &path, "commit", key, payload)?;
    Ok(Some(CommitResult {
        repo_root: repo_root.to_path_buf(),
        open_dir: record
            .get("result")
            .and_then(|result| result.get("open_dir"))
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_default(),
        commit_id: required_string(&record, &["result", "commit_id"])?,
        branch: required_string(&record, &["result", "branch"])?,
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan: idempotency_result_plan(&record, &path)?,
    }))
}

fn save_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &CommitResult,
) -> Result<(), CliError> {
    require_idempotency_key(key)?;
    let path = idempotency_record_path(repo_root, "commit", key);
    let record = json!({
        "version": 1,
        "command": "commit",
        "key": key,
        "payload_hash": idempotency_payload_hash(payload)?,
        "payload": payload,
        "result": {
            "commit_id": &result.commit_id,
            "branch": &result.branch,
            "open_dir": &result.open_dir,
            "plan": &result.plan,
        },
        "created_at": now_iso_utc(),
    });
    write_json_atomic(&path, &record)
}

fn ensure_commit_idempotency(
    repo_root: &Path,
    key: &str,
    payload: &Value,
    result: &CommitResult,
) -> Result<(), CliError> {
    let path = idempotency_record_path(repo_root, "commit", key);
    if let Some(existing) = read_optional_json(&path)? {
        verify_idempotency_record(&existing, &path, "commit", key, payload)?;
        let existing_commit = required_string(&existing, &["result", "commit_id"])?;
        let existing_branch = required_string(&existing, &["result", "branch"])?;
        let existing_open_dir = existing
            .pointer("/result/open_dir")
            .and_then(Value::as_str)
            .map(Path::new)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "commit idempotency result is missing open_dir: {}",
                    path.display()
                ))
            })?;
        if existing_commit != result.commit_id
            || existing_branch != result.branch
            || existing_open_dir != result.open_dir
            || existing.pointer("/result/plan") != Some(&result.plan)
        {
            return Err(CliError::IdempotencyConflict(format!(
                "commit idempotency result conflicts with recovered publication: {}",
                path.display()
            )));
        }
        return Ok(());
    }
    save_commit_idempotency(repo_root, key, payload, result)
}

fn extinguish_operation_plan(
    options: &ExtinguishOptions,
    context: &OpenContext,
    active_state_path: &Path,
    fire: &codefire_core::Fire,
    resolution_uid: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "extinguish",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "fire": {
            "display_id": &fire.display_id,
            "fire_uid": &fire.fire_uid,
            "source_atom": &fire.source.atom_id,
            "target_atom": &fire.target.atom_id,
            "reason": &fire.reason,
            "status": &fire.status,
        },
        "resolution": {
            "resolution_uid": resolution_uid,
            "resolution_type": &options.resolution,
            "refresh": options.refresh,
            "has_rationale": !options.rationale.is_empty(),
            "has_evidence": !options.evidence.is_empty() || !options.evidence_refs.is_empty(),
            "evidence_refs": &options.evidence_refs,
        },
        "operations": [
            {"kind": "update_fire_status", "path": active_state_path.join("fires.json"), "status": "extinguished"},
            {"kind": "append_resolution", "path": active_state_path.join("resolutions.json"), "resolution_uid": resolution_uid},
        ],
        "next_actions": [
            ready_verify_details_action(
                &context.branch,
                &context.open_dir,
                Some(&context.repo_root),
            ),
        ],
    })
}

struct CommitOperationSummary {
    extinguished_fire_count: usize,
    active_resolution_count: usize,
    evidence_ref_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitObjectStage {
    ManifestBlob,
    SealedRoot,
    CandidateCommit,
    BranchSnapshot,
}

#[derive(Debug)]
struct PlannedCommitObject {
    role: &'static str,
    path: Option<String>,
    object_type: &'static str,
    object_id: String,
    payload: Option<Value>,
    worktree_source: Option<PathBuf>,
    stage: CommitObjectStage,
}

impl PlannedCommitObject {
    fn marker_value(&self) -> Value {
        json!({
            "role": self.role,
            "path": &self.path,
            "object_type": self.object_type,
            "object_id": &self.object_id,
            "state": "planned",
        })
    }

    fn storing_phase(&self) -> &'static str {
        match self.stage {
            CommitObjectStage::ManifestBlob => "storing_manifest_blobs",
            CommitObjectStage::SealedRoot => "storing_commit_roots",
            CommitObjectStage::CandidateCommit => "storing_candidate_commit",
            CommitObjectStage::BranchSnapshot => "storing_branch_snapshot",
        }
    }

    fn stored_phase(&self) -> &'static str {
        match self.role {
            "manifest_blob" => "stored_manifest_blob",
            "content_manifest" => "stored_content_manifest",
            "atom_index" => "stored_atom_index",
            "trace_graph" => "stored_trace_graph",
            "fire_ledger" => "stored_fire_ledger",
            "resolution_ledger" => "stored_resolution_ledger",
            "verification" => "stored_verification",
            "policy" => "stored_policy",
            "candidate_commit" => "validating_candidate_commit",
            "branch_snapshot" => "stored_branch_snapshot",
            _ => "stored_commit_object",
        }
    }

    fn store_gap_failpoint(&self) -> &'static str {
        match self.stage {
            CommitObjectStage::ManifestBlob => "after_manifest_blob_store_before_marker",
            CommitObjectStage::SealedRoot => "after_root_store_before_marker",
            CommitObjectStage::CandidateCommit => "after_candidate_store_before_marker",
            CommitObjectStage::BranchSnapshot => "after_branch_snapshot_store_before_marker",
        }
    }
}

struct ManifestCommitPlan {
    blobs: Vec<PlannedCommitObject>,
    manifest: PlannedCommitObject,
}

fn commit_operation_plan(
    options: &CommitOptions,
    context: &OpenContext,
    active_state_path: &Path,
    scan: &codefire_core::ScanResult,
    verification: &codefire_core::Verification,
    parents: &[String],
    summary: &CommitOperationSummary,
) -> Value {
    const DEFAULT_COMMIT_CHANGED_ATOM_LIMIT: usize = 50;
    let changed_atom_limit = if options.full_output {
        scan.changed_atoms.len()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let changed_atoms = scan
        .changed_atoms
        .iter()
        .take(changed_atom_limit)
        .cloned()
        .collect::<Vec<_>>();
    let changed_atoms_sample = changed_atoms.clone();
    let changed_atoms_omitted = scan.changed_atoms.len().saturating_sub(changed_atoms.len());
    let non_atom_changed_file_limit = if options.full_output {
        scan.non_atom_changed_files.len()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let non_atom_changed_files = scan
        .non_atom_changed_files
        .iter()
        .take(non_atom_changed_file_limit)
        .cloned()
        .collect::<Vec<_>>();
    let non_atom_changed_files_omitted = scan
        .non_atom_changed_files
        .len()
        .saturating_sub(non_atom_changed_files.len());
    let change_coverage_limit = if options.full_output {
        scan.change_coverage
            .as_ref()
            .map(|coverage| coverage.entries.len())
            .unwrap_or_default()
    } else {
        DEFAULT_COMMIT_CHANGED_ATOM_LIMIT
    };
    let change_coverage =
        change_coverage_data_json(scan.change_coverage.as_ref(), change_coverage_limit);
    let decision = codefire_core::committable_decision(verification);
    let next_actions = if !decision.is_committable() {
        vec![json!({
            "kind": "verify",
            "command": format!("codefire verify --path {} --details --json", context.open_dir.display()),
            "target": {"branch": &context.branch, "open_dir": &context.open_dir, "repo_root": &context.repo_root},
        })]
    } else if options.dry_run {
        vec![json!({
            "kind": "apply_commit",
            "command": format!("codefire commit --path {} -m <message>", context.open_dir.display()),
            "target": {"branch": &context.branch, "open_dir": &context.open_dir, "repo_root": &context.repo_root},
        })]
    } else {
        Vec::new()
    };
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "commit",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "message": &options.message,
        "parents": parents,
        "changed_atom_count": scan.changed_atoms.len(),
        "changed_atoms": changed_atoms,
        "changed_atoms_sample": changed_atoms_sample,
        "changed_atoms_sample_limit": changed_atom_limit,
        "changed_atoms_omitted": changed_atoms_omitted,
        "changed_atoms_truncated": changed_atoms_omitted > 0,
        "non_atom_changed_file_count": scan.non_atom_changed_files.len(),
        "non_atom_changed_files": non_atom_changed_files,
        "non_atom_changed_files_omitted": non_atom_changed_files_omitted,
        "non_atom_changed_files_truncated": non_atom_changed_files_omitted > 0,
        "change_coverage": change_coverage,
        "extinguished_fire_count": summary.extinguished_fire_count,
        "active_resolution_count": summary.active_resolution_count,
        "evidence_ref_count": summary.evidence_ref_count,
        "verification": {
            "result": decision.outcome.as_str(),
            "primary_reason": decision.primary_reason,
            "open_required_fires": verification.open_required_fires,
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
            "failed_checks": verification.failed_checks.len(),
            "effective_policy": &verification.effective_policy,
            "change_coverage": &verification.change_coverage,
        },
        "objects": [
            "blob",
            "content_manifest",
            "atom_index",
            "trace_graph",
            "fire_ledger",
            "resolution_ledger",
            "verification",
            "policy",
            "commit",
            "branch",
        ],
        "transaction": {
            "marker": active_state_path.join("commit_transaction.json"),
            "cleanup": "removed_after_success",
        },
        "operations": [
            {"kind": "begin_commit_transaction", "path": active_state_path.join("commit_transaction.json")},
            {"kind": "store_objects", "path": context.repo_root.join(".codefire").join("objects")},
            {"kind": "update_branch_head", "branch": &context.branch},
            {"kind": "update_open_registry", "path": &context.registry_path},
            {"kind": "reset_active_state", "path": active_state_path},
            {"kind": "clear_commit_transaction", "path": active_state_path.join("commit_transaction.json")},
        ],
        "next_actions": next_actions,
    })
}

fn begin_commit_transaction(
    context: &OpenContext,
    active_state: &active_state::ActiveStateDir,
    options: &CommitOptions,
    parents: &[String],
    request_payload: &Value,
    operation_plan: &Value,
) -> Result<String, CliError> {
    let transaction_path = commit_transaction_path(active_state);
    if active_state.exists(Path::new("commit_transaction.json"))? {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction must be inspected before another commit can start: {}",
            transaction_path.display()
        )));
    }
    let started_at = now_iso_utc();
    let mut nonce = [0u8; 16];
    getrandom::getrandom(&mut nonce).map_err(|error| {
        CliError::InvalidRepository(format!(
            "failed to generate commit transaction identity: {error}"
        ))
    })?;
    let transaction_id = format!(
        "commit_tx_{}",
        nonce
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    let active_reset = build_commit_active_reset_authority(active_state)?;
    write_pending_commit_transaction(
        active_state,
        &json!({
            "version": 1,
            "transaction_id": transaction_id,
            "status": "pending",
            "phase": "started",
            "branch": &context.branch,
            "open_dir": &context.open_dir,
            "parents": parents,
            "message": &options.message,
            "request_payload": request_payload,
            "idempotency_key": &options.idempotency_key,
            "operation_plan": operation_plan,
            "active_reset": active_reset,
            "objects": [],
            "started_at": started_at,
            "updated_at": started_at,
        }),
    )?;
    Ok(transaction_id)
}

fn update_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    phase: &str,
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed during commit".to_string(),
        ));
    }
    record["phase"] = Value::String(phase.to_string());
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

fn plan_commit_transaction_objects(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    planned_objects: &[PlannedCommitObject],
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed while planning immutable objects".to_string(),
        ));
    }
    let existing_objects = record
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("commit transaction objects must be a list".to_string())
        })?;
    if !existing_objects.is_empty() {
        return Err(CliError::InvalidRepository(
            "commit transaction objects were already planned".to_string(),
        ));
    }
    let mut candidate_objects = planned_objects
        .iter()
        .filter(|planned| planned.role == "candidate_commit");
    let candidate = candidate_objects.next().ok_or_else(|| {
        CliError::InvalidRepository(
            "commit transaction plan is missing the candidate commit".to_string(),
        )
    })?;
    if candidate_objects.next().is_some() {
        return Err(CliError::InvalidRepository(
            "commit transaction plan contains multiple candidate commits".to_string(),
        ));
    }
    record["phase"] = Value::String("objects_planned".to_string());
    record["objects"] = Value::Array(
        planned_objects
            .iter()
            .map(PlannedCommitObject::marker_value)
            .collect(),
    );
    record["candidate_commit"] = json!({
        "object_id": &candidate.object_id,
        "state": "planned",
    });
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

fn store_planned_commit_object(
    objects_root: &Path,
    open_dir: &Path,
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    object_index: usize,
    planned: &mut PlannedCommitObject,
) -> Result<(), CliError> {
    update_commit_transaction(active_state, transaction_id, planned.storing_phase())?;
    let payload = match planned.payload.take() {
        Some(payload) => payload,
        None => {
            let source = planned.worktree_source.as_ref().ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "planned {} has neither inline payload nor worktree source",
                    planned.role
                ))
            })?;
            manifest_blob_payload(&fs::read(open_dir.join(source))?)
        }
    };
    let materialized_id = codefire_store::object_id(planned.object_type, &payload)?;
    if materialized_id != planned.object_id {
        return Err(CliError::InvalidRepository(format!(
            "planned {} source changed before storage: planned {}, materialized {materialized_id}",
            planned.role, planned.object_id
        )));
    }
    let stored_id = codefire_store::store_object(objects_root, planned.object_type, payload)?;
    if stored_id != planned.object_id {
        return Err(CliError::InvalidRepository(format!(
            "stored {} id differs from planned object: planned {}, stored {stored_id}",
            planned.role, planned.object_id
        )));
    }
    maybe_fail_commit_transaction_for_test(active_state.path(), planned.store_gap_failpoint())?;
    mark_commit_object_stored(active_state, transaction_id, object_index, planned)?;
    if planned.role == "content_manifest" {
        maybe_fail_commit_transaction_for_test(active_state.path(), "after_content_manifest")?;
    }
    Ok(())
}

fn mark_commit_object_stored(
    active_state: &active_state::ActiveStateDir,
    transaction_id: &str,
    object_index: usize,
    planned: &PlannedCommitObject,
) -> Result<(), CliError> {
    let mut record = read_commit_transaction(active_state)?;
    if required_string(&record, &["transaction_id"])? != transaction_id {
        return Err(CliError::InvalidRepository(
            "commit transaction id changed while recording an immutable object".to_string(),
        ));
    }
    let object = record
        .get_mut("objects")
        .and_then(Value::as_array_mut)
        .and_then(|objects| objects.get_mut(object_index))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "commit transaction object entry {object_index} is missing or invalid"
            ))
        })?;
    let marker_string = |field: &str| {
        object.get(field).and_then(Value::as_str).ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "commit transaction object entry {object_index} must contain string field {field}"
            ))
        })
    };
    if marker_string("role")? != planned.role
        || marker_string("object_type")? != planned.object_type
        || marker_string("object_id")? != planned.object_id
        || marker_string("state")? != "planned"
        || object.get("path").and_then(Value::as_str) != planned.path.as_deref()
    {
        return Err(CliError::InvalidRepository(
            "planned commit object changed before storage completed".to_string(),
        ));
    }
    object.insert("state".to_string(), Value::String("stored".to_string()));
    if planned.role == "candidate_commit" {
        if required_string(&record, &["candidate_commit", "object_id"])? != planned.object_id {
            return Err(CliError::InvalidRepository(
                "planned candidate commit id changed before storage completed".to_string(),
            ));
        }
        record["candidate_commit"]["state"] = Value::String("stored".to_string());
    }
    record["phase"] = Value::String(planned.stored_phase().to_string());
    record["updated_at"] = Value::String(now_iso_utc());
    write_pending_commit_transaction(active_state, &record)
}

fn complete_commit_transaction(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    active_state.remove_file_if_exists(Path::new("commit_transaction.json"))?;
    Ok(())
}

fn commit_transaction_path(active_state: &active_state::ActiveStateDir) -> PathBuf {
    active_state.display_path(Path::new("commit_transaction.json"))
}

pub(crate) const COMMIT_TRANSACTION_BYTE_LIMIT: u64 = 64 * 1024 * 1024;
const COMMIT_TRANSACTION_RECOVERY_RESERVE_BYTES: u64 = 4096;
const COMMIT_TRANSACTION_PENDING_BYTE_LIMIT: u64 =
    COMMIT_TRANSACTION_BYTE_LIMIT - COMMIT_TRANSACTION_RECOVERY_RESERVE_BYTES;

const COMMIT_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "objects_planned",
    "storing_manifest_blobs",
    "stored_manifest_blob",
    "storing_commit_roots",
    "stored_content_manifest",
    "stored_atom_index",
    "stored_trace_graph",
    "stored_fire_ledger",
    "stored_resolution_ledger",
    "stored_verification",
    "stored_policy",
    "storing_candidate_commit",
    "validating_candidate_commit",
    "storing_branch_snapshot",
    "stored_branch_snapshot",
    "publishing_branch",
    "resetting_active_state",
    "publishing_idempotency",
    "idempotency_published",
];

const COMMIT_PREPUBLICATION_PHASES: &[&str] = &[
    "started",
    "objects_planned",
    "storing_manifest_blobs",
    "stored_manifest_blob",
    "storing_commit_roots",
    "stored_content_manifest",
    "stored_atom_index",
    "stored_trace_graph",
    "stored_fire_ledger",
    "stored_resolution_ledger",
    "stored_verification",
    "stored_policy",
    "storing_candidate_commit",
    "validating_candidate_commit",
    "storing_branch_snapshot",
    "stored_branch_snapshot",
    "publishing_branch",
];

pub(crate) fn validate_commit_transaction_record(transaction: &Value) -> Result<(), String> {
    if transaction.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("commit_transaction.json version must be 1".to_string());
    }
    for field in [
        "transaction_id",
        "status",
        "phase",
        "branch",
        "open_dir",
        "message",
        "started_at",
        "updated_at",
    ] {
        if transaction.get(field).and_then(Value::as_str).is_none() {
            return Err(format!(
                "commit_transaction.json must contain string field {field}"
            ));
        }
    }
    let status = transaction["status"]
        .as_str()
        .expect("commit transaction status was checked");
    if status != "pending" {
        return Err(format!(
            "commit_transaction.json status must be pending, found {status}"
        ));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("commit transaction phase was checked");
    if !COMMIT_TRANSACTION_PHASES.contains(&phase) {
        return Err(format!(
            "commit_transaction.json contains unsupported phase: {phase}"
        ));
    }
    if transaction
        .get("idempotency_key")
        .is_some_and(|key| !key.is_null() && !key.is_string())
    {
        return Err("commit_transaction.json idempotency_key must be a string or null".to_string());
    }
    for field in ["request_payload", "operation_plan"] {
        if transaction
            .get(field)
            .is_some_and(|value| !value.is_object())
        {
            return Err(format!(
                "commit_transaction.json {field} must be an object when present"
            ));
        }
    }
    if matches!(
        phase,
        "resetting_active_state" | "publishing_idempotency" | "idempotency_published"
    ) && ["request_payload", "idempotency_key", "operation_plan"]
        .iter()
        .any(|field| transaction.get(*field).is_none())
    {
        return Err(format!(
            "commit_transaction.json phase {phase} requires request_payload, idempotency_key, and operation_plan"
        ));
    }
    if let Some(authority) = transaction.get("active_reset") {
        validate_commit_active_reset_authority(authority)?;
        if matches!(phase, "publishing_idempotency" | "idempotency_published")
            && !commit_active_reset_complete(authority)
        {
            return Err(format!(
                "commit_transaction.json phase {phase} requires a completed active_reset authority"
            ));
        }
        if phase != "resetting_active_state"
            && !matches!(phase, "publishing_idempotency" | "idempotency_published")
            && !commit_active_reset_unstarted(authority)
        {
            return Err(format!(
                "commit_transaction.json phase {phase} cannot contain active_reset progress"
            ));
        }
    }
    let parents = transaction
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| "commit_transaction.json must contain list field parents".to_string())?;
    if parents.iter().any(|parent| !parent.is_string()) {
        return Err("commit_transaction.json parents entries must be strings".to_string());
    }
    let objects = transaction
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| "commit_transaction.json must contain list field objects".to_string())?;
    let mut candidate_entry = None;
    let mut has_legacy_entry = false;
    for (index, entry) in objects.iter().enumerate() {
        if entry.is_string() {
            has_legacy_entry = true;
            continue;
        }
        let object = entry.as_object().ok_or_else(|| {
            format!(
                "commit_transaction.json object entry {index} must be a legacy string or object"
            )
        })?;
        for field in ["role", "object_type", "object_id", "state"] {
            if !object.get(field).is_some_and(Value::is_string) {
                return Err(format!(
                    "commit_transaction.json object entry {index} must contain string field {field}"
                ));
            }
        }
        let state = object["state"]
            .as_str()
            .expect("commit object state was checked");
        if !matches!(state, "planned" | "stored") {
            return Err(format!(
                "commit_transaction.json object entry {index} contains unsupported state: {state}"
            ));
        }
        if object
            .get("path")
            .is_some_and(|path| !path.is_null() && !path.is_string())
        {
            return Err(format!(
                "commit_transaction.json object entry {index} path must be a string or null"
            ));
        }
        if object.get("role").and_then(Value::as_str) == Some("candidate_commit")
            && candidate_entry.replace(entry).is_some()
        {
            return Err(
                "commit_transaction.json contains multiple candidate commit entries".to_string(),
            );
        }
    }
    if objects.is_empty() {
        if phase != "started" || transaction.get("candidate_commit").is_some() {
            return Err(
                "an empty commit transaction plan is valid only at phase started without a candidate"
                    .to_string(),
            );
        }
    } else if !has_legacy_entry {
        let candidate = candidate_entry.ok_or_else(|| {
            "commit_transaction.json object plan is missing candidate_commit".to_string()
        })?;
        let alias = transaction
            .get("candidate_commit")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                "commit_transaction.json object plan is missing candidate alias".to_string()
            })?;
        for field in ["object_id", "state"] {
            if !alias.get(field).is_some_and(Value::is_string) {
                return Err(format!(
                    "commit_transaction.json candidate alias must contain string field {field}"
                ));
            }
        }
        if alias.get("object_id") != candidate.get("object_id")
            || alias.get("state") != candidate.get("state")
        {
            return Err(
                "commit_transaction.json candidate alias does not match its object entry"
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn read_commit_transaction(active_state: &active_state::ActiveStateDir) -> Result<Value, CliError> {
    let bytes = active_state.read_required(
        Path::new("commit_transaction.json"),
        COMMIT_TRANSACTION_BYTE_LIMIT,
    )?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_pending_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    transaction: &Value,
) -> Result<(), CliError> {
    write_active_json_atomic_bounded(
        active_state,
        Path::new("commit_transaction.json"),
        transaction,
        COMMIT_TRANSACTION_PENDING_BYTE_LIMIT,
        "commit transaction marker",
    )
}

fn recover_commit_transaction(
    context: &OpenContext,
    lock: &RepoLock,
    active_state: &active_state::ActiveStateDir,
    options: &CommitOptions,
    requested_payload: &Value,
) -> Result<Option<CommitResult>, CliError> {
    let path = commit_transaction_path(active_state);
    if !active_state.exists(Path::new("commit_transaction.json"))? {
        return Ok(None);
    }
    let transaction = read_commit_transaction(active_state)?;
    validate_commit_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if transaction.get("status").and_then(Value::as_str) != Some("pending") {
        return Err(CliError::InvalidRepository(format!(
            "commit transaction is not pending and cannot be recovered automatically: {}",
            path.display()
        )));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("validated commit transaction phase")
        .to_string();
    let marker_payload = transaction.get("request_payload").cloned();
    let marker_key = match transaction.get("idempotency_key") {
        Some(Value::String(key)) => Some(key.clone()),
        Some(Value::Null) => None,
        _ => {
            return Err(CliError::OpenRecoveryPending(Box::new(
                OpenRecoveryPendingError {
                    operation: "commit".to_string(),
                    phase,
                    repo_root: context.repo_root.clone(),
                    branch: context.branch.clone(),
                    open_dir: context.open_dir.clone(),
                },
            )))
        }
    };
    if marker_payload.as_ref() != Some(requested_payload)
        || marker_key.as_deref() != options.idempotency_key.as_deref()
    {
        return Err(CliError::OpenRecoveryPending(Box::new(
            OpenRecoveryPendingError {
                operation: "commit".to_string(),
                phase,
                repo_root: context.repo_root.clone(),
                branch: context.branch.clone(),
                open_dir: context.open_dir.clone(),
            },
        )));
    }
    let marker_payload = marker_payload.expect("exact commit retry has a request payload");
    let pending_transaction_id = transaction["transaction_id"]
        .as_str()
        .expect("validated commit transaction id")
        .to_string();
    let current_format_entries = transaction["objects"]
        .as_array()
        .expect("validated commit transaction objects")
        .iter()
        .all(Value::is_object);
    let marker_branch = transaction["branch"]
        .as_str()
        .expect("validated commit transaction branch");
    let marker_open_dir = Path::new(
        transaction["open_dir"]
            .as_str()
            .expect("validated commit transaction open_dir"),
    );
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    let branch_head = required_string(&branch, &["head"])?;
    let mut registry = context.registry.clone();
    let mut registry_base = required_string(&registry, &["open", "current_base_commit"])?;
    let marker_parents = transaction["parents"]
        .as_array()
        .expect("validated commit transaction parents")
        .iter()
        .map(|parent| {
            parent
                .as_str()
                .expect("validated commit transaction parent")
                .to_string()
        })
        .collect::<Vec<_>>();
    let current_parents = commit_parents_for_open(context, active_state)?;
    let can_abort_before_retry = COMMIT_PREPUBLICATION_PHASES.contains(&phase.as_str())
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && marker_parents == current_parents
        && branch_head == registry_base
        && current_parents.first() == Some(&branch_head);
    if can_abort_before_retry {
        archive_commit_transaction(
            active_state,
            transaction,
            "aborted",
            "aborted_before_publication",
            &phase,
        )?;
        return Ok(None);
    }

    let postpublication_phase = matches!(
        phase.as_str(),
        "publishing_branch"
            | "resetting_active_state"
            | "publishing_idempotency"
            | "idempotency_published"
    );
    let candidate_id = transaction
        .pointer("/candidate_commit/object_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let candidate_stored = transaction
        .pointer("/candidate_commit/state")
        .and_then(Value::as_str)
        == Some("stored");
    let every_object_stored = transaction["objects"]
        .as_array()
        .expect("validated commit transaction objects")
        .iter()
        .all(|entry| entry.get("state").and_then(Value::as_str) == Some("stored"));
    let objects_root = context.repo_root.join(".codefire").join("objects");
    let candidate_parents_match = match candidate_id.as_deref() {
        Some(candidate) if candidate_stored => {
            commit_object_parents(&objects_root, candidate)? == marker_parents
        }
        _ => false,
    };
    if phase == "publishing_branch"
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && candidate_stored
        && every_object_stored
    {
        let candidate = candidate_id.as_deref().expect("stored candidate has an id");
        let branch_is_candidate = branch_head == candidate;
        let registry_is_parent = marker_parents.first() == Some(&registry_base);
        let registry_is_candidate = registry_base == candidate;
        if branch_is_candidate
            && (registry_is_parent || registry_is_candidate)
            && branch.get("state").and_then(Value::as_str) == Some("open-clean")
            && (!registry_is_candidate
                || registry
                    .pointer("/state/last_known")
                    .and_then(Value::as_str)
                    == Some("open-clean"))
            && candidate_parents_match
            && commit_candidate_matches_open_subject(
                &objects_root,
                candidate,
                context,
                active_state,
                &marker_parents,
            )?
        {
            codefire_store::validate_sealed_commit(&objects_root, candidate).map_err(|error| {
                CliError::StoreContext {
                    error: Box::new(error),
                    repo_root: context.repo_root.clone(),
                    path: codefire_store::object_record_path(&objects_root, candidate)
                        .unwrap_or_else(|| objects_root.clone()),
                }
            })?;
            lock.ensure_repo_root(&context.repo_root)?;
            if registry_is_parent {
                registry["open"]["current_base_commit"] = Value::String(candidate.to_string());
                registry["state"]["last_known"] = Value::String("open-clean".to_string());
                write_json_atomic(&context.registry_path, &registry)?;
                registry_base = candidate.to_string();
            }
            transition_commit_transaction_to_active_reset(active_state, &pending_transaction_id)?;
            reset_commit_active_state(active_state, &pending_transaction_id)?;
        }
    }
    if phase == "resetting_active_state"
        && transaction.get("active_reset").is_some()
        && current_format_entries
        && marker_branch == context.branch
        && marker_open_dir == context.open_dir
        && candidate_stored
        && every_object_stored
    {
        let candidate = candidate_id.as_deref().expect("stored candidate has an id");
        if branch_head == candidate
            && registry_base == candidate
            && branch.get("state").and_then(Value::as_str) == Some("open-clean")
            && registry
                .pointer("/state/last_known")
                .and_then(Value::as_str)
                == Some("open-clean")
            && candidate_parents_match
        {
            codefire_store::validate_sealed_commit(&objects_root, candidate).map_err(|error| {
                CliError::StoreContext {
                    error: Box::new(error),
                    repo_root: context.repo_root.clone(),
                    path: codefire_store::object_record_path(&objects_root, candidate)
                        .unwrap_or_else(|| objects_root.clone()),
                }
            })?;
            lock.ensure_repo_root(&context.repo_root)?;
            reset_commit_active_state(active_state, &pending_transaction_id)?;
        }
    }
    let transaction_after_reset = read_commit_transaction(active_state)?;
    validate_commit_transaction_record(&transaction_after_reset)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if matches!(
        transaction_after_reset.get("phase").and_then(Value::as_str),
        Some("publishing_idempotency" | "idempotency_published")
    ) {
        if let Some(authority) = transaction_after_reset.get("active_reset") {
            preflight_commit_active_reset_authority(active_state, authority)?;
        }
    }
    let durable_reset_progress_complete = transaction_after_reset
        .get("active_reset")
        .map(commit_active_reset_complete)
        .unwrap_or(true);
    let active_state_record = read_active_state_for_commit_recovery(active_state)?;
    let mut reset_files_absent = true;
    for name in [
        "fires.json",
        "resolutions.json",
        "scan.json",
        "verification.json",
        "state_transaction.json",
    ] {
        if active_state.exists(Path::new(name))? {
            reset_files_absent = false;
            break;
        }
    }
    let active_reset_complete = active_state_record.get("state").and_then(Value::as_str)
        == Some("open-clean")
        && reset_files_absent;
    let fully_published = candidate_id
        .as_deref()
        .is_some_and(|candidate| branch_head == candidate && registry_base == candidate)
        && branch.get("state").and_then(Value::as_str) == Some("open-clean")
        && registry
            .pointer("/state/last_known")
            .and_then(Value::as_str)
            == Some("open-clean");
    if !postpublication_phase
        || !current_format_entries
        || marker_branch != context.branch
        || marker_open_dir != context.open_dir
        || !candidate_stored
        || !every_object_stored
        || !candidate_parents_match
        || !durable_reset_progress_complete
        || !active_reset_complete
        || !fully_published
    {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction at phase {phase} requires explicit inspection; branch/open state or transaction subject may be partially published: {}",
            path.display()
        )));
    }
    let candidate_id = candidate_id.expect("fully published transaction has a candidate");
    codefire_store::validate_sealed_commit(&objects_root, &candidate_id).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: codefire_store::object_record_path(&objects_root, &candidate_id)
                .unwrap_or(objects_root),
        }
    })?;
    let operation_plan = transaction.get("operation_plan").cloned().ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "published commit transaction is missing operation_plan: {}",
            path.display()
        ))
    })?;
    let result = CommitResult {
        repo_root: context.repo_root.clone(),
        open_dir: context.open_dir.clone(),
        commit_id: candidate_id,
        branch: context.branch.clone(),
        dry_run: false,
        blocked: false,
        exit_code: ExitCode::Success,
        plan: operation_plan,
    };
    if let Some(key) = marker_key.as_deref() {
        ensure_commit_idempotency(&context.repo_root, key, &marker_payload, &result)?;
    }
    let completed_transaction = read_commit_transaction(active_state)?;
    archive_commit_transaction(
        active_state,
        completed_transaction,
        "recovered",
        "completed_published_commit",
        &phase,
    )?;
    Ok(Some(result))
}

fn read_active_state_for_commit_recovery(
    active_state: &active_state::ActiveStateDir,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(Path::new("state.json"), 1024 * 1024)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn commit_object_parents(objects: &Path, commit_id: &str) -> Result<Vec<String>, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    commit
        .get("parents")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!("commit {commit_id} is missing its parents list"))
        })?
        .iter()
        .map(|parent| {
            parent.as_str().map(str::to_string).ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "commit {commit_id} contains a non-string parent"
                ))
            })
        })
        .collect()
}

fn commit_candidate_matches_open_subject(
    objects: &Path,
    commit_id: &str,
    context: &OpenContext,
    active_state: &active_state::ActiveStateDir,
    expected_parents: &[String],
) -> Result<bool, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let root_matches = |name: &str, object_type: &str, payload: &Value| {
        let expected = required_string(&commit, &["roots", name])?;
        Ok::<bool, CliError>(codefire_store::object_id(object_type, payload)? == expected)
    };

    let active_state_record = read_active_state_for_commit_recovery(active_state)?;
    if active_state_record
        .get("pending_merge_parent")
        .and_then(Value::as_str)
        != expected_parents.get(1).map(String::as_str)
    {
        return Ok(false);
    }

    let manifest = build_manifest_fingerprint(&context.open_dir)?;
    if !root_matches("content_manifest", "content_manifest", &manifest)? {
        return Ok(false);
    }
    let atom_index = codefire_core::build_atom_index(&context.open_dir)?;
    let atom_value = serde_json::to_value(&atom_index)?;
    if !root_matches("atom_index", "atom_index", &atom_value)? {
        return Ok(false);
    }
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    codefire_core::validate_trace_graph_endpoints(&atom_index, &trace_graph)?;
    let trace_value = serde_json::to_value(&trace_graph)?;
    if !root_matches("trace_graph", "trace_graph", &trace_value)? {
        return Ok(false);
    }

    let fires = read_active_json_optional(active_state, Path::new("fires.json"))?
        .unwrap_or_else(|| json!([]));
    if !root_matches(
        "fire_delta",
        "fire_ledger",
        &json!({"type": "fire_ledger", "version": 1, "fires": fires}),
    )? {
        return Ok(false);
    }
    let resolutions = read_active_json_optional(active_state, Path::new("resolutions.json"))?
        .unwrap_or_else(|| json!([]));
    if !root_matches(
        "resolution_ledger",
        "resolution_ledger",
        &json!({"type": "resolution_ledger", "version": 1, "resolutions": resolutions}),
    )? {
        return Ok(false);
    }
    let verification = read_active_json_required(active_state, Path::new("verification.json"))?;
    if !root_matches("verification", "verification", &verification)? {
        return Ok(false);
    }
    let parsed_policy = codefire_core::parse_effective_verification_policy(&context.open_dir)?;
    if !root_matches(
        "policy",
        "policy",
        &json!({
            "type": "policy",
            "version": 1,
            "policy": parsed_policy.policy,
            "effective_policy": parsed_policy.effective,
        }),
    )? {
        return Ok(false);
    }
    Ok(true)
}

fn archive_commit_transaction(
    active_state: &active_state::ActiveStateDir,
    mut transaction: Value,
    status: &str,
    action: &str,
    phase: &str,
) -> Result<(), CliError> {
    let transaction_id = transaction["transaction_id"]
        .as_str()
        .expect("validated commit transaction id");
    let suffix = transaction_id.strip_prefix("commit_tx_").filter(|suffix| {
        matches!(suffix.len(), 12 | 32) && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    let Some(suffix) = suffix else {
        return Err(CliError::InvalidRepository(format!(
            "pending commit transaction uses an unsupported recovery identity: {}",
            commit_transaction_path(active_state).display()
        )));
    };
    let archive_name = format!("commit_transaction.{status}.commit_tx_{suffix}.json");
    let archive_relative = Path::new(&archive_name);
    let archive_path = active_state.display_path(archive_relative);
    transaction["status"] = Value::String(status.to_string());
    transaction["recovery"] = json!({
        "action": action,
        "reason": "safe_retry",
        "phase": phase,
    });
    if active_state.exists(archive_relative)? {
        let bytes = active_state.read_required(archive_relative, COMMIT_TRANSACTION_BYTE_LIMIT)?;
        let existing: Value = serde_json::from_slice(&bytes)?;
        if existing != transaction {
            return Err(CliError::InvalidRepository(format!(
                "commit transaction recovery archive conflicts with the pending marker: {}",
                archive_path.display()
            )));
        }
    } else {
        write_active_json_atomic_bounded(
            active_state,
            archive_relative,
            &transaction,
            COMMIT_TRANSACTION_BYTE_LIMIT,
            "commit transaction recovery archive",
        )?;
    }
    active_state.remove_file_if_exists(Path::new("commit_transaction.json"))?;
    Ok(())
}

fn reject_pending_commit_transaction_for_preview(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    let path = commit_transaction_path(active_state);
    if !active_state.exists(Path::new("commit_transaction.json"))? {
        Ok(())
    } else {
        let transaction = read_commit_transaction(active_state)?;
        validate_commit_transaction_record(&transaction).map_err(|message| {
            CliError::InvalidRepository(format!("{message}: {}", path.display()))
        })?;
        Err(CliError::InvalidRepository(format!(
            "pending commit transaction must be inspected before commit preview; dry-run never recovers or mutates it: {}",
            path.display()
        )))
    }
}

#[cfg(test)]
fn maybe_mutate_commit_worktree_for_test(open_dir: &Path, point: &str) -> Result<(), CliError> {
    let injection = {
        let mut guard = COMMIT_WORKTREE_MUTATION
            .lock()
            .expect("commit worktree mutation lock poisoned");
        guard
            .iter()
            .position(|injection| injection.open_dir == open_dir && injection.point == point)
            .map(|index| guard.swap_remove(index))
    };
    if let Some(injection) = injection {
        fs::write(open_dir.join(injection.relative_path), injection.contents)?;
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_mutate_commit_worktree_for_test(_open_dir: &Path, _point: &str) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_commit_worktree_mutation_for_test(
    open_dir: &Path,
    point: &str,
    relative_path: &Path,
    contents: &[u8],
) {
    let mut guard = COMMIT_WORKTREE_MUTATION
        .lock()
        .expect("commit worktree mutation lock poisoned");
    guard.retain(|injection| injection.open_dir != open_dir || injection.point != point);
    guard.push(CommitWorktreeMutationInjection {
        open_dir: open_dir.to_path_buf(),
        point: point.to_string(),
        relative_path: relative_path.to_path_buf(),
        contents: contents.to_vec(),
    });
}

#[cfg(test)]
fn maybe_fail_commit_transaction_for_test(
    active_state_path: &Path,
    phase: &str,
) -> Result<(), CliError> {
    let mut guard = COMMIT_TRANSACTION_FAILURE_PHASE
        .lock()
        .expect("commit transaction failure phase lock poisoned");
    if let Some(index) = guard.iter().position(|injection| {
        injection.phase == phase && injection.active_state_path == active_state_path
    }) {
        guard.swap_remove(index);
        return Err(CliError::Usage(format!(
            "injected commit transaction failure at {phase}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_commit_transaction_for_test(
    _active_state_path: &Path,
    _phase: &str,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_commit_transaction_failure_for_test(active_state_path: &Path, phase: &str) {
    let mut guard = COMMIT_TRANSACTION_FAILURE_PHASE
        .lock()
        .expect("commit transaction failure phase lock poisoned");
    guard.retain(|injection| {
        injection.active_state_path != active_state_path || injection.phase != phase
    });
    guard.push(CommitTransactionFailureInjection {
        active_state_path: active_state_path.to_path_buf(),
        phase: phase.to_string(),
    });
}

#[cfg(test)]
fn maybe_corrupt_commit_candidate_for_test(active_state_path: &Path, mut commit: Value) -> Value {
    let mut guard = COMMIT_CANDIDATE_CORRUPTION
        .lock()
        .expect("commit candidate corruption lock poisoned");
    let injection = guard
        .as_ref()
        .is_some_and(|injection| injection.active_state_path == active_state_path)
        .then(|| guard.take().expect("matching candidate corruption exists"));
    if let Some(injection) = injection {
        match injection.kind {
            CommitCandidateCorruptionKind::InvalidCertificate => {
                commit["certificate"]["result"] = Value::String("failed".to_string());
            }
            CommitCandidateCorruptionKind::MissingRoot => {
                commit["roots"]
                    .as_object_mut()
                    .expect("commit roots must be an object")
                    .remove("policy");
            }
            CommitCandidateCorruptionKind::WrongRootType => {
                let atom_index = commit["roots"]["atom_index"].clone();
                commit["roots"]["policy"] = atom_index;
            }
            CommitCandidateCorruptionKind::InvalidParent => {
                commit["parents"][0] = Value::String("CF-BLOB-not-a-parent".to_string());
            }
            CommitCandidateCorruptionKind::InvalidSignature => {
                commit["signature"]["signature"] = Value::String("0".repeat(64));
            }
        }
    }
    commit
}

#[cfg(not(test))]
fn maybe_corrupt_commit_candidate_for_test(_active_state_path: &Path, commit: Value) -> Value {
    commit
}

#[cfg(test)]
fn set_commit_candidate_corruption_for_test(
    active_state_path: &Path,
    kind: CommitCandidateCorruptionKind,
) {
    *COMMIT_CANDIDATE_CORRUPTION
        .lock()
        .expect("commit candidate corruption lock poisoned") =
        Some(CommitCandidateCorruptionInjection {
            active_state_path: active_state_path.to_path_buf(),
            kind,
        });
}

#[cfg(test)]
fn maybe_mutate_locked_open_context_for_test(start: &Path) -> Result<(), CliError> {
    let injection = {
        let mut guard = LOCKED_OPEN_CONTEXT_MUTATION
            .lock()
            .expect("locked open context mutation lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.start == start)
        {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        let mut registry = read_json(&injection.registry_path)?;
        let branch_name = required_string(&registry, &["branch", "name"])?;
        registry["open"]["current_base_commit"] = Value::String(injection.base_commit.clone());
        write_json_atomic(&injection.registry_path, &registry)?;
        let repo_root = find_repo_root(start)?;
        let mut branch = load_branch_record(&repo_root, &branch_name)?;
        branch["head"] = Value::String(injection.base_commit);
        save_branch_record(&repo_root, &branch)?;
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_mutate_locked_open_context_for_test(_start: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_locked_open_context_mutation_for_test(
    start: &Path,
    registry_path: &Path,
    base_commit: &str,
) {
    *LOCKED_OPEN_CONTEXT_MUTATION
        .lock()
        .expect("locked open context mutation lock poisoned") = Some(LockedOpenContextMutation {
        start: start.to_path_buf(),
        registry_path: registry_path.to_path_buf(),
        base_commit: base_commit.to_string(),
    });
}

#[cfg(test)]
fn maybe_change_open_target_before_use_for_test(requested_path: &Path) -> Result<(), CliError> {
    let should_change = {
        let mut guard = OPEN_TARGET_PRE_USE_MUTATION
            .lock()
            .expect("open target pre-use mutation lock poisoned");
        if guard.as_ref().is_some_and(|path| path == requested_path) {
            *guard = None;
            true
        } else {
            false
        }
    };
    if should_change {
        fs::create_dir_all(requested_path)?;
        fs::write(requested_path.join("concurrent-owner.txt"), b"occupied\n")?;
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_change_open_target_before_use_for_test(_requested_path: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_open_target_pre_use_mutation_for_test(requested_path: &Path) {
    *OPEN_TARGET_PRE_USE_MUTATION
        .lock()
        .expect("open target pre-use mutation lock poisoned") = Some(requested_path.to_path_buf());
}

#[cfg(all(test, unix))]
fn maybe_insert_nested_open_target_symlink_for_test(
    requested_path: &Path,
    target_path: &Path,
    materialization_path: &Path,
) -> std::io::Result<()> {
    let injection = {
        let mut guard = OPEN_TARGET_NESTED_SYMLINK_INJECTION
            .lock()
            .expect("open target nested symlink injection lock poisoned");
        if guard.as_ref().is_some_and(|injection| {
            injection.requested_path == requested_path && injection.target_path == target_path
        }) {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        fs::create_dir_all(&injection.outside_path)?;
        std::os::unix::fs::symlink(
            &injection.outside_path,
            materialization_path.join(injection.nested_path),
        )?;
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_insert_nested_open_target_symlink_for_test(
    _requested_path: &Path,
    _target_path: &Path,
    _materialization_path: &Path,
) -> std::io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
fn set_open_target_nested_symlink_injection_for_test(
    requested_path: &Path,
    target_path: &Path,
    nested_path: &Path,
    outside_path: &Path,
) {
    *OPEN_TARGET_NESTED_SYMLINK_INJECTION
        .lock()
        .expect("open target nested symlink injection lock poisoned") =
        Some(OpenTargetNestedSymlinkInjection {
            requested_path: requested_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            nested_path: nested_path.to_path_buf(),
            outside_path: outside_path.to_path_buf(),
        });
}

#[cfg(test)]
fn maybe_swap_close_target_before_delete_for_test(target: &Path) -> Result<(), CliError> {
    let injection = {
        let mut guard = CLOSE_TARGET_SWAP_INJECTION
            .lock()
            .expect("close target swap injection lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.target == target)
        {
            guard.take()
        } else {
            None
        }
    };
    if let Some(injection) = injection {
        fs::rename(&injection.target, &injection.moved_original)?;
        fs::create_dir(&injection.target)?;
        fs::write(injection.target.join("victim.txt"), b"victim\n")?;
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_swap_close_target_before_delete_for_test(_target: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_close_target_swap_injection_for_test(target: &Path, moved_original: &Path) {
    *CLOSE_TARGET_SWAP_INJECTION
        .lock()
        .expect("close target swap injection lock poisoned") = Some(CloseTargetSwapInjection {
        target: target.to_path_buf(),
        moved_original: moved_original.to_path_buf(),
    });
}

#[cfg(test)]
fn maybe_fail_state_transaction_for_test(
    active_state_path: &Path,
    operation: &str,
    phase: &str,
) -> Result<(), CliError> {
    let mut guard = STATE_TRANSACTION_FAILURE
        .lock()
        .expect("state transaction failure lock poisoned");
    if let Some(index) = guard.iter().position(|injection| {
        injection.active_state_path == active_state_path
            && injection.operation == operation
            && injection.phase == phase
    }) {
        guard.swap_remove(index);
        return Err(CliError::Usage(format!(
            "injected {operation} state transaction failure at {phase}"
        )));
    }
    Ok(())
}

#[cfg(not(test))]
fn maybe_fail_state_transaction_for_test(
    _active_state_path: &Path,
    _operation: &str,
    _phase: &str,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
fn set_state_transaction_failure_for_test(active_state_path: &Path, operation: &str, phase: &str) {
    let mut guard = STATE_TRANSACTION_FAILURE
        .lock()
        .expect("state transaction failure lock poisoned");
    guard.retain(|injection| {
        injection.active_state_path != active_state_path
            || injection.operation != operation
            || injection.phase != phase
    });
    guard.push(StateTransactionFailureInjection {
        active_state_path: active_state_path.to_path_buf(),
        operation: operation.to_string(),
        phase: phase.to_string(),
    });
}

#[cfg(all(test, unix))]
fn maybe_swap_active_state_after_validation_for_test(
    context: &OpenContext,
) -> Result<(), CliError> {
    use std::os::unix::fs::symlink;

    let injection = {
        let mut guard = ACTIVE_STATE_SWAP_AFTER_VALIDATION
            .lock()
            .expect("active state swap injection lock poisoned");
        if guard.as_ref().is_some_and(|injection| {
            injection.open_dir == context.open_dir
                && injection.active_path == context.active_state.path()
        }) {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };

    fs::rename(&injection.active_path, &injection.moved_path)?;
    if let Err(error) = symlink(&injection.outside_path, &injection.active_path) {
        let _ = fs::rename(&injection.moved_path, &injection.active_path);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_swap_active_state_after_validation_for_test(
    _context: &OpenContext,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
fn set_active_state_swap_after_validation_for_test(
    open_dir: &Path,
    active_path: &Path,
    moved_path: &Path,
    outside_path: &Path,
) {
    *ACTIVE_STATE_SWAP_AFTER_VALIDATION
        .lock()
        .expect("active state swap injection lock poisoned") = Some(ActiveStateSwapInjection {
        open_dir: open_dir.to_path_buf(),
        active_path: active_path.to_path_buf(),
        moved_path: moved_path.to_path_buf(),
        outside_path: outside_path.to_path_buf(),
    });
}

#[cfg(all(test, unix))]
fn maybe_swap_validated_store_before_verify_persist_for_test(
    repo_root: &Path,
) -> Result<(), CliError> {
    let injection = {
        let mut guard = VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST
            .lock()
            .expect("validated store swap injection lock poisoned");
        if guard
            .as_ref()
            .is_some_and(|injection| injection.repo_root == repo_root)
        {
            guard.take()
        } else {
            None
        }
    };
    let Some(injection) = injection else {
        return Ok(());
    };
    fs::rename(&injection.objects_path, &injection.moved_path)?;
    if let Err(error) = fs::rename(&injection.replacement_path, &injection.objects_path) {
        let _ = fs::rename(&injection.moved_path, &injection.objects_path);
        return Err(CliError::Io(error));
    }
    Ok(())
}

#[cfg(not(all(test, unix)))]
fn maybe_swap_validated_store_before_verify_persist_for_test(
    _repo_root: &Path,
) -> Result<(), CliError> {
    Ok(())
}

#[cfg(all(test, unix))]
fn set_validated_store_swap_before_verify_persist_for_test(
    repo_root: &Path,
    objects_path: &Path,
    moved_path: &Path,
    replacement_path: &Path,
) {
    *VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST
        .lock()
        .expect("validated store swap injection lock poisoned") =
        Some(ValidatedStoreSwapInjection {
            repo_root: repo_root.to_path_buf(),
            objects_path: objects_path.to_path_buf(),
            moved_path: moved_path.to_path_buf(),
            replacement_path: replacement_path.to_path_buf(),
        });
}

fn commit_parents_for_open(
    context: &OpenContext,
    active_state: &active_state::ActiveStateDir,
) -> Result<Vec<String>, CliError> {
    let mut parents = vec![required_string(
        &context.registry,
        &["open", "current_base_commit"],
    )?];
    if let Some(state) = read_active_json_optional(active_state, Path::new("state.json"))? {
        if let Some(parent) = state.get("pending_merge_parent").and_then(Value::as_str) {
            parents.push(parent.to_string());
        }
    }
    Ok(parents)
}

struct VerificationCommandBatch {
    failures: Vec<codefire_core::FailedCheck>,
    records: Vec<codefire_core::VerificationExecutionRecord>,
}

fn validate_verification_command_plans(
    open_dir: &Path,
    policy: &codefire_core::VerificationPolicy,
) -> Result<(), CliError> {
    for check in &policy.verification {
        resolve_verification_cwd(open_dir, &check.cwd)?;
        if secret_redaction::redact_text(&check.command).count > 0 {
            return Err(CliError::Usage(format!(
                "verification command '{}' contains secret-like inline metadata; use a secret-free helper and an explicitly inherited environment instead",
                check.id
            )));
        }
        if check
            .env
            .iter()
            .any(|(key, value)| secret_redaction::redact_text(&format!("{key}={value}")).count > 0)
        {
            return Err(CliError::Usage(format!(
                "verification command '{}' contains a secret-like explicit environment entry; policy environment values are persisted and cannot contain secrets",
                check.id
            )));
        }
    }
    Ok(())
}

fn run_verification_commands(
    open_dir: &Path,
    policy: &codefire_core::VerificationPolicy,
    subject: &codefire_core::AssessmentSubject,
) -> Result<VerificationCommandBatch, CliError> {
    let mut failures = Vec::new();
    let mut records = Vec::with_capacity(policy.verification.len());
    ensure_live_subject_matches_worktree(open_dir, subject, "before verification execution")?;
    for check in &policy.verification {
        let worktree_before = ensure_live_subject_matches_worktree(
            open_dir,
            subject,
            "before verification command execution",
        )?;
        let cwd = resolve_verification_cwd(open_dir, &check.cwd)?;
        let mut environment = if check.inherit_env {
            std::env::vars().collect::<BTreeMap<_, _>>()
        } else {
            command_execution::minimal_command_environment()
        };
        environment.extend(check.env.clone());
        let environment_policy = if check.inherit_env {
            "inherited_snapshot_with_explicit_overrides"
        } else {
            "minimal_allowlist_with_explicit_overrides"
        };
        let environment_digest = command_execution::canonical_digest(&json!(&environment))?;
        let input_digest = command_execution::canonical_digest(&json!({
            "type": "codefire_verification_execution_input",
            "version": 1,
            "profile": "trusted_local_shell_v1",
            "phase": "verification_command",
            "subject_digest": &subject.subject_digest,
            "command": &check.command,
            "cwd": &cwd.repo_relative,
            "environment_policy": environment_policy,
            "environment_digest": &environment_digest,
            "timeout_ms": check.timeout_ms,
            "max_output_bytes": check.max_output_bytes,
            "allow_failure": check.allow_failure,
        }))?;
        let output =
            command_execution::run_bounded_command(&command_execution::CommandExecutionSpec {
                program: "sh".to_string(),
                args: vec!["-c".to_string(), check.command.clone()],
                cwd: cwd.resolved.clone(),
                clear_env: true,
                env: environment,
                timeout_ms: check.timeout_ms,
                stdout_limit: check.max_output_bytes,
                stderr_limit: check.max_output_bytes,
            })?;
        let worktree_after = worktree_manifest_digest(open_dir)?;
        let subject_unchanged = worktree_before == worktree_after
            && subject.worktree_digest.as_deref() == Some(worktree_after.as_str());
        let stdout = secret_redaction::redact_text(&output.stdout.text);
        let stderr = secret_redaction::redact_text(&output.stderr.text);
        let command = secret_redaction::redact_text(&check.command);
        let redacted_value_count = command.count + stdout.count + stderr.count;
        let mut combined = String::with_capacity(stdout.value.len() + stderr.value.len() + 160);
        combined.push_str(&stdout.value);
        combined.push_str(&stderr.value);
        if output.stdout.truncated || output.stderr.truncated {
            combined.push_str(&format!(
                "\n[verification output truncated at {} bytes per stream]",
                check.max_output_bytes
            ));
        }
        if output.timed_out {
            combined.push_str(&format!(
                "\nverification command timed out after {}ms",
                check.timeout_ms
            ));
        }
        if !subject_unchanged {
            combined.push_str("\nverification subject changed while the command was running");
        }
        let success = output.success && subject_unchanged;
        let must_block =
            output.timed_out || !subject_unchanged || (!success && !check.allow_failure);
        if must_block {
            failures.push(codefire_core::FailedCheck {
                id: check.id.clone(),
                command: command.value,
                output: combined,
            });
        }
        let explicit_env_keys = check.env.keys().cloned().collect::<Vec<_>>();
        records.push(codefire_core::VerificationExecutionRecord {
            id: check.id.clone(),
            profile: "trusted_local_shell_v1".to_string(),
            phase: "verification_command".to_string(),
            trust_kind: "local_unisolated".to_string(),
            subject_digest: subject.subject_digest.clone(),
            input_digest,
            environment_digest,
            worktree_before,
            worktree_after,
            subject_unchanged,
            cwd: cwd.repo_relative,
            cwd_kind: "repo_relative".to_string(),
            environment_policy: environment_policy.to_string(),
            explicit_env_keys,
            timeout_ms: check.timeout_ms,
            max_output_bytes: check.max_output_bytes,
            stdout_bytes_seen: output.stdout.bytes_seen,
            stdout_bytes_stored: output.stdout.bytes_stored,
            stdout_truncated: output.stdout.truncated,
            stderr_bytes_seen: output.stderr.bytes_seen,
            stderr_bytes_stored: output.stderr.bytes_stored,
            stderr_truncated: output.stderr.truncated,
            exit_code: output.exit_code,
            success,
            timed_out: output.timed_out,
            termination_scope: output.termination_scope.to_string(),
            redacted_value_count,
        });
        if !subject_unchanged {
            break;
        }
    }
    Ok(VerificationCommandBatch { failures, records })
}

struct VerificationCwd {
    resolved: PathBuf,
    repo_relative: String,
}

fn resolve_verification_cwd(open_dir: &Path, requested: &str) -> Result<VerificationCwd, CliError> {
    let root = open_dir.canonicalize()?;
    let candidate = root.join(requested).canonicalize().map_err(|error| {
        CliError::Usage(format!(
            "verification cwd must exist below the open directory: {requested} ({error})"
        ))
    })?;
    if !candidate.is_dir() || !candidate.starts_with(&root) {
        return Err(CliError::Usage(format!(
            "verification cwd escapes the open directory: {requested}"
        )));
    }
    let relative = candidate
        .strip_prefix(&root)
        .expect("candidate checked below root");
    let repo_relative = if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        to_posix(relative)
    };
    Ok(VerificationCwd {
        resolved: candidate,
        repo_relative,
    })
}

fn worktree_manifest_digest(open_dir: &Path) -> Result<String, CliError> {
    let manifest = build_manifest_fingerprint(open_dir)?;
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&manifest)?)
    ))
}

fn ensure_verification_subject_matches_worktree(
    open_dir: &Path,
    verification: &codefire_core::Verification,
    boundary: &str,
) -> Result<String, CliError> {
    let subject = verification.subject.as_ref().ok_or_else(|| {
        CliError::InvalidRepository(
            "current verification is missing its assessment subject".to_string(),
        )
    })?;
    ensure_live_subject_matches_worktree(open_dir, subject, boundary)
}

fn ensure_live_subject_matches_worktree(
    open_dir: &Path,
    subject: &codefire_core::AssessmentSubject,
    boundary: &str,
) -> Result<String, CliError> {
    if subject.kind != "live_open" || subject.trust_kind != "local_unisolated" || subject.immutable
    {
        return Err(CliError::InvalidRepository(
            "local verification carries an incompatible assessment subject".to_string(),
        ));
    }
    let expected = subject.worktree_digest.as_deref().ok_or_else(|| {
        CliError::InvalidRepository(
            "local verification assessment subject is missing its worktree digest".to_string(),
        )
    })?;
    let actual = worktree_manifest_digest(open_dir)?;
    if actual != expected {
        return Err(CliError::InvalidRepository(format!(
            "assessment subject no longer matches the worktree {boundary}; rerun verification"
        )));
    }
    Ok(actual)
}

fn load_base_atom_index(
    objects: &Path,
    base_commit: &str,
) -> Result<codefire_core::AtomIndex, CliError> {
    let validated = codefire_store::validate_sealed_commit_pinned(objects, base_commit)?;
    load_validated_base_atom_index(&validated)
}

fn load_validated_base_atom_index(
    validated: &codefire_store::ValidatedSealedCommit,
) -> Result<codefire_core::AtomIndex, CliError> {
    let atom_index = validated.read_root("atom_index")?;
    Ok(serde_json::from_value(atom_index)?)
}

fn state_transaction_path(active_state: &active_state::ActiveStateDir) -> PathBuf {
    active_state.display_path(Path::new("state_transaction.json"))
}

pub(crate) const STATE_TRANSACTION_BYTE_LIMIT: u64 = 1024 * 1024;

fn begin_state_transaction(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
) -> Result<(), CliError> {
    begin_state_transaction_with_request_hash(context, operation, intended_state, None)
}

fn begin_state_transaction_with_request_hash(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
    request_hash: Option<&str>,
) -> Result<(), CliError> {
    let active_state = &context.active_state;
    if state_transaction_phases(operation).is_none() {
        return Err(CliError::InvalidRepository(format!(
            "unsupported state transaction operation: {operation}"
        )));
    }
    match (active_ledger_state_operation(operation), request_hash) {
        (true, None) => {
            return Err(CliError::InvalidRepository(format!(
            "active-ledger state transaction operation {operation} requires an exact request hash"
        )))
        }
        (false, Some(_)) => {
            return Err(CliError::InvalidRepository(format!(
                "state transaction operation {operation} cannot carry an active-ledger request hash"
            )))
        }
        (_, Some(hash)) if !active_ledger_hash_is_valid(hash) => {
            return Err(CliError::InvalidRepository(format!(
                "active-ledger state transaction operation {operation} has an invalid request hash"
            )))
        }
        _ => {}
    }
    if !matches!(
        intended_state,
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported state transaction target: {intended_state}"
        )));
    }
    let current_registry = read_json(&context.registry_path)?;
    let open_instance_id = required_string(&current_registry, &["open", "open_instance_id"])?;
    let current_base_commit = required_string(&current_registry, &["open", "current_base_commit"])?;
    if active_state.exists(Path::new("state_transaction.json"))? {
        validate_pending_state_transaction_for_recovery(active_state)?;
        let pending = read_state_transaction(active_state)?;
        if pending.get("version").and_then(Value::as_u64) != Some(2)
            || required_string(&pending, &["operation"])? != operation
            || required_string(&pending, &["intended_state"])? != intended_state
            || required_string(&pending, &["subject", "branch"])? != context.branch
            || required_string(&pending, &["subject", "open_instance_id"])? != open_instance_id
            || required_string(&pending, &["subject", "current_base_commit"])?
                != current_base_commit
            || pending.get("request_hash").and_then(Value::as_str) != request_hash
        {
            return Err(CliError::InvalidRepository(format!(
                "pending state transaction does not match the exact {operation} retry subject: {}",
                state_transaction_path(active_state).display()
            )));
        }
        return Ok(());
    }
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    let active = read_active_json_required(active_state, Path::new("state.json"))?;
    let branch_state = required_string(&branch, &["state"])?;
    let registry_state = required_string(&current_registry, &["state", "last_known"])?;
    let active_state_value = required_string(&active, &["state"])?;
    if branch_state != registry_state || branch_state != active_state_value {
        return Err(CliError::InvalidRepository(
            "cannot begin a state transaction from divergent stable authorities".to_string(),
        ));
    }
    let pre_state = branch_state;
    let now = now_iso_utc();
    let mut transaction = json!({
        "version": 2,
        "status": "pending",
        "operation": operation,
        "phase": "started",
        "intended_state": intended_state,
        "subject": {
            "branch": &context.branch,
            "open_instance_id": open_instance_id,
            "current_base_commit": current_base_commit,
        },
        "pre_state": {
            "branch": &pre_state,
            "registry": &pre_state,
            "active": &pre_state,
        },
        "started_at": now,
        "updated_at": now,
    });
    if let Some(request_hash) = request_hash {
        transaction["request_hash"] = Value::String(request_hash.to_string());
    }
    write_active_json_atomic_bounded(
        active_state,
        Path::new("state_transaction.json"),
        &transaction,
        STATE_TRANSACTION_BYTE_LIMIT,
        "state transaction marker",
    )
}

fn validate_pending_state_transaction_for_recovery(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    let path = state_transaction_path(active_state);
    let transaction = read_state_transaction(active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))
}

fn read_state_transaction(active_state: &active_state::ActiveStateDir) -> Result<Value, CliError> {
    let bytes = active_state.read_required(
        Path::new("state_transaction.json"),
        STATE_TRANSACTION_BYTE_LIMIT,
    )?;
    Ok(serde_json::from_slice(&bytes)?)
}

const SCAN_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "verification_invalidated",
    "fires_written",
    "scan_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];
const VERIFY_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "verification_written",
    "registry_written",
    "active_state_written",
    "branch_written",
];
const ACTIVE_LEDGER_STATE_TRANSACTION_PHASES: &[&str] = &[
    "started",
    "registry_written",
    "active_state_written",
    "branch_written",
];

fn active_ledger_state_operation(operation: &str) -> bool {
    matches!(
        operation,
        "fire" | "fire-batch" | "extinguish" | "extinguish-batch"
    )
}

fn state_transaction_phases(operation: &str) -> Option<&'static [&'static str]> {
    match operation {
        "scan" => Some(SCAN_STATE_TRANSACTION_PHASES),
        "verify" => Some(VERIFY_STATE_TRANSACTION_PHASES),
        operation if active_ledger_state_operation(operation) => {
            Some(ACTIVE_LEDGER_STATE_TRANSACTION_PHASES)
        }
        _ => None,
    }
}

pub(crate) fn state_transaction_phase_is_supported(operation: &str, phase: &str) -> bool {
    state_transaction_phases(operation).is_some_and(|phases| phases.contains(&phase))
}

fn state_transaction_transition_is_supported(
    operation: &str,
    current_phase: &str,
    next_phase: &str,
) -> bool {
    state_transaction_phases(operation).is_some_and(|phases| {
        phases
            .windows(2)
            .any(|transition| transition[0] == current_phase && transition[1] == next_phase)
    })
}

pub(crate) fn validate_state_transaction_record(transaction: &Value) -> Result<(), String> {
    let version = transaction
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "state_transaction.json version must be 1 or 2".to_string())?;
    if !matches!(version, 1 | 2) {
        return Err("state_transaction.json version must be 1 or 2".to_string());
    }
    for field in [
        "status",
        "operation",
        "phase",
        "intended_state",
        "started_at",
        "updated_at",
    ] {
        if transaction.get(field).and_then(Value::as_str).is_none() {
            return Err(format!(
                "state_transaction.json must contain string field {field}"
            ));
        }
    }
    if transaction.get("status").and_then(Value::as_str) != Some("pending") {
        return Err("state_transaction.json status must be pending".to_string());
    }
    let operation = transaction["operation"]
        .as_str()
        .expect("operation string was checked above");
    if state_transaction_phases(operation).is_none() {
        return Err(format!(
            "state_transaction.json contains unsupported operation: {operation}"
        ));
    }
    let phase = transaction["phase"]
        .as_str()
        .expect("phase string was checked above");
    if !state_transaction_phase_is_supported(operation, phase) {
        return Err(format!(
            "state_transaction.json phase {phase} is unsupported for operation {operation}"
        ));
    }
    let intended_state = transaction["intended_state"]
        .as_str()
        .expect("intended_state string was checked above");
    if !matches!(
        intended_state,
        "open-clean" | "open-burning" | "open-consistent"
    ) {
        return Err(format!(
            "state_transaction.json contains unsupported intended_state: {intended_state}"
        ));
    }
    if version == 2 {
        for field in [
            "/subject/branch",
            "/subject/open_instance_id",
            "/subject/current_base_commit",
            "/pre_state/branch",
            "/pre_state/registry",
            "/pre_state/active",
        ] {
            if transaction.pointer(field).and_then(Value::as_str).is_none() {
                return Err(format!(
                    "state_transaction.json version 2 must contain string field {field}"
                ));
            }
        }
        let pre_branch = transaction
            .pointer("/pre_state/branch")
            .and_then(Value::as_str)
            .expect("version 2 pre-state branch was checked");
        let pre_registry = transaction
            .pointer("/pre_state/registry")
            .and_then(Value::as_str)
            .expect("version 2 pre-state registry was checked");
        let pre_active = transaction
            .pointer("/pre_state/active")
            .and_then(Value::as_str)
            .expect("version 2 pre-state active was checked");
        if pre_branch != pre_registry || pre_branch != pre_active {
            return Err(
                "state_transaction.json version 2 pre-state authorities must agree".to_string(),
            );
        }
        if !matches!(
            pre_branch,
            "open-clean" | "open-burning" | "open-consistent"
        ) {
            return Err(format!(
                "state_transaction.json contains unsupported pre-state: {pre_branch}"
            ));
        }
        if active_ledger_state_operation(operation) {
            let request_hash = transaction
                .get("request_hash")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "state_transaction.json version 2 operation {operation} must contain string field request_hash"
                    )
                })?;
            if !active_ledger_hash_is_valid(request_hash) {
                return Err(format!(
                    "state_transaction.json version 2 operation {operation} contains invalid request_hash"
                ));
            }
        } else if transaction.get("request_hash").is_some() {
            return Err(format!(
                "state_transaction.json version 2 operation {operation} cannot contain request_hash"
            ));
        }
    }
    Ok(())
}

fn invalidate_active_verification(
    active_state: &active_state::ActiveStateDir,
) -> Result<(), CliError> {
    active_state.remove_file_if_exists(Path::new("verification.json"))?;
    Ok(())
}

fn update_state_transaction_phase(
    active_state: &active_state::ActiveStateDir,
    phase: &str,
) -> Result<(), CliError> {
    let path = state_transaction_path(active_state);
    let mut transaction = read_state_transaction(active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    let operation = transaction["operation"]
        .as_str()
        .expect("validated state transaction operation")
        .to_string();
    let current_phase = transaction["phase"]
        .as_str()
        .expect("validated state transaction phase")
        .to_string();
    let phases = state_transaction_phases(&operation)
        .expect("validated state transaction operation must have phases");
    let current_index = phases
        .iter()
        .position(|candidate| *candidate == current_phase)
        .expect("validated current state transaction phase must be supported");
    let target_index = phases
        .iter()
        .position(|candidate| *candidate == phase)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!(
                "state transaction operation {operation} cannot advance from {current_phase} to {phase}: {}",
                path.display()
            ))
        })?;
    if target_index <= current_index {
        return Ok(());
    }
    if target_index != current_index + 1
        || !state_transaction_transition_is_supported(&operation, &current_phase, phase)
    {
        return Err(CliError::InvalidRepository(format!(
            "state transaction operation {operation} cannot advance from {current_phase} to {phase}: {}",
            path.display()
        )));
    }
    transaction["phase"] = Value::String(phase.to_string());
    transaction["updated_at"] = Value::String(now_iso_utc());
    write_active_json_atomic_bounded(
        active_state,
        Path::new("state_transaction.json"),
        &transaction,
        STATE_TRANSACTION_BYTE_LIMIT,
        "state transaction marker",
    )
}

enum StateTransactionAuthority<'a> {
    Scan {
        scan: &'a codefire_core::ScanResult,
        fires: &'a [codefire_core::Fire],
    },
    Verify {
        scan: &'a codefire_core::ScanResult,
        fires: &'a [codefire_core::Fire],
        verification: &'a codefire_core::Verification,
    },
}

impl StateTransactionAuthority<'_> {
    fn operation(&self) -> &'static str {
        match self {
            Self::Scan { .. } => "scan",
            Self::Verify { .. } => "verify",
        }
    }

    fn scan(&self) -> &codefire_core::ScanResult {
        match self {
            Self::Scan { scan, .. } | Self::Verify { scan, .. } => scan,
        }
    }

    fn fires(&self) -> &[codefire_core::Fire] {
        match self {
            Self::Scan { fires, .. } | Self::Verify { fires, .. } => fires,
        }
    }
}

fn persist_transactional_open_state(
    context: &OpenContext,
    operation: &str,
    state: &str,
    validated_base: &codefire_store::ValidatedSealedCommit,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    if authority.operation() != operation {
        return Err(CliError::InvalidRepository(format!(
            "state transaction authority operation {} does not match {operation}",
            authority.operation()
        )));
    }
    publish_transactional_open_state_fields(context, operation, state)?;
    ensure_validated_store_subject(context, validated_base)?;
    complete_state_transaction(context, state, authority)?;
    ensure_validated_store_subject(context, validated_base)
}

fn publish_transactional_open_state_fields(
    context: &OpenContext,
    operation: &str,
    state: &str,
) -> Result<(), CliError> {
    let mut registry = read_json(&context.registry_path)?;
    registry["state"]["last_known"] = Value::String(state.to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "after_registry_write_before_phase",
    )?;
    update_state_transaction_phase(&context.active_state, "registry_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "registry_written",
    )?;

    let mut active_state_record =
        read_active_json_optional(&context.active_state, Path::new("state.json"))?
            .unwrap_or_else(|| json!({}));
    active_state_record["state"] = Value::String(state.to_string());
    write_active_json_atomic(
        &context.active_state,
        Path::new("state.json"),
        &active_state_record,
    )?;
    update_state_transaction_phase(&context.active_state, "active_state_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "active_state_written",
    )?;

    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["state"] = Value::String(state.to_string());
    save_branch_record(&context.repo_root, &branch)?;
    update_state_transaction_phase(&context.active_state, "branch_written")?;
    maybe_fail_state_transaction_for_test(
        context.active_state.path(),
        operation,
        "branch_written",
    )?;
    Ok(())
}

fn ensure_validated_store_subject(
    context: &OpenContext,
    validated: &codefire_store::ValidatedSealedCommit,
) -> Result<(), CliError> {
    validated
        .ensure_subject_identity()
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: validated.objects_root().to_path_buf(),
        })
}

fn complete_state_transaction(
    context: &OpenContext,
    intended_state: &str,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    let transaction_path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction).map_err(|message| {
        CliError::InvalidRepository(format!("{message}: {}", transaction_path.display()))
    })?;
    let transaction_operation = required_string(&transaction, &["operation"])?;
    if transaction_operation != authority.operation() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction expected authority operation {}, but marker contains {transaction_operation}: {}",
            authority.operation(),
            transaction_path.display()
        )));
    }
    let transaction_phase = required_string(&transaction, &["phase"])?;
    if transaction_phase != "branch_written" {
        return Err(CliError::InvalidRepository(format!(
            "state transaction cannot complete from phase {transaction_phase}: {}",
            transaction_path.display()
        )));
    }
    let marker_state = required_string(&transaction, &["intended_state"])?;
    if marker_state != intended_state {
        return Err(CliError::InvalidRepository(format!(
            "state transaction expected {intended_state}, but marker contains {marker_state}: {}",
            transaction_path.display()
        )));
    }

    let registry = read_json(&context.registry_path)?;
    let active_state_record =
        read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    for (subject, actual) in [
        (
            "open registry",
            required_string(&registry, &["state", "last_known"])?,
        ),
        (
            "active state",
            required_string(&active_state_record, &["state"])?,
        ),
        ("branch", required_string(&branch, &["state"])?),
    ] {
        if actual != intended_state {
            return Err(CliError::InvalidRepository(format!(
                "state transaction expected {intended_state}, but {subject} contains {actual}"
            )));
        }
    }
    validate_state_transaction_authorities(context, intended_state, &registry, &branch, authority)?;
    context
        .active_state
        .remove_file_if_exists(Path::new("state_transaction.json"))?;
    Ok(())
}

fn validate_state_transaction_authorities(
    context: &OpenContext,
    intended_state: &str,
    registry: &Value,
    branch: &Value,
    authority: StateTransactionAuthority<'_>,
) -> Result<(), CliError> {
    let persisted_fires: Vec<codefire_core::Fire> = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("fires.json"))?,
    )?;
    if persisted_fires != authority.fires() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction persisted fires differ from the computed authority: {}",
            context
                .active_state
                .display_path(Path::new("fires.json"))
                .display()
        )));
    }

    let persisted_scan: codefire_core::ScanResult = serde_json::from_value(
        read_active_json_required(&context.active_state, Path::new("scan.json"))?,
    )?;
    if &persisted_scan != authority.scan() {
        return Err(CliError::InvalidRepository(format!(
            "state transaction persisted scan differs from the computed authority: {}",
            context
                .active_state
                .display_path(Path::new("scan.json"))
                .display()
        )));
    }
    let open_fires = persisted_fires
        .iter()
        .filter(|fire| fire.status == "open")
        .cloned()
        .collect::<Vec<_>>();
    if persisted_scan.open_fires != open_fires {
        return Err(CliError::InvalidRepository(
            "state transaction scan.open_fires differs from persisted fires".to_string(),
        ));
    }

    let registry_base = required_string(registry, &["open", "current_base_commit"])?;
    let branch_head = required_string(branch, &["head"])?;
    if persisted_scan.base_commit != registry_base || persisted_scan.base_commit != branch_head {
        return Err(CliError::InvalidRepository(format!(
            "state transaction scan base {}, registry base {registry_base}, and branch head {branch_head} disagree",
            persisted_scan.base_commit
        )));
    }

    let derived_state = match authority {
        StateTransactionAuthority::Scan { .. } => {
            if read_active_json_optional(&context.active_state, Path::new("verification.json"))?
                .is_some()
            {
                return Err(CliError::InvalidRepository(
                    "state transaction scan completed with a stale verification authority"
                        .to_string(),
                ));
            }
            scan_branch_state(
                scan_change_count(&persisted_scan),
                persisted_scan.open_fires.len(),
            )
        }
        StateTransactionAuthority::Verify { verification, .. } => {
            let persisted_verification: codefire_core::Verification = serde_json::from_value(
                read_active_json_required(&context.active_state, Path::new("verification.json"))?,
            )?;
            if &persisted_verification != verification {
                return Err(CliError::InvalidRepository(format!(
                    "state transaction persisted verification differs from the computed authority: {}",
                    context
                        .active_state
                        .display_path(Path::new("verification.json"))
                        .display()
                )));
            }
            let open_required_fires = persisted_scan
                .open_fires
                .iter()
                .filter(|fire| fire.status == "open" && fire.severity == "required")
                .count();
            if persisted_verification.open_required_fires != open_required_fires
                || persisted_verification.duplicate_atom_ids
                    != persisted_scan.atom_index.duplicate_atom_ids
            {
                return Err(CliError::InvalidRepository(
                    "state transaction verification summary differs from the persisted scan"
                        .to_string(),
                ));
            }
            verification_open_state(&persisted_verification, &persisted_scan)
        }
    };
    if derived_state != intended_state {
        return Err(CliError::InvalidRepository(format!(
            "state transaction derived {derived_state} from persisted authorities, expected {intended_state}"
        )));
    }
    Ok(())
}

pub(crate) fn ensure_active_ledger_state_transaction_request(
    context: &OpenContext,
    operation: &str,
    request: &Value,
) -> Result<(), CliError> {
    if !active_ledger_state_operation(operation) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported active-ledger state transaction operation: {operation}"
        )));
    }
    if !context
        .active_state
        .exists(Path::new("state_transaction.json"))?
    {
        return Ok(());
    }
    let path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    let request_hash = active_ledger_request_hash(request)?;
    if transaction.get("version").and_then(Value::as_u64) != Some(2)
        || required_string(&transaction, &["operation"])? != operation
        || required_string(&transaction, &["request_hash"])? != request_hash
    {
        return Err(CliError::IdempotencyConflict(format!(
            "pending state transaction does not belong to the exact {operation}/{request_hash} request: {}",
            path.display()
        )));
    }
    Ok(())
}

pub(crate) fn set_open_state_transactional(
    context: &OpenContext,
    operation: &str,
    state: &str,
    request: &Value,
) -> Result<(), CliError> {
    if !active_ledger_state_operation(operation) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported active-ledger state transaction operation: {operation}"
        )));
    }
    ensure_active_ledger_state_transaction_request(context, operation, request)?;
    ensure_completed_active_ledger_transition(&context.active_state, operation, request)?;
    let request_hash = active_ledger_request_hash(request)?;
    let has_state_transaction = context
        .active_state
        .exists(Path::new("state_transaction.json"))?;
    let descriptor = validate_active_ledger_descriptor(&context.active_state)?.ok_or_else(|| {
        CliError::InvalidRepository(format!(
            "active-ledger state publication lost its completed receipt for {operation}/{request_hash}"
        ))
    })?;
    if !has_state_transaction
        && descriptor.publication_status == ActiveLedgerPublicationStatus::Released
    {
        let registry = read_json(&context.registry_path)?;
        let active = read_active_json_required(&context.active_state, Path::new("state.json"))?;
        let branch = load_branch_record(&context.repo_root, &context.branch)?;
        for (authority, actual) in [
            (
                "registry",
                required_string(&registry, &["state", "last_known"])?,
            ),
            ("active", required_string(&active, &["state"])?),
            ("branch", required_string(&branch, &["state"])?),
        ] {
            if actual != state {
                return Err(CliError::InvalidRepository(format!(
                    "released active-ledger receipt {operation}/{request_hash} expected {state}, but {authority} contains {actual}"
                )));
            }
        }
        return Ok(());
    }
    begin_state_transaction_with_request_hash(context, operation, state, Some(&request_hash))?;
    maybe_fail_state_transaction_for_test(context.active_state.path(), operation, "started")?;
    publish_transactional_open_state_fields(context, operation, state)?;
    complete_active_ledger_state_transaction(context, operation, state, &request_hash)
}

fn complete_active_ledger_state_transaction(
    context: &OpenContext,
    operation: &str,
    intended_state: &str,
    request_hash: &str,
) -> Result<(), CliError> {
    let path = state_transaction_path(&context.active_state);
    let transaction = read_state_transaction(&context.active_state)?;
    validate_state_transaction_record(&transaction)
        .map_err(|message| CliError::InvalidRepository(format!("{message}: {}", path.display())))?;
    if required_string(&transaction, &["operation"])? != operation
        || required_string(&transaction, &["phase"])? != "branch_written"
        || required_string(&transaction, &["intended_state"])? != intended_state
        || required_string(&transaction, &["request_hash"])? != request_hash
    {
        return Err(CliError::InvalidRepository(format!(
            "active-ledger state transaction does not match completed {operation}/{intended_state}/{request_hash} publication: {}",
            path.display()
        )));
    }
    let registry = read_json(&context.registry_path)?;
    let active = read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let branch = load_branch_record(&context.repo_root, &context.branch)?;
    for (authority, actual) in [
        (
            "registry",
            required_string(&registry, &["state", "last_known"])?,
        ),
        ("active", required_string(&active, &["state"])?),
        ("branch", required_string(&branch, &["state"])?),
    ] {
        if actual != intended_state {
            return Err(CliError::InvalidRepository(format!(
                "active-ledger state transaction expected {intended_state}, but {authority} contains {actual}"
            )));
        }
    }
    release_completed_active_ledger_transition(&context.active_state, operation, request_hash)?;
    context
        .active_state
        .remove_file_if_exists(Path::new("state_transaction.json"))?;
    Ok(())
}

fn set_open_state(context: &OpenContext, state: &str) -> Result<(), CliError> {
    let mut registry = context.registry.clone();
    registry["state"]["last_known"] = Value::String(state.to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    let mut active_state_record =
        read_active_json_optional(&context.active_state, Path::new("state.json"))?
            .unwrap_or_else(|| json!({}));
    active_state_record["state"] = Value::String(state.to_string());
    write_active_json_atomic(
        &context.active_state,
        Path::new("state.json"),
        &active_state_record,
    )?;
    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["state"] = Value::String(state.to_string());
    save_branch_record(&context.repo_root, &branch)?;
    Ok(())
}

fn initial_decision_contracts() -> Result<
    (
        codefire_core::EffectivePolicy,
        codefire_core::ChangeCoverage,
    ),
    CliError,
> {
    let policy = codefire_core::default_verification_policy();
    let effective =
        codefire_core::effective_policy(&policy, &codefire_core::PolicyProvenance::default())?;
    let empty_index = codefire_core::AtomIndex {
        type_tag: "atom_index".to_string(),
        version: 1,
        hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
        atoms: Vec::new(),
        duplicate_atom_ids: Vec::new(),
    };
    let coverage =
        codefire_core::build_change_coverage(Vec::new(), &empty_index, &empty_index, &[], None)?;
    Ok((effective, coverage))
}

fn initial_roots(
    objects: &Path,
    now: &str,
    effective_policy: &codefire_core::EffectivePolicy,
    change_coverage: &codefire_core::ChangeCoverage,
) -> Result<Value, CliError> {
    let content_manifest = codefire_store::store_object(
        objects,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": []}),
    )?;
    let atom_index = codefire_store::store_object(
        objects,
        "atom_index",
        json!({"type": "atom_index", "version": 1, "atoms": [], "duplicate_atom_ids": []}),
    )?;
    let trace_graph = codefire_store::store_object(
        objects,
        "trace_graph",
        json!({"type": "trace_graph", "version": 1, "links": []}),
    )?;
    let fire_delta = codefire_store::store_object(
        objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": []}),
    )?;
    let resolution_ledger = codefire_store::store_object(
        objects,
        "resolution_ledger",
        json!({"type": "resolution_ledger", "version": 1, "resolutions": []}),
    )?;
    let verification = codefire_store::store_object(
        objects,
        "verification",
        json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "trace_completeness_required": true,
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "missing_evidence_refs": [],
            "duplicate_atom_ids": [],
            "verified_at": now,
            "effective_policy": effective_policy,
            "change_coverage": change_coverage,
        }),
    )?;
    let policy = codefire_store::store_object(
        objects,
        "policy",
        json!({
            "type": "policy",
            "version": 1,
            "policy": {},
            "effective_policy": effective_policy,
        }),
    )?;
    Ok(json!({
        "content_manifest": content_manifest,
        "atom_index": atom_index,
        "trace_graph": trace_graph,
        "fire_delta": fire_delta,
        "resolution_ledger": resolution_ledger,
        "verification": verification,
        "policy": policy,
    }))
}

fn ensure_repo_layout(repo_root: &Path) -> Result<(), CliError> {
    let cf = repo_root.join(".codefire");
    for path in [
        cf.clone(),
        cf.join("objects"),
        cf.join("branches"),
        cf.join("opened"),
        cf.join("active"),
        cf.join("cache"),
        cf.join("locks"),
        cf.join("remotes"),
        cf.join("idempotency"),
    ] {
        fs::create_dir_all(path)?;
    }
    for subdir in codefire_store::known_object_subdirs() {
        fs::create_dir_all(cf.join("objects").join(subdir))?;
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, CliError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn stable_repo_id(repo_root: &Path) -> String {
    format!(
        "repo_{:012x}",
        stable_hash_48(repo_root.to_string_lossy().as_bytes())
    )
}

fn stable_hash_48(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash & 0x0000_ffff_ffff_ffff
}

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
#[derive(Debug)]
struct CommitWorktreeMutationInjection {
    open_dir: PathBuf,
    point: String,
    relative_path: PathBuf,
    contents: Vec<u8>,
}
#[cfg(test)]
static COMMIT_WORKTREE_MUTATION: Mutex<Vec<CommitWorktreeMutationInjection>> =
    Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
struct CommitTransactionFailureInjection {
    active_state_path: PathBuf,
    phase: String,
}
#[cfg(test)]
static COMMIT_TRANSACTION_FAILURE_PHASE: Mutex<Vec<CommitTransactionFailureInjection>> =
    Mutex::new(Vec::new());
#[cfg(test)]
#[derive(Debug)]
struct CommitCandidateCorruptionInjection {
    active_state_path: PathBuf,
    kind: CommitCandidateCorruptionKind,
}
#[cfg(test)]
#[derive(Debug, Clone, Copy)]
enum CommitCandidateCorruptionKind {
    InvalidCertificate,
    MissingRoot,
    WrongRootType,
    InvalidParent,
    InvalidSignature,
}
#[cfg(test)]
static COMMIT_CANDIDATE_CORRUPTION: Mutex<Option<CommitCandidateCorruptionInjection>> =
    Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
struct LockedOpenContextMutation {
    start: PathBuf,
    registry_path: PathBuf,
    base_commit: String,
}
#[cfg(test)]
static LOCKED_OPEN_CONTEXT_MUTATION: Mutex<Option<LockedOpenContextMutation>> = Mutex::new(None);
#[cfg(test)]
static OPEN_TARGET_PRE_USE_MUTATION: Mutex<Option<PathBuf>> = Mutex::new(None);
#[cfg(all(test, unix))]
#[derive(Debug)]
struct OpenTargetNestedSymlinkInjection {
    requested_path: PathBuf,
    target_path: PathBuf,
    nested_path: PathBuf,
    outside_path: PathBuf,
}
#[cfg(all(test, unix))]
static OPEN_TARGET_NESTED_SYMLINK_INJECTION: Mutex<Option<OpenTargetNestedSymlinkInjection>> =
    Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
struct CloseTargetSwapInjection {
    target: PathBuf,
    moved_original: PathBuf,
}
#[cfg(test)]
static CLOSE_TARGET_SWAP_INJECTION: Mutex<Option<CloseTargetSwapInjection>> = Mutex::new(None);
#[cfg(test)]
#[derive(Debug)]
struct StateTransactionFailureInjection {
    active_state_path: PathBuf,
    operation: String,
    phase: String,
}
#[cfg(test)]
static STATE_TRANSACTION_FAILURE: Mutex<Vec<StateTransactionFailureInjection>> =
    Mutex::new(Vec::new());
#[cfg(all(test, unix))]
#[derive(Debug)]
struct ActiveStateSwapInjection {
    open_dir: PathBuf,
    active_path: PathBuf,
    moved_path: PathBuf,
    outside_path: PathBuf,
}
#[cfg(all(test, unix))]
static ACTIVE_STATE_SWAP_AFTER_VALIDATION: Mutex<Option<ActiveStateSwapInjection>> =
    Mutex::new(None);
#[cfg(all(test, unix))]
#[derive(Debug)]
struct ValidatedStoreSwapInjection {
    repo_root: PathBuf,
    objects_path: PathBuf,
    moved_path: PathBuf,
    replacement_path: PathBuf,
}
#[cfg(all(test, unix))]
static VALIDATED_STORE_SWAP_BEFORE_VERIFY_PERSIST: Mutex<Option<ValidatedStoreSwapInjection>> =
    Mutex::new(None);

pub(crate) const REPOSITORY_JSON_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

fn read_active_json_optional(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
) -> Result<Option<Value>, CliError> {
    active_state
        .read_optional(relative, REPOSITORY_JSON_BYTE_LIMIT)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(CliError::from))
        .transpose()
}

fn read_active_json_required(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
) -> Result<Value, CliError> {
    let bytes = active_state.read_required(relative, REPOSITORY_JSON_BYTE_LIMIT)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_active_json_atomic(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
    value: &Value,
) -> Result<(), CliError> {
    write_active_json_atomic_bounded(
        active_state,
        relative,
        value,
        REPOSITORY_JSON_BYTE_LIMIT,
        "active-state JSON file",
    )
}

fn write_active_json_atomic_bounded(
    active_state: &active_state::ActiveStateDir,
    relative: &Path,
    value: &Value,
    limit: u64,
    context: &str,
) -> Result<(), CliError> {
    let mut rendered = serde_json::to_vec_pretty(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "{context} exceeds the {limit}-byte limit at {}: {} bytes",
            active_state.display_path(relative).display(),
            rendered.len()
        )));
    }
    active_state.write_atomic(relative, &rendered, limit)?;
    Ok(())
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), CliError> {
    write_json_atomic_bounded(
        path,
        value,
        REPOSITORY_JSON_BYTE_LIMIT,
        "repository JSON file",
    )
}

fn write_json_atomic_bounded(
    path: &Path,
    value: &Value,
    limit: u64,
    context: &str,
) -> Result<(), CliError> {
    let mut rendered = serde_json::to_vec_pretty(value)?;
    rendered.push(b'\n');
    if rendered.len() as u64 > limit {
        return Err(CliError::InvalidRepository(format!(
            "{context} exceeds the {limit}-byte limit at {}: {} bytes",
            path.display(),
            rendered.len()
        )));
    }
    write_bytes_atomic(path, &rendered)
}

fn write_bytes_atomic(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::Usage(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let temp_path = unique_temp_path(path);
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
    }
    fs::rename(temp_path, path)?;
    sync_directory_best_effort(parent)?;
    Ok(())
}

fn unique_temp_path(path: &Path) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!("{extension}."))
        .unwrap_or_default();
    path.with_extension(format!("{extension}{}.{}.tmp", std::process::id(), counter))
}

fn sync_directory_best_effort(path: &Path) -> Result<(), CliError> {
    match File::open(path) {
        Ok(dir) => {
            let _ = dir.sync_all();
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(CliError::Io(error)),
    }
}

#[cfg(test)]
fn read_status(start: &Path) -> Result<Status, CliError> {
    let mut validation_cache = codefire_store::SealedCommitValidationCache::default();
    read_status_with_cache(start, &mut validation_cache)
}

fn read_status_with_cache(
    start: &Path,
    validation_cache: &mut codefire_store::SealedCommitValidationCache,
) -> Result<Status, CliError> {
    let context = open_context_for_status(start)?;

    let base = required_string(&context.registry, &["open", "current_base_commit"])?;
    let objects_root = context.repo_root.join(".codefire").join("objects");
    let base_commit_path = codefire_store::object_record_path(&objects_root, &base)
        .unwrap_or_else(|| objects_root.clone());
    codefire_store::validate_sealed_commit_with_cache(&objects_root, &base, validation_cache)
        .map_err(|error| CliError::StoreContext {
            error: Box::new(error),
            repo_root: context.repo_root.clone(),
            path: base_commit_path,
        })?;
    let raw_state = required_string(&context.registry, &["state", "last_known"])?;
    let open_fires = if let Some(fires) =
        read_active_json_optional(&context.active_state, Path::new("fires.json"))?
    {
        fires
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.get("status").and_then(Value::as_str) == Some("open"))
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    Ok(Status {
        branch: context.branch,
        state: status_state(&raw_state, open_fires),
        base,
        open_fires,
    })
}

fn common_ancestor(objects: &Path, left: &str, right: &str) -> Result<Option<String>, CliError> {
    let left_distances = ancestor_distances(objects, left)?;
    nearest_common_ancestor_from_right(objects, right, &left_distances)
}

#[derive(Debug, Clone, Copy)]
struct AncestorInfo {
    distance: usize,
    generation: Option<u64>,
}

fn ancestor_distances(
    objects: &Path,
    commit_id: &str,
) -> Result<HashMap<String, AncestorInfo>, CliError> {
    let mut distances = HashMap::new();
    let mut queue = VecDeque::from([(commit_id.to_string(), 0usize)]);
    while let Some((current, distance)) = queue.pop_front() {
        if distances
            .get(&current)
            .is_some_and(|known: &AncestorInfo| known.distance <= distance)
        {
            continue;
        }
        let commit = commit_info(objects, &current)?;
        distances.insert(
            current,
            AncestorInfo {
                distance,
                generation: commit.generation,
            },
        );
        for parent in commit.parents {
            queue.push_back((parent, distance + 1));
        }
    }
    Ok(distances)
}

fn nearest_common_ancestor_from_right(
    objects: &Path,
    right: &str,
    left_distances: &HashMap<String, AncestorInfo>,
) -> Result<Option<String>, CliError> {
    let mut queue = VecDeque::from([(right.to_string(), 0usize)]);
    let mut seen = HashMap::<String, usize>::new();
    let mut best: Option<(usize, usize, Reverse<u64>, String)> = None;
    while let Some((current, right_distance)) = queue.pop_front() {
        if best
            .as_ref()
            .is_some_and(|(best_sum, _, _, _)| right_distance > *best_sum)
        {
            break;
        }
        if seen
            .get(&current)
            .is_some_and(|known| *known <= right_distance)
        {
            continue;
        }
        seen.insert(current.clone(), right_distance);
        let commit = commit_info(objects, &current)?;
        if let Some(left_info) = left_distances.get(&current) {
            let candidate = (
                left_info.distance + right_distance,
                left_info.distance.max(right_distance),
                Reverse(commit.generation.or(left_info.generation).unwrap_or(0)),
                current.clone(),
            );
            if best.as_ref().map_or(true, |best| candidate < *best) {
                best = Some(candidate);
            }
        }
        for parent in commit.parents {
            queue.push_back((parent, right_distance + 1));
        }
    }
    Ok(best.map(|(_, _, _, commit)| commit))
}

#[derive(Debug)]
struct CommitInfo {
    parents: Vec<String>,
    generation: Option<u64>,
}

fn commit_info(objects: &Path, commit_id: &str) -> Result<CommitInfo, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .map(|parents| {
            parents
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let generation = commit.get("generation").and_then(Value::as_u64);
    Ok(CommitInfo {
        parents,
        generation,
    })
}

fn write_merged_file(
    worktree: &ExistingWorktree,
    rel_path: &str,
    data: Option<&[u8]>,
) -> Result<(), CliError> {
    let path = safe_relative_manifest_path(rel_path)?;
    if let Some(data) = data {
        worktree.write_file_atomic(&path, data)?;
    } else {
        worktree.delete_file(&path)?;
    }
    Ok(())
}

fn conflict_content(target_data: Option<&[u8]>, source_data: Option<&[u8]>) -> Vec<u8> {
    let target = conflict_text(target_data);
    let source = conflict_text(source_data);
    format!("<<<<<<< target\n{target}=======\n{source}>>>>>>> source\n").into_bytes()
}

fn conflict_markers_are_safe(target_data: Option<&[u8]>, source_data: Option<&[u8]>) -> bool {
    let target_safe = target_data.map_or(true, is_text_conflict_side);
    let source_safe = source_data.map_or(true, is_text_conflict_side);
    target_safe && source_safe
}

fn is_text_conflict_side(data: &[u8]) -> bool {
    !data.contains(&0) && std::str::from_utf8(data).is_ok()
}

fn binary_merge_conflict(
    path: &str,
    target_data: Option<&[u8]>,
    source_data: Option<&[u8]>,
) -> BinaryMergeConflict {
    BinaryMergeConflict {
        path: path.to_string(),
        target_path: target_data.map(|_| conflict_side_path(path, "target")),
        source_path: source_data.map(|_| conflict_side_path(path, "source")),
        target_bytes: target_data.map(<[u8]>::len),
        source_bytes: source_data.map(<[u8]>::len),
    }
}

fn conflict_side_path(path: &str, side: &str) -> String {
    format!(".codefire-conflicts/{path}/{side}")
}

fn conflict_text(data: Option<&[u8]>) -> String {
    data.map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(|| "<deleted>\n".to_string())
}

fn optional_repo_root(start: &Path) -> Option<PathBuf> {
    find_repo_root(start).ok()
}

fn validated_active_state_dir(
    repo_root: &Path,
    registry: &Value,
) -> Result<active_state::ActiveStateDir, CliError> {
    let open_instance_id = required_string(registry, &["open", "open_instance_id"])?;
    let recorded = PathBuf::from(required_string(registry, &["open", "active_state_path"])?);
    let expected = repo_root
        .join(".codefire")
        .join("active")
        .join(ref_file_name(&open_instance_id));
    if recorded != expected {
        return Err(CliError::InvalidMarker(format!(
            "active_state_path does not match repository/open instance: expected {}, found {}",
            expected.display(),
            recorded.display()
        )));
    }
    Ok(active_state::ActiveStateDir::bind(&expected)?)
}

fn open_context_for_status(start: &Path) -> Result<OpenContext, CliError> {
    if let Some(context) = legacy_python_status_context(start)? {
        return Ok(context);
    }
    open_context(start)
}

/// The Python fallback historically emitted only marker, opened registry,
/// object store, and active fire data. Keep that exact read-only status shape
/// readable without weakening the current snapshot required by mutations.
fn legacy_python_status_context(start: &Path) -> Result<Option<OpenContext>, CliError> {
    let Some(marker_path) = find_open_marker(start)? else {
        return Ok(None);
    };
    let marker = read_json(&marker_path)?;
    let open_dir = marker_path
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("marker has no parent directory".to_string()))?
        .to_path_buf();
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repo_root = repo_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()))?;
    let branch = required_string(&marker, &["branch", "name"])?;
    let registry_path = opened_registry_path(&repo_root, &branch);
    let registry = read_json(&registry_path)?;
    let active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let is_exact_legacy_shape = !repo_root.join(".codefire").join("repo.json").exists()
        && !branch_record_path(&repo_root, &branch).exists()
        && !active_state_path.join("state.json").exists();
    if !is_exact_legacy_shape {
        return Ok(None);
    }

    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    if registry_open_path != open_dir {
        return Err(CliError::InvalidMarker(
            "opened_path does not match registry".to_string(),
        ));
    }
    let marker_open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    if marker_open_instance_id != registry_open_instance_id {
        return Err(CliError::InvalidMarker(
            "open_instance_id does not match registry".to_string(),
        ));
    }
    let active_state = validated_active_state_dir(&repo_root, &registry)?;
    Ok(Some(OpenContext {
        repo_root,
        open_dir,
        branch,
        registry_path,
        active_state,
        registry,
        authority: OpenContextAuthority::LegacyPythonStatus,
    }))
}

fn open_context(start: &Path) -> Result<OpenContext, CliError> {
    open_context_with_recovery(start, None)
}

fn open_context_for_recovery(start: &Path, operation: &str) -> Result<OpenContext, CliError> {
    open_context_with_recovery(start, Some(operation))
}

fn open_context_with_recovery(
    start: &Path,
    allowed_operation: Option<&str>,
) -> Result<OpenContext, CliError> {
    if legacy_python_status_context(start)?.is_some() {
        return Err(CliError::InvalidRepository(
            "legacy Python-compatible open metadata is read-only; use status or migrate before mutation"
                .to_string(),
        ));
    }
    let context = load_verified_open_context(start)?;
    let pending = match &context.authority {
        OpenContextAuthority::Verified(snapshot) => match &snapshot.disposition {
            open_snapshot::OpenSnapshotDisposition::Steady => None,
            open_snapshot::OpenSnapshotDisposition::PendingCommit { phase } => {
                Some(("commit".to_string(), phase.clone()))
            }
            open_snapshot::OpenSnapshotDisposition::PendingState { operation, phase } => {
                Some((operation.clone(), phase.clone()))
            }
            open_snapshot::OpenSnapshotDisposition::PendingActiveLedger {
                operation,
                phase,
                ..
            } => Some((operation.clone(), phase.clone())),
        },
        OpenContextAuthority::LegacyPythonStatus => None,
    };
    let Some((operation, phase)) = pending else {
        return Ok(context);
    };
    if allowed_operation == Some(operation.as_str()) {
        return Ok(context);
    }
    Err(CliError::OpenRecoveryPending(Box::new(
        OpenRecoveryPendingError {
            operation,
            phase,
            repo_root: context.repo_root,
            branch: context.branch,
            open_dir: context.open_dir,
        },
    )))
}

fn load_verified_open_context(start: &Path) -> Result<OpenContext, CliError> {
    let marker_path =
        find_open_marker(start)?.ok_or_else(|| CliError::NotOpen(start.to_path_buf()))?;
    let marker = read_json(&marker_path)?;
    let open_dir = marker_path
        .parent()
        .ok_or_else(|| CliError::InvalidMarker("marker has no parent directory".to_string()))?
        .to_path_buf();
    let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repo_root = repo_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()))?;
    let branch = required_string(&marker, &["branch", "name"])?;
    let repository_record_path = repo_root.join(".codefire").join("repo.json");
    let repository = read_json(&repository_record_path)?;
    let branch_record_path = branch_record_path(&repo_root, &branch);
    let branch_record = read_json(&branch_record_path)?;
    let registry_path = opened_registry_path(&repo_root, &branch);
    let registry = read_json(&registry_path)?;
    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let marker_open_path = PathBuf::from(required_string(&marker, &["open", "opened_path"])?);
    let marker_open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    let registry_active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let expected_active_state_path = repo_root
        .join(".codefire")
        .join("active")
        .join(ref_file_name(&marker_open_instance_id));
    let expected_repository_path = repo_root.join(".codefire");
    let marker_repository_path = PathBuf::from(required_string(&marker, &["repository", "path"])?);
    let repository_id = required_string(&repository, &["repository_id"])?;
    let marker_repository_id = required_string(&marker, &["repository", "repository_id"])?;
    let marker_branch = required_string(&marker, &["branch", "name"])?;
    let registry_branch = required_string(&registry, &["branch", "name"])?;
    let branch_record_name = required_string(&branch_record, &["name"])?;
    let marker_opened_from_commit = required_string(&marker, &["branch", "opened_from_commit"])?;
    let registry_opened_from_commit = required_string(&registry, &["open", "opened_from_commit"])?;
    let branch_head = required_string(&branch_record, &["head"])?;
    let registry_current_base = required_string(&registry, &["open", "current_base_commit"])?;

    let locator = open_snapshot::OpenSnapshotIdentityAuthorities {
        repo_root: &repo_root,
        repository_record_path: &repository_record_path,
        marker_path: &marker_path,
        branch_record_path: &branch_record_path,
        registry_path: &registry_path,
        expected_repository_path: &expected_repository_path,
        marker_repository_path: &marker_repository_path,
        repository_id: &repository_id,
        marker_repository_id: &marker_repository_id,
        branch: &branch,
        marker_branch: &marker_branch,
        registry_branch: &registry_branch,
        branch_record_name: &branch_record_name,
        open_dir: &open_dir,
        marker_open_path: &marker_open_path,
        registry_open_path: &registry_open_path,
        marker_open_instance_id: &marker_open_instance_id,
        registry_open_instance_id: &registry_open_instance_id,
        marker_opened_from_commit: &marker_opened_from_commit,
        registry_opened_from_commit: &registry_opened_from_commit,
        expected_active_state_path: &expected_active_state_path,
        registry_active_state_path: &registry_active_state_path,
        branch_head: &branch_head,
        registry_current_base: &registry_current_base,
        pending_commit_transaction: None,
    };
    open_snapshot::verify_open_snapshot_locator(&locator)
        .map_err(CliError::OpenSnapshotDivergence)?;
    validate_open_snapshot_commit(&repo_root, &branch_head)?;
    if registry_current_base != branch_head {
        validate_open_snapshot_commit(&repo_root, &registry_current_base)?;
    }

    let active_state = validated_active_state_dir(&repo_root, &registry)?;
    let active_state_record_path = active_state.display_path(Path::new("state.json"));
    let active_state_record = read_active_json_required(&active_state, Path::new("state.json"))?;
    let pending_commit_transaction =
        read_active_json_optional(&active_state, Path::new("commit_transaction.json"))?;
    let pending_state_transaction =
        read_active_json_optional(&active_state, Path::new("state_transaction.json"))?;
    let active_ledger_descriptor = validate_active_ledger_descriptor(&active_state)?;
    if let Some(transaction) = pending_commit_transaction.as_ref() {
        validate_commit_transaction_record(transaction).map_err(|message| {
            CliError::InvalidRepository(format!(
                "{message}: {}",
                active_state
                    .display_path(Path::new("commit_transaction.json"))
                    .display()
            ))
        })?;
    }
    if let Some(transaction) = pending_state_transaction.as_ref() {
        validate_state_transaction_record(transaction).map_err(|message| {
            CliError::InvalidRepository(format!(
                "{message}: {}",
                active_state
                    .display_path(Path::new("state_transaction.json"))
                    .display()
            ))
        })?;
    }
    let branch_state = required_string(&branch_record, &["state"])?;
    let registry_state = required_string(&registry, &["state", "last_known"])?;
    let active_state_value = required_string(&active_state_record, &["state"])?;
    let identity = open_snapshot::OpenSnapshotIdentityAuthorities {
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        ..locator
    };
    let state = open_snapshot::OpenSnapshotStateAuthorities {
        active_state_record_path: &active_state_record_path,
        branch_state: &branch_state,
        registry_state: &registry_state,
        active_state: &active_state_value,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        pending_state_transaction: pending_state_transaction.as_ref(),
        active_ledger_descriptor: active_ledger_descriptor.as_ref(),
    };
    let verified_snapshot = open_snapshot::verify_open_snapshot(&identity, &state)
        .map_err(CliError::OpenSnapshotDivergence)?;
    Ok(OpenContext {
        repo_root: verified_snapshot.repo_root.clone(),
        open_dir: verified_snapshot.open_dir.clone(),
        branch: verified_snapshot.branch.clone(),
        registry_path,
        active_state,
        registry,
        authority: OpenContextAuthority::Verified(Box::new(verified_snapshot)),
    })
}

fn validate_open_snapshot_commit(repo_root: &Path, commit_id: &str) -> Result<(), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    let path =
        codefire_store::object_record_path(&objects, commit_id).unwrap_or_else(|| objects.clone());
    codefire_store::validate_sealed_commit(&objects, commit_id).map_err(|error| {
        CliError::StoreContext {
            error: Box::new(error),
            repo_root: repo_root.to_path_buf(),
            path,
        }
    })
}

#[derive(Debug)]
#[must_use = "branch-list results own an unfinished command ledger and must be completed"]
struct BranchListResult {
    repo_root: PathBuf,
    metadata_snapshot: RepositoryMetadataSnapshot,
    branches: Vec<Branch>,
    opened: Vec<Value>,
    store_work: codefire_store::SealedValidationWork,
    ref_work: BranchRefWork,
    opened_work: BranchRefWork,
    opened_snapshot: Option<OpenedRegistrySnapshot>,
    ledger: codefire_core::DecisionWorkLedger,
    retained_bytes: u64,
}

impl BranchListResult {
    fn decision_work_snapshot(&self) -> &codefire_core::DecisionWorkSnapshot {
        self.ledger.snapshot()
    }

    fn prepare_opened_registries(mut self) -> Result<Self, CliError> {
        if let Err(error) = self.metadata_snapshot.ensure_current() {
            let error = branch_list_ref_error(error);
            let reason = branch_list_cli_terminal_reason(&error);
            self.abort_after_branch_list_failure(reason);
            return Err(error);
        }
        let opened_root = self.repo_root.join(".codefire").join("opened");
        let snapshot = match OpenedRegistrySnapshot::capture(
            &opened_root,
            REPOSITORY_JSON_BYTE_LIMIT,
            &mut self.ledger,
        ) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let error = opened_registry_ref_error(error);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        if snapshot.parent_identity() != self.metadata_snapshot.identity() {
            let _ = snapshot.release(&mut self.ledger);
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::SubjectChanged,
            );
            return Err(CliError::OpenedRegistrySubjectChanged {
                subject: opened_root,
                message: "repository metadata identity changed between branch and opened capture"
                    .to_string(),
            });
        }
        let projection = project_opened_registries(&snapshot, &mut self.ledger);
        match projection {
            Ok((opened, retained_bytes)) => {
                let Some(total_retained) = self.retained_bytes.checked_add(retained_bytes) else {
                    let _ = self.ledger.release(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        retained_bytes,
                    );
                    let _ = snapshot.release(&mut self.ledger);
                    self.abort_after_branch_list_failure(
                        codefire_core::DecisionWorkTerminalReason::CorruptSubject,
                    );
                    return Err(CliError::InvalidRepository(
                        "opened registry projection size overflow".to_string(),
                    ));
                };
                self.retained_bytes = total_retained;
                self.opened = opened;
                self.opened_snapshot = Some(snapshot);
                Ok(self)
            }
            Err(error) => {
                let _ = snapshot.release(&mut self.ledger);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                Err(error)
            }
        }
    }

    fn revalidate_opened_registries(&mut self) -> Result<(), CliError> {
        let Some(snapshot) = self.opened_snapshot.take() else {
            return Ok(());
        };
        match snapshot.revalidate(REPOSITORY_JSON_BYTE_LIMIT, &mut self.ledger) {
            Ok(work) => {
                self.opened_work = work;
                Ok(())
            }
            Err(error) => {
                let error = opened_registry_ref_error(error);
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                Err(error)
            }
        }
    }

    fn output_branch_list(
        mut self,
        json_output: bool,
        metrics: Option<CommandMetrics>,
    ) -> Result<(), CliError> {
        let encoded = if json_output {
            let data = BranchListDataWire {
                type_tag: "codefire_branch_list",
                version: 1,
                branches: BranchesWire(&self.branches),
                metrics: metrics.as_ref().map(metrics_wire),
            };
            let envelope = CommandResultWire {
                schema: automation::COMMAND_RESULT_SCHEMA,
                command: "branch-list",
                ok: true,
                exit_code: 0,
                repo: Some(&self.repo_root),
                data,
                diagnostics: &[],
                next_actions: &[],
            };
            encode_pretty_json_output(&mut self.ledger, &envelope)
        } else {
            encode_text_output(&mut self.ledger, |writer| {
                write_branches(writer, &self.branches)?;
                if let Some(metrics) = metrics.as_ref() {
                    write_metrics(writer, metrics)?;
                }
                Ok(())
            })
        };
        let (bytes, retained_bytes) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => {
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        self.retained_bytes = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| CliError::InvalidRepository("output size overflow".to_string()))?;
        self.publish_and_finish(bytes)
    }

    fn output_local_list(mut self, json_output: bool) -> Result<(), CliError> {
        let encoded = if json_output {
            let data = LocalListDataWire {
                type_tag: "codefire_local_list",
                version: 1,
                branches: BranchesWire(&self.branches),
                opened: &self.opened,
            };
            let envelope = CommandResultWire {
                schema: automation::COMMAND_RESULT_SCHEMA,
                command: "list",
                ok: true,
                exit_code: 0,
                repo: Some(&self.repo_root),
                data,
                diagnostics: &[],
                next_actions: &[],
            };
            encode_pretty_json_output(&mut self.ledger, &envelope)
        } else {
            encode_text_output(&mut self.ledger, |writer| {
                writeln!(writer, "Branches:")?;
                write_branches(writer, &self.branches)?;
                writeln!(writer, "Opened:")?;
                if self.opened.is_empty() {
                    writeln!(writer, "none")?;
                } else {
                    for item in &self.opened {
                        writeln!(
                            writer,
                            "{}\t{}",
                            item.get("branch")
                                .and_then(Value::as_str)
                                .unwrap_or("(unknown)"),
                            item.get("path").and_then(Value::as_str).unwrap_or("")
                        )?;
                    }
                }
                Ok(())
            })
        };
        let (bytes, retained_bytes) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => {
                let reason = branch_list_cli_terminal_reason(&error);
                self.abort_after_branch_list_failure(reason);
                return Err(error);
            }
        };
        self.retained_bytes = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or_else(|| CliError::InvalidRepository("output size overflow".to_string()))?;
        if let Err(error) = self.revalidate_opened_registries() {
            drop(bytes);
            return Err(error);
        }
        self.publish_and_finish(bytes)
    }

    fn publish_and_finish(mut self, bytes: Vec<u8>) -> Result<(), CliError> {
        if let Err(error) = self.metadata_snapshot.ensure_current() {
            drop(bytes);
            let error = branch_list_ref_error(error);
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::SubjectChanged,
            );
            return Err(error);
        }
        let write_result = std::io::stdout().lock().write_all(&bytes);
        drop(bytes);
        if let Err(error) = write_result {
            self.abort_after_branch_list_failure(
                codefire_core::DecisionWorkTerminalReason::CorruptSubject,
            );
            return Err(CliError::Io(error));
        }
        self.finish_complete().map(drop)
    }

    fn abort_after_branch_list_failure(
        &mut self,
        reason: codefire_core::DecisionWorkTerminalReason,
    ) {
        if let Some(snapshot) = self.opened_snapshot.take() {
            let _ = snapshot.release(&mut self.ledger);
        }
        if self.retained_bytes > 0 {
            let _ = self.ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            );
            self.retained_bytes = 0;
        }
        if self.ledger.snapshot().terminal_reason.is_none() {
            let _ = self.ledger.finish(reason);
        }
    }

    fn finish_complete(mut self) -> Result<codefire_core::DecisionWorkSnapshot, CliError> {
        self.ledger
            .release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                self.retained_bytes,
            )
            .map_err(branch_list_accounting_error)?;
        self.retained_bytes = 0;
        self.ledger
            .finish(codefire_core::DecisionWorkTerminalReason::Complete)
            .map_err(branch_list_accounting_error)?;
        Ok(self.ledger.snapshot().clone())
    }
}

fn write_branches<W: Write + ?Sized>(writer: &mut W, branches: &[Branch]) -> std::io::Result<()> {
    for branch in branches {
        writeln!(writer, "{}\t{}\t{}", branch.name, branch.head, branch.state)?;
    }
    Ok(())
}

fn list_branches(start: &Path) -> Result<BranchListResult, CliError> {
    let ledger = codefire_core::DecisionWorkLedger::new(
        codefire_core::DecisionWorkProfile::local_v1(),
        codefire_core::DecisionWorkCommandKind::LocalGraph,
        "branch-list",
    )
    .map_err(branch_list_accounting_error)?;
    list_branches_with_ledger_and_hook(start, ledger, |_| Ok(()))
}

fn list_branches_with_ledger_and_hook<F>(
    start: &Path,
    mut ledger: codefire_core::DecisionWorkLedger,
    before_finalize: F,
) -> Result<BranchListResult, CliError>
where
    F: FnOnce(&Path) -> Result<(), CliError>,
{
    let repo_root = match find_repo_root(start) {
        Ok(repo_root) => repo_root,
        Err(error) => {
            if ledger.snapshot().terminal_reason.is_none() {
                let _ = ledger.finish(branch_list_cli_terminal_reason(&error));
            }
            return Err(error);
        }
    };
    let branches_root = repo_root.join(".codefire").join("branches");
    let objects_root = repo_root.join(".codefire").join("objects");
    let metadata_snapshot = match RepositoryMetadataSnapshot::capture(&repo_root.join(".codefire"))
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    let branch_snapshot = match metadata_snapshot.capture_branch_refs(
        &branches_root,
        REPOSITORY_JSON_BYTE_LIMIT,
        &mut ledger,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    let mut validation =
        match codefire_store::begin_sealed_commit_validation_session_with_recoverable_ledger(
            &objects_root,
            ledger,
        ) {
            Ok(validation) => validation,
            Err(begin_error) => {
                let (error, mut ledger) = begin_error.into_parts();
                branch_snapshot
                    .release(&mut ledger)
                    .map_err(branch_list_ref_error)?;
                finish_branch_list_store_failure(&mut ledger, &error);
                return Err(branch_list_store_error(error));
            }
        };
    let mut validation_error = None;
    for document in branch_snapshot.documents() {
        if let Err(error) = validation.validate_commit(document.head()) {
            validation_error = Some(error);
            break;
        }
    }
    if let Some(error) = validation_error {
        let reason = branch_list_store_terminal_reason(&error);
        let mut ledger = validation
            .abort_and_take_decision_work_ledger(reason)
            .map_err(branch_list_store_error)?
            .expect("branch-list validation always owns a decision-work ledger");
        branch_snapshot
            .release(&mut ledger)
            .map_err(branch_list_ref_error)?;
        return Err(branch_list_store_error(error));
    }
    if let Err(error) = before_finalize(&branches_root) {
        let reason = branch_list_cli_terminal_reason(&error);
        let mut ledger = validation
            .abort_and_take_decision_work_ledger(reason)
            .map_err(branch_list_store_error)?
            .expect("branch-list validation always owns a decision-work ledger");
        branch_snapshot
            .release(&mut ledger)
            .map_err(branch_list_ref_error)?;
        return Err(error);
    }
    let store_work = match validation.finalize() {
        Ok(work) => work,
        Err(error) => {
            let mut ledger = validation
                .take_decision_work_ledger()
                .expect("finalized branch-list validation can return its ledger")
                .expect("branch-list validation always owns a decision-work ledger");
            branch_snapshot
                .release(&mut ledger)
                .map_err(branch_list_ref_error)?;
            return Err(branch_list_store_error(error));
        }
    };
    let mut ledger = validation
        .take_decision_work_ledger()
        .expect("finalized branch-list validation can return its ledger")
        .expect("branch-list validation always owns a decision-work ledger");
    let mut branches = Vec::with_capacity(branch_snapshot.documents().len());
    let mut retained_bytes = 0_u64;
    for document in branch_snapshot.documents() {
        let state = document
            .value()
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("closed");
        let branch_bytes = compact_branch_result_size(document.name(), document.head(), state)?;
        let next_retained = retained_bytes.checked_add(branch_bytes).ok_or_else(|| {
            CliError::InvalidRepository("branch result size overflow".to_string())
        })?;
        if let Err(error) = ledger.apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                branch_bytes,
            ),
        ) {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            branch_snapshot
                .release(&mut ledger)
                .map_err(branch_list_ref_error)?;
            return Err(branch_list_accounting_error(error));
        }
        retained_bytes = next_retained;
        branches.push(Branch {
            name: document.name().to_string(),
            head: document.head().to_string(),
            state: state.to_string(),
        });
    }
    debug_assert!(branches
        .iter()
        .all(|branch| branch_snapshot.heads().contains(&branch.head)));
    let ref_work = match branch_snapshot.revalidate(REPOSITORY_JSON_BYTE_LIMIT, &mut ledger) {
        Ok(work) => work,
        Err(error) => {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            finish_branch_list_failure(&mut ledger, &error);
            return Err(branch_list_ref_error(error));
        }
    };
    if let Err(error) = metadata_snapshot.ensure_current() {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        finish_branch_list_failure(&mut ledger, &error);
        return Err(branch_list_ref_error(error));
    }
    Ok(BranchListResult {
        repo_root,
        metadata_snapshot,
        branches,
        opened: Vec::new(),
        store_work,
        ref_work,
        opened_work: BranchRefWork::default(),
        opened_snapshot: None,
        ledger,
        retained_bytes,
    })
}

#[derive(Serialize)]
struct BorrowedBranchResult<'a> {
    name: &'a str,
    head: &'a str,
    state: &'a str,
}

#[derive(Serialize)]
struct BorrowedOpenedRegistryResult<'a> {
    branch: &'a str,
    path: &'a str,
    open_instance_id: Option<&'a Value>,
    current_base_commit: Option<&'a Value>,
    opened_from_commit: Option<&'a Value>,
    registry_path: &'a Path,
}

struct BranchesWire<'a>(&'a [Branch]);

impl Serialize for BranchesWire<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for branch in self.0 {
            sequence.serialize_element(&BorrowedBranchResult {
                name: &branch.name,
                head: &branch.head,
                state: &branch.state,
            })?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
struct BranchListDataWire<'a> {
    #[serde(rename = "type")]
    type_tag: &'static str,
    version: u64,
    branches: BranchesWire<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metrics: Option<MetricsWire<'a>>,
}

#[derive(Serialize)]
struct LocalListDataWire<'a> {
    #[serde(rename = "type")]
    type_tag: &'static str,
    version: u64,
    branches: BranchesWire<'a>,
    opened: &'a [Value],
}

#[derive(Serialize)]
struct CommandResultWire<'a, T> {
    schema: &'static str,
    command: &'static str,
    ok: bool,
    exit_code: i32,
    repo: Option<&'a Path>,
    data: T,
    diagnostics: &'a [Value],
    next_actions: &'a [Value],
}

#[derive(Default)]
struct JsonSizeWriter(u64);

impl Write for JsonSizeWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let amount = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("serialized JSON size does not fit u64"))?;
        self.0 = self
            .0
            .checked_add(amount)
            .ok_or_else(|| std::io::Error::other("serialized JSON size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn reserve_output_bytes(
    ledger: &mut codefire_core::DecisionWorkLedger,
    bytes: u64,
) -> Result<(), CliError> {
    ledger
        .apply(
            codefire_core::DecisionWorkReservation::new(
                codefire_core::DecisionWorkPhase::PublicationPreflight,
                codefire_core::DecisionWorkSurface::Atomic,
            )
            .acquire(codefire_core::DecisionWorkDimension::RetainedBytes, bytes)
            .count(
                codefire_core::DecisionWorkCounter::ResponseBytesEncoded,
                bytes,
            ),
        )
        .map_err(branch_list_accounting_error)
}

fn encode_pretty_json_output(
    ledger: &mut codefire_core::DecisionWorkLedger,
    value: &impl Serialize,
) -> Result<(Vec<u8>, u64), CliError> {
    let mut counter = JsonSizeWriter::default();
    serde_json::to_writer_pretty(&mut counter, value)?;
    let retained_bytes = counter
        .0
        .checked_add(1)
        .ok_or_else(|| CliError::InvalidRepository("JSON output size overflow".to_string()))?;
    reserve_output_bytes(ledger, retained_bytes)?;
    let capacity = usize::try_from(retained_bytes)
        .map_err(|_| CliError::InvalidRepository("JSON output is too large".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity);
    if let Err(error) = serde_json::to_writer_pretty(&mut bytes, value) {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        return Err(error.into());
    }
    bytes.push(b'\n');
    debug_assert_eq!(bytes.len(), capacity);
    Ok((bytes, retained_bytes))
}

fn encode_text_output<F>(
    ledger: &mut codefire_core::DecisionWorkLedger,
    render: F,
) -> Result<(Vec<u8>, u64), CliError>
where
    F: Fn(&mut dyn Write) -> std::io::Result<()>,
{
    let mut counter = JsonSizeWriter::default();
    render(&mut counter)?;
    let retained_bytes = counter.0;
    reserve_output_bytes(ledger, retained_bytes)?;
    let capacity = usize::try_from(retained_bytes)
        .map_err(|_| CliError::InvalidRepository("text output is too large".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity);
    if let Err(error) = render(&mut bytes) {
        let _ = ledger.release(
            codefire_core::DecisionWorkDimension::RetainedBytes,
            retained_bytes,
        );
        return Err(error.into());
    }
    debug_assert_eq!(bytes.len(), capacity);
    Ok((bytes, retained_bytes))
}

fn compact_branch_result_size(name: &str, head: &str, state: &str) -> Result<u64, CliError> {
    let mut writer = JsonSizeWriter::default();
    serde_json::to_writer(&mut writer, &BorrowedBranchResult { name, head, state })?;
    Ok(writer.0)
}

fn project_opened_registries(
    snapshot: &OpenedRegistrySnapshot,
    ledger: &mut codefire_core::DecisionWorkLedger,
) -> Result<(Vec<Value>, u64), CliError> {
    let mut retained_bytes = 0_u64;
    let result = (|| {
        let mut opened = Vec::with_capacity(snapshot.documents().len());
        for document in snapshot.documents() {
            let registry = document.value();
            let branch = registry
                .get("branch")
                .and_then(Value::as_str)
                .or_else(|| {
                    registry
                        .get("branch")
                        .and_then(|branch| branch.get("name"))
                        .and_then(Value::as_str)
                })
                .or_else(|| registry.get("name").and_then(Value::as_str))
                .ok_or_else(|| {
                    CliError::InvalidRepository(format!(
                        "opened registry at {} has no branch name",
                        document.display_path().display()
                    ))
                })?;
            let open = registry.get("open").and_then(Value::as_object);
            let open_path = open
                .and_then(|open| open.get("path").or_else(|| open.get("opened_path")))
                .and_then(Value::as_str)
                .unwrap_or("");
            let borrowed = BorrowedOpenedRegistryResult {
                branch,
                path: open_path,
                open_instance_id: open.and_then(|open| open.get("open_instance_id")),
                current_base_commit: open.and_then(|open| open.get("current_base_commit")),
                opened_from_commit: open.and_then(|open| open.get("opened_from_commit")),
                registry_path: document.display_path(),
            };
            let mut counter = JsonSizeWriter::default();
            serde_json::to_writer(&mut counter, &borrowed)?;
            let next_retained = retained_bytes.checked_add(counter.0).ok_or_else(|| {
                CliError::InvalidRepository("opened registry projection size overflow".to_string())
            })?;
            ledger
                .apply(
                    codefire_core::DecisionWorkReservation::new(
                        codefire_core::DecisionWorkPhase::PublicationPreflight,
                        codefire_core::DecisionWorkSurface::Atomic,
                    )
                    .acquire(
                        codefire_core::DecisionWorkDimension::RetainedBytes,
                        counter.0,
                    ),
                )
                .map_err(branch_list_accounting_error)?;
            retained_bytes = next_retained;
            opened.push(json!({
                "branch": branch,
                "path": open_path,
                "open_instance_id": borrowed.open_instance_id.cloned().unwrap_or(Value::Null),
                "current_base_commit": borrowed.current_base_commit.cloned().unwrap_or(Value::Null),
                "opened_from_commit": borrowed.opened_from_commit.cloned().unwrap_or(Value::Null),
                "registry_path": document.display_path(),
            }));
        }
        Ok(opened)
    })();
    match result {
        Ok(opened) => Ok((opened, retained_bytes)),
        Err(error) => {
            let _ = ledger.release(
                codefire_core::DecisionWorkDimension::RetainedBytes,
                retained_bytes,
            );
            Err(error)
        }
    }
}

fn finish_branch_list_failure(
    ledger: &mut codefire_core::DecisionWorkLedger,
    error: &BranchRefSnapshotError,
) {
    if ledger.snapshot().terminal_reason.is_none() {
        let reason = match error {
            BranchRefSnapshotError::SubjectChanged { .. } => {
                codefire_core::DecisionWorkTerminalReason::SubjectChanged
            }
            _ => codefire_core::DecisionWorkTerminalReason::CorruptSubject,
        };
        let _ = ledger.finish(reason);
    }
}

fn finish_branch_list_store_failure(
    ledger: &mut codefire_core::DecisionWorkLedger,
    error: &codefire_store::StoreError,
) {
    if ledger.snapshot().terminal_reason.is_none() {
        let _ = ledger.finish(branch_list_store_terminal_reason(error));
    }
}

fn branch_list_store_terminal_reason(
    error: &codefire_store::StoreError,
) -> codefire_core::DecisionWorkTerminalReason {
    if matches!(
        error,
        codefire_store::StoreError::SealedValidationSubjectChanged { .. }
    ) {
        codefire_core::DecisionWorkTerminalReason::SubjectChanged
    } else {
        codefire_core::DecisionWorkTerminalReason::CorruptSubject
    }
}

fn branch_list_cli_terminal_reason(error: &CliError) -> codefire_core::DecisionWorkTerminalReason {
    if matches!(
        error,
        CliError::BranchReferenceSubjectChanged { .. }
            | CliError::OpenedRegistrySubjectChanged { .. }
    ) {
        codefire_core::DecisionWorkTerminalReason::SubjectChanged
    } else {
        codefire_core::DecisionWorkTerminalReason::CorruptSubject
    }
}

fn branch_list_ref_error(error: BranchRefSnapshotError) -> CliError {
    match error {
        BranchRefSnapshotError::Io(error) => CliError::Io(error),
        BranchRefSnapshotError::Json(error) => CliError::Json(error),
        BranchRefSnapshotError::Accounting(error) => branch_list_accounting_error(error),
        BranchRefSnapshotError::SubjectChanged { subject, reason } => {
            CliError::BranchReferenceSubjectChanged {
                subject,
                message: reason,
            }
        }
        BranchRefSnapshotError::InvalidRecord { path, reason } => CliError::InvalidRepository(
            format!("invalid branch reference at {}: {reason}", path.display()),
        ),
    }
}

fn opened_registry_ref_error(error: BranchRefSnapshotError) -> CliError {
    match error {
        BranchRefSnapshotError::Io(error) => CliError::Io(error),
        BranchRefSnapshotError::Json(error) => CliError::Json(error),
        BranchRefSnapshotError::Accounting(error) => branch_list_accounting_error(error),
        BranchRefSnapshotError::SubjectChanged { subject, reason } => {
            CliError::OpenedRegistrySubjectChanged {
                subject,
                message: reason,
            }
        }
        BranchRefSnapshotError::InvalidRecord { path, reason } => CliError::InvalidRepository(
            format!("invalid opened registry at {}: {reason}", path.display()),
        ),
    }
}

fn branch_list_accounting_error(error: codefire_core::DecisionWorkAccountingError) -> CliError {
    match error.budget_exceeded() {
        Some(exhaustion) => CliError::from(exhaustion.clone()),
        None => CliError::InvalidRepository(format!(
            "decision-work accounting failed while listing branches: {error}"
        )),
    }
}

fn branch_list_store_error(error: codefire_store::StoreError) -> CliError {
    match error {
        codefire_store::StoreError::DecisionWorkAccounting(accounting) => {
            branch_list_accounting_error(*accounting)
        }
        error => CliError::from(error),
    }
}

fn load_branch_record(repo_root: &Path, branch: &str) -> Result<Value, CliError> {
    let path = branch_record_path(repo_root, branch);
    if !path.exists() {
        return Err(CliError::Usage(format!("unknown branch: {branch}")));
    }
    read_json(&path)
}

fn save_branch_record(repo_root: &Path, branch: &Value) -> Result<(), CliError> {
    codefire_store::store_object(
        &repo_root.join(".codefire").join("objects"),
        "branch",
        branch.clone(),
    )?;
    write_branch_record_reference(repo_root, branch)
}

fn write_branch_record_reference(repo_root: &Path, branch: &Value) -> Result<(), CliError> {
    let name = required_string(branch, &["name"])?;
    write_json_atomic(&branch_record_path(repo_root, &name), branch)
}

fn branch_record_path(repo_root: &Path, branch: &str) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("branches")
        .join(format!("{}.json", ref_file_name(branch)))
}

fn opened_registry_path(repo_root: &Path, branch: &str) -> PathBuf {
    repo_root
        .join(".codefire")
        .join("opened")
        .join(format!("{}.json", ref_file_name(branch)))
}

fn materialize_commit(
    validated: &codefire_store::ValidatedSealedCommit,
    target: &StagedWorktree,
) -> Result<(), CliError> {
    let manifest = validated.read_root("content_manifest")?;
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let rel_path = required_string(entry, &["path"])?;
        let blob_id = required_string(entry, &["blob"])?;
        let blob = validated.read_object(&blob_id)?;
        let encoding = required_string(&blob, &["encoding"])?;
        if encoding != "base64" {
            return Err(CliError::InvalidRepository(format!(
                "unsupported blob encoding: {encoding}"
            )));
        }
        let content = required_string(&blob, &["content"])?;
        let output_path = safe_relative_manifest_path(&rel_path)?;
        target
            .write_file(&output_path, &decode_base64(&content)?)
            .map_err(CliError::Io)?;
    }
    Ok(())
}

fn planned_commit_object(
    role: &'static str,
    path: Option<String>,
    object_type: &'static str,
    payload: Value,
    stage: CommitObjectStage,
) -> Result<PlannedCommitObject, CliError> {
    let object_id = codefire_store::object_id(object_type, &payload)?;
    Ok(PlannedCommitObject {
        role,
        path,
        object_type,
        object_id,
        payload: Some(payload),
        worktree_source: None,
        stage,
    })
}

fn manifest_blob_payload(data: &[u8]) -> Value {
    json!({
        "type": "blob",
        "version": 1,
        "encoding": "base64",
        "content": encode_base64(data),
    })
}

fn build_manifest_plan(open_dir: &Path) -> Result<ManifestCommitPlan, CliError> {
    let mut entries = Vec::new();
    let mut blobs = Vec::new();
    for rel in rel_files(open_dir)? {
        let data = fs::read(open_dir.join(&rel))?;
        let path = to_posix(&rel);
        let blob_payload = manifest_blob_payload(&data);
        let blob = PlannedCommitObject {
            role: "manifest_blob",
            path: Some(path.clone()),
            object_type: "blob",
            object_id: codefire_store::object_id("blob", &blob_payload)?,
            payload: None,
            worktree_source: Some(rel),
            stage: CommitObjectStage::ManifestBlob,
        };
        entries.push(json!({
            "path": path,
            "kind": "file",
            "mode": "100644",
            "blob": &blob.object_id,
        }));
        blobs.push(blob);
    }
    let manifest = planned_commit_object(
        "content_manifest",
        None,
        "content_manifest",
        json!({"type": "content_manifest", "version": 1, "entries": entries}),
        CommitObjectStage::SealedRoot,
    )?;
    Ok(ManifestCommitPlan { blobs, manifest })
}

fn source_manifest_unchanged(
    objects: &Path,
    base_commit: &str,
    open_dir: &Path,
) -> Result<bool, CliError> {
    let validated = codefire_store::validate_sealed_commit_pinned(objects, base_commit)?;
    source_manifest_unchanged_validated(&validated, open_dir)
}

fn source_manifest_unchanged_validated(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &Path,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint(open_dir)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

fn source_manifest_unchanged_existing(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &ExistingWorktree,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint_from_files(open_dir.read_manifest_files()?)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

fn source_manifest_unchanged_secure(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &secure_worktree::SecureWorktree,
) -> Result<bool, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint_from_files(open_dir.read_manifest_files()?)?;
    Ok(manifest_blob_map(&base_manifest)? == manifest_blob_map(&current_manifest)?)
}

fn manifest_file_changes_since_base(
    validated: &codefire_store::ValidatedSealedCommit,
    open_dir: &Path,
) -> Result<Vec<codefire_core::ManifestFileChange>, CliError> {
    let base_manifest = validated.read_root("content_manifest")?;
    let current_manifest = build_manifest_fingerprint(open_dir)?;
    let base_files = manifest_blob_map(&base_manifest)?;
    let current_files = manifest_blob_map(&current_manifest)?;
    let paths = base_files
        .keys()
        .chain(current_files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    for path in paths {
        let base_blob = base_files.get(&path).cloned();
        let current_blob = current_files.get(&path).cloned();
        if base_blob == current_blob {
            continue;
        }
        let status = match (base_blob.as_ref(), current_blob.as_ref()) {
            (None, Some(_)) => "added",
            (Some(_), None) => "deleted",
            (Some(_), Some(_)) => "modified",
            (None, None) => continue,
        };
        changes.push(codefire_core::ManifestFileChange {
            path,
            status: status.to_string(),
            base_blob,
            current_blob,
        });
    }
    Ok(changes)
}

fn manifest_blob_map(manifest: &Value) -> Result<BTreeMap<String, String>, CliError> {
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    let mut files = BTreeMap::new();
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let path = required_string(entry, &["path"])?;
        let blob = required_string(entry, &["blob"])?;
        files.insert(path, blob);
    }
    Ok(files)
}

fn build_manifest_fingerprint(open_dir: &Path) -> Result<Value, CliError> {
    let files = rel_files(open_dir)?
        .into_iter()
        .map(|relative| {
            let data = fs::read(open_dir.join(&relative))?;
            Ok((relative, data))
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    build_manifest_fingerprint_from_files(files)
}

fn build_manifest_fingerprint_from_files(
    files: Vec<(PathBuf, Vec<u8>)>,
) -> Result<Value, CliError> {
    let mut entries = Vec::with_capacity(files.len());
    for (rel, data) in files {
        let blob_payload = json!({
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": encode_base64(&data),
        });
        let blob_id = codefire_store::object_id("blob", &blob_payload)?;
        entries.push(json!({
            "path": to_posix(&rel),
            "kind": "file",
            "mode": "100644",
            "blob": blob_id,
        }));
    }
    Ok(json!({"type": "content_manifest", "version": 1, "entries": entries}))
}

fn rel_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    let mut files = Vec::new();
    collect_rel_files(root, Path::new(""), &mut files)?;
    files.sort_by_key(|path| to_posix(path));
    Ok(files)
}

fn collect_rel_files(
    root: &Path,
    rel_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), CliError> {
    let mut entries = fs::read_dir(root.join(rel_dir))?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rel = rel_dir.join(name.as_ref());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if matches!(name.as_ref(), ".codefire" | "__pycache__" | ".pytest_cache") {
                continue;
            }
            collect_rel_files(root, &rel, files)?;
        } else if file_type.is_file() {
            let rel_path = to_posix(&rel);
            let import_operational_temp = rel.components().count() == 1
                && ((rel_path.starts_with(".codefire-import.lock.bootstrap-")
                    && (rel_path.ends_with(".tmp") || rel_path.ends_with(".part")))
                    || (rel_path.starts_with(".codefire-import.write-")
                        && rel_path.ends_with(".tmp")));
            if !import_operational_temp
                && !matches!(
                    rel_path.as_str(),
                    ".codefire-open"
                        | ".codefire-import.lock"
                        | ".codefire-import-transaction.json"
                )
            {
                files.push(rel);
            }
        }
    }
    Ok(())
}

fn to_posix(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn safe_relative_manifest_path(rel_path: &str) -> Result<PathBuf, CliError> {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        return Err(CliError::InvalidRepository(format!(
            "manifest path must be relative: {rel_path}"
        )));
    }
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => output.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::InvalidRepository(format!(
                    "manifest path escapes target: {rel_path}"
                )));
            }
        }
    }
    Ok(output)
}

fn decode_base64(input: &str) -> Result<Vec<u8>, CliError> {
    let bytes: Vec<u8> = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if bytes.len() % 4 != 0 {
        return Err(CliError::InvalidRepository(
            "base64 content length is invalid".to_string(),
        ));
    }
    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for (chunk_index, chunk) in bytes.chunks(4).enumerate() {
        let last = chunk_index + 1 == bytes.len() / 4;
        let a = base64_value(chunk[0])?;
        let b = base64_value(chunk[1])?;
        let c_padding = chunk[2] == b'=';
        let d_padding = chunk[3] == b'=';
        if c_padding && !d_padding {
            return Err(CliError::InvalidRepository(
                "base64 padding is invalid".to_string(),
            ));
        }
        if (c_padding || d_padding) && !last {
            return Err(CliError::InvalidRepository(
                "base64 padding before final chunk".to_string(),
            ));
        }
        let c = if c_padding {
            0
        } else {
            base64_value(chunk[2])?
        };
        let d = if d_padding {
            0
        } else {
            base64_value(chunk[3])?
        };
        output.push((a << 2) | (b >> 4));
        if !c_padding {
            output.push(((b & 0x0f) << 4) | (c >> 2));
        }
        if !d_padding {
            output.push(((c & 0x03) << 6) | d);
        }
    }
    Ok(output)
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(ALPHABET[(a >> 2) as usize] as char);
        encoded.push(ALPHABET[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            encoded.push(ALPHABET[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() == 3 {
            encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn base64_value(byte: u8) -> Result<u8, CliError> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(CliError::InvalidRepository(
            "base64 content contains an invalid character".to_string(),
        )),
    }
}

fn is_empty_dir(path: &Path) -> Result<bool, CliError> {
    Ok(path.is_dir() && fs::read_dir(path)?.next().is_none())
}

#[derive(Debug)]
struct RepoLock {
    repo_root: PathBuf,
    path: PathBuf,
    identity: secure_worktree::LockFileIdentity,
    file: File,
}

const REPO_LOCK_OWNER_BYTE_LIMIT: u64 = 16 * 1024;

fn acquire_locked_open_context(
    start: &Path,
    options: &LockOptions,
) -> Result<(RepoLock, OpenContext), CliError> {
    acquire_locked_open_context_with_recovery(start, options, None)
}

fn acquire_locked_open_context_for_recovery(
    start: &Path,
    options: &LockOptions,
    operation: &str,
) -> Result<(RepoLock, OpenContext), CliError> {
    acquire_locked_open_context_with_recovery(start, options, Some(operation))
}

fn acquire_locked_open_context_with_recovery(
    start: &Path,
    options: &LockOptions,
    recovery_operation: Option<&str>,
) -> Result<(RepoLock, OpenContext), CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = RepoLock::acquire_with_options(&repo_root, options)?;
    maybe_mutate_locked_open_context_for_test(start)?;
    let context = match recovery_operation {
        Some(operation) => open_context_for_recovery(start, operation)?,
        None => open_context(start)?,
    };
    lock.ensure_repo_root(&context.repo_root)?;
    Ok((lock, context))
}

impl RepoLock {
    fn acquire_with_options(repo_root: &Path, options: &LockOptions) -> Result<Self, CliError> {
        let repo_root = fs::canonicalize(repo_root)?;
        let path = repo_root.join(".codefire").join("locks").join("repo.lock");
        let owner_path = repo_lock_owner_path(&path);
        let (file, identity) = secure_worktree::open_lock_file_nofollow(&path)?;
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let start = Instant::now();
        loop {
            match Fs2FileExt::try_lock_exclusive(&file) {
                Ok(()) => {
                    secure_worktree::ensure_lock_file_identity_nofollow(&path, identity)?;
                    publish_repo_lock_owner(&owner_path)?;
                    return Ok(Self {
                        repo_root: repo_root.clone(),
                        path,
                        identity,
                        file,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if !options.wait {
                        return Err(repo_lock_contention_with_owner(
                            &repo_root,
                            &path,
                            Some(&owner_path),
                            options,
                        ));
                    }
                    if timeout.is_some_and(|timeout| start.elapsed() >= timeout) {
                        return Err(repo_lock_contention_with_owner(
                            &repo_root,
                            &path,
                            Some(&owner_path),
                            options,
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }

    fn ensure_repo_root(&self, repo_root: &Path) -> Result<(), CliError> {
        let resolved = fs::canonicalize(repo_root)?;
        if resolved != self.repo_root {
            return Err(CliError::InvalidMarker(format!(
                "open marker repository changed while acquiring the repository lock: locked {}, resolved {}",
                self.repo_root.display(),
                resolved.display()
            )));
        }
        secure_worktree::ensure_lock_file_identity_nofollow(&self.path, self.identity)?;
        Ok(())
    }
}

fn repo_lock_contention_with_owner(
    repo_root: &Path,
    path: &Path,
    owner_path: Option<&Path>,
    options: &LockOptions,
) -> CliError {
    let owner = owner_path.and_then(read_repo_lock_owner);
    let mut message = format!("CodeFire repository is locked: {}", path.display());
    if let Some(timeout_ms) = options.timeout_ms {
        message.push_str(&format!("; timed out after {timeout_ms}ms"));
    }
    if let Some(owner) = owner {
        if let Some(pid) = owner.get("pid").and_then(Value::as_u64) {
            message.push_str(&format!("; owner pid {pid}"));
        }
        if let Some(created_at) = owner.get("created_at").and_then(Value::as_str) {
            message.push_str(&format!("; locked at {created_at}"));
        }
    }
    CliError::RepositoryLockContention {
        message,
        repo_root: repo_root.to_path_buf(),
        lock_path: path.to_path_buf(),
    }
}

fn repo_lock_owner_path(lock_path: &Path) -> PathBuf {
    lock_path.with_file_name("repo.lock.owner.json")
}

fn publish_repo_lock_owner(path: &Path) -> Result<(), CliError> {
    let mut owner = serde_json::to_vec_pretty(&json!({
        "version": 1,
        "pid": std::process::id(),
        "created_at": now_iso_utc(),
    }))?;
    owner.push(b'\n');
    let temp_path = unique_temp_path(path);
    secure_worktree::write_file_atomic_nofollow(path, &temp_path, &owner)?;
    Ok(())
}

fn read_repo_lock_owner(path: &Path) -> Option<Value> {
    let bytes =
        secure_worktree::read_file_nofollow_bounded(path, REPO_LOCK_OWNER_BYTE_LIMIT).ok()?;
    serde_json::from_slice(&bytes).ok()
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = Fs2FileExt::unlock(&self.file);
    }
}

struct FileLock {
    _lock: stable_lock::StableFileLock,
}

impl FileLock {
    fn acquire_with_options(path: PathBuf, options: &LockOptions) -> Result<Self, CliError> {
        let deadline = stable_lock::LockDeadline::from_options(options);
        stable_lock::StableFileLock::acquire(
            &path,
            stable_lock::StableLockMode::Exclusive,
            &deadline,
        )
        .map(|lock| Self { _lock: lock })
    }
}

fn find_repo_root(start: &Path) -> Result<PathBuf, CliError> {
    let mut current = start.canonicalize()?;
    if current.is_file() {
        current.pop();
    }
    let mut candidate = Some(current.as_path());
    while let Some(path) = candidate {
        if path.join(".codefire").is_dir() {
            return Ok(path.to_path_buf());
        }
        candidate = path.parent();
    }
    if let Some(marker_path) = find_open_marker(&current)? {
        let marker = read_json(&marker_path)?;
        let repo_dir = PathBuf::from(required_string(&marker, &["repository", "path"])?);
        return repo_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CliError::InvalidMarker("repository path has no parent".to_string()));
    }
    Err(CliError::Usage(
        "not inside a CodeFire repository; run 'codefire init' first".to_string(),
    ))
}

fn find_open_marker(start: &Path) -> Result<Option<PathBuf>, CliError> {
    let mut current = start.canonicalize()?;
    if current.is_file() {
        current.pop();
    }
    loop {
        let marker = current.join(".codefire-open");
        if marker.exists() {
            return Ok(Some(marker));
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

fn read_json(path: &Path) -> Result<Value, CliError> {
    let bytes = secure_worktree::read_file_nofollow_bounded(path, REPOSITORY_JSON_BYTE_LIMIT)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn required_string(value: &Value, path: &[&str]) -> Result<String, CliError> {
    let mut current = value;
    for key in path {
        current = current
            .get(*key)
            .ok_or_else(|| CliError::InvalidMarker(format!("missing {}", path.join("."))))?;
    }
    current
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| CliError::InvalidMarker(format!("{} must be a string", path.join("."))))
}

fn ref_file_name(name: &str) -> String {
    codefire_util::percent_encode_path_segment(name)
}

fn now_iso_utc() -> String {
    codefire_util::now_iso_utc()
}

#[cfg(test)]
mod tests;
