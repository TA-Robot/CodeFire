use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

mod automation;
mod context;
mod exit_code;
mod http;
mod remote;
mod view;
use automation::{
    command_result_envelope, scan_data_json, scan_diagnostics_json, scan_next_actions,
    status_data_json, status_next_actions, verification_data_json, verification_diagnostics_json,
    verification_next_actions,
};
use context::{build_context_pack, parse_context_args, print_context_summary};
use exit_code::{
    core_error_exit_code, store_error_exit_code, usage_exit_code, verification_exit_code, ExitCode,
};
use http::{http_json, http_remote_path, parse_cf_http_url, serve_http};
use remote::{
    apply_merge_request, copy_object_graph, list_merge_requests, list_remote_branches,
    load_remote_branch, parse_cf_url, remote_dirs, request_merge, review_merge_request,
    upload_branch, write_object_records, RemoteProjectOptions, RequestApplyOptions,
    RequestMergeOptions, RequestReviewOptions, UploadOptions,
};
use view::{
    diff_commitish_with_options, manifest_contents, patch_export_with_options,
    review_pack_with_options, show_commitish, DiffAlgorithm, DiffOptions, PatchExportOptions,
    ReviewPackOptions,
};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        if !matches!(error, CliError::VerificationFailed(_)) {
            eprintln!("error: {error}");
        }
        std::process::exit(error.exit_code());
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match args.first().map(String::as_str) {
        Some("atom-index") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let index = codefire_core::build_atom_index(&start)?;
            println!("{}", serde_json::to_string_pretty(&index)?);
            Ok(())
        }
        Some("trace-graph") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let trace_graph = codefire_core::current_trace_graph(&start)?;
            println!("{}", serde_json::to_string_pretty(&trace_graph)?);
            Ok(())
        }
        Some("missing-links") => {
            let start = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let missing = codefire_core::current_required_link_missing(&start)?;
            println!("{}", serde_json::to_string_pretty(&missing)?);
            Ok(())
        }
        Some("context") => {
            let options = parse_context_args(&args[1..])?;
            let pack = build_context_pack(&options)?;
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
                        Vec::new(),
                    ))?
                );
            } else {
                print_context_summary(&pack.data);
            }
            Ok(())
        }
        Some("scan") => {
            let options = parse_path_json_args(&args[1..], "scan")?;
            let scan = run_scan(&options.path)?;
            if options.json_output {
                let repo_root = open_context(&options.path).ok().map(|context| context.repo_root);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "scan",
                        true,
                        0,
                        repo_root.as_deref(),
                        scan_data_json(&scan),
                        scan_diagnostics_json(&scan),
                        scan_next_actions(&scan),
                    ))?
                );
            } else {
                print_scan(&scan);
            }
            Ok(())
        }
        Some("verify") => {
            let options = parse_verify_args(&args[1..])?;
            let verification = run_verify(&options.path)?;
            let exit_code = verification_exit_code(&verification);
            if options.json_output {
                let repo_root = open_context(&options.path).ok().map(|context| context.repo_root);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "verify",
                        verification.result == "passed",
                        exit_code.code(),
                        repo_root.as_deref(),
                        verification_data_json(&verification),
                        verification_diagnostics_json(&verification),
                        verification_next_actions(&verification),
                    ))?
                );
            } else {
                print_verification(&verification, options.details);
            }
            if verification.result == "passed" {
                Ok(())
            } else {
                Err(CliError::VerificationFailed(exit_code))
            }
        }
        Some("extinguish") => {
            let options = parse_extinguish_args(&args[1..])?;
            let result = run_extinguish(&options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
            } else if options.dry_run {
                println!(
                    "extinguish dry-run: {} would be {}",
                    result.display_id,
                    if options.refresh {
                        "refreshed"
                    } else {
                        "extinguished"
                    }
                );
            } else {
                println!(
                    "{} {}",
                    if options.refresh {
                        "refreshed"
                    } else {
                        "extinguished"
                    },
                    result.display_id
                );
            }
            Ok(())
        }
        Some("commit") => {
            let options = parse_commit_args(&args[1..])?;
            let result = run_commit(&options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
            } else if options.dry_run {
                println!("commit dry-run: branch {} would be sealed", result.branch);
                println!(
                    "Changed atoms: {}",
                    result.plan["changed_atoms"].as_array().map(Vec::len).unwrap_or(0)
                );
            } else {
                println!("Sealed commit created.");
                println!("Commit: {}", result.commit_id);
                println!("Branch: {}", result.branch);
                println!("State: open-clean");
            }
            Ok(())
        }
        Some("clone") => {
            let options = parse_clone_args(&args[1..])?;
            let result = clone_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            let result = upload_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            let options = parse_remote_project_args(&args[1..], "list")?;
            let branches = list_remote_branches(&options.project_url)?;
            for branch in branches {
                println!("{}\t{}", branch.name, branch.head);
            }
            Ok(())
        }
        Some("request-merge") => {
            let options = parse_request_merge_args(&args[1..])?;
            let result = request_merge(&options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            let requests = list_merge_requests(&options.project_url)?;
            for request in requests {
                println!(
                    "{}\t{}\t{}\t{}",
                    request.id, request.status, request.source_url, request.target_url
                );
            }
            Ok(())
        }
        Some("request-review") => {
            let options = parse_request_review_args(&args[1..])?;
            let result = review_merge_request(&options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            let result = apply_merge_request(&options)?;
            if options.json_output {
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            let target = args.get(1).ok_or_else(|| {
                CliError::Usage("usage: codefire-rs show <branch-or-commit>".to_string())
            })?;
            let repo_root = optional_repo_root(&env::current_dir()?);
            let output = show_commitish(repo_root.as_deref(), target)?;
            print!("{output}");
            Ok(())
        }
        Some("diff") => {
            let options = parse_diff_args(&args[1..])?;
            let repo_root = optional_repo_root(&env::current_dir()?);
            let output = diff_commitish_with_options(
                repo_root.as_deref(),
                &options.left,
                &options.right,
                &options.diff,
            )?;
            print!("{output}");
            Ok(())
        }
        Some("review-pack") => {
            let options = parse_review_pack_args(&args[1..])?;
            let repo_root = optional_repo_root(&env::current_dir()?);
            let output = review_pack_with_options(repo_root.as_deref(), &options.review)?;
            if let Some(path) = options.output {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, output)?;
            } else {
                print!("{output}");
            }
            Ok(())
        }
        Some("patch") => match args.get(1).map(String::as_str) {
            Some("export") => {
                let options = parse_patch_export_args(&args[2..])?;
                let repo_root = optional_repo_root(&env::current_dir()?);
                let output = patch_export_with_options(repo_root.as_deref(), &options.patch)?;
                if let Some(path) = options.output {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(path, output)?;
                } else {
                    print!("{output}");
                }
                Ok(())
            }
            Some("import") => {
                let options = parse_patch_import_args(&args[2..])?;
                let result = import_patch(&env::current_dir()?, &options)?;
                if options.json_output {
                    println!("{}", serde_json::to_string_pretty(&result)?);
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
                "usage: codefire-rs patch export <source> [--base <base>] [--output <path>] | codefire-rs patch import <patch-file> [--dry-run] [--json]".to_string(),
            )),
        },
        Some("merge") => {
            let options = parse_merge_args(&args[1..])?;
            let result = merge_branch(&env::current_dir()?, &options)?;
            if options.json_output {
                println!("{}", render_merge_result_json(&result)?);
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
                println!("{}", serde_json::to_string_pretty(&result.plan)?);
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
            Some("list") => {
                let start = args
                    .get(2)
                    .map(PathBuf::from)
                    .unwrap_or(env::current_dir()?);
                let branches = list_branches(&start)?;
                print_branches(&branches);
                Ok(())
            }
            Some(command) => Err(CliError::Usage(format!(
                "unsupported branch command: {command}"
            ))),
            None => Err(CliError::Usage(
                "missing branch command; expected: list".to_string(),
            )),
        },
        Some("status") => {
            let options = parse_path_json_args(&args[1..], "status")?;
            let status = read_status(&options.path)?;
            if options.json_output {
                let repo_root = open_context(&options.path).ok().map(|context| context.repo_root);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&command_result_envelope(
                        "status",
                        true,
                        0,
                        repo_root.as_deref(),
                        status_data_json(&status),
                        Vec::new(),
                        status_next_actions(&status),
                    ))?
                );
            } else {
                print_status(&status);
            }
            Ok(())
        }
        Some("--version") | Some("version") => {
            println!("codefire-rs foundation {}", codefire_core::VERSION);
            Ok(())
        }
        Some(command) => Err(CliError::Usage(format!("unsupported command: {command}"))),
        None => {
            println!("codefire-rs foundation {}", codefire_core::VERSION);
            Ok(())
        }
    }
}

