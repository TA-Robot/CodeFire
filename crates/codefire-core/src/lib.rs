use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: u32 = 1;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum CoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io(error) => write!(f, "{error}"),
            CoreError::Json(error) => write!(f, "{error}"),
            CoreError::Config(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        CoreError::Io(error)
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        CoreError::Json(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    #[serde(rename = "type")]
    pub selector_type: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Atom {
    pub atom_id: String,
    pub kind: String,
    pub artifact_path: String,
    pub selector: Selector,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomIndex {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub atoms: Vec<Atom>,
    pub duplicate_atom_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLink {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub link_id: String,
    pub link_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceGraph {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub links: Vec<TraceLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredLinkRule {
    #[serde(rename = "type")]
    pub link_type: String,
    pub target_kind: String,
    pub min: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TracePolicy {
    pub required_links: BTreeMap<String, Vec<RequiredLinkRule>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub require_no_required_fires: bool,
    pub require_no_stale_resolutions: bool,
    pub require_trace_completeness: bool,
    pub require_verification_success: bool,
    pub reject_duplicate_atom_ids: bool,
    pub no_change_required_requires_rationale: bool,
    pub required_links: BTreeMap<String, Vec<RequiredLinkRule>>,
    pub verification: Vec<VerificationCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCommand {
    pub id: String,
    pub command: String,
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingRequiredLink {
    pub atom_id: String,
    pub required_type: String,
    pub target_kind: String,
    pub min: usize,
    pub found: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FireAtomRef {
    pub atom_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash_at_fire: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fire {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub fire_uid: String,
    pub display_id: String,
    pub status: String,
    pub severity: String,
    pub source: FireAtomRef,
    pub target: FireAtomRef,
    pub reason: String,
    pub trace_path: Vec<String>,
    pub created_by: String,
    pub created_at: String,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obsolete_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_uid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub base_commit: String,
    pub atom_index: AtomIndex,
    pub trace_graph: TraceGraph,
    pub changed_atoms: Vec<String>,
    pub open_fires: Vec<Fire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedCheck {
    pub id: String,
    pub command: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleResolution {
    pub resolution_uid: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingEvidenceRef {
    pub resolution_uid: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verification {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub result: String,
    #[serde(default = "default_true")]
    pub trace_completeness_required: bool,
    pub open_required_fires: usize,
    pub failed_checks: Vec<FailedCheck>,
    pub missing_required_links: Vec<MissingRequiredLink>,
    pub stale_resolutions: Vec<StaleResolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_evidence_refs: Vec<MissingEvidenceRef>,
    pub duplicate_atom_ids: Vec<String>,
    pub verified_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionAtomBasis {
    pub atom_id: String,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionTraceLinkBasis {
    pub link_id: String,
    pub link_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionBasis {
    pub source_atom: ResolutionAtomBasis,
    pub target_atom: ResolutionAtomBasis,
    pub trace_links: Vec<ResolutionTraceLinkBasis>,
    pub policy_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub resolution_uid: String,
    pub fire_uid: String,
    pub resolution_type: String,
    pub rationale: String,
    pub evidence: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    pub basis: ResolutionBasis,
    pub resolved_at: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionRequest {
    pub resolution_uid: String,
    pub resolution_type: String,
    pub rationale: String,
    pub evidence: String,
    pub evidence_refs: Vec<String>,
    pub resolved_at: String,
}

pub fn build_atom_index(open_dir: &Path) -> Result<AtomIndex, CoreError> {
    let config = Config::parse(open_dir)?;
    let mut atoms = Vec::new();
    for rel in rel_files(open_dir)? {
        let path = open_dir.join(&rel);
        let rel_posix = to_posix(&rel);
        if config.matches("requirements", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "requirement")?);
        } else if config.matches("designs", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "design")?);
        } else if config.matches("adrs", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "adr")?);
        } else if config.matches("ops", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "ops")?);
        } else if config.matches("code", &rel_posix) {
            atoms.extend(extract_explicit_cf_atoms(&path, &rel_posix, "code")?);
        } else if config.matches("tests", &rel_posix) {
            atoms.extend(extract_explicit_cf_atoms(&path, &rel_posix, "test")?);
        }
    }
    atoms.sort_by(|left, right| {
        left.atom_id
            .cmp(&right.atom_id)
            .then_with(|| left.artifact_path.cmp(&right.artifact_path))
    });
    let mut counts = BTreeMap::new();
    for atom in &atoms {
        *counts.entry(atom.atom_id.clone()).or_insert(0usize) += 1;
    }
    let duplicate_atom_ids = counts
        .into_iter()
        .filter_map(|(atom_id, count)| (count > 1).then_some(atom_id))
        .collect();
    Ok(AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        atoms,
        duplicate_atom_ids,
    })
}

pub fn current_trace_graph(open_dir: &Path) -> Result<TraceGraph, CoreError> {
    let links = parse_links(&open_dir.join("codefire.links.yaml"))?
        .into_iter()
        .map(TraceLinkInput::into_trace_link)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TraceGraph {
        type_tag: "trace_graph".to_string(),
        version: VERSION,
        links,
    })
}

pub fn parse_links(path: &Path) -> Result<Vec<TraceLinkInput>, CoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut links = Vec::new();
    let mut current = None;
    let mut current_line = 0usize;

    for (line_index, raw_line) in read_text_lossy(path)?.lines().enumerate() {
        let line_no = line_index + 1;
        let stripped = raw_line.split('#').next().unwrap_or_default().trim();
        if stripped.is_empty() {
            continue;
        }
        if let Some(value) = stripped.strip_prefix("- from:") {
            flush_link(&mut links, current.take(), current_line)?;
            current = Some(TraceLinkInput {
                from: unquote(value.trim()),
                to: None,
                link_type: None,
            });
            current_line = line_no;
            continue;
        }
        let Some(link) = current.as_mut() else {
            continue;
        };
        if let Some(value) = stripped.strip_prefix("to:") {
            link.to = Some(unquote(value.trim()));
        } else if let Some(value) = stripped.strip_prefix("type:") {
            link.link_type = Some(unquote(value.trim()));
        }
    }
    flush_link(&mut links, current, current_line)?;
    Ok(links)
}

pub fn parse_trace_policy(open_dir: &Path) -> Result<TracePolicy, CoreError> {
    Ok(TracePolicy {
        required_links: parse_verification_policy(open_dir)?.required_links,
    })
}

pub fn parse_verification_policy(open_dir: &Path) -> Result<VerificationPolicy, CoreError> {
    let path = open_dir.join("codefire.policy.yaml");
    let mut policy = default_verification_policy();
    if !path.exists() {
        return Ok(policy);
    }

    let mut required_links = BTreeMap::<String, Vec<RequiredLinkRule>>::new();
    let mut in_required_links = false;
    let mut in_commit_policy = false;
    let mut in_extinguish_policy = false;
    let mut in_no_change_required_policy = false;
    let mut in_verification = false;
    let mut current_kind = None::<String>;
    let mut current_rule = None::<RequiredLinkRuleDraft>;
    let mut current_check = None::<VerificationCommandDraft>;
    let mut checks = Vec::new();

    for raw_line in read_text_lossy(&path)?.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim_end();
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }
        if !raw_line.starts_with([' ', '\t']) {
            if stripped == "required_links:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = true;
                in_commit_policy = false;
                in_verification = false;
                current_kind = None;
            } else if stripped == "commit_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = true;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                current_kind = None;
            } else if stripped == "extinguish_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = true;
                in_no_change_required_policy = false;
                in_verification = false;
                current_kind = None;
            } else if stripped == "verification:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = true;
                current_kind = None;
            } else {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                current_kind = None;
            }
            continue;
        }
        if in_commit_policy {
            if let Some((key, value)) = stripped.split_once(':') {
                set_commit_policy_bool(&mut policy, key.trim(), value.trim())?;
            }
            continue;
        }
        if in_extinguish_policy {
            if line.starts_with("  ") && stripped == "no-change-required:" {
                in_no_change_required_policy = true;
                continue;
            }
            if in_no_change_required_policy {
                if let Some(value) = stripped.strip_prefix("requires_rationale:") {
                    policy.no_change_required_requires_rationale = parse_bool_field(value.trim())?;
                    continue;
                }
            }
        }
        if in_verification {
            if let Some(value) = stripped.strip_prefix("- id:") {
                flush_verification_check(&mut checks, current_check.take())?;
                current_check = Some(VerificationCommandDraft {
                    id: Some(unquote(value.trim())),
                    command: None,
                    cwd: Some(".".to_string()),
                });
                continue;
            }
            if let Some(value) = stripped.strip_prefix("- command:") {
                flush_verification_check(&mut checks, current_check.take())?;
                current_check = Some(VerificationCommandDraft {
                    id: None,
                    command: Some(unquote(value.trim())),
                    cwd: Some(".".to_string()),
                });
                continue;
            }
            let Some(check) = current_check.as_mut() else {
                continue;
            };
            if let Some(value) = stripped.strip_prefix("id:") {
                check.id = Some(unquote(value.trim()));
            } else if let Some(value) = stripped.strip_prefix("command:") {
                check.command = Some(unquote(value.trim()));
            } else if let Some(value) = stripped.strip_prefix("cwd:") {
                check.cwd = Some(unquote(value.trim()));
            }
            continue;
        }
        if !in_required_links {
            continue;
        }
        if line.starts_with("  ") && !line.starts_with("    ") && stripped.ends_with(':') {
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            let kind = stripped.trim_end_matches(':').to_string();
            required_links.entry(kind.clone()).or_default();
            current_kind = Some(kind);
            continue;
        }
        if let Some(value) = stripped.strip_prefix("- type:") {
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            current_rule = Some(RequiredLinkRuleDraft {
                link_type: Some(unquote(value.trim())),
                target_kind: None,
                min: None,
            });
            continue;
        }
        let Some(rule) = current_rule.as_mut() else {
            continue;
        };
        if let Some(value) = stripped.strip_prefix("type:") {
            rule.link_type = Some(unquote(value.trim()));
        } else if let Some(value) = stripped.strip_prefix("target_kind:") {
            rule.target_kind = Some(unquote(value.trim()));
        } else if let Some(value) = stripped.strip_prefix("min:") {
            rule.min = Some(parse_usize_field("required_links min", value.trim())?);
        }
    }
    flush_required_rule(&mut required_links, &current_kind, current_rule)?;
    flush_verification_check(&mut checks, current_check)?;
    if required_links.is_empty() {
        required_links = policy.required_links;
    }
    policy.required_links = required_links;
    policy.verification = checks;
    Ok(policy)
}

fn set_commit_policy_bool(
    policy: &mut VerificationPolicy,
    key: &str,
    value: &str,
) -> Result<(), CoreError> {
    let parsed = parse_bool_field(value)?;
    match key {
        "require_no_required_fires" => policy.require_no_required_fires = parsed,
        "require_no_stale_resolutions" => policy.require_no_stale_resolutions = parsed,
        "require_trace_completeness" => policy.require_trace_completeness = parsed,
        "require_verification_success" => policy.require_verification_success = parsed,
        "reject_duplicate_atom_ids" => policy.reject_duplicate_atom_ids = parsed,
        _ => {}
    }
    Ok(())
}

fn parse_bool_field(value: &str) -> Result<bool, CoreError> {
    match unquote(value).trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(CoreError::Config(format!(
            "invalid boolean value in policy: {value}"
        ))),
    }
}

pub fn default_verification_policy() -> VerificationPolicy {
    VerificationPolicy {
        require_no_required_fires: true,
        require_no_stale_resolutions: true,
        require_trace_completeness: true,
        require_verification_success: true,
        reject_duplicate_atom_ids: true,
        no_change_required_requires_rationale: true,
        required_links: default_trace_policy().required_links,
        verification: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerificationCommandDraft {
    id: Option<String>,
    command: Option<String>,
    cwd: Option<String>,
}

fn flush_verification_check(
    checks: &mut Vec<VerificationCommand>,
    draft: Option<VerificationCommandDraft>,
) -> Result<(), CoreError> {
    let Some(draft) = draft else {
        return Ok(());
    };
    let command = draft.command.ok_or_else(|| {
        CoreError::Config(
            "invalid codefire.policy.yaml: verification entry missing command".to_string(),
        )
    })?;
    let id = draft
        .id
        .unwrap_or_else(|| format!("verification-{}", checks.len() + 1));
    checks.push(VerificationCommand {
        id,
        command,
        cwd: draft.cwd.unwrap_or_else(|| ".".to_string()),
    });
    Ok(())
}

pub fn build_verification(
    scan: &ScanResult,
    missing_required_links: Vec<MissingRequiredLink>,
    failed_checks: Vec<FailedCheck>,
    stale_resolutions: Vec<StaleResolution>,
    missing_evidence_refs: Vec<MissingEvidenceRef>,
    policy: &VerificationPolicy,
    verified_at: &str,
) -> Verification {
    let duplicate_atom_ids = scan.atom_index.duplicate_atom_ids.clone();
    let mut passed = true;
    if policy.require_no_required_fires && !scan.open_fires.is_empty() {
        passed = false;
    }
    if policy.require_trace_completeness && !missing_required_links.is_empty() {
        passed = false;
    }
    if policy.require_no_stale_resolutions && !stale_resolutions.is_empty() {
        passed = false;
    }
    if !missing_evidence_refs.is_empty() {
        passed = false;
    }
    if policy.require_verification_success && !failed_checks.is_empty() {
        passed = false;
    }
    if policy.reject_duplicate_atom_ids && !duplicate_atom_ids.is_empty() {
        passed = false;
    }
    Verification {
        type_tag: "verification".to_string(),
        version: VERSION,
        result: if passed { "passed" } else { "failed" }.to_string(),
        trace_completeness_required: policy.require_trace_completeness,
        open_required_fires: scan.open_fires.len(),
        failed_checks,
        missing_required_links,
        stale_resolutions,
        missing_evidence_refs,
        duplicate_atom_ids,
        verified_at: verified_at.to_string(),
    }
}

pub fn stale_resolutions(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    policy: &VerificationPolicy,
    resolutions: &[Resolution],
) -> Result<Vec<StaleResolution>, CoreError> {
    let atoms = atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom.content_hash.clone()))
        .collect::<BTreeMap<_, _>>();
    let link_hashes = trace_graph
        .links
        .iter()
        .map(|link| (link.link_id.as_str(), link.link_hash.as_str()))
        .collect::<BTreeMap<_, _>>();
    let current_policy_hash = policy_hash(policy)?;
    let mut stale = Vec::new();

    for resolution in resolutions {
        if resolution.status != "active" {
            continue;
        }
        let source_id = resolution.basis.source_atom.atom_id.as_str();
        if atoms.get(source_id).cloned() != resolution.basis.source_atom.content_hash {
            stale.push(StaleResolution {
                resolution_uid: resolution.resolution_uid.clone(),
                reason: "source atom changed".to_string(),
            });
            continue;
        }
        let target_id = resolution.basis.target_atom.atom_id.as_str();
        if atoms.get(target_id).cloned() != resolution.basis.target_atom.content_hash {
            stale.push(StaleResolution {
                resolution_uid: resolution.resolution_uid.clone(),
                reason: "target atom changed".to_string(),
            });
            continue;
        }
        if current_policy_hash != resolution.basis.policy_hash {
            stale.push(StaleResolution {
                resolution_uid: resolution.resolution_uid.clone(),
                reason: "policy changed".to_string(),
            });
            continue;
        }
        if resolution.basis.trace_links.iter().any(|link| {
            link_hashes.get(link.link_id.as_str()).copied() != Some(link.link_hash.as_str())
        }) {
            stale.push(StaleResolution {
                resolution_uid: resolution.resolution_uid.clone(),
                reason: "trace link changed".to_string(),
            });
        }
    }
    Ok(stale)
}

pub fn build_resolution(
    fire: &Fire,
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    policy: &VerificationPolicy,
    request: ResolutionRequest,
) -> Result<Resolution, CoreError> {
    let atoms = atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom.content_hash.clone()))
        .collect::<BTreeMap<_, _>>();
    let source_id = fire.source.atom_id.clone();
    let target_id = fire.target.atom_id.clone();
    let trace_links = trace_graph
        .links
        .iter()
        .filter(|link| {
            (link.from == source_id && link.to == target_id)
                || (link.from == target_id && link.to == source_id)
        })
        .map(|link| ResolutionTraceLinkBasis {
            link_id: link.link_id.clone(),
            link_hash: link.link_hash.clone(),
        })
        .collect::<Vec<_>>();
    Ok(Resolution {
        type_tag: "resolution".to_string(),
        version: VERSION,
        resolution_uid: request.resolution_uid,
        fire_uid: fire.fire_uid.clone(),
        resolution_type: request.resolution_type,
        rationale: request.rationale,
        evidence: request.evidence,
        evidence_refs: request.evidence_refs,
        basis: ResolutionBasis {
            source_atom: ResolutionAtomBasis {
                atom_id: source_id.clone(),
                content_hash: atoms.get(source_id.as_str()).cloned(),
            },
            target_atom: ResolutionAtomBasis {
                atom_id: target_id.clone(),
                content_hash: atoms.get(target_id.as_str()).cloned(),
            },
            trace_links,
            policy_hash: policy_hash(policy)?,
        },
        resolved_at: request.resolved_at,
        status: "active".to_string(),
    })
}

