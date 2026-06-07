use super::{open_context, parse_lock_option, CliError, LockOptions, RepoLock};
use crate::limited_yaml;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static LINK_BATCH_TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
const YAML_CONTEXT: &str = "link batch YAML";

#[derive(Debug)]
pub(super) struct LinkBatchOptions {
    pub(super) path: PathBuf,
    pub(super) batch_path: PathBuf,
    pub(super) dry_run: bool,
    pub(super) json_output: bool,
    pub(super) lock: LockOptions,
}

#[derive(Debug)]
pub(super) struct LinkBatchResult {
    pub(super) repo_root: PathBuf,
    pub(super) open_dir: PathBuf,
    pub(super) links_file: PathBuf,
    pub(super) item_count: usize,
    pub(super) dry_run: bool,
    pub(super) valid: bool,
    pub(super) diagnostics: Vec<Value>,
    pub(super) plan: Value,
    pub(super) added_links: Vec<LinkBatchLink>,
}

#[derive(Debug, Clone, Default)]
struct LinkBatchDefaults {
    link_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct LinkBatchLink {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) link_type: String,
}

#[derive(Debug, Clone, Default)]
struct LinkBatchLinkSpec {
    from: Option<String>,
    to: Option<String>,
    link_type: Option<String>,
}

#[derive(Debug)]
struct LinkBatchFile {
    version: u64,
    defaults: LinkBatchDefaults,
    links: Vec<LinkBatchLinkSpec>,
}

pub(super) fn parse_link_batch_args(args: &[String]) -> Result<LinkBatchOptions, CliError> {
    let mut path = None;
    let mut batch_path = None;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        if parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--batch" => {
                index += 1;
                batch_path = Some(PathBuf::from(required_arg(args, index, "--batch")?));
            }
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--batch=") => {
                batch_path = Some(PathBuf::from(value.trim_start_matches("--batch=")));
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported link option: {option}"
                )));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected link argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(LinkBatchOptions {
        path: path.unwrap_or(std::env::current_dir()?),
        batch_path: batch_path.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire link --batch <file> [--path <open-dir>] [--dry-run] [--json]"
                    .to_string(),
            )
        })?,
        dry_run,
        json_output,
        lock,
    })
}

pub(super) fn run_link_batch(options: &LinkBatchOptions) -> Result<LinkBatchResult, CliError> {
    let batch = parse_link_batch_file(&options.batch_path)?;
    if batch.version != 1 {
        return Err(CliError::Usage(format!(
            "unsupported link batch version: {}",
            batch.version
        )));
    }
    if batch.links.is_empty() {
        return Err(CliError::Usage(
            "link batch requires at least one link".to_string(),
        ));
    }

    let context = open_context(&options.path)?;
    let _lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(
            &context.repo_root,
            &options.lock,
        )?)
    };
    let resolved = resolve_links(&batch)?;
    let diagnostics = validate_links(&context.open_dir, &resolved)?;
    let links_file = context.open_dir.join("codefire.links.yaml");
    let valid = diagnostics.is_empty();
    if !valid && !options.dry_run {
        return Err(CliError::Usage(link_batch_validation_message(&diagnostics)));
    }
    let plan = link_batch_operation_plan(
        options,
        &context.open_dir,
        &links_file,
        &resolved,
        &diagnostics,
    );

    if !options.dry_run && valid {
        append_links(&links_file, &resolved)?;
    }

    Ok(LinkBatchResult {
        repo_root: context.repo_root,
        open_dir: context.open_dir,
        links_file,
        item_count: resolved.len(),
        dry_run: options.dry_run,
        valid,
        diagnostics,
        plan,
        added_links: if options.dry_run {
            Vec::new()
        } else {
            resolved
        },
    })
}

pub(super) fn link_batch_data_json(result: &LinkBatchResult) -> Value {
    json!({
        "type": "codefire_link_batch_result",
        "version": 1,
        "dry_run": result.dry_run,
        "valid": result.valid,
        "item_count": result.item_count,
        "open_dir": &result.open_dir,
        "links_file": &result.links_file,
        "diagnostics": &result.diagnostics,
        "plan": &result.plan,
        "added_links": result.added_links.iter().map(link_json).collect::<Vec<_>>(),
    })
}

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn resolve_links(batch: &LinkBatchFile) -> Result<Vec<LinkBatchLink>, CliError> {
    let mut links = Vec::with_capacity(batch.links.len());
    for (index, spec) in batch.links.iter().enumerate() {
        let from = required_link_field(spec.from.as_deref(), index, "from")?;
        let to = required_link_field(spec.to.as_deref(), index, "to")?;
        let link_type = spec
            .link_type
            .as_deref()
            .or(batch.defaults.link_type.as_deref())
            .ok_or_else(|| missing_link_field(index, "type"))?;
        links.push(LinkBatchLink {
            from: from.to_string(),
            to: to.to_string(),
            link_type: link_type.to_string(),
        });
    }
    Ok(links)
}