#[derive(Debug)]
enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Core(codefire_core::CoreError),
    Store(codefire_store::StoreError),
    Usage(String),
    VerificationFailed(ExitCode),
    NotOpen(PathBuf),
    InvalidMarker(String),
    InvalidRepository(String),
    LockContention(String),
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            CliError::Io(_) => ExitCode::GenericFailure,
            CliError::Json(_) => ExitCode::RepositoryCorruption,
            CliError::Core(error) => core_error_exit_code(error),
            CliError::Store(error) => store_error_exit_code(error),
            CliError::Usage(message) => usage_exit_code(message),
            CliError::VerificationFailed(exit_code) => *exit_code,
            CliError::NotOpen(_) => ExitCode::InvalidUsageOrConfig,
            CliError::InvalidMarker(_) | CliError::InvalidRepository(_) => {
                ExitCode::RepositoryCorruption
            }
            CliError::LockContention(_) => ExitCode::LockContention,
        }
        .code()
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
            CliError::VerificationFailed(_) => write!(f, "verification failed"),
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

#[derive(Debug, PartialEq, Eq)]
struct Status {
    branch: String,
    state: String,
    base: String,
    open_fires: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Branch {
    name: String,
    head: String,
    state: String,
}

#[derive(Debug)]
struct InitOptions {
    path: PathBuf,
    force: bool,
}

#[derive(Debug)]
struct InitResult {
    repo_root: PathBuf,
    main_commit: String,
}

#[derive(Debug)]
struct OpenOptions {
    branch: String,
    path: PathBuf,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct OpenResult {
    branch: String,
    open_dir: PathBuf,
    plan: Value,
}

#[derive(Debug)]
struct VerifyOptions {
    path: PathBuf,
    details: bool,
    json_output: bool,
}

#[derive(Debug)]
struct PathJsonOptions {
    path: PathBuf,
    json_output: bool,
}

#[derive(Debug)]
struct ExtinguishOptions {
    path: PathBuf,
    fire_id: String,
    resolution: String,
    rationale: String,
    evidence: String,
    refresh: bool,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct ExtinguishResult {
    display_id: String,
    plan: Value,
}

#[derive(Debug)]
struct CommitOptions {
    path: PathBuf,
    message: String,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct CommitResult {
    commit_id: String,
    branch: String,
    plan: Value,
}

#[derive(Debug)]
struct CloneOptions {
    source: String,
    new_branch: String,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct CloneResult {
    plan: Value,
}

#[derive(Debug)]
struct DiffArgs {
    left: String,
    right: String,
    diff: DiffOptions,
}

#[derive(Debug)]
struct ReviewPackArgs {
    review: ReviewPackOptions,
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct PatchExportArgs {
    patch: PatchExportOptions,
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct PatchImportOptions {
    path: PathBuf,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct ServeOptions {
    storage_root: PathBuf,
    host: String,
    port: u16,
}

#[derive(Debug)]
struct MergeOptions {
    source_branch: String,
    target_branch: String,
    dry_run: bool,
    json_output: bool,
}

#[derive(Debug)]
struct MergeResult {
    source_branch: String,
    target_branch: String,
    source_head: String,
    target_head: String,
    base: String,
    target_dir: PathBuf,
    file_actions: Vec<MergeFileAction>,
    conflicts: Vec<String>,
    semantic_conflicts: Vec<SemanticConflictCandidate>,
    dry_run: bool,
    applied: bool,
}

#[derive(Debug)]
struct MergeFileAction {
    path: String,
    action: MergeAction,
    content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeAction {
    WriteSource,
    DeleteTarget,
    WriteConflictMarkers,
}

impl MergeAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::WriteSource => "write_source",
            Self::DeleteTarget => "delete_target",
            Self::WriteConflictMarkers => "write_conflict_markers",
        }
    }
}

#[derive(Debug)]
struct SemanticConflictCandidate {
    atom_id: String,
    base_hash: Option<String>,
    source_hash: Option<String>,
    target_hash: Option<String>,
    source_path: Option<String>,
    target_path: Option<String>,
}

struct OpenContext {
    repo_root: PathBuf,
    open_dir: PathBuf,
    branch: String,
    registry_path: PathBuf,
    registry: Value,
}

struct ScanExecution {
    context: OpenContext,
    active_state_path: PathBuf,
    scan: codefire_core::ScanResult,
    fires: Vec<codefire_core::Fire>,
}

fn print_status(status: &Status) {
    println!("Branch: {}", status.branch);
    println!("State: {}", status.state);
    println!("Base: {}", status.base);
    println!("Open fires: {}", status.open_fires);
}

fn print_branches(branches: &[Branch]) {
    for branch in branches {
        println!("{}\t{}\t{}", branch.name, branch.head, branch.state);
    }
}

fn print_scan(scan: &codefire_core::ScanResult) {
    let state = if scan.changed_atoms.is_empty() && scan.open_fires.is_empty() {
        "open-clean"
    } else {
        "open-burning"
    };
    println!("Branch state: {state}");
    println!("Changed atoms:");
    for atom_id in &scan.changed_atoms {
        println!("  {atom_id}");
    }
    println!("Open fires:");
    for fire in &scan.open_fires {
        println!(
            "  {}  {} -> {}  {}",
            fire.display_id, fire.source.atom_id, fire.target.atom_id, fire.reason
        );
    }
}

fn print_verification(verification: &codefire_core::Verification, details: bool) {
    if verification.result == "passed" {
        println!("Verification passed.");
        return;
    }
    let mut blocking = Vec::new();
    if verification.open_required_fires > 0 {
        blocking.push("open fires");
    }
    if !verification.missing_required_links.is_empty() {
        blocking.push("missing required links");
    }
    if !verification.stale_resolutions.is_empty() {
        blocking.push("stale resolutions");
    }
    if !verification.duplicate_atom_ids.is_empty() {
        blocking.push("duplicate atom ids");
    }
    if !verification.failed_checks.is_empty() {
        blocking.push("failed checks");
    }
    println!("Verification failed.");
    println!(
        "Blocking checks: {}",
        if blocking.is_empty() {
            "none".to_string()
        } else {
            blocking.join(", ")
        }
    );
    println!("Open fires: {}", verification.open_required_fires);
    println!(
        "Missing required links: {}",
        verification.missing_required_links.len()
    );
    println!(
        "Stale resolutions: {}",
        verification.stale_resolutions.len()
    );
    println!(
        "Duplicate atom ids: {}",
        verification.duplicate_atom_ids.len()
    );
    println!("Failed checks: {}", verification.failed_checks.len());
    if details {
        print_verification_details(verification);
    }
}

fn print_verification_details(verification: &codefire_core::Verification) {
    if !verification.missing_required_links.is_empty() {
        println!("Missing required link details:");
        for item in verification.missing_required_links.iter().take(5) {
            println!(
                "  {} requires {} -> {} min {}",
                item.atom_id, item.required_type, item.target_kind, item.min
            );
        }
    }
    if !verification.stale_resolutions.is_empty() {
        println!("Stale resolution details:");
        for item in verification.stale_resolutions.iter().take(5) {
            println!("  {}: {}", item.resolution_uid, item.reason);
        }
    }
    if !verification.duplicate_atom_ids.is_empty() {
        println!("Duplicate Atom ID details:");
        for atom_id in verification.duplicate_atom_ids.iter().take(5) {
            println!("  {atom_id}");
        }
    }
    if !verification.failed_checks.is_empty() {
        println!("Failed check details:");
        for item in verification.failed_checks.iter().take(5) {
            let summary = item.output.trim().lines().next().unwrap_or_default();
            println!("  {}: {} {}", item.id, item.command, summary);
        }
    }
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
    for value in args {
        match value.as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported open option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
    }
    match positional.as_slice() {
        [branch, path] => Ok(OpenOptions {
            branch: branch.to_string(),
            path: PathBuf::from(path),
            dry_run,
            json_output,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs open <branch> <path> [--dry-run] [--json]".to_string(),
        )),
    }
}

fn parse_verify_args(args: &[String]) -> Result<VerifyOptions, CliError> {
    let mut path = None;
    let mut details = false;
    let mut json_output = false;
    for arg in args {
        if arg == "--details" {
            details = true;
        } else if arg == "--json" {
            json_output = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err(CliError::Usage(format!(
                "unexpected verify argument: {arg}"
            )));
        }
    }
    Ok(VerifyOptions {
        path: path.unwrap_or(env::current_dir()?),
        details,
        json_output,
    })
}

fn parse_path_json_args(args: &[String], command: &str) -> Result<PathJsonOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    for arg in args {
        if arg == "--json" {
            json_output = true;
        } else if arg.starts_with("--") {
            return Err(CliError::Usage(format!(
                "unsupported {command} option: {arg}"
            )));
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err(CliError::Usage(format!(
                "unexpected {command} argument: {arg}"
            )));
        }
    }
    Ok(PathJsonOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
    })
}

fn parse_extinguish_args(args: &[String]) -> Result<ExtinguishOptions, CliError> {
    let mut path = None;
    let mut fire_id = None;
    let mut resolution = "addressed".to_string();
    let mut rationale = String::new();
    let mut evidence = String::new();
    let mut refresh = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            "--resolution" => {
                index += 1;
                resolution = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--resolution requires a value".to_string()))?
                    .to_string();
            }
            "--rationale" => {
                index += 1;
                rationale = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--rationale requires a value".to_string()))?
                    .to_string();
            }
            "--evidence" => {
                index += 1;
                evidence = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--evidence requires a value".to_string()))?
                    .to_string();
            }
            "--refresh" => refresh = true,
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
            CliError::Usage("usage: codefire-rs extinguish <fire-id> [--path <open-dir>] --resolution <type> (--rationale <text>|--evidence <text>)".to_string())
        })?,
        resolution,
        rationale,
        evidence,
        refresh,
        dry_run,
        json_output,
    })
}

