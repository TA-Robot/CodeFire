use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

mod change_coverage;
mod effective_policy;
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

#[derive(Clone, Copy)]
struct LimitedYamlLine<'a> {
    indent: usize,
    text: &'a str,
}

fn limited_yaml_line<'a>(
    raw_line: &'a str,
    line_no: usize,
    file_name: &str,
) -> Result<Option<LimitedYamlLine<'a>>, CoreError> {
    let indent = raw_line
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == b' ')
        .count();
    if raw_line[indent..].starts_with('\t') {
        return Err(CoreError::Config(format!(
            "unsupported {file_name}: tab indentation at line {line_no}"
        )));
    }
    let without_comment = strip_limited_yaml_comment(raw_line);
    let text = without_comment.trim_end();
    let stripped = text.trim_start_matches(' ');
    if stripped.is_empty() {
        return Ok(None);
    }
    Ok(Some(LimitedYamlLine {
        indent: text.len() - stripped.len(),
        text: stripped,
    }))
}

fn strip_limited_yaml_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_double => escaped = true,
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '#' if !in_single && !in_double => return &line[..index],
            _ => {}
        }
    }
    line
}

fn limited_yaml_key(text: &str) -> Option<(&str, &str)> {
    let (key, value) = text.split_once(':')?;
    let key = key.trim();
    if key.is_empty() || key.starts_with('-') {
        return None;
    }
    Some((key, value.trim()))
}

pub fn parse_verification_policy(open_dir: &Path) -> Result<VerificationPolicy, CoreError> {
    Ok(parse_effective_verification_policy(open_dir)?.policy)
}

pub fn parse_effective_verification_policy(
    open_dir: &Path,
) -> Result<ParsedVerificationPolicy, CoreError> {
    let path = open_dir.join("codefire.policy.yaml");
    let mut policy = default_verification_policy();
    let mut provenance = PolicyProvenance::default();
    if !path.exists() {
        let effective = effective_policy(&policy, &provenance)?;
        return Ok(ParsedVerificationPolicy { policy, effective });
    }

    let mut required_links = BTreeMap::<String, Vec<RequiredLinkRule>>::new();
    let mut in_required_links = false;
    let mut in_commit_policy = false;
    let mut in_extinguish_policy = false;
    let mut in_no_change_required_policy = false;
    let mut in_verification = false;
    let mut in_verification_required_wrapper = false;
    let mut saw_required_links = false;
    let mut current_kind = None::<String>;
    let mut current_rule = None::<RequiredLinkRuleDraft>;
    let mut current_check = None::<VerificationCommandDraft>;
    let mut checks = Vec::new();

    for (line_index, raw_line) in read_text_lossy(&path)?.lines().enumerate() {
        let line_no = line_index + 1;
        let Some(line) = limited_yaml_line(raw_line, line_no, "codefire.policy.yaml")? else {
            continue;
        };
        let stripped = line.text;
        if line.indent == 0 {
            if stripped == "version:" || stripped.starts_with("version:") {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "required_links:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = true;
                saw_required_links = true;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "commit_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = true;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "extinguish_policy:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = true;
                in_no_change_required_policy = false;
                in_verification = false;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else if stripped == "verification:" {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                in_required_links = false;
                in_commit_policy = false;
                in_extinguish_policy = false;
                in_no_change_required_policy = false;
                in_verification = true;
                in_verification_required_wrapper = false;
                current_kind = None;
            } else {
                flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
                flush_verification_check(&mut checks, current_check.take())?;
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: root key at line {line_no}"
                )));
            }
            continue;
        }
        if in_commit_policy {
            if line.indent != 2 {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: unsupported commit_policy indentation at line {line_no}"
                )));
            }
            let Some((key, value)) = limited_yaml_key(stripped) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: commit_policy entry at line {line_no} must be key: value"
                )));
            };
            set_commit_policy_bool(&mut policy, key, value, line_no)?;
            provenance.record_repository_rule(commit_policy_rule_id(key));
            continue;
        }
        if in_extinguish_policy {
            if line.indent == 2 && stripped == "no-change-required:" {
                in_no_change_required_policy = true;
                continue;
            }
            if in_no_change_required_policy {
                if line.indent != 4 {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: unsupported no-change-required indentation at line {line_no}"
                    )));
                }
                let Some((key, value)) = limited_yaml_key(stripped) else {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: no-change-required entry at line {line_no} must be key: value"
                    )));
                };
                if key == "requires_rationale" {
                    policy.no_change_required_requires_rationale = parse_bool_field(value.trim())?;
                    provenance
                        .record_repository_rule("extinguish.no_change_required.requires_rationale");
                    continue;
                }
            }
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: extinguish_policy entry at line {line_no}"
            )));
        }
        if in_verification {
            if line.indent == 2 && stripped == "required:" {
                flush_verification_check(&mut checks, current_check.take())?;
                in_verification_required_wrapper = true;
                continue;
            }
            let item_indent = if in_verification_required_wrapper {
                4
            } else {
                2
            };
            let field_indent = if in_verification_required_wrapper {
                6
            } else {
                4
            };
            if line.indent == item_indent && stripped.starts_with("- ") {
                let item = stripped.trim_start_matches("- ").trim();
                let Some((key, value)) = limited_yaml_key(item) else {
                    return Err(CoreError::Config(format!(
                        "unsupported codefire.policy.yaml: verification item at line {line_no} must be '- key: value'"
                    )));
                };
                flush_verification_check(&mut checks, current_check.take())?;
                let mut check = VerificationCommandDraft {
                    id: None,
                    command: None,
                    cwd: Some(".".to_string()),
                    timeout_ms: None,
                    max_output_bytes: None,
                    inherit_env: None,
                    env: BTreeMap::new(),
                    allow_failure: None,
                };
                set_verification_check_field(&mut check, key, value, line_no)?;
                current_check = Some(check);
                continue;
            }
            if line.indent != field_indent {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: unsupported verification indentation at line {line_no}; use verification: [items] or legacy verification.required: [items]"
                )));
            }
            let Some(check) = current_check.as_mut() else {
                return Err(CoreError::Config(format!(
                    "invalid codefire.policy.yaml: verification field before item at line {line_no}"
                )));
            };
            let Some((key, value)) = limited_yaml_key(stripped) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: verification entry at line {line_no} must be key: value"
                )));
            };
            set_verification_check_field(check, key, value, line_no)?;
            continue;
        }
        if !in_required_links {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: nested entry outside supported section at line {line_no}"
            )));
        }
        if line.indent == 2 && stripped.ends_with(':') {
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            let kind = stripped.trim_end_matches(':').to_string();
            required_links.entry(kind.clone()).or_default();
            current_kind = Some(kind);
            continue;
        }
        if line.indent == 4 && stripped.starts_with("- ") {
            let item = stripped.trim_start_matches("- ").trim();
            let Some((key, value)) = limited_yaml_key(item) else {
                return Err(CoreError::Config(format!(
                    "unsupported codefire.policy.yaml: required link rule at line {line_no} must be '- key: value'"
                )));
            };
            flush_required_rule(&mut required_links, &current_kind, current_rule.take())?;
            let mut rule = RequiredLinkRuleDraft {
                link_type: None,
                target_kind: None,
                min: None,
            };
            set_required_rule_field(&mut rule, key, value, line_no)?;
            current_rule = Some(rule);
            continue;
        }
        if line.indent != 6 {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: unsupported required_links indentation at line {line_no}"
            )));
        }
        let Some(rule) = current_rule.as_mut() else {
            return Err(CoreError::Config(format!(
                "invalid codefire.policy.yaml: required link field before rule at line {line_no}"
            )));
        };
        let Some((key, value)) = limited_yaml_key(stripped) else {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: required link entry at line {line_no} must be key: value"
            )));
        };
        set_required_rule_field(rule, key, value, line_no)?;
    }
    flush_required_rule(&mut required_links, &current_kind, current_rule)?;
    flush_verification_check(&mut checks, current_check)?;
    if required_links.is_empty() {
        required_links = policy.required_links;
    }
    policy.required_links = required_links;
    policy.verification = checks;
    if saw_required_links {
        provenance.record_repository_prefix("trace.required_links.");
    }
    if !policy.verification.is_empty() {
        provenance.record_repository_prefix("verification.command.");
    }
    let effective = effective_policy(&policy, &provenance)?;
    Ok(ParsedVerificationPolicy { policy, effective })
}

fn commit_policy_rule_id(key: &str) -> &'static str {
    match key {
        "require_no_required_fires" => "commit.require_no_required_fires",
        "require_no_stale_resolutions" => "commit.require_no_stale_resolutions",
        "require_trace_completeness" => "commit.require_trace_completeness",
        "require_verification_success" => "commit.require_verification_success",
        "reject_duplicate_atom_ids" => "commit.reject_duplicate_atom_ids",
        "require_commit_signature" => "commit.require_commit_signature",
        _ => unreachable!("set_commit_policy_bool accepted a known key"),
    }
}

fn set_commit_policy_bool(
    policy: &mut VerificationPolicy,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), CoreError> {
    let parsed = parse_bool_field(value)?;
    match key {
        "require_no_required_fires" => {
            policy.require_no_required_fires = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_no_stale_resolutions" => {
            policy.require_no_stale_resolutions = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_trace_completeness" => policy.require_trace_completeness = parsed,
        "require_verification_success" => {
            policy.require_verification_success = safety_floor_policy_value(key, parsed, line_no)?
        }
        "reject_duplicate_atom_ids" => {
            policy.reject_duplicate_atom_ids = safety_floor_policy_value(key, parsed, line_no)?
        }
        "require_commit_signature" => policy.require_commit_signature = parsed,
        _ => {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: commit_policy field {key} at line {line_no}"
            )));
        }
    }
    Ok(())
}

fn safety_floor_policy_value(key: &str, parsed: bool, line_no: usize) -> Result<bool, CoreError> {
    if parsed {
        Ok(true)
    } else {
        Err(CoreError::Config(format!(
            "invalid codefire.policy.yaml: commit_policy.{key} must be true at line {line_no}; rule {} source repository_config adoption rejected because safety-floor values cannot be relaxed; only commit.require_trace_completeness may relax a committable check",
            commit_policy_rule_id(key),
        )))
    }
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
        require_commit_signature: false,
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
    timeout_ms: Option<u64>,
    max_output_bytes: Option<usize>,
    inherit_env: Option<bool>,
    env: BTreeMap<String, String>,
    allow_failure: Option<bool>,
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
        timeout_ms: draft
            .timeout_ms
            .unwrap_or(DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS),
        max_output_bytes: draft
            .max_output_bytes
            .unwrap_or(DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES),
        inherit_env: draft.inherit_env.unwrap_or(false),
        env: draft.env,
        allow_failure: draft.allow_failure.unwrap_or(false),
    });
    Ok(())
}

