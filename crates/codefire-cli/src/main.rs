use serde_json::{json, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

mod automation;
mod batch;
mod cli_model;
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
mod limited_yaml;
mod link_batch;
mod merge_patch_idempotency;
mod metrics;
mod migration;
mod open_clone_idempotency;
mod remote;
mod repo_layout;
mod signatures;
mod storage;
mod verification;
mod view;
use automation::{
    cli_command, command_result_envelope, scan_data_json_with_full, scan_diagnostics_json,
    scan_next_actions, status_data_json, status_next_actions, verification_data_json_with_filter,
    verification_diagnostics_json_with_filter, verification_next_actions,
};
use batch::{
    batch_extinguish_data_json, batch_template_data_json, has_batch_extinguish_arg,
    has_batch_template_arg, parse_batch_template_args, parse_extinguish_batch_args,
    print_batch_template_result, run_batch_template, run_extinguish_batch,
};
pub(crate) use cli_model::*;
use completion::{completion_script, help_text};
use context::{build_context_pack, parse_context_args, print_context_summary};
use doctor::{
    doctor_report_data_json, doctor_report_diagnostics_json, doctor_report_next_actions,
    parse_doctor_args, print_doctor_report, run_doctor,
};
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
use fire::{
    fire_batch_data_json, fire_data_json, parse_fire_args, parse_fire_batch_args,
    print_fire_result, run_fire, run_fire_batch,
};
use http::{http_json, http_remote_path, is_cf_http_url, parse_cf_http_url, serve_http};
use idempotency::{
    idempotency_payload_hash, idempotency_record_path, idempotency_result_plan,
    require_idempotency_key, verify_idempotency_record,
};
use link_batch::{link_batch_data_json, parse_link_batch_args, run_link_batch};
use merge_patch_idempotency::{
    load_merge_idempotency, load_patch_import_idempotency, merge_idempotency_payload,
    patch_import_idempotency_payload, save_merge_idempotency, save_patch_import_idempotency,
};
use metrics::{
    attach_metrics, branch_list_metrics, print_metrics, scan_metrics, status_metrics,
    verification_metrics,
};
use migration::{
    migration_report_data_json, migration_report_diagnostics_json, migration_report_next_actions,
    parse_migrate_args, print_migration_report, run_migrate,
};
use open_clone_idempotency::{
    clone_idempotency_payload, load_clone_idempotency, load_open_idempotency,
    open_idempotency_payload, save_clone_idempotency, save_open_idempotency,
};
use remote::{
    apply_merge_request, copy_object_graph, list_merge_requests, list_remote_branches,
    load_remote_branch, parse_cf_url, read_optional_json, remote_dirs, request_merge,
    review_merge_request, upload_branch, write_object_records, RemoteProjectOptions,
    RequestApplyOptions, RequestMergeOptions, RequestReviewOptions, UploadOptions,
};
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

fn status_state(raw_state: &str, open_fires: usize) -> String {
    if open_fires > 0 {
        "open-burning".to_string()
    } else {
        raw_state.to_string()
    }
}

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        if !matches!(
            error,
            CliError::VerificationFailed(_) | CliError::CommandFailed(_)
        ) {
            eprintln!("error: {error}");
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
            let trace_graph = codefire_core::current_trace_graph(&options.path)?;
            let repo_root = open_context(&options.path)
                .ok()
                .map(|context| context.repo_root);
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
            let missing = codefire_core::current_required_link_missing(&options.path)?;
            let repo_root = open_context(&options.path)
                .ok()
                .map(|context| context.repo_root);
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
                    let repo_root = find_repo_root(&path)?;
                    let branches = list_branches(&path)?;
                    let opened = list_opened_registries(&repo_root)?;
                    let data = local_list_data_json(&branches, opened);
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "list",
                                true,
                                0,
                                Some(&repo_root),
                                data,
                                Vec::new(),
                                Vec::new(),
                            ))?
                        );
                    } else {
                        print_local_list(&branches, &data);
                    }
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
                    if options.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&command_result_envelope(
                                "list",
                                true,
                                0,
                                None,
                                remote_branch_list_data_json(&project_url, &branches),
                                Vec::new(),
                                Vec::new(),
                            ))?
                        );
                    } else {
                        for branch in branches {
                            println!("{}\t{}", branch.name, branch.head);
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
            if options.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "request-list",
                        true,
                        0,
                        None,
                        merge_request_list_data_json(&options.project_url, &requests),
                        Vec::new(),
                        Vec::new(),
                    ))?
                );
            } else {
                for request in requests {
                    println!(
                        "{}\t{}\t{}\t{}",
                        request.id, request.status, request.source_url, request.target_url
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
                let branches = list_branches(&start)?;
                let metrics = options
                    .metrics
                    .then(|| branch_list_metrics(started.elapsed(), &branches));
                if options.json_output {
                    let repo_root = find_repo_root(&start).ok();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&command_result_envelope(
                            "branch-list",
                            true,
                            0,
                            repo_root.as_deref(),
                            attach_metrics(json!({
                                "type": "codefire_branch_list",
                                "version": 1,
                                "branches": branches.iter().map(branch_json).collect::<Vec<_>>(),
                            }), metrics.as_ref()),
                            Vec::new(),
                            Vec::new(),
                        ))?
                    );
                } else {
                    print_branches(&branches);
                    if let Some(metrics) = metrics.as_ref() {
                        print_metrics(metrics);
                    }
                }
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
                    print_cli_error_json("branch", &error)?;
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
            let status = read_status(&options.path)?;
            let metrics = options
                .metrics
                .then(|| status_metrics(started.elapsed(), &status));
            if options.json_output {
                let repo_root = open_context(&options.path).ok().map(|context| context.repo_root);
                let data = attach_metrics(
                    status_data_json_with_prediction(&status, &options.path),
                    metrics.as_ref(),
                );
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "status",
                        true,
                        0,
                        repo_root.as_deref(),
                        data,
                        Vec::new(),
                        status_next_actions(&status),
                    ))?
                );
            } else {
                print_status(&status);
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
                let report = run_doctor(&options)?;
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
                            Some(&report.repo_root),
                            doctor_report_data_json(&report),
                            doctor_report_diagnostics_json(&report),
                            doctor_report_next_actions(&report),
                        ))?
                    );
                } else {
                    print_doctor_report(&report);
                }
                if report.ok {
                    Ok(())
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

fn command_help_for_args(args: &[String]) -> Option<&'static str> {
    let command = args.first()?.as_str();
    if !wants_help(&args[1..]) {
        return None;
    }
    match command {
        "status" | "scan" | "verify" | "fire" | "extinguish" | "commit" | "init" | "open"
        | "clone" | "upload" | "list" | "request-merge" | "request-list" | "request-review"
        | "request-apply" | "review-pack" | "storage" | "doctor" | "show" | "diff" | "context"
        | "explain" | "atom-index" | "trace-graph" | "missing-links" | "link" => {
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
        "open" => {
            "usage: codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>]\n\nOpen a sealed branch into a working directory.\n"
        }
        "clone" => {
            "usage: codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a new sealed branch from an existing branch.\n"
        }
        "upload" => {
            "usage: codefire upload <branch> <remote-url> [--dry-run] [--json] [--idempotency-key <key>] [--request-key-id <key>]\n\nUpload a sealed branch to a remote project.\n"
        }
        "list" => {
            "usage: codefire list [path|<remote-project-url>] [--json]\n\nList local repository branches/open worktrees or remote project branches.\n"
        }
        "request-merge" => {
            "usage: codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>]\n\nCreate a remote merge request.\n"
        }
        "request-list" => {
            "usage: codefire request-list <remote-project-url> [--json]\n\nList remote merge requests.\n"
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
            "usage: codefire scan [path|--path <path>] [--json] [--metrics] [--full]\n\nDetect changed atoms and open fires for an open directory.\n"
        }
        "verify" => {
            "usage: codefire verify [path|--path <path>] [--details] [--blocking-only] [--json] [--metrics]\n\nRun CodeFire verification checks for an open directory.\n"
        }
        "fire" => {
            "usage: codefire fire <source-atom> --to <target-atom> --reason <text> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"from\":\"REQ-id\",\"to\":\"DES-id\",\"reason\":\"manual review\"}]}\n"
        }
        "extinguish" => {
            "usage: codefire extinguish <fire-id> [--path <open-dir>] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>) [--dry-run] [--json]\n       codefire extinguish --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]\n       codefire extinguish --batch-template [--path <open-dir>] [--resolution <type>] [--rationale <text>|--evidence <text>|--evidence-ref <id>] [--output <file>] [--json]\n\nBatch JSON example:\n  {\"version\":1,\"fires\":[{\"id\":\"FIRE-1\",\"resolution\":\"addressed\",\"rationale\":\"fixed\"}]}\n"
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
            "usage: codefire migrate check [path] [--json] [--target-format v0.6]\n       codefire migrate dry-run [path] [--json] [--target-format v0.6]\n       codefire migrate apply [path] [--json] [--target-format v0.6]\n\nCheck or prepare CodeFire repository layout migration.\n"
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
enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Core(codefire_core::CoreError),
    Store(codefire_store::StoreError),
    Usage(String),
    AuthenticationOrSignature(String),
    IdempotencyConflict(String),
    HttpPayloadTooLarge(String),
    MigrationIncompatibility(String),
    VerificationFailed(ExitCode),
    CommandFailed(ExitCode),
    NotOpen(PathBuf),
    InvalidMarker(String),
    InvalidRepository(String),
    LockContention(String),
    BatchFileRead {
        operation: &'static str,
        path: PathBuf,
        kind: ErrorKind,
        message: String,
    },
    EvidenceCommandFailed {
        exit_code: Option<i32>,
        timed_out: bool,
        cwd: PathBuf,
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
            CliError::Store(error) => store_error_exit_code(error),
            CliError::Usage(message) => usage_exit_code(message),
            CliError::AuthenticationOrSignature(_) => ExitCode::AuthenticationOrSignatureFailure,
            CliError::IdempotencyConflict(_) => ExitCode::IdempotencyConflict,
            CliError::HttpPayloadTooLarge(_) => ExitCode::InvalidUsageOrConfig,
            CliError::MigrationIncompatibility(_) => ExitCode::MigrationIncompatibility,
            CliError::VerificationFailed(exit_code) => *exit_code,
            CliError::CommandFailed(exit_code) => *exit_code,
            CliError::NotOpen(_) => ExitCode::InvalidUsageOrConfig,
            CliError::InvalidMarker(_) | CliError::InvalidRepository(_) => {
                ExitCode::RepositoryCorruption
            }
            CliError::LockContention(_) => ExitCode::LockContention,
            CliError::BatchFileRead { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::EvidenceCommandFailed { .. } => ExitCode::InvalidUsageOrConfig,
            CliError::MissingEvidenceRef { .. } => ExitCode::ObjectReferenceInvalid,
        }
    }

    fn exit_code(&self) -> i32 {
        self.exit_status().code()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Json(error) => write!(f, "{error}"),
            CliError::Core(error) => write!(f, "{error}"),
            CliError::Store(error) => write!(f, "{error}"),
            CliError::Usage(message) => write!(f, "{message}"),
            CliError::AuthenticationOrSignature(message) => write!(f, "{message}"),
            CliError::IdempotencyConflict(message) => write!(f, "{message}"),
            CliError::HttpPayloadTooLarge(message) => write!(f, "{message}"),
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
            CliError::LockContention(message) => write!(f, "{message}"),
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

impl From<codefire_store::StoreError> for CliError {
    fn from(error: codefire_store::StoreError) -> Self {
        CliError::Store(error)
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
    let scan = run_scan(&options.path)?;
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
    let execution = compute_verify(&options.path, true)?;
    let verification = execution.verification;
    let scan = execution.scan;
    let metrics = options
        .metrics
        .then(|| verification_metrics(started.elapsed(), &verification));
    let exit_code = verification_exit_code(&verification);
    if options.json_output {
        let repo_root = open_context(&options.path)
            .ok()
            .map(|context| context.repo_root);
        println!(
            "{}",
            serde_json::to_string_pretty(&command_result_envelope(
                "verify",
                verification.result == "passed",
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
    if verification.result == "passed" {
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
    if options.json_output {
        print_commit_result_json(&result)?;
        if result.blocked {
            return Err(CliError::CommandFailed(result.exit_code));
        }
    } else if options.dry_run {
        if result.blocked {
            return Err(CliError::Usage(
                "commit blocked: verification failed or consistency blockers remain".to_string(),
            ));
        }
        println!("commit dry-run: branch {} would be sealed", result.branch);
        println!(
            "Changed atoms: {}",
            result.plan["changed_atom_count"].as_u64().unwrap_or(0)
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

fn status_data_json_with_prediction(status: &Status, path: &Path) -> Value {
    let mut data = status_data_json(status);
    if let Ok(scan_execution) = compute_scan(path, false) {
        let scan = scan_execution.scan;
        data["scan_prediction"] = json!({
            "source": "preview_recomputed",
            "branch_state": scan_branch_state(scan.changed_atoms.len(), scan.open_fires.len()),
            "changed_count": scan.changed_atoms.len(),
            "open_fire_count": scan.open_fires.len(),
            "tool_migration": &scan.tool_migration,
            "base_commit": &scan.base_commit,
        });
    }
    data
}

fn print_branches(branches: &[Branch]) {
    for branch in branches {
        println!("{}\t{}\t{}", branch.name, branch.head, branch.state);
    }
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

fn print_local_list(branches: &[Branch], data: &Value) {
    println!("Branches:");
    print_branches(branches);
    println!("Opened:");
    let opened = data
        .get("opened")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if opened.is_empty() {
        println!("none");
        return;
    }
    for item in opened {
        println!(
            "{}\t{}",
            item.get("branch")
                .and_then(Value::as_str)
                .unwrap_or("(unknown)"),
            item.get("path").and_then(Value::as_str).unwrap_or("")
        );
    }
}

fn local_list_data_json(branches: &[Branch], opened: Vec<Value>) -> Value {
    json!({
        "type": "codefire_local_list",
        "version": 1,
        "branches": branches.iter().map(branch_json).collect::<Vec<_>>(),
        "opened": opened,
    })
}

fn remote_branch_list_data_json(project_url: &str, branches: &[remote::RemoteBranch]) -> Value {
    json!({
        "type": "codefire_remote_branch_list",
        "version": 1,
        "project_url": project_url,
        "branches": branches.iter().map(|branch| json!({
            "name": &branch.name,
            "head": &branch.head,
        })).collect::<Vec<_>>(),
    })
}

fn merge_request_list_data_json(
    project_url: &str,
    requests: &[remote::MergeRequestListItem],
) -> Value {
    json!({
        "type": "codefire_merge_request_list",
        "version": 1,
        "project_url": project_url,
        "merge_requests": requests.iter().map(|request| json!({
            "id": &request.id,
            "status": &request.status,
            "source_url": &request.source_url,
            "target_url": &request.target_url,
        })).collect::<Vec<_>>(),
    })
}

fn list_opened_registries(repo_root: &Path) -> Result<Vec<Value>, CliError> {
    let opened_root = repo_root.join(".codefire").join("opened");
    if !opened_root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(opened_root)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut opened = Vec::with_capacity(paths.len());
    for path in paths {
        let registry = read_json(&path)?;
        let branch = required_string(&registry, &["branch", "name"])?;
        let open = registry.get("open").cloned().unwrap_or_else(|| json!({}));
        let open_path = open
            .get("path")
            .or_else(|| open.get("opened_path"))
            .and_then(Value::as_str)
            .unwrap_or("");
        opened.push(json!({
            "branch": branch,
            "path": open_path,
            "open_instance_id": open.get("open_instance_id").cloned().unwrap_or(Value::Null),
            "current_base_commit": open.get("current_base_commit").cloned().unwrap_or(Value::Null),
            "opened_from_commit": open.get("opened_from_commit").cloned().unwrap_or(Value::Null),
            "registry_path": path,
        }));
    }
    Ok(opened)
}

fn branch_show_data(start: &Path, branch: Option<&str>) -> Result<(PathBuf, Value), CliError> {
    let repo_root = find_repo_root(start)?;
    let branch_name = match branch {
        Some(branch) => branch.to_string(),
        None => open_context(start)?.branch,
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

fn print_scan(scan: &codefire_core::ScanResult) {
    print!("{}", render_scan(scan));
}

fn render_scan(scan: &codefire_core::ScanResult) -> String {
    let state = scan_branch_state(scan.changed_atoms.len(), scan.open_fires.len());
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
    if let Some(migration) = &scan.tool_migration {
        output.push_str(&format!(
            "Tool migration: {} hash schema {} -> {} affected_atoms={}\n",
            migration.kind,
            migration.from_hash_schema_version,
            migration.to_hash_schema_version,
            migration.affected_atoms.len()
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
        "base": value.get("base").cloned().unwrap_or_else(|| json!(null)),
        "source": value.get("source").cloned().unwrap_or_else(|| json!(null)),
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
        "base": value.get("base").cloned().unwrap_or_else(|| json!(null)),
        "source": value.get("source").cloned().unwrap_or_else(|| json!(null)),
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
        _ => None,
    };
    command_result_envelope(
        command,
        false,
        error.exit_code(),
        repo_root,
        json!({ "type": "codefire_command_error" }),
        vec![cli_error_diagnostic(error)],
        cli_error_next_actions(error),
    )
}

fn cli_error_diagnostic(error: &CliError) -> Value {
    match error {
        CliError::MissingEvidenceRef {
            evidence_id,
            repo_root,
        } => json!({
            "kind": "missing_evidence_ref",
            "severity": "blocking",
            "evidence_id": evidence_id,
            "repo": repo_root,
            "message": error.to_string(),
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
                "message": error.to_string(),
            })
        }
        CliError::Usage(message) if batch_schema_command(message).is_some() => json!({
            "kind": "batch_schema_error",
            "severity": "blocking",
            "blocking": true,
            "repairable": true,
            "command": batch_schema_command(message).expect("checked above"),
            "message": error.to_string(),
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
            "message": error.to_string(),
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
            "exit_code": exit_code,
            "timed_out": timed_out,
            "cwd": cwd,
            "message": error.to_string(),
        }),
        _ => json!({
            "kind": "command_error",
            "severity": "blocking",
            "message": error.to_string(),
        }),
    }
}

fn cli_error_next_actions(error: &CliError) -> Vec<Value> {
    match error {
        CliError::Usage(message) if message.starts_with("unknown branch:") => vec![json!({
            "kind": "list_branches",
            "command": cli_command("branch list --json"),
            "reason": "inspect available branch names before retrying the command",
            "target": {"error": message},
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
        _ => Vec::new(),
    }
}

fn batch_file_diagnostic_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "batch_file_missing",
        ErrorKind::PermissionDenied => "batch_file_permission_denied",
        _ => "batch_file_read_failed",
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
    for value in args {
        match value.as_str() {
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported list option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
    }
    let target = match positional.as_slice() {
        [] => ListTarget::Local(env::current_dir()?),
        [value] if is_remote_project_url(value) => ListTarget::Remote(value.clone()),
        [value] => ListTarget::Local(PathBuf::from(value)),
        _ => {
            return Err(CliError::Usage(
                "usage: codefire list [path|<remote-project-url>] [--json]".to_string(),
            ));
        }
    };
    Ok(ListOptions {
        target,
        json_output,
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
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if parse_path_json_metrics_option(arg, &mut json_output, &mut metrics) {
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
            CliError::Usage("usage: codefire extinguish <fire-id> [--path <open-dir>] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>)".to_string())
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
    for value in args {
        match value.as_str() {
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported {command} option: {option}"
                )));
            }
            value => positional.push(value.to_string()),
        }
    }
    match positional.as_slice() {
        [project_url] => Ok(RemoteProjectOptions {
            project_url: project_url.clone(),
            json_output,
        }),
        _ => Err(CliError::Usage(format!(
            "usage: codefire {command} <cf-project-url> [--json]"
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
    let repo_root = absolute_path(path)?;
    let cf = repo_root.join(".codefire");
    if cf.exists() && !force {
        return Err(CliError::Usage(".codefire already exists".to_string()));
    }

    ensure_repo_layout(&repo_root)?;
    let now = now_iso_utc();
    write_json_atomic(
        &cf.join("repo.json"),
        &json!({
            "version": 1,
            "repository_id": stable_repo_id(&repo_root),
            "created_at": now,
        }),
    )?;

    let objects = cf.join("objects");
    let roots = initial_roots(&objects, &now)?;
    let commit_payload = json!({
        "type": "commit",
        "version": 1,
        "parents": [],
        "message": "Initial empty CodeFire repository",
        "roots": roots,
        "certificate": {
            "result": "consistent",
            "open_required_fires": 0,
            "failed_checks": 0,
            "missing_required_links": 0,
            "stale_resolutions": 0,
            "duplicate_atom_ids": 0,
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
    let _lock = RepoLock::acquire_with_options(&repo_root, &options.lock)?;
    let idempotency_payload = merge_idempotency_payload(&repo_root, options)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_merge_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    let mut result = build_merge_result(&repo_root, options)?;
    if options.dry_run {
        return Ok(result);
    }
    apply_merge_result(&repo_root, &mut result)?;
    if let Some(key) = options.idempotency_key.as_deref() {
        save_merge_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

fn build_merge_result(repo_root: &Path, options: &MergeOptions) -> Result<MergeResult, CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    let source = load_branch_record(repo_root, &options.source_branch)?;
    let source_head = required_string(&source, &["head"])?;
    let source_state = source
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed");
    if source_state == "open-burning" {
        return Err(CliError::Usage(format!(
            "cannot merge from branch '{}' while it is open-burning",
            options.source_branch
        )));
    }
    let target = load_branch_record(repo_root, &options.target_branch)?;
    let target_head = required_string(&target, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &source_head)?;
    codefire_store::validate_sealed_commit(&objects, &target_head)?;

    let registry_path = opened_registry_path(repo_root, &options.target_branch);
    if !registry_path.exists() {
        return Err(CliError::Usage(format!(
            "target branch must be open: {}",
            options.target_branch
        )));
    }
    let registry = read_json(&registry_path)?;
    if required_string(&registry, &["state", "last_known"])? != "open-clean" {
        return Err(CliError::Usage(
            "target branch must be open-clean before merge".to_string(),
        ));
    }
    let base = common_ancestor(&objects, &source_head, &target_head)?
        .ok_or_else(|| CliError::Usage("no common ancestor found".to_string()))?;
    let target_dir = PathBuf::from(required_string(&registry, &["open", "path"])?);
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

fn apply_merge_result(repo_root: &Path, result: &mut MergeResult) -> Result<(), CliError> {
    for action in &result.file_actions {
        write_merged_file(&result.target_dir, &action.path, action.content.as_deref())?;
    }
    let registry_path = opened_registry_path(repo_root, &result.target_branch);
    let registry = read_json(&registry_path)?;
    let active_state_path =
        PathBuf::from(required_string(&registry, &["open", "active_state_path"])?);
    let state_path = active_state_path.join("state.json");
    let mut state = if state_path.exists() {
        read_json(&state_path)?
    } else {
        json!({})
    };
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
    write_json_atomic(&state_path, &state)?;
    let context = OpenContext {
        repo_root: repo_root.to_path_buf(),
        open_dir: result.target_dir.clone(),
        branch: result.target_branch.clone(),
        registry_path,
        registry,
    };
    set_open_state(&context, &active_state_path, "open-burning")?;
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
    if result.dry_run && result.conflicts.is_empty() {
        actions.push(json!({
            "kind": "apply_merge",
            "source_branch": &result.source_branch,
            "target_branch": &result.target_branch,
            "hint": format!("run codefire merge {} --into {}", result.source_branch, result.target_branch),
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
    let context = open_context(start)?;
    let _lock = RepoLock::acquire_with_options(&context.repo_root, &options.lock)?;
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
        safe_manifest_output_path(&context.open_dir, &path)?;
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
        for entry in entries {
            apply_patch_entry(&context.open_dir, entry)?;
        }
        let active_state_path = PathBuf::from(required_string(
            &context.registry,
            &["open", "active_state_path"],
        )?);
        let state_path = active_state_path.join("state.json");
        let mut state = if state_path.exists() {
            read_json(&state_path)?
        } else {
            json!({})
        };
        state["pending_patch_base"] = Value::String(patch_base.clone());
        state["pending_patch_source"] = patch
            .get("source")
            .and_then(|source| source.get("commit"))
            .cloned()
            .unwrap_or_else(|| json!(null));
        state["patch_paths"] = Value::Array(paths.iter().cloned().map(Value::String).collect());
        write_json_atomic(&state_path, &state)?;
        set_open_state(&context, &active_state_path, "open-burning")?;
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

fn apply_patch_entry(open_dir: &Path, entry: &Value) -> Result<(), CliError> {
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
            write_merged_file(open_dir, &path, Some(&bytes))
        }
        "delete" => write_merged_file(open_dir, &path, None),
        action => Err(CliError::Usage(format!(
            "unsupported patch entry action: {action}"
        ))),
    }
}

fn clone_branch(start: &Path, options: &CloneOptions) -> Result<CloneResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = RepoLock::acquire_with_options(&repo_root, &options.lock)?;
    let idempotency_payload = clone_idempotency_payload(options, &repo_root);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_clone_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    if branch_record_path(&repo_root, &options.new_branch).exists() {
        return Err(CliError::Usage(format!(
            "branch already exists: {}",
            options.new_branch
        )));
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
        write_object_records(
            &repo_root.join(".codefire").join("objects"),
            records.iter().cloned(),
        )?;
        let source_head = required_string(&branch_record, &["head"])?;
        codefire_store::validate_sealed_commit(
            &repo_root.join(".codefire").join("objects"),
            &source_head,
        )?;
        let plan =
            clone_operation_plan(options, &repo_root, &source_head, remote.endpoint_scheme());
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
        return Ok(result);
    }
    if options.source.starts_with("cf://") {
        let remote = parse_cf_url(&options.source)?;
        let source = load_remote_branch(&remote)?;
        let source_head = required_string(&source, &["head"])?;
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
    let source = load_branch_record(&repo_root, &options.source)?;
    let source_head = required_string(&source, &["head"])?;
    let source_state = source
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed");
    if source_state == "open-burning" {
        return Err(CliError::Usage(format!(
            "cannot clone from branch '{}' while it is open-burning",
            options.source
        )));
    }
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

fn open_branch(options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_branch_from(&env::current_dir()?, options)
}

fn open_branch_from(start: &Path, options: &OpenOptions) -> Result<OpenResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = RepoLock::acquire_with_options(&repo_root, &options.lock)?;
    let cf = repo_root.join(".codefire");
    let objects = cf.join("objects");
    let mut branch = load_branch_record(&repo_root, &options.branch)?;
    let branch_head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &branch_head)?;

    let target = absolute_path(&options.path)?;
    let idempotency_payload = open_idempotency_payload(options, &repo_root, &target, &branch_head);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_open_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    if target.exists() && !is_empty_dir(&target)? {
        return Err(CliError::Usage(format!(
            "target path must not exist or must be empty: {}",
            target.display()
        )));
    }
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    if registry_path.exists() {
        return Err(CliError::Usage(format!(
            "branch is already open: {}",
            options.branch
        )));
    }
    let opened_at = now_iso_utc();
    let open_instance_id = format!(
        "open_{:012x}",
        stable_hash_48(format!("{}:{}:{opened_at}", options.branch, target.display()).as_bytes())
    );
    let active_path = cf.join("active").join(&open_instance_id);
    let plan = open_operation_plan(
        options,
        &repo_root,
        &target,
        &branch_head,
        &registry_path,
        &active_path,
    );
    if options.dry_run {
        return Ok(OpenResult {
            branch: options.branch.clone(),
            open_dir: target,
            plan,
        });
    }

    fs::create_dir_all(&target)?;
    materialize_commit(&objects, &branch_head, &target)?;
    let repo_json = read_json(&cf.join("repo.json"))?;
    let repository_id = required_string(&repo_json, &["repository_id"])?;
    write_json_atomic(
        &target.join(".codefire-open"),
        &json!({
            "version": 1,
            "repository": {"path": cf, "repository_id": repository_id},
            "branch": {"name": options.branch, "opened_from_commit": branch_head},
            "open": {"open_instance_id": open_instance_id, "opened_path": target, "opened_at": opened_at},
        }),
    )?;
    write_json_atomic(
        &registry_path,
        &json!({
            "version": 1,
            "branch": {"name": options.branch},
            "open": {
                "open_instance_id": open_instance_id,
                "path": target,
                "opened_from_commit": branch_head,
                "current_base_commit": branch_head,
                "active_state_path": active_path,
            },
            "state": {"last_known": "open-clean"},
        }),
    )?;
    fs::create_dir_all(&active_path)?;
    write_json_atomic(
        &active_path.join("state.json"),
        &json!({"state": "open-clean"}),
    )?;
    branch["state"] = Value::String("open-clean".to_string());
    save_branch_record(&repo_root, &branch)?;

    let result = OpenResult {
        branch: options.branch.clone(),
        open_dir: target,
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_open_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
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
            {"kind": "materialize_commit", "commit": branch_head, "target": target},
            {"kind": "write_open_marker", "path": target.join(".codefire-open")},
            {"kind": "write_open_registry", "path": registry_path},
            {"kind": "create_active_state", "path": active_path},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "open-clean"},
        ],
        "next_actions": [
            {"kind": "status", "command": "codefire status --json", "target": {"path": target}},
        ],
    })
}

fn clone_operation_plan(
    options: &CloneOptions,
    repo_root: &Path,
    source_head: &str,
    source_kind: &str,
) -> Value {
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
            {"kind": "open", "command": format!("codefire open {} <path>", options.new_branch), "target": {"branch": &options.new_branch}},
        ],
    })
}

fn run_scan(start: &Path) -> Result<codefire_core::ScanResult, CliError> {
    Ok(compute_scan(start, true)?.scan)
}

fn compute_scan(start: &Path, persist: bool) -> Result<ScanExecution, CliError> {
    let context = open_context(start)?;
    let objects = context.repo_root.join(".codefire").join("objects");
    let base_commit = required_string(&context.registry, &["open", "current_base_commit"])?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let current = codefire_core::build_atom_index(&context.open_dir)?;
    let base_index = load_base_atom_index(&objects, &base_commit)?;
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    let fires_path = active_state_path.join("fires.json");
    let fires = if fires_path.exists() {
        serde_json::from_value(read_json(&fires_path)?)?
    } else {
        Vec::new()
    };
    let state_path = active_state_path.join("state.json");
    let state_data = if state_path.exists() {
        read_json(&state_path)?
    } else {
        json!({"state": "open-clean"})
    };
    let reason = if state_data.get("pending_merge_parent").is_some() {
        "merge_changed"
    } else {
        "atom_changed"
    };
    let now = now_iso_utc();
    let source_unchanged =
        source_manifest_unchanged(&objects, &base_commit, &context.open_dir).unwrap_or(false);
    let (scan, fires) = codefire_core::build_scan_result_with_options(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        codefire_core::ScanBuildOptions {
            reason,
            now: &now,
            source_unchanged,
        },
    )?;
    if persist {
        fs::create_dir_all(&active_state_path)?;
        write_json_atomic(&fires_path, &serde_json::to_value(&fires)?)?;
        write_json_atomic(
            &active_state_path.join("scan.json"),
            &serde_json::to_value(&scan)?,
        )?;
        let state = scan_branch_state(scan.changed_atoms.len(), scan.open_fires.len());
        set_open_state(&context, &active_state_path, state)?;
    }
    Ok(ScanExecution {
        context,
        active_state_path,
        scan,
        fires,
    })
}

#[cfg(test)]
fn run_verify(start: &Path) -> Result<codefire_core::Verification, CliError> {
    Ok(compute_verify(start, true)?.verification)
}

fn compute_verify(start: &Path, persist: bool) -> Result<VerifyExecution, CliError> {
    let scan_execution = compute_scan(start, persist)?;
    let context = scan_execution.context;
    let active_state_path = scan_execution.active_state_path;
    let scan = scan_execution.scan;
    let policy = codefire_core::parse_verification_policy(&context.open_dir)?;
    let trace_policy = codefire_core::TracePolicy {
        required_links: policy.required_links.clone(),
    };
    let missing_required_links =
        codefire_core::required_link_missing(&scan.atom_index, &scan.trace_graph, &trace_policy);
    let failed_checks = run_verification_commands(&context.open_dir, &policy)?;
    let resolutions_path = active_state_path.join("resolutions.json");
    let resolutions: Vec<codefire_core::Resolution> = if resolutions_path.exists() {
        serde_json::from_value(read_json(&resolutions_path)?)?
    } else {
        Vec::new()
    };
    let stale_resolutions = codefire_core::stale_resolutions(
        &scan.atom_index,
        &scan.trace_graph,
        &policy,
        &resolutions,
    )?;
    let missing_evidence_refs = collect_missing_evidence_refs(&context.repo_root, &resolutions);
    let verification = codefire_core::build_verification(
        &scan,
        missing_required_links,
        failed_checks,
        stale_resolutions,
        missing_evidence_refs,
        &policy,
        &now_iso_utc(),
    );
    if persist {
        persist_verification_result_state(&context, &active_state_path, &scan, &verification)?;
    }
    Ok(VerifyExecution { scan, verification })
}

fn persist_verification_result_state(
    context: &OpenContext,
    active_state_path: &Path,
    scan: &codefire_core::ScanResult,
    verification: &codefire_core::Verification,
) -> Result<(), CliError> {
    write_json_atomic(
        &active_state_path.join("verification.json"),
        &serde_json::to_value(verification)?,
    )?;
    set_open_state(
        context,
        active_state_path,
        verification_open_state(verification, scan),
    )
}

fn verification_open_state(
    verification: &codefire_core::Verification,
    scan: &codefire_core::ScanResult,
) -> &'static str {
    if verification.result != "passed" {
        return "open-burning";
    }
    if scan.changed_atoms.is_empty() && scan.open_fires.is_empty() {
        "open-clean"
    } else {
        "open-consistent"
    }
}

fn run_extinguish(options: &ExtinguishOptions) -> Result<ExtinguishResult, CliError> {
    let context = open_context(&options.path)?;
    let _lock = RepoLock::acquire_with_options(&context.repo_root, &options.lock)?;
    let idempotency_payload = extinguish_idempotency_payload(options, &context);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_extinguish_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let policy = codefire_core::parse_verification_policy(&context.open_dir)?;
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

    let scan_execution = compute_scan(&context.open_dir, !options.dry_run)?;
    let scan = scan_execution.scan;
    let fires_path = active_state_path.join("fires.json");
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
        &policy,
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

    let resolutions_path = active_state_path.join("resolutions.json");
    let mut resolutions: Vec<codefire_core::Resolution> = if resolutions_path.exists() {
        serde_json::from_value(read_json(&resolutions_path)?)?
    } else {
        Vec::new()
    };
    if options.refresh {
        for existing in &mut resolutions {
            if existing.fire_uid == fires[fire_index].fire_uid && existing.status == "active" {
                existing.status = "superseded".to_string();
            }
        }
    }
    resolutions.push(resolution);
    let remaining_open_fires = fires.iter().filter(|fire| fire.status == "open").count();
    write_json_atomic(&fires_path, &serde_json::to_value(fires)?)?;
    write_json_atomic(&resolutions_path, &serde_json::to_value(resolutions)?)?;
    set_open_state(
        &context,
        &active_state_path,
        scan_branch_state(scan.changed_atoms.len(), remaining_open_fires),
    )?;
    let result = ExtinguishResult {
        display_id,
        fire_uid,
        source_atom,
        target_atom,
        reason,
        remaining_open_fire_count: remaining_open_fires,
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_extinguish_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
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
                CliError::Store(error)
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
    let context = open_context(&options.path)?;
    let _lock = RepoLock::acquire_with_options(&context.repo_root, &options.lock)?;
    let idempotency_payload = commit_idempotency_payload(options, &context);
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) =
                load_commit_idempotency(&context.repo_root, key, &idempotency_payload)?
            {
                return Ok(result);
            }
        }
    }
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let verify_execution = compute_verify(&context.open_dir, !options.dry_run)?;
    let verification = verify_execution.verification;

    let objects = context.repo_root.join(".codefire").join("objects");
    let scan = verify_execution.scan;
    let fires: Vec<codefire_core::Fire> = if active_state_path.join("fires.json").exists() {
        serde_json::from_value(read_json(&active_state_path.join("fires.json"))?)?
    } else {
        Vec::new()
    };
    let extinguished_fires = fires
        .iter()
        .filter(|fire| fire.status == "extinguished")
        .map(|fire| fire.fire_uid.clone())
        .collect::<Vec<_>>();
    let resolutions: Vec<codefire_core::Resolution> =
        if active_state_path.join("resolutions.json").exists() {
            serde_json::from_value(read_json(&active_state_path.join("resolutions.json"))?)?
        } else {
            Vec::new()
        };
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
    let policy = codefire_core::parse_verification_policy(&context.open_dir)?;
    let parents = commit_parents_for_open(&context, &active_state_path)?;
    let plan = commit_operation_plan(
        options,
        &context,
        &active_state_path,
        &scan,
        &verification,
        &parents,
        &summary,
    );
    if verification.result != "passed" {
        if options.dry_run {
            return Ok(CommitResult {
                repo_root: context.repo_root,
                open_dir: context.open_dir,
                commit_id: String::new(),
                branch: context.branch,
                dry_run: true,
                blocked: true,
                exit_code: verification_exit_code(&verification),
                plan,
            });
        }
        return Err(CliError::Usage(
            "commit blocked: verification failed or consistency blockers remain".to_string(),
        ));
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

    let manifest_id = codefire_store::store_object(
        &objects,
        "content_manifest",
        build_manifest(&objects, &context.open_dir)?,
    )?;
    let atom_id = codefire_store::store_object(
        &objects,
        "atom_index",
        serde_json::to_value(&scan.atom_index)?,
    )?;
    let trace_id = codefire_store::store_object(
        &objects,
        "trace_graph",
        serde_json::to_value(&scan.trace_graph)?,
    )?;
    let fire_id = codefire_store::store_object(
        &objects,
        "fire_ledger",
        json!({"type": "fire_ledger", "version": 1, "fires": fires}),
    )?;
    let resolution_id = codefire_store::store_object(
        &objects,
        "resolution_ledger",
        json!({"type": "resolution_ledger", "version": 1, "resolutions": resolutions}),
    )?;
    let verification_id = codefire_store::store_object(
        &objects,
        "verification",
        serde_json::to_value(&verification)?,
    )?;
    let policy_id = codefire_store::store_object(
        &objects,
        "policy",
        json!({"type": "policy", "version": 1, "policy": policy}),
    )?;

    let commit = json!({
        "type": "commit",
        "version": 1,
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
            "result": verification.result,
            "open_required_fires": verification.open_required_fires,
            "failed_checks": verification.failed_checks.len(),
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "missing_evidence_refs": verification.missing_evidence_refs.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
        },
        "changed_atoms": scan.changed_atoms,
        "extinguished_fires": extinguished_fires,
        "created_at": now_iso_utc(),
    });
    let commit = signatures::maybe_sign_commit(
        commit,
        options.signer.as_deref(),
        options.key_id.as_deref(),
        commit_signature_required(&context.open_dir)?,
    )?;
    let commit_id = codefire_store::store_object(&objects, "commit", commit)?;

    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["head"] = Value::String(commit_id.clone());
    branch["state"] = Value::String("open-clean".to_string());
    save_branch_record(&context.repo_root, &branch)?;
    let mut registry = context.registry.clone();
    registry["open"]["current_base_commit"] = Value::String(commit_id.clone());
    registry["state"]["last_known"] = Value::String("open-clean".to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    reset_active(&active_state_path, "open-clean")?;

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
        save_commit_idempotency(&context.repo_root, key, &idempotency_payload, &result)?;
    }
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

fn commit_signature_required(open_dir: &Path) -> Result<bool, CliError> {
    let path = open_dir.join("codefire.policy.yaml");
    if !path.exists() {
        return Ok(false);
    }
    let contents = fs::read_to_string(path)?;
    let mut in_commit_policy = false;
    for raw_line in contents.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim_end();
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }
        if !raw_line.starts_with([' ', '\t']) {
            in_commit_policy = stripped == "commit_policy:";
            continue;
        }
        if in_commit_policy {
            if let Some(value) = stripped.strip_prefix("require_commit_signature:") {
                return parse_policy_bool(value.trim());
            }
        }
    }
    Ok(false)
}

fn parse_policy_bool(value: &str) -> Result<bool, CliError> {
    match value
        .trim_matches(['"', '\''])
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(CliError::Usage(format!(
            "invalid boolean value in policy: {value}"
        ))),
    }
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
            {"kind": "verify", "command": "codefire verify --details --json", "target": {"branch": &context.branch}},
        ],
    })
}

struct CommitOperationSummary {
    extinguished_fire_count: usize,
    active_resolution_count: usize,
    evidence_ref_count: usize,
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
    let next_actions = if verification.result != "passed" {
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
        "extinguished_fire_count": summary.extinguished_fire_count,
        "active_resolution_count": summary.active_resolution_count,
        "evidence_ref_count": summary.evidence_ref_count,
        "verification": {
            "result": &verification.result,
            "open_required_fires": verification.open_required_fires,
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
            "failed_checks": verification.failed_checks.len(),
        },
        "objects": [
            "content_manifest",
            "atom_index",
            "trace_graph",
            "fire_ledger",
            "resolution_ledger",
            "verification",
            "policy",
            "commit",
        ],
        "operations": [
            {"kind": "store_objects", "path": context.repo_root.join(".codefire").join("objects")},
            {"kind": "update_branch_head", "branch": &context.branch},
            {"kind": "update_open_registry", "path": &context.registry_path},
            {"kind": "reset_active_state", "path": active_state_path},
        ],
        "next_actions": next_actions,
    })
}

fn commit_parents_for_open(
    context: &OpenContext,
    active_state_path: &Path,
) -> Result<Vec<String>, CliError> {
    let mut parents = vec![required_string(
        &context.registry,
        &["open", "current_base_commit"],
    )?];
    let state_path = active_state_path.join("state.json");
    if state_path.exists() {
        let state = read_json(&state_path)?;
        if let Some(parent) = state.get("pending_merge_parent").and_then(Value::as_str) {
            parents.push(parent.to_string());
        }
    }
    Ok(parents)
}

fn run_verification_commands(
    open_dir: &Path,
    policy: &codefire_core::VerificationPolicy,
) -> Result<Vec<codefire_core::FailedCheck>, CliError> {
    let mut failures = Vec::new();
    for check in &policy.verification {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&check.command)
            .current_dir(open_dir.join(&check.cwd))
            .output()?;
        if !output.status.success() {
            let mut combined = String::new();
            combined.push_str(&String::from_utf8_lossy(&output.stdout));
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            failures.push(codefire_core::FailedCheck {
                id: check.id.clone(),
                command: check.command.clone(),
                output: combined,
            });
        }
    }
    Ok(failures)
}

fn load_base_atom_index(
    objects: &Path,
    base_commit: &str,
) -> Result<codefire_core::AtomIndex, CliError> {
    let commit = codefire_store::read_object(objects, base_commit)?;
    let atom_index_id = required_string(&commit, &["roots", "atom_index"])?;
    let atom_index = codefire_store::read_object(objects, &atom_index_id)?;
    Ok(serde_json::from_value(atom_index)?)
}

fn set_open_state(
    context: &OpenContext,
    active_state_path: &Path,
    state: &str,
) -> Result<(), CliError> {
    let mut registry = context.registry.clone();
    registry["state"]["last_known"] = Value::String(state.to_string());
    write_json_atomic(&context.registry_path, &registry)?;
    let state_path = active_state_path.join("state.json");
    let mut active_state = if state_path.exists() {
        read_json(&state_path)?
    } else {
        json!({})
    };
    active_state["state"] = Value::String(state.to_string());
    write_json_atomic(&state_path, &active_state)?;
    let mut branch = load_branch_record(&context.repo_root, &context.branch)?;
    branch["state"] = Value::String(state.to_string());
    save_branch_record(&context.repo_root, &branch)?;
    Ok(())
}

fn initial_roots(objects: &Path, now: &str) -> Result<Value, CliError> {
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
    let verification = codefire_store::store_object(
        objects,
        "verification",
        json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "duplicate_atom_ids": [],
            "verified_at": now,
        }),
    )?;
    let policy = codefire_store::store_object(
        objects,
        "policy",
        json!({"type": "policy", "version": 1, "policy": {}}),
    )?;
    Ok(json!({
        "content_manifest": content_manifest,
        "atom_index": atom_index,
        "trace_graph": trace_graph,
        "fire_delta": fire_delta,
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

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), CliError> {
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
        file.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
        file.write_all(b"\n")?;
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

fn read_status(start: &Path) -> Result<Status, CliError> {
    let context = open_context(start)?;

    let base = required_string(&context.registry, &["open", "current_base_commit"])?;
    let objects_root = context.repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects_root, &base)?;
    let raw_state = required_string(&context.registry, &["state", "last_known"])?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let fires_path = active_state_path.join("fires.json");
    let open_fires = if fires_path.exists() {
        let fires: Value = read_json(&fires_path)?;
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

fn write_merged_file(root: &Path, rel_path: &str, data: Option<&[u8]>) -> Result<(), CliError> {
    let path = safe_manifest_output_path(root, rel_path)?;
    if let Some(data) = data {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)?;
    } else if path.exists() {
        fs::remove_file(path)?;
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

fn open_context(start: &Path) -> Result<OpenContext, CliError> {
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
    let registry_path = opened_registry_path(&repo_root, &branch);
    let registry = read_json(&registry_path)?;
    let registry_open_path = PathBuf::from(required_string(&registry, &["open", "path"])?);
    if registry_open_path != open_dir {
        return Err(CliError::InvalidMarker(
            "opened_path does not match registry".to_string(),
        ));
    }
    let open_instance_id = required_string(&marker, &["open", "open_instance_id"])?;
    let registry_open_instance_id = required_string(&registry, &["open", "open_instance_id"])?;
    if open_instance_id != registry_open_instance_id {
        return Err(CliError::InvalidMarker(
            "open_instance_id does not match registry".to_string(),
        ));
    }
    Ok(OpenContext {
        repo_root,
        open_dir,
        branch,
        registry_path,
        registry,
    })
}

fn list_branches(start: &Path) -> Result<Vec<Branch>, CliError> {
    let repo_root = find_repo_root(start)?;
    let branches_root = repo_root.join(".codefire").join("branches");
    let objects_root = repo_root.join(".codefire").join("objects");
    let mut branch_paths = Vec::new();
    for entry in fs::read_dir(branches_root)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            branch_paths.push(path);
        }
    }
    branch_paths.sort();

    let mut branches = Vec::with_capacity(branch_paths.len());
    for path in branch_paths {
        let record = read_json(&path)?;
        let name = required_string(&record, &["name"])?;
        let head = required_string(&record, &["head"])?;
        codefire_store::validate_sealed_commit(&objects_root, &head)?;
        let state = record
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("closed")
            .to_string();
        branches.push(Branch { name, head, state });
    }
    Ok(branches)
}

fn load_branch_record(repo_root: &Path, branch: &str) -> Result<Value, CliError> {
    let path = branch_record_path(repo_root, branch);
    if !path.exists() {
        return Err(CliError::Usage(format!("unknown branch: {branch}")));
    }
    read_json(&path)
}

fn save_branch_record(repo_root: &Path, branch: &Value) -> Result<(), CliError> {
    let name = required_string(branch, &["name"])?;
    write_json_atomic(&branch_record_path(repo_root, &name), branch)?;
    codefire_store::store_object(
        &repo_root.join(".codefire").join("objects"),
        "branch",
        branch.clone(),
    )?;
    Ok(())
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

fn materialize_commit(objects: &Path, commit_id: &str, target: &Path) -> Result<(), CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let manifest_id = required_string(&commit, &["roots", "content_manifest"])?;
    let manifest = codefire_store::read_object(objects, &manifest_id)?;
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
        let blob = codefire_store::read_object(objects, &blob_id)?;
        let encoding = required_string(&blob, &["encoding"])?;
        if encoding != "base64" {
            return Err(CliError::InvalidRepository(format!(
                "unsupported blob encoding: {encoding}"
            )));
        }
        let content = required_string(&blob, &["content"])?;
        let output_path = safe_manifest_output_path(target, &rel_path)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, decode_base64(&content)?)?;
    }
    Ok(())
}

fn build_manifest(objects: &Path, open_dir: &Path) -> Result<Value, CliError> {
    let mut entries = Vec::new();
    for rel in rel_files(open_dir)? {
        let data = fs::read(open_dir.join(&rel))?;
        let blob_id = codefire_store::store_object(
            objects,
            "blob",
            json!({
                "type": "blob",
                "version": 1,
                "encoding": "base64",
                "content": encode_base64(&data),
            }),
        )?;
        entries.push(json!({
            "path": to_posix(&rel),
            "kind": "file",
            "mode": "100644",
            "blob": blob_id,
        }));
    }
    Ok(json!({"type": "content_manifest", "version": 1, "entries": entries}))
}

fn source_manifest_unchanged(
    objects: &Path,
    base_commit: &str,
    open_dir: &Path,
) -> Result<bool, CliError> {
    let commit = codefire_store::read_object(objects, base_commit)?;
    let manifest_id = required_string(&commit, &["roots", "content_manifest"])?;
    let base_manifest = codefire_store::read_object(objects, &manifest_id)?;
    let current_manifest = build_manifest_fingerprint(open_dir)?;
    Ok(base_manifest == current_manifest)
}

fn build_manifest_fingerprint(open_dir: &Path) -> Result<Value, CliError> {
    let mut entries = Vec::new();
    for rel in rel_files(open_dir)? {
        let data = fs::read(open_dir.join(&rel))?;
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
        } else if file_type.is_file() && to_posix(&rel) != ".codefire-open" {
            files.push(rel);
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

fn safe_manifest_output_path(target: &Path, rel_path: &str) -> Result<PathBuf, CliError> {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        return Err(CliError::InvalidRepository(format!(
            "manifest path must be relative: {rel_path}"
        )));
    }
    let mut output = target.to_path_buf();
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

fn reset_active(active_state_path: &Path, state: &str) -> Result<(), CliError> {
    for name in [
        "fires.json",
        "resolutions.json",
        "scan.json",
        "verification.json",
    ] {
        let path = active_state_path.join(name);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    write_json_atomic(
        &active_state_path.join("state.json"),
        &json!({"state": state}),
    )?;
    Ok(())
}

#[derive(Debug)]
struct RepoLock {
    path: PathBuf,
}

impl RepoLock {
    fn acquire_with_options(repo_root: &Path, options: &LockOptions) -> Result<Self, CliError> {
        let path = repo_root.join(".codefire").join("locks").join("repo.lock");
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let start = Instant::now();
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(
                        serde_json::to_string_pretty(&json!({
                            "version": 1,
                            "pid": std::process::id(),
                            "created_at": now_iso_utc(),
                        }))?
                        .as_bytes(),
                    )?;
                    file.write_all(b"\n")?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !options.wait {
                        return Err(repo_lock_contention(&path, options));
                    }
                    if timeout.is_some_and(|timeout| start.elapsed() >= timeout) {
                        return Err(repo_lock_contention(&path, options));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }
}

fn repo_lock_contention(path: &Path, options: &LockOptions) -> CliError {
    let owner = read_json(path).ok();
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
    CliError::LockContention(message)
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct FileLock {
    path: PathBuf,
}

impl FileLock {
    fn acquire_with_options(path: PathBuf, options: &LockOptions) -> Result<Self, CliError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let timeout = options.timeout_ms.map(Duration::from_millis);
        let start = Instant::now();
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(
                        serde_json::to_string_pretty(&json!({
                            "version": 1,
                            "pid": std::process::id(),
                            "created_at": now_iso_utc(),
                        }))?
                        .as_bytes(),
                    )?;
                    file.write_all(b"\n")?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !options.wait {
                        return Err(file_lock_contention(&path, options));
                    }
                    if timeout.is_some_and(|timeout| start.elapsed() >= timeout) {
                        return Err(file_lock_contention(&path, options));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(CliError::Io(error)),
            }
        }
    }
}

fn file_lock_contention(path: &Path, options: &LockOptions) -> CliError {
    let owner = read_json(path).ok();
    let mut message = format!("CodeFire resource is locked: {}", path.display());
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
    CliError::LockContention(message)
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
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
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
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