fn parse_commit_args(args: &[String]) -> Result<CommitOptions, CliError> {
    let mut path = None;
    let mut message = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--path requires a value".to_string())
                })?));
            }
            "-m" | "--message" => {
                index += 1;
                message = Some(
                    args.get(index)
                        .ok_or_else(|| CliError::Usage("-m requires a value".to_string()))?
                        .to_string(),
                );
            }
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
        json_output,
    })
}

fn parse_clone_args(args: &[String]) -> Result<CloneOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    for value in args {
        match value.as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported clone option: {option}"
                )));
            }
            _ => positional.push(value.clone()),
        }
    }
    match positional.as_slice() {
        [source, new_branch] => Ok(CloneOptions {
            source: source.to_string(),
            new_branch: new_branch.to_string(),
            dry_run,
            json_output,
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs clone <source-branch> <new-branch> [--dry-run] [--json]"
                .to_string(),
        )),
    }
}

fn parse_diff_args(args: &[String]) -> Result<DiffArgs, CliError> {
    let mut positional = Vec::new();
    let mut algorithm = DiffAlgorithm::Myers;
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
                rename_detection,
                atom_diff,
                trace_diff,
                impact_diff,
                json_output,
            },
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs diff [--algorithm myers|patience|histogram] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <left> <right>".to_string(),
        )),
    }
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
                    "usage: codefire-rs review-pack <source> [--base <base>] [--output <path>] [--algorithm myers|patience|histogram] [--no-rename-detection]"
                        .to_string(),
                )
            })?,
            base,
            algorithm,
            rename_detection,
        },
        output,
    })
}

