use super::{
    compute_scan, open_context, parse_lock_option, read_json, required_string, run_extinguish,
    CliError, ExtinguishOptions, LockOptions,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

const DEFAULT_EVIDENCE_CANDIDATE_LIMIT: usize = 10;

#[derive(Debug)]
pub(super) struct InteractiveExtinguishOptions {
    pub(super) path: PathBuf,
    pub(super) json_output: bool,
    pub(super) dry_run: bool,
    pub(super) evidence_limit: usize,
}

#[derive(Debug)]
pub(super) struct InteractiveExtinguishResult {
    pub(super) plan: Value,
}

#[derive(Debug)]
pub(super) struct AllMatchingExtinguishOptions {
    pub(super) path: PathBuf,
    pub(super) query: String,
    pub(super) resolution: String,
    pub(super) rationale: String,
    pub(super) evidence: String,
    pub(super) evidence_refs: Vec<String>,
    pub(super) refresh: bool,
    pub(super) dry_run: bool,
    pub(super) json_output: bool,
    pub(super) lock: LockOptions,
    pub(super) edit_rationale: bool,
}

#[derive(Debug)]
pub(super) struct AllMatchingExtinguishResult {
    pub(super) item_count: usize,
    pub(super) plan: Value,
}

#[derive(Debug)]
struct EvidenceCandidate {
    id: String,
    created_at: String,
    label: Option<String>,
    artifact_ref: Option<String>,
    command: Option<Value>,
    modified_at_unix: u64,
}

#[derive(Debug)]
struct MatchedFire {
    id: String,
    source_atom: String,
    target_atom: String,
    severity: String,
    reason: String,
    validation_plan: Value,
}

pub(super) fn has_interactive_extinguish_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--interactive")
}

pub(super) fn has_all_matching_extinguish_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--all-matching" || arg.starts_with("--all-matching="))
}

pub(super) fn parse_interactive_extinguish_args(
    args: &[String],
) -> Result<InteractiveExtinguishOptions, CliError> {
    let mut path = None;
    let mut json_output = false;
    let mut dry_run = false;
    let mut evidence_limit = DEFAULT_EVIDENCE_CANDIDATE_LIMIT;
    let mut saw_interactive = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--interactive" => saw_interactive = true,
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--json" => json_output = true,
            "--dry-run" => dry_run = true,
            "--evidence-limit" => {
                index += 1;
                evidence_limit =
                    parse_positive_usize(required_arg(args, index, "--evidence-limit")?)?;
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--evidence-limit=") => {
                evidence_limit =
                    parse_positive_usize(value.trim_start_matches("--evidence-limit="))?;
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported interactive extinguish option: {value}"
                )));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected interactive extinguish argument: {value}"
                )));
            }
        }
        index += 1;
    }
    if !saw_interactive {
        return Err(CliError::Usage(
            "usage: codefire-rs extinguish --interactive [--path <open-dir>] [--json] [--dry-run]"
                .to_string(),
        ));
    }
    Ok(InteractiveExtinguishOptions {
        path: path.unwrap_or(env::current_dir()?),
        json_output,
        dry_run,
        evidence_limit,
    })
}

