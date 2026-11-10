use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

mod atom_extraction;
mod change_coverage;
mod effective_policy;
mod policy_parsing;
mod text_merge;
mod trace_evaluation;
mod work_budget;

pub use change_coverage::{
    build_change_coverage, validate_change_coverage, ChangeCoverage, ChangeCoverageEntry,
    ChangeCoverageRelation, ChangeCoverageSummary, ManifestFileChange,
};
pub use effective_policy::{
    effective_policy, effective_policy_matches_policy, validate_effective_policy, EffectivePolicy,
    EffectivePolicyRule, PolicyProvenance, PolicyRuleSource, PolicySafetyClass,
};
pub use text_merge::{merge_text_three_way, TextMerge};
pub use trace_evaluation::{TraceEvaluationIndex, TraceWorkCounters};
pub use work_budget::{
    logical_retained_frame_bytes, DecisionWorkAccountingError, DecisionWorkBudgetExceeded,
    DecisionWorkCommandKind, DecisionWorkCounter, DecisionWorkDimension, DecisionWorkLedger,
    DecisionWorkLedgerIdentity, DecisionWorkLimit, DecisionWorkMeasurement, DecisionWorkPermit,
    DecisionWorkPhase, DecisionWorkProfile, DecisionWorkReservation, DecisionWorkSnapshot,
    DecisionWorkSurface, DecisionWorkTerminalReason, DECISION_WORK_PROFILE_V1,
    HTTP_REQUEST_DEADLINE_MS, LOGICAL_FRAME_BASE_BYTES, RESOURCE_BUDGET_EXIT_CODE,
};

pub const VERSION: u32 = 1;
pub const ATOM_HASH_SCHEMA_VERSION: u32 = 4;
pub const DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS: u64 = 300_000;
pub const DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES: usize = 64 * 1024;
pub const TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT: usize = 100;
pub const TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT: usize = 256;

pub use atom_extraction::{
    extract_explicit_cf_atoms, extract_markdown_atoms, extract_python_atoms, is_valid_atom_id,
    kind_from_atom_id,
};
use policy_parsing::limited_yaml_key;
use policy_parsing::limited_yaml_line;
pub use policy_parsing::{
    default_verification_policy, parse_effective_verification_policy, parse_verification_policy,
};
fn legacy_atom_hash_schema_version() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

fn default_verification_command_timeout_ms() -> u64 {
    DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS
}

fn default_verification_command_max_output_bytes() -> usize {
    DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES
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
    TraceGraphEndpointIntegrity(TraceGraphEndpointError),
    SealedSnapshotSemanticIntegrity(SealedSnapshotSemanticError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io(error) => write!(f, "{error}"),
            CoreError::Json(error) => write!(f, "{error}"),
            CoreError::Config(message) => write!(f, "{message}"),
            CoreError::TraceGraphEndpointIntegrity(error) => write!(f, "{error}"),
            CoreError::SealedSnapshotSemanticIntegrity(error) => write!(f, "{error}"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceGraphEndpointRole {
    From,
    To,
}

impl fmt::Display for TraceGraphEndpointRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceGraphEndpointRole::From => write!(f, "from"),
            TraceGraphEndpointRole::To => write!(f, "to"),
        }
    }
}

impl TraceGraphEndpointRole {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceGraphEndpointRole::From => "from",
            TraceGraphEndpointRole::To => "to",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceGraphEndpointIssueKind {
    MissingAtom,
    AmbiguousAtomId,
}

impl TraceGraphEndpointIssueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceGraphEndpointIssueKind::MissingAtom => "missing_atom",
            TraceGraphEndpointIssueKind::AmbiguousAtomId => "ambiguous_atom_id",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceGraphEndpointIssue {
    pub link_index: usize,
    pub link_id: String,
    pub link_id_truncated: bool,
    pub link_id_omitted_bytes: usize,
    pub endpoint: TraceGraphEndpointRole,
    pub atom_id: String,
    pub atom_id_truncated: bool,
    pub atom_id_omitted_bytes: usize,
    pub kind: TraceGraphEndpointIssueKind,
    pub atom_occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceGraphEndpointError {
    pub issues: Vec<TraceGraphEndpointIssue>,
    pub total_issues: usize,
    pub omitted_issues: usize,
}

impl fmt::Display for TraceGraphEndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "trace graph endpoint integrity failed: {} issue(s)",
            self.total_issues
        )?;
        let displayed_issue_count = self.issues.len().min(TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT);
        for issue in self.issues.iter().take(displayed_issue_count) {
            let link_id = bounded_endpoint_display_sample(
                &issue.link_id,
                issue.link_id_truncated || issue.link_id_omitted_bytes > 0,
            );
            let atom_id = bounded_endpoint_display_sample(
                &issue.atom_id,
                issue.atom_id_truncated || issue.atom_id_omitted_bytes > 0,
            );
            write!(
                f,
                "; link {} {} endpoint {} ",
                link_id, issue.endpoint, atom_id
            )?;
            match issue.kind {
                TraceGraphEndpointIssueKind::MissingAtom => {
                    write!(f, "does not resolve to an Atom")?
                }
                TraceGraphEndpointIssueKind::AmbiguousAtomId => {
                    write!(f, "resolves to {} Atoms", issue.atom_occurrences)?
                }
            }
        }
        let omitted_issue_count = self
            .total_issues
            .saturating_sub(displayed_issue_count)
            .max(self.omitted_issues)
            .max(self.issues.len().saturating_sub(displayed_issue_count));
        if omitted_issue_count > 0 {
            write!(f, "; {omitted_issue_count} additional issue(s) omitted")?;
        }
        Ok(())
    }
}

impl std::error::Error for TraceGraphEndpointError {}

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
    #[serde(default = "legacy_atom_hash_schema_version")]
    pub hash_schema_version: u32,
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

