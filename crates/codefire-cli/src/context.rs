use crate::{
    load_base_atom_index, open_context, read_json, required_string, scan_branch_state, CliError,
    OpenContext,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_CONTEXT_LIMIT: usize = 100;
const MAX_CONTEXT_DEPTH: usize = 8;

#[derive(Debug)]
pub(crate) struct ContextOptions {
    pub(crate) path: PathBuf,
    pub(crate) selector: ContextSelector,
    pub(crate) depth: usize,
    pub(crate) limit: usize,
    pub(crate) json_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContextSelector {
    Atom(String),
    Fire(String),
    Changed,
    Branch,
}

pub(crate) struct ContextPack {
    pub(crate) repo_root: PathBuf,
    pub(crate) data: Value,
}

struct ContextSnapshot {
    open: OpenContext,
    scan: codefire_core::ScanResult,
    active_scan_path: PathBuf,
    scan_snapshot_source: &'static str,
    scan_fire_source: &'static str,
    preview_recomputed: bool,
}

struct Selection {
    atom_ids: BTreeSet<String>,
    truncated_atoms: bool,
}

pub(crate) fn parse_context_args(args: &[String]) -> Result<ContextOptions, CliError> {
    let mut path = None;
    let mut selector = None;
    let mut depth = 1usize;
    let mut limit = DEFAULT_CONTEXT_LIMIT;
    let mut json_output = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--branch" => set_selector(&mut selector, ContextSelector::Branch)?,
            "--changed" => set_selector(&mut selector, ContextSelector::Changed)?,
            "--atom" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--atom requires a value".to_string()))?;
                set_selector(&mut selector, ContextSelector::Atom(value.clone()))?;
            }
            "--fire" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--fire requires a value".to_string()))?;
                set_selector(&mut selector, ContextSelector::Fire(value.clone()))?;
            }
            "--depth" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--depth requires a value".to_string()))?;
                depth = value.parse::<usize>().map_err(|_| {
                    CliError::Usage("--depth must be a non-negative integer".to_string())
                })?;
            }
            "--limit" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--limit requires a value".to_string()))?;
                limit = value.parse::<usize>().map_err(|_| {
                    CliError::Usage("--limit must be a positive integer".to_string())
                })?;
                if limit == 0 {
                    return Err(CliError::Usage(
                        "--limit must be a positive integer".to_string(),
                    ));
                }
            }
            "--path" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--path requires a value".to_string()))?;
                path = Some(PathBuf::from(value));
            }
            value if value.starts_with("--") => {
                return Err(CliError::Usage(format!(
                    "unsupported context option: {value}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::Usage(format!(
                    "unexpected context argument: {value}"
                )));
            }
        }
        index += 1;
    }
    Ok(ContextOptions {
        path: path.unwrap_or(env::current_dir()?),
        selector: selector.unwrap_or(ContextSelector::Branch),
        depth: depth.min(MAX_CONTEXT_DEPTH),
        limit,
        json_output,
    })
}

pub(crate) fn build_context_pack(options: &ContextOptions) -> Result<ContextPack, CliError> {
    let snapshot = context_snapshot(&options.path)?;
    let data = context_data_json(&snapshot, &options.selector, options.depth, options.limit)?;
    Ok(ContextPack {
        repo_root: snapshot.open.repo_root,
        data,
    })
}

pub(crate) fn print_context_summary(data: &Value) {
    let selector = &data["selector"];
    println!(
        "Context: {} {}",
        selector["kind"].as_str().unwrap_or("branch"),
        selector["value"].as_str().unwrap_or("")
    );
    println!(
        "Branch: {}",
        data["branch"]["name"].as_str().unwrap_or("(unknown)")
    );
    println!(
        "Changed atoms: {}",
        data["scan"]["changed_atoms"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0)
    );
    println!(
        "Open fires: {}",
        data["scan"]["open_fires"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0)
    );
}

fn set_selector(
    selector: &mut Option<ContextSelector>,
    value: ContextSelector,
) -> Result<(), CliError> {
    if selector.is_some() {
        return Err(CliError::Usage(
            "context selector must be one of --branch, --changed, --atom, --fire".to_string(),
        ));
    }
    *selector = Some(value);
    Ok(())
}

