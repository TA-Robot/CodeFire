use crate::remote::project::{
    prepare_unpublished_remote_project, prepare_unpublished_remote_project_with_ledger,
    remote_project_stable_id, BootstrapStageFootprint, PrelockedRemoteProject, RemoteAuthority,
    RemoteProject, RemoteProjectVisibility, UnpublishedRemoteProject,
    UnpublishedRemoteProjectPublication,
};
use crate::stable_lock::{LockDeadline, StableFileLock, StableLockMode};
use crate::{CliError, LockOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) struct RemoteProjectGate {
    project: RemoteProject,
    mode: StableLockMode,
    lock: StableFileLock,
    visibility: RemoteProjectVisibility,
}

pub(crate) struct PreparedRemoteProjectBootstrap {
    // Field order is a safety contract. The publication owner removes or
    // quarantines the hidden stage while `gate` still holds its exclusive
    // inode lock; its own cleanup field precedes its bootstrap locks.
    publication: UnpublishedRemoteProjectPublication,
    gate: RemoteProjectGate,
}

pub(crate) enum RemoteProjectExclusivePreparation {
    Existing(Box<RemoteProjectGate>),
    Unpublished(Box<PreparedRemoteProjectBootstrap>),
}

pub(crate) struct RemoteProjectGates {
    gates: Vec<RemoteProjectGate>,
}

pub(crate) struct RemoteProjectMergeGates {
    target_root: PathBuf,
    state: RemoteProjectMergeGateState,
}

enum RemoteProjectMergeGateState {
    Existing(RemoteProjectGates),
    UnpublishedTarget {
        source: Box<RemoteProjectGate>,
        target: Box<UnpublishedRemoteProject>,
    },
}

impl RemoteProjectGate {
    pub(crate) fn acquire_shared(
        project_root: &Path,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let deadline = LockDeadline::from_options(options);
        Self::acquire(project_root, StableLockMode::Shared, &deadline)
    }

    pub(crate) fn acquire_exclusive(
        project_root: &Path,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let deadline = LockDeadline::from_options(options);
        Self::acquire(project_root, StableLockMode::Exclusive, &deadline)
    }