pub(super) fn run_interactive_extinguish(
    options: &InteractiveExtinguishOptions,
) -> Result<InteractiveExtinguishResult, CliError> {
    let context = open_context(&options.path)?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let scan = compute_scan(&context.open_dir, false)?;
    let open_fires = scan
        .fires
        .iter()
        .filter(|fire| fire.status == "open")
        .map(|fire| {
            json!({
                "display_id": &fire.display_id,
                "fire_uid": &fire.fire_uid,
                "source_atom": &fire.source.atom_id,
                "target_atom": &fire.target.atom_id,
                "severity": &fire.severity,
                "reason": &fire.reason,
            })
        })
        .collect::<Vec<_>>();
    let evidence_candidates =
        recent_evidence_candidates(&context.repo_root, options.evidence_limit)?;
    let first_evidence_id = evidence_candidates
        .first()
        .map(|candidate| candidate.id.clone());
    let draft_commands = scan
        .fires
        .iter()
        .filter(|fire| fire.status == "open")
        .map(|fire| {
            let mut command = format!(
                "codefire-rs extinguish {} --resolution addressed --edit-rationale",
                fire.display_id
            );
            if let Some(evidence_id) = &first_evidence_id {
                command.push_str(&format!(" --evidence-ref {evidence_id}"));
            }
            json!({
                "fire": &fire.display_id,
                "command": command,
            })
        })
        .collect::<Vec<_>>();
    let plan = json!({
        "type": "codefire_extinguish_interactive_plan",
        "version": 1,
        "command": "extinguish-interactive",
        "dry_run": options.dry_run,
        "branch": &context.branch,
        "open_dir": &context.open_dir,
        "active_state_path": active_state_path,
        "open_fires": open_fires,
        "evidence_candidates": evidence_candidates_json(&evidence_candidates),
        "editor": {
            "flag": "--edit-rationale",
            "env": ["CODEFIRE_EDITOR", "EDITOR"],
        },
        "draft_commands": draft_commands,
        "next_actions": [
            {"kind": "extinguish", "command": "codefire-rs extinguish FIRE-001 --resolution addressed --edit-rationale --evidence-ref <CF-EVIDENCE-id>"},
            {"kind": "multi_extinguish", "command": "codefire-rs extinguish --all-matching \"REQ-id -> DES-id\" --resolution addressed --edit-rationale --evidence-ref <CF-EVIDENCE-id>"},
            {"kind": "verify", "command": "codefire-rs verify --details --json"},
        ],
    });
    Ok(InteractiveExtinguishResult { plan })
}

pub(super) fn print_interactive_extinguish_result(result: &InteractiveExtinguishResult) {
    let fires = result.plan["open_fires"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0);
    let evidence = result.plan["evidence_candidates"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0);
    println!("interactive extinguish plan: {fires} open fires, {evidence} evidence candidates");
    if let Some(commands) = result.plan["draft_commands"].as_array() {
        for command in commands.iter().take(5) {
            if let Some(text) = command["command"].as_str() {
                println!("{text}");
            }
        }
    }
}

pub(super) fn parse_all_matching_extinguish_args(
    args: &[String],
) -> Result<AllMatchingExtinguishOptions, CliError> {
    let mut path = None;
    let mut query = None;
    let mut resolution = "addressed".to_string();
    let mut rationale = String::new();
    let mut evidence = String::new();
    let mut evidence_refs = Vec::new();
    let mut refresh = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut edit_rationale = false;
    let mut index = 0usize;
    while index < args.len() {
        if parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--all-matching" => {
                index += 1;
                query = Some(required_arg(args, index, "--all-matching")?.to_string());
            }
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(required_arg(args, index, "--path")?));
            }
            "--resolution" => {
                index += 1;
                resolution = required_arg(args, index, "--resolution")?.to_string();
            }
            "--rationale" => {
                index += 1;
                rationale = required_arg(args, index, "--rationale")?.to_string();
            }
            "--evidence" => {
                index += 1;
                evidence = required_arg(args, index, "--evidence")?.to_string();
            }
            "--evidence-ref" => {
                index += 1;
                evidence_refs.push(required_arg(args, index, "--evidence-ref")?.to_string());
            }
            "--refresh" => refresh = true,
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            "--edit-rationale" => edit_rationale = true,
            value if value.starts_with("--all-matching=") => {
                query = Some(value.trim_start_matches("--all-matching=").to_string());
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--evidence-ref=") => {
                evidence_refs.push(value.trim_start_matches("--evidence-ref=").to_string());
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported all-matching extinguish option: {value}"
                )));
            }
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected all-matching extinguish argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(AllMatchingExtinguishOptions {
        path: path.unwrap_or(env::current_dir()?),
        query: query.ok_or_else(|| {
            CliError::Usage(
                "usage: codefire-rs extinguish --all-matching \"SOURCE -> TARGET\" [--path <open-dir>] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>)".to_string(),
            )
        })?,
        resolution,
        rationale,
        evidence,
        evidence_refs,
        refresh,
        dry_run,
        json_output,
        lock,
        edit_rationale,
    })
}