fn context_snapshot(start: &Path) -> Result<ContextSnapshot, CliError> {
    let open = open_context(start)?;
    let objects = open.repo_root.join(".codefire").join("objects");
    let base_commit = required_string(&open.registry, &["open", "current_base_commit"])?;
    let active_state_path = PathBuf::from(required_string(
        &open.registry,
        &["open", "active_state_path"],
    )?);
    let active_scan_path = active_state_path.join("scan.json");
    if active_scan_path.exists() {
        let scan = serde_json::from_value(read_json(&active_scan_path)?)?;
        return Ok(ContextSnapshot {
            open,
            scan,
            active_scan_path,
            scan_snapshot_source: "active_scan",
            scan_fire_source: "active",
            preview_recomputed: false,
        });
    }

    let current = codefire_core::build_atom_index(&open.open_dir)?;
    let base_index = load_base_atom_index(&objects, &base_commit)?;
    let trace_graph = codefire_core::current_trace_graph(&open.open_dir)?;
    let fires_path = active_state_path.join("fires.json");
    let fires = if fires_path.exists() {
        serde_json::from_value(read_json(&fires_path)?)?
    } else {
        Vec::new()
    };
    let state_path = active_state_path.join("state.json");
    let state_data = if state_path.exists() {
        read_json(&state_path)?
    } else {
        json!({"state": "open-clean"})
    };
    let reason = if state_data.get("pending_merge_parent").is_some() {
        "merge_changed"
    } else {
        "atom_changed"
    };
    let (scan, _) = codefire_core::build_scan_result(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        reason,
        "preview-context",
    )?;
    Ok(ContextSnapshot {
        open,
        scan,
        active_scan_path,
        scan_snapshot_source: "preview_recomputed",
        scan_fire_source: "preview",
        preview_recomputed: true,
    })
}

