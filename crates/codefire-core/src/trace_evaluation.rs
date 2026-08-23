use crate::{AtomIndex, MissingRequiredLink, TraceGraph, TracePolicy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Aggregate correctness work performed by one compiled Trace evaluator.
///
/// These counters are observational only: they neither define nor enforce a
/// resource budget. Callers must not interpret successful index construction
/// as budget compliance; enforcement belongs to a separately bounded API.
///
/// Each field has an intentionally mechanical definition so scale tests can
/// detect an algorithmic regression without depending on wall-clock timing:
///
/// - `atoms_indexed`: Atom records read while building the kind index;
/// - `rules_evaluated`: applicable required-link rules examined;
/// - `links_indexed`: Trace Link records read by the single compile pass;
/// - `index_entries`: retained Atom rule-subject, adjacency, and required-count entries;
/// - `lookups`: target-kind, adjacency, policy-kind, and required-count probes;
/// - `adjacency_visits`: adjacency entries returned to callers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceWorkCounters {
    pub atoms_indexed: usize,
    pub rules_evaluated: usize,
    pub links_indexed: usize,
    pub index_entries: usize,
    pub lookups: usize,
    pub adjacency_visits: usize,
}

/// One immutable Trace graph compilation plus cumulative query counters.
///
/// Adjacency vectors retain source Link order. A self-loop contributes one
/// adjacency entry, matching the former `if/else if` scan, while duplicate
/// Links remain duplicate entries. Required-link counts retain the existing
/// directed (`from` -> target kind) semantics. Compilation intentionally does
/// not validate endpoints or duplicate Atom IDs; owning callers must retain
/// their existing corruption-validation path.
#[derive(Debug, Clone)]
pub struct TraceEvaluationIndex {
    atoms: Vec<(String, String)>,
    adjacency: BTreeMap<String, Vec<String>>,
    required_counts: BTreeMap<(String, String, String), usize>,
    counters: TraceWorkCounters,
}

impl TraceEvaluationIndex {
    pub fn compile(atom_index: &AtomIndex, trace_graph: &TraceGraph) -> Self {
        let atoms = atom_index
            .atoms
            .iter()
            .map(|atom| (atom.atom_id.clone(), atom.kind.clone()))
            .collect::<Vec<_>>();
        let mut atom_kinds = BTreeMap::<&str, &str>::new();
        for (atom_id, kind) in &atoms {
            // Intentional last-record-wins compatibility for callers that use
            // this evaluator before duplicate-Atom validation.
            atom_kinds.insert(atom_id.as_str(), kind.as_str());
        }

        let mut adjacency = BTreeMap::<String, Vec<String>>::new();
        let mut required_counts = BTreeMap::<(String, String, String), usize>::new();
        let mut counters = TraceWorkCounters {
            atoms_indexed: atom_index.atoms.len(),
            ..TraceWorkCounters::default()
        };

        for link in &trace_graph.links {
            checked_counter_add(&mut counters.links_indexed, 1);

            adjacency
                .entry(link.from.clone())
                .or_default()
                .push(link.to.clone());
            if link.from != link.to {
                adjacency
                    .entry(link.to.clone())
                    .or_default()
                    .push(link.from.clone());
            }

            checked_counter_add(&mut counters.lookups, 1);
            if let Some(target_kind) = atom_kinds.get(link.to.as_str()) {
                *required_counts
                    .entry((
                        link.from.clone(),
                        link.link_type.clone(),
                        (*target_kind).to_string(),
                    ))
                    .or_insert(0) += 1;
            }
        }

        let adjacency_entries = adjacency.values().fold(0, |total, entries| {
            checked_counter_sum(total, entries.len())
        });
        counters.index_entries = checked_counter_sum(
            checked_counter_sum(atoms.len(), adjacency_entries),
            required_counts.len(),
        );
        Self {
            atoms,
            adjacency,
            required_counts,
            counters,
        }
    }

    pub fn adjacent_atoms(&mut self, atom_id: &str) -> Vec<(String, Vec<String>)> {
        checked_counter_add(&mut self.counters.lookups, 1);
        let Some(targets) = self.adjacency.get(atom_id) else {
            return Vec::new();
        };
        checked_counter_add(&mut self.counters.adjacency_visits, targets.len());
        targets
            .iter()
            .map(|target| (target.clone(), vec![atom_id.to_string(), target.clone()]))
            .collect()
    }

    pub fn missing_required_links(&mut self, policy: &TracePolicy) -> Vec<MissingRequiredLink> {
        let mut missing = Vec::new();
        for (atom_id, atom_kind) in &self.atoms {
            checked_counter_add(&mut self.counters.lookups, 1);
            let Some(rules) = policy.required_links.get(atom_kind) else {
                continue;
            };
            for rule in rules {
                checked_counter_add(&mut self.counters.rules_evaluated, 1);
                checked_counter_add(&mut self.counters.lookups, 1);
                let found = self
                    .required_counts
                    .get(&(
                        atom_id.clone(),
                        rule.link_type.clone(),
                        rule.target_kind.clone(),
                    ))
                    .copied()
                    .unwrap_or_default();
                if found < rule.min {
                    missing.push(MissingRequiredLink {
                        atom_id: atom_id.clone(),
                        required_type: rule.link_type.clone(),
                        target_kind: rule.target_kind.clone(),
                        min: rule.min,
                        found,
                    });
                }
            }
        }
        missing
    }

    pub fn counters(&self) -> TraceWorkCounters {
        self.counters
    }
}

fn checked_counter_add(counter: &mut usize, amount: usize) {
    *counter = checked_counter_sum(*counter, amount);
}

fn checked_counter_sum(left: usize, right: usize) -> usize {
    left.checked_add(right).expect(
        "Trace work counter overflow is unreachable while its addressable backing indexes exist",
    )
}
