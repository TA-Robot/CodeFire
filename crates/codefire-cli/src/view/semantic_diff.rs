use super::{root_object, DiffAlgorithm, DiffOptions, ResolvedCommitish};
use crate::CliError;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn load_atom_index(
    objects: &Path,
    commit_id: &str,
) -> Result<codefire_core::AtomIndex, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    Ok(serde_json::from_value(root_object(
        objects,
        &commit,
        "atom_index",
    )?)?)
}

pub(super) fn load_trace_graph(
    objects: &Path,
    commit_id: &str,
) -> Result<codefire_core::TraceGraph, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    Ok(serde_json::from_value(root_object(
        objects,
        &commit,
        "trace_graph",
    )?)?)
}

pub(super) fn render_atom_diff(
    left: &codefire_core::AtomIndex,
    right: &codefire_core::AtomIndex,
    output: &mut String,
) {
    let left_atoms = atom_map(left);
    let right_atoms = atom_map(right);
    let ids = left_atoms
        .keys()
        .chain(right_atoms.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for atom_id in ids {
        match (left_atoms.get(atom_id), right_atoms.get(atom_id)) {
            (None, Some(atom)) => rows.push(format!(
                "+ atom {} kind={} path={} hash={}",
                atom.atom_id, atom.kind, atom.artifact_path, atom.content_hash
            )),
            (Some(atom), None) => rows.push(format!(
                "- atom {} kind={} path={} hash={}",
                atom.atom_id, atom.kind, atom.artifact_path, atom.content_hash
            )),
            (Some(left_atom), Some(right_atom)) if *left_atom != *right_atom => rows.push(format!(
                "~ atom {} {} -> {} path {} -> {}",
                atom_id,
                left_atom.content_hash,
                right_atom.content_hash,
                left_atom.artifact_path,
                right_atom.artifact_path
            )),
            _ => {}
        }
    }
    output.push_str("Atom diff:\n");
    if rows.is_empty() {
        output.push_str("  no atom changes\n");
    } else {
        for row in rows {
            output.push_str("  ");
            output.push_str(&row);
            output.push('\n');
        }
    }
}

pub(super) fn render_trace_diff(
    left: &codefire_core::TraceGraph,
    right: &codefire_core::TraceGraph,
    output: &mut String,
) {
    let left_links = trace_link_map(left);
    let right_links = trace_link_map(right);
    let ids = left_links
        .keys()
        .chain(right_links.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for link_id in ids {
        match (left_links.get(link_id), right_links.get(link_id)) {
            (None, Some(link)) => rows.push(format!(
                "+ link {} {} -{}-> {} hash={}",
                link.link_id, link.from, link.link_type, link.to, link.link_hash
            )),
            (Some(link), None) => rows.push(format!(
                "- link {} {} -{}-> {} hash={}",
                link.link_id, link.from, link.link_type, link.to, link.link_hash
            )),
            (Some(left_link), Some(right_link)) if *left_link != *right_link => rows.push(format!(
                "~ link {} {} -{}-> {} hash {} -> {}",
                link_id,
                right_link.from,
                right_link.link_type,
                right_link.to,
                left_link.link_hash,
                right_link.link_hash
            )),
            _ => {}
        }
    }
    output.push_str("Trace diff:\n");
    if rows.is_empty() {
        output.push_str("  no trace changes\n");
    } else {
        for row in rows {
            output.push_str("  ");
            output.push_str(&row);
            output.push('\n');
        }
    }
}

#[derive(Debug)]
pub(super) struct DiffImpact {
    missing_added: Vec<codefire_core::MissingRequiredLink>,
    missing_removed: Vec<codefire_core::MissingRequiredLink>,
    missing_persisting: Vec<codefire_core::MissingRequiredLink>,
    expected_fires: Vec<codefire_core::Fire>,
    next_actions: Vec<Value>,
}

pub(super) fn diff_impact(
    left: &ResolvedCommitish,
    right: &ResolvedCommitish,
    left_index: &codefire_core::AtomIndex,
    left_graph: &codefire_core::TraceGraph,
    right_index: &codefire_core::AtomIndex,
    right_graph: &codefire_core::TraceGraph,
) -> Result<DiffImpact, CliError> {
    let left_policy = load_verification_policy(&left.objects, &left.commit_id)?;
    let right_policy = load_verification_policy(&right.objects, &right.commit_id)?;
    let left_missing = codefire_core::required_link_missing(
        left_index,
        left_graph,
        &codefire_core::TracePolicy {
            required_links: left_policy.required_links,
        },
    );
    let right_missing = codefire_core::required_link_missing(
        right_index,
        right_graph,
        &codefire_core::TracePolicy {
            required_links: right_policy.required_links,
        },
    );
    let left_missing_map = missing_map(&left_missing);
    let right_missing_map = missing_map(&right_missing);

    let mut missing_added = Vec::new();
    let mut missing_removed = Vec::new();
    let mut missing_persisting = Vec::new();
    for (key, item) in &right_missing_map {
        if left_missing_map.contains_key(key) {
            missing_persisting.push((*item).clone());
        } else {
            missing_added.push((*item).clone());
        }
    }
    for (key, item) in &left_missing_map {
        if !right_missing_map.contains_key(key) {
            missing_removed.push((*item).clone());
        }
    }
    sort_missing(&mut missing_added);
    sort_missing(&mut missing_removed);
    sort_missing(&mut missing_persisting);

    let (scan, _) = codefire_core::build_scan_result(
        right_index.clone(),
        left_index.clone(),
        right_graph.clone(),
        Vec::new(),
        left.commit_id.clone(),
        "diff_impact",
        "1970-01-01T00:00:00Z",
    )?;
    let mut expected_fires = scan.open_fires;
    expected_fires.sort_by(|left, right| {
        left.source
            .atom_id
            .cmp(&right.source.atom_id)
            .then_with(|| left.target.atom_id.cmp(&right.target.atom_id))
    });
    let next_actions = next_actions(&missing_added, &expected_fires);
    Ok(DiffImpact {
        missing_added,
        missing_removed,
        missing_persisting,
        expected_fires,
        next_actions,
    })
}

fn load_verification_policy(
    objects: &Path,
    commit_id: &str,
) -> Result<codefire_core::VerificationPolicy, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let policy_object = root_object(objects, &commit, "policy")?;
    let Some(policy) = policy_object.get("policy") else {
        return Ok(codefire_core::default_verification_policy());
    };
    if policy.as_object().is_some_and(serde_json::Map::is_empty) {
        return Ok(codefire_core::default_verification_policy());
    }
    Ok(serde_json::from_value(policy.clone())?)
}

fn missing_map(
    items: &[codefire_core::MissingRequiredLink],
) -> BTreeMap<String, &codefire_core::MissingRequiredLink> {
    items.iter().map(|item| (missing_key(item), item)).collect()
}

fn missing_key(item: &codefire_core::MissingRequiredLink) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        item.atom_id, item.required_type, item.target_kind, item.min
    )
}