pub fn policy_hash(policy: &VerificationPolicy) -> Result<String, CoreError> {
    Ok(digest_bytes(&serde_json::to_vec(policy)?))
}

pub fn required_link_missing(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    policy: &TracePolicy,
) -> Vec<MissingRequiredLink> {
    let atoms = atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect::<BTreeMap<_, _>>();
    let mut missing = Vec::new();
    for atom in &atom_index.atoms {
        let Some(rules) = policy.required_links.get(&atom.kind) else {
            continue;
        };
        for rule in rules {
            let found = trace_graph
                .links
                .iter()
                .filter(|link| {
                    link.from == atom.atom_id
                        && link.link_type == rule.link_type
                        && atoms
                            .get(link.to.as_str())
                            .is_some_and(|target| target.kind == rule.target_kind)
                })
                .count();
            if found < rule.min {
                missing.push(MissingRequiredLink {
                    atom_id: atom.atom_id.clone(),
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

pub fn current_required_link_missing(
    open_dir: &Path,
) -> Result<Vec<MissingRequiredLink>, CoreError> {
    let atom_index = build_atom_index(open_dir)?;
    let trace_graph = current_trace_graph(open_dir)?;
    let policy = parse_trace_policy(open_dir)?;
    Ok(required_link_missing(&atom_index, &trace_graph, &policy))
}

pub fn changed_atoms(current: &AtomIndex, base: &AtomIndex) -> Vec<String> {
    let old = base
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom.content_hash.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut changed = current
        .atoms
        .iter()
        .filter_map(|atom| {
            (old.get(atom.atom_id.as_str()).copied() != Some(atom.content_hash.as_str()))
                .then_some(atom.atom_id.clone())
        })
        .collect::<Vec<_>>();
    changed.sort();
    changed
}

pub fn build_scan_result(
    current: AtomIndex,
    base_index: AtomIndex,
    trace_graph: TraceGraph,
    mut fires: Vec<Fire>,
    base_commit: String,
    reason: &str,
    now: &str,
) -> Result<(ScanResult, Vec<Fire>), CoreError> {
    let changed = changed_atoms(&current, &base_index);
    let current_atoms = current
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom))
        .collect::<BTreeMap<_, _>>();
    let mut existing_keys = fires
        .iter()
        .filter(|fire| fire.status != "obsolete")
        .map(|fire| fire.key.clone())
        .collect::<HashSet<_>>();
    for source in &changed {
        for (target, trace_path) in adjacent_atoms(source, &trace_graph) {
            if target == *source {
                continue;
            }
            let key = fire_key(source, &target, reason, &trace_path, &base_commit)?;
            if !existing_keys.insert(key.clone()) {
                continue;
            }
            let source_hash = current_atoms
                .get(source.as_str())
                .map(|atom| atom.content_hash.clone());
            let target_hash = current_atoms
                .get(target.as_str())
                .map(|atom| atom.content_hash.clone());
            let fire_digest = fire_digest_hex(&key);
            fires.push(Fire {
                type_tag: "fire".to_string(),
                version: VERSION,
                fire_uid: fire_uid_from_digest(&fire_digest),
                display_id: fire_display_id_from_digest(&fire_digest),
                status: "open".to_string(),
                severity: "required".to_string(),
                source: FireAtomRef {
                    atom_id: source.clone(),
                    content_hash_at_fire: source_hash,
                },
                target: FireAtomRef {
                    atom_id: target,
                    content_hash_at_fire: target_hash,
                },
                reason: reason.to_string(),
                trace_path,
                created_by: "scan".to_string(),
                created_at: now.to_string(),
                key,
                obsolete_at: None,
                resolution_uid: None,
            });
        }
    }

    let changed_set = changed.iter().map(String::as_str).collect::<HashSet<_>>();
    for fire in &mut fires {
        if fire.created_by == "scan"
            && fire.status == "open"
            && !changed_set.contains(fire.source.atom_id.as_str())
        {
            fire.status = "obsolete".to_string();
            fire.obsolete_at = Some(now.to_string());
        }
    }

    let open_fires = fires
        .iter()
        .filter(|fire| fire.status == "open" && fire.severity == "required")
        .cloned()
        .collect::<Vec<_>>();
    let scan = ScanResult {
        type_tag: "scan".to_string(),
        version: VERSION,
        base_commit,
        atom_index: current,
        trace_graph,
        changed_atoms: changed,
        open_fires,
    };
    Ok((scan, fires))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceLinkInput {
    pub from: String,
    pub to: Option<String>,
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequiredLinkRuleDraft {
    link_type: Option<String>,
    target_kind: Option<String>,
    min: Option<usize>,
}

impl TraceLinkInput {
    fn into_trace_link(self) -> Result<TraceLink, CoreError> {
        let to = self.to.ok_or_else(|| {
            CoreError::Config("invalid codefire.links.yaml: link missing to".to_string())
        })?;
        let link_type = self.link_type.ok_or_else(|| {
            CoreError::Config("invalid codefire.links.yaml: link missing type".to_string())
        })?;
        let link_id = format!("LINK-{}-{}-{}", self.from, link_type, to);
        let link_hash = trace_link_hash(&self.from, &to, &link_type, &link_id)?;
        Ok(TraceLink {
            from: self.from,
            to,
            link_type,
            link_id,
            link_hash,
        })
    }
}

fn flush_link(
    links: &mut Vec<TraceLinkInput>,
    current: Option<TraceLinkInput>,
    current_line: usize,
) -> Result<(), CoreError> {
    let Some(link) = current else {
        return Ok(());
    };
    let mut missing = Vec::new();
    if link.from.is_empty() {
        missing.push("from");
    }
    if link.to.is_none() {
        missing.push("to");
    }
    if link.link_type.is_none() {
        missing.push("type");
    }
    if !missing.is_empty() {
        return Err(CoreError::Config(format!(
            "invalid codefire.links.yaml: link at line {current_line} missing {}",
            missing.join(", ")
        )));
    }
    links.push(link);
    Ok(())
}

fn default_trace_policy() -> TracePolicy {
    TracePolicy {
        required_links: BTreeMap::from([
            (
                "requirement".to_string(),
                vec![
                    RequiredLinkRule {
                        link_type: "refined_by".to_string(),
                        target_kind: "design".to_string(),
                        min: 1,
                    },
                    RequiredLinkRule {
                        link_type: "verified_by".to_string(),
                        target_kind: "test".to_string(),
                        min: 1,
                    },
                ],
            ),
            (
                "design".to_string(),
                vec![RequiredLinkRule {
                    link_type: "implemented_by".to_string(),
                    target_kind: "code".to_string(),
                    min: 1,
                }],
            ),
        ]),
    }
}

fn flush_required_rule(
    required_links: &mut BTreeMap<String, Vec<RequiredLinkRule>>,
    current_kind: &Option<String>,
    draft: Option<RequiredLinkRuleDraft>,
) -> Result<(), CoreError> {
    let Some(draft) = draft else {
        return Ok(());
    };
    let Some(kind) = current_kind else {
        return Err(CoreError::Config(
            "invalid codefire.policy.yaml: required link rule has no source kind".to_string(),
        ));
    };
    let link_type = draft.link_type.ok_or_else(|| {
        CoreError::Config(format!(
            "invalid codefire.policy.yaml: required_links.{kind} missing type"
        ))
    })?;
    let target_kind = draft.target_kind.ok_or_else(|| {
        CoreError::Config(format!(
            "invalid codefire.policy.yaml: required_links.{kind} missing target_kind"
        ))
    })?;
    required_links
        .entry(kind.clone())
        .or_default()
        .push(RequiredLinkRule {
            link_type,
            target_kind,
            min: draft.min.unwrap_or(1),
        });
    Ok(())
}

fn parse_usize_field(field: &str, value: &str) -> Result<usize, CoreError> {
    unquote(value).parse::<usize>().map_err(|_| {
        CoreError::Config(format!(
            "invalid codefire.policy.yaml: {field} must be an integer"
        ))
    })
}

fn trace_link_hash(
    from: &str,
    to: &str,
    link_type: &str,
    link_id: &str,
) -> Result<String, CoreError> {
    let value = serde_json::json!({
        "from": from,
        "link_id": link_id,
        "to": to,
        "type": link_type,
    });
    Ok(digest_bytes(&serde_json::to_vec(&value)?))
}

fn adjacent_atoms(atom_id: &str, trace_graph: &TraceGraph) -> Vec<(String, Vec<String>)> {
    let mut adjacent = Vec::new();
    for link in &trace_graph.links {
        if link.from == atom_id {
            adjacent.push((link.to.clone(), vec![link.from.clone(), link.to.clone()]));
        } else if link.to == atom_id {
            adjacent.push((link.from.clone(), vec![link.to.clone(), link.from.clone()]));
        }
    }
    adjacent
}

fn fire_key(
    source: &str,
    target: &str,
    reason: &str,
    trace_path: &[String],
    base_commit: &str,
) -> Result<String, CoreError> {
    let trace_path_hash = digest_bytes(&serde_json::to_vec(trace_path)?);
    let value = serde_json::json!({
        "source_atom_id": source,
        "target_atom_id": target,
        "reason": reason,
        "trace_path_hash": trace_path_hash,
        "base_commit": base_commit,
    });
    Ok(digest_bytes(&serde_json::to_vec(&value)?))
}

pub fn extract_markdown_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut headings = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some((level, atom_id)) = markdown_atom_heading(line) {
            headings.push((line_index, level, atom_id));
        }
    }

    let mut atoms = Vec::with_capacity(headings.len());
    for (index, (start, level, atom_id)) in headings.iter().enumerate() {
        let mut end = lines.len();
        for (next_start, next_level, _) in &headings[index + 1..] {
            if next_level <= level {
                end = *next_start;
                break;
            }
        }
        let content = normalized_block_content(&lines[*start..end]);
        atoms.push(Atom {
            atom_id: atom_id.clone(),
            kind: kind_from_atom_id(atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "markdown_heading".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
        });
    }
    Ok(atoms)
}

pub fn extract_explicit_cf_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut atoms = Vec::new();
    for line in &lines {
        if let Some(atom_id) = explicit_cf_atom_id(line) {
            let content = format!("{}\n", line.trim());
            atoms.push(Atom {
                atom_id: atom_id.clone(),
                kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
                artifact_path: artifact_path.to_string(),
                selector: Selector {
                    selector_type: "explicit_cf_atom".to_string(),
                    value: atom_id,
                },
                content_hash: hash_text(&content),
            });
        }
    }
    Ok(atoms)
}

pub fn kind_from_atom_id(atom_id: &str, fallback: &str) -> &'static str {
    if atom_id.starts_with("REQ-") {
        "requirement"
    } else if atom_id.starts_with("DES-") {
        "design"
    } else if atom_id.starts_with("TEST-") {
        "test"
    } else if atom_id.starts_with("CODE-") || atom_id.starts_with("CODE:") {
        "code"
    } else if atom_id.starts_with("ADR-") {
        "adr"
    } else if atom_id.starts_with("OPS-") {
        "ops"
    } else if atom_id.starts_with("API-") {
        "api"
    } else if atom_id.starts_with("DB-") {
        "db"
    } else {
        match fallback {
            "requirement" => "requirement",
            "design" => "design",
            "test" => "test",
            "code" => "code",
            "adr" => "adr",
            "ops" => "ops",
            "api" => "api",
            "db" => "db",
            _ => "unknown",
        }
    }
}

fn markdown_atom_heading(line: &str) -> Option<(usize, String)> {
    let mut chars = line.chars().peekable();
    let mut level = 0usize;
    while chars.peek() == Some(&'#') && level < 6 {
        chars.next();
        level += 1;
    }
    if level == 0 || chars.next() != Some(' ') {
        return None;
    }
    let rest: String = chars.collect();
    let atom_id: String = rest
        .chars()
        .take_while(|ch| ch.is_ascii_uppercase() || is_atom_tail_char(*ch))
        .collect();
    if !is_valid_heading_atom_id(&atom_id) {
        return None;
    }
    let next = rest.chars().nth(atom_id.len());
    if matches!(next, None | Some(':') | Some(' ') | Some('\t')) {
        Some((level, atom_id))
    } else {
        None
    }
}

fn is_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')
}

fn is_valid_heading_atom_id(atom_id: &str) -> bool {
    let Some((prefix, rest)) = atom_id.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_uppercase())
        && !rest.is_empty()
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

fn explicit_cf_atom_id(line: &str) -> Option<String> {
    let (_, after) = line.split_once("cf-atom:")?;
    let atom_id: String = after
        .trim_start()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-'))
        .collect();
    (!atom_id.is_empty()).then_some(atom_id)
}

fn normalized_block_content(lines: &[&str]) -> String {
    let joined = lines.join("\n");
    let trimmed = joined.trim();
    format!("{trimmed}\n")
}

fn hash_text(text: &str) -> String {
    digest_bytes(text.as_bytes())
}

fn digest_bytes(bytes: &[u8]) -> String {
    codefire_util::sha256_prefixed(bytes)
}

const FIRE_UID_HEX_LENGTH: usize = 32;
const FIRE_DISPLAY_HEX_LENGTH: usize = 12;

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

fn read_text_lossy(path: &Path) -> Result<String, CoreError> {
    Ok(String::from_utf8_lossy(&fs::read(path)?).into_owned())
}

fn split_lines_without_terminators(text: &str) -> Vec<&str> {
    text.lines().collect()
}

fn rel_files(open_dir: &Path) -> Result<Vec<PathBuf>, CoreError> {
    let mut files = Vec::new();
    collect_rel_files(open_dir, Path::new(""), &mut files)?;
    files.sort_by_key(|path| to_posix(path));
    Ok(files)
}

fn collect_rel_files(
    root: &Path,
    rel_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), CoreError> {
    let dir = root.join(rel_dir);
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rel = rel_dir.join(name.as_ref());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if matches!(name.as_ref(), ".codefire" | "__pycache__" | ".pytest_cache") {
                continue;
            }
            collect_rel_files(root, &rel, files)?;
        } else if file_type.is_file() && to_posix(&rel) != ".codefire-open" {
            files.push(rel);
        }
    }
    Ok(())
}

