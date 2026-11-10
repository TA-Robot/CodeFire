use super::{
    acquire_locked_open_context, command_arg, compute_scan_locked, invalidate_active_verification,
    now_iso_utc, open_context, parse_lock_option, read_active_json_required,
    safe_relative_manifest_path, write_active_json_atomic, CliError, ExistingWorktree, ExitCode,
    LockOptions, OpenContext,
};
use codefire_core::{MergeConflictKind, UnresolvedMergeConflict};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};

const CONFLICT_FILE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;

#[derive(Debug)]
pub(crate) struct MergeContinueOptions {
    pub(crate) path: PathBuf,
    pub(crate) accept_semantic_conflicts: bool,
    pub(crate) dry_run: bool,
    pub(crate) json_output: bool,
    pub(crate) lock: LockOptions,
}

#[derive(Debug)]
pub(crate) struct MergeContinueResult {
    pub(crate) repo_root: PathBuf,
    pub(crate) open_dir: PathBuf,
    pub(crate) branch: String,
    pub(crate) dry_run: bool,
    pub(crate) blocked: bool,
    pub(crate) applied: bool,
    pub(crate) already_resolved: bool,
    pub(crate) conflicts: Vec<UnresolvedMergeConflict>,
    blockers: Vec<MergeContinueBlocker>,
}

impl MergeContinueResult {
    pub(crate) fn exit_code(&self) -> ExitCode {
        if self.blocked {
            ExitCode::MergeConflict
        } else {
            ExitCode::Success
        }
    }
}

#[derive(Debug, Clone)]
struct MergeContinueBlocker {
    kind: MergeConflictKind,
    subject: String,
    reason: &'static str,
    path: Option<String>,
}

#[derive(Debug)]
struct ActiveMergeState {
    text_conflicts: Vec<String>,
    binary_conflicts: Vec<ActiveBinaryConflict>,
    semantic_conflicts: Vec<String>,
}

#[derive(Debug)]
struct ActiveBinaryConflict {
    path: String,
    side_paths: Vec<String>,
}

pub(crate) fn parse_merge_continue_args(args: &[String]) -> Result<MergeContinueOptions, CliError> {
    if args.first().map(String::as_str) != Some("--continue") {
        return Err(CliError::Usage(merge_continue_usage().to_string()));
    }
    let mut path = None;
    let mut accept_semantic_conflicts = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut lock = LockOptions::default();
    let mut index = 1usize;
    while index < args.len() {
        if parse_lock_option(args, &mut index, &mut lock)? {
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            "--accept-semantic-conflicts" => accept_semantic_conflicts = true,
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    CliError::Usage("--path requires a value".to_string())
                })?));
            }
            value if value.starts_with("--path=") => {
                path = Some(PathBuf::from(value.trim_start_matches("--path=")));
            }
            option if option.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported merge --continue option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected merge --continue argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(MergeContinueOptions {
        path: path.unwrap_or(env::current_dir()?),
        accept_semantic_conflicts,
        dry_run,
        json_output,
        lock,
    })
}

pub(crate) fn run_merge_continue(
    options: &MergeContinueOptions,
) -> Result<MergeContinueResult, CliError> {
    if options.dry_run {
        let context = open_context(&options.path)?;
        return inspect_or_apply_merge_continue(options, &context, None);
    }
    let (lock, context) = acquire_locked_open_context(&options.path, &options.lock)?;
    inspect_or_apply_merge_continue(options, &context, Some(&lock))
}

