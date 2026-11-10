//! Parsing of codefire.policy.yaml into the verification policy and its effective-policy provenance.

use crate::effective_policy::PolicyProvenance;
use crate::{
    default_trace_policy, effective_policy, flush_required_rule, parse_env_field, parse_u64_field,
    parse_usize_field, read_text_lossy, set_required_rule_field, unquote, CoreError,
    ParsedVerificationPolicy, RequiredLinkRule, RequiredLinkRuleDraft, VerificationCommand,
    VerificationPolicy, DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES,
    DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS,
};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct LimitedYamlLine<'a> {
    pub(crate) indent: usize,
    pub(crate) text: &'a str,
}

pub(crate) fn limited_yaml_line<'a>(
    raw_line: &'a str,
    line_no: usize,
    file_name: &str,
) -> Result<Option<LimitedYamlLine<'a>>, CoreError> {
    let indent = raw_line
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count();
    if raw_line[indent..].starts_with('\t') {
        return Err(CoreError::Config(format!(
            "unsupported {file_name}: tab indentation at line {line_no}"
        )));
    }
    let without_comment = strip_limited_yaml_comment(raw_line);
    let text = without_comment.trim_end();
    let stripped = text.trim_start_matches(' ');
    if stripped.is_empty() {
        return Ok(None);
    }
    Ok(Some(LimitedYamlLine {
        indent: text.len() - stripped.len(),
        text: stripped,
    }))
}

pub(crate) fn strip_limited_yaml_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_double => escaped = true,
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '#' if !in_single && !in_double => return &line[..index],
            _ => {}
        }
    }
    line
}

pub(crate) fn limited_yaml_key(text: &str) -> Option<(&str, &str)> {
    let (key, value) = text.split_once(':')?;
    let key = key.trim();
    if key.is_empty() || key.starts_with('-') {
        return None;
    }
    Some((key, value.trim()))
}

pub fn parse_verification_policy(open_dir: &Path) -> Result<VerificationPolicy, CoreError> {
    Ok(parse_effective_verification_policy(open_dir)?.policy)
}

