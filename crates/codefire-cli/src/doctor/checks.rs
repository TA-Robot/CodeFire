use super::DoctorIssue;
use crate::repo_layout::{object_subdir_requirement, LayoutRequirement, REPO_DIRS};
use crate::{read_json, CliError};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const ACTIVE_STATE_FILES: &[&str] = &[
    "state.json",
    "fires.json",
    "resolutions.json",
    "scan.json",
    "verification.json",
];

#[derive(Debug)]
pub(super) struct DoctorChecks {
    pub(super) objects: usize,
    pub(super) branches: usize,
    pub(super) opened: usize,
    pub(super) active_files: usize,
    pub(super) skipped_checks: Vec<String>,
    pub(super) issues: Vec<DoctorIssue>,
}

pub(super) fn run_checks(cf: &Path, quick: bool) -> Result<DoctorChecks, CliError> {
    let mut issues = Vec::new();
    check_repo_layout(cf, &mut issues);
    let mut skipped_checks = Vec::new();
    let objects = if quick {
        skipped_checks.push("object_store_integrity".to_string());
        0
    } else {
        check_object_records(&cf.join("objects"), &mut issues)?
    };
    Ok(DoctorChecks {
        objects,
        branches: check_branch_heads(cf, &mut issues)?,
        opened: check_opened_registry(cf, &mut issues)?,
        active_files: check_active_state_files(cf, &mut issues)?,
        skipped_checks,
        issues,
    })
}

fn check_repo_layout(cf: &Path, issues: &mut Vec<DoctorIssue>) {
    if !cf.is_dir() {
        issues.push(DoctorIssue::error(
            "missing_codefire_dir",
            "missing .codefire directory",
            Some(cf.to_path_buf()),
        ));
        return;
    }
    for entry in REPO_DIRS {
        let path = cf.join(entry.relative);
        if !path.is_dir() {
            let message = format!("missing .codefire/{} directory", entry.relative);
            match entry.requirement {
                LayoutRequirement::Required => issues.push(DoctorIssue::error(
                    "missing_repo_directory",
                    message,
                    Some(path),
                )),
                LayoutRequirement::AutoCreate => issues.push(DoctorIssue::warning(
                    "missing_repairable_directory",
                    message,
                    Some(path),
                )),
            }
        }
    }
    let objects = cf.join("objects");
    for subdir in codefire_store::known_object_subdirs() {
        let path = objects.join(subdir);
        if !path.is_dir() {
            let message = format!("missing .codefire/objects/{subdir} directory");
            match object_subdir_requirement(subdir) {
                LayoutRequirement::Required => issues.push(DoctorIssue::error(
                    "missing_object_subdirectory",
                    message,
                    Some(path),
                )),
                LayoutRequirement::AutoCreate => issues.push(DoctorIssue::warning(
                    "missing_repairable_object_subdirectory",
                    message,
                    Some(path),
                )),
            }
        }
    }
}

fn check_object_records(objects: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    if !objects.exists() {
        return Ok(0);
    }
    let mut checked = 0usize;
    for path in collect_json_files(objects)? {
        let record = match fs::read_to_string(&path)
            .map_err(CliError::from)
            .and_then(|text| {
                serde_json::from_str::<codefire_store::ObjectRecord>(&text).map_err(CliError::from)
            }) {
            Ok(record) => record,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_object_record",
                    error.to_string(),
                    Some(path),
                ));
                continue;
            }
        };
        match codefire_store::validate_object_record(&record, None, Some(&path)) {
            Ok(()) => checked += 1,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "object_integrity_error",
                    error.to_string(),
                    Some(path.clone()),
                ));
                continue;
            }
        }
        if let Some(expected_subdir) = codefire_store::object_subdir(&record.type_tag) {
            if path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                != Some(expected_subdir)
            {
                issues.push(DoctorIssue::error(
                    "object_type_directory_mismatch",
                    format!(
                        "{} object should be stored under objects/{expected_subdir}",
                        record.type_tag
                    ),
                    Some(path),
                ));
            }
        } else {
            issues.push(DoctorIssue::error(
                "unknown_object_type",
                format!("unknown object type: {}", record.type_tag),
                Some(path),
            ));
        }
    }
    if checked == 0 {
        issues.push(DoctorIssue::warning(
            "empty_object_store",
            "object store has no valid object records",
            Some(objects.to_path_buf()),
        ));
    }
    Ok(checked)
}