fn inspect_or_apply_merge_continue(
    options: &MergeContinueOptions,
    context: &OpenContext,
    lock: Option<&super::RepoLock>,
) -> Result<MergeContinueResult, CliError> {
    let mut state = read_active_json_required(&context.active_state, Path::new("state.json"))?;
    let merge_state = parse_active_merge_state(&state)?.ok_or_else(|| {
        CliError::Usage("merge --continue requires a pending local merge".to_string())
    })?;
    let conflicts = merge_state.unresolved_conflicts();
    let already_resolved = conflicts.is_empty();
    let worktree = ExistingWorktree::bind(&context.open_dir)?;
    let blockers =
        inspect_resolution_blockers(&worktree, &merge_state, options.accept_semantic_conflicts)?;
    worktree.ensure_path_identity()?;
    let blocked = !blockers.is_empty();
    let mut result = MergeContinueResult {
        repo_root: context.repo_root.clone(),
        open_dir: context.open_dir.clone(),
        branch: context.branch.clone(),
        dry_run: options.dry_run,
        blocked,
        applied: false,
        already_resolved,
        conflicts,
        blockers,
    };
    if options.dry_run || blocked || already_resolved {
        return Ok(result);
    }
    let lock = lock.expect("live merge --continue must hold the repository lock");
    lock.ensure_repo_root(&context.repo_root)?;
    let final_blockers =
        inspect_resolution_blockers(&worktree, &merge_state, options.accept_semantic_conflicts)?;
    if !final_blockers.is_empty() {
        result.blocked = true;
        result.blockers = final_blockers;
        return Ok(result);
    }
    worktree.ensure_path_identity()?;
    invalidate_active_verification(&context.active_state)?;
    state["merge_resolution"] = json!({
        "type": "merge_resolution",
        "version": 1,
        "resolved_at": now_iso_utc(),
        "conflicts": &result.conflicts,
        "semantic_conflicts_accepted": options.accept_semantic_conflicts,
    });
    state["merge_conflicts"] = Value::Array(Vec::new());
    state["binary_merge_conflicts"] = Value::Array(Vec::new());
    state["semantic_conflict_candidates"] = Value::Array(Vec::new());
    write_active_json_atomic(&context.active_state, Path::new("state.json"), &state)?;
    compute_scan_locked(&context.open_dir, lock)?;
    result.applied = true;
    Ok(result)
}

pub(crate) fn unresolved_merge_conflicts_from_state(
    state: &Value,
) -> Result<Vec<UnresolvedMergeConflict>, CliError> {
    Ok(parse_active_merge_state(state)?
        .map(|state| state.unresolved_conflicts())
        .unwrap_or_default())
}

fn parse_active_merge_state(state: &Value) -> Result<Option<ActiveMergeState>, CliError> {
    let pending_parent = match state.get("pending_merge_parent") {
        None => None,
        Some(Value::String(parent)) if !parent.trim().is_empty() => Some(parent),
        Some(_) => {
            return Err(CliError::InvalidRepository(
                "active state pending_merge_parent must be a non-empty string".to_string(),
            ));
        }
    };
    if let Some(base) = state.get("pending_merge_base") {
        if !base.is_string() {
            return Err(CliError::InvalidRepository(
                "active state pending_merge_base must be a string".to_string(),
            ));
        }
    }

    let mut merge_paths = Vec::new();
    let mut merge_path_set = BTreeSet::new();
    for value in optional_array(state, "merge_conflicts")? {
        let path = value
            .as_str()
            .filter(|path| !path.trim().is_empty())
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "active state merge_conflicts entries must be non-empty paths".to_string(),
                )
            })?;
        safe_relative_manifest_path(path)?;
        if !merge_path_set.insert(path.to_string()) {
            return Err(CliError::InvalidRepository(format!(
                "active state contains duplicate merge conflict path: {path}"
            )));
        }
        merge_paths.push(path.to_string());
    }

    let mut binary_by_path = BTreeMap::new();
    for value in optional_array(state, "binary_merge_conflicts")? {
        let record = value.as_object().ok_or_else(|| {
            CliError::InvalidRepository(
                "active state binary_merge_conflicts entries must be objects".to_string(),
            )
        })?;
        let path = record
            .get("path")
            .and_then(Value::as_str)
            .filter(|path| !path.trim().is_empty())
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "active binary merge conflict path must be a non-empty string".to_string(),
                )
            })?;
        safe_relative_manifest_path(path)?;
        if !merge_path_set.contains(path) {
            return Err(CliError::InvalidRepository(format!(
                "binary merge conflict path is absent from merge_conflicts: {path}"
            )));
        }
        let mut side_paths = Vec::new();
        for field in ["target_path", "source_path"] {
            match record.get(field) {
                None | Some(Value::Null) => {}
                Some(Value::String(side_path)) if !side_path.trim().is_empty() => {
                    safe_relative_manifest_path(side_path)?;
                    let side = field.trim_end_matches("_path");
                    let expected = format!(".codefire-conflicts/{path}/{side}");
                    if side_path != &expected {
                        return Err(CliError::InvalidRepository(format!(
                            "active binary merge conflict {field} does not match {path}: expected {expected}, found {side_path}"
                        )));
                    }
                    side_paths.push(side_path.clone());
                }
                Some(_) => {
                    return Err(CliError::InvalidRepository(format!(
                        "active binary merge conflict {field} must be a path or null"
                    )));
                }
            }
        }
        if side_paths.is_empty() {
            return Err(CliError::InvalidRepository(format!(
                "binary merge conflict has no recorded side: {path}"
            )));
        }
        if binary_by_path
            .insert(
                path.to_string(),
                ActiveBinaryConflict {
                    path: path.to_string(),
                    side_paths,
                },
            )
            .is_some()
        {
            return Err(CliError::InvalidRepository(format!(
                "active state contains duplicate binary merge conflict: {path}"
            )));
        }
    }

    let mut semantic_conflicts = Vec::new();
    let mut semantic_set = BTreeSet::new();
    for value in optional_array(state, "semantic_conflict_candidates")? {
        let atom_id = value
            .get("atom_id")
            .and_then(Value::as_str)
            .filter(|atom_id| !atom_id.trim().is_empty())
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "active semantic conflict candidate atom_id must be a non-empty string"
                        .to_string(),
                )
            })?;
        if !semantic_set.insert(atom_id.to_string()) {
            return Err(CliError::InvalidRepository(format!(
                "active state contains duplicate semantic conflict candidate: {atom_id}"
            )));
        }
        semantic_conflicts.push(atom_id.to_string());
    }

    let has_conflicts =
        !merge_paths.is_empty() || !binary_by_path.is_empty() || !semantic_conflicts.is_empty();
    if pending_parent.is_none() {
        if has_conflicts {
            return Err(CliError::InvalidRepository(
                "active merge conflict metadata exists without pending_merge_parent".to_string(),
            ));
        }
        return Ok(None);
    }
    let mut text_conflicts = Vec::new();
    for path in merge_paths {
        if !binary_by_path.contains_key(&path) {
            text_conflicts.push(path);
        }
    }
    Ok(Some(ActiveMergeState {
        text_conflicts,
        binary_conflicts: binary_by_path.into_values().collect(),
        semantic_conflicts,
    }))
}