fn to_posix(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[derive(Debug, Clone)]
struct Config {
    patterns: HashMap<&'static str, Vec<String>>,
}

impl Config {
    fn parse(open_dir: &Path) -> Result<Self, CoreError> {
        let defaults = default_config_patterns();
        let path = open_dir.join("codefire.yaml");
        if !path.exists() {
            return Ok(Self { patterns: defaults });
        }

        let mut parsed: HashMap<&'static str, Vec<String>> =
            defaults.keys().map(|key| (*key, Vec::new())).collect();
        let mut current = None;
        for (line_index, raw_line) in read_text_lossy(&path)?.lines().enumerate() {
            let line_without_comment = raw_line.split('#').next().unwrap_or_default();
            let stripped = line_without_comment.trim();
            if stripped.is_empty() {
                continue;
            }
            if !raw_line.starts_with([' ', '\t']) {
                current = None;
            }
            if let Some(key) = config_section_key(stripped) {
                current = Some(key);
                continue;
            }
            if let Some(section) = current {
                if let Some(value) = stripped.strip_prefix("- path:") {
                    let value = unquote(value.trim());
                    if value.is_empty() {
                        return Err(CoreError::Config(format!(
                            "invalid codefire.yaml: empty path at line {}",
                            line_index + 1
                        )));
                    }
                    parsed.get_mut(section).expect("known section").push(value);
                    continue;
                }
                if stripped.starts_with("- ") {
                    return Err(CoreError::Config(format!(
                        "invalid codefire.yaml: artifact entry at line {} must start with 'path:'",
                        line_index + 1
                    )));
                }
            }
        }
        for (key, value) in defaults {
            if parsed.get(key).is_some_and(Vec::is_empty) {
                parsed.insert(key, value);
            }
        }
        Ok(Self { patterns: parsed })
    }

    fn matches(&self, section: &'static str, rel: &str) -> bool {
        self.patterns
            .get(section)
            .is_some_and(|patterns| patterns.iter().any(|pattern| path_matches(pattern, rel)))
    }
}

fn default_config_patterns() -> HashMap<&'static str, Vec<String>> {
    HashMap::from([
        (
            "requirements",
            vec![
                "docs/spec/**/*.md".to_string(),
                "docs/requirements/**/*.md".to_string(),
            ],
        ),
        ("designs", vec!["docs/design/**/*.md".to_string()]),
        (
            "adrs",
            vec![
                "adr/**/*.md".to_string(),
                "docs/adr/**/*.md".to_string(),
                "docs/adrs/**/*.md".to_string(),
            ],
        ),
        (
            "ops",
            vec![
                "docs/ops/**/*.md".to_string(),
                "docs/runbooks/**/*.md".to_string(),
                "runbooks/**/*.md".to_string(),
            ],
        ),
        ("apis", Vec::new()),
        ("db", Vec::new()),
        ("code", vec!["src/**/*.py".to_string()]),
        ("tests", vec!["tests/**/*.py".to_string()]),
    ])
}