pub(super) fn run_all_matching_extinguish(
    options: &AllMatchingExtinguishOptions,
) -> Result<AllMatchingExtinguishResult, CliError> {
    let (source_atom, target_atom) = parse_source_target_query(&options.query)?;
    if options.rationale.is_empty()
        && options.evidence.is_empty()
        && options.evidence_refs.is_empty()
        && !options.edit_rationale
    {
        return Err(CliError::Usage(
            "extinguish --all-matching requires --rationale, --evidence, or --evidence-ref"
                .to_string(),
        ));
    }
    let rationale = if options.edit_rationale {
        edit_rationale_with_configured_editor(&options.rationale)?
    } else {
        options.rationale.clone()
    };

    let context = open_context(&options.path)?;
    let scan = compute_scan(&context.open_dir, false)?;
    let fire_ids = scan
        .fires
        .iter()
        .filter(|fire| {
            fire.status == "open"
                && fire.source.atom_id == source_atom
                && fire.target.atom_id == target_atom
        })
        .map(|fire| fire.display_id.clone())
        .collect::<Vec<_>>();
    if fire_ids.is_empty() {
        return Err(CliError::Usage(format!(
            "no open fires match: {} -> {}",
            source_atom, target_atom
        )));
    }

    let mut matched = Vec::with_capacity(fire_ids.len());
    for id in &fire_ids {
        let validation = run_extinguish(&ExtinguishOptions {
            path: options.path.clone(),
            fire_id: id.clone(),
            resolution: options.resolution.clone(),
            rationale: rationale.clone(),
            evidence: options.evidence.clone(),
            evidence_refs: options.evidence_refs.clone(),
            refresh: options.refresh,
            dry_run: true,
            json_output: true,
            lock: options.lock,
            idempotency_key: None,
            edit_rationale: false,
        })?;
        let fire = scan
            .fires
            .iter()
            .find(|fire| fire.display_id == *id)
            .expect("validated fire id should exist");
        matched.push(MatchedFire {
            id: id.clone(),
            source_atom: fire.source.atom_id.clone(),
            target_atom: fire.target.atom_id.clone(),
            severity: fire.severity.clone(),
            reason: fire.reason.clone(),
            validation_plan: validation.plan,
        });
    }

    let plan = all_matching_operation_plan(
        options,
        &context.branch,
        &context.open_dir,
        &matched,
        &rationale,
    );
    if !options.dry_run {
        for id in &fire_ids {
            run_extinguish(&ExtinguishOptions {
                path: options.path.clone(),
                fire_id: id.clone(),
                resolution: options.resolution.clone(),
                rationale: rationale.clone(),
                evidence: options.evidence.clone(),
                evidence_refs: options.evidence_refs.clone(),
                refresh: options.refresh,
                dry_run: false,
                json_output: false,
                lock: options.lock,
                idempotency_key: None,
                edit_rationale: false,
            })?;
        }
    }
    Ok(AllMatchingExtinguishResult {
        item_count: matched.len(),
        plan,
    })
}

pub(super) fn print_all_matching_extinguish_result(
    options: &AllMatchingExtinguishOptions,
    result: &AllMatchingExtinguishResult,
) {
    if options.dry_run {
        println!(
            "extinguish all-matching dry-run: {} fires would be processed",
            result.item_count
        );
    } else {
        println!("extinguished {} matching fires", result.item_count);
    }
}

pub(super) fn prepare_extinguish_options(
    mut options: ExtinguishOptions,
) -> Result<ExtinguishOptions, CliError> {
    if options.edit_rationale {
        options.rationale = edit_rationale_with_configured_editor(&options.rationale)?;
        options.edit_rationale = false;
    }
    Ok(options)
}

pub(crate) fn resolve_rationale_with_editor(
    initial_text: &str,
    editor: &Path,
) -> Result<String, CliError> {
    let path = env::temp_dir().join(format!(
        "codefire-rationale-{}-{}.txt",
        std::process::id(),
        unique_suffix()
    ));
    fs::write(&path, initial_text)?;
    let status = Command::new(editor).arg(&path).status()?;
    if !status.success() {
        return Err(CliError::Usage(format!(
            "editor exited with status: {status}"
        )));
    }
    let edited = fs::read_to_string(&path)?.trim().to_string();
    let _ = fs::remove_file(&path);
    if edited.is_empty() {
        return Err(CliError::Usage(
            "edited rationale must not be empty".to_string(),
        ));
    }
    Ok(edited)
}

