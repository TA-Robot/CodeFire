//! Branch merge: planning, applying, and reporting.

use crate::branch_lifecycle::refresh_open_clean_for_branch_operation;
use crate::cli_error::CliError;
use crate::cli_output::command_arg;
use crate::json_files::{read_active_json_optional, write_active_json_atomic};
use crate::manifest::safe_relative_manifest_path;
use crate::merge_patch_idempotency::{
    load_merge_idempotency, merge_idempotency_payload, save_merge_idempotency,
};
use crate::open_context::{open_context, validated_active_state_dir};
use crate::repo_lock::RepoLock;
use crate::scan_verify::{compute_scan, compute_scan_locked, load_base_atom_index};
use crate::secure_worktree::ExistingWorktree;
use crate::view::manifest_contents;
use crate::{
    find_repo_root, load_branch_record, opened_registry_path, read_json, required_string,
    scan_branch_state, scan_change_count, set_open_state, BinaryMergeConflict, MergeAction,
    MergeFileAction, MergeOptions, MergeResult, SemanticConflictCandidate,
};
use serde_json::{json, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};

pub(crate) fn merge_branch(start: &Path, options: &MergeOptions) -> Result<MergeResult, CliError> {
    let repo_root = find_repo_root(start)?;
    let lock = if options.dry_run {
        None
    } else {
        Some(RepoLock::acquire_with_options(&repo_root, &options.lock)?)
    };
    if options.source_branch == options.target_branch {
        return Err(branch_operation_error(
            &repo_root,
            Some(&options.source_branch),
            None,
            format!(
                "cannot merge branch '{}' into itself",
                options.source_branch
            ),
        ));
    }
    let idempotency_payload = merge_idempotency_payload(&repo_root, options)?;
    if !options.dry_run {
        if let Some(key) = options.idempotency_key.as_deref() {
            if let Some(result) = load_merge_idempotency(&repo_root, key, &idempotency_payload)? {
                return Ok(result);
            }
        }
    }
    let mut result = build_merge_result(&repo_root, options, lock.as_ref())?;
    if options.dry_run {
        return Ok(result);
    }
    apply_merge_result(
        &repo_root,
        &mut result,
        lock.as_ref().expect("live merge must hold repo lock"),
    )?;
    if let Some(key) = options.idempotency_key.as_deref() {
        save_merge_idempotency(&repo_root, key, &idempotency_payload, &result)?;
    }
    Ok(result)
}