fn sort_missing(items: &mut [codefire_core::MissingRequiredLink]) {
    items.sort_by(|left, right| {
        left.atom_id
            .cmp(&right.atom_id)
            .then_with(|| left.required_type.cmp(&right.required_type))
            .then_with(|| left.target_kind.cmp(&right.target_kind))
    });
}

fn next_actions(
    missing_added: &[codefire_core::MissingRequiredLink],
    expected_fires: &[codefire_core::Fire],
) -> Vec<Value> {
    let mut actions = Vec::new();
    for item in missing_added {
        actions.push(json!({
            "kind": "add_required_trace_link",
            "atom_id": &item.atom_id,
            "required_type": &item.required_type,
            "target_kind": &item.target_kind,
            "found": item.found,
            "min": item.min,
            "hint": format!("add {} link from {} to a {} atom", item.required_type, item.atom_id, item.target_kind),
        }));
    }
    for fire in expected_fires {
        actions.push(json!({
            "kind": "review_fire_impact",
            "source_atom": &fire.source.atom_id,
            "target_atom": &fire.target.atom_id,
            "reason": &fire.reason,
            "hint": format!("review impact from {} to {}", fire.source.atom_id, fire.target.atom_id),
        }));
    }
    actions
}

pub(super) fn render_impact_diff(impact: &DiffImpact, output: &mut String) {
    output.push_str("Impact diff:\n");
    output.push_str(&format!(
        "  missing required links: +{} -{} unchanged {}\n",
        impact.missing_added.len(),
        impact.missing_removed.len(),
        impact.missing_persisting.len()
    ));
    for item in &impact.missing_added {
        output.push_str(&format!(
            "  + missing {} requires {} -> {} min {} found {}\n",
            item.atom_id, item.required_type, item.target_kind, item.min, item.found
        ));
    }
    for item in &impact.missing_removed {
        output.push_str(&format!(
            "  - missing {} requires {} -> {} min {} found {}\n",
            item.atom_id, item.required_type, item.target_kind, item.min, item.found
        ));
    }
    output.push_str(&format!(
        "  expected fires: {}\n",
        impact.expected_fires.len()
    ));
    for fire in &impact.expected_fires {
        output.push_str(&format!(
            "  * fire {} -> {} reason={}\n",
            fire.source.atom_id, fire.target.atom_id, fire.reason
        ));
    }
    output.push_str("Next actions:\n");
    if impact.next_actions.is_empty() {
        output.push_str("  none\n");
    } else {
        for action in &impact.next_actions {
            output.push_str("  ");
            output.push_str(&action.to_string());
            output.push('\n');
        }
    }
}