fn edit_rationale_with_configured_editor(initial_text: &str) -> Result<String, CliError> {
    let editor = env::var_os("CODEFIRE_EDITOR")
        .or_else(|| env::var_os("EDITOR"))
        .ok_or_else(|| {
            CliError::Usage(
                "--edit-rationale requires CODEFIRE_EDITOR or EDITOR to be set".to_string(),
            )
        })?;
    resolve_rationale_with_editor(initial_text, Path::new(&editor))
}

fn recent_evidence_candidates(
    repo_root: &Path,
    limit: usize,
) -> Result<Vec<EvidenceCandidate>, CliError> {
    let evidence_dir = repo_root.join(".codefire").join("objects").join("evidence");
    if !evidence_dir.exists() {
        return Ok(Vec::new());
    }
    let mut candidates = Vec::new();
    for entry in fs::read_dir(evidence_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let record = read_json(&path)?;
        let payload = &record["payload"];
        if payload.get("type").and_then(Value::as_str) != Some("evidence") {
            continue;
        }
        let modified_at_unix = entry
            .metadata()?
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        candidates.push(EvidenceCandidate {
            id: record["object_id"].as_str().unwrap_or_default().to_string(),
            created_at: payload["created_at"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            label: payload["label"].as_str().map(str::to_string),
            artifact_ref: payload["artifact_ref"].as_str().map(str::to_string),
            command: payload.get("command").cloned(),
            modified_at_unix,
        });
    }
    candidates.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.modified_at_unix.cmp(&left.modified_at_unix))
            .then_with(|| left.id.cmp(&right.id))
    });
    candidates.truncate(limit);
    Ok(candidates)
}

fn evidence_candidates_json(candidates: &[EvidenceCandidate]) -> Vec<Value> {
    candidates
        .iter()
        .map(|candidate| {
            json!({
                "id": &candidate.id,
                "created_at": &candidate.created_at,
                "label": &candidate.label,
                "artifact_ref": &candidate.artifact_ref,
                "command": &candidate.command,
            })
        })
        .collect()
}

fn parse_source_target_query(query: &str) -> Result<(String, String), CliError> {
    let (source, target) = query.split_once("->").ok_or_else(|| {
        CliError::Usage(
            "--all-matching requires a query shaped as \"SOURCE -> TARGET\"".to_string(),
        )
    })?;
    let source = source.trim();
    let target = target.trim();
    if source.is_empty() || target.is_empty() {
        return Err(CliError::Usage(
            "--all-matching source and target must be non-empty".to_string(),
        ));
    }
    Ok((source.to_string(), target.to_string()))
}

fn all_matching_operation_plan(
    options: &AllMatchingExtinguishOptions,
    branch: &str,
    open_dir: &Path,
    fires: &[MatchedFire],
    rationale: &str,
) -> Value {
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": "extinguish-all-matching",
        "dry_run": options.dry_run,
        "would_apply": !options.dry_run,
        "branch": branch,
        "open_dir": open_dir,
        "query": &options.query,
        "resolution": {
            "resolution_type": &options.resolution,
            "refresh": options.refresh,
            "has_rationale": !rationale.is_empty(),
            "has_evidence": !options.evidence.is_empty(),
            "evidence_refs": &options.evidence_refs,
        },
        "fires": fires.iter().map(|fire| {
            json!({
                "display_id": &fire.id,
                "source_atom": &fire.source_atom,
                "target_atom": &fire.target_atom,
                "severity": &fire.severity,
                "reason": &fire.reason,
                "validation_plan": &fire.validation_plan,
            })
        }).collect::<Vec<_>>(),
        "next_actions": [
            {"kind": "verify", "command": "codefire-rs verify --details --json"},
        ],
    })
}

fn required_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

fn parse_positive_usize(value: &str) -> Result<usize, CliError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| CliError::Usage(format!("expected positive integer: {value}")))?;
    if parsed == 0 {
        return Err(CliError::Usage(
            "expected positive integer greater than zero".to_string(),
        ));
    }
    Ok(parsed)
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}