pub(crate) fn build_merge_result(
    repo_root: &Path,
    options: &MergeOptions,
    lock: Option<&RepoLock>,
) -> Result<MergeResult, CliError> {
    if options.source_branch == options.target_branch {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.source_branch),
            None,
            format!(
                "cannot merge branch '{}' into itself",
                options.source_branch
            ),
        ));
    }
    let objects = repo_root.join(".codefire").join("objects");
    let source = load_branch_record(repo_root, &options.source_branch).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(&options.source_branch), None)
    })?;
    let source_head = required_string(&source, &["head"])?;
    ensure_sealed_source_state("merge", &options.source_branch, branch_state(&source)).map_err(
        |error| with_branch_operation_context(error, repo_root, Some(&options.source_branch), None),
    )?;
    let target = load_branch_record(repo_root, &options.target_branch).map_err(|error| {
        with_branch_operation_context(error, repo_root, Some(&options.target_branch), None)
    })?;
    let target_head = required_string(&target, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &source_head)?;
    codefire_store::validate_sealed_commit(&objects, &target_head)?;

    let registry_path = opened_registry_path(repo_root, &options.target_branch);
    if !registry_path.exists() {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.target_branch),
            None,
            format!("target branch must be open: {}", options.target_branch),
        ));
    }
    let registry = read_json(&registry_path)?;
    let target_dir = PathBuf::from(required_string(&registry, &["open", "path"])?);
    let target_state = if let Some(lock) = lock {
        refresh_open_clean_for_branch_operation(
            repo_root,
            &options.target_branch,
            &target_dir,
            lock,
        )?;
        required_string(&read_json(&registry_path)?, &["state", "last_known"])?
    } else {
        let scan = compute_scan(&target_dir)?;
        scan_branch_state(scan_change_count(&scan.scan), scan.scan.open_fires.len()).to_string()
    };
    if target_state != "open-clean" {
        return Err(branch_operation_error(
            repo_root,
            Some(&options.target_branch),
            Some(&target_dir),
            format!(
                "target branch has unsealed open work ({target_state}); it must be open-clean before merge"
            ),
        ));
    }
    let base = common_ancestor(&objects, &source_head, &target_head)?
        .ok_or_else(|| CliError::Usage("no common ancestor found".to_string()))?;
    let base_files = manifest_contents(&objects, &base)?;
    let source_files = manifest_contents(&objects, &source_head)?;
    let target_files = manifest_contents(&objects, &target_head)?;

    let paths = base_files
        .keys()
        .chain(source_files.keys())
        .chain(target_files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut conflicts = Vec::new();
    let mut binary_conflicts = Vec::new();
    let mut file_actions = Vec::new();
    for path in paths {
        let base_data = base_files.get(&path);
        let source_data = source_files.get(&path);
        let target_data = target_files.get(&path);
        if source_data == base_data {
            continue;
        }
        if target_data == base_data {
            file_actions.push(MergeFileAction {
                path,
                action: if source_data.is_some() {
                    MergeAction::WriteSource
                } else {
                    MergeAction::DeleteTarget
                },
                content: source_data.cloned(),
            });
            continue;
        }
        if source_data == target_data {
            continue;
        }
        let target_bytes = target_data.map(Vec::as_slice);
        let source_bytes = source_data.map(Vec::as_slice);
        // Both branches changed the file. Combine their line-level changes;
        // only a region changed differently on both sides is a conflict.
        match line_level_merge(base_data.map(Vec::as_slice), target_bytes, source_bytes) {
            Some(codefire_core::TextMerge::Clean(content)) => {
                file_actions.push(MergeFileAction {
                    path,
                    action: MergeAction::WriteMerged,
                    content: Some(content),
                });
                continue;
            }
            Some(codefire_core::TextMerge::Conflict { content, .. }) => {
                conflicts.push(path.clone());
                file_actions.push(MergeFileAction {
                    path,
                    action: MergeAction::WriteConflictMarkers,
                    content: Some(content),
                });
                continue;
            }
            // Deleted on one side, binary, or too far apart to compare:
            // keep the whole-file conflict.
            None => {}
        }
        conflicts.push(path.clone());
        if conflict_markers_are_safe(target_bytes, source_bytes) {
            let content = conflict_content(target_bytes, source_bytes);
            file_actions.push(MergeFileAction {
                path,
                action: MergeAction::WriteConflictMarkers,
                content: Some(content),
            });
        } else {
            let conflict = binary_merge_conflict(&path, target_bytes, source_bytes);
            if let (Some(side_path), Some(bytes)) = (&conflict.target_path, target_bytes) {
                file_actions.push(MergeFileAction {
                    path: side_path.clone(),
                    action: MergeAction::WriteConflictSide,
                    content: Some(bytes.to_vec()),
                });
            }
            if let (Some(side_path), Some(bytes)) = (&conflict.source_path, source_bytes) {
                file_actions.push(MergeFileAction {
                    path: side_path.clone(),
                    action: MergeAction::WriteConflictSide,
                    content: Some(bytes.to_vec()),
                });
            }
            binary_conflicts.push(conflict);
        }
    }

    let semantic_conflicts =
        semantic_conflict_candidates(&objects, &base, &source_head, &target_head)?;
    Ok(MergeResult {
        repo_root: repo_root.to_path_buf(),
        source_branch: options.source_branch.clone(),
        target_branch: options.target_branch.clone(),
        source_head,
        target_head,
        base,
        target_dir,
        file_actions,
        conflicts,
        binary_conflicts,
        semantic_conflicts,
        dry_run: options.dry_run,
        applied: false,
    })
}