fn set_verification_check_field(
    check: &mut VerificationCommandDraft,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), CoreError> {
    match key {
        "id" => check.id = Some(unquote(value)),
        "command" => check.command = Some(unquote(value)),
        "cwd" => check.cwd = Some(unquote(value)),
        "timeout_ms" => {
            check.timeout_ms = Some(parse_u64_field("verification timeout_ms", value.trim())?)
        }
        "max_output_bytes" => {
            check.max_output_bytes = Some(parse_usize_field(
                "verification max_output_bytes",
                value.trim(),
            )?)
        }
        "inherit_env" => check.inherit_env = Some(parse_bool_field(value)?),
        "env" => check.env = parse_env_field(value, line_no)?,
        "allow_failure" => check.allow_failure = Some(parse_bool_field(value)?),
        _ => {
            return Err(CoreError::Config(format!(
                "unsupported codefire.policy.yaml: verification field {key} at line {line_no}"
            )));
        }
    }
    Ok(())
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

pub fn extract_markdown_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut headings = Vec::new();
    let mut open_fence = None::<(char, usize)>;
    for (line_index, line) in lines.iter().enumerate() {
        if let Some((fence_char, fence_len)) = markdown_fence_marker(line) {
            if let Some((open_char, open_len)) = open_fence {
                if fence_char == open_char && fence_len >= open_len {
                    open_fence = None;
                }
            } else {
                open_fence = Some((fence_char, fence_len));
            }
            continue;
        }
        if open_fence.is_some() {
            continue;
        }
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

fn markdown_fence_marker(line: &str) -> Option<(char, usize)> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let trimmed = &line[indent..];
    let fence_char = trimmed.chars().next()?;
    if !matches!(fence_char, '`' | '~') {
        return None;
    }
    let fence_len = trimmed.chars().take_while(|ch| *ch == fence_char).count();
    (fence_len >= 3).then_some((fence_char, fence_len))
}

const PYTHON_EXPLICIT_MARKER_LOOKBACK_LINES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PythonSymbolKind {
    Class,
    Def,
    Method,
}

impl PythonSymbolKind {
    fn id_component(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Def => "def",
            Self::Method => "method",
        }
    }

    fn selector_type(self) -> &'static str {
        match self {
            Self::Class => "python_class",
            Self::Def => "python_def",
            Self::Method => "python_method",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PythonSymbolBlock {
    start: usize,
    end: usize,
    kind: PythonSymbolKind,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PythonStringState {
    Continued(u8),
    Triple(u8),
}

impl PythonStringState {
    fn quote(self) -> u8 {
        match self {
            Self::Continued(quote) | Self::Triple(quote) => quote,
        }
    }

    fn width(self) -> usize {
        match self {
            Self::Continued(_) => 1,
            Self::Triple(_) => 3,
        }
    }
}

#[derive(Debug, Default)]
struct PythonLexState {
    string: Option<PythonStringState>,
}

#[derive(Debug)]
struct PythonLexLine {
    code: String,
    comment_start: Option<usize>,
}

/// Extracts the supported Python symbol profile used by the reference implementation.
///
/// Explicit `cf-atom` markers in the three lines immediately preceding a symbol (or on the
/// symbol line) own that symbol once and replace its derived ID. Other markers remain standalone
/// comment atoms. Symbol and marker scans are linear in the source size; only the constant-size
/// marker ownership window is searched per symbol.
pub fn extract_python_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let lexical_lines = python_lexical_lines(&lines);
    let mut explicit_by_line = BTreeMap::new();
    for (line_index, (line, lexical)) in lines.iter().zip(&lexical_lines).enumerate() {
        let Some(comment_start) = lexical.comment_start else {
            continue;
        };
        if let Some(atom_id) =
            explicit_cf_atom_id(&line[comment_start..], artifact_path, line_index + 1)?
        {
            explicit_by_line.insert(line_index, atom_id);
        }
    }

    let blocks = python_symbol_blocks(&lines, &lexical_lines);
    let mut used_explicit = HashSet::with_capacity(explicit_by_line.len());
    let mut atoms = Vec::with_capacity(blocks.len().saturating_add(explicit_by_line.len()));
    for block in blocks {
        let first_candidate = block
            .start
            .saturating_sub(PYTHON_EXPLICIT_MARKER_LOOKBACK_LINES);
        let explicit_line = (first_candidate..=block.start).rev().find(|line_index| {
            explicit_by_line.contains_key(line_index) && !used_explicit.contains(line_index)
        });
        let atom_id = if let Some(line_index) = explicit_line {
            used_explicit.insert(line_index);
            explicit_by_line
                .get(&line_index)
                .expect("selected explicit marker exists")
                .clone()
        } else {
            let prefix = if fallback_kind == "test" {
                "TEST"
            } else {
                "CODE"
            };
            format!(
                "{prefix}:{artifact_path}::{}:{}",
                block.kind.id_component(),
                block.name
            )
        };
        let content_start = explicit_line.unwrap_or(block.start);
        let content = normalized_block_content(&lines[content_start..block.end]);
        atoms.push(Atom {
            kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: block.kind.selector_type().to_string(),
                value: block.name,
            },
            content_hash: hash_text(&content),
            atom_id,
        });
    }

    for (line_index, atom_id) in explicit_by_line {
        if used_explicit.contains(&line_index) {
            continue;
        }
        let content = normalized_block_content(&lines[line_index..line_index + 1]);
        atoms.push(Atom {
            kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "python_comment".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
            atom_id,
        });
    }
    Ok(atoms)
}

fn python_lexical_lines(lines: &[&str]) -> Vec<PythonLexLine> {
    let mut state = PythonLexState::default();
    lines
        .iter()
        .map(|line| python_lex_line(line, &mut state))
        .collect()
}

fn python_lex_line(line: &str, state: &mut PythonLexState) -> PythonLexLine {
    let bytes = line.as_bytes();
    let mut masked = bytes.to_vec();
    let mut index = 0usize;
    let mut comment_start = None;

    if let Some(open) = state.string {
        let quote = open.quote();
        let width = open.width();
        if let Some(end) = find_python_quote_end(bytes, 0, quote, width) {
            mask_python_bytes(&mut masked, 0, end);
            index = end;
            state.string = None;
        } else {
            mask_python_bytes(&mut masked, 0, bytes.len());
            if matches!(open, PythonStringState::Continued(_))
                && !python_line_continues_string(bytes)
            {
                state.string = None;
            }
            return PythonLexLine {
                code: String::from_utf8(masked).expect("masking preserves UTF-8"),
                comment_start,
            };
        }
    }

    while index < bytes.len() {
        match bytes[index] {
            b'#' => {
                comment_start = Some(index);
                mask_python_bytes(&mut masked, index, bytes.len());
                break;
            }
            quote @ (b'\'' | b'"') => {
                let width = if bytes.get(index + 1) == Some(&quote)
                    && bytes.get(index + 2) == Some(&quote)
                {
                    3
                } else {
                    1
                };
                let prefix_start = python_string_prefix_start(bytes, index);
                if let Some(end) = find_python_quote_end(bytes, index + width, quote, width) {
                    mask_python_bytes(&mut masked, prefix_start, end);
                    index = end;
                } else {
                    mask_python_bytes(&mut masked, prefix_start, bytes.len());
                    state.string = match width {
                        3 => Some(PythonStringState::Triple(quote)),
                        1 if python_line_continues_string(bytes) => {
                            Some(PythonStringState::Continued(quote))
                        }
                        _ => None,
                    };
                    break;
                }
            }
            _ => index += 1,
        }
    }

    PythonLexLine {
        code: String::from_utf8(masked).expect("masking preserves UTF-8"),
        comment_start,
    }
}

fn find_python_quote_end(bytes: &[u8], mut index: usize, quote: u8, width: usize) -> Option<usize> {
    while index.saturating_add(width) <= bytes.len() {
        if bytes[index..index + width]
            .iter()
            .all(|candidate| *candidate == quote)
            && !python_quote_is_escaped(bytes, index)
        {
            return Some(index + width);
        }
        index += 1;
    }
    None
}

fn python_quote_is_escaped(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    let mut backslashes = 0usize;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        cursor -= 1;
        backslashes += 1;
    }
    backslashes % 2 == 1
}

fn python_line_continues_string(bytes: &[u8]) -> bool {
    let mut backslashes = 0usize;
    for byte in bytes.iter().rev() {
        if *byte != b'\\' {
            break;
        }
        backslashes += 1;
    }
    backslashes % 2 == 1
}

fn python_string_prefix_start(bytes: &[u8], quote_index: usize) -> usize {
    let mut start = quote_index;
    while start > 0
        && quote_index - start < 2
        && matches!(
            bytes[start - 1],
            b'r' | b'R' | b'b' | b'B' | b'u' | b'U' | b'f' | b'F'
        )
    {
        start -= 1;
    }
    if start == quote_index {
        return start;
    }
    if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        quote_index
    } else {
        start
    }
}

fn mask_python_bytes(bytes: &mut [u8], start: usize, end: usize) {
    bytes[start..end].fill(b' ');
}

fn python_symbol_blocks(lines: &[&str], lexical_lines: &[PythonLexLine]) -> Vec<PythonSymbolBlock> {
    let mut blocks = Vec::<PythonSymbolBlock>::new();
    let mut open_blocks = Vec::<(usize, usize)>::new();
    let mut class_stack = Vec::<(usize, String)>::new();

    for (line_index, lexical) in lexical_lines.iter().enumerate() {
        let code = lexical.code.as_str();
        if python_block_boundary(code) {
            let boundary_indent = leading_space_count(code);
            while open_blocks
                .last()
                .is_some_and(|(indent, _)| boundary_indent <= *indent)
            {
                let (_, block_index) = open_blocks.pop().expect("checked as non-empty");
                blocks[block_index].end = line_index;
            }
        }

        let Some((indent, is_class, symbol_name)) = python_symbol_declaration(code) else {
            continue;
        };
        while class_stack
            .last()
            .is_some_and(|(class_indent, _)| indent <= *class_indent)
        {
            class_stack.pop();
        }
        let (kind, name) = if is_class {
            (PythonSymbolKind::Class, symbol_name.clone())
        } else if let Some((_, class_name)) = class_stack.last() {
            (
                PythonSymbolKind::Method,
                format!("{class_name}.{symbol_name}"),
            )
        } else {
            (PythonSymbolKind::Def, symbol_name.clone())
        };
        let block_index = blocks.len();
        blocks.push(PythonSymbolBlock {
            start: line_index,
            end: lines.len(),
            kind,
            name,
        });
        open_blocks.push((indent, block_index));
        if is_class {
            class_stack.push((indent, symbol_name));
        }
    }
    debug_assert_eq!(lines.len(), lexical_lines.len());
    blocks
}

fn python_block_boundary(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('@')
}

