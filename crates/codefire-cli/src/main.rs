#[cfg(test)]
use fs2::FileExt as Fs2FileExt;
use serde_json::{json, Value};
use std::env;
use std::fs::{self};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Mutex;
use std::time::Instant;

mod active_state;
mod assessment_subject;
mod automation;
mod batch;
mod branch_lifecycle;
mod branch_list;
mod branch_refs;
mod cli_args;
mod cli_error;
mod cli_model;
mod cli_output;
mod close_transaction;
mod close_worktree;
mod command_execution;
mod command_registry;
mod commit;
mod commit_active_reset;
mod compatibility;
mod completion;
mod context;
mod doctor;
mod evidence;
mod exit_code;
mod explain;
mod extinguish;
mod extinguish_ux;
mod fire;
mod help;
mod http;
mod http_tls;
mod idempotency;
mod import_workflow;
mod json_files;
mod limited_yaml;
mod link_batch;
mod local_commands;
mod manifest;
mod merge;
mod merge_patch_idempotency;
mod merge_resolution;
mod metrics;
mod migration;
mod open_clone_idempotency;
mod open_context;
mod open_snapshot;
mod open_transaction;
mod pagination;
mod remote;
mod repo_layout;
mod repo_lock;
mod scan_verify;
mod secret_redaction;
mod secure_worktree;
mod signatures;
mod stable_lock;
mod state_transaction;
mod storage;
mod test_hooks;
mod verification;
mod view;
use automation::{command_result_envelope, status_next_actions_with_prediction};
#[cfg(test)]
use branch_refs::{BranchRefSnapshot, DIRECTORY_ITERATION_SCRATCH_BYTES};
pub(crate) use cli_model::*;
use command_registry::capabilities_data_json;
#[cfg(test)]
use commit_active_reset::commit_active_reset_state_postimage_bytes;
use completion::{completion_script, help_text};
use context::{build_context_pack, parse_context_args, print_context_summary};
#[cfg(test)]
use doctor::{doctor_report_data_json, doctor_report_diagnostics_json, doctor_report_next_actions};
use doctor::{parse_doctor_args, run_doctor};
use evidence::{
    evidence_add_data_json, evidence_batch_data_json, parse_evidence_add_args,
    print_evidence_add_result, run_evidence_add, run_evidence_batch,
};
use exit_code::ExitCode;
use explain::{parse_explain_args, print_explain_result, run_explain};
use http::serve_http;
use idempotency::{idempotency_payload_hash, idempotency_record_path, require_idempotency_key};
use import_workflow::{import_existing_project, import_result_data_json, parse_import_args};
use link_batch::{link_batch_data_json, parse_link_batch_args, run_link_batch};
use merge_patch_idempotency::{
    load_patch_import_idempotency, patch_import_idempotency_payload, save_patch_import_idempotency,
};
use merge_resolution::{
    merge_continue_data_json, merge_continue_diagnostics_json, merge_continue_next_actions,
    parse_merge_continue_args, render_merge_continue, run_merge_continue,
};
use metrics::{attach_metrics, branch_list_metrics, print_metrics, status_metrics};
use migration::{
    migration_report_data_json, migration_report_diagnostics_json, migration_report_next_actions,
    parse_migrate_args, print_migration_report, run_migrate,
};
use open_clone_idempotency::{load_open_idempotency, open_idempotency_payload};
use pagination::{paginate, PageRequest};
use remote::{
    apply_merge_request, list_merge_requests, list_remote_branches, print_remote_doctor_report,
    read_optional_json, remote_doctor_data_json, remote_doctor_diagnostics_json, request_merge,
    review_merge_request, run_remote_doctor, upload_branch, write_object_records,
};
use secure_worktree::{ExistingWorktree, StagedWorktree};
use storage::{
    parse_storage_report_args, print_storage_report, run_storage_report, storage_report_data_json,
    storage_report_diagnostics_json, storage_report_next_actions,
};
use view::{
    diff_commitish_with_options, patch_export_with_options, review_pack_with_options,
    show_commitish, show_commitish_data,
};