fn parse_patch_export_args(args: &[String]) -> Result<PatchExportArgs, CliError> {
    let mut source = None;
    let mut base = None;
    let mut output = None;
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
                    "usage: codefire-rs patch export <source> [--base <base>] [--output <path>]"
                        .to_string(),
                )
            })?,
            base,
        },
        output,
    })
}

fn parse_patch_import_args(args: &[String]) -> Result<PatchImportOptions, CliError> {
    let mut path = None;
    let mut dry_run = false;
    let mut json_output = false;
    for value in args {
        match value.as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
    }
    Ok(PatchImportOptions {
        path: path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire-rs patch import <patch-file> [--dry-run] [--json]".to_string(),
            )
        })?,
        dry_run,
        json_output,
    })
}

fn parse_upload_args(args: &[String]) -> Result<UploadOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs upload <branch> <cf-url> [--dry-run] [--json]".to_string(),
        )),
    }
}

fn parse_remote_project_args(
    args: &[String],
    command: &str,
) -> Result<RemoteProjectOptions, CliError> {
    let positional = positional_args(args)?;
    match positional.as_slice() {
        [project_url] => Ok(RemoteProjectOptions {
            project_url: project_url.clone(),
        }),
        _ => Err(CliError::Usage(format!(
            "usage: codefire-rs {command} <cf-project-url>"
        ))),
    }
}

