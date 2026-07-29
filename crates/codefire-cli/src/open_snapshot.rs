use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};

pub(crate) const OPEN_SNAPSHOT_DIVERGENCE_KIND: &str = "open_snapshot_divergence";

/// Proof that every stable authority describing an open workspace agreed at
/// the point it was loaded. The fields remain in [`OpenContext`]; this token
/// prevents callers from constructing a context without running the shared
/// validator first.
#[derive(Debug)]
pub(crate) struct VerifiedOpenSnapshot {
    pub(crate) repo_root: PathBuf,
    pub(crate) open_dir: PathBuf,
    pub(crate) branch: String,
    pub(crate) disposition: OpenSnapshotDisposition,
    _repository_id: String,
    _open_instance_id: String,
    _branch_head: String,
    _state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OpenSnapshotDisposition {
    Steady,
    PendingCommit {
        phase: String,
    },
    PendingState {
        operation: String,
        phase: String,
    },
    PendingActiveLedger {
        operation: String,
        phase: String,
        request_hash: String,
    },
}

#[derive(Debug)]
pub(crate) struct VerifiedOpenIdentity {
    _private: (),
}

#[derive(Debug)]
pub(crate) struct VerifiedOpenLocator {
    _private: (),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenSnapshotDivergence {
    pub(crate) phase: String,
    pub(crate) authority: &'static str,
    pub(crate) expected: String,
    pub(crate) actual: String,
    pub(crate) expected_record: PathBuf,
    pub(crate) actual_record: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) branch: String,
}

impl fmt::Display for OpenSnapshotDivergence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "open snapshot divergence at phase {} for {}: expected {} from {}, found {} in {}",
            self.phase,
            self.authority,
            self.expected,
            self.expected_record.display(),
            self.actual,
            self.actual_record.display()
        )
    }
}

impl std::error::Error for OpenSnapshotDivergence {}

pub(crate) struct OpenSnapshotIdentityAuthorities<'a> {
    pub(crate) repo_root: &'a Path,
    pub(crate) repository_record_path: &'a Path,
    pub(crate) marker_path: &'a Path,
    pub(crate) branch_record_path: &'a Path,
    pub(crate) registry_path: &'a Path,
    pub(crate) expected_repository_path: &'a Path,
    pub(crate) marker_repository_path: &'a Path,
    pub(crate) repository_id: &'a str,
    pub(crate) marker_repository_id: &'a str,
    pub(crate) branch: &'a str,
    pub(crate) marker_branch: &'a str,
    pub(crate) registry_branch: &'a str,
    pub(crate) branch_record_name: &'a str,
    pub(crate) open_dir: &'a Path,
    pub(crate) marker_open_path: &'a Path,
    pub(crate) registry_open_path: &'a Path,
    pub(crate) marker_open_instance_id: &'a str,
    pub(crate) registry_open_instance_id: &'a str,
    pub(crate) marker_opened_from_commit: &'a str,
    pub(crate) registry_opened_from_commit: &'a str,
    pub(crate) expected_active_state_path: &'a Path,
    pub(crate) registry_active_state_path: &'a Path,
    pub(crate) branch_head: &'a str,
    pub(crate) registry_current_base: &'a str,
    pub(crate) pending_commit_transaction: Option<&'a Value>,
}

pub(crate) struct OpenSnapshotStateAuthorities<'a> {
    pub(crate) active_state_record_path: &'a Path,
    pub(crate) branch_state: &'a str,
    pub(crate) registry_state: &'a str,
    pub(crate) active_state: &'a str,
    pub(crate) pending_commit_transaction: Option<&'a Value>,
    pub(crate) pending_state_transaction: Option<&'a Value>,
    pub(crate) active_ledger_descriptor:
        Option<&'a crate::fire::active_ledger_transaction::ValidatedActiveLedgerDescriptor>,
}

pub(crate) fn verify_open_snapshot(
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    state: &OpenSnapshotStateAuthorities<'_>,
) -> Result<VerifiedOpenSnapshot, Box<OpenSnapshotDivergence>> {
    let verified_identity = verify_open_snapshot_identity(identity)?;
    verify_open_snapshot_state(&verified_identity, identity, state)
}