pub(crate) fn apply_merge_result(
    repo_root: &Path,
    result: &mut MergeResult,
    lock: &RepoLock,
) -> Result<(), CliError> {
    let target = ExistingWorktree::bind(&result.target_dir)?;
    for action in &result.file_actions {
        write_merged_file(&target, &action.path, action.content.as_deref())?;
    }
    let registry_path = opened_registry_path(repo_root, &result.target_branch);
    let registry = read_json(&registry_path)?;
    let active_state = validated_active_state_dir(repo_root, &registry)?;
    let mut state = read_active_json_optional(&active_state, Path::new("state.json"))?
        .unwrap_or_else(|| json!({}));
    state["pending_merge_parent"] = Value::String(result.source_head.clone());
    state["pending_merge_base"] = Value::String(result.base.clone());
    state["merge_conflicts"] = Value::Array(
        result
            .conflicts
            .iter()
            .cloned()
            .map(Value::String)
            .collect(),
    );
    state["binary_merge_conflicts"] = Value::Array(
        result
            .binary_conflicts
            .iter()
            .map(binary_merge_conflict_json)
            .collect(),
    );
    state["semantic_conflict_candidates"] = Value::Array(
        result
            .semantic_conflicts
            .iter()
            .map(semantic_conflict_json)
            .collect(),
    );
    write_active_json_atomic(&active_state, Path::new("state.json"), &state)?;
    compute_scan_locked(&result.target_dir, lock)?;
    let context = open_context(&result.target_dir)?;
    set_open_state(&context, "open-burning")?;
    result.applied = true;
    Ok(())
}

pub(crate) fn semantic_conflict_candidates(
    objects: &Path,
    base_commit: &str,
    source_commit: &str,
    target_commit: &str,
) -> Result<Vec<SemanticConflictCandidate>, CliError> {
    let base_index = load_base_atom_index(objects, base_commit)?;
    let source_index = load_base_atom_index(objects, source_commit)?;
    let target_index = load_base_atom_index(objects, target_commit)?;
    let source_changed = codefire_core::changed_atoms(&source_index, &base_index)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let target_changed = codefire_core::changed_atoms(&target_index, &base_index)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let base_atoms = atom_by_id(&base_index);
    let source_atoms = atom_by_id(&source_index);
    let target_atoms = atom_by_id(&target_index);
    let mut candidates = Vec::new();
    for atom_id in source_changed.intersection(&target_changed) {
        let source_atom = source_atoms.get(atom_id.as_str()).copied();
        let target_atom = target_atoms.get(atom_id.as_str()).copied();
        let source_hash = source_atom.map(|atom| atom.content_hash.clone());
        let target_hash = target_atom.map(|atom| atom.content_hash.clone());
        if source_hash == target_hash {
            continue;
        }
        candidates.push(SemanticConflictCandidate {
            atom_id: atom_id.clone(),
            base_hash: base_atoms
                .get(atom_id.as_str())
                .map(|atom| atom.content_hash.clone()),
            source_hash,
            target_hash,
            source_path: source_atom.map(|atom| atom.artifact_path.clone()),
            target_path: target_atom.map(|atom| atom.artifact_path.clone()),
        });
    }
    candidates.sort_by(|left, right| left.atom_id.cmp(&right.atom_id));
    Ok(candidates)
}

pub(crate) fn atom_by_id(index: &codefire_core::AtomIndex) -> BTreeMap<&str, &codefire_core::Atom> {
    index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect()
}

pub(crate) fn ensure_sealed_source_state(
    command: &str,
    branch: &str,
    state: &str,
) -> Result<(), CliError> {
    match state {
        "closed" | "open-clean" => Ok(()),
        "open-burning" => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is open-burning"
        ))),
        "open-consistent" => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is open-consistent; commit or close the source branch first"
        ))),
        other => Err(CliError::Usage(format!(
            "cannot {command} from branch '{branch}' while it is in unsupported state '{other}'"
        ))),
    }
}

pub(crate) fn branch_operation_error(
    repo_root: &Path,
    branch: Option<&str>,
    path: Option<&Path>,
    message: impl Into<String>,
) -> CliError {
    CliError::BranchOperation {
        message: message.into(),
        repo_root: repo_root.to_path_buf(),
        branch: branch.map(str::to_string),
        path: path.map(Path::to_path_buf),
    }
}

pub(crate) fn with_branch_operation_context(
    error: CliError,
    repo_root: &Path,
    branch: Option<&str>,
    path: Option<&Path>,
) -> CliError {
    match error {
        CliError::Usage(message) => branch_operation_error(repo_root, branch, path, message),
        other => other,
    }
}