fn parse_request_merge_args(args: &[String]) -> Result<RequestMergeOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs request-merge <source-url> <target-url> [--dry-run] [--json]"
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
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs request-review <project-url> <mr-id> [--reviewer <name>] [--decision approve|reject] [--comment <text>] [--dry-run] [--json]".to_string(),
        )),
    }
}

fn parse_request_apply_args(args: &[String]) -> Result<RequestApplyOptions, CliError> {
    let mut positional = Vec::new();
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
        }),
        _ => Err(CliError::Usage(
            "usage: codefire-rs request-apply <project-url> <mr-id> [--dry-run] [--json]"
                .to_string(),
        )),
    }
}

fn parse_serve_args(args: &[String]) -> Result<ServeOptions, CliError> {
    let mut storage_root = None;
    let mut host = "127.0.0.1".to_string();
    let mut port = 8080u16;
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
            "--tls-cert" | "--tls-key" => {
                return Err(CliError::Usage(
                    "Rust HTTP server does not implement TLS yet; use cf+http".to_string(),
                ));
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
    Ok(ServeOptions {
        storage_root: storage_root.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire-rs serve <storage-root> [--host <host>] [--port <port>]"
                    .to_string(),
            )
        })?,
        host,
        port,
    })
}

fn positional_args(args: &[String]) -> Result<Vec<String>, CliError> {
    let mut positional = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let value = &args[index];
        if value.starts_with("--") {
            skip_ignored_option(args, &mut index)?;
        } else {
            positional.push(value.clone());
        }
        index += 1;
    }
    Ok(positional)
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