impl ActiveMergeState {
    fn unresolved_conflicts(&self) -> Vec<UnresolvedMergeConflict> {
        let mut conflicts = Vec::with_capacity(
            self.text_conflicts.len() + self.binary_conflicts.len() + self.semantic_conflicts.len(),
        );
        conflicts.extend(
            self.text_conflicts
                .iter()
                .map(|path| UnresolvedMergeConflict {
                    kind: MergeConflictKind::Text,
                    subject: path.clone(),
                }),
        );
        conflicts.extend(
            self.binary_conflicts
                .iter()
                .map(|conflict| UnresolvedMergeConflict {
                    kind: MergeConflictKind::Binary,
                    subject: conflict.path.clone(),
                }),
        );
        conflicts.extend(
            self.semantic_conflicts
                .iter()
                .map(|atom_id| UnresolvedMergeConflict {
                    kind: MergeConflictKind::Semantic,
                    subject: atom_id.clone(),
                }),
        );
        conflicts
    }
}

fn optional_array<'a>(state: &'a Value, field: &str) -> Result<&'a [Value], CliError> {
    match state.get(field) {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(CliError::InvalidRepository(format!(
            "active state {field} must be a list"
        ))),
    }
}

fn inspect_resolution_blockers(
    worktree: &ExistingWorktree,
    state: &ActiveMergeState,
    accept_semantic_conflicts: bool,
) -> Result<Vec<MergeContinueBlocker>, CliError> {
    let mut blockers = Vec::new();
    for path in &state.text_conflicts {
        let relative = safe_relative_manifest_path(path)?;
        if worktree
            .read_file_optional_bounded(&relative, CONFLICT_FILE_BYTE_LIMIT)?
            .is_some_and(|contents| contains_generated_conflict_marker(&contents))
        {
            blockers.push(MergeContinueBlocker {
                kind: MergeConflictKind::Text,
                subject: path.clone(),
                reason: "conflict_markers_present",
                path: Some(path.clone()),
            });
        }
    }
    for conflict in &state.binary_conflicts {
        for side_path in &conflict.side_paths {
            let relative = safe_relative_manifest_path(side_path)?;
            if worktree.entry_exists_nofollow(&relative)? {
                blockers.push(MergeContinueBlocker {
                    kind: MergeConflictKind::Binary,
                    subject: conflict.path.clone(),
                    reason: "conflict_sidecar_present",
                    path: Some(side_path.clone()),
                });
            }
        }
    }
    if !accept_semantic_conflicts {
        blockers.extend(
            state
                .semantic_conflicts
                .iter()
                .map(|atom_id| MergeContinueBlocker {
                    kind: MergeConflictKind::Semantic,
                    subject: atom_id.clone(),
                    reason: "semantic_acknowledgement_required",
                    path: None,
                }),
        );
    }
    Ok(blockers)
}

fn contains_generated_conflict_marker(contents: &[u8]) -> bool {
    contents.split(|byte| *byte == b'\n').any(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        matches!(line, b"<<<<<<< target" | b">>>>>>> source")
    })
}