fn config_section_key(stripped: &str) -> Option<&'static str> {
    let section = stripped.strip_suffix(':')?;
    match section {
        "requirements" => Some("requirements"),
        "designs" => Some("designs"),
        "adrs" => Some("adrs"),
        "ops" => Some("ops"),
        "apis" => Some("apis"),
        "db" => Some("db"),
        "code" => Some("code"),
        "tests" => Some("tests"),
        _ => None,
    }
}

fn path_matches(pattern: &str, rel: &str) -> bool {
    wildcard_match(pattern, rel)
        || pattern.split_once("**/").is_some_and(|(prefix, suffix)| {
            rel.starts_with(prefix) && wildcard_match(suffix, &rel[prefix.len()..])
                || wildcard_match(&format!("{prefix}{suffix}"), rel)
        })
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut p, mut t) = (0usize, 0usize);
    let mut star = None;
    let mut star_text = 0usize;
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == text[t] || pattern[p] == b'?') {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_text = t;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            star_text += 1;
            t = star_text;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn markdown_atoms_match_heading_scope_and_python_hash_format() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("spec.md");
        fs::write(
            &path,
            "# Intro\n\n## REQ-AUTH-001: Session expiration\n\nBody\n\n### DES-NESTED-IGNORED: detail\n\nNested\n\n## REQ-AUTH-002 Next\nTail\n",
        )
        .unwrap();

        let atoms = extract_markdown_atoms(&path, "docs/spec/auth.md", "requirement").unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms[0].atom_id, "REQ-AUTH-001");
        assert_eq!(atoms[0].kind, "requirement");
        assert_eq!(atoms[0].selector.selector_type, "markdown_heading");
        assert_eq!(
            atoms[0].content_hash,
            "sha256:c0bf4a8b5b28a4f811bea3dad3ed81c94931b3b93b7b300eed250531f09aab10"
        );
        assert_eq!(atoms[1].atom_id, "DES-NESTED-IGNORED");
        assert_eq!(atoms[1].kind, "design");
        assert_eq!(atoms[2].atom_id, "REQ-AUTH-002");
    }

    #[test]
    fn explicit_cf_atoms_extract_comment_markers_without_symbol_parsing() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("app.py");
        fs::write(
            &path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n",
        )
        .unwrap();

        let atoms = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].atom_id, "CODE-SessionPolicy");
        assert_eq!(atoms[0].kind, "code");
        assert_eq!(atoms[0].selector.selector_type, "explicit_cf_atom");
        assert_eq!(
            atoms[0].content_hash,
            "sha256:e77db9516bb440255ebb1177be2f1aa3b5fcba7f46e43dd86871783fdf97cee4"
        );
    }

    #[test]
    fn build_atom_index_uses_defaults_config_and_detects_duplicates() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("docs/spec/auth.md"),
            "## REQ-AUTH-001: Session expiration\n\nRequirement.\n",
        );
        write_file(
            &temp.path().join("docs/design/auth.md"),
            "## DES-AUTH-001: Session design\n\nDesign.\n",
        );
        write_file(
            &temp.path().join("src/app.py"),
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n",
        );
        write_file(
            &temp.path().join("tests/test_app.py"),
            "# cf-atom: TEST-session-policy\ndef test_session_policy():\n    pass\n",
        );
        write_file(
            &temp.path().join("src/duplicate.py"),
            "# cf-atom: CODE-SessionPolicy\n",
        );

        let index = build_atom_index(temp.path()).unwrap();
        let ids = index
            .atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(index.type_tag, "atom_index");
        assert_eq!(
            ids,
            vec![
                "CODE-SessionPolicy",
                "CODE-SessionPolicy",
                "DES-AUTH-001",
                "REQ-AUTH-001",
                "TEST-session-policy",
            ]
        );
        assert_eq!(index.duplicate_atom_ids, vec!["CODE-SessionPolicy"]);
    }

    #[test]
    fn codefire_yaml_overrides_artifact_patterns() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.yaml"),
            "requirements:\n  - path: product/*.md\ncode:\n  - path: lib/*.rs\n",
        );
        write_file(
            &temp.path().join("product/auth.md"),
            "## REQ-CUSTOM-001: Custom requirement\n",
        );
        write_file(
            &temp.path().join("lib/main.rs"),
            "// cf-atom: CODE-RustMain\n",
        );

        let index = build_atom_index(temp.path()).unwrap();
        let ids = index
            .atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["CODE-RustMain", "REQ-CUSTOM-001"]);
    }

    #[test]
    fn trace_graph_parses_links_and_matches_python_hash() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.links.yaml"),
            "version: 1\nlinks:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n    type: refined_by\n",
        );

        let graph = current_trace_graph(temp.path()).unwrap();

        assert_eq!(graph.type_tag, "trace_graph");
        assert_eq!(graph.links.len(), 1);
        assert_eq!(graph.links[0].from, "REQ-AUTH-001");
        assert_eq!(graph.links[0].to, "DES-AUTH-001");
        assert_eq!(graph.links[0].link_type, "refined_by");
        assert_eq!(
            graph.links[0].link_id,
            "LINK-REQ-AUTH-001-refined_by-DES-AUTH-001"
        );
        assert_eq!(
            graph.links[0].link_hash,
            "sha256:54748236f430e7b4e61322f644932c04e96e75a7d174a1a993aeacd98f08bb89"
        );
    }

    #[test]
    fn trace_graph_reports_missing_required_link_fields() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.links.yaml"),
            "links:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n",
        );

        let error = current_trace_graph(temp.path()).unwrap_err();

        assert!(matches!(error, CoreError::Config(message) if message.contains("missing type")));
    }

    #[test]
    fn trace_graph_is_empty_when_links_file_is_absent() {
        let temp = tempdir().unwrap();

        let graph = current_trace_graph(temp.path()).unwrap();

        assert!(graph.links.is_empty());
    }

    #[test]
    fn required_link_missing_uses_default_policy() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("docs/spec/auth.md"),
            "## REQ-AUTH-001: Session expiration\n",
        );
        write_file(
            &temp.path().join("docs/design/auth.md"),
            "## DES-AUTH-001: Session design\n",
        );
        write_file(
            &temp.path().join("src/app.py"),
            "# cf-atom: CODE-SessionPolicy\n",
        );
        write_file(
            &temp.path().join("codefire.links.yaml"),
            "links:\n  - from: REQ-AUTH-001\n    to: DES-AUTH-001\n    type: refined_by\n",
        );

        let missing = current_required_link_missing(temp.path()).unwrap();

        assert_eq!(missing.len(), 2);
        assert!(missing.iter().any(|item| item.atom_id == "REQ-AUTH-001"
            && item.required_type == "verified_by"
            && item.target_kind == "test"));
        assert!(missing.iter().any(|item| item.atom_id == "DES-AUTH-001"
            && item.required_type == "implemented_by"
            && item.target_kind == "code"));
    }

    #[test]
    fn required_link_missing_uses_custom_policy() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("docs/adr/platform.md"),
            "## ADR-PLATFORM-001: Runtime choice\n",
        );
        write_file(
            &temp.path().join("docs/ops/deploy.md"),
            "## OPS-DEPLOY-001: Runtime deployment\n",
        );
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "required_links:\n  adr:\n    - type: operated_by\n      target_kind: ops\n      min: 1\n",
        );

        let missing = current_required_link_missing(temp.path()).unwrap();

        assert_eq!(
            missing,
            vec![MissingRequiredLink {
                atom_id: "ADR-PLATFORM-001".to_string(),
                required_type: "operated_by".to_string(),
                target_kind: "ops".to_string(),
                min: 1,
                found: 0,
            }]
        );
    }

    #[test]
    fn verification_policy_parses_commit_booleans_and_commands() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "commit_policy:\n  require_no_required_fires: false\n  reject_duplicate_atom_ids: false\nverification:\n  - id: unit\n    command: cargo test\n    cwd: crates/app\n",
        );

        let policy = parse_verification_policy(temp.path()).unwrap();

        assert!(!policy.require_no_required_fires);
        assert!(!policy.reject_duplicate_atom_ids);
        assert!(policy.require_trace_completeness);
        assert_eq!(
            policy.verification,
            vec![VerificationCommand {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                cwd: "crates/app".to_string(),
            }]
        );
    }

    #[test]
    fn verification_result_respects_policy_booleans() {
        let scan = ScanResult {
            type_tag: "scan".to_string(),
            version: VERSION,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: AtomIndex {
                type_tag: "atom_index".to_string(),
                version: VERSION,
                atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req")],
                duplicate_atom_ids: vec!["REQ-AUTH-001".to_string()],
            },
            trace_graph: TraceGraph {
                type_tag: "trace_graph".to_string(),
                version: VERSION,
                links: Vec::new(),
            },
            changed_atoms: Vec::new(),
            open_fires: Vec::new(),
        };
        let policy = VerificationPolicy {
            reject_duplicate_atom_ids: false,
            ..default_verification_policy()
        };

        let verification = build_verification(
            &scan,
            vec![MissingRequiredLink {
                atom_id: "REQ-AUTH-001".to_string(),
                required_type: "verified_by".to_string(),
                target_kind: "test".to_string(),
                min: 1,
                found: 0,
            }],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            &policy,
            "2026-06-04T00:00:00Z",
        );

        assert_eq!(verification.result, "failed");
        assert_eq!(verification.duplicate_atom_ids, vec!["REQ-AUTH-001"]);
        assert_eq!(verification.missing_required_links.len(), 1);
    }

    #[test]
    fn stale_resolutions_detect_atom_policy_and_trace_changes() {
        let policy = default_verification_policy();
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req1"),
                atom("DES-AUTH-001", "design", "sha256:des1"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let trace_graph = TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: vec![TraceLinkInput {
                from: "REQ-AUTH-001".to_string(),
                to: Some("DES-AUTH-001".to_string()),
                link_type: Some("refined_by".to_string()),
            }
            .into_trace_link()
            .unwrap()],
        };
        let fire = Fire {
            type_tag: "fire".to_string(),
            version: VERSION,
            fire_uid: "fire_001".to_string(),
            display_id: "FIRE-001".to_string(),
            status: "extinguished".to_string(),
            severity: "required".to_string(),
            source: FireAtomRef {
                atom_id: "REQ-AUTH-001".to_string(),
                content_hash_at_fire: Some("sha256:req1".to_string()),
            },
            target: FireAtomRef {
                atom_id: "DES-AUTH-001".to_string(),
                content_hash_at_fire: Some("sha256:des1".to_string()),
            },
            reason: "atom_changed".to_string(),
            trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
            created_by: "scan".to_string(),
            created_at: "2026-06-04T00:00:00Z".to_string(),
            key: "sha256:key".to_string(),
            obsolete_at: None,
            resolution_uid: Some("res_001".to_string()),
        };
        let resolution = build_resolution(
            &fire,
            &atom_index,
            &trace_graph,
            &policy,
            ResolutionRequest {
                resolution_uid: "res_001".to_string(),
                resolution_type: "addressed".to_string(),
                rationale: "checked".to_string(),
                evidence: String::new(),
                evidence_refs: Vec::new(),
                resolved_at: "2026-06-04T00:00:00Z".to_string(),
            },
        )
        .unwrap();
        assert!(stale_resolutions(
            &atom_index,
            &trace_graph,
            &policy,
            std::slice::from_ref(&resolution),
        )
        .unwrap()
        .is_empty());

        let changed_index = AtomIndex {
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
                atom("DES-AUTH-001", "design", "sha256:des1"),
            ],
            ..atom_index
        };
        let stale = stale_resolutions(
            &changed_index,
            &trace_graph,
            &policy,
            std::slice::from_ref(&resolution),
        )
        .unwrap();

        assert_eq!(
            stale,
            vec![StaleResolution {
                resolution_uid: "res_001".to_string(),
                reason: "source atom changed".to_string(),
            }]
        );
    }

    #[test]
    fn scan_result_detects_changed_atoms_and_creates_trace_fires() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![
                atom("DES-AUTH-001", "design", "sha256:design2"),
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
            duplicate_atom_ids: Vec::new(),
        };
        let trace_graph = TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: vec![TraceLinkInput {
                from: "REQ-AUTH-001".to_string(),
                to: Some("DES-AUTH-001".to_string()),
                link_type: Some("refined_by".to_string()),
            }
            .into_trace_link()
            .unwrap()],
        };

        let (scan, fires) = build_scan_result(
            current,
            base,
            trace_graph,
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            "atom_changed",
            "2026-06-04T00:00:00Z",
        )
        .unwrap();

        assert_eq!(scan.changed_atoms, vec!["DES-AUTH-001", "REQ-AUTH-001"]);
        assert_eq!(scan.open_fires.len(), 2);
        assert!(fires[0].fire_uid.starts_with("fire_sha256_"));
        assert_eq!(fires[0].fire_uid.len(), "fire_sha256_".len() + 32);
        assert!(fires[0].display_id.starts_with("FIRE-"));
        assert_eq!(fires[0].display_id.len(), "FIRE-".len() + 12);
        assert_eq!(fires[0].status, "open");
        assert_eq!(fires[0].severity, "required");
        assert_eq!(fires[0].created_by, "scan");
    }

    #[test]
    fn scan_result_uses_stable_sha256_fire_identity() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
                atom("DES-AUTH-001", "design", "sha256:design2"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
            duplicate_atom_ids: Vec::new(),
        };
        let trace_graph = TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: vec![TraceLinkInput {
                from: "REQ-AUTH-001".to_string(),
                to: Some("DES-AUTH-001".to_string()),
                link_type: Some("refined_by".to_string()),
            }
            .into_trace_link()
            .unwrap()],
        };

        let first = build_scan_result(
            current.clone(),
            base.clone(),
            trace_graph.clone(),
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            "atom_changed",
            "2026-06-04T00:00:00Z",
        )
        .unwrap()
        .1;
        let with_obsolete = vec![Fire {
            status: "obsolete".to_string(),
            obsolete_at: Some("2026-06-05T00:00:00Z".to_string()),
            ..first[0].clone()
        }];
        let second = build_scan_result(
            current,
            base,
            trace_graph,
            with_obsolete,
            "CF-COMMIT-base".to_string(),
            "atom_changed",
            "2026-06-06T00:00:00Z",
        )
        .unwrap()
        .1;

        assert_eq!(first[0].fire_uid, second[1].fire_uid);
        assert_eq!(first[0].display_id, second[1].display_id);
        assert!(first[0].fire_uid.starts_with("fire_sha256_"));
    }

    #[test]
    fn scan_result_obsoletes_scan_fires_when_source_is_no_longer_changed() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            atoms: vec![atom("REQ-AUTH-001", "requirement", "sha256:req1")],
            duplicate_atom_ids: Vec::new(),
        };
        let base = current.clone();
        let existing = Fire {
            type_tag: "fire".to_string(),
            version: VERSION,
            fire_uid: "fire_existing".to_string(),
            display_id: "FIRE-001".to_string(),
            status: "open".to_string(),
            severity: "required".to_string(),
            source: FireAtomRef {
                atom_id: "REQ-AUTH-001".to_string(),
                content_hash_at_fire: Some("sha256:req1".to_string()),
            },
            target: FireAtomRef {
                atom_id: "DES-AUTH-001".to_string(),
                content_hash_at_fire: None,
            },
            reason: "atom_changed".to_string(),
            trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
            created_by: "scan".to_string(),
            created_at: "2026-06-04T00:00:00Z".to_string(),
            key: "sha256:key".to_string(),
            obsolete_at: None,
            resolution_uid: None,
        };

        let (scan, fires) = build_scan_result(
            current,
            base,
            TraceGraph {
                type_tag: "trace_graph".to_string(),
                version: VERSION,
                links: Vec::new(),
            },
            vec![existing],
            "CF-COMMIT-base".to_string(),
            "atom_changed",
            "2026-06-04T00:01:00Z",
        )
        .unwrap();

        assert!(scan.changed_atoms.is_empty());
        assert!(scan.open_fires.is_empty());
        assert_eq!(fires[0].status, "obsolete");
        assert_eq!(
            fires[0].obsolete_at.as_deref(),
            Some("2026-06-04T00:01:00Z")
        );
    }

    fn write_file(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn atom(atom_id: &str, kind: &str, content_hash: &str) -> Atom {
        Atom {
            atom_id: atom_id.to_string(),
            kind: kind.to_string(),
            artifact_path: "artifact".to_string(),
            selector: Selector {
                selector_type: "test".to_string(),
                value: atom_id.to_string(),
            },
            content_hash: content_hash.to_string(),
        }
    }
}