pub(crate) fn verify_open_snapshot_identity(
    authorities: &OpenSnapshotIdentityAuthorities<'_>,
) -> Result<VerifiedOpenIdentity, Box<OpenSnapshotDivergence>> {
    let _verified_locator = verify_open_snapshot_locator(authorities)?;
    if authorities.branch_head != authorities.registry_current_base
        && !valid_commit_publication_window(authorities)
    {
        return Err(Box::new(divergence(
            authorities,
            "branch_head",
            authorities.branch_head,
            authorities.registry_current_base,
            authorities.branch_record_path,
            authorities.registry_path,
        )));
    }

    Ok(VerifiedOpenIdentity { _private: () })
}

pub(crate) fn verify_open_snapshot_locator(
    authorities: &OpenSnapshotIdentityAuthorities<'_>,
) -> Result<VerifiedOpenLocator, Box<OpenSnapshotDivergence>> {
    ensure_path_equal(
        authorities,
        "repository_path",
        authorities.expected_repository_path,
        authorities.marker_repository_path,
        authorities.repository_record_path,
        authorities.marker_path,
    )?;
    ensure_equal(
        authorities,
        "repository_id",
        authorities.repository_id,
        authorities.marker_repository_id,
        authorities.repository_record_path,
        authorities.marker_path,
    )?;
    ensure_equal(
        authorities,
        "marker_branch",
        authorities.branch,
        authorities.marker_branch,
        authorities.branch_record_path,
        authorities.marker_path,
    )?;
    ensure_equal(
        authorities,
        "registry_branch",
        authorities.branch,
        authorities.registry_branch,
        authorities.branch_record_path,
        authorities.registry_path,
    )?;
    ensure_equal(
        authorities,
        "branch_record_name",
        authorities.branch,
        authorities.branch_record_name,
        authorities.registry_path,
        authorities.branch_record_path,
    )?;
    ensure_path_equal(
        authorities,
        "marker_open_path",
        authorities.open_dir,
        authorities.marker_open_path,
        authorities.registry_path,
        authorities.marker_path,
    )?;
    ensure_path_equal(
        authorities,
        "registry_open_path",
        authorities.open_dir,
        authorities.registry_open_path,
        authorities.marker_path,
        authorities.registry_path,
    )?;
    ensure_equal(
        authorities,
        "open_instance_id",
        authorities.marker_open_instance_id,
        authorities.registry_open_instance_id,
        authorities.marker_path,
        authorities.registry_path,
    )?;
    ensure_equal(
        authorities,
        "opened_from_commit",
        authorities.marker_opened_from_commit,
        authorities.registry_opened_from_commit,
        authorities.marker_path,
        authorities.registry_path,
    )?;
    ensure_path_equal(
        authorities,
        "active_state_path",
        authorities.expected_active_state_path,
        authorities.registry_active_state_path,
        authorities.registry_path,
        authorities.registry_path,
    )?;

    Ok(VerifiedOpenLocator { _private: () })
}