fn check_branch_heads(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let branches = cf.join("branches");
    if !branches.exists() {
        return Ok(0);
    }
    let objects = cf.join("objects");
    let mut checked = 0usize;
    for path in collect_json_files(&branches)? {
        let branch = match read_json(&path) {
            Ok(branch) => branch,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_branch_record",
                    error.to_string(),
                    Some(path),
                ));
                continue;
            }
        };
        let head = match branch.get("head").and_then(Value::as_str) {
            Some(head) => head,
            None => {
                issues.push(DoctorIssue::error(
                    "missing_branch_head",
                    "branch record must contain head",
                    Some(path),
                ));
                continue;
            }
        };
        match codefire_store::validate_sealed_commit(&objects, head) {
            Ok(()) => checked += 1,
            Err(error) => issues.push(DoctorIssue::error(
                "invalid_branch_head",
                error.to_string(),
                Some(path),
            )),
        }
    }
    Ok(checked)
}

fn check_opened_registry(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let opened = cf.join("opened");
    if !opened.exists() {
        return Ok(0);
    }
    let objects = cf.join("objects");
    let mut checked = 0usize;
    for path in collect_json_files(&opened)? {
        let registry = match read_json(&path) {
            Ok(registry) => registry,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_open_registry",
                    error.to_string(),
                    Some(path),
                ));
                continue;
            }
        };
        let branch_name = match registry
            .get("branch")
            .and_then(|branch| branch.get("name"))
            .and_then(Value::as_str)
        {
            Some(branch) => Some(branch.to_string()),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_branch_name",
                    "opened registry must contain branch.name",
                    Some(path.clone()),
                ));
                None
            }
        };
        let open_instance_id = match registry
            .get("open")
            .and_then(|open| open.get("open_instance_id"))
            .and_then(Value::as_str)
        {
            Some(open_instance_id) => Some(open_instance_id.to_string()),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_instance_id",
                    "opened registry must contain open.open_instance_id",
                    Some(path.clone()),
                ));
                None
            }
        };
        let open_path = match registry
            .get("open")
            .and_then(|open| open.get("path"))
            .and_then(Value::as_str)
        {
            Some(open_path) => Some(PathBuf::from(open_path)),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_path",
                    "opened registry must contain open.path",
                    Some(path.clone()),
                ));
                None
            }
        };
        match registry
            .get("open")
            .and_then(|open| open.get("current_base_commit"))
            .and_then(Value::as_str)
        {
            Some(base) => {
                if let Err(error) = codefire_store::validate_sealed_commit(&objects, base) {
                    issues.push(DoctorIssue::error(
                        "invalid_open_base_commit",
                        error.to_string(),
                        Some(path.clone()),
                    ));
                }
            }
            None => issues.push(DoctorIssue::error(
                "missing_open_base_commit",
                "opened registry must contain open.current_base_commit",
                Some(path.clone()),
            )),
        }
        if let Some(open_path) = open_path.as_deref() {
            if !open_path.is_dir() {
                issues.push(DoctorIssue::error(
                    "missing_open_path_dir",
                    format!(
                        "opened registry path does not exist: {}",
                        open_path.display()
                    ),
                    Some(path.clone()),
                ));
            } else {
                check_open_marker(
                    cf,
                    &path,
                    &registry,
                    open_path,
                    branch_name.as_deref(),
                    open_instance_id.as_deref(),
                    issues,
                );
            }
        }
        match registry
            .get("open")
            .and_then(|open| open.get("active_state_path"))
            .and_then(Value::as_str)
        {
            Some(active_path) if Path::new(active_path).is_dir() => checked += 1,
            Some(active_path) => issues.push(DoctorIssue::error(
                "missing_active_state_dir",
                format!("active state path does not exist: {active_path}"),
                Some(path),
            )),
            None => issues.push(DoctorIssue::error(
                "missing_active_state_path",
                "opened registry must contain open.active_state_path",
                Some(path),
            )),
        }
    }
    Ok(checked)
}