pub(crate) fn branch_state(value: &Value) -> &str {
    value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("closed")
}

pub(crate) fn render_merge_dry_run(result: &MergeResult) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "merge dry-run {} into {}; target unchanged\n",
        result.source_branch, result.target_branch
    ));
    output.push_str(&format!("  base: {}\n", result.base));
    output.push_str(&format!("  source: {}\n", result.source_head));
    output.push_str(&format!("  target: {}\n", result.target_head));
    output.push_str(&format!("  target dir: {}\n", result.target_dir.display()));
    output.push_str(&format!(
        "  file actions: {} conflicts: {} binary conflicts: {} semantic candidates: {}\n",
        result.file_actions.len(),
        result.conflicts.len(),
        result.binary_conflicts.len(),
        result.semantic_conflicts.len()
    ));
    if result.file_actions.is_empty() {
        output.push_str("  no file changes predicted\n");
    } else {
        for action in &result.file_actions {
            output.push_str(&format!("  {}: {}\n", action.action.as_str(), action.path));
        }
    }
    if !result.binary_conflicts.is_empty() {
        output.push_str("Binary conflicts:\n");
        for conflict in &result.binary_conflicts {
            output.push_str(&format!(
                "  {} target={} source={}\n",
                conflict.path,
                conflict.target_path.as_deref().unwrap_or("(deleted)"),
                conflict.source_path.as_deref().unwrap_or("(deleted)")
            ));
        }
    }
    if !result.semantic_conflicts.is_empty() {
        output.push_str("Semantic conflict candidates:\n");
        for candidate in &result.semantic_conflicts {
            output.push_str(&format!(
                "  {} source={} target={}\n",
                candidate.atom_id,
                candidate.source_hash.as_deref().unwrap_or("(missing)"),
                candidate.target_hash.as_deref().unwrap_or("(missing)")
            ));
        }
    }
    output
}

#[cfg(test)]
pub(crate) fn render_merge_result_json(result: &MergeResult) -> Result<String, CliError> {
    Ok(serde_json::to_string_pretty(&merge_result_json_value(
        result,
    ))?)
}

pub(crate) fn merge_result_json_value(result: &MergeResult) -> Value {
    json!({
        "type": "codefire_merge_plan",
        "version": 1,
        "repo": &result.repo_root,
        "dry_run": result.dry_run,
        "applied": result.applied,
        "source": {
            "branch": &result.source_branch,
            "commit": &result.source_head,
        },
        "target": {
            "branch": &result.target_branch,
            "commit": &result.target_head,
            "open_dir": result.target_dir,
        },
        "base": {"commit": &result.base},
        "file_actions": result.file_actions.iter().map(merge_file_action_json).collect::<Vec<_>>(),
        "conflicts": &result.conflicts,
        "binary_conflicts": result.binary_conflicts.iter().map(binary_merge_conflict_json).collect::<Vec<_>>(),
        "semantic_conflicts": result.semantic_conflicts.iter().map(semantic_conflict_json).collect::<Vec<_>>(),
        "next_actions": merge_next_actions(result),
    })
}

pub(crate) fn merge_file_action_json(action: &MergeFileAction) -> Value {
    json!({
        "path": &action.path,
        "action": action.action.as_str(),
        "bytes": action.content.as_ref().map(Vec::len),
    })
}

pub(crate) fn binary_merge_conflict_json(conflict: &BinaryMergeConflict) -> Value {
    json!({
        "path": &conflict.path,
        "target_path": &conflict.target_path,
        "source_path": &conflict.source_path,
        "target_bytes": conflict.target_bytes,
        "source_bytes": conflict.source_bytes,
    })
}

pub(crate) fn semantic_conflict_json(candidate: &SemanticConflictCandidate) -> Value {
    json!({
        "atom_id": &candidate.atom_id,
        "base_hash": &candidate.base_hash,
        "source_hash": &candidate.source_hash,
        "target_hash": &candidate.target_hash,
        "source_path": &candidate.source_path,
        "target_path": &candidate.target_path,
    })
}

