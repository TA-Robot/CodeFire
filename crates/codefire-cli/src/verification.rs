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
    if verification.result == "passed" {
        println!("Verification passed.");
        return;
    }
    let blocking = blocking_check_names(verification);
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
        "Missing evidence refs: {}",
        verification.missing_evidence_refs.len()
    );
    println!(
        "Duplicate atom ids: {}",
        verification.duplicate_atom_ids.len()
    );
    println!("Failed checks: {}", verification.failed_checks.len());
    if details {
        if blocking_only {
            println!("Detail filter: blocking-only");
        }
        print_blocking_verification_details(verification);
    }
}

fn blocking_check_names(verification: &codefire_core::Verification) -> Vec<&'static str> {
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

fn print_blocking_verification_details(verification: &codefire_core::Verification) {
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
    if !verification.missing_evidence_refs.is_empty() {
        println!("Missing evidence ref details:");
        for item in verification.missing_evidence_refs.iter().take(5) {
            println!("  {}: {}", item.resolution_uid, item.evidence_id);
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