    /// Acquire an existing exclusive gate or prepare a missing project without
    /// exposing its layout. A target that appears under the bootstrap locks is
    /// reopened only after those locks have been released, so ordinary project
    /// gate order is never nested beneath bootstrap namespace order.
    pub(crate) fn prepare_exclusive_or_bootstrap(
        project_root: &Path,
        options: &LockOptions,
    ) -> Result<RemoteProjectExclusivePreparation, CliError> {
        let deadline = LockDeadline::from_options(options);
        match Self::acquire(project_root, StableLockMode::Exclusive, &deadline) {
            Ok(gate) => return Ok(RemoteProjectExclusivePreparation::Existing(Box::new(gate))),
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        match prepare_unpublished_remote_project(project_root, &deadline)? {
            Some(unpublished) => Ok(RemoteProjectExclusivePreparation::Unpublished(Box::new(
                PreparedRemoteProjectBootstrap::new(unpublished),
            ))),
            None => {
                // `prepare_unpublished_remote_project` has returned and dropped
                // both bootstrap locks before the ordinary gate is reacquired.
                Self::acquire(project_root, StableLockMode::Exclusive, &deadline)
                    .map(Box::new)
                    .map(RemoteProjectExclusivePreparation::Existing)
            }
        }
    }

    /// Ledger-aware form used by request-wide atomic mutations. Missing-stage
    /// recovery and fixed-layout creation are charged to this exact ledger;
    /// the returned raw owner retains only its opaque identity and permits.
    #[allow(dead_code)] // Production seam for the following HTTP bootstrap slice.
    pub(crate) fn prepare_exclusive_or_bootstrap_with_ledger(
        project_root: &Path,
        options: &LockOptions,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<RemoteProjectExclusivePreparation, CliError> {
        let deadline = LockDeadline::from_options(options);
        match Self::acquire(project_root, StableLockMode::Exclusive, &deadline) {
            Ok(gate) => return Ok(RemoteProjectExclusivePreparation::Existing(Box::new(gate))),
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        match prepare_unpublished_remote_project_with_ledger(project_root, &deadline, ledger)? {
            Some(unpublished) => Ok(RemoteProjectExclusivePreparation::Unpublished(Box::new(
                PreparedRemoteProjectBootstrap::new(unpublished),
            ))),
            None => {
                // The ledger-aware recovery function has returned and dropped
                // bootstrap locks before normal gate order is entered again.
                Self::acquire(project_root, StableLockMode::Exclusive, &deadline)
                    .map(Box::new)
                    .map(RemoteProjectExclusivePreparation::Existing)
            }
        }
    }

    /// Acquire an exclusive gate, atomically bootstrapping a missing project.
    ///
    /// `before_new_project_publish` runs only for a new project and is the last
    /// cancellable point before its layout becomes visible. A `true`
    /// `published_now()` result means the caller must not perform another
    /// deadline check before completing its durable mutation reducer.
    // The returned gate owns the same hidden project and gate capability that
    // crossed publication; callers never reopen the just-published target.
    #[allow(dead_code)]
    pub(crate) fn acquire_exclusive_or_bootstrap<F>(
        project_root: &Path,
        options: &LockOptions,
        before_new_project_publish: F,
    ) -> Result<Self, CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        Self::prepare_exclusive_or_bootstrap(project_root, options)?
            .publish_if_unpublished(before_new_project_publish)
    }

    pub(crate) fn acquire(
        project_root: &Path,
        mode: StableLockMode,
        deadline: &LockDeadline,
    ) -> Result<Self, CliError> {
        let project = RemoteProject::open(project_root)?;
        let lock = project.acquire_gate_lock(mode, deadline)?;
        let gate = Self {
            project,
            mode,
            lock,
            visibility: RemoteProjectVisibility::Existing,
        };
        gate.ensure_project(project_root)?;
        if mode == StableLockMode::Shared {
            gate.ensure_no_pending_transaction()?;
        }
        Ok(gate)
    }

    fn adopt_prevalidated(prelocked: PrelockedRemoteProject) -> Self {
        let (project, lock, visibility) = prelocked.into_parts();
        Self {
            project,
            mode: StableLockMode::Exclusive,
            lock,
            visibility,
        }
    }

    pub(crate) fn ensure_project(&self, project_root: &Path) -> Result<(), CliError> {
        let actual = std::fs::canonicalize(project_root)?;
        if actual != self.project.display_root() {
            return Err(CliError::InvalidRepository(format!(
                "remote project changed while acquiring its gate: expected {}, found {}",
                self.project.display_root().display(),
                actual.display()
            )));
        }
        self.project.ensure_bound()?;
        self.lock.ensure_identity()
    }

    pub(crate) fn project(&self) -> &RemoteProject {
        &self.project
    }

    #[allow(dead_code)]
    pub(crate) fn published_now(&self) -> bool {
        self.visibility == RemoteProjectVisibility::PublishedNow
    }

    pub(crate) fn ensure_exclusive(&self, project_root: &Path) -> Result<(), CliError> {
        self.ensure_project(project_root)?;
        if self.mode != StableLockMode::Exclusive {
            return Err(CliError::InvalidRepository(
                "remote publication requires an exclusive project gate".to_string(),
            ));
        }
        Ok(())
    }

    fn ensure_no_pending_transaction(&self) -> Result<(), CliError> {
        let authority = RemoteAuthority::PendingTransaction;
        let path = authority.path(self.project.display_root())?;
        match self.project.read_authority_bounded(&authority, 64 * 1024) {
            Ok(_) => Err(CliError::LockContention(format!(
                "remote project has a pending publication transaction requiring recovery: {}",
                path.display()
            ))),
            Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl PreparedRemoteProjectBootstrap {
    fn new(unpublished: UnpublishedRemoteProject) -> Self {
        let (prelocked, publication) = unpublished.into_hidden_parts();
        Self {
            publication,
            gate: RemoteProjectGate::adopt_prevalidated(prelocked),
        }
    }

    pub(crate) fn gate(&self) -> &RemoteProjectGate {
        debug_assert_eq!(self.gate.visibility, RemoteProjectVisibility::Unpublished);
        &self.gate
    }

    /// Disjoint borrows avoid a self-referential owner: Store/publication may
    /// borrow the hidden gate while the caller reserves rollback work through
    /// the independent visibility half. Neither capability escapes by value,
    /// so publication cleanup still drops before the hidden gate.
    #[allow(dead_code)] // Production seam for the following HTTP bootstrap slice.
    pub(crate) fn hidden_parts_mut(
        &mut self,
    ) -> (&RemoteProjectGate, &mut UnpublishedRemoteProjectPublication) {
        debug_assert_eq!(self.gate.visibility, RemoteProjectVisibility::Unpublished);
        (&self.gate, &mut self.publication)
    }

    #[allow(dead_code)] // Production seam for the following HTTP bootstrap slice.
    pub(crate) fn reserve_additional_footprint(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
        footprint: BootstrapStageFootprint,
    ) -> Result<(), CliError> {
        self.publication
            .reserve_additional_footprint(ledger, footprint)
    }

    #[allow(dead_code)] // Production seam for the following HTTP bootstrap slice.
    pub(crate) fn arm_with_ledger(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.publication.arm_with_ledger(ledger)
    }

    #[allow(dead_code)] // Production seam for the following HTTP bootstrap slice.
    pub(crate) fn cancel_with_ledger(
        &mut self,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> Result<(), CliError> {
        self.publication.cancel_with_ledger(ledger)
    }

    pub(crate) fn publish<F>(self, before_publish: F) -> Result<RemoteProjectGate, CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        let Self {
            publication,
            mut gate,
        } = self;
        publication.publish(&mut gate.project, &mut gate.lock, before_publish)?;
        gate.visibility = RemoteProjectVisibility::PublishedNow;
        Ok(gate)
    }
}

impl RemoteProjectExclusivePreparation {
    pub(crate) fn gate(&self) -> &RemoteProjectGate {
        match self {
            Self::Existing(gate) => gate,
            Self::Unpublished(prepared) => prepared.gate(),
        }
    }

    pub(crate) fn is_unpublished(&self) -> bool {
        matches!(self, Self::Unpublished(_))
    }

    pub(crate) fn publish_if_unpublished<F>(
        self,
        before_publish: F,
    ) -> Result<RemoteProjectGate, CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        debug_assert_eq!(self.gate().mode, StableLockMode::Exclusive);
        let was_unpublished = self.is_unpublished();
        match self {
            Self::Existing(gate) => {
                debug_assert!(!was_unpublished);
                Ok(*gate)
            }
            Self::Unpublished(prepared) => {
                debug_assert!(was_unpublished);
                (*prepared).publish(before_publish)
            }
        }
    }
}

impl std::fmt::Debug for RemoteProjectGate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteProjectGate")
            .field("project_root", &self.project.display_root())
            .field("stable_id", &self.project.stable_id())
            .field("mode", &self.mode)
            .field("visibility", &self.visibility)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for RemoteProjectGates {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteProjectGates")
            .field("gate_count", &self.gates.len())
            .finish()
    }
}

impl RemoteProjectGates {
    pub(crate) fn acquire_for_merge(
        source_root: &Path,
        target_root: &Path,
        options: &LockOptions,
    ) -> Result<Self, CliError> {
        let deadline = LockDeadline::from_options(options);
        Self::acquire_for_merge_with_deadline(source_root, target_root, &deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn acquire_for_merge_or_bootstrap_target(
        source_root: &Path,
        target_root: &Path,
        options: &LockOptions,
    ) -> Result<RemoteProjectMergeGates, CliError> {
        let deadline = LockDeadline::from_options(options);
        match prepare_unpublished_remote_project(target_root, &deadline)? {
            Some(target) => {
                // Missing-target requests already own the global bootstrap namespace
                // and an unreachable hidden-target gate.  The source is therefore the
                // only externally reachable stable gate acquired in this branch;
                // opposite missing-target requests serialize on that namespace before
                // either can wait for a source gate.
                let source =
                    RemoteProjectGate::acquire(source_root, StableLockMode::Shared, &deadline)?;
                Ok(RemoteProjectMergeGates {
                    target_root: target_root.to_path_buf(),
                    state: RemoteProjectMergeGateState::UnpublishedTarget {
                        source: Box::new(source),
                        target: Box::new(target),
                    },
                })
            }
            None => Ok(RemoteProjectMergeGates {
                target_root: target_root.to_path_buf(),
                state: RemoteProjectMergeGateState::Existing(
                    Self::acquire_for_merge_with_deadline(source_root, target_root, &deadline)?,
                ),
            }),
        }
    }

    fn acquire_for_merge_with_deadline(
        source_root: &Path,
        target_root: &Path,
        deadline: &LockDeadline,
    ) -> Result<Self, CliError> {
        let mut requested = BTreeMap::<String, (PathBuf, StableLockMode)>::new();
        for (root, mode) in [
            (source_root, StableLockMode::Shared),
            (target_root, StableLockMode::Exclusive),
        ] {
            let stable_id = remote_project_stable_id(root)?;
            let canonical_root = std::fs::canonicalize(root)?;
            if let Some((existing_root, existing_mode)) = requested.get_mut(&stable_id) {
                if *existing_root != canonical_root {
                    return Err(CliError::InvalidRepository(format!(
                        "distinct remote project roots claim the same stable identity {stable_id}: {} and {}",
                        existing_root.display(),
                        canonical_root.display()
                    )));
                }
                if mode == StableLockMode::Exclusive {
                    *existing_mode = StableLockMode::Exclusive;
                }
            } else {
                requested.insert(stable_id, (canonical_root, mode));
            }
        }
        let mut gates = Vec::with_capacity(requested.len());
        for (expected_id, (root, mode)) in requested {
            let gate = RemoteProjectGate::acquire(&root, mode, deadline)?;
            let actual_id = gate.project().stable_id().to_string();
            if actual_id != expected_id {
                return Err(CliError::InvalidRepository(format!(
                    "remote project identity changed while acquiring ordered gates: expected {expected_id}, found {actual_id}"
                )));
            }
            gates.push(gate);
        }
        Ok(Self { gates })
    }

    pub(crate) fn project_for(&self, project_root: &Path) -> Result<&RemoteProject, CliError> {
        let canonical = std::fs::canonicalize(project_root)?;
        self.gates
            .iter()
            .find(|gate| gate.project().display_root() == canonical)
            .map(RemoteProjectGate::project)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "ordered remote gates do not contain {}",
                    project_root.display()
                ))
            })
    }

    pub(crate) fn exclusive_for(
        &self,
        project_root: &Path,
    ) -> Result<&RemoteProjectGate, CliError> {
        let canonical = std::fs::canonicalize(project_root)?;
        let gate = self
            .gates
            .iter()
            .find(|gate| gate.project().display_root() == canonical)
            .ok_or_else(|| {
                CliError::InvalidRepository(format!(
                    "ordered remote gates do not contain {}",
                    project_root.display()
                ))
            })?;
        gate.ensure_exclusive(project_root)?;
        Ok(gate)
    }
}

impl RemoteProjectMergeGates {
    #[allow(dead_code)]
    pub(crate) fn target_is_unpublished(&self) -> bool {
        matches!(
            &self.state,
            RemoteProjectMergeGateState::UnpublishedTarget { .. }
        )
    }

