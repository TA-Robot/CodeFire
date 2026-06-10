use super::*;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn parse_import_args(args: &[String]) -> Result<ImportOptions, CliError> {
    let mut path = None;
    let mut branch = "main".to_string();
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
            "--branch" => {
                branch = take_option_value(args, &mut index, "--branch")?;
            }
            value if value.starts_with("--branch=") => {
                branch = value.trim_start_matches("--branch=").to_string();
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported import option: {value}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected import argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(ImportOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        branch,
        dry_run,
        json_output,
        lock,
        idempotency_key,
    })
}

pub(crate) fn import_existing_project(options: &ImportOptions) -> Result<ImportResult, CliError> {
    let target = absolute_path(&options.path)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_import_idempotency_marker(&target, key)? {
                return Ok(result);
            }
        }
    }
    validate_import_target(&target)?;
    let initial_files = rel_files(&target)?.len();
    let opened_at = now_iso_utc();
    let repo_root = target.clone();
    let cf = repo_root.join(".codefire");
    let active_path = cf.join("active").join(import_open_instance_id(
        &options.branch,
        &target,
        &opened_at,
    ));
    let registry_path = opened_registry_path(&repo_root, &options.branch);
    let plan = import_operation_plan(
        options,
        &repo_root,
        &target,
        &registry_path,
        &active_path,
        initial_files,
    );
    if options.dry_run {
        return Ok(ImportResult {
            repo_root,
            open_dir: target,
            branch: options.branch.clone(),
            base_commit: String::new(),
            initial_files,
            initial_state: "planned".to_string(),
            plan,
        });
    }

    let init = init_repo(&repo_root, false)?;
    let _lock = RepoLock::acquire_with_options(&repo_root, &options.lock)?;
    if options.branch != "main" {
        rename_initial_branch(&repo_root, &init.main_commit, &options.branch)?;
    }
    let repo_json = read_json(&cf.join("repo.json"))?;
    let repository_id = required_string(&repo_json, &["repository_id"])?;
    let open_instance_id = active_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            CliError::InvalidRepository("invalid import open instance path".to_string())
        })?;
    write_json_atomic(
        &target.join(".codefire-open"),
        &json!({
            "version": 1,
            "repository": {"path": cf, "repository_id": repository_id},
            "branch": {"name": options.branch, "opened_from_commit": init.main_commit},
            "open": {"open_instance_id": open_instance_id, "opened_path": target, "opened_at": opened_at},
            "import": {"mode": "adopt_existing_project", "initial_files": initial_files},
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
                "opened_from_commit": init.main_commit,
                "current_base_commit": init.main_commit,
                "active_state_path": active_path,
            },
            "state": {"last_known": "open-burning"},
            "import": {"mode": "adopt_existing_project", "initial_files": initial_files},
        }),
    )?;
    fs::create_dir_all(&active_path)?;
    write_json_atomic(
        &active_path.join("state.json"),
        &json!({
            "state": "open-burning",
            "import": {
                "mode": "adopt_existing_project",
                "initial_files": initial_files,
                "base_commit": init.main_commit,
            },
        }),
    )?;
    let mut branch = load_branch_record(&repo_root, &options.branch)?;
    branch["state"] = Value::String("open-burning".to_string());
    save_branch_record(&repo_root, &branch)?;

    let scan = compute_scan(&target, true)?.scan;
    let initial_state = scan_branch_state(scan_change_count(&scan), scan.open_fires.len());
    let result = ImportResult {
        repo_root,
        open_dir: target,
        branch: options.branch.clone(),
        base_commit: init.main_commit,
        initial_files,
        initial_state: initial_state.to_string(),
        plan,
    };
    if let Some(key) = options.idempotency_key.as_deref() {
        save_import_idempotency_marker(&result.repo_root, key, &result)?;
    }
    Ok(result)
}

pub(crate) fn import_result_data_json(result: &ImportResult) -> Value {
    json!({
        "type": "codefire_import_result",
        "version": 1,
        "repo_root": &result.repo_root,
        "open_dir": &result.open_dir,
        "branch": &result.branch,
        "base_commit": empty_string_as_null(&result.base_commit),
        "initial_files": result.initial_files,
        "initial_state": &result.initial_state,
        "plan": &result.plan,
    })
}