#[cfg(test)]
use automation::scan_data_json_with_full;
#[cfg(test)]
use automation::scan_next_actions;
#[cfg(test)]
use automation::status_data_json;
#[cfg(test)]
use automation::verification_data_json_with_filter;
#[cfg(test)]
use batch::batch_extinguish_data_json;
#[cfg(test)]
use batch::parse_extinguish_batch_args;
#[cfg(test)]
use batch::run_extinguish_batch;
#[cfg(test)]
use branch_lifecycle::open_branch_from;
#[cfg(test)]
use branch_lifecycle::suggested_branch_open_path;
use branch_lifecycle::{
    clone_branch, close_branch, close_operation_plan, is_primary_branch, open_branch,
    open_next_step, open_operation_plan, open_target_resolution_error,
    refresh_open_clean_for_branch_operation, resolve_open_target, resolve_path_without_symlinks,
    validate_close_open_marker,
};
#[cfg(test)]
use branch_list::encode_pretty_json_output;
#[cfg(test)]
use branch_list::encode_text_output;
#[cfg(test)]
use branch_list::finish_branch_list_failure;
use branch_list::list_branches;
#[cfg(test)]
use branch_list::list_branches_with_ledger_and_hook;
#[cfg(test)]
use branch_list::project_opened_registries;
#[cfg(test)]
use branch_list::write_branches;
#[cfg(test)]
use branch_list::BranchListDataWire;
#[cfg(test)]
use branch_list::BranchesWire;
#[cfg(test)]
use branch_list::JsonSizeWriter;
#[cfg(test)]
use branch_refs::BranchRefSnapshotError;
#[cfg(test)]
use branch_refs::BranchRefWork;
#[cfg(test)]
use branch_refs::OpenedRegistrySnapshot;
#[cfg(test)]
use branch_refs::RepositoryMetadataSnapshot;
#[cfg(test)]
use cli_args::parse_commit_args;
#[cfg(test)]
use cli_args::parse_extinguish_args;
use cli_args::{
    parse_branch_show_args, parse_clone_args, parse_close_args, parse_common_mutation_option,
    parse_diff_args, parse_init_args, parse_list_args, parse_lock_option, parse_merge_args,
    parse_open_args, parse_patch_export_args, parse_patch_import_args, parse_path_json_args,
    parse_read_only_debug_args, parse_remote_project_args, parse_request_apply_args,
    parse_request_merge_args, parse_request_review_args, parse_review_pack_args, parse_serve_args,
    parse_show_args, parse_upload_args, take_option_value,
};
use cli_error::CliError;
use cli_output::args_want_json;
use cli_output::branch_error_repo_root;
#[cfg(test)]
use cli_output::branch_json;
use cli_output::branch_show_data;
use cli_output::branch_unsupported_command_envelope;
#[cfg(test)]
use cli_output::cli_error_envelope;
use cli_output::command_arg;
#[cfg(test)]
use cli_output::commit_result_data_json;
use cli_output::diff_result_envelope;
use cli_output::is_remote_project_url;
#[cfg(test)]
use cli_output::local_list_data_json;
#[cfg(test)]
use cli_output::lock_contention_envelope;
use cli_output::merge_in_progress_line;
use cli_output::merge_request_list_data_json;
use cli_output::patch_export_result_data;
#[cfg(test)]
use cli_output::plan_result_envelope;
use cli_output::print_branch_show;
use cli_output::print_cli_error_json;
use cli_output::print_data_result_json;
use cli_output::print_lock_contention_json;
use cli_output::print_plan_result_json;
use cli_output::print_status;
use cli_output::remote_branch_list_data_json;
#[cfg(test)]
use cli_output::render_scan;
use cli_output::review_pack_result_data;
use cli_output::status_data_json_with_prediction;
use cli_output::unscanned_changes_line;
#[cfg(test)]
use commit::run_commit;
use commit::{
    commit_transaction_path, materialize_commit, read_commit_transaction,
    reject_pending_commit_transaction_for_preview, validate_commit_transaction_record,
    write_pending_commit_transaction, COMMIT_TRANSACTION_BYTE_LIMIT,
};
#[cfg(test)]
use commit_active_reset::build_commit_active_reset_authority;
#[cfg(test)]
use commit_active_reset::commit_active_reset_complete;
#[cfg(test)]
use commit_active_reset::reset_commit_active_state;
#[cfg(test)]
use exit_code::verification_exit_code;
use extinguish::{run_extinguish, validate_evidence_refs};
#[cfg(test)]
use fire::active_ledger_transaction::active_ledger_request_hash;
#[cfg(test)]
use fire::active_ledger_transaction::apply_active_ledger_transition;
#[cfg(test)]
use fire::active_ledger_transaction::recover_active_ledger_transition;
#[cfg(test)]
use fire::active_ledger_transaction::ActiveLedgerWrite;
#[cfg(test)]
use fire::run_fire;
use help::{command_help_for_args, command_name_for_args, subcommand_help, wants_help};
#[cfg(test)]
use json_files::write_active_json_atomic_bounded;
#[cfg(test)]
use json_files::write_json_atomic_bounded;
use json_files::{
    read_active_json_optional, read_active_json_required, unique_temp_path,
    write_active_json_atomic, write_bytes_atomic, write_json_atomic, REPOSITORY_JSON_BYTE_LIMIT,
};
use local_commands::{local_workflow_handler, read_batch_file};
#[cfg(test)]
use manifest::build_manifest_plan;
#[cfg(test)]
use manifest::to_posix;
use manifest::{
    rel_files, safe_relative_manifest_path, source_manifest_unchanged_existing,
    source_manifest_unchanged_secure,
};
use merge::ancestor_distances;
use merge::branch_operation_error;
use merge::branch_state;
use merge::common_ancestor;
use merge::ensure_sealed_source_state;
use merge::merge_branch;
#[cfg(test)]
use merge::merge_next_actions;
use merge::merge_next_steps;
use merge::merge_result_json_value;
use merge::render_merge_dry_run;
#[cfg(test)]
use merge::render_merge_result_json;
use merge::with_branch_operation_context;
use merge::write_merged_file;
#[cfg(test)]
use metrics::metrics_wire;
#[cfg(test)]
use metrics::scan_metrics;
#[cfg(test)]
use metrics::verification_metrics;
#[cfg(test)]
use open_context::open_context_for_recovery;
use open_context::{
    open_context, open_context_for_status, optional_repo_root, validated_active_state_dir,
};
#[cfg(test)]
use remote::copy_object_graph;
#[cfg(test)]
use remote::load_remote_branch;
#[cfg(test)]
use remote::parse_cf_url;
#[cfg(test)]
use remote::remote_dirs;
#[cfg(test)]
use remote::RequestApplyOptions;
#[cfg(test)]
use remote::RequestMergeOptions;
#[cfg(test)]
use remote::RequestReviewOptions;
#[cfg(test)]
use remote::UploadOptions;
use repo_lock::{
    acquire_locked_open_context, acquire_locked_open_context_for_recovery,
    repo_lock_contention_with_owner, repo_lock_owner_path, FileLock, RepoLock,
    REPO_LOCK_OWNER_BYTE_LIMIT,
};
use scan_verify::compute_scan;
use scan_verify::compute_scan_locked;
use scan_verify::compute_scan_with_cache;
use scan_verify::compute_verify;
#[cfg(test)]
use scan_verify::run_scan;
#[cfg(test)]
use scan_verify::run_scan_with_lock_options;
#[cfg(test)]
use scan_verify::run_verify;
#[cfg(test)]
use scan_verify::run_verify_with_lock_options;
#[cfg(test)]
use scan_verify::verification_open_state;
use scan_verify::worktree_manifest_digest;
#[cfg(test)]
use state_transaction::begin_state_transaction;
#[cfg(test)]
use state_transaction::begin_state_transaction_with_request_hash;
#[cfg(test)]
use state_transaction::complete_state_transaction;
#[cfg(test)]
use state_transaction::state_transaction_path;
#[cfg(test)]
use state_transaction::update_state_transaction_phase;
#[cfg(test)]
use state_transaction::StateTransactionAuthority;
#[cfg(test)]
use state_transaction::ACTIVE_LEDGER_STATE_TRANSACTION_PHASES;
use state_transaction::{
    ensure_active_ledger_state_transaction_request, invalidate_active_verification,
    set_open_state_transactional, state_transaction_transition_is_supported,
    validate_pending_state_transaction_for_recovery, validate_state_transaction_record,
    STATE_TRANSACTION_BYTE_LIMIT,
};
#[cfg(test)]
use std::collections::BTreeSet;
#[cfg(test)]
use std::time::Duration;
use test_hooks::maybe_change_open_target_before_use_for_test;
use test_hooks::maybe_fail_commit_transaction_for_test;
use test_hooks::maybe_insert_nested_open_target_symlink_for_test;
use test_hooks::maybe_swap_close_target_before_delete_for_test;
#[cfg(test)]
use test_hooks::set_active_state_swap_after_validation_for_test;
#[cfg(test)]
use test_hooks::set_close_target_swap_injection_for_test;
#[cfg(test)]
use test_hooks::set_commit_candidate_corruption_for_test;
#[cfg(test)]
use test_hooks::set_commit_transaction_failure_for_test;
#[cfg(test)]
use test_hooks::set_commit_worktree_mutation_for_test;
#[cfg(test)]
use test_hooks::set_locked_open_context_mutation_for_test;
#[cfg(test)]
use test_hooks::set_open_target_nested_symlink_injection_for_test;
#[cfg(test)]
use test_hooks::set_open_target_pre_use_mutation_for_test;
#[cfg(test)]
use test_hooks::set_state_transaction_failure_for_test;
#[cfg(test)]
use test_hooks::set_validated_store_swap_before_verify_persist_for_test;
#[cfg(test)]
use test_hooks::CommitCandidateCorruptionKind;
#[cfg(test)]
use verification::parse_verify_args;
#[cfg(test)]
use view::DiffAlgorithm;
#[cfg(test)]
use view::DiffOptions;
#[cfg(test)]
use view::PatchExportOptions;
#[cfg(test)]
use view::ReviewPackOptions;
#[cfg(test)]
use view::DEFAULT_PATCH_EXPORT_MAX_FILE_BYTES;
#[cfg(test)]
use view::DEFAULT_PATCH_EXPORT_MAX_PAYLOAD_BYTES;
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
                if let Some(line) = open_next_step(&options.new_branch) {
                    println!("{line}");
                }
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
                print!("{}", merge_next_steps(&result));
            } else {
                println!(
                    "merged {} into {} with conflicts; target is open-burning",
                    options.source_branch, options.target_branch
                );
                for conflict in &result.conflicts {
                    println!("  conflict: {conflict}");
                }
                print!("{}", merge_next_steps(&result));
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
                if let Some(line) = merge_in_progress_line(&options.path) {
                    println!("{line}");
                }
                let stale = data["scan_prediction"]["stale_resolution_count"]
                    .as_u64()
                    .unwrap_or(0);
                if stale > 0 {
                    println!(
                        "Stale resolutions: {stale}; run `codefire verify` to list them with the refresh command"
                    );
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
