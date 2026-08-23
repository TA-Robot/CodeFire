use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DECISION_WORK_LEDGER_ID: AtomicU64 = AtomicU64::new(1);

pub const DECISION_WORK_PROFILE_V1: &str = "codefire.decision-work.v1";
pub const RESOURCE_BUDGET_EXIT_CODE: u8 = 60;
pub const HTTP_REQUEST_DEADLINE_MS: u64 = 30_000;
pub const LOGICAL_FRAME_BASE_BYTES: u64 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkDimension {
    GraphNodes,
    GraphEdges,
    LogicalDepth,
    AggregateBytes,
    RequestHeaderBytes,
    ResponseHeaderBytes,
    RequestBodyBytes,
    ResponseBytes,
    RetainedBytes,
    TemporaryBytes,
    ObjectRecords,
    SnapshotRevalidationRecords,
    SnapshotRevalidationBytes,
    MaximumFrontier,
    CounterValues,
}

impl DecisionWorkDimension {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GraphNodes => "graph_nodes",
            Self::GraphEdges => "graph_edges",
            Self::LogicalDepth => "logical_depth",
            Self::AggregateBytes => "aggregate_bytes",
            Self::RequestHeaderBytes => "request_header_bytes",
            Self::ResponseHeaderBytes => "response_header_bytes",
            Self::RequestBodyBytes => "request_body_bytes",
            Self::ResponseBytes => "response_bytes",
            Self::RetainedBytes => "retained_bytes",
            Self::TemporaryBytes => "temporary_bytes",
            Self::ObjectRecords => "object_records",
            Self::SnapshotRevalidationRecords => "snapshot_revalidation_records",
            Self::SnapshotRevalidationBytes => "snapshot_revalidation_bytes",
            Self::MaximumFrontier => "maximum_frontier",
            Self::CounterValues => "counter_values",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkCounter {
    FilesRead,
    FileBytesRead,
    BytesHashed,
    AtomsIndexed,
    RulesEvaluated,
    TraceLinksIndexed,
    UniqueNodesDiscovered,
    RecordReads,
    SemanticValidations,
    EdgeOccurrences,
    FrontierInsertionAttempts,
    Revisits,
    MemoHits,
    MemoMisses,
    ObjectRecords,
    ResponseBytesEncoded,
    SnapshotRevalidationRecords,
    SnapshotRevalidationBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkPhase {
    RequestHeaders,
    RequestBody,
    Validation,
    GraphDiscovery,
    RecordRead,
    SemanticValidation,
    Query,
    Encoding,
    Staging,
    SnapshotRevalidation,
    PublicationPreflight,
    ResponseHeaders,
    ResponseBody,
}

impl DecisionWorkPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestHeaders => "request_headers",
            Self::RequestBody => "request_body",
            Self::Validation => "validation",
            Self::GraphDiscovery => "graph_discovery",
            Self::RecordRead => "record_read",
            Self::SemanticValidation => "semantic_validation",
            Self::Query => "query",
            Self::Encoding => "encoding",
            Self::Staging => "staging",
            Self::SnapshotRevalidation => "snapshot_revalidation",
            Self::PublicationPreflight => "publication_preflight",
            Self::ResponseHeaders => "response_headers",
            Self::ResponseBody => "response_body",
        }
    }
}

impl fmt::Display for DecisionWorkPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkSurface {
    Atomic,
    SnapshotPaged,
}