pub(crate) fn merge_next_actions(result: &MergeResult) -> Vec<Value> {
    let mut actions = Vec::new();
    if result.dry_run && result.conflicts.is_empty() && result.semantic_conflicts.is_empty() {
        actions.push(json!({
            "kind": "apply_merge",
            "command": format!(
                "codefire merge {} --into {}",
                command_arg(&result.source_branch),
                command_arg(&result.target_branch)
            ),
            "reason": "apply the predicted merge to the open-clean target branch",
            "source_branch": &result.source_branch,
            "target_branch": &result.target_branch,
            "target": {
                "source_branch": &result.source_branch,
                "target_branch": &result.target_branch,
                "source_commit": &result.source_head,
                "target_commit": &result.target_head,
                "target_open_dir": &result.target_dir,
            },
            "hint": format!(
                "run codefire merge {} --into {}",
                command_arg(&result.source_branch),
                command_arg(&result.target_branch)
            ),
        }));
    }
    for conflict in &result.conflicts {
        let binary = result
            .binary_conflicts
            .iter()
            .find(|item| item.path == *conflict);
        actions.push(json!({
            "kind": "resolve_file_conflict",
            "path": conflict,
            "binary": binary.is_some(),
            "target_path": binary.and_then(|item| item.target_path.as_ref()),
            "source_path": binary.and_then(|item| item.source_path.as_ref()),
            "hint": if binary.is_some() {
                format!("merge saved binary conflict sides for {conflict}; choose or reconstruct the target file before commit")
            } else {
                format!("merge writes conflict markers for {conflict}; resolve them before commit")
            },
        }));
    }
    for candidate in &result.semantic_conflicts {
        actions.push(json!({
            "kind": "review_semantic_conflict",
            "atom_id": &candidate.atom_id,
            "source_hash": &candidate.source_hash,
            "target_hash": &candidate.target_hash,
            "hint": format!("review concurrent semantic changes to {}", candidate.atom_id),
        }));
    }
    actions
}

