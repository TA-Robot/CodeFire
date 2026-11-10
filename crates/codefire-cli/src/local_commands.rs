//! Handlers for the local loop commands: scan, verify, fire, extinguish, commit.

use crate::automation::{
    command_result_envelope, scan_data_json_with_full, scan_diagnostics_json, scan_next_actions,
    verification_data_json_with_filter, verification_diagnostics_json_with_filter,
    verification_next_actions,
};
use crate::batch::{
    batch_extinguish_data_json, batch_template_data_json, has_batch_extinguish_arg,
    has_batch_template_arg, parse_batch_template_args, parse_extinguish_batch_args,
    print_batch_template_result, run_batch_template, run_extinguish_batch,
};
use crate::cli_args::{parse_commit_args, parse_extinguish_args, parse_path_json_args};
use crate::cli_error::CliError;
use crate::cli_output::{
    args_want_json, merge_in_progress_line, print_cli_error_json, print_commit_result_json,
    print_plan_result_json, print_scan, stale_resolution_count,
};
use crate::commit::run_commit;
use crate::exit_code::verification_exit_code;
use crate::extinguish::run_extinguish;
use crate::extinguish_ux::{
    has_all_matching_extinguish_arg, has_interactive_extinguish_arg,
    parse_all_matching_extinguish_args, parse_interactive_extinguish_args,
    prepare_extinguish_options, print_all_matching_extinguish_result,
    print_interactive_extinguish_result, run_all_matching_extinguish, run_interactive_extinguish,
};
use crate::fire::{
    fire_batch_data_json, fire_data_json, parse_fire_args, parse_fire_batch_args,
    print_fire_result, run_fire, run_fire_batch,
};
use crate::metrics::{attach_metrics, print_metrics, scan_metrics, verification_metrics};
use crate::open_context::open_context;
use crate::scan_verify::{run_scan_with_lock_options, run_verify_with_lock_options};
use crate::verification::{parse_verify_args, print_verification};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub(crate) fn read_batch_file(path: &Path, operation: &'static str) -> Result<String, CliError> {
    fs::read_to_string(path).map_err(|error| CliError::BatchFileRead {
        operation,
        path: path.to_path_buf(),
        kind: error.kind(),
        message: error.to_string(),
    })
}

pub(crate) type CommandHandler = fn(&[String]) -> Result<(), CliError>;

pub(crate) fn local_workflow_handler(command: &str) -> Option<CommandHandler> {
    match command {
        "scan" => Some(run_scan_command),
        "verify" => Some(run_verify_command),
        "fire" => Some(run_fire_command),
        "extinguish" => Some(run_extinguish_command),
        "commit" => Some(run_commit_command),
        _ => None,
    }
}

pub(crate) fn run_scan_command(args: &[String]) -> Result<(), CliError> {
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
        if let Some(line) = merge_in_progress_line(&options.path) {
            println!("{line}");
        }
        let stale = stale_resolution_count(&options.path, &scan);
        if stale > 0 {
            println!(
                "Stale resolutions: {stale} recorded resolution(s) no longer match the current files; \
                 codefire verify lists them with the refresh command"
            );
        }
        if let Some(metrics) = metrics.as_ref() {
            print_metrics(metrics);
        }
    }
    Ok(())
}

pub(crate) fn run_verify_command(args: &[String]) -> Result<(), CliError> {
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

pub(crate) fn run_fire_command(args: &[String]) -> Result<(), CliError> {
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

pub(crate) fn run_extinguish_command(args: &[String]) -> Result<(), CliError> {
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

pub(crate) fn print_extinguish_batch_item_summary(plan: &Value) {
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

pub(crate) fn run_commit_command(args: &[String]) -> Result<(), CliError> {
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
            if let Some(report) = &result.blocked_report {
                print!("{report}");
            }
            println!("Nothing was sealed. Clear the blocking checks above, then run commit again.");
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
