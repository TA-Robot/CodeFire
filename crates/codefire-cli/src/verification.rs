use super::{parse_lock_option, CliError, LockOptions};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct VerifyOptions {
    pub(crate) path: PathBuf,
    pub(crate) details: bool,
    pub(crate) blocking_only: bool,
    pub(crate) json_output: bool,
    pub(crate) metrics: bool,
    pub(crate) lock: LockOptions,
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
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--details" => details = true,
            "--blocking-only" => blocking_only = true,
            "--json" => json_output = true,
            "--metrics" => metrics = true,
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
        index += 1;
    }
    Ok(VerifyOptions {
        path: path.unwrap_or(env::current_dir()?),
        details,
        blocking_only,
        json_output,
        metrics,
        lock,
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
    let decision = codefire_core::committable_decision(verification);
    if decision.is_committable() {
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
        "Unresolved merge conflicts: {}\n",
        verification.unresolved_merge_conflicts.len()
    ));
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
    if let Some(policy) = &verification.effective_policy {
        output.push_str(&format!(
            "Effective policy: {} ({} rule(s))\n",
            policy.digest,
            policy.rules.len()
        ));
    }
    if let Some(coverage) = &verification.change_coverage {
        output.push_str(&format!(
            "Change coverage: {} changed file(s), {} non-Atom, {} deleted-Atom, {} out-of-Atom, {} Atom file(s) with unclassified spans [{}]\n",
            coverage.summary.changed_files,
            coverage.summary.non_atom_files,
            coverage.summary.deleted_atom_files,
            coverage.summary.out_of_atom_files,
            coverage.summary.atom_files_with_unclassified_spans,
            coverage.span_precision,
        ));
    }
    if let Some(subject) = &verification.subject {
        output.push_str(&format!(
            "Assessment subject: {} ({})\n",
            subject.subject_digest, subject.trust_kind
        ));
    }
    if !verification.execution.is_empty() {
        output.push_str(&format!(
            "Execution profile: local_verification_v1 ({} command(s))\n",
            verification.execution.len()
        ));
    }
    if details {
        if blocking_only {
            output.push_str("Detail filter: blocking-only\n");
        }
        render_blocking_verification_details(verification, blocking_only, &mut output);
    }
    output
}

fn blocking_check_names(verification: &codefire_core::Verification) -> Vec<&'static str> {
    codefire_core::committable_decision(verification)
        .blocking_reasons
        .into_iter()
        .map(blocking_reason_label)
        .collect()
}

fn blocking_reason_label(reason: codefire_core::CommittableReason) -> &'static str {
    match reason {
        codefire_core::CommittableReason::UnresolvedMergeConflicts => "unresolved merge conflicts",
        codefire_core::CommittableReason::OpenRequiredFires => "open fires",
        codefire_core::CommittableReason::MissingRequiredLinks => "missing required links",
        codefire_core::CommittableReason::StaleResolutions => "stale resolutions",
        codefire_core::CommittableReason::MissingEvidenceRefs => "missing evidence refs",
        codefire_core::CommittableReason::DuplicateAtomIds => "duplicate atom ids",
        codefire_core::CommittableReason::FailedChecks => "failed checks",
    }
}

fn render_blocking_verification_details(
    verification: &codefire_core::Verification,
    blocking_only: bool,
    output: &mut String,
) {
    let decision = codefire_core::committable_decision(verification);
    if !verification.unresolved_merge_conflicts.is_empty() {
        output.push_str("Unresolved merge conflict details:\n");
        for item in verification.unresolved_merge_conflicts.iter().take(5) {
            output.push_str(&format!("  {}: {}\n", item.kind.as_str(), item.subject));
        }
        push_omitted_note(verification.unresolved_merge_conflicts.len(), output);
    }
    if (!blocking_only
        || decision.is_blocking(codefire_core::CommittableReason::MissingRequiredLinks))
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
        push_omitted_note(verification.missing_required_links.len(), output);
    }
    if !verification.stale_resolutions.is_empty() {
        output.push_str("Stale resolution details:\n");
        for item in verification
            .stale_resolutions
            .iter()
            .take(STALE_RESOLUTION_DETAIL_LIMIT)
        {
            output.push_str(&stale_resolution_line(item));
        }
        let hidden = verification
            .stale_resolutions
            .len()
            .saturating_sub(STALE_RESOLUTION_DETAIL_LIMIT);
        if hidden > 0 {
            output.push_str(&format!("  ... and {hidden} more (use --json for all)\n"));
        }
        output.push_str(
            "  refresh each with: codefire extinguish <FIRE-id> --refresh --resolution <type> --rationale <text>\n",
        );
    }
    if !verification.missing_evidence_refs.is_empty() {
        output.push_str("Missing evidence ref details:\n");
        for item in verification.missing_evidence_refs.iter().take(5) {
            output.push_str(&format!(
                "  {}: {}\n",
                item.resolution_uid, item.evidence_id
            ));
        }
        push_omitted_note(verification.missing_evidence_refs.len(), output);
    }
    if !verification.duplicate_atom_ids.is_empty() {
        output.push_str("Duplicate Atom ID details:\n");
        for atom_id in verification.duplicate_atom_ids.iter().take(5) {
            output.push_str(&format!("  {atom_id}\n"));
        }
        push_omitted_note(verification.duplicate_atom_ids.len(), output);
    }
    if !verification.failed_checks.is_empty() {
        output.push_str("Failed check details:\n");
        for item in verification.failed_checks.iter().take(5) {
            output.push_str(&failed_check_block(verification, item));
        }
        push_omitted_note(verification.failed_checks.len(), output);
    }
}

/// Lists above show at most five entries; say how many were left out.
fn push_omitted_note(total: usize, output: &mut String) {
    let hidden = total.saturating_sub(5);
    if hidden > 0 {
        output.push_str(&format!("  ... and {hidden} more (use --json for all)\n"));
    }
}

/// A failing test run reports its cause at the end of its output.
const FAILED_CHECK_TAIL_LINES: usize = 20;

fn failed_check_block(
    verification: &codefire_core::Verification,
    item: &codefire_core::FailedCheck,
) -> String {
    let mut block = format!("  {}: {}\n", item.id, item.command);
    if let Some(record) = verification
        .execution
        .iter()
        .find(|record| record.id == item.id)
    {
        let status = match record.exit_code {
            _ if record.timed_out => format!("timed out after {}ms", record.timeout_ms),
            Some(code) => format!("exit code {code}"),
            None => "terminated without an exit code".to_string(),
        };
        block.push_str(&format!("    {status}\n"));
    }
    let lines = item.output.trim().lines().collect::<Vec<_>>();
    let hidden = lines.len().saturating_sub(FAILED_CHECK_TAIL_LINES);
    if hidden > 0 {
        block.push_str(&format!(
            "    ... {hidden} earlier line(s) omitted (use --json for the stored output)\n"
        ));
    }
    for line in &lines[hidden..] {
        block.push_str(&format!("    | {line}\n"));
    }
    block
}

/// Every listed entry names the fire to refresh, so the cap is generous.
const STALE_RESOLUTION_DETAIL_LIMIT: usize = 50;

fn stale_resolution_line(item: &codefire_core::StaleResolution) -> String {
    let fire = if item.fire_display_id.is_empty() {
        &item.fire_uid
    } else {
        &item.fire_display_id
    };
    if fire.is_empty() {
        return format!("  {}: {}\n", item.resolution_uid, item.reason);
    }
    format!(
        "  {fire} ({})  {} -> {}: {}\n",
        item.resolution_uid, item.source_atom_id, item.target_atom_id, item.reason
    )
}