fn python_symbol_declaration(line: &str) -> Option<(usize, bool, String)> {
    let indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
    let trimmed = line.trim_start();
    let (is_class, after_keyword) = python_keyword_tail(trimmed, "class")
        .map(|tail| (true, tail))
        .or_else(|| python_keyword_tail(trimmed, "def").map(|tail| (false, tail)))
        .or_else(|| {
            python_keyword_tail(trimmed, "async")
                .and_then(|tail| python_keyword_tail(tail, "def"))
                .map(|tail| (false, tail))
        })?;
    let mut chars = after_keyword.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    let mut name = String::from(first);
    name.extend(chars.take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_'));
    Some((indent, is_class, name))
}

fn python_keyword_tail<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let tail = line.strip_prefix(keyword)?;
    let first = tail.chars().next()?;
    first.is_whitespace().then(|| tail.trim_start())
}

pub fn extract_explicit_cf_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut markers = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some(atom_id) = explicit_cf_atom_id(line, artifact_path, line_index + 1)? {
            markers.push((line_index, atom_id));
        }
    }
    let mut atoms = Vec::with_capacity(markers.len());
    for (index, (start, atom_id)) in markers.iter().enumerate() {
        let marker_end = markers
            .get(index + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(lines.len());
        let end = explicit_cf_atom_span_end(&lines, *start, marker_end, artifact_path);
        let content = normalized_block_content(&lines[*start..end]);
        atoms.push(Atom {
            atom_id: atom_id.clone(),
            kind: kind_from_atom_id(atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "explicit_cf_atom".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
        });
    }
    Ok(atoms)
}

fn explicit_cf_atom_span_end(
    lines: &[&str],
    marker_start: usize,
    marker_end: usize,
    artifact_path: &str,
) -> usize {
    if artifact_path.ends_with(".py") {
        python_explicit_cf_atom_span_end(lines, marker_start, marker_end).unwrap_or(marker_end)
    } else {
        marker_end
    }
}

fn python_explicit_cf_atom_span_end(
    lines: &[&str],
    marker_start: usize,
    marker_end: usize,
) -> Option<usize> {
    let mut index = marker_start + 1;
    while index < marker_end {
        let trimmed = lines[index].trim_start();
        if trimmed.trim().is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }
        if trimmed.starts_with('@') {
            let decorator_indent = leading_space_count(lines[index]);
            while index < marker_end && lines[index].trim_start().starts_with('@') {
                index += 1;
            }
            if index < marker_end && is_python_symbol_line(lines[index].trim_start()) {
                let symbol_indent = leading_space_count(lines[index]).min(decorator_indent);
                return Some(python_block_end(lines, index, marker_end, symbol_indent));
            }
            return None;
        }
        if is_python_symbol_line(trimmed) {
            return Some(python_block_end(
                lines,
                index,
                marker_end,
                leading_space_count(lines[index]),
            ));
        }
        return None;
    }
    None
}

fn is_python_symbol_line(trimmed: &str) -> bool {
    trimmed.starts_with("class ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("async def ")
}

fn python_block_end(
    lines: &[&str],
    symbol_start: usize,
    marker_end: usize,
    block_indent: usize,
) -> usize {
    let mut end = symbol_start + 1;
    for (index, line) in lines
        .iter()
        .enumerate()
        .take(marker_end)
        .skip(symbol_start + 1)
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            end = index + 1;
            continue;
        }
        let indent = leading_space_count(line);
        if indent <= block_indent {
            return trim_trailing_blank_lines(lines, end);
        }
        end = index + 1;
    }
    trim_trailing_blank_lines(lines, end)
}

fn trim_trailing_blank_lines(lines: &[&str], mut end: usize) -> usize {
    while end > 0 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

fn leading_space_count(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
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
        .take_while(|ch| ch.is_ascii_uppercase() || is_dash_atom_tail_char(*ch))
        .collect();
    if !is_valid_atom_id(&atom_id) {
        return None;
    }
    if !is_auto_markdown_atom_id(&atom_id) {
        return None;
    }
    let next = rest.chars().nth(atom_id.len());
    if matches!(next, None | Some(':') | Some(' ') | Some('\t')) {
        Some((level, atom_id))
    } else {
        None
    }
}

fn is_auto_markdown_atom_id(atom_id: &str) -> bool {
    if atom_id.starts_with("PROC-") || atom_id.contains("-PROC-") {
        return false;
    }
    let Some((prefix, _)) = atom_id.split_once('-') else {
        return atom_id.starts_with("CODE:");
    };
    matches!(
        prefix,
        "REQ" | "DES" | "TEST" | "ADR" | "OPS" | "API" | "DB"
    )
}

fn is_atom_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-' | ':' | '/')
}

pub fn is_valid_atom_id(atom_id: &str) -> bool {
    is_valid_dash_atom_id(atom_id) || is_valid_code_path_atom_id(atom_id)
}

fn is_valid_dash_atom_id(atom_id: &str) -> bool {
    let Some((prefix, rest)) = atom_id.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_uppercase())
        && !rest.is_empty()
        && rest.chars().all(is_dash_atom_tail_char)
}

fn is_valid_code_path_atom_id(atom_id: &str) -> bool {
    let Some(rest) = atom_id.strip_prefix("CODE:") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(is_code_path_atom_tail_char)
}

fn is_dash_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')
}

fn is_code_path_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-' | ':' | '/')
}