pub(super) struct DiffJsonContext<'a> {
    pub(super) left: &'a ResolvedCommitish,
    pub(super) right: &'a ResolvedCommitish,
    pub(super) left_index: Option<&'a codefire_core::AtomIndex>,
    pub(super) right_index: Option<&'a codefire_core::AtomIndex>,
    pub(super) left_graph: Option<&'a codefire_core::TraceGraph>,
    pub(super) right_graph: Option<&'a codefire_core::TraceGraph>,
    pub(super) impact: Option<&'a DiffImpact>,
    pub(super) options: &'a DiffOptions,
}

pub(super) fn render_diff_json(context: DiffJsonContext<'_>) -> Result<String, CliError> {
    let value = json!({
        "type": "codefire_diff",
        "version": 1,
        "left": {"label": &context.left.label, "commit": &context.left.commit_id},
        "right": {"label": &context.right.label, "commit": &context.right.commit_id},
        "options": {
            "algorithm": diff_algorithm_name(context.options.algorithm),
            "rename_detection": context.options.rename_detection,
            "atoms": context.options.atom_diff,
            "trace": context.options.trace_diff,
            "impact": context.options.impact_diff,
        },
        "atoms": if context.options.atom_diff {
            atom_diff_json(context.left_index.expect("left atom index loaded"), context.right_index.expect("right atom index loaded"))
        } else {
            json!(null)
        },
        "trace": if context.options.trace_diff {
            trace_diff_json(context.left_graph.expect("left trace graph loaded"), context.right_graph.expect("right trace graph loaded"))
        } else {
            json!(null)
        },
        "impact": context.impact.map(impact_json),
        "next_actions": context.impact.map(|impact| impact.next_actions.clone()).unwrap_or_default(),
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

fn diff_algorithm_name(algorithm: DiffAlgorithm) -> &'static str {
    match algorithm {
        DiffAlgorithm::Myers => "myers",
        DiffAlgorithm::Patience => "patience",
        DiffAlgorithm::Histogram => "histogram",
    }
}

fn atom_diff_json(left: &codefire_core::AtomIndex, right: &codefire_core::AtomIndex) -> Value {
    let left_atoms = atom_map(left);
    let right_atoms = atom_map(right);
    let ids = left_atoms
        .keys()
        .chain(right_atoms.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for atom_id in ids {
        match (left_atoms.get(atom_id), right_atoms.get(atom_id)) {
            (None, Some(atom)) => added.push(atom_json(atom)),
            (Some(atom), None) => removed.push(atom_json(atom)),
            (Some(left_atom), Some(right_atom)) if *left_atom != *right_atom => {
                changed.push(json!({
                    "atom_id": atom_id,
                    "before": atom_json(left_atom),
                    "after": atom_json(right_atom),
                }));
            }
            _ => {}
        }
    }
    json!({"added": added, "removed": removed, "changed": changed})
}

fn trace_diff_json(left: &codefire_core::TraceGraph, right: &codefire_core::TraceGraph) -> Value {
    let left_links = trace_link_map(left);
    let right_links = trace_link_map(right);
    let ids = left_links
        .keys()
        .chain(right_links.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for link_id in ids {
        match (left_links.get(link_id), right_links.get(link_id)) {
            (None, Some(link)) => added.push(trace_link_json(link)),
            (Some(link), None) => removed.push(trace_link_json(link)),
            (Some(left_link), Some(right_link)) if *left_link != *right_link => {
                changed.push(json!({
                    "link_id": link_id,
                    "before": trace_link_json(left_link),
                    "after": trace_link_json(right_link),
                }));
            }
            _ => {}
        }
    }
    json!({"added": added, "removed": removed, "changed": changed})
}

fn atom_json(atom: &codefire_core::Atom) -> Value {
    json!({
        "atom_id": &atom.atom_id,
        "kind": &atom.kind,
        "artifact_path": &atom.artifact_path,
        "content_hash": &atom.content_hash,
    })
}

fn trace_link_json(link: &codefire_core::TraceLink) -> Value {
    json!({
        "link_id": &link.link_id,
        "from": &link.from,
        "to": &link.to,
        "type": &link.link_type,
        "link_hash": &link.link_hash,
    })
}

fn impact_json(impact: &DiffImpact) -> Value {
    json!({
        "missing_required_links": {
            "added": impact.missing_added.iter().map(missing_json).collect::<Vec<_>>(),
            "removed": impact.missing_removed.iter().map(missing_json).collect::<Vec<_>>(),
            "persisting": impact.missing_persisting.iter().map(missing_json).collect::<Vec<_>>(),
        },
        "expected_fires": impact.expected_fires.iter().map(fire_json).collect::<Vec<_>>(),
    })
}

fn missing_json(item: &codefire_core::MissingRequiredLink) -> Value {
    json!({
        "atom_id": &item.atom_id,
        "required_type": &item.required_type,
        "target_kind": &item.target_kind,
        "min": item.min,
        "found": item.found,
    })
}

fn fire_json(fire: &codefire_core::Fire) -> Value {
    json!({
        "source_atom": &fire.source.atom_id,
        "target_atom": &fire.target.atom_id,
        "reason": &fire.reason,
        "trace_path": &fire.trace_path,
    })
}

fn atom_map(index: &codefire_core::AtomIndex) -> BTreeMap<&str, &codefire_core::Atom> {
    index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect()
}

fn trace_link_map(graph: &codefire_core::TraceGraph) -> BTreeMap<&str, &codefire_core::TraceLink> {
    graph
        .links
        .iter()
        .map(|link| (link.link_id.as_str(), link))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_diff_reports_added_removed_and_changed_atoms() {
        let left = codefire_core::AtomIndex {
            type_tag: "atom_index".to_string(),
            version: 1,
            hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-OLD", "requirement", "docs/old.md", "old-hash"),
                atom("REQ-SAME", "requirement", "docs/same.md", "hash-a"),
                atom("REQ-CHANGED", "requirement", "docs/change.md", "hash-a"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let right = codefire_core::AtomIndex {
            type_tag: "atom_index".to_string(),
            version: 1,
            hash_schema_version: codefire_core::ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-NEW", "requirement", "docs/new.md", "new-hash"),
                atom("REQ-SAME", "requirement", "docs/same.md", "hash-a"),
                atom("REQ-CHANGED", "requirement", "docs/change.md", "hash-b"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let mut output = String::new();
        render_atom_diff(&left, &right, &mut output);
        assert!(output.contains("- atom REQ-OLD"));
        assert!(output.contains("+ atom REQ-NEW"));
        assert!(output.contains("~ atom REQ-CHANGED hash-a -> hash-b"));
        assert!(!output.contains("REQ-SAME"));
    }

    #[test]
    fn trace_diff_reports_added_removed_and_changed_links() {
        let left = codefire_core::TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: 1,
            links: vec![
                link("L-OLD", "REQ-A", "DES-A", "refined_by", "old-hash"),
                link("L-SAME", "REQ-B", "DES-B", "refined_by", "same-hash"),
                link("L-CHANGED", "REQ-C", "DES-C", "refined_by", "hash-a"),
            ],
        };
        let right = codefire_core::TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: 1,
            links: vec![
                link("L-NEW", "REQ-D", "DES-D", "refined_by", "new-hash"),
                link("L-SAME", "REQ-B", "DES-B", "refined_by", "same-hash"),
                link("L-CHANGED", "REQ-C", "DES-C2", "refined_by", "hash-b"),
            ],
        };
        let mut output = String::new();
        render_trace_diff(&left, &right, &mut output);
        assert!(output.contains("- link L-OLD REQ-A -refined_by-> DES-A"));
        assert!(output.contains("+ link L-NEW REQ-D -refined_by-> DES-D"));
        assert!(
            output.contains("~ link L-CHANGED REQ-C -refined_by-> DES-C2 hash hash-a -> hash-b")
        );
        assert!(!output.contains("L-SAME"));
    }

    fn atom(
        atom_id: &str,
        kind: &str,
        artifact_path: &str,
        content_hash: &str,
    ) -> codefire_core::Atom {
        codefire_core::Atom {
            atom_id: atom_id.to_string(),
            kind: kind.to_string(),
            artifact_path: artifact_path.to_string(),
            selector: codefire_core::Selector {
                selector_type: "heading".to_string(),
                value: atom_id.to_string(),
            },
            content_hash: content_hash.to_string(),
        }
    }

    fn link(
        link_id: &str,
        from: &str,
        to: &str,
        link_type: &str,
        link_hash: &str,
    ) -> codefire_core::TraceLink {
        codefire_core::TraceLink {
            from: from.to_string(),
            to: to.to_string(),
            link_type: link_type.to_string(),
            link_id: link_id.to_string(),
            link_hash: link_hash.to_string(),
        }
    }
}
