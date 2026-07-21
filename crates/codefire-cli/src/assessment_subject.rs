use crate::{read_json, required_string, CliError, OpenContext};
use codefire_core::AssessmentSubject;
use serde_json::{json, Value};
use std::path::Path;

const SUBJECT_TYPE: &str = "codefire_assessment_subject";
const SUBJECT_VERSION: u32 = 1;

pub(crate) fn live_open_subject(
    context: &OpenContext,
    scan: &codefire_core::ScanResult,
) -> Result<AssessmentSubject, CliError> {
    let repository_id = repository_id(&context.repo_root)?;
    let open_instance_id = required_string(&context.registry, &["open", "open_instance_id"])?;
    let base_commit = required_string(&context.registry, &["open", "current_base_commit"])?;
    let worktree_digest = crate::worktree_manifest_digest(&context.open_dir)?;
    let content = json!({
        "base_commit": &scan.base_commit,
        "atom_index": &scan.atom_index,
        "trace_graph": &scan.trace_graph,
        "changed_atoms": &scan.changed_atoms,
        "non_atom_changed_files": &scan.non_atom_changed_files,
        "open_fires": &scan.open_fires,
        "tool_migration": &scan.tool_migration,
    });
    build_subject(SubjectParts {
        kind: "live_open",
        trust_kind: "local_unisolated",
        content: &content,
        worktree_digest: Some(worktree_digest),
        immutable: false,
        repository_id: Some(repository_id),
        branch: Some(context.branch.clone()),
        open_instance_id: Some(open_instance_id),
        base_commit: Some(base_commit),
        source_commit: None,
    })
}

pub(crate) fn sealed_pair_subject(
    repo_root: Option<&Path>,
    base_commit: &str,
    source_commit: &str,
) -> Result<AssessmentSubject, CliError> {
    let repository_id = repo_root.map(repository_id).transpose()?;
    let content = json!({
        "base_commit": base_commit,
        "source_commit": source_commit,
    });
    build_subject(SubjectParts {
        kind: "sealed_commit_pair",
        trust_kind: "sealed_validated",
        content: &content,
        worktree_digest: None,
        immutable: true,
        repository_id,
        branch: None,
        open_instance_id: None,
        base_commit: Some(base_commit.to_string()),
        source_commit: Some(source_commit.to_string()),
    })
}

pub(crate) fn subject_json(subject: &AssessmentSubject) -> Result<Value, CliError> {
    serde_json::to_value(subject).map_err(CliError::from)
}

struct SubjectParts<'a> {
    kind: &'a str,
    trust_kind: &'a str,
    content: &'a Value,
    worktree_digest: Option<String>,
    immutable: bool,
    repository_id: Option<String>,
    branch: Option<String>,
    open_instance_id: Option<String>,
    base_commit: Option<String>,
    source_commit: Option<String>,
}

fn build_subject(parts: SubjectParts<'_>) -> Result<AssessmentSubject, CliError> {
    let content_digest = digest_value(parts.content)?;
    let identity = json!({
        "type": SUBJECT_TYPE,
        "version": SUBJECT_VERSION,
        "kind": parts.kind,
        "trust_kind": parts.trust_kind,
        "content_digest": &content_digest,
        "worktree_digest": &parts.worktree_digest,
        "immutable": parts.immutable,
        "repository_id": &parts.repository_id,
        "branch": &parts.branch,
        "open_instance_id": &parts.open_instance_id,
        "base_commit": &parts.base_commit,
        "source_commit": &parts.source_commit,
    });
    let subject_digest = digest_value(&identity)?;
    Ok(AssessmentSubject {
        type_tag: SUBJECT_TYPE.to_string(),
        version: SUBJECT_VERSION,
        kind: parts.kind.to_string(),
        trust_kind: parts.trust_kind.to_string(),
        subject_digest,
        content_digest,
        worktree_digest: parts.worktree_digest,
        immutable: parts.immutable,
        repository_id: parts.repository_id,
        branch: parts.branch,
        open_instance_id: parts.open_instance_id,
        base_commit: parts.base_commit,
        source_commit: parts.source_commit,
    })
}

fn repository_id(repo_root: &Path) -> Result<String, CliError> {
    let repo = read_json(&repo_root.join(".codefire").join("repo.json"))?;
    required_string(&repo, &["repository_id"])
}

fn digest_value(value: &Value) -> Result<String, CliError> {
    Ok(format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(value)?)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_subject_digest_is_deterministic_and_identity_bound() {
        let first = sealed_pair_subject(None, "CF-COMMIT-base", "CF-COMMIT-source").unwrap();
        let same = sealed_pair_subject(None, "CF-COMMIT-base", "CF-COMMIT-source").unwrap();
        let different = sealed_pair_subject(None, "CF-COMMIT-base", "CF-COMMIT-other").unwrap();
        assert_eq!(first, same);
        assert_ne!(first.subject_digest, different.subject_digest);
        assert_eq!(first.trust_kind, "sealed_validated");
        assert!(first.immutable);
    }
}