fn required_link_field<'a>(
    value: Option<&'a str>,
    index: usize,
    field: &str,
) -> Result<&'a str, CliError> {
    let value = value.ok_or_else(|| missing_link_field(index, field))?;
    if value.trim().is_empty() {
        return Err(missing_link_field(index, field));
    }
    Ok(value)
}

fn missing_link_field(index: usize, field: &str) -> CliError {
    CliError::Usage(format!("link batch item {} missing {field}", index + 1))
}

fn validate_links(open_dir: &Path, links: &[LinkBatchLink]) -> Result<Vec<Value>, CliError> {
    let atom_index = codefire_core::build_atom_index(open_dir)?;
    let atom_ids = atom_index
        .atoms
        .iter()
        .map(|atom| atom.atom_id.as_str())
        .collect::<BTreeSet<_>>();
    let existing = codefire_core::current_trace_graph(open_dir)?
        .links
        .into_iter()
        .map(|link| (link.from, link.to, link.link_type))
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for (index, link) in links.iter().enumerate() {
        if link.from == link.to {
            diagnostics.push(link_validation_diagnostic(
                index,
                "self_link",
                format!("link batch self-link is not allowed: {}", link.from),
                link,
            ));
        }
        if !atom_ids.contains(link.from.as_str()) {
            diagnostics.push(link_validation_diagnostic(
                index,
                "unknown_from_atom",
                format!("link batch references unknown from atom: {}", link.from),
                link,
            ));
        }
        if !atom_ids.contains(link.to.as_str()) {
            diagnostics.push(link_validation_diagnostic(
                index,
                "unknown_to_atom",
                format!("link batch references unknown to atom: {}", link.to),
                link,
            ));
        }
        let key = (link.from.clone(), link.to.clone(), link.link_type.clone());
        if !seen.insert(key.clone()) {
            diagnostics.push(link_validation_diagnostic(
                index,
                "duplicate_batch_link",
                format!(
                    "duplicate link batch item: {} --{}--> {}",
                    link.from, link.link_type, link.to
                ),
                link,
            ));
        }
        if existing.contains(&key) {
            diagnostics.push(link_validation_diagnostic(
                index,
                "existing_link",
                format!(
                    "link already exists: {} --{}--> {}",
                    link.from, link.link_type, link.to
                ),
                link,
            ));
        }
    }
    Ok(diagnostics)
}

fn link_validation_diagnostic(
    index: usize,
    kind: &str,
    message: String,
    link: &LinkBatchLink,
) -> Value {
    json!({
        "kind": kind,
        "severity": "error",
        "message": message,
        "item_index": index,
        "item_number": index + 1,
        "link": link_json(link),
    })
}

fn link_batch_validation_message(diagnostics: &[Value]) -> String {
    let mut message = format!(
        "link batch validation failed: {} issue(s)",
        diagnostics.len()
    );
    for diagnostic in diagnostics {
        if let Some(detail) = diagnostic.get("message").and_then(Value::as_str) {
            message.push_str("; ");
            message.push_str(detail);
        }
    }
    message
}

fn append_links(path: &Path, links: &[LinkBatchLink]) -> Result<(), CliError> {
    let mut text = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if text.trim().is_empty() {
        text.push_str("version: 1\nlinks:\n");
    } else {
        validate_existing_links_yaml(&text)?;
        text = insert_links_into_existing_yaml(text, links)?;
    }
    if text.ends_with("links:\n") {
        append_link_lines(&mut text, links);
    }
    let temp_path = unique_yaml_temp_path(path);
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
}

fn append_link_lines(text: &mut String, links: &[LinkBatchLink]) {
    for link in links {
        text.push_str(&format!(
            "  - from: {}\n    to: {}\n    type: {}\n",
            yaml_quote(&link.from),
            yaml_quote(&link.to),
            yaml_quote(&link.link_type)
        ));
    }
}

