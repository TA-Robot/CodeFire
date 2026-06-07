use super::CliError;
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct VerifyOptions {
    pub(crate) path: PathBuf,
    pub(crate) details: bool,
    pub(crate) blocking_only: bool,
    pub(crate) json_output: bool,
    pub(crate) metrics: bool,
}

impl VerifyOptions {
    pub(crate) fn diagnostic_filter(&self) -> &'static str {
        if self.blocking_only {
            "blocking_only"
        } else {
            "all"
        }
    }
}

pub(crate) fn parse_verify_args(args: &[String]) -> Result<VerifyOptions, CliError> {
    let mut path = None;
    let mut details = false;
    let mut blocking_only = false;
    let mut json_output = false;
    let mut metrics = false;
    for arg in args {
        match arg.as_str() {
            "--details" => details = true,
            "--blocking-only" => blocking_only = true,
            "--json" => json_output = true,
            "--metrics" => metrics = true,
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported verify option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected verify argument: {value}"
                )));
            }
        }
    }
    Ok(VerifyOptions {
        path: path.unwrap_or(env::current_dir()?),
        details,
        blocking_only,
        json_output,
        metrics,
    })
}

pub(crate) fn print_verification(
    verification: &codefire_core::Verification,
    details: bool,
    blocking_only: bool,
) {
    print!(
        "{}",
        render_verification(verification, details, blocking_only)
    );
}

pub(crate) fn render_verification(
    verification: &codefire_core::Verification,
    details: bool,
    blocking_only: bool,
) -> String {
    let mut output = String::new();
    if verification.result == "passed" {
        output.push_str("Verification passed.\n");
    } else {
        output.push_str("Verification failed.\n");
    }
    let blocking = blocking_check_names(verification);
    output.push_str(&format!(
        "Blocking checks: {}",
        if blocking.is_empty() {
            "none".to_string()
        } else {
            blocking.join(", ")
        }
    ));
    output.push('\n');
    output.push_str(&format!(
        "Open fires: {}\n",
        verification.open_required_fires
    ));
    output.push_str(&format!(
        "Missing required links: {}",
        verification.missing_required_links.len()
    ));
    output.push('\n');
    output.push_str(&format!(
        "Stale resolutions: {}",
        verification.stale_resolutions.len()
    ));
    output.push('\n');
    output.push_str(&format!(
        "Missing evidence refs: {}",
        verification.missing_evidence_refs.len()
    ));
    output.push('\n');
    output.push_str(&format!(
        "Duplicate atom ids: {}",
        verification.duplicate_atom_ids.len()
    ));
    output.push('\n');
    output.push_str(&format!(
        "Failed checks: {}\n",
        verification.failed_checks.len()
    ));
    if details {
        if blocking_only {
            output.push_str("Detail filter: blocking-only\n");
        }
        render_blocking_verification_details(verification, blocking_only, &mut output);
    }
    output
}

fn blocking_check_names(verification: &codefire_core::Verification) -> Vec<&'static str> {
    let mut blocking = Vec::new();
    if verification.open_required_fires > 0 {
        blocking.push("open fires");
    }
    if verification.trace_completeness_required && !verification.missing_required_links.is_empty() {
        blocking.push("missing required links");
    }
    if !verification.stale_resolutions.is_empty() {
        blocking.push("stale resolutions");
    }
    if !verification.missing_evidence_refs.is_empty() {
        blocking.push("missing evidence refs");
    }
    if !verification.duplicate_atom_ids.is_empty() {
        blocking.push("duplicate atom ids");
    }
    if !verification.failed_checks.is_empty() {
        blocking.push("failed checks");
    }
    blocking
}

fn render_blocking_verification_details(
    verification: &codefire_core::Verification,
    blocking_only: bool,
    output: &mut String,
) {
    if (!blocking_only || verification.trace_completeness_required)
        && !verification.missing_required_links.is_empty()
    {
        output.push_str("Missing required link details:\n");
        for item in verification.missing_required_links.iter().take(5) {
            output.push_str(&format!(
                "  {} requires {} -> {} min {}",
                item.atom_id, item.required_type, item.target_kind, item.min
            ));
            output.push('\n');
        }
    }
    if !verification.stale_resolutions.is_empty() {
        output.push_str("Stale resolution details:\n");
        for item in verification.stale_resolutions.iter().take(5) {
            output.push_str(&format!("  {}: {}\n", item.resolution_uid, item.reason));
        }
    }
    if !verification.missing_evidence_refs.is_empty() {
        output.push_str("Missing evidence ref details:\n");
        for item in verification.missing_evidence_refs.iter().take(5) {
            output.push_str(&format!(
                "  {}: {}\n",
                item.resolution_uid, item.evidence_id
            ));
        }
    }
    if !verification.duplicate_atom_ids.is_empty() {
        output.push_str("Duplicate Atom ID details:\n");
        for atom_id in verification.duplicate_atom_ids.iter().take(5) {
            output.push_str(&format!("  {atom_id}\n"));
        }
    }
    if !verification.failed_checks.is_empty() {
        output.push_str("Failed check details:\n");
        for item in verification.failed_checks.iter().take(5) {
            let summary = item.output.trim().lines().next().unwrap_or_default();
            output.push_str(&format!("  {}: {} {}\n", item.id, item.command, summary));
        }
    }
}