fn validate_import_target(target: &Path) -> Result<(), CliError> {
    if !target.exists() {
        return Err(CliError::Usage(format!(
            "import target does not exist: {}",
            target.display()
        )));
    }
    if !target.is_dir() {
        return Err(CliError::Usage(format!(
            "import target must be a directory: {}",
            target.display()
        )));
    }
    if target.join(".codefire").exists() {
        return Err(CliError::Usage(format!(
            "import target is already a CodeFire repository: {}",
            target.display()
        )));
    }
    if target.join(".codefire-open").exists() {
        return Err(CliError::Usage(format!(
            "import target is already a CodeFire open directory: {}",
            target.display()
        )));
    }
    Ok(())
}

fn import_open_instance_id(branch: &str, target: &Path, opened_at: &str) -> String {
    format!(
        "import_{:012x}",
        stable_hash_48(format!("{branch}:{}:{opened_at}", target.display()).as_bytes())
    )
}

fn rename_initial_branch(repo_root: &Path, head: &str, branch_name: &str) -> Result<(), CliError> {
    let now = now_iso_utc();
    let branch = json!({
        "type": "branch",
        "version": 1,
        "name": branch_name,
        "head": head,
        "state": "closed",
        "created_from": "import",
        "created_at": now,
    });
    save_branch_record(repo_root, &branch)?;
    let main_path = branch_record_path(repo_root, "main");
    if main_path.exists() {
        fs::remove_file(main_path)?;
    }
    Ok(())
}

fn import_operation_plan(
    options: &ImportOptions,
    repo_root: &Path,
    target: &Path,
    registry_path: &Path,
    active_path: &Path,
    initial_files: usize,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "import",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "repo": repo_root,
        "target": target,
        "branch": &options.branch,
        "initial_files": initial_files,
        "operations": [
            {"kind": "initialize_repository", "path": repo_root},
            {"kind": "write_open_marker", "path": target.join(".codefire-open")},
            {"kind": "write_open_registry", "path": registry_path},
            {"kind": "create_active_state", "path": active_path},
            {"kind": "scan_imported_tree", "path": target},
            {"kind": "update_branch_state", "branch": &options.branch, "state": "open-burning_or_scan_result"},
        ],
        "next_actions": [
            {"kind": "scan", "command": "codefire scan --json", "target": {"path": target}},
            {"kind": "verify", "command": "codefire verify --json", "target": {"path": target}},
            {"kind": "commit", "command": "codefire commit -m \"Initial import\"", "target": {"path": target}},
        ],
    })
}

fn empty_string_as_null(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

fn save_import_idempotency_marker(
    repo_root: &Path,
    key: &str,
    result: &ImportResult,
) -> Result<(), CliError> {
    if key.is_empty() {
        return Ok(());
    }
    let path = repo_root
        .join(".codefire")
        .join("idempotency")
        .join("import")
        .join(format!("{}.json", ref_file_name(key)));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_atomic(
        &path,
        &json!({
            "version": 1,
            "operation": "import",
            "key": key,
            "repo_root": &result.repo_root,
            "open_dir": &result.open_dir,
            "branch": &result.branch,
            "base_commit": &result.base_commit,
            "initial_files": result.initial_files,
            "initial_state": &result.initial_state,
            "created_at": now_iso_utc(),
        }),
    )
}

fn load_import_idempotency_marker(
    target: &Path,
    key: &str,
) -> Result<Option<ImportResult>, CliError> {
    if key.is_empty() {
        return Ok(None);
    }
    let path = target
        .join(".codefire")
        .join("idempotency")
        .join("import")
        .join(format!("{}.json", ref_file_name(key)));
    let Some(record) = read_optional_json(&path)? else {
        return Ok(None);
    };
    let repo_root = PathBuf::from(required_string(&record, &["repo_root"])?);
    let open_dir = PathBuf::from(required_string(&record, &["open_dir"])?);
    let branch = required_string(&record, &["branch"])?;
    let base_commit = required_string(&record, &["base_commit"])?;
    let initial_files = record
        .get("initial_files")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let initial_state = record
        .get("initial_state")
        .and_then(Value::as_str)
        .unwrap_or("open-burning")
        .to_string();
    Ok(Some(ImportResult {
        repo_root: repo_root.clone(),
        open_dir: open_dir.clone(),
        branch: branch.clone(),
        base_commit,
        initial_files,
        initial_state,
        plan: json!({
            "type": "codefire_operation_plan",
            "version": 1,
            "command": "import",
            "replayed": true,
            "repo": repo_root,
            "target": open_dir,
            "branch": branch,
        }),
    }))
}