pub fn validate_trace_graph_endpoints(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
) -> Result<(), CoreError> {
    // Endpoint resolution must remain correct for deserialized or manually constructed indexes;
    // `duplicate_atom_ids` is derived metadata and is not the authority for identity counts.
    let mut atom_occurrences = HashMap::with_capacity(atom_index.atoms.len());
    for atom in &atom_index.atoms {
        *atom_occurrences
            .entry(atom.atom_id.as_str())
            .or_insert(0usize) += 1;
    }

    // Retain references while ordering so comparisons use the original strings without cloning
    // them. Only the final diagnostic samples are copied and byte-bounded.
    let mut retained_issue_refs = Vec::with_capacity(
        trace_graph
            .links
            .len()
            .saturating_mul(2)
            .min(TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT),
    );
    let mut total_issues = 0usize;
    for (link_index, link) in trace_graph.links.iter().enumerate() {
        for (endpoint, atom_id) in [
            (TraceGraphEndpointRole::From, link.from.as_str()),
            (TraceGraphEndpointRole::To, link.to.as_str()),
        ] {
            let occurrences = atom_occurrences.get(atom_id).copied().unwrap_or_default();
            let kind = match occurrences {
                0 => TraceGraphEndpointIssueKind::MissingAtom,
                1 => continue,
                _ => TraceGraphEndpointIssueKind::AmbiguousAtomId,
            };
            total_issues = total_issues.saturating_add(1);
            retain_bounded_endpoint_issue_ref(
                trace_graph,
                &mut retained_issue_refs,
                EndpointIssueRef {
                    link_index,
                    endpoint,
                    kind,
                    atom_occurrences: occurrences,
                },
            );
        }
    }

    if total_issues == 0 {
        Ok(())
    } else {
        let issues = retained_issue_refs
            .into_iter()
            .map(|issue_ref| endpoint_issue_sample(trace_graph, issue_ref))
            .collect::<Vec<_>>();
        Err(CoreError::TraceGraphEndpointIntegrity(
            TraceGraphEndpointError {
                omitted_issues: total_issues.saturating_sub(issues.len()),
                issues,
                total_issues,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy)]
struct EndpointIssueRef {
    link_index: usize,
    endpoint: TraceGraphEndpointRole,
    kind: TraceGraphEndpointIssueKind,
    atom_occurrences: usize,
}

fn retain_bounded_endpoint_issue_ref(
    trace_graph: &TraceGraph,
    issues: &mut Vec<EndpointIssueRef>,
    issue: EndpointIssueRef,
) {
    let insertion_index = issues
        .binary_search_by(|existing| compare_endpoint_issue_refs(trace_graph, existing, &issue))
        .unwrap_or_else(|index| index);
    if insertion_index >= TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT {
        return;
    }
    issues.insert(insertion_index, issue);
    if issues.len() > TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT {
        issues.pop();
    }
}

fn compare_endpoint_issue_refs(
    trace_graph: &TraceGraph,
    left: &EndpointIssueRef,
    right: &EndpointIssueRef,
) -> std::cmp::Ordering {
    let left_link = &trace_graph.links[left.link_index];
    let right_link = &trace_graph.links[right.link_index];
    left_link
        .link_id
        .cmp(&right_link.link_id)
        .then_with(|| left.endpoint.cmp(&right.endpoint))
        .then_with(|| {
            endpoint_atom_id(left_link, left.endpoint)
                .cmp(endpoint_atom_id(right_link, right.endpoint))
        })
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.link_index.cmp(&right.link_index))
}

fn endpoint_issue_sample(
    trace_graph: &TraceGraph,
    issue_ref: EndpointIssueRef,
) -> TraceGraphEndpointIssue {
    let link = &trace_graph.links[issue_ref.link_index];
    let atom_id = endpoint_atom_id(link, issue_ref.endpoint);
    let (link_id, link_id_omitted_bytes) = bounded_endpoint_sample(&link.link_id);
    let (atom_id, atom_id_omitted_bytes) = bounded_endpoint_sample(atom_id);
    TraceGraphEndpointIssue {
        link_index: issue_ref.link_index,
        link_id,
        link_id_truncated: link_id_omitted_bytes > 0,
        link_id_omitted_bytes,
        endpoint: issue_ref.endpoint,
        atom_id,
        atom_id_truncated: atom_id_omitted_bytes > 0,
        atom_id_omitted_bytes,
        kind: issue_ref.kind,
        atom_occurrences: issue_ref.atom_occurrences,
    }
}

fn endpoint_atom_id(link: &TraceLink, endpoint: TraceGraphEndpointRole) -> &str {
    match endpoint {
        TraceGraphEndpointRole::From => &link.from,
        TraceGraphEndpointRole::To => &link.to,
    }
}

fn bounded_endpoint_sample(value: &str) -> (String, usize) {
    if value.len() <= TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT {
        return (value.to_string(), 0);
    }
    let mut end = TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_string(), value.len() - end)
}

fn bounded_endpoint_display_sample(value: &str, known_truncated: bool) -> String {
    let (mut sample, omitted_bytes) = bounded_endpoint_sample(value);
    if known_truncated || omitted_bytes > 0 {
        sample.push('…');
    }
    sample
}

#[cfg(test)]
fn compare_endpoint_issues(
    left: &TraceGraphEndpointIssue,
    right: &TraceGraphEndpointIssue,
) -> std::cmp::Ordering {
    left.link_id
        .cmp(&right.link_id)
        .then_with(|| left.endpoint.cmp(&right.endpoint))
        .then_with(|| left.atom_id.cmp(&right.atom_id))
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.link_index.cmp(&right.link_index))
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_commit_signature: bool,
    pub no_change_required_requires_rationale: bool,
    pub required_links: BTreeMap<String, Vec<RequiredLinkRule>>,
    pub verification: Vec<VerificationCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVerificationPolicy {
    pub policy: VerificationPolicy,
    pub effective: EffectivePolicy,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCommand {
    pub id: String,
    pub command: String,
    pub cwd: String,
    #[serde(default = "default_verification_command_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_verification_command_max_output_bytes")]
    pub max_output_bytes: usize,
    #[serde(default)]
    pub inherit_env: bool,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub allow_failure: bool,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_atom_changed_files: Vec<NonAtomFileChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_coverage: Option<ChangeCoverage>,
    pub open_fires: Vec<Fire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_migration: Option<ToolMigration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonAtomFileChange {
    pub path: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_blob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_blob: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolMigration {
    pub kind: String,
    pub from_hash_schema_version: u32,
    pub to_hash_schema_version: u32,
    pub source_unchanged: bool,
    pub affected_atoms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanBuildOptions<'a> {
    pub reason: &'a str,
    pub now: &'a str,
    pub source_unchanged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedCheck {
    pub id: String,
    pub command: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessmentSubject {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub kind: String,
    pub trust_kind: String,
    pub subject_digest: String,
    pub content_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_digest: Option<String>,
    pub immutable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationExecutionRecord {
    pub id: String,
    pub profile: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phase: String,
    pub trust_kind: String,
    pub subject_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub environment_digest: String,
    pub worktree_before: String,
    pub worktree_after: String,
    pub subject_unchanged: bool,
    pub cwd: String,
    pub cwd_kind: String,
    pub environment_policy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explicit_env_keys: Vec<String>,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
    pub stdout_bytes_seen: u64,
    pub stdout_bytes_stored: usize,
    pub stdout_truncated: bool,
    pub stderr_bytes_seen: u64,
    pub stderr_bytes_stored: usize,
    pub stderr_truncated: bool,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub timed_out: bool,
    pub termination_scope: String,
    #[serde(default)]
    pub redacted_value_count: usize,
}

/// A resolution whose recorded basis no longer matches the current state.
///
/// The fire and atom fields identify what to refresh. They are omitted when
/// empty so records written before they existed keep their encoding.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleResolution {
    pub resolution_uid: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fire_uid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fire_display_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_atom_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_atom_id: String,
}

impl StaleResolution {
    fn for_resolution(resolution: &Resolution, reason: &str) -> Self {
        Self {
            resolution_uid: resolution.resolution_uid.clone(),
            reason: reason.to_string(),
            fire_uid: resolution.fire_uid.clone(),
            fire_display_id: fire_display_id_from_uid(&resolution.fire_uid).unwrap_or_default(),
            source_atom_id: resolution.basis.source_atom.atom_id.clone(),
            target_atom_id: resolution.basis.target_atom.atom_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingEvidenceRef {
    pub resolution_uid: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeConflictKind {
    Text,
    Binary,
    Semantic,
}

impl MergeConflictKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Binary => "binary",
            Self::Semantic => "semantic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedMergeConflict {
    pub kind: MergeConflictKind,
    pub subject: String,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_merge_conflicts: Vec<UnresolvedMergeConflict>,
    pub duplicate_atom_ids: Vec<String>,
    pub verified_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<AssessmentSubject>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution: Vec<VerificationExecutionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy: Option<EffectivePolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_coverage: Option<ChangeCoverage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommittableOutcome {
    Passed,
    Failed,
}

impl CommittableOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommittableReason {
    UnresolvedMergeConflicts,
    OpenRequiredFires,
    MissingRequiredLinks,
    StaleResolutions,
    MissingEvidenceRefs,
    DuplicateAtomIds,
    FailedChecks,
}

impl CommittableReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnresolvedMergeConflicts => "unresolved_merge_conflicts",
            Self::OpenRequiredFires => "open_required_fires",
            Self::MissingRequiredLinks => "missing_required_links",
            Self::StaleResolutions => "stale_resolutions",
            Self::MissingEvidenceRefs => "missing_evidence_refs",
            Self::DuplicateAtomIds => "duplicate_atom_ids",
            Self::FailedChecks => "failed_checks",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommittableDecision {
    pub outcome: CommittableOutcome,
    pub primary_reason: Option<CommittableReason>,
    pub blocking_reasons: Vec<CommittableReason>,
    pub warning_reasons: Vec<CommittableReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_coverage_digest: Option<String>,
}

impl CommittableDecision {
    pub fn is_committable(&self) -> bool {
        self.outcome == CommittableOutcome::Passed
    }

    pub fn is_blocking(&self, reason: CommittableReason) -> bool {
        self.blocking_reasons.contains(&reason)
    }

    pub fn is_warning(&self, reason: CommittableReason) -> bool {
        self.warning_reasons.contains(&reason)
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SealedSnapshotSemanticIssue {
    OpenRequiredFireCountMismatch {
        computed: usize,
        recorded: usize,
    },
    DuplicateAtomIdsMismatch {
        computed: Vec<String>,
        atom_index: Vec<String>,
        verification: Vec<String>,
    },
    MissingRequiredLinksMismatch {
        computed: Vec<MissingRequiredLink>,
        verification: Vec<MissingRequiredLink>,
    },
    StaleResolutionsMismatch {
        computed: Vec<StaleResolution>,
        verification: Vec<StaleResolution>,
    },
    VerificationResultMismatch {
        computed: String,
        recorded: String,
    },
    TraceCompletenessRequiredMismatch {
        computed: bool,
        recorded: bool,
    },
    DuplicateFireUid {
        fire_uid: String,
        occurrences: usize,
    },
    DuplicateActiveResolutionUid {
        resolution_uid: String,
        occurrences: usize,
    },
    ActiveResolutionFireCardinality {
        fire_uid: String,
        fire_occurrences: usize,
        resolution_uids: Vec<String>,
    },
    ActiveResolutionPerFireCardinality {
        fire_uid: String,
        resolution_uids: Vec<String>,
    },
    FireResolutionIdentityMismatch {
        fire_uid: String,
        declared_resolution_uid: Option<String>,
        active_resolution_uids: Vec<String>,
    },
    InvalidEffectivePolicyContract {},
    EffectivePolicyProjectionMismatch {},
    InvalidChangeCoverageContract {},
}

impl SealedSnapshotSemanticIssue {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::OpenRequiredFireCountMismatch { .. } => "open_required_fire_count_mismatch",
            Self::DuplicateAtomIdsMismatch { .. } => "duplicate_atom_ids_mismatch",
            Self::MissingRequiredLinksMismatch { .. } => "missing_required_links_mismatch",
            Self::StaleResolutionsMismatch { .. } => "stale_resolutions_mismatch",
            Self::VerificationResultMismatch { .. } => "verification_result_mismatch",
            Self::TraceCompletenessRequiredMismatch { .. } => {
                "trace_completeness_required_mismatch"
            }
            Self::DuplicateFireUid { .. } => "duplicate_fire_uid",
            Self::DuplicateActiveResolutionUid { .. } => "duplicate_active_resolution_uid",
            Self::ActiveResolutionFireCardinality { .. } => "active_resolution_fire_cardinality",
            Self::ActiveResolutionPerFireCardinality { .. } => {
                "active_resolution_per_fire_cardinality"
            }
            Self::FireResolutionIdentityMismatch { .. } => "fire_resolution_identity_mismatch",
            Self::InvalidEffectivePolicyContract { .. } => "invalid_effective_policy_contract",
            Self::EffectivePolicyProjectionMismatch { .. } => {
                "effective_policy_projection_mismatch"
            }
            Self::InvalidChangeCoverageContract { .. } => "invalid_change_coverage_contract",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedSnapshotSemanticError {
    pub issues: Vec<SealedSnapshotSemanticIssue>,
}

impl fmt::Display for SealedSnapshotSemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "sealed snapshot semantic integrity failed: {} issue(s)",
            self.issues.len()
        )?;
        for issue in &self.issues {
            write!(f, "; {}", issue.kind())?;
        }
        Ok(())
    }
}

impl std::error::Error for SealedSnapshotSemanticError {}

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
            atoms.extend(extract_source_atoms(&path, &rel_posix, "code")?);
        } else if config.matches("tests", &rel_posix) {
            atoms.extend(extract_source_atoms(&path, &rel_posix, "test")?);
        }
    }
    atoms.sort_by(|left, right| {
        left.atom_id
            .cmp(&right.atom_id)
            .then_with(|| left.artifact_path.cmp(&right.artifact_path))
    });
    let duplicate_atom_ids = authoritative_duplicate_atom_ids(&atoms);
    Ok(AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
        atoms,
        duplicate_atom_ids,
    })
}

fn extract_source_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    if path.extension().and_then(|extension| extension.to_str()) == Some("py") {
        extract_python_atoms(path, artifact_path, fallback_kind)
    } else {
        extract_explicit_cf_atoms(path, artifact_path, fallback_kind)
    }
}

fn authoritative_duplicate_atom_ids(atoms: &[Atom]) -> Vec<String> {
    let mut counts = BTreeMap::new();
    for atom in atoms {
        *counts.entry(atom.atom_id.clone()).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .filter_map(|(atom_id, count)| (count > 1).then_some(atom_id))
        .collect()
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
    let mut in_links = false;
    let mut seen_links = false;
    let mut skipping_extra_root = false;

    for (line_index, raw_line) in read_text_lossy(path)?.lines().enumerate() {
        let line_no = line_index + 1;
        let Some(line) = limited_yaml_line(raw_line, line_no, "codefire.links.yaml")? else {
            continue;
        };
        if line.indent == 0 {
            flush_link(&mut links, current.take(), current_line)?;
            current_line = 0;
            current = None;
            match limited_yaml_key(line.text) {
                Some(("version", _)) => {
                    in_links = false;
                    skipping_extra_root = false;
                }
                Some(("links", "")) => {
                    in_links = true;
                    seen_links = true;
                    skipping_extra_root = false;
                }
                Some((key, _)) => {
                    if seen_links {
                        in_links = false;
                        skipping_extra_root = true;
                        continue;
                    }
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.links.yaml: root key {key} at line {line_no}"
                    )));
                }
                None => {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.links.yaml: line {line_no} must be key: value"
                    )));
                }
            }
            continue;
        }
        if !in_links {
            if skipping_extra_root {
                continue;
            }
            return Err(CoreError::Config(format!(
                "unsupported codefire.links.yaml: nested entry outside links at line {line_no}"
            )));
        }
        if line.indent == 2 {
            let Some(value) = line.text.strip_prefix("- from:") else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.links.yaml: link item at line {line_no} must start with '- from:'"
                )));
            };
            flush_link(&mut links, current.take(), current_line)?;
            current = Some(TraceLinkInput {
                from: unquote(value.trim()),
                to: None,
                link_type: None,
            });
            current_line = line_no;
            continue;
        }
        if line.indent != 4 {
            return Err(CoreError::Config(format!(
                "unsupported codefire.links.yaml: unsupported indentation at line {line_no}"
            )));
        }
        let Some(link) = current.as_mut() else {
            return Err(CoreError::Config(format!(
                "invalid codefire.links.yaml: link field before link item at line {line_no}"
            )));
        };
        match limited_yaml_key(line.text) {
            Some(("from", value)) => link.from = unquote(value),
            Some(("to", value)) => link.to = Some(unquote(value)),
            Some(("type", value)) => link.link_type = Some(unquote(value)),
            Some((key, _)) => {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.links.yaml: link field {key} at line {line_no}"
                )));
            }
            None => {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.links.yaml: line {line_no} must be key: value"
                )));
            }
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