pub(crate) fn common_ancestor(
    objects: &Path,
    left: &str,
    right: &str,
) -> Result<Option<String>, CliError> {
    let left_distances = ancestor_distances(objects, left)?;
    nearest_common_ancestor_from_right(objects, right, &left_distances)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AncestorInfo {
    pub(crate) distance: usize,
    pub(crate) generation: Option<u64>,
}

pub(crate) fn ancestor_distances(
    objects: &Path,
    commit_id: &str,
) -> Result<HashMap<String, AncestorInfo>, CliError> {
    let mut distances = HashMap::new();
    let mut queue = VecDeque::from([(commit_id.to_string(), 0usize)]);
    while let Some((current, distance)) = queue.pop_front() {
        if distances
            .get(&current)
            .is_some_and(|known: &AncestorInfo| known.distance <= distance)
        {
            continue;
        }
        let commit = commit_info(objects, &current)?;
        distances.insert(
            current,
            AncestorInfo {
                distance,
                generation: commit.generation,
            },
        );
        for parent in commit.parents {
            queue.push_back((parent, distance + 1));
        }
    }
    Ok(distances)
}

pub(crate) fn nearest_common_ancestor_from_right(
    objects: &Path,
    right: &str,
    left_distances: &HashMap<String, AncestorInfo>,
) -> Result<Option<String>, CliError> {
    let mut queue = VecDeque::from([(right.to_string(), 0usize)]);
    let mut seen = HashMap::<String, usize>::new();
    let mut best: Option<(usize, usize, Reverse<u64>, String)> = None;
    while let Some((current, right_distance)) = queue.pop_front() {
        if best
            .as_ref()
            .is_some_and(|(best_sum, _, _, _)| right_distance > *best_sum)
        {
            break;
        }
        if seen
            .get(&current)
            .is_some_and(|known| *known <= right_distance)
        {
            continue;
        }
        seen.insert(current.clone(), right_distance);
        let commit = commit_info(objects, &current)?;
        if let Some(left_info) = left_distances.get(&current) {
            let candidate = (
                left_info.distance + right_distance,
                left_info.distance.max(right_distance),
                Reverse(commit.generation.or(left_info.generation).unwrap_or(0)),
                current.clone(),
            );
            if best.as_ref().map_or(true, |best| candidate < *best) {
                best = Some(candidate);
            }
        }
        for parent in commit.parents {
            queue.push_back((parent, right_distance + 1));
        }
    }
    Ok(best.map(|(_, _, _, commit)| commit))
}

#[derive(Debug)]
pub(crate) struct CommitInfo {
    pub(crate) parents: Vec<String>,
    pub(crate) generation: Option<u64>,
}

pub(crate) fn commit_info(objects: &Path, commit_id: &str) -> Result<CommitInfo, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .map(|parents| {
            parents
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let generation = commit.get("generation").and_then(Value::as_u64);
    Ok(CommitInfo {
        parents,
        generation,
    })
}

pub(crate) fn write_merged_file(
    worktree: &ExistingWorktree,
    rel_path: &str,
    data: Option<&[u8]>,
) -> Result<(), CliError> {
    let path = safe_relative_manifest_path(rel_path)?;
    if let Some(data) = data {
        worktree.write_file_atomic(&path, data)?;
    } else {
        worktree.delete_file(&path)?;
    }
    Ok(())
}

/// What to do in the target directory after a merge wrote its result.
pub(crate) fn merge_next_steps(result: &MergeResult) -> String {
    let target = result.target_dir.display();
    let mut steps = format!("Next, in {target}:\n");
    if !result.conflicts.is_empty() {
        steps.push_str(
            "  1. edit each conflicted file: keep the lines you want and delete the\n\
             \x20    `<<<<<<< target`, `=======` and `>>>>>>> source` marker lines\n",
        );
        if !result.binary_conflicts.is_empty() {
            steps.push_str(
                "     binary files: both versions are under .codefire-conflicts/; put the one\n\
                 \x20    you want at the original path and delete that directory\n",
            );
        }
    }
    let needs_continue = !result.conflicts.is_empty() || !result.semantic_conflicts.is_empty();
    if needs_continue {
        steps.push_str(&format!(
            "  {}. codefire merge --continue    lists what is still unresolved; when only\n\
             \x20    atoms changed on both branches remain, review them and add\n\
             \x20    --accept-semantic-conflicts to record that you did\n",
            if result.conflicts.is_empty() { 1 } else { 2 }
        ));
    }
    steps.push_str(
        "  then: codefire scan, extinguish the merge fires, codefire verify, codefire commit -m <message>\n",
    );
    steps
}

pub(crate) fn line_level_merge(
    base: Option<&[u8]>,
    target: Option<&[u8]>,
    source: Option<&[u8]>,
) -> Option<codefire_core::TextMerge> {
    let (base, target, source) = (base?, target?, source?);
    if ![base, target, source]
        .into_iter()
        .all(is_text_conflict_side)
    {
        return None;
    }
    codefire_core::merge_text_three_way(base, target, source)
}

pub(crate) fn conflict_content(target_data: Option<&[u8]>, source_data: Option<&[u8]>) -> Vec<u8> {
    let target = conflict_text(target_data);
    let source = conflict_text(source_data);
    format!("<<<<<<< target\n{target}=======\n{source}>>>>>>> source\n").into_bytes()
}

pub(crate) fn conflict_markers_are_safe(
    target_data: Option<&[u8]>,
    source_data: Option<&[u8]>,
) -> bool {
    let target_safe = target_data.map_or(true, is_text_conflict_side);
    let source_safe = source_data.map_or(true, is_text_conflict_side);
    target_safe && source_safe
}

pub(crate) fn is_text_conflict_side(data: &[u8]) -> bool {
    !data.contains(&0) && std::str::from_utf8(data).is_ok()
}

pub(crate) fn binary_merge_conflict(
    path: &str,
    target_data: Option<&[u8]>,
    source_data: Option<&[u8]>,
) -> BinaryMergeConflict {
    BinaryMergeConflict {
        path: path.to_string(),
        target_path: target_data.map(|_| conflict_side_path(path, "target")),
        source_path: source_data.map(|_| conflict_side_path(path, "source")),
        target_bytes: target_data.map(<[u8]>::len),
        source_bytes: source_data.map(<[u8]>::len),
    }
}

pub(crate) fn conflict_side_path(path: &str, side: &str) -> String {
    format!(".codefire-conflicts/{path}/{side}")
}

pub(crate) fn conflict_text(data: Option<&[u8]>) -> String {
    data.map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(|| "<deleted>\n".to_string())
}