    #[allow(dead_code)]
    pub(crate) fn target_project(&self) -> Result<&RemoteProject, CliError> {
        match &self.state {
            RemoteProjectMergeGateState::Existing(gates) => gates.project_for(&self.target_root),
            RemoteProjectMergeGateState::UnpublishedTarget { target, .. } => Ok(target.project()),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn publish_target<F>(self, before_publish: F) -> Result<RemoteProjectGates, CliError>
    where
        F: FnOnce() -> Result<(), CliError>,
    {
        match self.state {
            RemoteProjectMergeGateState::Existing(gates) => Ok(gates),
            RemoteProjectMergeGateState::UnpublishedTarget { source, target } => {
                let prelocked = target.publish(before_publish)?;
                let target = RemoteProjectGate::adopt_prevalidated(prelocked);
                let mut gates = vec![*source, target];
                gates.sort_by(|left, right| {
                    left.project().stable_id().cmp(right.project().stable_id())
                });
                Ok(RemoteProjectGates { gates })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::project::ensure_remote_project_layout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Barrier};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn projects(label: &str) -> (PathBuf, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-{label}-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let first = root.join("projects/org/first");
        let second = root.join("projects/org/second");
        ensure_remote_project_layout(&first).unwrap();
        ensure_remote_project_layout(&second).unwrap();
        (root, first, second)
    }

    fn bootstrap_ledger(label: &str) -> codefire_core::DecisionWorkLedger {
        codefire_core::DecisionWorkLedger::new(
            codefire_core::DecisionWorkProfile::http_v1(),
            codefire_core::DecisionWorkCommandKind::HttpRequest,
            format!("bootstrap-{label}"),
        )
        .unwrap()
    }

    fn ledger_bootstrap(
        project: &Path,
        ledger: &mut codefire_core::DecisionWorkLedger,
    ) -> PreparedRemoteProjectBootstrap {
        match RemoteProjectGate::prepare_exclusive_or_bootstrap_with_ledger(
            project,
            &LockOptions::default(),
            ledger,
        )
        .unwrap()
        {
            RemoteProjectExclusivePreparation::Unpublished(prepared) => *prepared,
            RemoteProjectExclusivePreparation::Existing(_) => {
                panic!("ledger bootstrap fixture unexpectedly found an existing project")
            }
        }
    }

    fn assert_bootstrap_ledger_released(ledger: &codefire_core::DecisionWorkLedger) {
        assert!(!ledger.has_open_permits());
        assert!(ledger
            .snapshot()
            .current
            .values()
            .all(|amount| *amount == 0));
    }

    #[test]
    fn same_project_merge_collapses_to_one_exclusive_gate() {
        let (root, project, _) = projects("collapse");
        RemoteProjectGates::acquire_for_merge(&project, &project, &LockOptions::default()).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_reader_preserves_and_rejects_pending_transaction() {
        let (root, project, _) = projects("pending");
        let pending = project.join("pending_transaction.json");
        fs::write(&pending, b"{\"type\":\"remote_publication_transaction\"}\n").unwrap();
        let error =
            RemoteProjectGate::acquire_shared(&project, &LockOptions::default()).unwrap_err();
        assert!(matches!(error, CliError::LockContention(_)));
        assert!(pending.is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn opposite_cross_project_requests_share_one_stable_order() {
        let (root, first, second) = projects("order");
        let barrier = Arc::new(Barrier::new(3));
        let options = LockOptions {
            wait: true,
            timeout_ms: Some(2_000),
        };
        let first_thread = {
            let barrier = Arc::clone(&barrier);
            let source = first.clone();
            let target = second.clone();
            std::thread::spawn(move || {
                barrier.wait();
                RemoteProjectGates::acquire_for_merge(&source, &target, &options).map(drop)
            })
        };
        let second_thread = {
            let barrier = Arc::clone(&barrier);
            let source = second.clone();
            let target = first.clone();
            std::thread::spawn(move || {
                barrier.wait();
                RemoteProjectGates::acquire_for_merge(&source, &target, &options).map(drop)
            })
        };
        barrier.wait();
        first_thread.join().unwrap().unwrap();
        second_thread.join().unwrap().unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gate_detects_descriptor_and_locks_directory_replacement() {
        let (root, project, _) = projects("replacement");
        let gate = RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap();
        let descriptor = project.join("remote_project.json");
        fs::rename(&descriptor, project.join("remote_project.old.json")).unwrap();
        fs::write(&descriptor, b"{}\n").unwrap();
        assert!(gate.ensure_exclusive(&project).is_err());
        drop(gate);

        fs::remove_file(&descriptor).unwrap();
        fs::rename(project.join("remote_project.old.json"), &descriptor).unwrap();
        let gate = RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap();
        fs::rename(project.join("locks"), project.join("locks-old")).unwrap();
        fs::create_dir(project.join("locks")).unwrap();
        assert!(gate.ensure_exclusive(&project).is_err());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ordered_gates_reject_distinct_roots_with_duplicate_stable_id() {
        let (root, first, second) = projects("duplicate-id");
        let first_descriptor: serde_json::Value =
            serde_json::from_slice(&fs::read(first.join("remote_project.json")).unwrap()).unwrap();
        let mut second_descriptor: serde_json::Value =
            serde_json::from_slice(&fs::read(second.join("remote_project.json")).unwrap()).unwrap();
        second_descriptor["stable_project_id"] = first_descriptor["stable_project_id"].clone();
        fs::write(
            second.join("remote_project.json"),
            codefire_store::canonical_json(&second_descriptor).unwrap(),
        )
        .unwrap();
        let error = RemoteProjectGates::acquire_for_merge(&first, &second, &LockOptions::default())
            .unwrap_err();
        assert!(matches!(error, CliError::InvalidRepository(_)));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gate_detects_project_root_replacement() {
        let (root, project, _) = projects("root-swap");
        let gate = RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap();
        let moved = root.join("projects/org/first-moved");
        fs::rename(&project, &moved).unwrap();
        ensure_remote_project_layout(&project).unwrap();
        assert!(gate.ensure_exclusive(&project).is_err());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_holds_project_gate_before_layout_visibility() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let org_root = root.join("projects/org");
        let gate = RemoteProjectGate::acquire_exclusive_or_bootstrap(
            &project,
            &LockOptions::default(),
            || {
                assert!(!project.exists());
                let stage = fs::read_dir(&org_root)?
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .find(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| name.starts_with(".bootstrap-"))
                    })
                    .expect("bootstrap stage must exist before publication");
                let error = StableFileLock::acquire(
                    &stage.join("locks/project-gate.lock"),
                    StableLockMode::Exclusive,
                    &LockDeadline::from_options(&LockOptions::default()),
                )
                .unwrap_err();
                assert!(matches!(error, CliError::LockContention(_)));
                Ok(())
            },
        )
        .unwrap();
        assert!(gate.published_now());
        assert!(project.join("remote_project.json").is_file());
        let error =
            RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap_err();
        assert!(matches!(error, CliError::LockContention(_)));
        drop(gate);
        RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_preparation_exposes_writable_hidden_gate() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-prepared-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let prepared =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&project, &LockOptions::default())
                .unwrap();
        assert!(matches!(
            &prepared,
            RemoteProjectExclusivePreparation::Unpublished(_)
        ));
        let hidden_root = prepared.gate().project().display_root().to_path_buf();
        prepared
            .gate()
            .project()
            .write_authority_atomic(
                &RemoteAuthority::Generation,
                b"{\"current_generation\":7}\n",
            )
            .unwrap();

        assert_ne!(hidden_root, project);
        assert!(hidden_root.join("gc_state.json").is_file());
        assert!(!project.exists());
        let gate = prepared
            .publish_if_unpublished(|| {
                assert!(!project.exists());
                assert_eq!(
                    fs::read(hidden_root.join("gc_state.json"))?,
                    b"{\"current_generation\":7}\n"
                );
                Ok(())
            })
            .unwrap();

        assert!(gate.published_now());
        assert_eq!(
            fs::read(project.join("gc_state.json")).unwrap(),
            b"{\"current_generation\":7}\n"
        );
        assert!(matches!(
            RemoteProjectGate::acquire_exclusive(&project, &LockOptions::default()).unwrap_err(),
            CliError::LockContention(_)
        ));
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ledger_bootstrap_cancel_consumes_permits_and_releases_current() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-ledger-cancel-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let mut ledger = bootstrap_ledger("cancel");
        let mut prepared = ledger_bootstrap(&project, &mut ledger);
        prepared
            .reserve_additional_footprint(&mut ledger, BootstrapStageFootprint::new(1, 128))
            .unwrap();
        assert_eq!(ledger.open_permit_count(), 2);

        prepared.cancel_with_ledger(&mut ledger).unwrap();

        assert!(!project.exists());
        assert_bootstrap_ledger_released(&ledger);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ledger_bootstrap_rejects_foreign_ledger_without_losing_owner() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-ledger-foreign-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let mut owner = bootstrap_ledger("owner");
        let mut foreign = bootstrap_ledger("foreign");
        let mut prepared = ledger_bootstrap(&project, &mut owner);

        assert!(prepared
            .reserve_additional_footprint(&mut foreign, BootstrapStageFootprint::new(1, 1),)
            .is_err());
        assert!(prepared.arm_with_ledger(&mut foreign).is_err());
        assert_eq!(owner.open_permit_count(), 1);
        assert_eq!(foreign.open_permit_count(), 0);

        prepared.cancel_with_ledger(&mut owner).unwrap();
        assert_bootstrap_ledger_released(&owner);
        assert_bootstrap_ledger_released(&foreign);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ledger_bootstrap_footprint_accepts_limit_minus_one_and_exact_but_rejects_plus_one() {
        // The fixed layout owns 23 entries under the 4,096-entry recovery
        // contract, leaving exactly 4,073 caller-supplied entries.
        for (label, entries, accepted) in [
            ("minus-one", 4_072, true),
            ("exact", 4_073, true),
            ("plus-one", 4_074, false),
        ] {
            let root = std::env::temp_dir().join(format!(
                "codefire-project-gates-bootstrap-ledger-bound-{label}-{}-{}",
                std::process::id(),
                TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let project = root.join("projects/org/app");
            let mut ledger = bootstrap_ledger(label);
            let mut prepared = ledger_bootstrap(&project, &mut ledger);
            let result = prepared.reserve_additional_footprint(
                &mut ledger,
                BootstrapStageFootprint::new(entries, 0),
            );
            assert_eq!(result.is_ok(), accepted, "boundary case {label}");
            prepared.cancel_with_ledger(&mut ledger).unwrap();
            assert_bootstrap_ledger_released(&ledger);
            fs::remove_dir_all(root).unwrap();
        }

        let exact_payload = BootstrapStageFootprint::maximum_payload_bytes(0).unwrap();
        for (label, payload_bytes, accepted) in [
            ("bytes-minus-one", exact_payload - 1, true),
            ("bytes-exact", exact_payload, true),
            ("bytes-plus-one", exact_payload + 1, false),
        ] {
            let root = std::env::temp_dir().join(format!(
                "codefire-project-gates-bootstrap-ledger-bound-{label}-{}-{}",
                std::process::id(),
                TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let project = root.join("projects/org/app");
            let mut ledger = bootstrap_ledger(label);
            let mut prepared = ledger_bootstrap(&project, &mut ledger);
            let result = prepared.reserve_additional_footprint(
                &mut ledger,
                BootstrapStageFootprint::new(0, payload_bytes),
            );
            assert_eq!(result.is_ok(), accepted, "boundary case {label}");
            prepared.cancel_with_ledger(&mut ledger).unwrap();
            assert_bootstrap_ledger_released(&ledger);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn ledger_bootstrap_publish_finishes_with_no_permit_or_current_owner() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-ledger-publish-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let mut ledger = bootstrap_ledger("publish");
        let mut prepared = ledger_bootstrap(&project, &mut ledger);
        prepared
            .reserve_additional_footprint(&mut ledger, BootstrapStageFootprint::new(0, 0))
            .unwrap();
        ledger.seal_prepublication().unwrap();
        prepared.arm_with_ledger(&mut ledger).unwrap();
        assert_bootstrap_ledger_released(&ledger);

        let gate = prepared
            .publish(|| {
                assert!(!project.exists());
                Ok(())
            })
            .unwrap();

        assert!(gate.published_now());
        assert!(project.join("remote_project.json").is_file());
        assert_bootstrap_ledger_released(&ledger);
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ledger_cleanup_failure_quarantines_for_same_ledger_bounded_recovery() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-ledger-recovery-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let org_root = root.join("projects/org");
        let mut ledger = bootstrap_ledger("cleanup-recovery");
        let mut prepared = ledger_bootstrap(&project, &mut ledger);
        let hidden = prepared.gate().project().display_root().to_path_buf();
        prepared
            .hidden_parts_mut()
            .1
            .inject_bounded_cleanup_failure_once();

        let error = prepared.cancel_with_ledger(&mut ledger).unwrap_err();
        assert!(error
            .to_string()
            .contains("injected remote bootstrap bounded cleanup failure"));
        assert!(!hidden.exists());
        assert!(fs::read_dir(&org_root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_str().is_some_and(|name| name
                .starts_with(".bootstrap-")
                && name.ends_with(".removing"))));
        assert_bootstrap_ledger_released(&ledger);

        let replacement = ledger_bootstrap(&project, &mut ledger);
        assert!(!fs::read_dir(&org_root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.ends_with(".removing"))));
        assert!(
            ledger.snapshot().counters[&codefire_core::DecisionWorkCounter::UniqueNodesDiscovered]
                > 0
        );
        replacement.cancel_with_ledger(&mut ledger).unwrap();
        assert_bootstrap_ledger_released(&ledger);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_bootstrap_callback_failure_cleans_hidden_stage() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-prepared-cancel-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let prepared =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&project, &LockOptions::default())
                .unwrap();
        let hidden_root = prepared.gate().project().display_root().to_path_buf();

        let error = prepared
            .publish_if_unpublished(|| {
                Err(CliError::RequestDeadlineExceeded {
                    phase: "prepared bootstrap publication test",
                    limit_ms: 1,
                })
            })
            .unwrap_err();

        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert!(!project.exists());
        assert!(!hidden_root.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dropping_prepared_bootstrap_cleans_before_replanning() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-prepared-drop-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let prepared =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&project, &LockOptions::default())
                .unwrap();
        let hidden_root = prepared.gate().project().display_root().to_path_buf();
        let hidden_gate_path = hidden_root.join("locks/project-gate.lock");
        assert!(matches!(
            StableFileLock::acquire(
                &hidden_gate_path,
                StableLockMode::Exclusive,
                &LockDeadline::from_options(&LockOptions::default()),
            )
            .unwrap_err(),
            CliError::LockContention(_)
        ));

        drop(prepared);

        assert!(!project.exists());
        assert!(!hidden_root.exists());
        let replacement =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&project, &LockOptions::default())
                .unwrap();
        assert!(matches!(
            &replacement,
            RemoteProjectExclusivePreparation::Unpublished(_)
        ));
        drop(replacement);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_bootstrap_loser_replans_as_existing() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-prepared-race-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let winner =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&project, &LockOptions::default())
                .unwrap();
        assert!(matches!(
            &winner,
            RemoteProjectExclusivePreparation::Unpublished(_)
        ));

        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let contender_project = project.clone();
        let contender = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            RemoteProjectGate::prepare_exclusive_or_bootstrap(
                &contender_project,
                &LockOptions {
                    wait: true,
                    timeout_ms: Some(2_000),
                },
            )
        });
        started_rx.recv().unwrap();

        let winner_gate = winner.publish_if_unpublished(|| Ok(())).unwrap();
        drop(winner_gate);
        let replanned = contender.join().unwrap().unwrap();

        assert!(matches!(
            &replanned,
            RemoteProjectExclusivePreparation::Existing(_)
        ));
        drop(replanned);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn distinct_projects_roots_do_not_share_bootstrap_namespace() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-isolated-roots-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let first = root.join("first/projects/org/app");
        let second = root.join("second/projects/org/app");
        let first_prepared =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&first, &LockOptions::default())
                .unwrap();

        let second_prepared =
            RemoteProjectGate::prepare_exclusive_or_bootstrap(&second, &LockOptions::default())
                .unwrap();

        assert!(first_prepared.is_unpublished());
        assert!(second_prepared.is_unpublished());
        assert!(!first.exists());
        assert!(!second.exists());
        drop(second_prepared);
        drop(first_prepared);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_prepublication_failure_keeps_project_absent_and_cleans_stage() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-cancel-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let project = root.join("projects/org/app");
        let error = RemoteProjectGate::acquire_exclusive_or_bootstrap(
            &project,
            &LockOptions::default(),
            || {
                Err(CliError::RequestDeadlineExceeded {
                    phase: "bootstrap publication test",
                    limit_ms: 1,
                })
            },
        )
        .unwrap_err();
        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert!(!project.exists());
        let stages = fs::read_dir(root.join("projects/org"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".bootstrap-"))
            })
            .count();
        assert_eq!(stages, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_existing_project_does_not_run_publication_callback() {
        let (root, project, _) = projects("existing-bootstrap");
        let gate = RemoteProjectGate::acquire_exclusive_or_bootstrap(
            &project,
            &LockOptions::default(),
            || panic!("existing project must not cross a layout visibility point"),
        )
        .unwrap();
        assert!(!gate.published_now());
        drop(gate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn merge_bootstrap_cancel_keeps_target_hidden_and_releases_source() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-merge-cancel-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let source = root.join("projects/org/source");
        let target = root.join("projects/org/target");
        ensure_remote_project_layout(&source).unwrap();
        let prepared = RemoteProjectGates::acquire_for_merge_or_bootstrap_target(
            &source,
            &target,
            &LockOptions::default(),
        )
        .unwrap();
        assert!(prepared.target_is_unpublished());
        assert!(!target.exists());
        assert!(prepared.target_project().unwrap().display_root().is_dir());
        assert!(RemoteProjectGate::acquire_exclusive(&source, &LockOptions::default()).is_err());
        let error = prepared
            .publish_target(|| {
                Err(CliError::RequestDeadlineExceeded {
                    phase: "merge target publication test",
                    limit_ms: 1,
                })
            })
            .unwrap_err();
        assert!(matches!(error, CliError::RequestDeadlineExceeded { .. }));
        assert!(!target.exists());
        assert_eq!(
            fs::read_dir(root.join("projects/org"))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.starts_with(".bootstrap-"))
                })
                .count(),
            0
        );
        RemoteProjectGate::acquire_exclusive(&source, &LockOptions::default()).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn merge_bootstrap_preserves_concurrent_target_and_cleans_owned_stage() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-merge-race-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let source = root.join("projects/org/source");
        let target = root.join("projects/org/target");
        ensure_remote_project_layout(&source).unwrap();
        let prepared = RemoteProjectGates::acquire_for_merge_or_bootstrap_target(
            &source,
            &target,
            &LockOptions::default(),
        )
        .unwrap();
        let foreign = target.join("foreign-marker");
        let error = prepared
            .publish_target(|| {
                fs::create_dir(&target)?;
                fs::write(&foreign, b"preserve\n")?;
                Ok(())
            })
            .unwrap_err();
        assert!(
            matches!(error, CliError::Io(ref error) if error.kind() == std::io::ErrorKind::AlreadyExists)
        );
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve\n");
        assert_eq!(
            fs::read_dir(root.join("projects/org"))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.starts_with(".bootstrap-"))
                })
                .count(),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn merge_bootstrap_publishes_while_source_and_target_gates_remain_held() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-merge-publish-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let source = root.join("projects/org/source");
        let target = root.join("projects/org/target");
        ensure_remote_project_layout(&source).unwrap();
        let prepared = RemoteProjectGates::acquire_for_merge_or_bootstrap_target(
            &source,
            &target,
            &LockOptions::default(),
        )
        .unwrap();
        prepared
            .target_project()
            .unwrap()
            .write_authority_atomic(
                &RemoteAuthority::Generation,
                b"{\"current_generation\":0}\n",
            )
            .unwrap();
        assert!(!target.exists());
        assert!(RemoteProjectGate::acquire_exclusive(&source, &LockOptions::default()).is_err());
        let gates = prepared.publish_target(|| Ok(())).unwrap();
        gates.exclusive_for(&target).unwrap();
        assert_eq!(
            fs::read(target.join("gc_state.json")).unwrap(),
            b"{\"current_generation\":0}\n"
        );
        assert!(RemoteProjectGate::acquire_exclusive(&source, &LockOptions::default()).is_err());
        assert!(RemoteProjectGate::acquire_exclusive(&target, &LockOptions::default()).is_err());
        drop(gates);
        RemoteProjectGate::acquire_exclusive(&source, &LockOptions::default()).unwrap();
        RemoteProjectGate::acquire_exclusive(&target, &LockOptions::default()).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_target_namespace_order_unwinds_without_opposite_request_deadlock() {
        let root = std::env::temp_dir().join(format!(
            "codefire-project-gates-bootstrap-order-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let first_source = root.join("projects/org/first-source");
        let second_source = root.join("projects/org/second-source");
        let first_target = root.join("projects/org/first-target");
        let second_target = root.join("projects/org/second-target");
        ensure_remote_project_layout(&first_source).unwrap();
        ensure_remote_project_layout(&second_source).unwrap();
        let held =
            RemoteProjectGate::acquire_exclusive(&first_source, &LockOptions::default()).unwrap();

        let error = match RemoteProjectGates::acquire_for_merge_or_bootstrap_target(
            &first_source,
            &first_target,
            &LockOptions::default(),
        ) {
            Ok(_) => panic!("held source gate must reject missing-target preparation"),
            Err(error) => error,
        };
        assert!(matches!(error, CliError::LockContention(_)));
        assert!(!first_target.exists());
        drop(held);

        let opposite = RemoteProjectGates::acquire_for_merge_or_bootstrap_target(
            &second_source,
            &second_target,
            &LockOptions::default(),
        )
        .unwrap();
        assert!(opposite.target_is_unpublished());
        drop(opposite);
        assert!(!second_target.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