fn check_open_marker(
    cf: &Path,
    registry_path: &Path,
    registry: &Value,
    open_path: &Path,
    branch_name: Option<&str>,
    open_instance_id: Option<&str>,
    issues: &mut Vec<DoctorIssue>,
) {
    let marker_path = open_path.join(".codefire-open");
    if !marker_path.exists() {
        issues.push(DoctorIssue::error(
            "missing_open_marker",
            "open path is missing .codefire-open marker",
            Some(marker_path),
        ));
        return;
    }
    let marker = match read_json(&marker_path) {
        Ok(marker) => marker,
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_open_marker",
                error.to_string(),
                Some(marker_path),
            ));
            return;
        }
    };
    if path_string(&marker, &["repository", "path"]).as_deref() != Some(cf) {
        issues.push(DoctorIssue::error(
            "open_marker_repository_mismatch",
            "open marker repository.path does not match registry repository",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["branch", "name"]).as_deref() != branch_name {
        issues.push(DoctorIssue::error(
            "open_marker_branch_mismatch",
            "open marker branch.name does not match opened registry branch.name",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["open", "open_instance_id"]).as_deref() != open_instance_id {
        issues.push(DoctorIssue::error(
            "open_marker_instance_mismatch",
            "open marker open_instance_id does not match opened registry",
            Some(marker_path.clone()),
        ));
    }
    if path_string(&marker, &["open", "opened_path"]).as_deref() != Some(open_path) {
        issues.push(DoctorIssue::error(
            "open_marker_path_mismatch",
            "open marker opened_path does not match opened registry open.path",
            Some(marker_path),
        ));
    }
    if value_string(registry, &["state", "last_known"]).is_none() {
        issues.push(DoctorIssue::error(
            "missing_open_registry_state",
            "opened registry must contain state.last_known",
            Some(registry_path.to_path_buf()),
        ));
    }
}

fn check_active_state_files(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let active = cf.join("active");
    if !active.exists() {
        return Ok(0);
    }
    let mut checked = 0usize;
    for state_dir in collect_dirs(&active)? {
        if !state_dir.join("state.json").exists() {
            issues.push(DoctorIssue::error(
                "missing_active_state_file",
                "active state directory is missing required state.json",
                Some(state_dir.join("state.json")),
            ));
        }
        let mut state_value = None::<String>;
        let mut open_fire_count = None::<usize>;
        for &file_name in ACTIVE_STATE_FILES {
            let path = state_dir.join(file_name);
            if !path.exists() {
                continue;
            }
            match read_json(&path) {
                Ok(value) => match validate_active_state_shape(file_name, value.clone()) {
                    Ok(()) => {
                        if file_name == "state.json" {
                            state_value = value
                                .get("state")
                                .and_then(Value::as_str)
                                .map(str::to_string);
                        } else if file_name == "fires.json" {
                            open_fire_count = value.as_array().map(Vec::len);
                        }
                        checked += 1;
                    }
                    Err(message) => issues.push(DoctorIssue::error(
                        "invalid_active_state_shape",
                        message,
                        Some(path),
                    )),
                },
                Err(error) => issues.push(DoctorIssue::error(
                    "invalid_active_state_json",
                    error.to_string(),
                    Some(path),
                )),
            }
        }
        if matches!(
            state_value.as_deref(),
            Some("open-clean" | "open-consistent")
        ) && open_fire_count.unwrap_or(0) > 0
        {
            issues.push(DoctorIssue::error(
                "active_state_invariant_violation",
                "state is clean/consistent but fires.json contains open fires",
                Some(state_dir.to_path_buf()),
            ));
        }
    }
    Ok(checked)
}

fn validate_active_state_shape(file_name: &str, value: Value) -> Result<(), String> {
    match file_name {
        "state.json" => {
            let state = value
                .get("state")
                .and_then(Value::as_str)
                .ok_or_else(|| "state.json must contain string field state".to_string())?;
            if !matches!(state, "open-clean" | "open-burning" | "open-consistent") {
                return Err(format!("state.json contains unsupported state: {state}"));
            }
            if value
                .get("pending_merge_parent")
                .is_some_and(|parent| !parent.is_string())
            {
                return Err("state.json pending_merge_parent must be a string".to_string());
            }
            Ok(())
        }
        "fires.json" => serde_json::from_value::<Vec<codefire_core::Fire>>(value)
            .map(|_| ())
            .map_err(|error| format!("fires.json shape is invalid: {error}")),
        "resolutions.json" => serde_json::from_value::<Vec<codefire_core::Resolution>>(value)
            .map(|_| ())
            .map_err(|error| format!("resolutions.json shape is invalid: {error}")),
        "scan.json" => serde_json::from_value::<codefire_core::ScanResult>(value)
            .map(|_| ())
            .map_err(|error| format!("scan.json shape is invalid: {error}")),
        "verification.json" => serde_json::from_value::<codefire_core::Verification>(value)
            .map(|_| ())
            .map_err(|error| format!("verification.json shape is invalid: {error}")),
        _ => Ok(()),
    }
}

fn value_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(ToString::to_string)
}

fn path_string(value: &Value, path: &[&str]) -> Option<PathBuf> {
    value_string(value, path).map(PathBuf::from)
}

fn collect_json_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn collect_dirs(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                stack.push(path.clone());
                dirs.push(path);
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}