#[derive(Debug, Clone, Copy)]
struct VerificationSignals {
    unresolved_merge_conflicts: usize,
    open_required_fires: usize,
    missing_required_links: usize,
    failed_checks: usize,
    stale_resolutions: usize,
    missing_evidence_refs: usize,
    duplicate_atom_ids: usize,
}

fn committable_decision_from_signals(
    trace_completeness_required: bool,
    signals: VerificationSignals,
) -> CommittableDecision {
    let mut blocking_reasons = Vec::with_capacity(7);
    let mut warning_reasons = Vec::with_capacity(1);
    if signals.unresolved_merge_conflicts > 0 {
        blocking_reasons.push(CommittableReason::UnresolvedMergeConflicts);
    }
    if signals.open_required_fires > 0 {
        blocking_reasons.push(CommittableReason::OpenRequiredFires);
    }
    if signals.missing_required_links > 0 {
        if trace_completeness_required {
            blocking_reasons.push(CommittableReason::MissingRequiredLinks);
        } else {
            warning_reasons.push(CommittableReason::MissingRequiredLinks);
        }
    }
    if signals.stale_resolutions > 0 {
        blocking_reasons.push(CommittableReason::StaleResolutions);
    }
    if signals.missing_evidence_refs > 0 {
        blocking_reasons.push(CommittableReason::MissingEvidenceRefs);
    }
    if signals.duplicate_atom_ids > 0 {
        blocking_reasons.push(CommittableReason::DuplicateAtomIds);
    }
    if signals.failed_checks > 0 {
        blocking_reasons.push(CommittableReason::FailedChecks);
    }
    let primary_reason = blocking_reasons.first().copied();
    CommittableDecision {
        outcome: if primary_reason.is_some() {
            CommittableOutcome::Failed
        } else {
            CommittableOutcome::Passed
        },
        primary_reason,
        blocking_reasons,
        warning_reasons,
        effective_policy_digest: None,
        change_coverage_digest: None,
    }
}