pub(crate) fn merge_continue_data_json(result: &MergeContinueResult) -> Value {
    json!({
        "type": "codefire_merge_continue_result",
        "version": 1,
        "dry_run": result.dry_run,
        "blocked": result.blocked,
        "applied": result.applied,
        "already_resolved": result.already_resolved,
        "branch": &result.branch,
        "open_dir": &result.open_dir,
        "conflicts": &result.conflicts,
        "blockers": result.blockers.iter().map(blocker_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn merge_continue_diagnostics_json(result: &MergeContinueResult) -> Vec<Value> {
    result
        .blockers
        .iter()
        .map(|blocker| {
            json!({
                "kind": "unresolved_merge_conflict",
                "severity": "error",
                "blocking": true,
                "conflict_kind": blocker.kind.as_str(),
                "subject": &blocker.subject,
                "reason": blocker.reason,
                "path": &blocker.path,
            })
        })
        .collect()
}

pub(crate) fn merge_continue_next_actions(result: &MergeContinueResult) -> Vec<Value> {
    if result.blocked {
        return vec![json!({
            "kind": "resolve_merge_conflicts",
            "command": format!(
                "codefire merge --continue --path {}{} --dry-run --json",
                command_arg(&result.open_dir.to_string_lossy()),
                if result.blockers.iter().any(|item| item.kind == MergeConflictKind::Semantic) {
                    " --accept-semantic-conflicts"
                } else {
                    ""
                }
            ),
            "reason": "resolve every reported text/binary conflict and explicitly accept semantic candidates before continuing",
            "target": {
                "path": &result.open_dir,
                "blockers": result.blockers.len(),
            },
        })];
    }
    if result.dry_run && !result.already_resolved {
        return vec![json!({
            "kind": "continue_merge",
            "command": format!(
                "codefire merge --continue --path {}{}",
                command_arg(&result.open_dir.to_string_lossy()),
                if result.conflicts.iter().any(|item| item.kind == MergeConflictKind::Semantic) {
                    " --accept-semantic-conflicts"
                } else {
                    ""
                }
            ),
            "reason": "record the validated merge resolution and refresh verification state",
            "target": {"path": &result.open_dir},
        })];
    }
    vec![json!({
        "kind": "verify",
        "command": format!(
            "codefire verify --path {} --details --json",
            command_arg(&result.open_dir.to_string_lossy())
        ),
        "reason": "verify the resolved merge before sealing its two-parent commit",
        "target": {"path": &result.open_dir},
    })]
}

pub(crate) fn render_merge_continue(result: &MergeContinueResult) -> String {
    if result.blocked {
        let mut output = format!(
            "merge continue blocked: {} unresolved item(s)\n",
            result.blockers.len()
        );
        for blocker in &result.blockers {
            output.push_str(&format!(
                "  {} {}: {}\n",
                blocker.kind.as_str(),
                blocker.subject,
                blocker.reason
            ));
        }
        let has = |reason: &str| result.blockers.iter().any(|item| item.reason == reason);
        if has("conflict_markers_present") {
            output.push_str(
                "conflict_markers_present: edit the file, keep the lines you want, and delete the \
                 `<<<<<<< target`, `=======` and `>>>>>>> source` marker lines\n",
            );
        }
        if has("conflict_sidecar_present") {
            output.push_str(
                "conflict_sidecar_present: put the version you want at the original path and \
                 delete its directory under .codefire-conflicts/\n",
            );
        }
        if has("semantic_acknowledgement_required") {
            output.push_str(
                "semantic_acknowledgement_required: both branches changed this atom; review the \
                 merged result (`codefire diff <target-branch> <source-branch> --atoms` shows what each \
                 branch changed), then rerun with --accept-semantic-conflicts to record that you did\n",
            );
        }
        return output;
    }
    if result.already_resolved {
        return "merge conflicts are already resolved; run verify before commit\n".to_string();
    }
    if result.dry_run {
        return format!(
            "merge continue dry-run: {} conflict item(s) are ready to resolve\n",
            result.conflicts.len()
        );
    }
    format!(
        "merge resolution recorded for {} conflict item(s); run verify before commit\n",
        result.conflicts.len()
    )
}

fn blocker_json(blocker: &MergeContinueBlocker) -> Value {
    json!({
        "kind": blocker.kind.as_str(),
        "subject": &blocker.subject,
        "reason": blocker.reason,
        "path": &blocker.path,
    })
}

fn merge_continue_usage() -> &'static str {
    "usage: codefire merge --continue [path|--path <open-dir>] [--accept-semantic-conflicts] [--dry-run] [--json] [--wait-lock] [--lock-timeout <duration>]"
}