fn explicit_cf_atom_id(
    line: &str,
    artifact_path: &str,
    line_number: usize,
) -> Result<Option<String>, CoreError> {
    let Some((_, after)) = line.split_once("cf-atom:") else {
        return Ok(None);
    };
    let atom_id: String = after
        .trim_start()
        .chars()
        .take_while(|ch| is_atom_id_char(*ch))
        .collect();
    if atom_id.is_empty() {
        return Err(CoreError::Config(format!(
            "invalid Atom ID in {artifact_path}:{line_number}: cf-atom marker has no Atom ID"
        )));
    }
    if !is_valid_atom_id(&atom_id) {
        return Err(CoreError::Config(format!(
            "invalid Atom ID in {artifact_path}:{line_number}: {atom_id}"
        )));
    }
    Ok(Some(atom_id))
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
    fn markdown_atoms_ignore_headings_inside_fenced_code_blocks() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("spec.md");
        fs::write(
            &path,
            "## REQ-REAL-001: Real requirement\n\n```markdown\n## REQ-FAKE-001: sample only\n```\n\n   ~~~text\n### DES-FAKE-001: sample only\n   ~~~\n\n## REQ-REAL-002: Another requirement\n",
        )
        .unwrap();

        let atoms = extract_markdown_atoms(&path, "docs/spec/fenced.md", "requirement").unwrap();
        let ids = atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["REQ-REAL-001", "REQ-REAL-002"]);
    }

    #[test]
    fn markdown_atoms_ignore_process_style_headings_without_explicit_marker() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("process.md");
        fs::write(
            &path,
            "## REQ-PROC-001: Process note\n\nNot a CodeFire atom.\n\n## PROC-REQ-001: Local process id\n\nAlso not an atom.\n\n## REQ-REAL-001: Real requirement\n\nBody\n",
        )
        .unwrap();

        let atoms = extract_markdown_atoms(&path, "docs/process.md", "requirement").unwrap();
        let ids = atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["REQ-REAL-001"]);

        let explicit_path = temp.path().join("explicit.md");
        fs::write(
            &explicit_path,
            "<!-- cf-atom: REQ-PROC-001 -->\n## REQ-PROC-001: Explicit process requirement\n",
        )
        .unwrap();
        let explicit =
            extract_explicit_cf_atoms(&explicit_path, "docs/explicit.md", "requirement").unwrap();
        assert_eq!(explicit[0].atom_id, "REQ-PROC-001");
    }

    #[test]
    fn explicit_cf_atoms_extract_comment_markers_without_symbol_parsing() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("app.py");
        fs::write(
            &path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n# cf-atom: CODE:src/app.py::function:run\n",
        )
        .unwrap();

        let atoms = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

        assert_eq!(atoms.len(), 2);
        assert_eq!(atoms[0].atom_id, "CODE-SessionPolicy");
        assert_eq!(atoms[0].kind, "code");
        assert_eq!(atoms[0].selector.selector_type, "explicit_cf_atom");
        assert_eq!(atoms[1].atom_id, "CODE:src/app.py::function:run");
        assert_eq!(
            atoms[0].content_hash,
            "sha256:01e54e8f2a22283b0e25e51c1571921c2c51af29c5f8fc24517aecdbdaddce1f"
        );
    }

    #[test]
    fn python_derived_atoms_restore_reference_ids_selectors_and_hashes() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("models.py");
        fs::write(
            &path,
            "class ExperimentPlan:\n    hypothesis: Hypothesis\n    benchmark: str\n    baseline: str\n    metric: str\n    estimated_cost: float\n    command: str = \"\"\n    artifact_paths: tuple[str, ...] = field(default_factory=tuple)\n    plan_id: str = \"\"\n\n    @property\n    def uid(self) -> str:\n        return self.plan_id or stable_id(\n            \"plan\",\n            self.hypothesis.uid,\n            self.benchmark,\n            self.baseline,\n            self.metric,\n            self.command,\n        )\n\n\n@dataclass(frozen=True)\nclass ExperimentResult:\n    pass\n",
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "src/evoagent/models.py", "code").unwrap();
        let class = atoms
            .iter()
            .find(|atom| atom.atom_id == "CODE:src/evoagent/models.py::class:ExperimentPlan")
            .unwrap();
        let method = atoms
            .iter()
            .find(|atom| atom.atom_id == "CODE:src/evoagent/models.py::method:ExperimentPlan.uid")
            .unwrap();

        assert_eq!(class.selector.selector_type, "python_class");
        assert_eq!(class.selector.value, "ExperimentPlan");
        assert_eq!(
            class.content_hash,
            "sha256:5f8966552eba40250bfc050d5823b8a43194c6ce18062771759e7fa3e2a73bd9"
        );
        assert_eq!(method.selector.selector_type, "python_method");
        assert_eq!(method.selector.value, "ExperimentPlan.uid");
    }

    #[test]
    fn python_derived_atoms_distinguish_functions_and_class_owned_methods() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("planners.py");
        fs::write(
            &path,
            "def build_planner():\n    return StaticPlanner()\n\n\nclass StaticPlanner:\n    def propose(self):\n        return []\n\n\nclass CodexPlanner:\n    def propose(self):\n        return []\n",
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "src/planners.py", "code").unwrap();
        let ids = atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"CODE:src/planners.py::def:build_planner"));
        assert!(ids.contains(&"CODE:src/planners.py::class:StaticPlanner"));
        assert!(ids.contains(&"CODE:src/planners.py::method:StaticPlanner.propose"));
        assert!(ids.contains(&"CODE:src/planners.py::class:CodexPlanner"));
        assert!(ids.contains(&"CODE:src/planners.py::method:CodexPlanner.propose"));
    }

    #[test]
    fn python_test_symbols_use_the_reference_test_prefix() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test_planner.py");
        fs::write(
            &path,
            "class PlannerTests:\n    def test_plan(self):\n        assert True\n",
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "tests/test_planner.py", "test").unwrap();
        let ids = atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"TEST:tests/test_planner.py::class:PlannerTests"));
        assert!(ids.contains(&"TEST:tests/test_planner.py::method:PlannerTests.test_plan"));
        assert!(atoms.iter().all(|atom| atom.kind == "test"));
    }

    #[test]
    fn python_explicit_marker_keeps_async_function_body_ownership() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("async_worker.py");
        fs::write(
            &path,
            "# cf-atom: CODE-AsyncWorker\nasync def run_worker():\n    return 1\n",
        )
        .unwrap();
        let before = extract_python_atoms(&path, "src/async_worker.py", "code").unwrap();
        fs::write(
            &path,
            "# cf-atom: CODE-AsyncWorker\nasync def run_worker():\n    return 2\n",
        )
        .unwrap();
        let after = extract_python_atoms(&path, "src/async_worker.py", "code").unwrap();

        assert_eq!(before.len(), 1);
        assert_eq!(after.len(), 1);
        assert_eq!(before[0].atom_id, "CODE-AsyncWorker");
        assert_eq!(before[0].selector.selector_type, "python_def");
        assert_eq!(before[0].selector.value, "run_worker");
        assert_ne!(before[0].content_hash, after[0].content_hash);
    }

    #[test]
    fn python_explicit_marker_owns_only_nearest_symbol_once() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("owned.py");
        fs::write(
            &path,
            "# cf-atom: CODE-UnusedMarker\n# cf-atom: CODE-OwnedClass\n@dataclass\nclass Owned:\n    def run(self):\n        return True\n",
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "src/owned.py", "code").unwrap();
        let owned = atoms
            .iter()
            .find(|atom| atom.atom_id == "CODE-OwnedClass")
            .unwrap();

        assert_eq!(owned.selector.selector_type, "python_class");
        assert_eq!(owned.selector.value, "Owned");
        assert!(atoms.iter().any(|atom| atom.atom_id == "CODE-UnusedMarker"
            && atom.selector.selector_type == "python_comment"));
        assert!(atoms
            .iter()
            .any(|atom| atom.atom_id == "CODE:src/owned.py::method:Owned.run"));
        assert!(!atoms
            .iter()
            .any(|atom| atom.atom_id == "CODE:src/owned.py::class:Owned"));
    }

    #[test]
    fn python_explicit_marker_ownership_window_is_bounded() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("bounded.py");
        fs::write(
            &path,
            "# cf-atom: CODE-Standalone\n# first gap\n# second gap\n# third gap\nclass FourLinesAway:\n    pass\n",
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "src/bounded.py", "code").unwrap();

        assert!(atoms.iter().any(|atom| atom.atom_id == "CODE-Standalone"
            && atom.selector.selector_type == "python_comment"));
        assert!(atoms
            .iter()
            .any(|atom| atom.atom_id == "CODE:src/bounded.py::class:FourLinesAway"));
    }

    #[test]
    fn python_lexical_view_ignores_declarations_and_markers_inside_strings() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test_nameaudit.py");
        fs::write(
            &path,
            r##"class AuditTests:
    def test_scan(self):
        inline = "class InlineFixture:"
        source = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
# cf-atom: TEST-FakeMarker
"""
        raw_source = rf'''
async def false_async_fixture():
    return "ignored"
'''
        assert inline and source and raw_source

# cf-atom: TEST-RealFunction
def real_function():
    return True
"##,
        )
        .unwrap();

        let atoms = extract_python_atoms(&path, "tests/test_nameaudit.py", "test").unwrap();
        let ids = atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"TEST:tests/test_nameaudit.py::class:AuditTests"));
        assert!(ids.contains(&"TEST:tests/test_nameaudit.py::method:AuditTests.test_scan"));
        assert!(ids.contains(&"TEST-RealFunction"));
        assert!(!ids.iter().any(|atom_id| {
            atom_id.contains("InlineFixture")
                || atom_id.contains("ExcessivelyLongClassName")
                || atom_id.contains("excessively_long_method")
                || atom_id.contains("false_async_fixture")
                || *atom_id == "TEST-FakeMarker"
        }));
    }

    #[test]
    fn python_derived_and_explicit_atoms_keep_duplicate_detection() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("src/models.py"),
            "class ExperimentPlan:\n    pass\n",
        );
        write_file(
            &temp.path().join("src/duplicate.py"),
            "# cf-atom: CODE:src/models.py::class:ExperimentPlan\n",
        );

        let index = build_atom_index(temp.path()).unwrap();

        assert_eq!(index.hash_schema_version, 4);
        assert_eq!(
            index.duplicate_atom_ids,
            vec!["CODE:src/models.py::class:ExperimentPlan"]
        );
    }

    #[test]
    fn python_symbol_extraction_preserves_invalid_marker_diagnostics() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("invalid.py");
        fs::write(
            &path,
            "# cf-atom: code-invalid\nclass StillNotExtracted:\n    pass\n",
        )
        .unwrap();

        let error = extract_python_atoms(&path, "src/invalid.py", "code").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid Atom ID in src/invalid.py:1: code-invalid"
        );
    }

    #[test]
    fn explicit_cf_atom_body_changes_update_content_hash_for_python_and_rust() {
        let temp = tempdir().unwrap();
        let python_path = temp.path().join("app.py");
        fs::write(
            &python_path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    timeout = 30\n",
        )
        .unwrap();
        let first_python =
            extract_explicit_cf_atoms(&python_path, "src/app.py", "code").unwrap()[0].clone();
        fs::write(
            &python_path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    timeout = 60\n",
        )
        .unwrap();
        let second_python =
            extract_explicit_cf_atoms(&python_path, "src/app.py", "code").unwrap()[0].clone();

        let rust_path = temp.path().join("lib.rs");
        fs::write(
            &rust_path,
            "// cf-atom: CODE-RustPolicy\nfn timeout() -> u32 {\n    30\n}\n",
        )
        .unwrap();
        let first_rust =
            extract_explicit_cf_atoms(&rust_path, "src/lib.rs", "code").unwrap()[0].clone();
        fs::write(
            &rust_path,
            "// cf-atom: CODE-RustPolicy\nfn timeout() -> u32 {\n    60\n}\n",
        )
        .unwrap();
        let second_rust =
            extract_explicit_cf_atoms(&rust_path, "src/lib.rs", "code").unwrap()[0].clone();

        assert_eq!(first_python.atom_id, second_python.atom_id);
        assert_ne!(first_python.content_hash, second_python.content_hash);
        assert_eq!(first_rust.atom_id, second_rust.atom_id);
        assert_ne!(first_rust.content_hash, second_rust.content_hash);
    }

    #[test]
    fn python_explicit_cf_atom_hash_is_stable_when_later_marker_is_added() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("app.py");
        fs::write(
            &path,
            "# cf-atom: CODE-ExistingPolicy\nclass ExistingPolicy:\n    def allowed(self) -> bool:\n        return True\n\n\ndef helper() -> int:\n    return 1\n",
        )
        .unwrap();
        let before = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();
        fs::write(
            &path,
            "# cf-atom: CODE-ExistingPolicy\nclass ExistingPolicy:\n    def allowed(self) -> bool:\n        return True\n\n\n# cf-atom: CODE-Helper\ndef helper() -> int:\n    return 1\n",
        )
        .unwrap();
        let after = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

        assert_eq!(before.len(), 1);
        assert_eq!(after.len(), 2);
        assert_eq!(before[0].atom_id, "CODE-ExistingPolicy");
        assert_eq!(after[0].atom_id, "CODE-ExistingPolicy");
        assert_eq!(before[0].content_hash, after[0].content_hash);
        assert_eq!(after[1].atom_id, "CODE-Helper");
    }

    #[test]
    fn python_explicit_cf_atom_hash_tracks_symbol_body_changes() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test_app.py");
        fs::write(
            &path,
            "# cf-atom: TEST-existing-policy\ndef test_existing_policy():\n    assert True\n\n\ndef unrelated_helper():\n    return 1\n",
        )
        .unwrap();
        let before = extract_explicit_cf_atoms(&path, "tests/test_app.py", "test").unwrap();
        fs::write(
            &path,
            "# cf-atom: TEST-existing-policy\ndef test_existing_policy():\n    assert False\n\n\ndef unrelated_helper():\n    return 1\n",
        )
        .unwrap();
        let after = extract_explicit_cf_atoms(&path, "tests/test_app.py", "test").unwrap();

        assert_eq!(before[0].atom_id, after[0].atom_id);
        assert_ne!(before[0].content_hash, after[0].content_hash);
    }

    #[test]
    fn invalid_explicit_cf_atom_id_is_reported_with_location() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("app.py");
        fs::write(&path, "# cf-atom: code-lowercase\n").unwrap();

        let error = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid Atom ID in src/app.py:1: code-lowercase"
        );
    }

    #[test]
    fn atom_id_validator_is_shared_by_heading_and_explicit_forms() {
        assert!(is_valid_atom_id("REQ-AUTH-001"));
        assert!(is_valid_atom_id("CODE-SessionPolicy"));
        assert!(is_valid_atom_id("CODE:src/app.py::method:Store.find"));
        assert!(!is_valid_atom_id("code-lowercase"));
        assert!(!is_valid_atom_id("REQ-"));
        assert!(!is_valid_atom_id("CODE:src/app.py::method:bad name"));
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
    fn trace_graph_parses_links_and_uses_bounded_digest_ids() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.links.yaml"),
            "version: 1\nlinks:\n  - from: 'REQ-AUTH-001' # comment\n    to: \"DES-AUTH-001\"\n    type: refined_by\n",
        );

        let graph = current_trace_graph(temp.path()).unwrap();

        assert_eq!(graph.type_tag, "trace_graph");
        assert_eq!(graph.links.len(), 1);
        assert_eq!(graph.links[0].from, "REQ-AUTH-001");
        assert_eq!(graph.links[0].to, "DES-AUTH-001");
        assert_eq!(graph.links[0].link_type, "refined_by");
        assert!(graph.links[0].link_id.starts_with("LINK-sha256-"));
        assert_eq!(graph.links[0].link_id.len(), "LINK-sha256-".len() + 32);
        assert_eq!(
            graph.links[0].link_hash,
            trace_link_hash(
                "REQ-AUTH-001",
                "DES-AUTH-001",
                "refined_by",
                &graph.links[0].link_id,
            )
            .unwrap()
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
    fn trace_graph_rejects_unsupported_links_yaml() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.links.yaml"),
            "version: 1\nmetadata:\n  owner: docs\nlinks:\n  - from: REQ-AUTH-001\n",
        );

        let error = current_trace_graph(temp.path()).unwrap_err();

        assert!(
            matches!(error, CoreError::Config(message) if message.contains("unsupported codefire.links.yaml: root key metadata at line 2"))
        );
    }

    #[test]
    fn trace_graph_is_empty_when_links_file_is_absent() {
        let temp = tempdir().unwrap();

        let graph = current_trace_graph(temp.path()).unwrap();

        assert!(graph.links.is_empty());
    }

    #[test]
    fn trace_graph_endpoint_validator_reports_missing_source() {
        let target = atom("DES-AUTH-001", "design", "sha256:design");
        let link = TraceLinkInput {
            from: "REQ-MISSING-001".to_string(),
            to: Some(target.atom_id.clone()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap();
        let link_id = link.link_id.clone();
        let atom_index = atom_index(vec![target]);
        let trace_graph = trace_graph(vec![link]);

        let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "trace graph endpoint integrity failed: 1 issue(s); link {link_id} from endpoint REQ-MISSING-001 does not resolve to an Atom"
            )
        );
        assert!(matches!(
            error,
            CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
                issues,
                total_issues: 1,
                omitted_issues: 0,
            })
                if issues == vec![TraceGraphEndpointIssue {
                    link_index: 0,
                    link_id,
                    link_id_truncated: false,
                    link_id_omitted_bytes: 0,
                    endpoint: TraceGraphEndpointRole::From,
                    atom_id: "REQ-MISSING-001".to_string(),
                    atom_id_truncated: false,
                    atom_id_omitted_bytes: 0,
                    kind: TraceGraphEndpointIssueKind::MissingAtom,
                    atom_occurrences: 0,
                }]
        ));
    }

    #[test]
    fn trace_graph_endpoint_validator_reports_missing_target() {
        let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
        let link = TraceLinkInput {
            from: source.atom_id.clone(),
            to: Some("DES-MISSING-001".to_string()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap();
        let link_id = link.link_id.clone();
        let atom_index = atom_index(vec![source]);
        let trace_graph = trace_graph(vec![link]);

        let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

        assert!(matches!(
            error,
            CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
                issues,
                total_issues: 1,
                omitted_issues: 0,
            })
                if issues == vec![TraceGraphEndpointIssue {
                    link_index: 0,
                    link_id,
                    link_id_truncated: false,
                    link_id_omitted_bytes: 0,
                    endpoint: TraceGraphEndpointRole::To,
                    atom_id: "DES-MISSING-001".to_string(),
                    atom_id_truncated: false,
                    atom_id_omitted_bytes: 0,
                    kind: TraceGraphEndpointIssueKind::MissingAtom,
                    atom_occurrences: 0,
                }]
        ));
    }

    #[test]
    fn trace_graph_endpoint_validator_reports_duplicate_atom_id_ambiguity() {
        let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
        let duplicate_target = atom("DES-AUTH-001", "design", "sha256:design-a");
        let link = TraceLinkInput {
            from: source.atom_id.clone(),
            to: Some(duplicate_target.atom_id.clone()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap();
        let link_id = link.link_id.clone();
        let atom_index = atom_index(vec![
            source,
            duplicate_target,
            atom("DES-AUTH-001", "design", "sha256:design-b"),
        ]);
        let trace_graph = trace_graph(vec![link]);

        let error = validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap_err();

        assert!(matches!(
            error,
            CoreError::TraceGraphEndpointIntegrity(TraceGraphEndpointError {
                issues,
                total_issues: 1,
                omitted_issues: 0,
            })
                if issues == vec![TraceGraphEndpointIssue {
                    link_index: 0,
                    link_id,
                    link_id_truncated: false,
                    link_id_omitted_bytes: 0,
                    endpoint: TraceGraphEndpointRole::To,
                    atom_id: "DES-AUTH-001".to_string(),
                    atom_id_truncated: false,
                    atom_id_omitted_bytes: 0,
                    kind: TraceGraphEndpointIssueKind::AmbiguousAtomId,
                    atom_occurrences: 2,
                }]
        ));
    }

    #[test]
    fn trace_graph_endpoint_validator_accepts_uniquely_resolved_graph() {
        let source = atom("REQ-AUTH-001", "requirement", "sha256:requirement");
        let target = atom("DES-AUTH-001", "design", "sha256:design");
        let link = TraceLinkInput {
            from: source.atom_id.clone(),
            to: Some(target.atom_id.clone()),
            link_type: Some("refined_by".to_string()),
        }
        .into_trace_link()
        .unwrap();
        let atom_index = atom_index(vec![source, target]);
        let trace_graph = trace_graph(vec![link]);

        validate_trace_graph_endpoints(&atom_index, &trace_graph).unwrap();
    }

    #[test]
    fn trace_graph_endpoint_validator_bounds_and_orders_large_diagnostics() {
        let link_count = TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT + 37;
        let links = (0..link_count)
            .rev()
            .map(|index| TraceLink {
                from: format!("REQ-MISSING-{index:04}"),
                to: format!("DES-MISSING-{index:04}"),
                link_type: "refined_by".to_string(),
                link_id: format!("LINK-{index:04}"),
                link_hash: format!("sha256:{index:04}"),
            })
            .collect::<Vec<_>>();

        let error = validate_trace_graph_endpoints(&atom_index(Vec::new()), &trace_graph(links))
            .unwrap_err();
        let CoreError::TraceGraphEndpointIntegrity(error) = error else {
            panic!("expected trace graph endpoint integrity error");
        };

        assert_eq!(error.issues.len(), TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT);
        assert_eq!(error.total_issues, link_count * 2);
        assert_eq!(
            error.omitted_issues,
            link_count * 2 - TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT
        );
        assert!(error
            .issues
            .windows(2)
            .all(|pair| compare_endpoint_issues(&pair[0], &pair[1]).is_lt()));
        assert_eq!(error.issues.first().unwrap().link_id, "LINK-0000");
        assert!(error.to_string().ends_with(&format!(
            "{} additional issue(s) omitted",
            error.omitted_issues
        )));

        // Display is defensive even if an external caller constructs an overfull error value.
        let mut overfull_error = error.clone();
        let repeated_issue = overfull_error.issues[0].clone();
        for _ in 0..7 {
            overfull_error.issues.push(repeated_issue.clone());
        }
        overfull_error.total_issues = overfull_error.issues.len();
        overfull_error.omitted_issues = 0;
        let rendered = overfull_error.to_string();
        assert_eq!(
            rendered.matches("; link ").count(),
            TRACE_GRAPH_ENDPOINT_ISSUE_LIMIT
        );
        assert!(rendered.ends_with("7 additional issue(s) omitted"));
    }

    #[test]
    fn trace_graph_endpoint_validator_byte_bounds_utf8_diagnostic_samples() {
        let long_link_id = format!("LINK-{}{}", "a".repeat(250), "界".repeat(20));
        let long_atom_id = format!("REQ-{}{}", "b".repeat(251), "界".repeat(20));
        let (atom_prefix, _) = bounded_endpoint_sample(&long_atom_id);
        let target = atom("DES-target", "design", "sha256:target");
        let graph = trace_graph(vec![TraceLink {
            from: long_atom_id.clone(),
            to: target.atom_id.clone(),
            link_type: "refined_by".to_string(),
            link_id: long_link_id.clone(),
            link_hash: "sha256:long-diagnostic".to_string(),
        }]);
        // A bounded display prefix must not become the endpoint identity used by validation.
        let index = atom_index(vec![
            atom(&atom_prefix, "requirement", "sha256:prefix-only"),
            target,
        ]);

        let CoreError::TraceGraphEndpointIntegrity(error) =
            validate_trace_graph_endpoints(&index, &graph).unwrap_err()
        else {
            panic!("expected trace graph endpoint integrity error");
        };
        let issue = &error.issues[0];

        assert_eq!(error.total_issues, 1);
        assert!(issue.link_id_truncated);
        assert!(issue.atom_id_truncated);
        assert!(issue.link_id.len() <= TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT);
        assert!(issue.atom_id.len() <= TRACE_GRAPH_ENDPOINT_SAMPLE_BYTE_LIMIT);
        assert_eq!(
            issue.link_id_omitted_bytes,
            long_link_id.len() - issue.link_id.len()
        );
        assert_eq!(
            issue.atom_id_omitted_bytes,
            long_atom_id.len() - issue.atom_id.len()
        );
        assert!(std::str::from_utf8(issue.link_id.as_bytes()).is_ok());
        assert!(std::str::from_utf8(issue.atom_id.as_bytes()).is_ok());
        assert!(error.to_string().len() < 1_024);
        assert!(serde_json::to_vec(&error).unwrap().len() < 2_048);
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
    fn required_link_missing_counts_large_fixture_with_link_index() {
        let mut atoms = Vec::new();
        let mut links = Vec::new();
        for index in 0..256 {
            let requirement = format!("REQ-LARGE-{index:03}");
            let design = format!("DES-LARGE-{index:03}");
            atoms.push(atom(&requirement, "requirement", "sha256:req"));
            atoms.push(atom(&design, "design", "sha256:des"));
            links.push(
                TraceLinkInput {
                    from: requirement,
                    to: Some(design),
                    link_type: Some("refined_by".to_string()),
                }
                .into_trace_link()
                .unwrap(),
            );
        }
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms,
            duplicate_atom_ids: Vec::new(),
        };
        let trace_graph = TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links,
        };
        let policy = TracePolicy {
            required_links: BTreeMap::from([(
                "requirement".to_string(),
                vec![RequiredLinkRule {
                    link_type: "refined_by".to_string(),
                    target_kind: "design".to_string(),
                    min: 1,
                }],
            )]),
        };

        let missing = required_link_missing(&atom_index, &trace_graph, &policy);

        assert!(missing.is_empty());
    }

    #[test]
    fn trace_index_required_links_match_reference_on_graph_edge_cases() {
        let atom_index = atom_index(vec![
            atom("REQ-SOURCE", "requirement", "sha256:source"),
            atom("TARGET", "design", "sha256:target-design"),
            // The legacy evaluator's BTreeMap collection makes the last
            // duplicate kind authoritative before corruption is rejected by
            // its owning validation path. The index must not alter that fact.
            atom("TARGET", "test", "sha256:target-test"),
            atom("DES-A", "design", "sha256:a"),
            atom("TEST-B", "test", "sha256:b"),
        ]);
        let make_link = |from: &str, to: &str, link_type: &str| {
            TraceLinkInput {
                from: from.to_string(),
                to: Some(to.to_string()),
                link_type: Some(link_type.to_string()),
            }
            .into_trace_link()
            .unwrap()
        };
        let trace_graph = trace_graph(vec![
            make_link("REQ-SOURCE", "TARGET", "verified_by"),
            make_link("REQ-SOURCE", "TARGET", "verified_by"),
            make_link("REQ-SOURCE", "REQ-SOURCE", "self"),
            make_link("REQ-SOURCE", "DES-A", "refined_by"),
            make_link("DES-A", "TEST-B", "verified_by"),
            make_link("TEST-B", "REQ-SOURCE", "closes_cycle"),
            make_link("REQ-SOURCE", "TEST-B", "verified_by"),
        ]);
        let policy = TracePolicy {
            required_links: BTreeMap::from([(
                "requirement".to_string(),
                vec![
                    RequiredLinkRule {
                        link_type: "verified_by".to_string(),
                        target_kind: "test".to_string(),
                        min: 4,
                    },
                    RequiredLinkRule {
                        link_type: "self".to_string(),
                        target_kind: "requirement".to_string(),
                        min: 1,
                    },
                ],
            )]),
        };

        let reference = required_link_missing_reference(&atom_index, &trace_graph, &policy);
        let (indexed, counters) =
            required_link_missing_with_work(&atom_index, &trace_graph, &policy);

        assert_eq!(indexed, reference);
        assert_eq!(
            required_link_missing(&atom_index, &trace_graph, &policy),
            indexed
        );
        assert_eq!(
            indexed,
            vec![MissingRequiredLink {
                atom_id: "REQ-SOURCE".to_string(),
                required_type: "verified_by".to_string(),
                target_kind: "test".to_string(),
                min: 4,
                found: 3,
            }]
        );
        assert_eq!(counters.atoms_indexed, atom_index.atoms.len());
        assert_eq!(counters.links_indexed, trace_graph.links.len());
        assert_eq!(counters.rules_evaluated, 2);
    }

    #[test]
    fn trace_work_counters_have_linear_1x_3x_10x_slope() {
        for scale in [1_usize, 3, 10] {
            let mut atoms = Vec::with_capacity(scale * 2);
            let mut links = Vec::with_capacity(scale);
            for index in 0..scale {
                let requirement = format!("REQ-SCALE-{index:03}");
                let design = format!("DES-SCALE-{index:03}");
                atoms.push(atom(&requirement, "requirement", "sha256:req"));
                atoms.push(atom(&design, "design", "sha256:des"));
                links.push(
                    TraceLinkInput {
                        from: requirement,
                        to: Some(design),
                        link_type: Some("refined_by".to_string()),
                    }
                    .into_trace_link()
                    .unwrap(),
                );
            }
            let atom_index = atom_index(atoms);
            let trace_graph = trace_graph(links);
            let policy = TracePolicy {
                required_links: BTreeMap::from([(
                    "requirement".to_string(),
                    vec![RequiredLinkRule {
                        link_type: "refined_by".to_string(),
                        target_kind: "design".to_string(),
                        min: 2,
                    }],
                )]),
            };
            let mut evaluator = TraceEvaluationIndex::compile(&atom_index, &trace_graph);
            for atom in &atom_index.atoms {
                let _ = evaluator.adjacent_atoms(&atom.atom_id);
            }
            let missing = evaluator.missing_required_links(&policy);

            assert_eq!(missing.len(), scale);
            assert_eq!(
                evaluator.counters(),
                TraceWorkCounters {
                    atoms_indexed: 2 * scale,
                    rules_evaluated: scale,
                    links_indexed: scale,
                    index_entries: 5 * scale,
                    lookups: 6 * scale,
                    adjacency_visits: 2 * scale,
                }
            );
        }
    }

    #[test]
    fn verification_policy_parses_commit_booleans_and_commands() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "commit_policy:\n  require_no_required_fires: true\n  require_trace_completeness: false\n  reject_duplicate_atom_ids: true\nverification:\n  - id: unit\n    command: cargo test\n    cwd: crates/app\n",
        );

        let policy = parse_verification_policy(temp.path()).unwrap();

        assert!(policy.require_no_required_fires);
        assert!(policy.reject_duplicate_atom_ids);
        assert!(!policy.require_trace_completeness);
        assert_eq!(
            policy.verification,
            vec![VerificationCommand {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                cwd: "crates/app".to_string(),
                timeout_ms: DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS,
                max_output_bytes: DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES,
                inherit_env: false,
                env: BTreeMap::new(),
                allow_failure: false,
            }]
        );
    }

    #[test]
    fn verification_policy_rejects_safety_floor_relaxation() {
        for field in [
            "require_no_required_fires",
            "require_no_stale_resolutions",
            "require_verification_success",
            "reject_duplicate_atom_ids",
        ] {
            let temp = tempdir().unwrap();
            write_file(
                &temp.path().join("codefire.policy.yaml"),
                &format!("commit_policy:\n  {field}: false\n"),
            );

            let error = parse_verification_policy(temp.path()).unwrap_err();

            assert!(error
                .to_string()
                .contains(&format!("commit_policy.{field} must be true")));
        }
    }

    #[test]
    fn verification_policy_accepts_legacy_required_wrapper() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "verification:\n  required:\n    - id: unit\n      command: cargo test\n      cwd: crates/app\n",
        );

        let policy = parse_verification_policy(temp.path()).unwrap();

        assert_eq!(
            policy.verification,
            vec![VerificationCommand {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                cwd: "crates/app".to_string(),
                timeout_ms: DEFAULT_VERIFICATION_COMMAND_TIMEOUT_MS,
                max_output_bytes: DEFAULT_VERIFICATION_COMMAND_MAX_OUTPUT_BYTES,
                inherit_env: false,
                env: BTreeMap::new(),
                allow_failure: false,
            }]
        );
    }

    #[test]
    fn verification_policy_accepts_cli_owned_commit_signature_requirement() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "commit_policy:\n  require_commit_signature: true\n",
        );

        let policy = parse_verification_policy(temp.path()).unwrap();

        assert!(policy.require_commit_signature);
        assert_eq!(
            serde_json::to_value(&policy).unwrap()["require_commit_signature"],
            true
        );
        let default_policy_json = serde_json::to_value(default_verification_policy()).unwrap();
        assert!(default_policy_json
            .get("require_commit_signature")
            .is_none());
        let legacy_policy: VerificationPolicy =
            serde_json::from_value(default_policy_json).unwrap();
        assert!(!legacy_policy.require_commit_signature);
    }

    #[test]
    fn verification_policy_parses_command_execution_contract() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "verification:\n  - id: lint\n    command: cargo clippy\n    timeout_ms: 120000\n    max_output_bytes: 4096\n    inherit_env: false\n    env: RUST_BACKTRACE=1,CARGO_TERM_COLOR=never\n    allow_failure: true\n",
        );

        let policy = parse_verification_policy(temp.path()).unwrap();

        assert_eq!(policy.verification[0].timeout_ms, 120_000);
        assert_eq!(policy.verification[0].max_output_bytes, 4096);
        assert!(!policy.verification[0].inherit_env);
        assert_eq!(
            policy.verification[0].env.get("RUST_BACKTRACE"),
            Some(&"1".to_string())
        );
        assert_eq!(
            policy.verification[0].env.get("CARGO_TERM_COLOR"),
            Some(&"never".to_string())
        );
        assert!(policy.verification[0].allow_failure);
    }

    #[test]
    fn verification_policy_rejects_unsupported_yaml_fields() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.policy.yaml"),
            "commit_policy:\n  require_no_required_fires: true\n  typo_field: true\n",
        );

        let error = parse_verification_policy(temp.path()).unwrap_err();

        assert!(
            matches!(error, CoreError::Config(message) if message.contains("unsupported codefire.policy.yaml: commit_policy field typo_field at line 3"))
        );
    }

    #[test]
    fn stale_resolutions_accept_legacy_trace_link_basis() {
        let policy = default_verification_policy();
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
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
        let legacy_id = legacy_trace_link_id("REQ-AUTH-001", "DES-AUTH-001", "refined_by");
        let legacy_hash =
            trace_link_hash("REQ-AUTH-001", "DES-AUTH-001", "refined_by", &legacy_id).unwrap();
        let resolution = Resolution {
            type_tag: "resolution".to_string(),
            version: VERSION,
            resolution_uid: "res_legacy".to_string(),
            fire_uid: "fire_legacy".to_string(),
            resolution_type: "addressed".to_string(),
            rationale: "checked".to_string(),
            evidence: String::new(),
            evidence_refs: Vec::new(),
            basis: ResolutionBasis {
                source_atom: ResolutionAtomBasis {
                    atom_id: "REQ-AUTH-001".to_string(),
                    content_hash: Some("sha256:req1".to_string()),
                },
                target_atom: ResolutionAtomBasis {
                    atom_id: "DES-AUTH-001".to_string(),
                    content_hash: Some("sha256:des1".to_string()),
                },
                trace_links: vec![ResolutionTraceLinkBasis {
                    link_id: legacy_id,
                    link_hash: legacy_hash,
                }],
                policy_hash: policy_hash(&policy).unwrap(),
            },
            resolved_at: "2026-06-07T00:00:00Z".to_string(),
            status: "active".to_string(),
        };

        let stale = stale_resolutions(&atom_index, &trace_graph, &policy, &[resolution]).unwrap();

        assert!(stale.is_empty());
    }

    #[test]
    fn committable_decision_enforces_safety_floor_and_trace_warning() {
        let scan = ScanResult {
            type_tag: "scan".to_string(),
            version: VERSION,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: AtomIndex {
                type_tag: "atom_index".to_string(),
                version: VERSION,
                hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
                atoms: vec![
                    atom("REQ-AUTH-001", "requirement", "sha256:req"),
                    atom("REQ-AUTH-001", "requirement", "sha256:req-copy"),
                ],
                duplicate_atom_ids: vec!["REQ-AUTH-001".to_string()],
            },
            trace_graph: TraceGraph {
                type_tag: "trace_graph".to_string(),
                version: VERSION,
                links: Vec::new(),
            },
            changed_atoms: Vec::new(),
            non_atom_changed_files: Vec::new(),
            change_coverage: None,
            open_fires: vec![Fire {
                type_tag: "fire".to_string(),
                version: VERSION,
                fire_uid: "fire_required".to_string(),
                display_id: "FIRE-001".to_string(),
                status: "open".to_string(),
                severity: "required".to_string(),
                source: FireAtomRef {
                    atom_id: "REQ-AUTH-001".to_string(),
                    content_hash_at_fire: None,
                },
                target: FireAtomRef {
                    atom_id: "REQ-AUTH-001".to_string(),
                    content_hash_at_fire: None,
                },
                reason: "characterization".to_string(),
                trace_path: Vec::new(),
                created_by: "test".to_string(),
                created_at: "2026-06-04T00:00:00Z".to_string(),
                key: "required".to_string(),
                obsolete_at: None,
                resolution_uid: None,
            }],
            tool_migration: None,
        };
        let policy = VerificationPolicy {
            require_no_required_fires: false,
            require_no_stale_resolutions: false,
            require_trace_completeness: false,
            require_verification_success: false,
            reject_duplicate_atom_ids: false,
            ..default_verification_policy()
        };

        let verification = build_verification(
            &scan,
            VerificationInputs {
                unresolved_merge_conflicts: Vec::new(),
                missing_required_links: vec![MissingRequiredLink {
                    atom_id: "REQ-AUTH-001".to_string(),
                    required_type: "verified_by".to_string(),
                    target_kind: "test".to_string(),
                    min: 1,
                    found: 0,
                }],
                failed_checks: vec![FailedCheck {
                    id: "unit".to_string(),
                    command: "cargo test".to_string(),
                    output: "failed".to_string(),
                }],
                stale_resolutions: vec![StaleResolution {
                    resolution_uid: "resolution_1".to_string(),
                    reason: "source changed".to_string(),
                    ..Default::default()
                }],
                missing_evidence_refs: vec![MissingEvidenceRef {
                    resolution_uid: "resolution_1".to_string(),
                    evidence_id: "CF-EVIDENCE-missing".to_string(),
                }],
                ..VerificationInputs::default()
            },
            &policy,
            "2026-06-04T00:00:00Z",
        );

        assert_eq!(verification.result, "failed");
        let decision = committable_decision(&verification);
        assert_eq!(decision.outcome, CommittableOutcome::Failed);
        assert_eq!(
            decision.blocking_reasons,
            vec![
                CommittableReason::OpenRequiredFires,
                CommittableReason::StaleResolutions,
                CommittableReason::MissingEvidenceRefs,
                CommittableReason::DuplicateAtomIds,
                CommittableReason::FailedChecks,
            ]
        );
        assert_eq!(
            decision.warning_reasons,
            vec![CommittableReason::MissingRequiredLinks]
        );
        assert_eq!(
            decision.primary_reason,
            Some(CommittableReason::OpenRequiredFires)
        );
    }

    #[test]
    fn committable_decision_table_covers_every_reason_and_priority() {
        let none = VerificationSignals {
            unresolved_merge_conflicts: 0,
            open_required_fires: 0,
            missing_required_links: 0,
            failed_checks: 0,
            stale_resolutions: 0,
            missing_evidence_refs: 0,
            duplicate_atom_ids: 0,
        };
        assert!(committable_decision_from_signals(true, none).is_committable());

        let cases = [
            (
                CommittableReason::UnresolvedMergeConflicts,
                VerificationSignals {
                    unresolved_merge_conflicts: 1,
                    ..none
                },
            ),
            (
                CommittableReason::OpenRequiredFires,
                VerificationSignals {
                    open_required_fires: 1,
                    ..none
                },
            ),
            (
                CommittableReason::MissingRequiredLinks,
                VerificationSignals {
                    missing_required_links: 1,
                    ..none
                },
            ),
            (
                CommittableReason::StaleResolutions,
                VerificationSignals {
                    stale_resolutions: 1,
                    ..none
                },
            ),
            (
                CommittableReason::MissingEvidenceRefs,
                VerificationSignals {
                    missing_evidence_refs: 1,
                    ..none
                },
            ),
            (
                CommittableReason::DuplicateAtomIds,
                VerificationSignals {
                    duplicate_atom_ids: 1,
                    ..none
                },
            ),
            (
                CommittableReason::FailedChecks,
                VerificationSignals {
                    failed_checks: 1,
                    ..none
                },
            ),
        ];
        for (reason, signals) in cases {
            let decision = committable_decision_from_signals(true, signals);
            assert_eq!(decision.primary_reason, Some(reason));
            assert_eq!(decision.blocking_reasons, vec![reason]);
            assert!(decision.warning_reasons.is_empty());
        }

        let trace_warning = committable_decision_from_signals(
            false,
            VerificationSignals {
                missing_required_links: 1,
                ..none
            },
        );
        assert!(trace_warning.is_committable());
        assert!(trace_warning.blocking_reasons.is_empty());
        assert_eq!(
            trace_warning.warning_reasons,
            vec![CommittableReason::MissingRequiredLinks]
        );

        let every_reason = committable_decision_from_signals(
            true,
            VerificationSignals {
                unresolved_merge_conflicts: 1,
                open_required_fires: 1,
                missing_required_links: 1,
                failed_checks: 1,
                stale_resolutions: 1,
                missing_evidence_refs: 1,
                duplicate_atom_ids: 1,
            },
        );
        assert_eq!(
            every_reason.blocking_reasons,
            vec![
                CommittableReason::UnresolvedMergeConflicts,
                CommittableReason::OpenRequiredFires,
                CommittableReason::MissingRequiredLinks,
                CommittableReason::StaleResolutions,
                CommittableReason::MissingEvidenceRefs,
                CommittableReason::DuplicateAtomIds,
                CommittableReason::FailedChecks,
            ]
        );
        assert_eq!(
            every_reason.primary_reason,
            Some(CommittableReason::UnresolvedMergeConflicts)
        );
    }

    #[test]
    fn verification_merge_conflicts_are_typed_and_legacy_omission_is_empty() {
        let legacy: Verification = serde_json::from_value(serde_json::json!({
            "type": "verification",
            "version": 1,
            "result": "passed",
            "open_required_fires": 0,
            "failed_checks": [],
            "missing_required_links": [],
            "stale_resolutions": [],
            "missing_evidence_refs": [],
            "duplicate_atom_ids": [],
            "verified_at": "2026-07-12T00:00:00Z"
        }))
        .unwrap();
        assert!(legacy.unresolved_merge_conflicts.is_empty());
        assert!(committable_decision(&legacy).is_committable());

        let mut conflicted = legacy;
        conflicted.unresolved_merge_conflicts = vec![UnresolvedMergeConflict {
            kind: MergeConflictKind::Text,
            subject: "README.md".to_string(),
        }];
        let decision = committable_decision(&conflicted);
        assert_eq!(
            decision.primary_reason,
            Some(CommittableReason::UnresolvedMergeConflicts)
        );
        assert!(!decision.is_committable());
        assert_eq!(
            serde_json::to_value(&conflicted).unwrap()["unresolved_merge_conflicts"][0]["kind"],
            "text"
        );
    }

    #[test]
    fn sealed_snapshot_semantics_rejects_open_fire_recorded_as_passed() {
        let (atom_index, trace_graph, mut fires, mut resolutions, _, mut policy) =
            valid_semantic_snapshot();
        policy.require_no_required_fires = false;
        fires[0].status = "open".to_string();
        fires[0].resolution_uid = None;
        resolutions.clear();
        let scan = ScanResult {
            type_tag: "scan".to_string(),
            version: VERSION,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: atom_index.clone(),
            trace_graph: trace_graph.clone(),
            changed_atoms: Vec::new(),
            non_atom_changed_files: Vec::new(),
            change_coverage: None,
            open_fires: fires.clone(),
            tool_migration: None,
        };
        let mut verification = build_verification(
            &scan,
            VerificationInputs::default(),
            &policy,
            "2026-07-12T00:02:00Z",
        );
        assert_eq!(verification.open_required_fires, 1);
        assert_eq!(verification.result, "failed");
        verification.open_required_fires = 0;
        verification.result = "passed".to_string();

        let error = validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &fires,
            &resolutions,
            &verification,
            &policy,
        )
        .unwrap_err();

        let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
            panic!("expected sealed snapshot semantic integrity error");
        };
        assert!(error.issues.iter().any(|issue| matches!(
            issue,
            SealedSnapshotSemanticIssue::OpenRequiredFireCountMismatch {
                computed: 1,
                recorded: 0
            }
        )));
        assert!(error.issues.iter().any(|issue| matches!(
            issue,
            SealedSnapshotSemanticIssue::VerificationResultMismatch { computed, recorded }
                if computed == "failed" && recorded == "passed"
        )));
    }

    #[test]
    fn sealed_snapshot_semantics_reads_clean_historical_false_policy_without_relaxing_facts() {
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: Vec::new(),
            duplicate_atom_ids: Vec::new(),
        };
        let trace_graph = TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links: Vec::new(),
        };
        let scan = ScanResult {
            type_tag: "scan".to_string(),
            version: VERSION,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: atom_index.clone(),
            trace_graph: trace_graph.clone(),
            changed_atoms: Vec::new(),
            non_atom_changed_files: Vec::new(),
            change_coverage: None,
            open_fires: Vec::new(),
            tool_migration: None,
        };
        let policy = VerificationPolicy {
            require_no_required_fires: false,
            require_no_stale_resolutions: false,
            require_verification_success: false,
            reject_duplicate_atom_ids: false,
            required_links: BTreeMap::new(),
            ..default_verification_policy()
        };
        let verification = build_verification(
            &scan,
            VerificationInputs::default(),
            &policy,
            "2026-07-12T00:00:00Z",
        );

        assert!(committable_decision(&verification).is_committable());
        validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &[],
            &[],
            &verification,
            &policy,
        )
        .unwrap();
    }

    #[test]
    fn sealed_snapshot_semantics_recomputes_stale_missing_and_duplicate_facts() {
        let (mut atom_index, trace_graph, fires, resolutions, verification, mut policy) =
            valid_semantic_snapshot();
        atom_index
            .atoms
            .push(atom("REQ-AUTH-001", "requirement", "sha256:req"));
        policy.required_links = BTreeMap::from([(
            "requirement".to_string(),
            vec![RequiredLinkRule {
                link_type: "verified_by".to_string(),
                target_kind: "test".to_string(),
                min: 1,
            }],
        )]);

        let error = validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &fires,
            &resolutions,
            &verification,
            &policy,
        )
        .unwrap_err();

        let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
            panic!("expected sealed snapshot semantic integrity error");
        };
        for expected_kind in [
            "duplicate_atom_ids_mismatch",
            "missing_required_links_mismatch",
            "stale_resolutions_mismatch",
            "verification_result_mismatch",
        ] {
            assert!(
                error
                    .issues
                    .iter()
                    .any(|issue| issue.kind() == expected_kind),
                "missing semantic issue {expected_kind}"
            );
        }
    }

    #[test]
    fn sealed_snapshot_semantics_rejects_ambiguous_active_resolution_relation() {
        let (atom_index, trace_graph, fires, mut resolutions, verification, policy) =
            valid_semantic_snapshot();
        let duplicate = resolutions[0].clone();
        resolutions.push(duplicate);

        let error = validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &fires,
            &resolutions,
            &verification,
            &policy,
        )
        .unwrap_err();

        let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
            panic!("expected sealed snapshot semantic integrity error");
        };
        assert!(error.issues.iter().any(|issue| matches!(
            issue,
            SealedSnapshotSemanticIssue::DuplicateActiveResolutionUid {
                resolution_uid,
                occurrences: 2,
            } if resolution_uid == "res_001"
        )));
        assert!(error.issues.iter().any(|issue| matches!(
            issue,
            SealedSnapshotSemanticIssue::ActiveResolutionPerFireCardinality {
                fire_uid,
                resolution_uids,
            } if fire_uid == "fire_001" && resolution_uids == &["res_001", "res_001"]
        )));
        assert!(error.issues.iter().any(|issue| matches!(
            issue,
            SealedSnapshotSemanticIssue::FireResolutionIdentityMismatch {
                fire_uid,
                declared_resolution_uid: Some(declared),
                active_resolution_uids,
            } if fire_uid == "fire_001"
                && declared == "res_001"
                && active_resolution_uids == &["res_001", "res_001"]
        )));
    }

    #[test]
    fn sealed_snapshot_semantics_accepts_a_consistent_snapshot() {
        let (atom_index, trace_graph, fires, resolutions, verification, policy) =
            valid_semantic_snapshot();

        validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &fires,
            &resolutions,
            &verification,
            &policy,
        )
        .unwrap();
    }

    #[test]
    fn sealed_snapshot_semantics_rejects_trace_requirement_metadata_drift() {
        let (atom_index, trace_graph, fires, resolutions, mut verification, policy) =
            valid_semantic_snapshot();
        verification.trace_completeness_required = false;

        let error = validate_sealed_snapshot_semantics(
            &atom_index,
            &trace_graph,
            &fires,
            &resolutions,
            &verification,
            &policy,
        )
        .unwrap_err();

        let CoreError::SealedSnapshotSemanticIntegrity(error) = error else {
            panic!("expected sealed snapshot semantic integrity error");
        };
        assert_eq!(
            error.issues,
            vec![
                SealedSnapshotSemanticIssue::TraceCompletenessRequiredMismatch {
                    computed: true,
                    recorded: false,
                }
            ]
        );
    }

    #[test]
    fn stale_resolutions_detect_atom_policy_and_trace_changes() {
        let policy = default_verification_policy();
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
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
                fire_uid: "fire_001".to_string(),
                fire_display_id: String::new(),
                source_atom_id: "REQ-AUTH-001".to_string(),
                target_atom_id: "DES-AUTH-001".to_string(),
            }]
        );

        assert_eq!(
            fire_display_id_from_uid("fire_sha256_571b5e2c0245bdd96db0c7407e4328bf").as_deref(),
            Some("FIRE-571B5E2C0245")
        );
        assert_eq!(fire_display_id_from_uid("fire_001"), None);
        let encoded = serde_json::to_value(StaleResolution {
            resolution_uid: "res_001".to_string(),
            reason: "policy changed".to_string(),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            encoded,
            serde_json::json!({"resolution_uid": "res_001", "reason": "policy changed"})
        );
    }

    #[test]
    fn scan_result_detects_changed_atoms_and_creates_trace_fires() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("DES-AUTH-001", "design", "sha256:design2"),
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
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
    fn indexed_scan_matches_reference_for_duplicate_self_cycle_and_diamond() {
        let current = atom_index(vec![
            atom("A", "requirement", "sha256:a2"),
            atom("B", "design", "sha256:b2"),
            atom("C", "test", "sha256:c2"),
            atom("D", "code", "sha256:d2"),
        ]);
        let base = atom_index(vec![
            atom("A", "requirement", "sha256:a1"),
            atom("B", "design", "sha256:b1"),
            atom("C", "test", "sha256:c1"),
            atom("D", "code", "sha256:d1"),
        ]);
        let make_link = |from: &str, to: &str| {
            TraceLinkInput {
                from: from.to_string(),
                to: Some(to.to_string()),
                link_type: Some("trace".to_string()),
            }
            .into_trace_link()
            .unwrap()
        };
        let graph = trace_graph(vec![
            make_link("A", "A"),
            make_link("A", "B"),
            make_link("A", "B"),
            make_link("B", "C"),
            make_link("C", "A"),
            make_link("A", "D"),
            make_link("B", "D"),
            make_link("C", "D"),
        ]);
        let options = ScanBuildOptions {
            reason: "atom_changed",
            now: "2026-07-19T00:00:00Z",
            source_unchanged: false,
        };
        let reference_graph = graph.clone();
        let reference = build_scan_result_against_base_with_adjacency(
            current.clone(),
            &base,
            graph.clone(),
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            options,
            |source| adjacent_atoms_reference(source, &reference_graph),
        )
        .unwrap();
        let compatible = build_scan_result_against_base(
            current.clone(),
            &base,
            graph.clone(),
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            options,
        )
        .unwrap();
        let (scan, fires, counters) = build_scan_result_against_base_with_work(
            current,
            &base,
            graph,
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            options,
        )
        .unwrap();

        assert_eq!((scan, fires), reference);
        assert_eq!(compatible, reference);
        assert_eq!(
            counters,
            TraceWorkCounters {
                atoms_indexed: 4,
                rules_evaluated: 0,
                links_indexed: 8,
                index_entries: 26,
                lookups: 12,
                adjacency_visits: 15,
            }
        );
    }

    #[test]
    fn indexed_scan_keeps_direct_only_bidirectional_fire_semantics() {
        let current = atom_index(vec![
            atom("A", "requirement", "sha256:a2"),
            atom("B", "design", "sha256:b"),
            atom("C", "design", "sha256:c"),
            atom("D", "code", "sha256:d"),
        ]);
        let base = atom_index(vec![
            atom("A", "requirement", "sha256:a1"),
            atom("B", "design", "sha256:b"),
            atom("C", "design", "sha256:c"),
            atom("D", "code", "sha256:d"),
        ]);
        let make_link = |from: &str, to: &str| {
            TraceLinkInput {
                from: from.to_string(),
                to: Some(to.to_string()),
                link_type: Some("trace".to_string()),
            }
            .into_trace_link()
            .unwrap()
        };
        let graph = trace_graph(vec![
            make_link("A", "B"),
            make_link("A", "C"),
            make_link("B", "D"),
            make_link("C", "D"),
        ]);

        let (_, fires) = build_scan_result(
            current,
            base,
            graph,
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            "atom_changed",
            "2026-07-19T00:00:00Z",
        )
        .unwrap();
        let targets = fires
            .iter()
            .map(|fire| fire.target.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(targets, vec!["B", "C"]);
    }

    #[test]
    fn scan_result_uses_stable_sha256_fire_identity() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
                atom("DES-AUTH-001", "design", "sha256:design2"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
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
    fn scan_result_classifies_source_unchanged_hash_schema_drift_as_tool_migration() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req2"),
                atom("DES-AUTH-001", "design", "sha256:design2"),
            ],
            duplicate_atom_ids: Vec::new(),
        };
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: 1,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req1"),
                atom("DES-AUTH-001", "design", "sha256:design1"),
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

        let (scan, fires) = build_scan_result_with_options(
            current.clone(),
            base.clone(),
            trace_graph.clone(),
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            ScanBuildOptions {
                reason: "atom_changed",
                now: "2026-06-04T00:00:00Z",
                source_unchanged: true,
            },
        )
        .unwrap();

        assert!(scan.changed_atoms.is_empty());
        assert!(scan.open_fires.is_empty());
        assert!(fires.is_empty());
        let migration = scan.tool_migration.unwrap();
        assert_eq!(migration.kind, "atom_hash_schema_changed");
        assert_eq!(migration.from_hash_schema_version, 1);
        assert_eq!(migration.to_hash_schema_version, ATOM_HASH_SCHEMA_VERSION);
        assert_eq!(
            migration.affected_atoms,
            vec!["DES-AUTH-001", "REQ-AUTH-001"]
        );

        let (scan, fires) = build_scan_result_with_options(
            current,
            base,
            trace_graph,
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            ScanBuildOptions {
                reason: "atom_changed",
                now: "2026-06-04T00:00:00Z",
                source_unchanged: false,
            },
        )
        .unwrap();
        assert_eq!(scan.changed_atoms, vec!["DES-AUTH-001", "REQ-AUTH-001"]);
        assert_eq!(scan.open_fires.len(), 2);
        assert_eq!(fires.len(), 2);
        assert!(scan.tool_migration.is_none());
    }

    #[test]
    fn string_aware_python_symbols_use_schema_four_tool_migration_without_fires() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("src/evoagent/models.py"),
            "class ExperimentPlan:\n    def uid(self):\n        return \"plan\"\n",
        );
        let current = build_atom_index(temp.path()).unwrap();
        let base = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: 2,
            atoms: Vec::new(),
            duplicate_atom_ids: Vec::new(),
        };

        let (scan, fires) = build_scan_result_with_options(
            current,
            base,
            TraceGraph {
                type_tag: "trace_graph".to_string(),
                version: VERSION,
                links: Vec::new(),
            },
            Vec::new(),
            "CF-COMMIT-base".to_string(),
            ScanBuildOptions {
                reason: "atom_changed",
                now: "2026-07-12T00:00:00Z",
                source_unchanged: true,
            },
        )
        .unwrap();

        assert_eq!(ATOM_HASH_SCHEMA_VERSION, 4);
        assert!(scan.changed_atoms.is_empty());
        assert!(scan.open_fires.is_empty());
        assert!(fires.is_empty());
        let migration = scan.tool_migration.unwrap();
        assert_eq!(migration.from_hash_schema_version, 2);
        assert_eq!(migration.to_hash_schema_version, 4);
        assert_eq!(
            migration.affected_atoms,
            vec![
                "CODE:src/evoagent/models.py::class:ExperimentPlan",
                "CODE:src/evoagent/models.py::method:ExperimentPlan.uid",
            ]
        );
    }

    #[test]
    fn scan_result_obsoletes_scan_fires_when_source_is_no_longer_changed() {
        let current = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
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

    fn valid_semantic_snapshot() -> (
        AtomIndex,
        TraceGraph,
        Vec<Fire>,
        Vec<Resolution>,
        Verification,
        VerificationPolicy,
    ) {
        let policy = VerificationPolicy {
            required_links: BTreeMap::new(),
            ..default_verification_policy()
        };
        let atom_index = AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms: vec![
                atom("REQ-AUTH-001", "requirement", "sha256:req"),
                atom("DES-AUTH-001", "design", "sha256:des"),
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
                content_hash_at_fire: Some("sha256:req".to_string()),
            },
            target: FireAtomRef {
                atom_id: "DES-AUTH-001".to_string(),
                content_hash_at_fire: Some("sha256:des".to_string()),
            },
            reason: "atom_changed".to_string(),
            trace_path: vec!["REQ-AUTH-001".to_string(), "DES-AUTH-001".to_string()],
            created_by: "scan".to_string(),
            created_at: "2026-07-12T00:00:00Z".to_string(),
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
                rationale: "reviewed".to_string(),
                evidence: "test evidence".to_string(),
                evidence_refs: Vec::new(),
                resolved_at: "2026-07-12T00:01:00Z".to_string(),
            },
        )
        .unwrap();
        let scan = ScanResult {
            type_tag: "scan".to_string(),
            version: VERSION,
            base_commit: "CF-COMMIT-base".to_string(),
            atom_index: atom_index.clone(),
            trace_graph: trace_graph.clone(),
            changed_atoms: Vec::new(),
            non_atom_changed_files: Vec::new(),
            change_coverage: None,
            open_fires: Vec::new(),
            tool_migration: None,
        };
        let verification = build_verification(
            &scan,
            VerificationInputs::default(),
            &policy,
            "2026-07-12T00:02:00Z",
        );
        (
            atom_index,
            trace_graph,
            vec![fire],
            vec![resolution],
            verification,
            policy,
        )
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

    fn atom_index(atoms: Vec<Atom>) -> AtomIndex {
        AtomIndex {
            type_tag: "atom_index".to_string(),
            version: VERSION,
            hash_schema_version: ATOM_HASH_SCHEMA_VERSION,
            atoms,
            duplicate_atom_ids: Vec::new(),
        }
    }

    fn trace_graph(links: Vec<TraceLink>) -> TraceGraph {
        TraceGraph {
            type_tag: "trace_graph".to_string(),
            version: VERSION,
            links,
        }
    }
}