pub(crate) fn verify_open_snapshot_state(
    _verified_identity: &VerifiedOpenIdentity,
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    state: &OpenSnapshotStateAuthorities<'_>,
) -> Result<VerifiedOpenSnapshot, Box<OpenSnapshotDivergence>> {
    let pending_state = match state.pending_state_transaction {
        Some(_) => Some(
            valid_pending_state_transition(identity, state).ok_or_else(|| {
                Box::new(transaction_divergence(
                    identity,
                    state,
                    "pending_state_transaction",
                    "valid subject-bound state transaction",
                    "invalid or foreign state transaction",
                ))
            })?,
        ),
        None => None,
    };
    let pending_commit = match identity.pending_commit_transaction {
        Some(_) => Some(
            valid_commit_transaction_for_subject(identity).ok_or_else(|| {
                Box::new(transaction_divergence(
                    identity,
                    state,
                    "pending_commit_transaction",
                    "valid subject-bound commit transaction",
                    "invalid or foreign commit transaction",
                ))
            })?,
        ),
        None => None,
    };
    let active_descriptor = state.active_ledger_descriptor;
    let active_pending = active_descriptor.filter(|descriptor| {
        descriptor.publication_status
            != crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released
    });
    if pending_state.is_some() && pending_commit.is_some() {
        return Err(Box::new(transaction_divergence(
            identity,
            state,
            "pending_transaction",
            "one pending open-state transaction",
            "simultaneous commit and state transactions",
        )));
    }
    if pending_commit.is_some() {
        if let Some(descriptor) = active_pending {
            return Err(Box::new(active_ledger_divergence(
                identity,
                state,
                &descriptor.phase,
                "one pending open-state transaction",
                "simultaneous commit and active-ledger transactions",
            )));
        }
    }
    if let Some(transaction) = state.pending_state_transaction {
        let operation = transaction
            .get("operation")
            .and_then(Value::as_str)
            .expect("validated state transaction operation");
        let active_state_operation = matches!(
            operation,
            "fire" | "fire-batch" | "extinguish" | "extinguish-batch"
        );
        if active_state_operation {
            let pair_is_valid = active_descriptor
                .is_some_and(|descriptor| valid_active_ledger_state_pair(transaction, descriptor));
            if !pair_is_valid {
                return Err(Box::new(active_ledger_divergence(
                    identity,
                    state,
                    transaction
                        .get("phase")
                        .and_then(Value::as_str)
                        .unwrap_or("pending"),
                    "matching completed active-ledger receipt and state transaction",
                    "missing, pending, released too early, or foreign active-ledger receipt",
                )));
            }
        } else if let Some(descriptor) = active_pending {
            return Err(Box::new(active_ledger_divergence(
                identity,
                state,
                &descriptor.phase,
                "one pending open-state transaction",
                "simultaneous state and active-ledger transactions",
            )));
        }
    }

    if pending_state.is_none() && !valid_commit_state_transition(identity, state) {
        ensure_equal(
            identity,
            "branch_state",
            state.branch_state,
            state.registry_state,
            identity.branch_record_path,
            identity.registry_path,
        )?;
        ensure_equal(
            identity,
            "active_state",
            state.branch_state,
            state.active_state,
            identity.branch_record_path,
            state.active_state_record_path,
        )?;
    }

    let disposition = if let Some((operation, phase)) = pending_state {
        OpenSnapshotDisposition::PendingState { operation, phase }
    } else if let Some(transaction) = pending_commit {
        OpenSnapshotDisposition::PendingCommit {
            phase: transaction
                .get("phase")
                .and_then(Value::as_str)
                .expect("validated commit transaction phase")
                .to_string(),
        }
    } else if let Some(descriptor) = active_pending {
        let phase = match descriptor.publication_status {
            crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::LedgerPending => {
                descriptor.phase.clone()
            }
            crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::StatePublicationPending => {
                "state_publication_pending".to_string()
            }
            crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => {
                unreachable!("released active-ledger receipts are steady")
            }
        };
        OpenSnapshotDisposition::PendingActiveLedger {
            operation: descriptor.operation.clone(),
            phase,
            request_hash: descriptor.request_hash.clone(),
        }
    } else {
        OpenSnapshotDisposition::Steady
    };
    Ok(VerifiedOpenSnapshot {
        repo_root: identity.repo_root.to_path_buf(),
        open_dir: identity.open_dir.to_path_buf(),
        branch: identity.branch.to_string(),
        disposition,
        _repository_id: identity.repository_id.to_string(),
        _open_instance_id: identity.marker_open_instance_id.to_string(),
        _branch_head: identity.branch_head.to_string(),
        _state: state.branch_state.to_string(),
    })
}

fn valid_pending_state_transition(
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    authorities: &OpenSnapshotStateAuthorities<'_>,
) -> Option<(String, String)> {
    let transaction = authorities.pending_state_transaction?;
    if crate::validate_state_transaction_record(transaction).is_err()
        || transaction.get("version").and_then(Value::as_u64) != Some(2)
        || transaction
            .pointer("/subject/branch")
            .and_then(Value::as_str)
            != Some(identity.branch)
        || transaction
            .pointer("/subject/open_instance_id")
            .and_then(Value::as_str)
            != Some(identity.marker_open_instance_id)
        || transaction
            .pointer("/subject/current_base_commit")
            .and_then(Value::as_str)
            != Some(identity.registry_current_base)
    {
        return None;
    }
    let operation = transaction.get("operation")?.as_str()?;
    let phase = transaction.get("phase")?.as_str()?;
    let intended = transaction.get("intended_state")?.as_str()?;
    let pre = transaction.pointer("/pre_state/branch")?.as_str()?;
    if transaction
        .pointer("/pre_state/registry")
        .and_then(Value::as_str)
        != Some(pre)
        || transaction
            .pointer("/pre_state/active")
            .and_then(Value::as_str)
            != Some(pre)
    {
        return None;
    }
    let actual = (
        authorities.branch_state,
        authorities.registry_state,
        authorities.active_state,
    );
    let preimage = (pre, pre, pre);
    let registry_written = (pre, intended, pre);
    let active_written = (pre, intended, intended);
    let branch_written = (intended, intended, intended);
    let active_ledger_operation = matches!(
        operation,
        "fire" | "fire-batch" | "extinguish" | "extinguish-batch"
    );
    let allowed = match phase {
        "started" if active_ledger_operation => {
            // The registry record is atomically replaced before the phase
            // marker advances. An exact owning retry may therefore observe
            // the postimage while the marker still says `started`.
            actual == preimage || actual == registry_written
        }
        "started" | "verification_invalidated" | "fires_written" => actual == preimage,
        "scan_written" | "verification_written" => actual == preimage || actual == registry_written,
        "registry_written" => actual == registry_written || actual == active_written,
        "active_state_written" => actual == active_written || actual == branch_written,
        "branch_written" => actual == branch_written,
        _ => false,
    };
    allowed.then(|| (operation.to_string(), phase.to_string()))
}