pub fn parse_effective_verification_policy(
    open_dir: &Path,
) -> Result<ParsedVerificationPolicy, CoreError> {
    let path = open_dir.join("codefire.policy.yaml");
    let mut policy = default_verification_policy();
    let mut provenance = PolicyProvenance::default();
    if !path.exists() {
        let effective = effective_policy(&policy, &provenance)?;
        return Ok(ParsedVerificationPolicy { policy, effective });
    }

    let mut required_links = BTreeMap::<String, Vec<RequiredLinkRule>>::new();
    let mut in_required_links = false;
    let mut in_commit_policy = false;
    let mut in_extinguish_policy = false;
    let mut in_no_change_required_policy = false;
    let mut in_verification = false;
    let mut in_verification_required_wrapper = false;
    let mut saw_required_links = false;
    let mut current_kind = None::<String>;
    let mut current_rule = None::<RequiredLinkRuleDraft>;
    let mut current_check = None::<VerificationCommandDraft>;
    let mut checks = Vec::new();

    for (line_index, raw_line) in read_text_lossy(&path)?.lines().enumerate() {
        let line_no = line_index + 1;
        let Some(line) = limited_yaml_line(raw_line, line_no, "codefire.policy.yaml")? else {
            continue;
        };
        let stripped = line.text;
        if line.indent == 0 {
            if stripped == "version:" || stripped.starts_with("version:") {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "required_links:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = true;
                saw_required_links = true;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "commit_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = true;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "extinguish_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = true;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "verification:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = true;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: root key at line {line_no}"
                )));
            }
            continue;
        }
        if in_commit_policy {
            if line.indent != 2 {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: unsupported commit_policy indentation at line {line_no}"
                )));
            }
            let Some((key, value)) = limited_yaml_key(stripped) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: commit_policy entry at line {line_no} must be key: value"
                )));
            };
            set_commit_policy_bool(&mut policy, key, value, line_no)?;
            provenance.record_repository_rule(commit_policy_rule_id(key));
            continue;
        }
        if in_extinguish_policy {
            if line.indent == 2 && stripped == "no-change-required:" {
                in_no_change_required_policy = true;
                continue;
            }
            if in_no_change_required_policy {
                if line.indent != 4 {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: unsupported no-change-required indentation at line {line_no}"
                    )));
                }
                let Some((key, value)) = limited_yaml_key(stripped) else {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: no-change-required entry at line {line_no} must be key: value"
                    )));
                };
                if key == "requires_rationale" {
                    policy.no_change_required_requires_rationale = parse_bool_field(value.trim())?;
                    provenance
                        .record_repository_rule("extinguish.no_change_required.requires_rationale");
                    continue;
                }
            }
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: extinguish_policy entry at line {line_no}"
            )));
        }
        if in_verification {
            if line.indent == 2 && stripped == "required:" {
                flush_verification_check(&mut checks, current_check.take())?;
                in_verification_required_wrapper = true;
                continue;
            }
            let item_indent = if in_verification_required_wrapper {
                4
            } else {
                2
            };
            let field_indent = if in_verification_required_wrapper {
                6
            } else {
                4
            };
            if line.indent == item_indent && stripped.starts_with("- ") {
                let item = stripped.trim_start_matches("- ").trim();
                let Some((key, value)) = limited_yaml_key(item) else {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: verification item at line {line_no} must be '- key: value'"
                    )));
                };
                flush_verification_check(&mut checks, current_check.take())?;
                let mut check = VerificationCommandDraft {
                    id: None,
                    command: None,
                    cwd: Some(".".to_string()),
                    timeout_ms: None,
                    max_output_bytes: None,
                    inherit_env: None,
                    env: BTreeMap::new(),
                    allow_failure: None,
                };
                set_verification_check_field(&mut check, key, value, line_no)?;
                current_check = Some(check);
                continue;
            }
            if line.indent != field_indent {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: unsupported verification indentation at line {line_no}; use verification: [items] or legacy verification.required: [items]"
                )));
            }
            let Some(check) = current_check.as_mut() else {
                return Err(CoreError::Config(format!(
                    "invalid codefire.policy.yaml: verification field before item at line {line_no}"
                )));
            };
            let Some((key, value)) = limited_yaml_key(stripped) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: verification entry at line {line_no} must be key: value"
                )));
            };
            set_verification_check_field(check, key, value, line_no)?;
            continue;
        }
        if !in_required_links {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: nested entry outside supported section at line {line_no}"
            )));
        }
        if line.indent == 2 && stripped.ends_with(':') {
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            let kind = stripped.trim_end_matches(':').to_string();
            required_links.entry(kind.clone()).or_default();
            current_kind = Some(kind);
            continue;
        }
        if line.indent == 4 && stripped.starts_with("- ") {
            let item = stripped.trim_start_matches("- ").trim();
            let Some((key, value)) = limited_yaml_key(item) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: required link rule at line {line_no} must be '- key: value'"
                )));
            };
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            let mut rule = RequiredLinkRuleDraft {
                link_type: None,
                target_kind: None,
                min: None,
            };
            set_required_rule_field(&mut rule, key, value, line_no)?;
            current_rule = Some(rule);
            continue;
        }
        if line.indent != 6 {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: unsupported required_links indentation at line {line_no}"
            )));
        }
        let Some(rule) = current_rule.as_mut() else {
            return Err(CoreError::Config(format!(
                "invalid codefire.policy.yaml: required link field before rule at line {line_no}"
            )));
        };
        let Some((key, value)) = limited_yaml_key(stripped) else {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: required link entry at line {line_no} must be key: value"
            )));
        };
        set_required_rule_field(rule, key, value, line_no)?;
    }
    flush_required_rule(&mut required_links, &current_kind, current_rule)?;
    flush_verification_check(&mut checks, current_check)?;
    if required_links.is_empty() {
        required_links = policy.required_links;
    }
    policy.required_links = required_links;
    policy.verification = checks;
    if saw_required_links {
        provenance.record_repository_prefix("trace.required_links.");
    }
    if !policy.verification.is_empty() {
        provenance.record_repository_prefix("verification.command.");
    }
    let effective = effective_policy(&policy, &provenance)?;
    Ok(ParsedVerificationPolicy { policy, effective })
}

pub(crate) fn commit_policy_rule_id(key: &str) -> &'static str {
    match key {
        "require_no_required_fires" => "commit.require_no_required_fires",
        "require_no_stale_resolutions" => "commit.require_no_stale_resolutions",
        "require_trace_completeness" => "commit.require_trace_completeness",
        "require_verification_success" => "commit.require_verification_success",
        "reject_duplicate_atom_ids" => "commit.reject_duplicate_atom_ids",
        "require_commit_signature" => "commit.require_commit_signature",
        _ => unreachable!("set_commit_policy_bool accepted a known key"),
    }
}

