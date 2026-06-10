mod batch;

#[cfg(test)]
pub(crate) use batch::FireBatchOptions;
pub(crate) use batch::{fire_batch_data_json, parse_fire_batch_args, run_fire_batch};

use super::{
    now_iso_utc, open_context, parse_lock_option, read_json, required_string, set_open_state,
    write_json_atomic, CliError, LockOptions, OpenContext, RepoLock,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const DEFAULT_MANUAL_FIRE_SEVERITY: &str = "required";
const MAX_FIRE_PLAN_ITEMS: usize = 20;
const FIRE_UID_HEX_LENGTH: usize = 32;
const FIRE_DISPLAY_HEX_LENGTH: usize = 12;

#[derive(Debug)]
pub(crate) struct FireOptions {
    pub(crate) path: PathBuf,
    pub(crate) source_atom: String,
    pub(crate) target_atom: String,
    pub(crate) reason: String,
    pub(crate) severity: String,
    pub(crate) dry_run: bool,
    pub(crate) full_output: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
}

#[derive(Debug, Clone)]
pub(crate) struct ManualFireSpec {
    pub(crate) source_atom: String,
    pub(crate) target_atom: String,
    pub(crate) reason: String,
    pub(crate) severity: String,
}

#[derive(Debug)]
pub(crate) struct FireResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) open_dir: PathBuf,
    pub(crate) branch: String,
    pub(crate) item_count: usize,
    pub(crate) dry_run: bool,
    pub(crate) plan: Value,
    pub(crate) fires: Vec<codefire_core::Fire>,
}

struct ManualFireContext {
    context: OpenContext,
    active_state_path: PathBuf,
    base_commit: String,
    atom_index: codefire_core::AtomIndex,
    trace_graph: codefire_core::TraceGraph,
    fires_path: PathBuf,
    existing_fires: Vec<codefire_core::Fire>,
}

pub(crate) fn parse_fire_args(args: &[String]) -> Result<FireOptions, CliError> {
    let mut path = None;
    let mut source_atom = None;
    let mut target_atom = None;
    let mut reason = None;
    let mut severity = DEFAULT_MANUAL_FIRE_SEVERITY.to_string();
    let mut dry_run = false;
    let mut full_output = false;
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
            "--to" | "--target" => {
                index += 1;
                target_atom = Some(required_arg(args, index, "--to")?.to_string());
            }
            "--reason" => {
                index += 1;
                reason = Some(required_arg(args, index, "--reason")?.to_string());
            }
            "--severity" => {
                index += 1;
                severity = required_arg(args, index, "--severity")?.to_string();
            }
            "--dry-run" => dry_run = true,
            "--full" => full_output = true,
            "--json" => json_output = true,
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            value if value.starts_with("--to=") => {
                target_atom = Some(value.trim_start_matches("--to=").to_string());
            }
            value if value.starts_with("--target=") => {
                target_atom = Some(value.trim_start_matches("--target=").to_string());
            }
            value if value.starts_with("--reason=") => {
                reason = Some(value.trim_start_matches("--reason=").to_string());
            }
            value if value.starts_with("--severity=") => {
                severity = value.trim_start_matches("--severity=").to_string();
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported fire option: {option}"
                )));
            }
            value if source_atom.is_none() => source_atom = Some(value.to_string()),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected fire argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(FireOptions {
        path: path.unwrap_or(std::env::current_dir()?),
        source_atom: source_atom.ok_or_else(single_fire_usage)?,
        target_atom: target_atom.ok_or_else(single_fire_usage)?,
        reason: reason.ok_or_else(single_fire_usage)?,
        severity,
        dry_run,
        full_output,
        json_output,
        lock,
    })
}

pub(crate) fn run_fire(options: &FireOptions) -> Result<FireResult, CliError> {
    let spec = ManualFireSpec {
        source_atom: options.source_atom.clone(),
        target_atom: options.target_atom.clone(),
        reason: options.reason.clone(),
        severity: options.severity.clone(),
    };
    run_manual_fire_specs(
        &options.path,
        &[spec],
        options.dry_run,
        options.full_output,
        &options.lock,
        "fire",
    )
}

pub(crate) fn run_manual_fire_specs(
    path: &Path,
    specs: &[ManualFireSpec],
    dry_run: bool,
    full_output: bool,
    lock: &LockOptions,
    command: &str,
) -> Result<FireResult, CliError> {
    if specs.is_empty() {
        return Err(CliError::Usage(
            "fire batch requires at least one fire".to_string(),
        ));
    }
    let _lock;
    let context;
    if dry_run {
        _lock = None;
        context = load_manual_fire_context(path)?;
    } else {
        let lock_context = open_context(path)?;
        _lock = Some(RepoLock::acquire_with_options(
            &lock_context.repo_root,
            lock,
        )?);
        context = load_manual_fire_context(path)?;
    }
    let new_fires = build_manual_fires(&context, specs)?;
    let plan = fire_operation_plan(command, dry_run, full_output, &context, specs, &new_fires);
    if !dry_run {
        let mut fires = context.existing_fires.clone();
        fires.extend(new_fires.iter().cloned());
        write_json_atomic(&context.fires_path, &serde_json::to_value(&fires)?)?;
        set_open_state(&context.context, &context.active_state_path, "open-burning")?;
    }
    Ok(FireResult {
        repo_root: context.context.repo_root,
        open_dir: context.context.open_dir,
        branch: context.context.branch,
        item_count: new_fires.len(),
        dry_run,
        plan,
        fires: if dry_run { Vec::new() } else { new_fires },
    })
}