impl DecisionWorkSurface {
    const fn continuation_supported(self) -> bool {
        matches!(self, Self::SnapshotPaged)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkTerminalReason {
    Complete,
    EarlyMatch,
    BudgetExceeded,
    CorruptSubject,
    SubjectChanged,
    DeadlineExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkCommandKind {
    LocalGraph,
    ObjectTransfer,
    HttpRequest,
}

impl fmt::Display for DecisionWorkDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionWorkMeasurement {
    Cumulative,
    Maximum,
    Concurrent,
}

impl fmt::Display for DecisionWorkMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Cumulative => "cumulative",
            Self::Maximum => "maximum",
            Self::Concurrent => "concurrent",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionWorkLimit {
    pub measurement: DecisionWorkMeasurement,
    pub limit: u64,
}

impl DecisionWorkLimit {
    pub const fn cumulative(limit: u64) -> Self {
        Self {
            measurement: DecisionWorkMeasurement::Cumulative,
            limit,
        }
    }

    pub const fn maximum(limit: u64) -> Self {
        Self {
            measurement: DecisionWorkMeasurement::Maximum,
            limit,
        }
    }

    pub const fn concurrent(limit: u64) -> Self {
        Self {
            measurement: DecisionWorkMeasurement::Concurrent,
            limit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionWorkProfile {
    profile_id: String,
    limits: BTreeMap<DecisionWorkDimension, DecisionWorkLimit>,
    applicable_commands: Vec<DecisionWorkCommandKind>,
    applicable_phases: Vec<DecisionWorkPhase>,
    /// Availability control advertised with the HTTP profile. It is not a
    /// deterministic ledger counter and is checked only before publication.
    request_deadline_ms: Option<u64>,
    /// Enforced by the process-global HTTP RAII limiter, not a request ledger.
    max_active_connections: Option<u64>,
    /// V1 rejects excess connections immediately and never queues them.
    max_queued_connections: Option<u64>,
}

impl DecisionWorkProfile {
    pub fn local_v1() -> Self {
        Self {
            profile_id: DECISION_WORK_PROFILE_V1.to_string(),
            limits: BTreeMap::from([
                (
                    DecisionWorkDimension::GraphNodes,
                    DecisionWorkLimit::cumulative(100_000),
                ),
                (
                    DecisionWorkDimension::GraphEdges,
                    DecisionWorkLimit::cumulative(400_000),
                ),
                (
                    DecisionWorkDimension::LogicalDepth,
                    DecisionWorkLimit::maximum(100_000),
                ),
                (
                    DecisionWorkDimension::AggregateBytes,
                    DecisionWorkLimit::cumulative(1024 * 1024 * 1024),
                ),
                (
                    DecisionWorkDimension::RetainedBytes,
                    DecisionWorkLimit::concurrent(1024 * 1024 * 1024),
                ),
                (
                    DecisionWorkDimension::TemporaryBytes,
                    DecisionWorkLimit::concurrent(1024 * 1024 * 1024),
                ),
                (
                    DecisionWorkDimension::ObjectRecords,
                    DecisionWorkLimit::cumulative(100_000),
                ),
                (
                    DecisionWorkDimension::SnapshotRevalidationRecords,
                    DecisionWorkLimit::cumulative(100_000),
                ),
                (
                    DecisionWorkDimension::SnapshotRevalidationBytes,
                    DecisionWorkLimit::cumulative(1024 * 1024 * 1024),
                ),
                (
                    DecisionWorkDimension::MaximumFrontier,
                    DecisionWorkLimit::maximum(100_000),
                ),
                (
                    DecisionWorkDimension::CounterValues,
                    DecisionWorkLimit::maximum(u64::MAX - 1),
                ),
            ]),
            applicable_commands: vec![
                DecisionWorkCommandKind::LocalGraph,
                DecisionWorkCommandKind::ObjectTransfer,
            ],
            applicable_phases: vec![
                DecisionWorkPhase::Validation,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkPhase::RecordRead,
                DecisionWorkPhase::SemanticValidation,
                DecisionWorkPhase::Query,
                DecisionWorkPhase::Staging,
                DecisionWorkPhase::SnapshotRevalidation,
                DecisionWorkPhase::PublicationPreflight,
            ],
            request_deadline_ms: None,
            max_active_connections: None,
            max_queued_connections: None,
        }
    }

    pub fn http_v1() -> Self {
        let mut profile = Self::local_v1();
        profile.limits.extend([
            (
                DecisionWorkDimension::RequestHeaderBytes,
                DecisionWorkLimit::cumulative(64 * 1024),
            ),
            (
                DecisionWorkDimension::ResponseHeaderBytes,
                DecisionWorkLimit::cumulative(64 * 1024),
            ),
            (
                DecisionWorkDimension::RequestBodyBytes,
                DecisionWorkLimit::cumulative(16 * 1024 * 1024),
            ),
            (
                DecisionWorkDimension::ResponseBytes,
                DecisionWorkLimit::cumulative(16 * 1024 * 1024),
            ),
            (
                DecisionWorkDimension::RetainedBytes,
                DecisionWorkLimit::concurrent(32 * 1024 * 1024),
            ),
            (
                DecisionWorkDimension::TemporaryBytes,
                DecisionWorkLimit::concurrent(16 * 1024 * 1024),
            ),
        ]);
        profile.request_deadline_ms = Some(HTTP_REQUEST_DEADLINE_MS);
        profile.max_active_connections = Some(32);
        profile.max_queued_connections = Some(0);
        profile
            .applicable_commands
            .push(DecisionWorkCommandKind::HttpRequest);
        profile.applicable_phases.extend([
            DecisionWorkPhase::RequestHeaders,
            DecisionWorkPhase::RequestBody,
            DecisionWorkPhase::Encoding,
            DecisionWorkPhase::ResponseHeaders,
            DecisionWorkPhase::ResponseBody,
        ]);
        profile
    }

    pub fn limit(&self, dimension: DecisionWorkDimension) -> Option<DecisionWorkLimit> {
        self.limits.get(&dimension).copied()
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub fn limits(&self) -> &BTreeMap<DecisionWorkDimension, DecisionWorkLimit> {
        &self.limits
    }

    pub fn applicable_commands(&self) -> &[DecisionWorkCommandKind] {
        &self.applicable_commands
    }

    pub fn applicable_phases(&self) -> &[DecisionWorkPhase] {
        &self.applicable_phases
    }

    pub fn request_deadline_ms(&self) -> Option<u64> {
        self.request_deadline_ms
    }

    pub fn max_active_connections(&self) -> Option<u64> {
        self.max_active_connections
    }

    pub fn max_queued_connections(&self) -> Option<u64> {
        self.max_queued_connections
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionWorkBudgetExceeded {
    pub kind: String,
    pub exit_code: u8,
    pub profile: String,
    pub dimension: DecisionWorkDimension,
    pub observed: u64,
    pub limit: u64,
    pub phase: DecisionWorkPhase,
    pub command: String,
    pub continuation_supported: bool,
}

impl fmt::Display for DecisionWorkBudgetExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "resource budget exceeded for {} during {}: {} observed, limit {}",
            self.command, self.phase, self.observed, self.limit
        )
    }
}

impl std::error::Error for DecisionWorkBudgetExceeded {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionWorkAccountingError {
    BudgetExceeded(DecisionWorkBudgetExceeded),
    MeasurementMismatch {
        dimension: DecisionWorkDimension,
        expected: DecisionWorkMeasurement,
        actual: DecisionWorkMeasurement,
    },
    MissingLimit(DecisionWorkDimension),
    InvalidLimit(DecisionWorkDimension),
    CommandNotApplicable(DecisionWorkCommandKind),
    PhaseNotApplicable(DecisionWorkPhase),
    TerminalAlreadySet {
        current: DecisionWorkTerminalReason,
        attempted: DecisionWorkTerminalReason,
    },
    LedgerTerminated(DecisionWorkTerminalReason),
    ReleaseUnderflow {
        dimension: DecisionWorkDimension,
        current: u64,
        amount: u64,
    },
    PrepublicationSealed,
    InvalidPermit,
    PermitExceeded,
    OpenPermits,
    SizeOverflow,
}

impl DecisionWorkAccountingError {
    pub fn budget_exceeded(&self) -> Option<&DecisionWorkBudgetExceeded> {
        match self {
            Self::BudgetExceeded(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for DecisionWorkAccountingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BudgetExceeded(error) => write!(f, "{error}"),
            Self::MeasurementMismatch {
                dimension,
                expected,
                actual,
            } => write!(
                f,
                "decision-work measurement mismatch for {dimension}: expected {expected}, got {actual}"
            ),
            Self::MissingLimit(dimension) => {
                write!(f, "decision-work profile has no {dimension} limit")
            }
            Self::InvalidLimit(dimension) => write!(
                f,
                "decision-work profile limit for {dimension} must be less than u64::MAX"
            ),
            Self::CommandNotApplicable(command) => write!(
                f,
                "decision-work command kind {command:?} is not applicable to this profile"
            ),
            Self::PhaseNotApplicable(phase) => write!(
                f,
                "decision-work phase {phase} is not applicable to this profile"
            ),
            Self::TerminalAlreadySet { current, attempted } => write!(
                f,
                "decision-work terminal reason is already {current:?}, cannot replace it with {attempted:?}"
            ),
            Self::LedgerTerminated(reason) => {
                write!(f, "decision-work ledger is already terminal: {reason:?}")
            }
            Self::ReleaseUnderflow {
                dimension,
                current,
                amount,
            } => write!(
                f,
                "decision-work release underflow for {dimension}: current {current}, release {amount}"
            ),
            Self::PrepublicationSealed => {
                f.write_str("decision-work ledger is sealed for publication")
            }
            Self::InvalidPermit => {
                f.write_str("decision-work permit does not belong to this ledger")
            }
            Self::PermitExceeded => {
                f.write_str("decision-work permit consumption exceeds its reserved upper bound")
            }
            Self::OpenPermits => {
                f.write_str("decision-work ledger still has open publication permits")
            }
            Self::SizeOverflow => f.write_str("decision-work logical size does not fit in u64"),
        }
    }
}

impl std::error::Error for DecisionWorkAccountingError {}

impl From<DecisionWorkBudgetExceeded> for DecisionWorkAccountingError {
    fn from(error: DecisionWorkBudgetExceeded) -> Self {
        Self::BudgetExceeded(error)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionWorkSnapshot {
    pub cumulative: BTreeMap<DecisionWorkDimension, u64>,
    pub maximum: BTreeMap<DecisionWorkDimension, u64>,
    pub current: BTreeMap<DecisionWorkDimension, u64>,
    pub peak: BTreeMap<DecisionWorkDimension, u64>,
    pub counters: BTreeMap<DecisionWorkCounter, u64>,
    pub phases: BTreeMap<DecisionWorkPhase, BTreeMap<DecisionWorkDimension, u64>>,
    pub phase_counters: BTreeMap<DecisionWorkPhase, BTreeMap<DecisionWorkCounter, u64>>,
    pub terminal_reason: Option<DecisionWorkTerminalReason>,
}

/// All accounting attached to one logical effect. `apply` preflights the
/// complete reservation against a bounded clone of the stable counter maps
/// and commits it as one unit only when every charge succeeds.
#[derive(Debug, Clone)]
pub struct DecisionWorkReservation {
    phase: DecisionWorkPhase,
    surface: DecisionWorkSurface,
    cumulative: BTreeMap<DecisionWorkDimension, u64>,
    maximum: BTreeMap<DecisionWorkDimension, u64>,
    concurrent: BTreeMap<DecisionWorkDimension, u64>,
    counters: BTreeMap<DecisionWorkCounter, u64>,
    overflow_dimension: Option<DecisionWorkDimension>,
    overflow_counter: Option<DecisionWorkCounter>,
}

#[derive(Debug)]
pub struct DecisionWorkPermit {
    ledger_id: u64,
    permit_id: u64,
}

/// Opaque identity for binding a ledger-independent prepared owner back to
/// the exact command ledger that acquired its memory and work reservations.
///
/// The numeric identity is deliberately private: consumers may retain and
/// compare the token, but cannot manufacture one for a different ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DecisionWorkLedgerIdentity {
    ledger_id: u64,
}

impl DecisionWorkReservation {
    pub fn new(phase: DecisionWorkPhase, surface: DecisionWorkSurface) -> Self {
        Self {
            phase,
            surface,
            cumulative: BTreeMap::new(),
            maximum: BTreeMap::new(),
            concurrent: BTreeMap::new(),
            counters: BTreeMap::new(),
            overflow_dimension: None,
            overflow_counter: None,
        }
    }

    pub fn charge(mut self, dimension: DecisionWorkDimension, amount: u64) -> Self {
        accumulate_reservation_value(
            &mut self.cumulative,
            dimension,
            amount,
            &mut self.overflow_dimension,
        );
        self
    }

    pub fn observe_maximum(mut self, dimension: DecisionWorkDimension, value: u64) -> Self {
        self.maximum
            .entry(dimension)
            .and_modify(|current| *current = (*current).max(value))
            .or_insert(value);
        self
    }

    pub fn acquire(mut self, dimension: DecisionWorkDimension, amount: u64) -> Self {
        accumulate_reservation_value(
            &mut self.concurrent,
            dimension,
            amount,
            &mut self.overflow_dimension,
        );
        self
    }

    pub fn count(mut self, counter: DecisionWorkCounter, amount: u64) -> Self {
        accumulate_reservation_value(
            &mut self.counters,
            counter,
            amount,
            &mut self.overflow_counter,
        );
        self
    }

    pub fn concurrent_amount(&self, dimension: DecisionWorkDimension) -> u64 {
        self.concurrent.get(&dimension).copied().unwrap_or_default()
    }

    fn remaining_after(&self, actual: &Self) -> Option<Self> {
        if self.phase != actual.phase
            || self.surface != actual.surface
            || actual.overflow_dimension.is_some()
            || actual.overflow_counter.is_some()
        {
            return None;
        }
        fn subtract<K: Copy + Ord>(
            upper: &BTreeMap<K, u64>,
            actual: &BTreeMap<K, u64>,
        ) -> Option<BTreeMap<K, u64>> {
            let mut remaining = upper.clone();
            for (key, amount) in actual {
                let next = remaining.get(key).copied()?.checked_sub(*amount)?;
                if next == 0 {
                    remaining.remove(key);
                } else {
                    remaining.insert(*key, next);
                }
            }
            Some(remaining)
        }
        let cumulative = subtract(&self.cumulative, &actual.cumulative)?;
        let concurrent = subtract(&self.concurrent, &actual.concurrent)?;
        let counters = subtract(&self.counters, &actual.counters)?;
        let maximum = self.maximum.clone();
        for (dimension, observed) in &actual.maximum {
            if *observed > maximum.get(dimension).copied()? {
                return None;
            }
        }
        Some(Self {
            phase: self.phase,
            surface: self.surface,
            cumulative,
            maximum,
            concurrent,
            counters,
            overflow_dimension: None,
            overflow_counter: None,
        })
    }

    fn is_empty(&self) -> bool {
        self.cumulative.is_empty()
            && self.maximum.is_empty()
            && self.concurrent.is_empty()
            && self.counters.is_empty()
    }
}

fn accumulate_reservation_value<K: Copy + Ord>(
    values: &mut BTreeMap<K, u64>,
    key: K,
    amount: u64,
    overflow: &mut Option<K>,
) {
    let current = values.get(&key).copied().unwrap_or_default();
    match current.checked_add(amount) {
        Some(next) => {
            values.insert(key, next);
        }
        None => {
            values.insert(key, u64::MAX);
            *overflow = Some(key);
        }
    }
}

#[derive(Debug)]
pub struct DecisionWorkLedger {
    ledger_id: u64,
    next_permit_id: u64,
    profile: DecisionWorkProfile,
    command_kind: DecisionWorkCommandKind,
    command: String,
    snapshot: DecisionWorkSnapshot,
    reserved: BTreeMap<u64, DecisionWorkReservation>,
    prepublication_sealed: bool,
}

impl DecisionWorkLedger {
    pub fn new(
        profile: DecisionWorkProfile,
        command_kind: DecisionWorkCommandKind,
        command: impl Into<String>,
    ) -> Result<Self, DecisionWorkAccountingError> {
        if !profile.applicable_commands.contains(&command_kind) {
            return Err(DecisionWorkAccountingError::CommandNotApplicable(
                command_kind,
            ));
        }
        let ledger_id = NEXT_DECISION_WORK_LEDGER_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| DecisionWorkAccountingError::SizeOverflow)?;
        Ok(Self {
            ledger_id,
            next_permit_id: 1,
            profile,
            command_kind,
            command: command.into(),
            snapshot: DecisionWorkSnapshot::default(),
            reserved: BTreeMap::new(),
            prepublication_sealed: false,
        })
    }

    pub fn profile(&self) -> &DecisionWorkProfile {
        &self.profile
    }

    pub fn snapshot(&self) -> &DecisionWorkSnapshot {
        &self.snapshot
    }

    /// Returns the opaque identity used by ledger-independent prepared
    /// owners. Such owners must compare this token before applying work,
    /// consuming permits, or releasing retained memory.
    pub fn identity(&self) -> DecisionWorkLedgerIdentity {
        DecisionWorkLedgerIdentity {
            ledger_id: self.ledger_id,
        }
    }

    pub fn matches_identity(&self, identity: DecisionWorkLedgerIdentity) -> bool {
        self.ledger_id == identity.ledger_id
    }

    /// Reports whether ordinary pre-publication accounting has been sealed.
    /// Prepared effect owners use this read-only witness before consuming a
    /// ledger-bound permit; only the combined transaction owner may seal.
    pub fn is_prepublication_sealed(&self) -> bool {
        self.prepublication_sealed
    }

    /// Side-effect-free witness used before a prepared owner transfers the
    /// ledger to an output writer.
    pub fn has_open_permits(&self) -> bool {
        !self.reserved.is_empty()
    }

    pub fn open_permit_count(&self) -> usize {
        self.reserved.len()
    }

    pub fn apply(
        &mut self,
        reservation: DecisionWorkReservation,
    ) -> Result<(), DecisionWorkAccountingError> {
        if self.prepublication_sealed {
            return Err(DecisionWorkAccountingError::PrepublicationSealed);
        }
        let mut actual = self.clone_for_accounting();
        if let Err(error) = actual.apply_direct(reservation) {
            return Err(self.propagate_capacity_error(error));
        }
        let mut capacity = actual.clone_for_accounting();
        for reserved in self.reserved.values().cloned() {
            if let Err(error) = capacity.apply_direct(reserved) {
                return Err(self.propagate_capacity_error(error));
            }
        }
        self.snapshot = actual.snapshot;
        Ok(())
    }

    fn apply_direct(
        &mut self,
        reservation: DecisionWorkReservation,
    ) -> Result<(), DecisionWorkAccountingError> {
        if let Some(reason) = self.snapshot.terminal_reason {
            return Err(DecisionWorkAccountingError::LedgerTerminated(reason));
        }
        if !self.profile.applicable_phases.contains(&reservation.phase) {
            return Err(DecisionWorkAccountingError::PhaseNotApplicable(
                reservation.phase,
            ));
        }
        if let Some(dimension) = reservation.overflow_dimension {
            return Err(self.fail_budget(dimension, reservation.phase, reservation.surface));
        }
        if reservation.overflow_counter.is_some() {
            return Err(self.fail_budget(
                DecisionWorkDimension::CounterValues,
                reservation.phase,
                reservation.surface,
            ));
        }

        let mut staged = Self {
            ledger_id: self.ledger_id,
            next_permit_id: self.next_permit_id,
            profile: self.profile.clone(),
            command_kind: self.command_kind,
            command: self.command.clone(),
            snapshot: self.snapshot.clone(),
            reserved: BTreeMap::new(),
            prepublication_sealed: false,
        };
        let result = (|| {
            for (dimension, amount) in reservation.cumulative {
                staged.charge(dimension, amount, reservation.phase, reservation.surface)?;
            }
            for (dimension, observed) in reservation.maximum {
                staged.observe_maximum(
                    dimension,
                    observed,
                    reservation.phase,
                    reservation.surface,
                )?;
            }
            for (dimension, amount) in reservation.concurrent {
                staged.acquire(dimension, amount, reservation.phase, reservation.surface)?;
            }
            for (counter, amount) in reservation.counters {
                staged.count(counter, amount, reservation.phase, reservation.surface)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.snapshot = staged.snapshot;
                Ok(())
            }
            Err(DecisionWorkAccountingError::BudgetExceeded(error)) => {
                self.snapshot.terminal_reason = Some(DecisionWorkTerminalReason::BudgetExceeded);
                Err(DecisionWorkAccountingError::BudgetExceeded(error))
            }
            Err(DecisionWorkAccountingError::SizeOverflow) => Err(self.fail_budget(
                DecisionWorkDimension::CounterValues,
                reservation.phase,
                reservation.surface,
            )),
            Err(error) => Err(error),
        }
    }

    pub fn reserve(
        &mut self,
        upper_bound: DecisionWorkReservation,
    ) -> Result<DecisionWorkPermit, DecisionWorkAccountingError> {
        if self.prepublication_sealed {
            return Err(DecisionWorkAccountingError::PrepublicationSealed);
        }
        let mut capacity = self.clone_for_accounting();
        for reserved in self.reserved.values().cloned() {
            capacity.apply_direct(reserved)?;
        }
        if let Err(error) = capacity.apply_direct(upper_bound.clone()) {
            return Err(self.propagate_capacity_error(error));
        }
        let permit_id = self.next_permit_id;
        self.next_permit_id = self
            .next_permit_id
            .checked_add(1)
            .ok_or(DecisionWorkAccountingError::SizeOverflow)?;
        self.reserved.insert(permit_id, upper_bound);
        Ok(DecisionWorkPermit {
            ledger_id: self.ledger_id,
            permit_id,
        })
    }

    pub fn consume_reserved(
        &mut self,
        permit: &mut DecisionWorkPermit,
        actual: DecisionWorkReservation,
    ) -> Result<(), DecisionWorkAccountingError> {
        if permit.ledger_id != self.ledger_id {
            return Err(DecisionWorkAccountingError::InvalidPermit);
        }
        let upper_bound = self
            .reserved
            .get(&permit.permit_id)
            .ok_or(DecisionWorkAccountingError::InvalidPermit)?;
        let remaining = upper_bound
            .remaining_after(&actual)
            .ok_or(DecisionWorkAccountingError::PermitExceeded)?;
        let mut actual_ledger = self.clone_for_accounting();
        actual_ledger.apply_direct(actual)?;
        let mut capacity = actual_ledger.clone_for_accounting();
        for (permit_id, reserved) in &self.reserved {
            if *permit_id == permit.permit_id {
                capacity.apply_direct(remaining.clone())?;
            } else {
                capacity.apply_direct(reserved.clone())?;
            }
        }
        self.snapshot = actual_ledger.snapshot;
        if remaining.is_empty() {
            self.reserved.remove(&permit.permit_id);
        } else {
            self.reserved.insert(permit.permit_id, remaining);
        }
        Ok(())
    }

    /// Atomically consumes realized work from two independent permits.
    ///
    /// This is used when one logical publication effect spans two phases (for
    /// example response-body encoding and response-header emission). Both
    /// permits, both actual reservations, and all remaining reserved capacity
    /// are preflighted on clones before either snapshot or permit remainder is
    /// changed.
    pub fn consume_reserved_pair(
        &mut self,
        first_permit: &DecisionWorkPermit,
        first_actual: DecisionWorkReservation,
        second_permit: &DecisionWorkPermit,
        second_actual: DecisionWorkReservation,
    ) -> Result<(), DecisionWorkAccountingError> {
        if first_permit.ledger_id != self.ledger_id
            || second_permit.ledger_id != self.ledger_id
            || first_permit.permit_id == second_permit.permit_id
        {
            return Err(DecisionWorkAccountingError::InvalidPermit);
        }
        let first_upper = self
            .reserved
            .get(&first_permit.permit_id)
            .ok_or(DecisionWorkAccountingError::InvalidPermit)?;
        let second_upper = self
            .reserved
            .get(&second_permit.permit_id)
            .ok_or(DecisionWorkAccountingError::InvalidPermit)?;
        let first_remaining = first_upper
            .remaining_after(&first_actual)
            .ok_or(DecisionWorkAccountingError::PermitExceeded)?;
        let second_remaining = second_upper
            .remaining_after(&second_actual)
            .ok_or(DecisionWorkAccountingError::PermitExceeded)?;

        let mut actual_ledger = self.clone_for_accounting();
        actual_ledger.apply_direct(first_actual)?;
        actual_ledger.apply_direct(second_actual)?;
        let mut capacity = actual_ledger.clone_for_accounting();
        for (permit_id, reserved) in &self.reserved {
            if *permit_id == first_permit.permit_id {
                if !first_remaining.is_empty() {
                    capacity.apply_direct(first_remaining.clone())?;
                }
            } else if *permit_id == second_permit.permit_id {
                if !second_remaining.is_empty() {
                    capacity.apply_direct(second_remaining.clone())?;
                }
            } else {
                capacity.apply_direct(reserved.clone())?;
            }
        }

        self.snapshot = actual_ledger.snapshot;
        if first_remaining.is_empty() {
            self.reserved.remove(&first_permit.permit_id);
        } else {
            self.reserved
                .insert(first_permit.permit_id, first_remaining);
        }
        if second_remaining.is_empty() {
            self.reserved.remove(&second_permit.permit_id);
        } else {
            self.reserved
                .insert(second_permit.permit_id, second_remaining);
        }
        Ok(())
    }

    pub fn cancel_reserved(
        &mut self,
        permit: &DecisionWorkPermit,
    ) -> Result<(), DecisionWorkAccountingError> {
        if permit.ledger_id != self.ledger_id || self.reserved.remove(&permit.permit_id).is_none() {
            return Err(DecisionWorkAccountingError::InvalidPermit);
        }
        Ok(())
    }

    pub fn seal_prepublication(&mut self) -> Result<(), DecisionWorkAccountingError> {
        if let Some(reason) = self.snapshot.terminal_reason {
            return Err(DecisionWorkAccountingError::LedgerTerminated(reason));
        }
        self.prepublication_sealed = true;
        Ok(())
    }

    fn clone_for_accounting(&self) -> Self {
        Self {
            ledger_id: self.ledger_id,
            next_permit_id: self.next_permit_id,
            profile: self.profile.clone(),
            command_kind: self.command_kind,
            command: self.command.clone(),
            snapshot: self.snapshot.clone(),
            reserved: BTreeMap::new(),
            prepublication_sealed: false,
        }
    }

    fn propagate_capacity_error(
        &mut self,
        error: DecisionWorkAccountingError,
    ) -> DecisionWorkAccountingError {
        if matches!(error, DecisionWorkAccountingError::BudgetExceeded(_)) {
            self.snapshot.terminal_reason = Some(DecisionWorkTerminalReason::BudgetExceeded);
        }
        error
    }

    fn charge(
        &mut self,
        dimension: DecisionWorkDimension,
        amount: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> Result<(), DecisionWorkAccountingError> {
        let limit = self.require_measurement(dimension, DecisionWorkMeasurement::Cumulative)?;
        let current = self
            .snapshot
            .cumulative
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        let next = self.checked_next(dimension, current, amount, limit, phase, surface)?;
        let phase_next = self.checked_phase_work(phase, dimension, amount)?;
        self.snapshot.cumulative.insert(dimension, next);
        self.snapshot
            .phases
            .entry(phase)
            .or_default()
            .insert(dimension, phase_next);
        Ok(())
    }

    fn observe_maximum(
        &mut self,
        dimension: DecisionWorkDimension,
        observed: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> Result<(), DecisionWorkAccountingError> {
        let limit = self.require_measurement(dimension, DecisionWorkMeasurement::Maximum)?;
        if observed > limit {
            return Err(self.exceeded(dimension, limit, phase, surface).into());
        }
        let previous = self
            .snapshot
            .maximum
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        if observed > previous {
            let phase_next = self.checked_phase_work(phase, dimension, observed - previous)?;
            self.snapshot.maximum.insert(dimension, observed);
            self.snapshot
                .phases
                .entry(phase)
                .or_default()
                .insert(dimension, phase_next);
        }
        Ok(())
    }

    fn acquire(
        &mut self,
        dimension: DecisionWorkDimension,
        amount: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> Result<(), DecisionWorkAccountingError> {
        let limit = self.require_measurement(dimension, DecisionWorkMeasurement::Concurrent)?;
        let current = self
            .snapshot
            .current
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        let next = self.checked_next(dimension, current, amount, limit, phase, surface)?;
        let phase_next = self.checked_phase_work(phase, dimension, amount)?;
        let peak = self
            .snapshot
            .peak
            .get(&dimension)
            .copied()
            .unwrap_or_default()
            .max(next);
        self.snapshot.current.insert(dimension, next);
        self.snapshot.peak.insert(dimension, peak);
        self.snapshot
            .phases
            .entry(phase)
            .or_default()
            .insert(dimension, phase_next);
        Ok(())
    }

    pub fn release(
        &mut self,
        dimension: DecisionWorkDimension,
        amount: u64,
    ) -> Result<(), DecisionWorkAccountingError> {
        self.require_measurement(dimension, DecisionWorkMeasurement::Concurrent)?;
        let current = self
            .snapshot
            .current
            .get(&dimension)
            .copied()
            .unwrap_or_default();
        let next =
            current
                .checked_sub(amount)
                .ok_or(DecisionWorkAccountingError::ReleaseUnderflow {
                    dimension,
                    current,
                    amount,
                })?;
        self.snapshot.current.insert(dimension, next);
        Ok(())
    }

    fn count(
        &mut self,
        counter: DecisionWorkCounter,
        amount: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> Result<(), DecisionWorkAccountingError> {
        let limit = self.require_measurement(
            DecisionWorkDimension::CounterValues,
            DecisionWorkMeasurement::Maximum,
        )?;
        let current = self
            .snapshot
            .counters
            .get(&counter)
            .copied()
            .unwrap_or_default();
        let next = current
            .checked_add(amount)
            .ok_or(DecisionWorkAccountingError::SizeOverflow)?;
        if next > limit {
            return Err(self
                .exceeded(DecisionWorkDimension::CounterValues, limit, phase, surface)
                .into());
        }
        let phase_current = self
            .snapshot
            .phase_counters
            .get(&phase)
            .and_then(|counters| counters.get(&counter))
            .copied()
            .unwrap_or_default();
        let phase_next = phase_current
            .checked_add(amount)
            .ok_or(DecisionWorkAccountingError::SizeOverflow)?;
        self.snapshot.counters.insert(counter, next);
        self.snapshot
            .phase_counters
            .entry(phase)
            .or_default()
            .insert(counter, phase_next);
        Ok(())
    }

    pub fn finish(
        &mut self,
        reason: DecisionWorkTerminalReason,
    ) -> Result<(), DecisionWorkAccountingError> {
        if !self.reserved.is_empty() {
            return Err(DecisionWorkAccountingError::OpenPermits);
        }
        if let Some(current) = self.snapshot.terminal_reason {
            return Err(DecisionWorkAccountingError::TerminalAlreadySet {
                current,
                attempted: reason,
            });
        }
        self.snapshot.terminal_reason = Some(reason);
        Ok(())
    }

    fn fail_budget(
        &mut self,
        dimension: DecisionWorkDimension,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> DecisionWorkAccountingError {
        self.snapshot.terminal_reason = Some(DecisionWorkTerminalReason::BudgetExceeded);
        let limit = self
            .profile
            .limit(dimension)
            .map(|configured| configured.limit)
            .unwrap_or(u64::MAX - 1);
        self.exceeded(dimension, limit, phase, surface).into()
    }

    fn require_measurement(
        &self,
        dimension: DecisionWorkDimension,
        expected: DecisionWorkMeasurement,
    ) -> Result<u64, DecisionWorkAccountingError> {
        let configured = self
            .profile
            .limit(dimension)
            .ok_or(DecisionWorkAccountingError::MissingLimit(dimension))?;
        if configured.limit == u64::MAX {
            return Err(DecisionWorkAccountingError::InvalidLimit(dimension));
        }
        if configured.measurement != expected {
            return Err(DecisionWorkAccountingError::MeasurementMismatch {
                dimension,
                expected,
                actual: configured.measurement,
            });
        }
        Ok(configured.limit)
    }

    fn checked_next(
        &self,
        dimension: DecisionWorkDimension,
        current: u64,
        amount: u64,
        limit: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> Result<u64, DecisionWorkAccountingError> {
        match current.checked_add(amount) {
            Some(next) if next <= limit => Ok(next),
            _ => Err(self.exceeded(dimension, limit, phase, surface).into()),
        }
    }

    fn checked_phase_work(
        &self,
        phase: DecisionWorkPhase,
        dimension: DecisionWorkDimension,
        amount: u64,
    ) -> Result<u64, DecisionWorkAccountingError> {
        self.snapshot
            .phases
            .get(&phase)
            .and_then(|dimensions| dimensions.get(&dimension))
            .copied()
            .unwrap_or_default()
            .checked_add(amount)
            .ok_or(DecisionWorkAccountingError::SizeOverflow)
    }

    fn exceeded(
        &self,
        dimension: DecisionWorkDimension,
        limit: u64,
        phase: DecisionWorkPhase,
        surface: DecisionWorkSurface,
    ) -> DecisionWorkBudgetExceeded {
        DecisionWorkBudgetExceeded {
            kind: "resource_budget_exceeded".to_string(),
            exit_code: RESOURCE_BUDGET_EXIT_CODE,
            profile: self.profile.profile_id.clone(),
            dimension,
            observed: limit.saturating_add(1),
            limit,
            phase,
            command: self.command.clone(),
            continuation_supported: surface.continuation_supported(),
        }
    }
}

pub fn logical_retained_frame_bytes(node_id: &str) -> Result<u64, DecisionWorkAccountingError> {
    let id_bytes =
        u64::try_from(node_id.len()).map_err(|_| DecisionWorkAccountingError::SizeOverflow)?;
    LOGICAL_FRAME_BASE_BYTES
        .checked_add(id_bytes)
        .ok_or(DecisionWorkAccountingError::SizeOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile(
        dimension: DecisionWorkDimension,
        limit: DecisionWorkLimit,
    ) -> DecisionWorkProfile {
        let mut limits = BTreeMap::from([(dimension, limit)]);
        limits
            .entry(DecisionWorkDimension::CounterValues)
            .or_insert(DecisionWorkLimit::maximum(u64::MAX - 1));
        DecisionWorkProfile {
            profile_id: DECISION_WORK_PROFILE_V1.to_string(),
            limits,
            applicable_commands: vec![DecisionWorkCommandKind::LocalGraph],
            applicable_phases: vec![
                DecisionWorkPhase::RequestHeaders,
                DecisionWorkPhase::RequestBody,
                DecisionWorkPhase::Validation,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkPhase::RecordRead,
                DecisionWorkPhase::SemanticValidation,
                DecisionWorkPhase::Query,
                DecisionWorkPhase::Encoding,
                DecisionWorkPhase::Staging,
                DecisionWorkPhase::SnapshotRevalidation,
                DecisionWorkPhase::PublicationPreflight,
                DecisionWorkPhase::ResponseHeaders,
                DecisionWorkPhase::ResponseBody,
            ],
            request_deadline_ms: None,
            max_active_connections: None,
            max_queued_connections: None,
        }
    }

    fn test_ledger(profile: DecisionWorkProfile, command: &str) -> DecisionWorkLedger {
        DecisionWorkLedger::new(profile, DecisionWorkCommandKind::LocalGraph, command).unwrap()
    }

    #[test]
    fn publication_permit_preserves_headroom_and_sealed_consumption_cannot_exhaust_budget() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::AggregateBytes,
                DecisionWorkLimit::cumulative(10),
            ),
            "publication permit",
        );
        let upper = DecisionWorkReservation::new(
            DecisionWorkPhase::PublicationPreflight,
            DecisionWorkSurface::Atomic,
        )
        .charge(DecisionWorkDimension::AggregateBytes, 7);
        let mut permit = ledger.reserve(upper).unwrap();

        ledger.seal_prepublication().unwrap();
        assert!(matches!(
            ledger.apply(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 1)
            ),
            Err(DecisionWorkAccountingError::PrepublicationSealed)
        ));
        ledger
            .consume_reserved(
                &mut permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 7),
            )
            .unwrap();
        assert_eq!(
            ledger
                .snapshot()
                .cumulative
                .get(&DecisionWorkDimension::AggregateBytes),
            Some(&7)
        );
        ledger.finish(DecisionWorkTerminalReason::Complete).unwrap();
    }

    #[test]
    fn publication_permit_pair_consumes_different_phases_atomically() {
        let mut profile = test_profile(
            DecisionWorkDimension::ResponseBytes,
            DecisionWorkLimit::cumulative(10),
        );
        profile.limits.insert(
            DecisionWorkDimension::ResponseHeaderBytes,
            DecisionWorkLimit::cumulative(10),
        );
        let mut ledger = test_ledger(profile, "publication permit pair");
        let response_permit = ledger
            .reserve(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::Encoding,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseBytes, 7),
            )
            .unwrap();
        let header_permit = ledger
            .reserve(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::ResponseHeaders,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseHeaderBytes, 5),
            )
            .unwrap();
        assert!(ledger.has_open_permits());
        assert_eq!(ledger.open_permit_count(), 2);
        ledger.seal_prepublication().unwrap();

        let before = ledger.snapshot().clone();
        assert!(matches!(
            ledger.consume_reserved_pair(
                &response_permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::Encoding,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseBytes, 4),
                &header_permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::ResponseHeaders,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseHeaderBytes, 6),
            ),
            Err(DecisionWorkAccountingError::PermitExceeded)
        ));
        assert_eq!(ledger.snapshot(), &before);
        assert_eq!(ledger.reserved.len(), 2);

        ledger
            .consume_reserved_pair(
                &response_permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::Encoding,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseBytes, 4),
                &header_permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::ResponseHeaders,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::ResponseHeaderBytes, 3),
            )
            .unwrap();
        assert_eq!(
            ledger.snapshot().phases[&DecisionWorkPhase::Encoding]
                [&DecisionWorkDimension::ResponseBytes],
            4
        );
        assert_eq!(
            ledger.snapshot().phases[&DecisionWorkPhase::ResponseHeaders]
                [&DecisionWorkDimension::ResponseHeaderBytes],
            3
        );
        ledger.cancel_reserved(&response_permit).unwrap();
        ledger.cancel_reserved(&header_permit).unwrap();
        assert!(!ledger.has_open_permits());
        assert_eq!(ledger.open_permit_count(), 0);
        ledger.finish(DecisionWorkTerminalReason::Complete).unwrap();
    }

    #[test]
    fn publication_permit_rejects_overconsumption_and_can_be_cancelled() {
        let profile = test_profile(
            DecisionWorkDimension::AggregateBytes,
            DecisionWorkLimit::cumulative(10),
        );
        let mut ledger = test_ledger(profile.clone(), "publication permit");
        let upper = DecisionWorkReservation::new(
            DecisionWorkPhase::PublicationPreflight,
            DecisionWorkSurface::Atomic,
        )
        .charge(DecisionWorkDimension::AggregateBytes, 4);
        let mut permit = ledger.reserve(upper).unwrap();
        assert!(matches!(
            ledger.consume_reserved(
                &mut permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 5)
            ),
            Err(DecisionWorkAccountingError::PermitExceeded)
        ));
        ledger.cancel_reserved(&permit).unwrap();
        ledger
            .apply(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 10),
            )
            .unwrap();

        let mut other = test_ledger(profile, "other publication permit");
        let mut foreign = other
            .reserve(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 1),
            )
            .unwrap();
        assert!(matches!(
            ledger.consume_reserved(
                &mut foreign,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 1)
            ),
            Err(DecisionWorkAccountingError::InvalidPermit)
        ));
    }

    #[test]
    fn publication_permit_headroom_blocks_unreserved_work_before_effect() {
        let profile = test_profile(
            DecisionWorkDimension::AggregateBytes,
            DecisionWorkLimit::cumulative(10),
        );
        let reserve_seven = || {
            DecisionWorkReservation::new(
                DecisionWorkPhase::PublicationPreflight,
                DecisionWorkSurface::Atomic,
            )
            .charge(DecisionWorkDimension::AggregateBytes, 7)
        };
        let apply = |amount| {
            DecisionWorkReservation::new(
                DecisionWorkPhase::PublicationPreflight,
                DecisionWorkSurface::Atomic,
            )
            .charge(DecisionWorkDimension::AggregateBytes, amount)
        };

        let mut exact = test_ledger(profile.clone(), "exact publication headroom");
        let permit = exact.reserve(reserve_seven()).unwrap();
        exact.apply(apply(3)).unwrap();
        assert_eq!(
            exact.snapshot().cumulative[&DecisionWorkDimension::AggregateBytes],
            3
        );
        exact.cancel_reserved(&permit).unwrap();

        let mut rejected = test_ledger(profile, "rejected publication headroom");
        let _permit = rejected.reserve(reserve_seven()).unwrap();
        let error = rejected.apply(apply(4)).unwrap_err();
        assert!(matches!(
            error,
            DecisionWorkAccountingError::BudgetExceeded(_)
        ));
        assert_eq!(
            rejected
                .snapshot()
                .cumulative
                .get(&DecisionWorkDimension::AggregateBytes)
                .copied()
                .unwrap_or_default(),
            0
        );
    }

    #[test]
    fn publication_permit_supports_repeated_maximum_and_concurrent_consumption() {
        let mut profile = test_profile(
            DecisionWorkDimension::LogicalDepth,
            DecisionWorkLimit::maximum(10),
        );
        profile.limits.insert(
            DecisionWorkDimension::RetainedBytes,
            DecisionWorkLimit::concurrent(8),
        );
        let mut ledger = test_ledger(profile, "maximum and concurrent publication permit");
        let mut permit = ledger
            .reserve(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .observe_maximum(DecisionWorkDimension::LogicalDepth, 10)
                .acquire(DecisionWorkDimension::RetainedBytes, 8),
            )
            .unwrap();
        ledger
            .consume_reserved(
                &mut permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .observe_maximum(DecisionWorkDimension::LogicalDepth, 6)
                .acquire(DecisionWorkDimension::RetainedBytes, 3),
            )
            .unwrap();
        ledger
            .release(DecisionWorkDimension::RetainedBytes, 3)
            .unwrap();
        ledger
            .consume_reserved(
                &mut permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .observe_maximum(DecisionWorkDimension::LogicalDepth, 7)
                .acquire(DecisionWorkDimension::RetainedBytes, 5),
            )
            .unwrap();
        assert_eq!(
            ledger.snapshot().maximum[&DecisionWorkDimension::LogicalDepth],
            7
        );
        assert_eq!(
            ledger.snapshot().peak[&DecisionWorkDimension::RetainedBytes],
            5
        );
        ledger
            .release(DecisionWorkDimension::RetainedBytes, 5)
            .unwrap();
        assert!(matches!(
            ledger.finish(DecisionWorkTerminalReason::Complete),
            Err(DecisionWorkAccountingError::OpenPermits)
        ));
        ledger.cancel_reserved(&permit).unwrap();
        ledger.finish(DecisionWorkTerminalReason::Complete).unwrap();
    }

    #[test]
    fn publication_permit_multidimension_failure_and_foreign_cancel_are_atomic() {
        let mut profile = test_profile(
            DecisionWorkDimension::AggregateBytes,
            DecisionWorkLimit::cumulative(10),
        );
        profile.limits.insert(
            DecisionWorkDimension::RetainedBytes,
            DecisionWorkLimit::concurrent(10),
        );
        let upper = || {
            DecisionWorkReservation::new(
                DecisionWorkPhase::PublicationPreflight,
                DecisionWorkSurface::Atomic,
            )
            .charge(DecisionWorkDimension::AggregateBytes, 4)
            .acquire(DecisionWorkDimension::RetainedBytes, 4)
        };
        let mut owner = test_ledger(profile.clone(), "permit owner");
        let mut permit = owner.reserve(upper()).unwrap();
        let before = owner.snapshot().clone();
        assert!(matches!(
            owner.consume_reserved(
                &mut permit,
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::AggregateBytes, 3)
                .acquire(DecisionWorkDimension::RetainedBytes, 5)
            ),
            Err(DecisionWorkAccountingError::PermitExceeded)
        ));
        assert_eq!(owner.snapshot(), &before);

        let mut foreign = test_ledger(profile, "foreign permit ledger");
        let foreign_before = foreign.snapshot().clone();
        assert!(matches!(
            foreign.cancel_reserved(&permit),
            Err(DecisionWorkAccountingError::InvalidPermit)
        ));
        assert_eq!(foreign.snapshot(), &foreign_before);
        assert_eq!(owner.snapshot(), &before);
        owner.cancel_reserved(&permit).unwrap();
        owner.finish(DecisionWorkTerminalReason::Complete).unwrap();
    }