fn parse_merge_args(args: &[String]) -> Result<MergeOptions, CliError> {
    let mut source_branch = None;
    let mut target_branch = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--into" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CliError::Usage("--into requires a target branch".to_string())
                })?;
                target_branch = Some(value.clone());
            }
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
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
                "usage: codefire-rs merge <source-branch> --into <target-branch> [--dry-run] [--json]"
                    .to_string(),
            )
        })?,
        target_branch: target_branch.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire-rs merge <source-branch> --into <target-branch> [--dry-run] [--json]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
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
    let _lock = RepoLock::acquire(&repo_root)?;
    let mut result = build_merge_result(&repo_root, options)?;
    if options.dry_run {
        return Ok(result);
    }
    apply_merge_result(&repo_root, &mut result)?;
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
        let content = conflict_content(
            target_data.map(Vec::as_slice),
            source_data.map(Vec::as_slice),
        );
        file_actions.push(MergeFileAction {
            path,
            action: MergeAction::WriteConflictMarkers,
            content: Some(content),
        });
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
        "  file actions: {} conflicts: {} semantic candidates: {}\n",
        result.file_actions.len(),
        result.conflicts.len(),
        result.semantic_conflicts.len()
    ));
    if result.file_actions.is_empty() {
        output.push_str("  no file changes predicted\n");
    } else {
        for action in &result.file_actions {
            output.push_str(&format!("  {}: {}\n", action.action.as_str(), action.path));
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

fn render_merge_result_json(result: &MergeResult) -> Result<String, CliError> {
    let value = json!({
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
        "semantic_conflicts": result.semantic_conflicts.iter().map(semantic_conflict_json).collect::<Vec<_>>(),
        "next_actions": merge_next_actions(result),
    });
    Ok(serde_json::to_string_pretty(&value)?)
}

fn merge_file_action_json(action: &MergeFileAction) -> Value {
    json!({
        "path": &action.path,
        "action": action.action.as_str(),
        "bytes": action.content.as_ref().map(Vec::len),
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
        actions.push(json!({
            "kind": "resolve_file_conflict",
            "path": conflict,
            "hint": format!("merge writes conflict markers for {conflict}; resolve them before commit"),
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
    let _lock = RepoLock::acquire(&context.repo_root)?;
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

    Ok(json!({
        "type": "codefire_patch_import",
        "version": 1,
        "dry_run": options.dry_run,
        "applied": !options.dry_run,
        "branch": &context.branch,
        "base": patch_base,
        "entries": entries.len(),
        "paths": paths,
    }))
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
    let _lock = RepoLock::acquire(&repo_root)?;
    if branch_record_path(&repo_root, &options.new_branch).exists() {
        return Err(CliError::Usage(format!(
            "branch already exists: {}",
            options.new_branch
        )));
    }
    if options.source.starts_with("cf+http://") {
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
        let plan = clone_operation_plan(options, &repo_root, &source_head, "cf+http");
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
        return Ok(CloneResult { plan });
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
        return Ok(CloneResult { plan });
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
    Ok(CloneResult { plan })
}

fn open_branch(options: &OpenOptions) -> Result<OpenResult, CliError> {
    open_branch_from(&env::current_dir()?, options)
}

fn open_branch_from(start: &Path, options: &OpenOptions) -> Result<OpenResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let _lock = RepoLock::acquire(&repo_root)?;
    let cf = repo_root.join(".codefire");
    let objects = cf.join("objects");
    let mut branch = load_branch_record(&repo_root, &options.branch)?;
    let branch_head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &branch_head)?;

    let target = absolute_path(&options.path)?;
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

    Ok(OpenResult {
        branch: options.branch.clone(),
        open_dir: target,
        plan,
    })
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
            {"kind": "status", "command": "codefire-rs status --json", "target": {"path": target}},
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
            {"kind": "open", "command": format!("codefire-rs open {} <path>", options.new_branch), "target": {"branch": &options.new_branch}},
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
    let (scan, fires) = codefire_core::build_scan_result(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        reason,
        &now,
    )?;
    if persist {
        fs::create_dir_all(&active_state_path)?;
        write_json_atomic(&fires_path, &serde_json::to_value(&fires)?)?;
        write_json_atomic(
            &active_state_path.join("scan.json"),
            &serde_json::to_value(&scan)?,
        )?;
        let state = if scan.changed_atoms.is_empty() && scan.open_fires.is_empty() {
            "open-clean"
        } else {
            "open-burning"
        };
        set_open_state(&context, &active_state_path, state)?;
    }
    Ok(ScanExecution {
        context,
        active_state_path,
        scan,
        fires,
    })
}

fn run_verify(start: &Path) -> Result<codefire_core::Verification, CliError> {
    Ok(compute_verify(start, true)?.verification)
}

struct VerifyExecution {
    scan: codefire_core::ScanResult,
    verification: codefire_core::Verification,
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
    let verification = codefire_core::build_verification(
        &scan,
        missing_required_links,
        failed_checks,
        stale_resolutions,
        &policy,
        &now_iso_utc(),
    );
    if persist {
        write_json_atomic(
            &active_state_path.join("verification.json"),
            &serde_json::to_value(&verification)?,
        )?;
        let state = if verification.result == "passed" {
            "open-consistent"
        } else {
            "open-burning"
        };
        set_open_state(&context, &active_state_path, state)?;
    }
    Ok(VerifyExecution { scan, verification })
}

fn run_extinguish(options: &ExtinguishOptions) -> Result<ExtinguishResult, CliError> {
    let context = open_context(&options.path)?;
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
    if options.rationale.is_empty() && options.evidence.is_empty() {
        return Err(CliError::Usage(
            "extinguish requires --rationale or --evidence".to_string(),
        ));
    }

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
            resolved_at,
        },
    )?;
    let display_id = fires[fire_index].display_id.clone();
    let plan = extinguish_operation_plan(
        options,
        &context,
        &active_state_path,
        &fires[fire_index],
        &resolution_uid,
    );
    if options.dry_run {
        return Ok(ExtinguishResult { display_id, plan });
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
    write_json_atomic(&fires_path, &serde_json::to_value(fires)?)?;
    write_json_atomic(&resolutions_path, &serde_json::to_value(resolutions)?)?;
    Ok(ExtinguishResult { display_id, plan })
}

fn run_commit(options: &CommitOptions) -> Result<CommitResult, CliError> {
    let context = open_context(&options.path)?;
    let _lock = RepoLock::acquire(&context.repo_root)?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let verify_execution = compute_verify(&context.open_dir, !options.dry_run)?;
    let verification = verify_execution.verification;
    if verification.result != "passed" {
        return Err(CliError::Usage(
            "commit blocked: verification failed or consistency blockers remain".to_string(),
        ));
    }

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
    let policy = codefire_core::parse_verification_policy(&context.open_dir)?;
    let parents = commit_parents_for_open(&context, &active_state_path)?;
    let plan = commit_operation_plan(
        options,
        &context,
        &active_state_path,
        &scan,
        &verification,
        &parents,
    );
    if options.dry_run {
        return Ok(CommitResult {
            commit_id: String::new(),
            branch: context.branch,
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
            "result": "consistent",
            "open_required_fires": verification.open_required_fires,
            "failed_checks": verification.failed_checks.len(),
            "missing_required_links": verification.missing_required_links.len(),
            "stale_resolutions": verification.stale_resolutions.len(),
            "duplicate_atom_ids": verification.duplicate_atom_ids.len(),
        },
        "changed_atoms": scan.changed_atoms,
        "extinguished_fires": extinguished_fires,
        "created_at": now_iso_utc(),
    });
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

    Ok(CommitResult {
        commit_id,
        branch: context.branch,
        plan,
    })
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
            "status": &fire.status,
        },
        "resolution": {
            "resolution_uid": resolution_uid,
            "resolution_type": &options.resolution,
            "refresh": options.refresh,
            "has_rationale": !options.rationale.is_empty(),
            "has_evidence": !options.evidence.is_empty(),
        },
        "operations": [
            {"kind": "update_fire_status", "path": active_state_path.join("fires.json"), "status": "extinguished"},
            {"kind": "append_resolution", "path": active_state_path.join("resolutions.json"), "resolution_uid": resolution_uid},
        ],
        "next_actions": [
            {"kind": "verify", "command": "codefire-rs verify --details --json", "target": {"branch": &context.branch}},
        ],
    })
}

fn commit_operation_plan(
    options: &CommitOptions,
    context: &OpenContext,
    active_state_path: &Path,
    scan: &codefire_core::ScanResult,
    verification: &codefire_core::Verification,
    parents: &[String],
) -> Value {
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
        "changed_atoms": &scan.changed_atoms,
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
        "next_actions": [
            {"kind": "apply_commit", "command": "codefire-rs commit -m <message>", "target": {"branch": &context.branch}},
        ],
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
    ] {
        fs::create_dir_all(path)?;
    }
    for subdir in [
        "blobs",
        "content_manifests",
        "atom_indexes",
        "trace_graphs",
        "fire_ledgers",
        "resolution_ledgers",
        "verifications",
        "policies",
        "commits",
        "branches",
    ] {
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

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), CliError> {
    let parent = path
        .parent()
        .ok_or_else(|| CliError::Usage(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let temp_path = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
        file.write_all(b"\n")?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
}

fn read_status(start: &Path) -> Result<Status, CliError> {
    let context = open_context(start)?;

    let base = required_string(&context.registry, &["open", "current_base_commit"])?;
    let objects_root = context.repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects_root, &base)?;
    let state = required_string(&context.registry, &["state", "last_known"])?;
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
        state,
        base,
        open_fires,
    })
}

fn common_ancestor(objects: &Path, left: &str, right: &str) -> Result<Option<String>, CliError> {
    let left_distances = ancestor_distances(objects, left)?;
    let right_distances = ancestor_distances(objects, right)?;
    Ok(left_distances
        .keys()
        .filter_map(|commit| {
            let left_distance = left_distances.get(commit)?;
            let right_distance = right_distances.get(commit)?;
            Some((
                *left_distance + *right_distance,
                (*left_distance).max(*right_distance),
                commit.clone(),
            ))
        })
        .min()
        .map(|(_, _, commit)| commit))
}

fn ancestor_distances(
    objects: &Path,
    commit_id: &str,
) -> Result<BTreeMap<String, usize>, CliError> {
    let mut distances = BTreeMap::new();
    let mut queue = vec![(commit_id.to_string(), 0usize)];
    let mut cursor = 0usize;
    while cursor < queue.len() {
        let (current, distance) = queue[cursor].clone();
        cursor += 1;
        if distances
            .get(&current)
            .is_some_and(|known| *known <= distance)
        {
            continue;
        }
        distances.insert(current.clone(), distance);
        for parent in commit_parents(objects, &current)? {
            queue.push((parent, distance + 1));
        }
    }
    Ok(distances)
}

fn commit_parents(objects: &Path, commit_id: &str) -> Result<Vec<String>, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    Ok(commit
        .get("parents")
        .and_then(Value::as_array)
        .map(|parents| {
            parents
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default())
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

struct RepoLock {
    path: PathBuf,
}

impl RepoLock {
    fn acquire(repo_root: &Path) -> Result<Self, CliError> {
        let path = repo_root.join(".codefire").join("locks").join("repo.lock");
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(Self { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(
                CliError::LockContention("CodeFire repository is locked".to_string()),
            ),
            Err(error) => Err(CliError::Io(error)),
        }
    }
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
    fn acquire(path: PathBuf) -> Result<Self, CliError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(Self { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(
                CliError::LockContention("CodeFire resource is locked".to_string()),
            ),
            Err(error) => Err(CliError::Io(error)),
        }
    }
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
    let mut encoded = String::with_capacity(name.len());
    for byte in name.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn now_iso_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_seconds_utc(seconds)
}

fn format_unix_seconds_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = second_of_day / 3_600;
    let minute = second_of_day % 3_600 / 60;
    let second = second_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 }.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_part + 2).div_euclid(5) + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    let adjusted_year = year + if month <= 2 { 1 } else { 0 };
    (adjusted_year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests;
