use super::DoctorIssue;
use crate::{read_json, CliError};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_REPO_DIRS: &[&str] = &["objects", "branches", "opened", "active"];
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
    pub(super) issues: Vec<DoctorIssue>,
}

pub(super) fn run_checks(cf: &Path) -> Result<DoctorChecks, CliError> {
    let mut issues = Vec::new();
    check_repo_layout(cf, &mut issues);
    Ok(DoctorChecks {
        objects: check_object_records(&cf.join("objects"), &mut issues)?,
        branches: check_branch_heads(cf, &mut issues)?,
        opened: check_opened_registry(cf, &mut issues)?,
        active_files: check_active_state_files(cf, &mut issues)?,
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
    for relative in REQUIRED_REPO_DIRS {
        let path = cf.join(relative);
        if !path.is_dir() {
            issues.push(DoctorIssue::error(
                "missing_repo_directory",
                format!("missing .codefire/{relative} directory"),
                Some(path),
            ));
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

fn check_active_state_files(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let active = cf.join("active");
    if !active.exists() {
        return Ok(0);
    }
    let mut checked = 0usize;
    for state_dir in collect_dirs(&active)? {
        for file_name in ACTIVE_STATE_FILES {
            let path = state_dir.join(file_name);
            if !path.exists() {
                continue;
            }
            match read_json(&path) {
                Ok(_) => checked += 1,
                Err(error) => issues.push(DoctorIssue::error(
                    "invalid_active_state_json",
                    error.to_string(),
                    Some(path),
                )),
            }
        }
    }
    Ok(checked)
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