    #[test]
    fn ledger_identity_binds_raw_prepared_owners_without_exposing_numeric_ids() {
        let profile = test_profile(
            DecisionWorkDimension::AggregateBytes,
            DecisionWorkLimit::cumulative(10),
        );
        let owner = test_ledger(profile.clone(), "prepared owner ledger");
        let foreign = test_ledger(profile, "prepared foreign ledger");

        let identity = owner.identity();

        assert!(owner.matches_identity(identity));
        assert!(!foreign.matches_identity(identity));
        assert_ne!(identity, foreign.identity());
    }

    #[test]
    fn publication_permit_counter_headroom_is_scoped_per_counter() {
        let mut profile = test_profile(
            DecisionWorkDimension::AggregateBytes,
            DecisionWorkLimit::cumulative(10),
        );
        profile.limits.insert(
            DecisionWorkDimension::CounterValues,
            DecisionWorkLimit::maximum(5),
        );
        let mut ledger = test_ledger(profile, "publication counter permit");
        let permit = ledger
            .reserve(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .count(DecisionWorkCounter::FilesRead, 4),
            )
            .unwrap();
        ledger
            .apply(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .count(DecisionWorkCounter::FilesRead, 1)
                .count(DecisionWorkCounter::BytesHashed, 5),
            )
            .unwrap();
        let before = ledger.snapshot().clone();
        assert!(matches!(
            ledger.apply(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::PublicationPreflight,
                    DecisionWorkSurface::Atomic,
                )
                .count(DecisionWorkCounter::FilesRead, 1)
            ),
            Err(DecisionWorkAccountingError::BudgetExceeded(_))
        ));
        assert_eq!(ledger.snapshot().counters, before.counters);
        ledger.cancel_reserved(&permit).unwrap();
    }

    #[test]
    fn cumulative_budget_accepts_exact_limit_and_rejects_next_work_before_effect() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::GraphNodes,
                DecisionWorkLimit::cumulative(3),
            ),
            "branch list",
        );

        ledger
            .charge(
                DecisionWorkDimension::GraphNodes,
                1,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .charge(
                DecisionWorkDimension::GraphNodes,
                2,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        let error = ledger
            .charge(
                DecisionWorkDimension::GraphNodes,
                1,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();
        let exhausted = error.budget_exceeded().unwrap();

        assert_eq!(exhausted.kind, "resource_budget_exceeded");
        assert_eq!(exhausted.exit_code, RESOURCE_BUDGET_EXIT_CODE);
        assert_eq!(exhausted.profile, DECISION_WORK_PROFILE_V1);
        assert_eq!(exhausted.dimension, DecisionWorkDimension::GraphNodes);
        assert_eq!(exhausted.observed, 4);
        assert_eq!(exhausted.limit, 3);
        assert_eq!(exhausted.phase, DecisionWorkPhase::Query);
        assert_eq!(exhausted.command, "branch list");
        assert!(!exhausted.continuation_supported);
        assert_eq!(
            ledger
                .snapshot()
                .cumulative
                .get(&DecisionWorkDimension::GraphNodes),
            Some(&3)
        );
        assert_eq!(
            ledger.snapshot().phases[&DecisionWorkPhase::Query][&DecisionWorkDimension::GraphNodes],
            3
        );
    }

    #[test]
    fn cumulative_overflow_is_a_budget_failure_without_mutating_counters() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::AggregateBytes,
                DecisionWorkLimit::cumulative(u64::MAX - 1),
            ),
            "object collect",
        );
        ledger
            .charge(
                DecisionWorkDimension::AggregateBytes,
                u64::MAX - 1,
                DecisionWorkPhase::RecordRead,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();

        let error = ledger
            .charge(
                DecisionWorkDimension::AggregateBytes,
                2,
                DecisionWorkPhase::RecordRead,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();
        let exhausted = error.budget_exceeded().unwrap();
        assert_eq!(exhausted.observed, u64::MAX);
        assert_eq!(
            ledger
                .snapshot()
                .cumulative
                .get(&DecisionWorkDimension::AggregateBytes),
            Some(&(u64::MAX - 1))
        );
    }

    #[test]
    fn maximum_uses_root_depth_zero_and_rejects_limit_plus_one() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::LogicalDepth,
                DecisionWorkLimit::maximum(2),
            ),
            "ancestry",
        );
        ledger
            .observe_maximum(
                DecisionWorkDimension::LogicalDepth,
                0,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .observe_maximum(
                DecisionWorkDimension::LogicalDepth,
                2,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        let error = ledger
            .observe_maximum(
                DecisionWorkDimension::LogicalDepth,
                3,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();

        assert_eq!(error.budget_exceeded().unwrap().observed, 3);
        assert_eq!(
            ledger
                .snapshot()
                .maximum
                .get(&DecisionWorkDimension::LogicalDepth),
            Some(&2)
        );
    }

    #[test]
    fn concurrent_budget_tracks_peak_and_release_without_resetting_phase_work() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::RetainedBytes,
                DecisionWorkLimit::concurrent(10),
            ),
            "bundle",
        );
        ledger
            .acquire(
                DecisionWorkDimension::RetainedBytes,
                6,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .release(DecisionWorkDimension::RetainedBytes, 4)
            .unwrap();
        ledger
            .acquire(
                DecisionWorkDimension::RetainedBytes,
                8,
                DecisionWorkPhase::Encoding,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        let error = ledger
            .acquire(
                DecisionWorkDimension::RetainedBytes,
                1,
                DecisionWorkPhase::Encoding,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();

        assert_eq!(error.budget_exceeded().unwrap().observed, 11);
        assert_eq!(
            ledger
                .snapshot()
                .current
                .get(&DecisionWorkDimension::RetainedBytes),
            Some(&10)
        );
        assert_eq!(
            ledger
                .snapshot()
                .peak
                .get(&DecisionWorkDimension::RetainedBytes),
            Some(&10)
        );
        assert_eq!(
            ledger.snapshot().phases[&DecisionWorkPhase::GraphDiscovery]
                [&DecisionWorkDimension::RetainedBytes],
            6
        );
        assert_eq!(
            ledger.snapshot().phases[&DecisionWorkPhase::Encoding]
                [&DecisionWorkDimension::RetainedBytes],
            8
        );
    }

    #[test]
    fn retained_frame_accounting_is_utf8_byte_based() {
        assert_eq!(logical_retained_frame_bytes("node").unwrap(), 260);
        assert_eq!(logical_retained_frame_bytes("火").unwrap(), 259);
    }

    #[test]
    fn built_in_profiles_freeze_http_limits_and_separate_deadline() {
        let profile = DecisionWorkProfile::http_v1();
        assert_eq!(profile.profile_id(), DECISION_WORK_PROFILE_V1);
        assert_eq!(
            profile.request_deadline_ms(),
            Some(HTTP_REQUEST_DEADLINE_MS)
        );
        assert_eq!(profile.max_active_connections(), Some(32));
        assert_eq!(profile.max_queued_connections(), Some(0));
        assert_eq!(
            profile.limit(DecisionWorkDimension::ResponseBytes),
            Some(DecisionWorkLimit::cumulative(16 * 1024 * 1024))
        );
    }

    #[test]
    fn stable_counter_vocabulary_records_correctness_work_and_terminal_reason() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::GraphNodes,
                DecisionWorkLimit::cumulative(10),
            ),
            "graph query",
        );
        ledger
            .count(
                DecisionWorkCounter::UniqueNodesDiscovered,
                2,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .count(
                DecisionWorkCounter::MemoHits,
                1,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .finish(DecisionWorkTerminalReason::EarlyMatch)
            .unwrap();

        assert_eq!(
            ledger.snapshot().counters[&DecisionWorkCounter::UniqueNodesDiscovered],
            2
        );
        assert_eq!(
            ledger.snapshot().phase_counters[&DecisionWorkPhase::Query]
                [&DecisionWorkCounter::MemoHits],
            1
        );
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(DecisionWorkTerminalReason::EarlyMatch)
        );
    }

    #[test]
    fn acquire_preflights_phase_counter_before_mutating_concurrent_state() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::RetainedBytes,
                DecisionWorkLimit::concurrent(10),
            ),
            "bundle",
        );
        ledger.snapshot.phases.insert(
            DecisionWorkPhase::Encoding,
            BTreeMap::from([(DecisionWorkDimension::RetainedBytes, u64::MAX)]),
        );

        let error = ledger
            .acquire(
                DecisionWorkDimension::RetainedBytes,
                1,
                DecisionWorkPhase::Encoding,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();

        assert_eq!(error, DecisionWorkAccountingError::SizeOverflow);
        assert!(!ledger
            .snapshot()
            .current
            .contains_key(&DecisionWorkDimension::RetainedBytes));
        assert!(!ledger
            .snapshot()
            .peak
            .contains_key(&DecisionWorkDimension::RetainedBytes));
    }

    #[test]
    fn reservation_preflights_every_dimension_and_counter_before_one_commit() {
        let mut profile = test_profile(
            DecisionWorkDimension::GraphNodes,
            DecisionWorkLimit::cumulative(1),
        );
        profile.limits.extend([
            (
                DecisionWorkDimension::AggregateBytes,
                DecisionWorkLimit::cumulative(1),
            ),
            (
                DecisionWorkDimension::RetainedBytes,
                DecisionWorkLimit::concurrent(1),
            ),
        ]);
        let mut ledger = test_ledger(profile, "object read");
        let reservation = DecisionWorkReservation::new(
            DecisionWorkPhase::RecordRead,
            DecisionWorkSurface::Atomic,
        )
        .charge(DecisionWorkDimension::GraphNodes, 1)
        .charge(DecisionWorkDimension::AggregateBytes, 2)
        .acquire(DecisionWorkDimension::RetainedBytes, 1)
        .count(DecisionWorkCounter::UniqueNodesDiscovered, 1)
        .count(DecisionWorkCounter::RecordReads, 1);

        let error = ledger.apply(reservation).unwrap_err();

        assert_eq!(
            error.budget_exceeded().unwrap().dimension,
            DecisionWorkDimension::AggregateBytes
        );
        assert!(ledger.snapshot().cumulative.is_empty());
        assert!(ledger.snapshot().current.is_empty());
        assert!(ledger.snapshot().peak.is_empty());
        assert!(ledger.snapshot().counters.is_empty());
        assert!(ledger.snapshot().phases.is_empty());
        assert!(ledger.snapshot().phase_counters.is_empty());
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(DecisionWorkTerminalReason::BudgetExceeded)
        );
        assert!(matches!(
            ledger.finish(DecisionWorkTerminalReason::Complete),
            Err(DecisionWorkAccountingError::TerminalAlreadySet { .. })
        ));
    }

    #[test]
    fn reservation_counter_overflow_is_typed_budget_exhaustion() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::CounterValues,
                DecisionWorkLimit::maximum(u64::MAX - 1),
            ),
            "counter overflow",
        );
        let reservation =
            DecisionWorkReservation::new(DecisionWorkPhase::Query, DecisionWorkSurface::Atomic)
                .count(DecisionWorkCounter::MemoHits, u64::MAX)
                .count(DecisionWorkCounter::MemoHits, 1);

        let error = ledger.apply(reservation).unwrap_err();
        let exhausted = error.budget_exceeded().unwrap();
        assert_eq!(exhausted.dimension, DecisionWorkDimension::CounterValues);
        assert_eq!(exhausted.limit, u64::MAX - 1);
        assert_eq!(exhausted.observed, u64::MAX);
        assert_eq!(
            ledger.snapshot().terminal_reason,
            Some(DecisionWorkTerminalReason::BudgetExceeded)
        );
    }

    #[test]
    fn counter_values_accept_exact_limit_and_reject_limit_plus_one_atomically() {
        let mut exact = test_ledger(
            test_profile(
                DecisionWorkDimension::CounterValues,
                DecisionWorkLimit::maximum(u64::MAX - 1),
            ),
            "counter boundary",
        );
        exact
            .apply(
                DecisionWorkReservation::new(DecisionWorkPhase::Query, DecisionWorkSurface::Atomic)
                    .count(DecisionWorkCounter::MemoHits, u64::MAX - 1),
            )
            .unwrap();
        let before = exact.snapshot().clone();
        let error = exact
            .apply(
                DecisionWorkReservation::new(DecisionWorkPhase::Query, DecisionWorkSurface::Atomic)
                    .count(DecisionWorkCounter::MemoHits, 1),
            )
            .unwrap_err();
        assert_eq!(
            error.budget_exceeded().unwrap().dimension,
            DecisionWorkDimension::CounterValues
        );
        assert_eq!(exact.snapshot().counters, before.counters);
        assert_eq!(exact.snapshot().phase_counters, before.phase_counters);
        assert_eq!(
            exact.snapshot().terminal_reason,
            Some(DecisionWorkTerminalReason::BudgetExceeded)
        );

        let mut over = test_ledger(
            test_profile(
                DecisionWorkDimension::CounterValues,
                DecisionWorkLimit::maximum(u64::MAX - 1),
            ),
            "counter boundary",
        );
        let error = over
            .apply(
                DecisionWorkReservation::new(DecisionWorkPhase::Query, DecisionWorkSurface::Atomic)
                    .count(DecisionWorkCounter::MemoHits, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(error.budget_exceeded().unwrap().observed, u64::MAX);
        assert!(over.snapshot().counters.is_empty());
    }

    #[test]
    fn command_and_phase_applicability_are_enforced() {
        let profile = test_profile(
            DecisionWorkDimension::GraphNodes,
            DecisionWorkLimit::cumulative(1),
        );
        assert!(matches!(
            DecisionWorkLedger::new(
                profile.clone(),
                DecisionWorkCommandKind::HttpRequest,
                "wrong command"
            ),
            Err(DecisionWorkAccountingError::CommandNotApplicable(
                DecisionWorkCommandKind::HttpRequest
            ))
        ));
        let mut restricted = profile;
        restricted.applicable_phases = vec![DecisionWorkPhase::Query];
        let mut ledger = test_ledger(restricted, "wrong phase");
        let error = ledger
            .apply(
                DecisionWorkReservation::new(
                    DecisionWorkPhase::Encoding,
                    DecisionWorkSurface::Atomic,
                )
                .charge(DecisionWorkDimension::GraphNodes, 1),
            )
            .unwrap_err();
        assert_eq!(
            error,
            DecisionWorkAccountingError::PhaseNotApplicable(DecisionWorkPhase::Encoding)
        );
        assert!(ledger.snapshot().cumulative.is_empty());
    }

    #[test]
    fn continuation_is_selected_per_failing_surface_not_per_ledger() {
        let profile = test_profile(
            DecisionWorkDimension::GraphNodes,
            DecisionWorkLimit::cumulative(0),
        );
        let mut atomic = test_ledger(profile.clone(), "graph bundle");
        let atomic_error = atomic
            .charge(
                DecisionWorkDimension::GraphNodes,
                1,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();
        let mut paged = test_ledger(profile, "branch list");
        let paged_error = paged
            .charge(
                DecisionWorkDimension::GraphNodes,
                1,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::SnapshotPaged,
            )
            .unwrap_err();

        assert!(
            !atomic_error
                .budget_exceeded()
                .unwrap()
                .continuation_supported
        );
        assert!(
            paged_error
                .budget_exceeded()
                .unwrap()
                .continuation_supported
        );
    }

    #[test]
    fn maximum_u64_profile_limit_is_rejected_before_accounting() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::GraphNodes,
                DecisionWorkLimit::cumulative(u64::MAX),
            ),
            "graph query",
        );
        let error = ledger
            .charge(
                DecisionWorkDimension::GraphNodes,
                1,
                DecisionWorkPhase::Query,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err();
        assert_eq!(
            error,
            DecisionWorkAccountingError::InvalidLimit(DecisionWorkDimension::GraphNodes)
        );
    }

    #[test]
    fn resource_budget_diagnostic_has_stable_canonical_json() {
        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::ResponseBytes,
                DecisionWorkLimit::cumulative(2),
            ),
            "http bundle",
        );
        let diagnostic = ledger
            .charge(
                DecisionWorkDimension::ResponseBytes,
                3,
                DecisionWorkPhase::ResponseBody,
                DecisionWorkSurface::Atomic,
            )
            .unwrap_err()
            .budget_exceeded()
            .unwrap()
            .clone();
        let actual = serde_json::to_value(diagnostic).unwrap();
        let expected = serde_json::json!({
            "kind": "resource_budget_exceeded",
            "exit_code": 60,
            "profile": "codefire.decision-work.v1",
            "dimension": "response_bytes",
            "observed": 3,
            "limit": 2,
            "phase": "response_body",
            "command": "http bundle",
            "continuation_supported": false
        });
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::to_string(&actual).unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
    }

    #[test]
    fn profile_and_snapshot_have_stable_canonical_json_shapes() {
        let local_profile = DecisionWorkProfile::local_v1();
        let local_value = serde_json::to_value(&local_profile).unwrap();
        assert_eq!(
            local_value["limits"]["retained_bytes"],
            serde_json::json!({"measurement": "concurrent", "limit": 1024 * 1024 * 1024_u64})
        );
        assert_eq!(
            local_value["limits"]["temporary_bytes"],
            serde_json::json!({"measurement": "concurrent", "limit": 1024 * 1024 * 1024_u64})
        );
        let profile = DecisionWorkProfile::http_v1();
        let profile_value = serde_json::to_value(&profile).unwrap();
        assert_eq!(profile_value["profile_id"], DECISION_WORK_PROFILE_V1);
        assert_eq!(profile_value["request_deadline_ms"], 30_000);
        assert_eq!(profile_value["max_active_connections"], 32);
        assert_eq!(profile_value["max_queued_connections"], 0);
        assert_eq!(
            profile_value["limits"]["retained_bytes"],
            serde_json::json!({"measurement": "concurrent", "limit": 32 * 1024 * 1024_u64})
        );
        assert_eq!(
            profile_value["limits"]["temporary_bytes"],
            serde_json::json!({"measurement": "concurrent", "limit": 16 * 1024 * 1024_u64})
        );
        assert_eq!(
            profile_value["limits"]["maximum_frontier"],
            serde_json::json!({"measurement": "maximum", "limit": 100_000})
        );
        assert_eq!(
            profile_value["limits"]["snapshot_revalidation_records"],
            serde_json::json!({"measurement": "cumulative", "limit": 100_000})
        );

        let mut ledger = test_ledger(
            test_profile(
                DecisionWorkDimension::GraphNodes,
                DecisionWorkLimit::cumulative(2),
            ),
            "graph query",
        );
        ledger
            .charge(
                DecisionWorkDimension::GraphNodes,
                2,
                DecisionWorkPhase::GraphDiscovery,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger
            .count(
                DecisionWorkCounter::RecordReads,
                1,
                DecisionWorkPhase::RecordRead,
                DecisionWorkSurface::Atomic,
            )
            .unwrap();
        ledger.finish(DecisionWorkTerminalReason::Complete).unwrap();
        let actual = serde_json::to_value(ledger.snapshot()).unwrap();
        let expected = serde_json::json!({
            "cumulative": {"graph_nodes": 2},
            "maximum": {},
            "current": {},
            "peak": {},
            "counters": {"record_reads": 1},
            "phases": {"graph_discovery": {"graph_nodes": 2}},
            "phase_counters": {"record_read": {"record_reads": 1}},
            "terminal_reason": "complete"
        });
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::to_string(&actual).unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
    }

    #[test]
    fn local_concurrent_memory_limits_accept_exact_and_reject_plus_one() {
        for dimension in [
            DecisionWorkDimension::RetainedBytes,
            DecisionWorkDimension::TemporaryBytes,
        ] {
            let profile = DecisionWorkProfile::local_v1();
            let limit = profile.limit(dimension).unwrap().limit;
            let mut exact = DecisionWorkLedger::new(
                profile.clone(),
                DecisionWorkCommandKind::LocalGraph,
                "local memory exact",
            )
            .unwrap();
            exact
                .apply(
                    DecisionWorkReservation::new(
                        DecisionWorkPhase::RecordRead,
                        DecisionWorkSurface::Atomic,
                    )
                    .acquire(dimension, limit),
                )
                .unwrap();
            assert_eq!(exact.snapshot().current[&dimension], limit);
            assert_eq!(exact.snapshot().peak[&dimension], limit);

            let mut exceeded = DecisionWorkLedger::new(
                profile,
                DecisionWorkCommandKind::LocalGraph,
                "local memory plus one",
            )
            .unwrap();
            exceeded
                .apply(
                    DecisionWorkReservation::new(
                        DecisionWorkPhase::RecordRead,
                        DecisionWorkSurface::Atomic,
                    )
                    .acquire(dimension, limit),
                )
                .unwrap();
            let exhaustion = exceeded
                .apply(
                    DecisionWorkReservation::new(
                        DecisionWorkPhase::RecordRead,
                        DecisionWorkSurface::Atomic,
                    )
                    .acquire(dimension, 1),
                )
                .unwrap_err()
                .budget_exceeded()
                .unwrap()
                .clone();
            assert_eq!(exhaustion.dimension, dimension);
            assert_eq!(exhaustion.observed, limit + 1);
            assert_eq!(exhaustion.limit, limit);
        }
    }
}