pub(crate) fn fire_data_json(result: &FireResult) -> Value {
    json!({
        "type": "codefire_fire_result",
        "version": 1,
        "dry_run": result.dry_run,
        "item_count": result.item_count,
        "branch": &result.branch,
        "open_dir": &result.open_dir,
        "plan": &result.plan,
        "fires": result.fires.iter().map(fire_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn print_fire_result(result: &FireResult) {
    if result.dry_run {
        println!("fire dry-run: {} fires validated", result.item_count);
    } else {
        for fire in &result.fires {
            println!(
                "opened {}: {} -> {} {}",
                fire.display_id, fire.source.atom_id, fire.target.atom_id, fire.reason
            );
        }
    }
}

pub(crate) fn required_arg<'a>(
    args: &'a [String],
    index: usize,
    option: &str,
) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))
}

pub(crate) fn validate_manual_fire_spec(spec: ManualFireSpec) -> Result<ManualFireSpec, CliError> {
    if spec.source_atom.trim().is_empty() {
        return Err(CliError::Usage("fire source atom is required".to_string()));
    }
    if spec.target_atom.trim().is_empty() {
        return Err(CliError::Usage("fire target atom is required".to_string()));
    }
    if spec.reason.trim().is_empty() {
        return Err(CliError::Usage("fire reason is required".to_string()));
    }
    if spec.severity.trim().is_empty() {
        return Err(CliError::Usage("fire severity is required".to_string()));
    }
    Ok(spec)
}

pub(crate) fn default_manual_fire_severity() -> String {
    DEFAULT_MANUAL_FIRE_SEVERITY.to_string()
}

fn single_fire_usage() -> CliError {
    CliError::Usage(
        "usage: codefire fire <source-atom> --to <target-atom> --reason <text> [--path <open-dir>] [--severity required] [--dry-run] [--full] [--json]"
            .to_string(),
    )
}

fn load_manual_fire_context(path: &Path) -> Result<ManualFireContext, CliError> {
    let context = open_context(path)?;
    let active_state_path = PathBuf::from(required_string(
        &context.registry,
        &["open", "active_state_path"],
    )?);
    let base_commit = required_string(&context.registry, &["open", "current_base_commit"])?;
    let atom_index = codefire_core::build_atom_index(&context.open_dir)?;
    let trace_graph = codefire_core::current_trace_graph(&context.open_dir)?;
    let fires_path = active_state_path.join("fires.json");
    let existing_fires = if fires_path.exists() {
        serde_json::from_value(read_json(&fires_path)?)?
    } else {
        Vec::new()
    };
    Ok(ManualFireContext {
        context,
        active_state_path,
        base_commit,
        atom_index,
        trace_graph,
        fires_path,
        existing_fires,
    })
}

fn build_manual_fires(
    context: &ManualFireContext,
    specs: &[ManualFireSpec],
) -> Result<Vec<codefire_core::Fire>, CliError> {
    let atoms = context
        .atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect::<BTreeMap<_, _>>();
    let mut used_keys = context
        .existing_fires
        .iter()
        .filter(|fire| fire.status != "obsolete")
        .map(|fire| fire.key.clone())
        .collect::<BTreeSet<_>>();
    let mut batch_keys = BTreeSet::new();
    let now = now_iso_utc();
    let mut fires = Vec::with_capacity(specs.len());
    for spec in specs {
        let spec = validate_manual_fire_spec(spec.clone())?;
        if spec.source_atom == spec.target_atom {
            return Err(CliError::Usage(format!(
                "manual fire self-link is not allowed: {}",
                spec.source_atom
            )));
        }
        let source = atoms.get(spec.source_atom.as_str()).ok_or_else(|| {
            CliError::Usage(format!(
                "manual fire references unknown source atom: {}",
                spec.source_atom
            ))
        })?;
        let target = atoms.get(spec.target_atom.as_str()).ok_or_else(|| {
            CliError::Usage(format!(
                "manual fire references unknown target atom: {}",
                spec.target_atom
            ))
        })?;
        let trace_path =
            manual_trace_path(&context.trace_graph, &spec.source_atom, &spec.target_atom);
        let key = manual_fire_key(
            &spec.source_atom,
            &spec.target_atom,
            &spec.reason,
            &spec.severity,
            &trace_path,
        )?;
        if !batch_keys.insert(key.clone()) {
            return Err(CliError::Usage(format!(
                "duplicate fire batch item: {} -> {} {}",
                spec.source_atom, spec.target_atom, spec.reason
            )));
        }
        if !used_keys.insert(key.clone()) {
            return Err(CliError::Usage(format!(
                "manual fire already exists: {} -> {} {}",
                spec.source_atom, spec.target_atom, spec.reason
            )));
        }
        let fire_digest = fire_digest_hex(&key);
        fires.push(codefire_core::Fire {
            type_tag: "fire".to_string(),
            version: codefire_core::VERSION,
            fire_uid: fire_uid_from_digest(&fire_digest),
            display_id: fire_display_id_from_digest(&fire_digest),
            status: "open".to_string(),
            severity: spec.severity,
            source: codefire_core::FireAtomRef {
                atom_id: source.atom_id.clone(),
                content_hash_at_fire: Some(source.content_hash.clone()),
            },
            target: codefire_core::FireAtomRef {
                atom_id: target.atom_id.clone(),
                content_hash_at_fire: Some(target.content_hash.clone()),
            },
            reason: spec.reason,
            trace_path,
            created_by: "manual".to_string(),
            created_at: now.clone(),
            key,
            obsolete_at: None,
            resolution_uid: None,
        });
    }
    Ok(fires)
}