fn valid_active_ledger_state_pair(
    transaction: &Value,
    descriptor: &crate::fire::active_ledger_transaction::ValidatedActiveLedgerDescriptor,
) -> bool {
    let operation = transaction.get("operation").and_then(Value::as_str);
    let phase = transaction.get("phase").and_then(Value::as_str);
    let request_hash = transaction.get("request_hash").and_then(Value::as_str);
    if operation != Some(descriptor.operation.as_str())
        || request_hash != Some(descriptor.request_hash.as_str())
    {
        return false;
    }
    match descriptor.publication_status {
        crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::LedgerPending => {
            false
        }
        crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::StatePublicationPending => true,
        crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => {
            phase == Some("branch_written")
        }
    }
}

fn valid_commit_publication_window(authorities: &OpenSnapshotIdentityAuthorities<'_>) -> bool {
    let Some(transaction) = valid_commit_transaction_for_subject(authorities) else {
        return false;
    };
    if transaction.get("phase").and_then(Value::as_str) != Some("publishing_branch") {
        return false;
    }
    let Some(candidate) = commit_candidate(transaction) else {
        return false;
    };
    let parent = transaction
        .get("parents")
        .and_then(Value::as_array)
        .and_then(|parents| parents.first())
        .and_then(Value::as_str);
    candidate == authorities.branch_head
        && (parent == Some(authorities.registry_current_base)
            || candidate == authorities.registry_current_base)
}

fn valid_commit_state_transition(
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    state: &OpenSnapshotStateAuthorities<'_>,
) -> bool {
    let Some(transaction) = valid_commit_transaction_for_subject(identity) else {
        return false;
    };
    if state.pending_commit_transaction != Some(transaction) {
        return false;
    }
    let phase = transaction.get("phase").and_then(Value::as_str);
    if !matches!(
        phase,
        Some(
            "publishing_branch"
                | "resetting_active_state"
                | "publishing_idempotency"
                | "idempotency_published"
        )
    ) {
        return false;
    }
    let Some(candidate) = commit_candidate(transaction) else {
        return false;
    };
    candidate == identity.branch_head
        && (candidate == identity.registry_current_base
            || (phase == Some("publishing_branch")
                && transaction
                    .get("parents")
                    .and_then(Value::as_array)
                    .and_then(|parents| parents.first())
                    .and_then(Value::as_str)
                    == Some(identity.registry_current_base)))
}

fn valid_commit_transaction_for_subject<'a>(
    authorities: &OpenSnapshotIdentityAuthorities<'a>,
) -> Option<&'a Value> {
    let transaction = authorities.pending_commit_transaction?;
    if crate::validate_commit_transaction_record(transaction).is_err()
        || transaction.get("branch").and_then(Value::as_str) != Some(authorities.branch)
        || transaction
            .get("open_dir")
            .and_then(Value::as_str)
            .map(Path::new)
            != Some(authorities.open_dir)
    {
        return None;
    }
    let phase = transaction.get("phase").and_then(Value::as_str)?;
    let parents = transaction.get("parents").and_then(Value::as_array)?;
    let first_parent = parents.first().and_then(Value::as_str)?;
    let postpublication = matches!(
        phase,
        "publishing_branch"
            | "resetting_active_state"
            | "publishing_idempotency"
            | "idempotency_published"
    );
    if !postpublication {
        return (authorities.branch_head == authorities.registry_current_base
            && first_parent == authorities.branch_head)
            .then_some(transaction);
    }

    let candidate = commit_candidate(transaction)?;
    let objects = authorities.repo_root.join(".codefire").join("objects");
    codefire_store::validate_sealed_commit(&objects, candidate).ok()?;
    let candidate_record = codefire_store::read_object(&objects, candidate).ok()?;
    if candidate_record.get("parents") != Some(&Value::Array(parents.clone())) {
        return None;
    }
    let valid_publication_tuple = if phase == "publishing_branch" {
        authorities.branch_head == candidate
            && (authorities.registry_current_base == candidate
                || authorities.registry_current_base == first_parent)
    } else {
        authorities.branch_head == candidate && authorities.registry_current_base == candidate
    };
    if !valid_publication_tuple {
        return None;
    }
    Some(transaction)
}