pub fn committable_decision(verification: &Verification) -> CommittableDecision {
    let mut decision = committable_decision_from_signals(
        verification.trace_completeness_required,
        VerificationSignals {
            unresolved_merge_conflicts: verification.unresolved_merge_conflicts.len(),
            open_required_fires: verification.open_required_fires,
            missing_required_links: verification.missing_required_links.len(),
            failed_checks: verification.failed_checks.len(),
            stale_resolutions: verification.stale_resolutions.len(),
            missing_evidence_refs: verification.missing_evidence_refs.len(),
            duplicate_atom_ids: verification.duplicate_atom_ids.len(),
        },
    );
    decision.effective_policy_digest = verification
        .effective_policy
        .as_ref()
        .map(|policy| policy.digest.clone());
    decision.change_coverage_digest = verification
        .change_coverage
        .as_ref()
        .map(|coverage| coverage.digest.clone());
    decision
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerificationInputs {
    pub unresolved_merge_conflicts: Vec<UnresolvedMergeConflict>,
    pub missing_required_links: Vec<MissingRequiredLink>,
    pub failed_checks: Vec<FailedCheck>,
    pub stale_resolutions: Vec<StaleResolution>,
    pub missing_evidence_refs: Vec<MissingEvidenceRef>,
    pub subject: Option<AssessmentSubject>,
    pub execution: Vec<VerificationExecutionRecord>,
}

fn open_required_fire_count(fires: &[Fire]) -> usize {
    fires
        .iter()
        .filter(|fire| fire.status == "open" && fire.severity == "required")
        .count()
}

pub fn build_verification(
    scan: &ScanResult,
    inputs: VerificationInputs,
    policy: &VerificationPolicy,
    verified_at: &str,
) -> Verification {
    let VerificationInputs {
        unresolved_merge_conflicts,
        missing_required_links,
        failed_checks,
        stale_resolutions,
        missing_evidence_refs,
        subject,
        execution,
    } = inputs;
    let duplicate_atom_ids = authoritative_duplicate_atom_ids(&scan.atom_index.atoms);
    let open_required_fires = open_required_fire_count(&scan.open_fires);
    let decision = committable_decision_from_signals(
        policy.require_trace_completeness,
        VerificationSignals {
            unresolved_merge_conflicts: unresolved_merge_conflicts.len(),
            open_required_fires,
            missing_required_links: missing_required_links.len(),
            failed_checks: failed_checks.len(),
            stale_resolutions: stale_resolutions.len(),
            missing_evidence_refs: missing_evidence_refs.len(),
            duplicate_atom_ids: duplicate_atom_ids.len(),
        },
    );
    Verification {
        type_tag: "verification".to_string(),
        version: VERSION,
        result: decision.outcome.as_str().to_string(),
        trace_completeness_required: policy.require_trace_completeness,
        open_required_fires,
        failed_checks,
        missing_required_links,
        stale_resolutions,
        missing_evidence_refs,
        unresolved_merge_conflicts,
        duplicate_atom_ids,
        verified_at: verified_at.to_string(),
        subject,
        execution,
        effective_policy: None,
        change_coverage: scan.change_coverage.clone(),
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
    let link_hashes = trace_link_hash_index(trace_graph)?;
    let current_policy_hash = policy_hash(policy)?;
    let mut stale = Vec::new();

    for resolution in resolutions {
        if resolution.status != "active" {
            continue;
        }
        let source_id = resolution.basis.source_atom.atom_id.as_str();
        if atoms.get(source_id).cloned() != resolution.basis.source_atom.content_hash {
            stale.push(StaleResolution::for_resolution(
                resolution,
                "source atom changed",
            ));
            continue;
        }
        let target_id = resolution.basis.target_atom.atom_id.as_str();
        if atoms.get(target_id).cloned() != resolution.basis.target_atom.content_hash {
            stale.push(StaleResolution::for_resolution(
                resolution,
                "target atom changed",
            ));
            continue;
        }
        if current_policy_hash != resolution.basis.policy_hash {
            stale.push(StaleResolution::for_resolution(
                resolution,
                "policy changed",
            ));
            continue;
        }
        if resolution.basis.trace_links.iter().any(|link| {
            link_hashes.get(link.link_id.as_str()).map(String::as_str)
                != Some(link.link_hash.as_str())
        }) {
            stale.push(StaleResolution::for_resolution(
                resolution,
                "trace link changed",
            ));
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
    required_link_missing_with_work(atom_index, trace_graph, policy).0
}

/// Evaluates required-link rules and reports deterministic correctness-work counters.
///
/// The compatibility wrapper [`required_link_missing`] returns exactly the first
/// tuple element. Keeping counters out of the domain result makes observation
/// incapable of changing policy semantics.
pub fn required_link_missing_with_work(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    policy: &TracePolicy,
) -> (Vec<MissingRequiredLink>, TraceWorkCounters) {
    let mut index = TraceEvaluationIndex::compile(atom_index, trace_graph);
    let missing = index.missing_required_links(policy);
    (missing, index.counters())
}

#[cfg(test)]
fn required_link_missing_reference(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    policy: &TracePolicy,
) -> Vec<MissingRequiredLink> {
    let atom_kinds = atom_index
        .atoms
        .iter()
        .map(|atom| (atom.atom_id.as_str(), atom.kind.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut link_counts = BTreeMap::<(String, String, String), usize>::new();
    for link in &trace_graph.links {
        if let Some(target_kind) = atom_kinds.get(link.to.as_str()) {
            *link_counts
                .entry((
                    link.from.clone(),
                    link.link_type.clone(),
                    (*target_kind).to_string(),
                ))
                .or_insert(0) += 1;
        }
    }
    let mut missing = Vec::new();
    for atom in &atom_index.atoms {
        let Some(rules) = policy.required_links.get(&atom.kind) else {
            continue;
        };
        for rule in rules {
            let found = link_counts
                .get(&(
                    atom.atom_id.clone(),
                    rule.link_type.clone(),
                    rule.target_kind.clone(),
                ))
                .copied()
                .unwrap_or_default();
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

pub fn validate_sealed_snapshot_semantics(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    fires: &[Fire],
    resolutions: &[Resolution],
    verification: &Verification,
    policy: &VerificationPolicy,
) -> Result<(), CoreError> {
    validate_sealed_snapshot_semantics_with_contract(
        atom_index,
        trace_graph,
        fires,
        resolutions,
        verification,
        policy,
        None,
    )
}

pub fn validate_sealed_snapshot_semantics_with_contract(
    atom_index: &AtomIndex,
    trace_graph: &TraceGraph,
    fires: &[Fire],
    resolutions: &[Resolution],
    verification: &Verification,
    policy: &VerificationPolicy,
    policy_contract: Option<&EffectivePolicy>,
) -> Result<(), CoreError> {
    let mut issues = Vec::new();

    if policy_contract.is_some_and(|contract| {
        !matches!(effective_policy_matches_policy(policy, contract), Ok(true))
    }) || verification
        .effective_policy
        .as_ref()
        .is_some_and(|contract| {
            !matches!(effective_policy_matches_policy(policy, contract), Ok(true))
        })
    {
        issues.push(SealedSnapshotSemanticIssue::InvalidEffectivePolicyContract {});
    }
    match (policy_contract, verification.effective_policy.as_ref()) {
        (Some(policy_contract), Some(verification_contract))
            if policy_contract == verification_contract => {}
        (None, None) => {}
        _ => issues.push(SealedSnapshotSemanticIssue::EffectivePolicyProjectionMismatch {}),
    }
    if verification
        .change_coverage
        .as_ref()
        .is_some_and(|coverage| validate_change_coverage(coverage).is_err())
    {
        issues.push(SealedSnapshotSemanticIssue::InvalidChangeCoverageContract {});
    }

    let open_required_fires = open_required_fire_count(fires);
    if verification.open_required_fires != open_required_fires {
        issues.push(SealedSnapshotSemanticIssue::OpenRequiredFireCountMismatch {
            computed: open_required_fires,
            recorded: verification.open_required_fires,
        });
    }

    let computed_duplicate_atom_ids = authoritative_duplicate_atom_ids(&atom_index.atoms);
    let mut recorded_index_duplicates = atom_index.duplicate_atom_ids.clone();
    recorded_index_duplicates.sort();
    let mut recorded_verification_duplicates = verification.duplicate_atom_ids.clone();
    recorded_verification_duplicates.sort();
    if computed_duplicate_atom_ids != recorded_index_duplicates
        || computed_duplicate_atom_ids != recorded_verification_duplicates
    {
        issues.push(SealedSnapshotSemanticIssue::DuplicateAtomIdsMismatch {
            computed: computed_duplicate_atom_ids.clone(),
            atom_index: recorded_index_duplicates,
            verification: recorded_verification_duplicates,
        });
    }

    let computed_missing_required_links = normalized_missing_required_links(required_link_missing(
        atom_index,
        trace_graph,
        &TracePolicy {
            required_links: policy.required_links.clone(),
        },
    ));
    let recorded_missing_required_links =
        normalized_missing_required_links(verification.missing_required_links.clone());
    if computed_missing_required_links != recorded_missing_required_links {
        issues.push(SealedSnapshotSemanticIssue::MissingRequiredLinksMismatch {
            computed: computed_missing_required_links.clone(),
            verification: recorded_missing_required_links,
        });
    }

    let computed_stale_resolutions = normalized_stale_resolutions(stale_resolutions(
        atom_index,
        trace_graph,
        policy,
        resolutions,
    )?);
    let recorded_stale_resolutions =
        normalized_stale_resolutions(verification.stale_resolutions.clone());
    if computed_stale_resolutions != recorded_stale_resolutions {
        issues.push(SealedSnapshotSemanticIssue::StaleResolutionsMismatch {
            computed: computed_stale_resolutions.clone(),
            verification: recorded_stale_resolutions,
        });
    }

    if verification.trace_completeness_required != policy.require_trace_completeness {
        issues.push(
            SealedSnapshotSemanticIssue::TraceCompletenessRequiredMismatch {
                computed: policy.require_trace_completeness,
                recorded: verification.trace_completeness_required,
            },
        );
    }

    let computed_decision = committable_decision_from_signals(
        policy.require_trace_completeness,
        VerificationSignals {
            unresolved_merge_conflicts: verification.unresolved_merge_conflicts.len(),
            open_required_fires,
            missing_required_links: computed_missing_required_links.len(),
            failed_checks: verification.failed_checks.len(),
            stale_resolutions: computed_stale_resolutions.len(),
            missing_evidence_refs: verification.missing_evidence_refs.len(),
            duplicate_atom_ids: computed_duplicate_atom_ids.len(),
        },
    );
    let computed_result = computed_decision.outcome.as_str();
    if verification.result != computed_result {
        issues.push(SealedSnapshotSemanticIssue::VerificationResultMismatch {
            computed: computed_result.to_string(),
            recorded: verification.result.clone(),
        });
    }

    append_fire_resolution_semantic_issues(fires, resolutions, &mut issues);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(CoreError::SealedSnapshotSemanticIntegrity(
            SealedSnapshotSemanticError { issues },
        ))
    }
}

fn normalized_missing_required_links(
    mut missing: Vec<MissingRequiredLink>,
) -> Vec<MissingRequiredLink> {
    missing.sort_by(|left, right| {
        left.atom_id
            .cmp(&right.atom_id)
            .then_with(|| left.required_type.cmp(&right.required_type))
            .then_with(|| left.target_kind.cmp(&right.target_kind))
            .then_with(|| left.min.cmp(&right.min))
            .then_with(|| left.found.cmp(&right.found))
    });
    missing
}

fn normalized_stale_resolutions(mut stale: Vec<StaleResolution>) -> Vec<StaleResolution> {
    stale.sort_by(|left, right| {
        left.resolution_uid
            .cmp(&right.resolution_uid)
            .then_with(|| left.reason.cmp(&right.reason))
    });
    stale
}

fn append_fire_resolution_semantic_issues(
    fires: &[Fire],
    resolutions: &[Resolution],
    issues: &mut Vec<SealedSnapshotSemanticIssue>,
) {
    let mut fires_by_uid = BTreeMap::<&str, Vec<&Fire>>::new();
    for fire in fires {
        fires_by_uid
            .entry(fire.fire_uid.as_str())
            .or_default()
            .push(fire);
    }
    for (fire_uid, matching_fires) in &fires_by_uid {
        if matching_fires.len() > 1 {
            issues.push(SealedSnapshotSemanticIssue::DuplicateFireUid {
                fire_uid: (*fire_uid).to_string(),
                occurrences: matching_fires.len(),
            });
        }
    }

    let mut active_by_uid = BTreeMap::<&str, Vec<&Resolution>>::new();
    let mut active_by_fire = BTreeMap::<&str, Vec<&Resolution>>::new();
    for resolution in resolutions
        .iter()
        .filter(|resolution| resolution.status == "active")
    {
        active_by_uid
            .entry(resolution.resolution_uid.as_str())
            .or_default()
            .push(resolution);
        active_by_fire
            .entry(resolution.fire_uid.as_str())
            .or_default()
            .push(resolution);
    }

    for (resolution_uid, matching_resolutions) in &active_by_uid {
        if matching_resolutions.len() > 1 {
            issues.push(SealedSnapshotSemanticIssue::DuplicateActiveResolutionUid {
                resolution_uid: (*resolution_uid).to_string(),
                occurrences: matching_resolutions.len(),
            });
        }
    }

    for (fire_uid, matching_resolutions) in &active_by_fire {
        let fire_occurrences = fires_by_uid.get(fire_uid).map_or(0, Vec::len);
        let mut resolution_uids = matching_resolutions
            .iter()
            .map(|resolution| resolution.resolution_uid.clone())
            .collect::<Vec<_>>();
        resolution_uids.sort();
        if fire_occurrences != 1 {
            issues.push(
                SealedSnapshotSemanticIssue::ActiveResolutionFireCardinality {
                    fire_uid: (*fire_uid).to_string(),
                    fire_occurrences,
                    resolution_uids: resolution_uids.clone(),
                },
            );
        }
        if matching_resolutions.len() > 1 {
            issues.push(
                SealedSnapshotSemanticIssue::ActiveResolutionPerFireCardinality {
                    fire_uid: (*fire_uid).to_string(),
                    resolution_uids,
                },
            );
        }
    }

    for (fire_uid, matching_fires) in fires_by_uid {
        if matching_fires.len() != 1 {
            continue;
        }
        let fire = matching_fires[0];
        let mut active_resolution_uids = active_by_fire
            .get(fire_uid)
            .into_iter()
            .flat_map(|matching| matching.iter())
            .map(|resolution| resolution.resolution_uid.clone())
            .collect::<Vec<_>>();
        active_resolution_uids.sort();
        let identity_matches = match fire.resolution_uid.as_deref() {
            None => active_resolution_uids.is_empty(),
            Some(declared) => {
                active_resolution_uids.len() == 1 && active_resolution_uids[0] == declared
            }
        };
        if !identity_matches {
            issues.push(
                SealedSnapshotSemanticIssue::FireResolutionIdentityMismatch {
                    fire_uid: fire_uid.to_string(),
                    declared_resolution_uid: fire.resolution_uid.clone(),
                    active_resolution_uids,
                },
            );
        }
    }
}

pub fn current_required_link_missing(
    open_dir: &Path,
) -> Result<Vec<MissingRequiredLink>, CoreError> {
    let atom_index = build_atom_index(open_dir)?;
    let trace_graph = current_trace_graph(open_dir)?;
    validate_trace_graph_endpoints(&atom_index, &trace_graph)?;
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
    fires: Vec<Fire>,
    base_commit: String,
    reason: &str,
    now: &str,
) -> Result<(ScanResult, Vec<Fire>), CoreError> {
    build_scan_result_with_options(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        ScanBuildOptions {
            reason,
            now,
            source_unchanged: false,
        },
    )
}

pub fn build_scan_result_with_options(
    current: AtomIndex,
    base_index: AtomIndex,
    trace_graph: TraceGraph,
    fires: Vec<Fire>,
    base_commit: String,
    options: ScanBuildOptions<'_>,
) -> Result<(ScanResult, Vec<Fire>), CoreError> {
    build_scan_result_against_base(
        current,
        &base_index,
        trace_graph,
        fires,
        base_commit,
        options,
    )
}

pub fn build_scan_result_against_base(
    current: AtomIndex,
    base_index: &AtomIndex,
    trace_graph: TraceGraph,
    fires: Vec<Fire>,
    base_commit: String,
    options: ScanBuildOptions<'_>,
) -> Result<(ScanResult, Vec<Fire>), CoreError> {
    let (scan, fires, _) = build_scan_result_against_base_with_work(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        options,
    )?;
    Ok((scan, fires))
}

/// Builds a scan result while exposing deterministic Trace correctness work.
///
/// Fire semantics remain direct-only and bidirectional. The Trace index is
/// compiled once before changed Atom lookups, so work is `O(atoms + links +
/// changed_atoms + adjacent_entries)` instead of `O(changed_atoms * links)`.
pub fn build_scan_result_against_base_with_work(
    current: AtomIndex,
    base_index: &AtomIndex,
    trace_graph: TraceGraph,
    fires: Vec<Fire>,
    base_commit: String,
    options: ScanBuildOptions<'_>,
) -> Result<(ScanResult, Vec<Fire>, TraceWorkCounters), CoreError> {
    let mut trace_index = TraceEvaluationIndex::compile(&current, &trace_graph);
    let (scan, fires) = build_scan_result_against_base_with_adjacency(
        current,
        base_index,
        trace_graph,
        fires,
        base_commit,
        options,
        |source| trace_index.adjacent_atoms(source),
    )?;
    Ok((scan, fires, trace_index.counters()))
}

fn build_scan_result_against_base_with_adjacency(
    current: AtomIndex,
    base_index: &AtomIndex,
    trace_graph: TraceGraph,
    mut fires: Vec<Fire>,
    base_commit: String,
    options: ScanBuildOptions<'_>,
    mut adjacent: impl FnMut(&str) -> Vec<(String, Vec<String>)>,
) -> Result<(ScanResult, Vec<Fire>), CoreError> {
    let raw_changed = changed_atoms(&current, base_index);
    let tool_schema_changed =
        current.hash_schema_version != base_index.hash_schema_version && options.source_unchanged;
    let changed = if tool_schema_changed {
        Vec::new()
    } else {
        raw_changed.clone()
    };
    let tool_migration = tool_schema_changed.then(|| ToolMigration {
        kind: "atom_hash_schema_changed".to_string(),
        from_hash_schema_version: base_index.hash_schema_version,
        to_hash_schema_version: current.hash_schema_version,
        source_unchanged: options.source_unchanged,
        affected_atoms: raw_changed.clone(),
    });
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
        for (target, trace_path) in adjacent(source) {
            if target == *source {
                continue;
            }
            let key = fire_key(source, &target, options.reason, &trace_path, &base_commit)?;
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
                reason: options.reason.to_string(),
                trace_path,
                created_by: "scan".to_string(),
                created_at: options.now.to_string(),
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
            fire.obsolete_at = Some(options.now.to_string());
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
        non_atom_changed_files: Vec::new(),
        change_coverage: None,
        open_fires,
        tool_migration,
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
    pub fn into_trace_link(self) -> Result<TraceLink, CoreError> {
        let to = self.to.ok_or_else(|| {
            CoreError::Config("invalid codefire.links.yaml: link missing to".to_string())
        })?;
        let link_type = self.link_type.ok_or_else(|| {
            CoreError::Config("invalid codefire.links.yaml: link missing type".to_string())
        })?;
        let link_id = trace_link_id(&self.from, &to, &link_type)?;
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

fn set_required_rule_field(
    rule: &mut RequiredLinkRuleDraft,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), CoreError> {
    match key {
        "type" => rule.link_type = Some(unquote(value)),
        "target_kind" => rule.target_kind = Some(unquote(value)),
        "min" => rule.min = Some(parse_usize_field("required_links min", value.trim())?),
        _ => {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: required link field {key} at line {line_no}"
            )));
        }
    }
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

fn parse_u64_field(field: &str, value: &str) -> Result<u64, CoreError> {
    let parsed = unquote(value).parse::<u64>().map_err(|_| {
        CoreError::Config(format!(
            "invalid codefire.policy.yaml: {field} must be an integer"
        ))
    })?;
    if parsed == 0 {
        return Err(CoreError::Config(format!(
            "invalid codefire.policy.yaml: {field} must be greater than zero"
        )));
    }
    Ok(parsed)
}

fn parse_env_field(value: &str, line_no: usize) -> Result<BTreeMap<String, String>, CoreError> {
    let rendered = unquote(value);
    let mut env = BTreeMap::new();
    for item in rendered
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let Some((key, value)) = item.split_once('=') else {
            return Err(CoreError::Config(format!(
                "invalid codefire.policy.yaml: verification env entry at line {line_no} must be KEY=VALUE"
            )));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(CoreError::Config(format!(
                "invalid codefire.policy.yaml: verification env key at line {line_no} is empty"
            )));
        }
        env.insert(key.to_string(), value.trim().to_string());
    }
    Ok(env)
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

const TRACE_LINK_ID_HEX_LENGTH: usize = 32;

fn trace_link_id(from: &str, to: &str, link_type: &str) -> Result<String, CoreError> {
    let value = serde_json::json!({
        "from": from,
        "to": to,
        "type": link_type,
    });
    let digest = codefire_util::sha256_hex(&serde_json::to_vec(&value)?);
    Ok(format!(
        "LINK-sha256-{}",
        &digest[..TRACE_LINK_ID_HEX_LENGTH]
    ))
}

fn legacy_trace_link_id(from: &str, to: &str, link_type: &str) -> String {
    format!("LINK-{from}-{link_type}-{to}")
}

fn trace_link_hash_index(trace_graph: &TraceGraph) -> Result<BTreeMap<String, String>, CoreError> {
    let mut link_hashes = BTreeMap::new();
    for link in &trace_graph.links {
        link_hashes.insert(link.link_id.clone(), link.link_hash.clone());
        let legacy_id = legacy_trace_link_id(&link.from, &link.to, &link.link_type);
        if legacy_id != link.link_id {
            link_hashes.insert(
                legacy_id.clone(),
                trace_link_hash(&link.from, &link.to, &link.link_type, &legacy_id)?,
            );
        }
    }
    Ok(link_hashes)
}

#[cfg(test)]
fn adjacent_atoms_reference(atom_id: &str, trace_graph: &TraceGraph) -> Vec<(String, Vec<String>)> {
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

const FIRE_UID_PREFIX: &str = "fire_sha256_";
const FIRE_UID_HEX_LENGTH: usize = 32;
const FIRE_DISPLAY_HEX_LENGTH: usize = 12;

fn fire_digest_hex(key: &str) -> String {
    codefire_util::sha256_hex(key.as_bytes())
}

fn fire_uid_from_digest(digest: &str) -> String {
    format!("{FIRE_UID_PREFIX}{}", &digest[..FIRE_UID_HEX_LENGTH])
}

/// Derives the display id (`FIRE-XXXXXXXXXXXX`) from a fire uid.
pub fn fire_display_id_from_uid(fire_uid: &str) -> Option<String> {
    let digest = fire_uid.strip_prefix(FIRE_UID_PREFIX)?;
    (digest.len() >= FIRE_DISPLAY_HEX_LENGTH && digest.is_ascii())
        .then(|| fire_display_id_from_digest(digest))
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
        } else if file_type.is_file() {
            let rel_path = to_posix(&rel);
            let import_operational_temp = rel.components().count() == 1
                && ((rel_path.starts_with(".codefire-import.lock.bootstrap-")
                    && (rel_path.ends_with(".tmp") || rel_path.ends_with(".part")))
                    || (rel_path.starts_with(".codefire-import.write-")
                        && rel_path.ends_with(".tmp")));
            if !import_operational_temp
                && !matches!(
                    rel_path.as_str(),
                    ".codefire-open"
                        | ".codefire-import.lock"
                        | ".codefire-import-transaction.json"
                )
            {
                files.push(rel);
            }
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
mod tests;