fn validate_existing_links_yaml(text: &str) -> Result<(), CliError> {
    let mut root_links_count = 0usize;
    let mut unsupported_roots = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || line.starts_with(' ') {
            continue;
        }
        if trimmed == "links:" {
            root_links_count += 1;
        } else if trimmed != "version: 1" && !trimmed.starts_with("version:") {
            unsupported_roots.push(trimmed.to_string());
        }
    }
    if root_links_count > 1 {
        return Err(CliError::Usage(
            "unsupported codefire.links.yaml: duplicate root links sections".to_string(),
        ));
    }
    if root_links_count == 0 && !unsupported_roots.is_empty() {
        return Err(CliError::Usage(format!(
            "unsupported codefire.links.yaml: missing root links section before root key {}",
            unsupported_roots[0]
        )));
    }
    Ok(())
}

fn insert_links_into_existing_yaml(
    mut text: String,
    links: &[LinkBatchLink],
) -> Result<String, CliError> {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(links_index) = lines.iter().position(|line| line.trim() == "links:") else {
        let mut output = text;
        output.push_str("links:\n");
        append_link_lines(&mut output, links);
        return Ok(output);
    };
    let mut insert_index = lines.len();
    for (index, line) in lines.iter().enumerate().skip(links_index + 1) {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !line.starts_with(' ')
            && trimmed.ends_with(':')
        {
            insert_index = index;
            break;
        }
    }
    let mut output = String::new();
    for line in &lines[..insert_index] {
        output.push_str(line);
        output.push('\n');
    }
    append_link_lines(&mut output, links);
    for line in &lines[insert_index..] {
        output.push_str(line);
        output.push('\n');
    }
    Ok(output)
}

fn unique_yaml_temp_path(path: &Path) -> PathBuf {
    let counter = LINK_BATCH_TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_extension(format!("yaml.{}.{}.tmp", std::process::id(), counter))
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn link_batch_operation_plan(
    options: &LinkBatchOptions,
    open_dir: &Path,
    links_file: &Path,
    links: &[LinkBatchLink],
    diagnostics: &[Value],
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "link-batch",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "open_dir": open_dir,
        "batch_file": &options.batch_path,
        "links_file": links_file,
        "item_count": links.len(),
        "valid": diagnostics.is_empty(),
        "diagnostics": diagnostics,
        "items": links.iter().map(link_json).collect::<Vec<_>>(),
        "operations": [
            {"kind": "validate_all_batch_items", "count": links.len()},
            {"kind": "validate_atom_references", "count": links.len()},
            {"kind": "append_trace_links", "count": links.len()},
        ],
        "next_actions": [
            {"kind": "verify", "command": "codefire verify --details --json", "target": {"path": open_dir}},
            {"kind": "trace_graph", "command": "codefire trace-graph", "target": {"path": open_dir}},
        ],
    })
}

fn link_json(link: &LinkBatchLink) -> Value {
    json!({
        "from": &link.from,
        "to": &link.to,
        "type": &link.link_type,
    })
}

fn parse_link_batch_file(path: &Path) -> Result<LinkBatchFile, CliError> {
    let text = fs::read_to_string(path)?;
    if text.trim_start().starts_with('{') {
        parse_link_batch_json(&text)
    } else {
        parse_link_batch_yaml(&text)
    }
}

fn parse_link_batch_json(text: &str) -> Result<LinkBatchFile, CliError> {
    let value: Value = serde_json::from_str(text)?;
    let version = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    let defaults = parse_json_defaults(value.get("defaults"))?;
    let links = value
        .get("links")
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage("link batch JSON missing links array".to_string()))?
        .iter()
        .map(parse_json_link)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LinkBatchFile {
        version,
        defaults,
        links,
    })
}

fn parse_json_defaults(value: Option<&Value>) -> Result<LinkBatchDefaults, CliError> {
    let Some(value) = value else {
        return Ok(LinkBatchDefaults::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("link batch defaults must be an object".to_string()))?;
    Ok(LinkBatchDefaults {
        link_type: optional_json_string(object.get("type"), "defaults.type")?,
    })
}

fn parse_json_link(value: &Value) -> Result<LinkBatchLinkSpec, CliError> {
    let object = value
        .as_object()
        .ok_or_else(|| CliError::Usage("link batch item must be an object".to_string()))?;
    Ok(LinkBatchLinkSpec {
        from: optional_json_string(object.get("from"), "link.from")?,
        to: optional_json_string(object.get("to"), "link.to")?,
        link_type: optional_json_string(object.get("type"), "link.type")?,
    })
}

fn optional_json_string(value: Option<&Value>, field: &str) -> Result<Option<String>, CliError> {
    value
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| CliError::Usage(format!("{field} must be a string")))
        })
        .transpose()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkBatchYamlSection {
    Root,
    Defaults,
    Links,
}