fn context_data_json(
    snapshot: &ContextSnapshot,
    selector: &ContextSelector,
    depth: usize,
    limit: usize,
) -> Result<Value, CliError> {
    let selection = selected_atom_ids(&snapshot.scan, selector, depth, limit)?;
    let selected_atom_ids = selection.atom_ids;
    let all_trace_links = snapshot
        .scan
        .trace_graph
        .links
        .iter()
        .filter(|link| {
            selected_atom_ids.contains(&link.from) || selected_atom_ids.contains(&link.to)
        })
        .collect::<Vec<_>>();
    let all_fires = snapshot
        .scan
        .open_fires
        .iter()
        .filter(|fire| match selector {
            ContextSelector::Fire(value) => fire.display_id == *value || fire.fire_uid == *value,
            ContextSelector::Atom(_) => {
                selected_atom_ids.contains(&fire.source.atom_id)
                    || selected_atom_ids.contains(&fire.target.atom_id)
            }
            ContextSelector::Changed => {
                snapshot.scan.changed_atoms.contains(&fire.source.atom_id)
                    || snapshot.scan.changed_atoms.contains(&fire.target.atom_id)
            }
            ContextSelector::Branch => true,
        })
        .collect::<Vec<_>>();
    let atom_count_before_limit = selected_atom_ids.len();
    let selected_atoms = selected_atom_ids
        .iter()
        .filter_map(|atom_id| {
            atom_by_id(&snapshot.scan.atom_index)
                .get(atom_id.as_str())
                .copied()
        })
        .take(limit)
        .map(atom_json)
        .collect::<Vec<_>>();
    let returned_atom_ids = selected_atoms
        .iter()
        .filter_map(|atom| atom.get("atom_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let known_atom_ids = atom_by_id(&snapshot.scan.atom_index)
        .keys()
        .map(|atom_id| atom_id.to_string())
        .collect::<BTreeSet<_>>();
    let trace_links = all_trace_links
        .iter()
        .take(limit)
        .map(|link| trace_link_json(link))
        .collect::<Vec<_>>();
    let trace_link_endpoint_metadata = all_trace_links
        .iter()
        .take(limit)
        .map(|link| trace_link_endpoint_metadata_json(link, &returned_atom_ids, &known_atom_ids))
        .collect::<Vec<_>>();
    let trace_links_with_omitted_endpoints = all_trace_links
        .iter()
        .take(limit)
        .filter(|link| {
            endpoint_presence(&link.from, &returned_atom_ids, &known_atom_ids) == "omitted"
                || endpoint_presence(&link.to, &returned_atom_ids, &known_atom_ids) == "omitted"
        })
        .count();
    let trace_links_with_missing_endpoints = all_trace_links
        .iter()
        .take(limit)
        .filter(|link| {
            endpoint_presence(&link.from, &returned_atom_ids, &known_atom_ids) == "missing"
                || endpoint_presence(&link.to, &returned_atom_ids, &known_atom_ids) == "missing"
        })
        .count();
    let fires = all_fires
        .iter()
        .take(limit)
        .map(|fire| fire_json(fire))
        .collect::<Vec<_>>();
    let scan_changed_atoms = snapshot
        .scan
        .changed_atoms
        .iter()
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let scan_open_fires = snapshot
        .scan
        .open_fires
        .iter()
        .take(limit)
        .map(fire_json)
        .collect::<Vec<_>>();
    let atoms_truncated =
        selection.truncated_atoms || atom_count_before_limit > selected_atoms.len();
    let trace_links_truncated = all_trace_links.len() > trace_links.len();
    let fires_truncated = all_fires.len() > fires.len();
    let scan_changed_atoms_truncated = snapshot.scan.changed_atoms.len() > scan_changed_atoms.len();
    let scan_open_fires_truncated = snapshot.scan.open_fires.len() > scan_open_fires.len();
    Ok(json!({
        "type": "codefire_context_pack",
        "version": 1,
        "selector": selector_json(selector, depth),
        "limits": {
            "max_depth": MAX_CONTEXT_DEPTH,
            "limit": limit,
        },
        "truncated": {
            "atoms": atoms_truncated,
            "trace_links": trace_links_truncated,
            "fires": fires_truncated,
            "scan_changed_atoms": scan_changed_atoms_truncated,
            "scan_open_fires": scan_open_fires_truncated,
        },
        "summary": {
            "atoms": {
                "total": atom_count_before_limit,
                "returned": selected_atoms.len(),
                "omitted": atom_count_before_limit.saturating_sub(selected_atoms.len()),
            },
            "trace_links": {
                "total": all_trace_links.len(),
                "returned": trace_links.len(),
                "omitted": all_trace_links.len().saturating_sub(trace_links.len()),
                "with_omitted_endpoints": trace_links_with_omitted_endpoints,
                "with_missing_endpoints": trace_links_with_missing_endpoints,
            },
            "fires": {
                "total": all_fires.len(),
                "returned": fires.len(),
                "omitted": all_fires.len().saturating_sub(fires.len()),
            },
            "scan_changed_atoms": {
                "total": snapshot.scan.changed_atoms.len(),
                "returned": scan_changed_atoms.len(),
                "omitted": snapshot.scan.changed_atoms.len().saturating_sub(scan_changed_atoms.len()),
            },
            "scan_open_fires": {
                "total": snapshot.scan.open_fires.len(),
                "returned": scan_open_fires.len(),
                "omitted": snapshot.scan.open_fires.len().saturating_sub(scan_open_fires.len()),
            },
        },
        "branch": {
            "name": &snapshot.open.branch,
            "open_dir": snapshot.open.open_dir,
            "base_commit": &snapshot.scan.base_commit,
        },
        "scan": {
            "branch_state": scan_branch_state(snapshot.scan.changed_atoms.len(), snapshot.scan.open_fires.len()),
            "snapshot_source": snapshot.scan_snapshot_source,
            "active_scan_path": snapshot.active_scan_path,
            "preview_recomputed": snapshot.preview_recomputed,
            "fire_source": snapshot.scan_fire_source,
            "changed_atoms": scan_changed_atoms,
            "open_fires": scan_open_fires,
        },
        "atoms": selected_atoms,
        "trace_links": trace_links,
        "trace_link_endpoints": trace_link_endpoint_metadata,
        "fires": fires,
    }))
}

fn selected_atom_ids(
    scan: &codefire_core::ScanResult,
    selector: &ContextSelector,
    depth: usize,
    limit: usize,
) -> Result<Selection, CliError> {
    match selector {
        ContextSelector::Branch => Ok(selection(BTreeSet::new(), false)),
        ContextSelector::Changed => {
            let total = scan.changed_atoms.len();
            Ok(selection(
                scan.changed_atoms.iter().take(limit).cloned().collect(),
                total > limit,
            ))
        }
        ContextSelector::Atom(atom_id) => {
            if !atom_by_id(&scan.atom_index).contains_key(atom_id.as_str()) {
                return Err(CliError::Usage(format!("unknown atom: {atom_id}")));
            }
            Ok(atom_neighborhood(&scan.trace_graph, atom_id, depth, limit))
        }
        ContextSelector::Fire(value) => {
            let fire = scan
                .open_fires
                .iter()
                .find(|fire| fire.display_id == *value || fire.fire_uid == *value)
                .ok_or_else(|| CliError::Usage(format!("unknown fire: {value}")))?;
            let mut ids = BTreeSet::new();
            ids.insert(fire.source.atom_id.clone());
            ids.insert(fire.target.atom_id.clone());
            for atom_id in &fire.trace_path {
                ids.insert(atom_id.clone());
            }
            let total = ids.len();
            if total > limit {
                ids = ids.into_iter().take(limit).collect();
            }
            Ok(selection(ids, total > limit))
        }
    }
}

fn selection(atom_ids: BTreeSet<String>, truncated_atoms: bool) -> Selection {
    Selection {
        atom_ids,
        truncated_atoms,
    }
}

fn atom_neighborhood(
    trace_graph: &codefire_core::TraceGraph,
    start: &str,
    depth: usize,
    limit: usize,
) -> Selection {
    let adjacency = trace_adjacency(trace_graph);
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    let mut truncated = false;
    seen.insert(start.to_string());
    queue.push_back((start.to_string(), 0usize));
    while let Some((atom_id, distance)) = queue.pop_front() {
        if distance >= depth {
            continue;
        }
        if let Some(next) = adjacency.get(atom_id.as_str()) {
            for neighbor in next {
                if seen.insert(neighbor.clone()) {
                    if seen.len() > limit {
                        seen.remove(neighbor);
                        truncated = true;
                        continue;
                    }
                    queue.push_back((neighbor.clone(), distance + 1));
                }
            }
        }
    }
    selection(seen, truncated)
}

fn trace_adjacency(trace_graph: &codefire_core::TraceGraph) -> BTreeMap<&str, Vec<String>> {
    let mut adjacency: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for link in &trace_graph.links {
        adjacency
            .entry(link.from.as_str())
            .or_default()
            .push(link.to.clone());
        adjacency
            .entry(link.to.as_str())
            .or_default()
            .push(link.from.clone());
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort();
        neighbors.dedup();
    }
    adjacency
}

fn atom_by_id(index: &codefire_core::AtomIndex) -> BTreeMap<&str, &codefire_core::Atom> {
    index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect()
}

fn selector_json(selector: &ContextSelector, depth: usize) -> Value {
    match selector {
        ContextSelector::Atom(value) => json!({"kind": "atom", "value": value, "depth": depth}),
        ContextSelector::Fire(value) => json!({"kind": "fire", "value": value, "depth": depth}),
        ContextSelector::Changed => json!({"kind": "changed", "value": null, "depth": depth}),
        ContextSelector::Branch => json!({"kind": "branch", "value": null, "depth": depth}),
    }
}

fn atom_json(atom: &codefire_core::Atom) -> Value {
    json!({
        "atom_id": &atom.atom_id,
        "kind": &atom.kind,
        "artifact_path": &atom.artifact_path,
        "selector": &atom.selector,
        "content_hash": &atom.content_hash,
    })
}

fn trace_link_json(link: &codefire_core::TraceLink) -> Value {
    json!({
        "from": &link.from,
        "to": &link.to,
        "type": &link.link_type,
        "link_id": &link.link_id,
        "link_hash": &link.link_hash,
    })
}

fn trace_link_endpoint_metadata_json(
    link: &codefire_core::TraceLink,
    returned_atom_ids: &BTreeSet<String>,
    known_atom_ids: &BTreeSet<String>,
) -> Value {
    json!({
        "link_id": &link.link_id,
        "from": {
            "atom_id": &link.from,
            "presence": endpoint_presence(&link.from, returned_atom_ids, known_atom_ids),
        },
        "to": {
            "atom_id": &link.to,
            "presence": endpoint_presence(&link.to, returned_atom_ids, known_atom_ids),
        },
    })
}

fn endpoint_presence(
    atom_id: &str,
    returned_atom_ids: &BTreeSet<String>,
    known_atom_ids: &BTreeSet<String>,
) -> &'static str {
    if returned_atom_ids.contains(atom_id) {
        "returned"
    } else if known_atom_ids.contains(atom_id) {
        "omitted"
    } else {
        "missing"
    }
}

fn fire_json(fire: &codefire_core::Fire) -> Value {
    json!({
        "fire_uid": &fire.fire_uid,
        "display_id": &fire.display_id,
        "status": &fire.status,
        "severity": &fire.severity,
        "source": &fire.source,
        "target": &fire.target,
        "reason": &fire.reason,
        "trace_path": &fire.trace_path,
        "created_by": &fire.created_by,
        "created_at": &fire.created_at,
        "obsolete_at": &fire.obsolete_at,
        "resolution_uid": &fire.resolution_uid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_context_args_accepts_selectors_depth_path_and_json() {
        let args = vec![
            "--atom".to_string(),
            "REQ-session".to_string(),
            "--depth".to_string(),
            "99".to_string(),
            "--limit".to_string(),
            "7".to_string(),
            "--path".to_string(),
            "/tmp/open".to_string(),
            "--json".to_string(),
        ];
        let parsed = parse_context_args(&args).unwrap();
        assert_eq!(
            parsed.selector,
            ContextSelector::Atom("REQ-session".to_string())
        );
        assert_eq!(parsed.depth, MAX_CONTEXT_DEPTH);
        assert_eq!(parsed.limit, 7);
        assert_eq!(parsed.path, PathBuf::from("/tmp/open"));
        assert!(parsed.json_output);
    }

    #[test]
    fn atom_neighborhood_walks_trace_links_by_depth() {
        let trace_graph = codefire_core::TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: 1,
            links: vec![
                codefire_core::TraceLink {
                    from: "REQ-session".to_string(),
                    to: "DES-session".to_string(),
                    link_type: "refined_by".to_string(),
                    link_id: "REQ-session->DES-session:refined_by".to_string(),
                    link_hash: "h1".to_string(),
                },
                codefire_core::TraceLink {
                    from: "DES-session".to_string(),
                    to: "CODE-session".to_string(),
                    link_type: "implemented_by".to_string(),
                    link_id: "DES-session->CODE-session:implemented_by".to_string(),
                    link_hash: "h2".to_string(),
                },
            ],
        };

        assert_eq!(
            atom_neighborhood(&trace_graph, "REQ-session", 1, DEFAULT_CONTEXT_LIMIT).atom_ids,
            BTreeSet::from(["DES-session".to_string(), "REQ-session".to_string()])
        );
        assert_eq!(
            atom_neighborhood(&trace_graph, "REQ-session", 2, DEFAULT_CONTEXT_LIMIT).atom_ids,
            BTreeSet::from([
                "CODE-session".to_string(),
                "DES-session".to_string(),
                "REQ-session".to_string(),
            ])
        );
    }

    #[test]
    fn atom_neighborhood_reports_truncation_when_limit_is_reached() {
        let trace_graph = codefire_core::TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: 1,
            links: vec![
                codefire_core::TraceLink {
                    from: "REQ-session".to_string(),
                    to: "DES-session".to_string(),
                    link_type: "refined_by".to_string(),
                    link_id: "REQ-session->DES-session:refined_by".to_string(),
                    link_hash: "h1".to_string(),
                },
                codefire_core::TraceLink {
                    from: "REQ-session".to_string(),
                    to: "TEST-session".to_string(),
                    link_type: "verified_by".to_string(),
                    link_id: "REQ-session->TEST-session:verified_by".to_string(),
                    link_hash: "h2".to_string(),
                },
            ],
        };

        let selection = atom_neighborhood(&trace_graph, "REQ-session", 1, 2);
        assert_eq!(selection.atom_ids.len(), 2);
        assert!(selection.truncated_atoms);
    }
}