fn manual_trace_path(
    trace_graph: &codefire_core::TraceGraph,
    source: &str,
    target: &str,
) -> Vec<String> {
    if trace_graph.links.iter().any(|link| {
        (link.from == source && link.to == target) || (link.from == target && link.to == source)
    }) {
        vec![source.to_string(), target.to_string()]
    } else {
        Vec::new()
    }
}

fn manual_fire_key(
    source: &str,
    target: &str,
    reason: &str,
    severity: &str,
    trace_path: &[String],
) -> Result<String, CliError> {
    let trace_path_hash = digest_bytes(&serde_json::to_vec(trace_path)?);
    let value = json!({
        "source_atom_id": source,
        "target_atom_id": target,
        "reason": reason,
        "severity": severity,
        "trace_path_hash": trace_path_hash,
    });
    Ok(digest_bytes(&serde_json::to_vec(&value)?))
}

fn digest_bytes(bytes: &[u8]) -> String {
    codefire_util::sha256_prefixed(bytes)
}

fn fire_digest_hex(key: &str) -> String {
    codefire_util::sha256_hex(key.as_bytes())
}

fn fire_uid_from_digest(digest: &str) -> String {
    format!("fire_sha256_{}", &digest[..FIRE_UID_HEX_LENGTH])
}

fn fire_display_id_from_digest(digest: &str) -> String {
    format!(
        "FIRE-{}",
        digest[..FIRE_DISPLAY_HEX_LENGTH].to_ascii_uppercase()
    )
}

fn fire_operation_plan(
    command: &str,
    dry_run: bool,
    full_output: bool,
    context: &ManualFireContext,
    specs: &[ManualFireSpec],
    fires: &[codefire_core::Fire],
) -> Value {
    let item_limit = if full_output {
        fires.len()
    } else {
        MAX_FIRE_PLAN_ITEMS
    };
    let items = fires
        .iter()
        .take(item_limit)
        .map(fire_json)
        .collect::<Vec<_>>();
    let items_omitted = fires.len().saturating_sub(items.len());
    let open_dir = context.context.open_dir.to_string_lossy();
    json!({
        "type": "codefire_operation_plan",
        "version": 1,
        "command": command,
        "dry_run": dry_run,
        "would_apply": !dry_run,
        "requires_lock_revalidation": !dry_run,
        "repo_root": &context.context.repo_root,
        "branch": &context.context.branch,
        "open_dir": &context.context.open_dir,
        "current_base_commit": &context.base_commit,
        "item_count": fires.len(),
        "items": items,
        "sample_items": fires.iter().take(item_limit).map(fire_json).collect::<Vec<_>>(),
        "sample_limit": item_limit,
        "items_omitted": items_omitted,
        "truncated": items_omitted > 0,
        "operations": [
            {"kind": "validate_all_fire_items", "count": specs.len()},
            {"kind": "append_manual_fires", "path": &context.fires_path, "count": fires.len()},
            {"kind": "set_open_state", "state": "open-burning"},
        ],
        "next_actions": [
            {
                "kind": "verify",
                "command": format!("codefire verify --path {} --details --json", open_dir),
                "target": {"branch": &context.context.branch, "open_dir": &context.context.open_dir, "repo_root": &context.context.repo_root}
            },
            {
                "kind": "extinguish",
                "command": format!("codefire extinguish <fire-id> --path {} --resolution addressed --rationale <text>", open_dir),
                "target": {"branch": &context.context.branch, "open_dir": &context.context.open_dir, "repo_root": &context.context.repo_root}
            },
        ],
    })
}

fn fire_json(fire: &codefire_core::Fire) -> Value {
    json!({
        "display_id": &fire.display_id,
        "fire_uid": &fire.fire_uid,
        "source_atom": &fire.source.atom_id,
        "target_atom": &fire.target.atom_id,
        "reason": &fire.reason,
        "severity": &fire.severity,
        "status": &fire.status,
        "created_by": &fire.created_by,
        "trace_path": &fire.trace_path,
    })
}