fn parse_link_batch_yaml(text: &str) -> Result<LinkBatchFile, CliError> {
    let mut version = None;
    let mut defaults = LinkBatchDefaults::default();
    let mut links = Vec::new();
    let mut current_link = None;
    let mut section = LinkBatchYamlSection::Root;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        if raw_line.contains('\t') {
            return Err(CliError::Usage(format!(
                "invalid link batch YAML: tabs are not supported at line {line_number}"
            )));
        }
        let without_comment = limited_yaml::strip_comment(raw_line);
        if without_comment.trim().is_empty() {
            continue;
        }
        let indent = without_comment
            .as_bytes()
            .iter()
            .take_while(|byte| **byte == b' ')
            .count();
        let text = without_comment.trim();
        match (indent, text, section) {
            (0, "defaults:", _) => section = LinkBatchYamlSection::Defaults,
            (0, "links:", _) => {
                push_current_link(&mut links, &mut current_link);
                section = LinkBatchYamlSection::Links;
            }
            (0, _, _) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                if key == "version" {
                    version = Some(limited_yaml::parse_u64(
                        &value,
                        "version",
                        line_number,
                        YAML_CONTEXT,
                    )?);
                    section = LinkBatchYamlSection::Root;
                } else {
                    return Err(CliError::Usage(format!(
                        "invalid link batch YAML: unsupported root key '{key}' at line {line_number}"
                    )));
                }
            }
            (2, _, LinkBatchYamlSection::Defaults) => {
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_default(&mut defaults, &key, value, line_number)?;
            }
            (2, _, LinkBatchYamlSection::Links) if text.starts_with("- ") => {
                push_current_link(&mut links, &mut current_link);
                let rest = text[2..].trim();
                let mut link = LinkBatchLinkSpec::default();
                if !rest.is_empty() {
                    let (key, value) =
                        limited_yaml::parse_key_value(rest, line_number, YAML_CONTEXT)?;
                    apply_yaml_link_field(&mut link, &key, value, line_number)?;
                }
                current_link = Some(link);
            }
            (4, _, LinkBatchYamlSection::Links) => {
                let link = current_link.as_mut().ok_or_else(|| {
                    CliError::Usage(format!(
                        "invalid link batch YAML: link field before list item at line {line_number}"
                    ))
                })?;
                let (key, value) = limited_yaml::parse_key_value(text, line_number, YAML_CONTEXT)?;
                apply_yaml_link_field(link, &key, value, line_number)?;
            }
            _ => {
                return Err(CliError::Usage(format!(
                    "invalid link batch YAML indentation or section at line {line_number}"
                )));
            }
        }
    }
    push_current_link(&mut links, &mut current_link);

    Ok(LinkBatchFile {
        version: version
            .ok_or_else(|| CliError::Usage("link batch YAML missing version".to_string()))?,
        defaults,
        links,
    })
}

fn push_current_link(links: &mut Vec<LinkBatchLinkSpec>, current: &mut Option<LinkBatchLinkSpec>) {
    if let Some(link) = current.take() {
        links.push(link);
    }
}

fn apply_yaml_default(
    defaults: &mut LinkBatchDefaults,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "type" => defaults.link_type = Some(value),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid link batch YAML: unsupported defaults key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

fn apply_yaml_link_field(
    link: &mut LinkBatchLinkSpec,
    key: &str,
    value: String,
    line_number: usize,
) -> Result<(), CliError> {
    match key {
        "from" => link.from = Some(value),
        "to" => link.to = Some(value),
        "type" => link.link_type = Some(value),
        _ => {
            return Err(CliError::Usage(format!(
                "invalid link batch YAML: unsupported link key '{key}' at line {line_number}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_batch_temp_paths_are_unique_for_same_target() {
        let path = PathBuf::from("codefire.links.yaml");
        let first = unique_yaml_temp_path(&path);
        let second = unique_yaml_temp_path(&path);
        assert_ne!(first, second);
        assert!(first.to_string_lossy().contains(".tmp"));
        assert!(second.to_string_lossy().contains(".tmp"));
    }
}
