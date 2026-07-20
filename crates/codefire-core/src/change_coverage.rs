use crate::{AtomIndex, CoreError, ToolMigration};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CHANGE_COVERAGE_PROFILE: &str = "file-relation-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFileChange {
    pub path: String,
    pub status: String,
    pub base_blob: Option<String>,
    pub current_blob: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeCoverageRelation {
    NonAtomFile,
    DeletedAtom,
    OutOfAtomChange,
    AtomChangeWithUnclassifiedSpans,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeCoverageEntry {
    pub path: String,
    pub status: String,
    pub relation: ChangeCoverageRelation,
    pub base_atom_ids: Vec<String>,
    pub current_atom_ids: Vec<String>,
    pub changed_atom_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_blob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_blob: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeCoverageSummary {
    pub changed_files: usize,
    pub non_atom_files: usize,
    pub deleted_atom_files: usize,
    pub out_of_atom_files: usize,
    pub atom_files_with_unclassified_spans: usize,
    pub tool_migrations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeCoverage {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub profile: String,
    pub span_precision: String,
    pub entries: Vec<ChangeCoverageEntry>,
    pub summary: ChangeCoverageSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_migration: Option<ToolMigration>,
    pub digest: String,
}

pub fn build_change_coverage(
    mut changes: Vec<ManifestFileChange>,
    current: &AtomIndex,
    base: &AtomIndex,
    changed_atoms: &[String],
    tool_migration: Option<ToolMigration>,
) -> Result<ChangeCoverage, CoreError> {
    changes.sort_by(|left, right| left.path.cmp(&right.path));
    for pair in changes.windows(2) {
        if pair[0].path == pair[1].path {
            return Err(CoreError::Config(format!(
                "duplicate changed manifest path {}",
                pair[0].path
            )));
        }
    }
    let base_by_path = atom_ids_by_path(base);
    let current_by_path = atom_ids_by_path(current);
    let changed = changed_atoms
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut entries = Vec::with_capacity(changes.len());
    for change in changes {
        let base_atom_ids = base_by_path.get(&change.path).cloned().unwrap_or_default();
        let current_atom_ids = current_by_path
            .get(&change.path)
            .cloned()
            .unwrap_or_default();
        let changed_atom_ids = current_atom_ids
            .iter()
            .filter(|atom_id| changed.contains(atom_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        let relation = if change.status == "deleted" && !base_atom_ids.is_empty() {
            ChangeCoverageRelation::DeletedAtom
        } else if !changed_atom_ids.is_empty() {
            // Atom selectors do not currently carry byte spans. Preserve that uncertainty
            // instead of claiming the entire file was semantically covered.
            ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans
        } else if !base_atom_ids.is_empty() || !current_atom_ids.is_empty() {
            ChangeCoverageRelation::OutOfAtomChange
        } else {
            ChangeCoverageRelation::NonAtomFile
        };
        entries.push(ChangeCoverageEntry {
            path: change.path,
            status: change.status,
            relation,
            base_atom_ids,
            current_atom_ids,
            changed_atom_ids,
            base_blob: change.base_blob,
            current_blob: change.current_blob,
        });
    }
    let summary = ChangeCoverageSummary {
        changed_files: entries.len(),
        non_atom_files: relation_count(&entries, ChangeCoverageRelation::NonAtomFile),
        deleted_atom_files: relation_count(&entries, ChangeCoverageRelation::DeletedAtom),
        out_of_atom_files: relation_count(&entries, ChangeCoverageRelation::OutOfAtomChange),
        atom_files_with_unclassified_spans: relation_count(
            &entries,
            ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans,
        ),
        tool_migrations: usize::from(tool_migration.is_some()),
    };
    let digest = change_coverage_digest(&entries, &summary, tool_migration.as_ref())?;
    Ok(ChangeCoverage {
        type_tag: "change_coverage".to_string(),
        version: 1,
        profile: CHANGE_COVERAGE_PROFILE.to_string(),
        span_precision: "file_only_no_span_claim".to_string(),
        entries,
        summary,
        tool_migration,
        digest,
    })
}

pub fn validate_change_coverage(coverage: &ChangeCoverage) -> Result<(), CoreError> {
    if coverage.type_tag != "change_coverage"
        || coverage.version != 1
        || coverage.profile != CHANGE_COVERAGE_PROFILE
        || coverage.span_precision != "file_only_no_span_claim"
    {
        return Err(CoreError::Config(
            "unsupported change coverage contract".to_string(),
        ));
    }
    let mut paths = BTreeSet::new();
    for entry in &coverage.entries {
        if !paths.insert(entry.path.as_str()) {
            return Err(CoreError::Config(format!(
                "duplicate change coverage path {}",
                entry.path
            )));
        }
        if !is_sorted_unique(&entry.base_atom_ids)
            || !is_sorted_unique(&entry.current_atom_ids)
            || !is_sorted_unique(&entry.changed_atom_ids)
            || !entry
                .changed_atom_ids
                .iter()
                .all(|atom_id| entry.current_atom_ids.binary_search(atom_id).is_ok())
        {
            return Err(CoreError::Config(format!(
                "change coverage atom identities are not normalized for {}",
                entry.path
            )));
        }
        let expected_relation = if entry.status == "deleted" && !entry.base_atom_ids.is_empty() {
            ChangeCoverageRelation::DeletedAtom
        } else if !entry.changed_atom_ids.is_empty() {
            ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans
        } else if !entry.base_atom_ids.is_empty() || !entry.current_atom_ids.is_empty() {
            ChangeCoverageRelation::OutOfAtomChange
        } else {
            ChangeCoverageRelation::NonAtomFile
        };
        if entry.relation != expected_relation {
            return Err(CoreError::Config(format!(
                "change coverage relation does not match Atom facts for {}",
                entry.path
            )));
        }
    }
    if coverage
        .tool_migration
        .as_ref()
        .is_some_and(|migration| migration.source_unchanged && !coverage.entries.is_empty())
    {
        return Err(CoreError::Config(
            "source-unchanged tool migration cannot contain changed-file coverage entries"
                .to_string(),
        ));
    }
    let computed_summary = ChangeCoverageSummary {
        changed_files: coverage.entries.len(),
        non_atom_files: relation_count(&coverage.entries, ChangeCoverageRelation::NonAtomFile),
        deleted_atom_files: relation_count(&coverage.entries, ChangeCoverageRelation::DeletedAtom),
        out_of_atom_files: relation_count(
            &coverage.entries,
            ChangeCoverageRelation::OutOfAtomChange,
        ),
        atom_files_with_unclassified_spans: relation_count(
            &coverage.entries,
            ChangeCoverageRelation::AtomChangeWithUnclassifiedSpans,
        ),
        tool_migrations: usize::from(coverage.tool_migration.is_some()),
    };
    if computed_summary != coverage.summary {
        return Err(CoreError::Config(
            "change coverage summary does not match entries".to_string(),
        ));
    }
    let computed = change_coverage_digest(
        &coverage.entries,
        &coverage.summary,
        coverage.tool_migration.as_ref(),
    )?;
    if computed != coverage.digest {
        return Err(CoreError::Config(format!(
            "change coverage digest mismatch: computed {computed}, recorded {}",
            coverage.digest
        )));
    }
    Ok(())
}

fn atom_ids_by_path(index: &AtomIndex) -> BTreeMap<String, Vec<String>> {
    let mut by_path = BTreeMap::<String, Vec<String>>::new();
    for atom in &index.atoms {
        by_path
            .entry(atom.artifact_path.clone())
            .or_default()
            .push(atom.atom_id.clone());
    }
    for atoms in by_path.values_mut() {
        atoms.sort();
    }
    by_path
}

fn relation_count(entries: &[ChangeCoverageEntry], relation: ChangeCoverageRelation) -> usize {
    entries
        .iter()
        .filter(|entry| entry.relation == relation)
        .count()
}

fn is_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn change_coverage_digest(
    entries: &[ChangeCoverageEntry],
    summary: &ChangeCoverageSummary,
    tool_migration: Option<&ToolMigration>,
) -> Result<String, CoreError> {
    #[derive(Serialize)]
    struct Subject<'a> {
        profile: &'a str,
        span_precision: &'a str,
        entries: &'a [ChangeCoverageEntry],
        summary: &'a ChangeCoverageSummary,
        tool_migration: Option<&'a ToolMigration>,
    }
    Ok(codefire_util::sha256_hex(&serde_json::to_vec(&Subject {
        profile: CHANGE_COVERAGE_PROFILE,
        span_precision: "file_only_no_span_claim",
        entries,
        summary,
        tool_migration,
    })?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Atom, Selector};

    fn index(atoms: &[(&str, &str, &str)]) -> AtomIndex {
        AtomIndex {
            type_tag: "atom_index".to_string(),
            version: 1,
            hash_schema_version: 4,
            atoms: atoms
                .iter()
                .map(|(id, path, hash)| Atom {
                    atom_id: (*id).to_string(),
                    kind: "code".to_string(),
                    artifact_path: (*path).to_string(),
                    selector: Selector {
                        selector_type: "line".to_string(),
                        value: "1".to_string(),
                    },
                    content_hash: (*hash).to_string(),
                })
                .collect(),
            duplicate_atom_ids: Vec::new(),
        }
    }

    #[test]
    fn classifies_non_atom_mixed_unknown_out_of_atom_and_deleted_atom_without_overclaim() {
        let base = index(&[
            ("CODE:mixed", "mixed.py", "old"),
            ("CODE:outside", "outside.py", "same"),
            ("CODE:deleted", "deleted.py", "old"),
        ]);
        let current = index(&[
            ("CODE:mixed", "mixed.py", "new"),
            ("CODE:outside", "outside.py", "same"),
        ]);
        let changes = ["plain.txt", "mixed.py", "outside.py", "deleted.py"]
            .into_iter()
            .map(|path| ManifestFileChange {
                path: path.to_string(),
                status: if path == "deleted.py" {
                    "deleted"
                } else {
                    "modified"
                }
                .to_string(),
                base_blob: Some("a".to_string()),
                current_blob: (path != "deleted.py").then(|| "b".to_string()),
            })
            .collect();
        let coverage =
            build_change_coverage(changes, &current, &base, &["CODE:mixed".to_string()], None)
                .unwrap();
        assert_eq!(coverage.summary.changed_files, 4);
        assert_eq!(coverage.summary.non_atom_files, 1);
        assert_eq!(coverage.summary.deleted_atom_files, 1);
        assert_eq!(coverage.summary.out_of_atom_files, 1);
        assert_eq!(coverage.summary.atom_files_with_unclassified_spans, 1);
        validate_change_coverage(&coverage).unwrap();
    }

    #[test]
    fn source_unchanged_tool_migration_is_a_typed_zero_file_coverage_fact() {
        let empty = index(&[]);
        let migration = ToolMigration {
            kind: "atom_hash_schema_changed".to_string(),
            from_hash_schema_version: 3,
            to_hash_schema_version: 4,
            source_unchanged: true,
            affected_atoms: vec!["CODE:migrated".to_string()],
        };
        let coverage =
            build_change_coverage(Vec::new(), &empty, &empty, &[], Some(migration.clone()))
                .unwrap();
        assert!(coverage.entries.is_empty());
        assert_eq!(coverage.summary.changed_files, 0);
        assert_eq!(coverage.summary.tool_migrations, 1);
        assert_eq!(coverage.tool_migration, Some(migration));
        validate_change_coverage(&coverage).unwrap();
    }
}