fn commit_candidate(transaction: &Value) -> Option<&str> {
    if transaction
        .pointer("/candidate_commit/state")
        .and_then(Value::as_str)
        != Some("stored")
        || !transaction
            .get("objects")
            .and_then(Value::as_array)
            .is_some_and(|objects| {
                !objects.is_empty()
                    && objects
                        .iter()
                        .all(|entry| entry.get("state").and_then(Value::as_str) == Some("stored"))
            })
    {
        return None;
    }
    transaction
        .pointer("/candidate_commit/object_id")
        .and_then(Value::as_str)
}

fn ensure_path_equal(
    authorities: &OpenSnapshotIdentityAuthorities<'_>,
    authority: &'static str,
    expected: &Path,
    actual: &Path,
    expected_record: &Path,
    actual_record: &Path,
) -> Result<(), Box<OpenSnapshotDivergence>> {
    if expected == actual {
        return Ok(());
    }
    Err(Box::new(divergence(
        authorities,
        authority,
        &expected.to_string_lossy(),
        &actual.to_string_lossy(),
        expected_record,
        actual_record,
    )))
}

fn ensure_equal(
    authorities: &OpenSnapshotIdentityAuthorities<'_>,
    authority: &'static str,
    expected: &str,
    actual: &str,
    expected_record: &Path,
    actual_record: &Path,
) -> Result<(), Box<OpenSnapshotDivergence>> {
    if expected == actual {
        return Ok(());
    }
    Err(Box::new(divergence(
        authorities,
        authority,
        expected,
        actual,
        expected_record,
        actual_record,
    )))
}

fn divergence(
    authorities: &OpenSnapshotIdentityAuthorities<'_>,
    authority: &'static str,
    expected: &str,
    actual: &str,
    expected_record: &Path,
    actual_record: &Path,
) -> OpenSnapshotDivergence {
    OpenSnapshotDivergence {
        phase: "steady".to_string(),
        authority,
        expected: expected.to_string(),
        actual: actual.to_string(),
        expected_record: expected_record.to_path_buf(),
        actual_record: actual_record.to_path_buf(),
        repo_root: authorities.repo_root.to_path_buf(),
        branch: authorities.branch.to_string(),
    }
}

fn active_ledger_divergence(
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    state: &OpenSnapshotStateAuthorities<'_>,
    phase: &str,
    expected: &str,
    actual: &str,
) -> OpenSnapshotDivergence {
    let actual_record = state
        .active_state_record_path
        .parent()
        .unwrap_or(state.active_state_record_path)
        .join(crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE);
    OpenSnapshotDivergence {
        phase: phase.to_string(),
        authority: "pending_active_ledger_transaction",
        expected: expected.to_string(),
        actual: actual.to_string(),
        expected_record: identity.registry_path.to_path_buf(),
        actual_record,
        repo_root: identity.repo_root.to_path_buf(),
        branch: identity.branch.to_string(),
    }
}

fn transaction_divergence(
    identity: &OpenSnapshotIdentityAuthorities<'_>,
    state: &OpenSnapshotStateAuthorities<'_>,
    authority: &'static str,
    expected: &str,
    actual: &str,
) -> OpenSnapshotDivergence {
    let transaction = state
        .pending_state_transaction
        .or(state.pending_commit_transaction);
    let phase = transaction
        .and_then(|value| value.get("phase"))
        .and_then(Value::as_str)
        .unwrap_or("pending")
        .to_string();
    let actual_record = state
        .active_state_record_path
        .parent()
        .unwrap_or(state.active_state_record_path)
        .join(if authority == "pending_state_transaction" {
            "state_transaction.json"
        } else {
            "commit_transaction.json"
        });
    OpenSnapshotDivergence {
        phase,
        authority,
        expected: expected.to_string(),
        actual: actual.to_string(),
        expected_record: identity.registry_path.to_path_buf(),
        actual_record,
        repo_root: identity.repo_root.to_path_buf(),
        branch: identity.branch.to_string(),
    }
}