pub(crate) fn set_commit_policy_bool(
    policy: &mut VerificationPolicy,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), CoreError> {
    let parsed = parse_bool_field(value)?;
    match key {
        "require_no_required_fires" => {
            policy.require_no_required_fires = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_no_stale_resolutions" => {
            policy.require_no_stale_resolutions = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_trace_completeness" => policy.require_trace_completeness = parsed,
        "require_verification_success" => {
            policy.require_verification_success = safety_floor_policy_value(key, parsed, line_no)?
        }
        "reject_duplicate_atom_ids" => {
            policy.reject_duplicate_atom_ids = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_commit_signature" => policy.require_commit_signature = parsed,
        _ => {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: commit_policy field {key} at line {line_no}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn safety_floor_policy_value(
    key: &str,
    parsed: bool,
    line_no: usize,
) -> Result<bool, CoreError> {
    if parsed {
        Ok(true)
    } else {
        Err(CoreError::Config(format!(
            "invalid codefire.policy.yaml: commit_policy.{key} must be true at line {line_no}; rule {} source repository_config adoption rejected because safety-floor values cannot be relaxed; only commit.require_trace_completeness may relax a committable check",
            commit_policy_rule_id(key),
        )))
    }
}

pub(crate) fn parse_bool_field(value: &str) -> Result<bool, CoreError> {
    match unquote(value).trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(CoreError::Config(format!(
            "invalid boolean value in policy: {value}"
        ))),
    }
}

pub fn default_verification_policy() -> VerificationPolicy {
    VerificationPolicy {
        require_no_required_fires: true,
        require_no_stale_resolutions: true,
        require_trace_completeness: true,
        require_verification_success: true,
        reject_duplicate_atom_ids: true,
        require_commit_signature: false,
        no_change_required_requires_rationale: true,
        required_links: default_trace_policy().required_links,
        verification: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerificationCommandDraft {
    pub(crate) id: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) cwd: Option<String>,
    pub(crate) timeout_ms: Option<u64>,
    pub(crate) max_output_bytes: Option<usize>,
    pub(crate) inherit_env: Option<bool>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) allow_failure: Option<bool>,
}

pub(crate) fn flush_verification_check(
    checks: &mut Vec<VerificationCommand>,
    draft: Option<VerificationCommandDraft>,
) -> Result<(), CoreError> {
    let Some(draft) = draft else {
        return Ok(());
    };
    let command = draft.command.ok_or_else(|| {
        CoreError::Config(
            "invalid codefire.policy.yaml: verification entry missing command".to_string(),
        )
    })?;
    let id = draft
        .id
        .unwrap_or_else(|| format!("verification-{}", checks.len() + 1));
    checks.push(VerificationCommand {
        id,
        command,
        cwd: draft.cwd.unwrap_or_else(|| ".".to_string()),
        timeout_ms: draft
            .timeout_ms
            .unwrap_or(DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS),
        max_output_bytes: draft
            .max_output_bytes
            .unwrap_or(DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES),
        inherit_env: draft.inherit_env.unwrap_or(false),
        env: draft.env,
        allow_failure: draft.allow_failure.unwrap_or(false),
    });
    Ok(())
}

pub(crate) fn set_verification_check_field(
    check: &mut VerificationCommandDraft,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), CoreError> {
    match key {
        "id" => check.id = Some(unquote(value)),
        "command" => check.command = Some(unquote(value)),
        "cwd" => check.cwd = Some(unquote(value)),
        "timeout_ms" => {
            check.timeout_ms = Some(parse_u64_field("verification timeout_ms", value.trim())?)
        }
        "max_output_bytes" => {
            check.max_output_bytes = Some(parse_usize_field(
                "verification max_output_bytes",
                value.trim(),
            )?)
        }
        "inherit_env" => check.inherit_env = Some(parse_bool_field(value)?),
        "env" => check.env = parse_env_field(value, line_no)?,
        "allow_failure" => check.allow_failure = Some(parse_bool_field(value)?),
        _ => {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: verification field {key} at line {line_no}"
            )));
        }
    }
    Ok(())
}
