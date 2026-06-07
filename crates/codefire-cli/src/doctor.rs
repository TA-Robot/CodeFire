mod checks;
mod report;

use super::{find_repo_root, CliError};
use std::path::PathBuf;

pub(crate) use report::{
    doctor_report_data_json, doctor_report_diagnostics_json, doctor_report_next_actions,
    print_doctor_report,
};

#[derive(Debug)]
pub(crate) struct DoctorOptions {
    pub(crate) start: PathBuf,
    pub(crate) json_output: bool,
    pub(crate) quick: bool,
}

#[derive(Debug)]
pub(crate) struct DoctorReport {
    pub(crate) repo_root: PathBuf,
    pub(crate) ok: bool,
    pub(crate) checked_objects: usize,
    pub(crate) checked_branches: usize,
    pub(crate) checked_opened: usize,
    pub(crate) checked_active_files: usize,
    pub(crate) quick: bool,
    pub(crate) skipped_checks: Vec<String>,
    pub(crate) issues: Vec<DoctorIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DoctorIssue {
    pub(crate) severity: DoctorSeverity,
    pub(crate) kind: String,
    pub(crate) message: String,
    pub(crate) path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DoctorSeverity {
    Error,
    Warning,
}

impl DoctorIssue {
    fn error(kind: &str, message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            severity: DoctorSeverity::Error,
            kind: kind.to_string(),
            message: message.into(),
            path,
        }
    }

    fn warning(kind: &str, message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            severity: DoctorSeverity::Warning,
            kind: kind.to_string(),
            message: message.into(),
            path,
        }
    }
}

pub(crate) fn parse_doctor_args(args: &[String]) -> Result<DoctorOptions, CliError> {
    let mut start = None;
    let mut json_output = false;
    let mut quick = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json_output = true,
            "--quick" => quick = true,
            "--full" => quick = false,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported doctor option: {option}"
                )));
            }
            value if start.is_none() => start = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected doctor argument: {value}"
                )));
            }
        }
    }
    Ok(DoctorOptions {
        start: start.unwrap_or(std::env::current_dir()?),
        json_output,
        quick,
    })
}

pub(crate) fn run_doctor(options: &DoctorOptions) -> Result<DoctorReport, CliError> {
    let repo_root = find_repo_root(&options.start)?;
    let cf = repo_root.join(".codefire");
    let checked = checks::run_checks(&cf, options.quick)?;
    let ok = !checked.issues.iter().any(report::issue_blocking);

    Ok(DoctorReport {
        repo_root,
        ok,
        checked_objects: checked.objects,
        checked_branches: checked.branches,
        checked_opened: checked.opened,
        checked_active_files: checked.active_files,
        quick: options.quick,
        skipped_checks: checked.skipped_checks,
        issues: checked.issues,
    })
}
