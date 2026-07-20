use super::{
    DoctorIssue, DoctorPendingAuthority, DoctorPendingAuthorityKind, DoctorPendingOperation,
    DoctorPendingRequestIdentity,
};
use crate::close_transaction::{close_transaction_path, doctor_close_transaction_summary};
use crate::open_transaction::{doctor_open_transaction_summary, open_transaction_path};
use crate::repo_layout::{object_subdir_requirement, LayoutRequirement, REPO_DIRS};
use crate::CliError;
use cap_fs_ext::DirExt;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const ACTIVE_CONTROL_FILE_BYTE_LIMIT: u64 = 1024 * 1024;
const ACTIVE_DATA_FILE_BYTE_LIMIT: u64 = codefire_store::MAX_OBJECT_RECORD_BYTES;
const MAX_DOCTOR_IDENTITY_BYTES: usize = 256;

const ACTIVE_STATE_FILES: &[&str] = &[
    "state.json",
    "fires.json",
    "resolutions.json",
    "scan.json",
    "verification.json",
    "commit_transaction.json",
    "state_transaction.json",
    crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE,
];

#[derive(Debug)]
pub(super) struct DoctorChecks {
    pub(super) objects: usize,
    pub(super) branches: usize,
    pub(super) opened: usize,
    pub(super) active_files: usize,
    pub(super) skipped_checks: Vec<String>,
    pub(super) issues: Vec<DoctorIssue>,
}

pub(super) fn run_checks(cf: &Path, quick: bool) -> Result<DoctorChecks, CliError> {
    run_checks_with_sealed_validation_hook(cf, quick, || Ok(()))
}

fn run_checks_with_sealed_validation_hook<F>(
    cf: &Path,
    quick: bool,
    before_finalize: F,
) -> Result<DoctorChecks, CliError>
where
    F: FnOnce() -> Result<(), CliError>,
{
    let mut issues = Vec::new();
    check_repo_layout(cf, &mut issues);
    let mut skipped_checks = Vec::new();
    let objects = if quick {
        skipped_checks.push("object_store_integrity".to_string());
        0
    } else {
        check_object_records(&cf.join("objects"), &mut issues)?
    };
    check_open_transaction(cf, &mut issues);
    check_close_transaction(cf, &mut issues);
    check_transaction_conflict(cf, &mut issues);
    let mut sealed_validation = DoctorSealedValidation::new(&cf.join("objects"));
    check_branch_heads(cf, &mut issues, &mut sealed_validation)?;
    let opened = check_opened_registry(cf, &mut issues, &mut sealed_validation)?;
    before_finalize()?;
    let validated_branches = sealed_validation.finish(&mut issues);
    Ok(DoctorChecks {
        objects,
        branches: validated_branches,
        opened,
        active_files: check_active_state_files(cf, &mut issues)?,
        skipped_checks,
        issues,
    })
}

fn check_transaction_conflict(cf: &Path, issues: &mut Vec<DoctorIssue>) {
    let open = cf.join("open_transaction.json");
    let close = cf.join("close_transaction.json");
    if fs::symlink_metadata(&open).is_ok() && fs::symlink_metadata(&close).is_ok() {
        issues.push(DoctorIssue::error(
            "conflicting_repository_transactions",
            "simultaneous pending open and close transaction authorities require manual inspection before either request can recover",
            Some(cf.to_path_buf()),
        ));
    }
}

fn check_close_transaction(cf: &Path, issues: &mut Vec<DoctorIssue>) {
    let Some(repo_root) = cf.parent() else {
        return;
    };
    let path = close_transaction_path(repo_root);
    match doctor_close_transaction_summary(repo_root) {
        Ok(Some(summary)) => issues.push(DoctorIssue::error(
            "pending_close_transaction",
            summary,
            Some(path),
        )),
        Ok(None) => {}
        Err(error) => issues.push(DoctorIssue::error(
            "invalid_close_transaction",
            error.to_string(),
            Some(path),
        )),
    }
}

fn check_open_transaction(cf: &Path, issues: &mut Vec<DoctorIssue>) {
    let Some(repo_root) = cf.parent() else {
        return;
    };
    let path = open_transaction_path(repo_root);
    match doctor_open_transaction_summary(repo_root) {
        Ok(Some(summary)) => issues.push(DoctorIssue::error(
            "pending_open_transaction",
            summary,
            Some(path),
        )),
        Ok(None) => {}
        Err(error) => issues.push(DoctorIssue::error(
            "invalid_open_transaction",
            error.to_string(),
            Some(path),
        )),
    }
}

fn check_repo_layout(cf: &Path, issues: &mut Vec<DoctorIssue>) {
    if !cf.is_dir() {
        issues.push(DoctorIssue::error(
            "missing_codefire_dir",
            "missing .codefire directory",
            Some(cf.to_path_buf()),
        ));
        return;
    }
    for entry in REPO_DIRS {
        let path = cf.join(entry.relative);
        if !path.is_dir() {
            let message = format!("missing .codefire/{} directory", entry.relative);
            match entry.requirement {
                LayoutRequirement::Required => issues.push(DoctorIssue::error(
                    "missing_repo_directory",
                    message,
                    Some(path),
                )),
                LayoutRequirement::AutoCreate => issues.push(DoctorIssue::warning(
                    "missing_repairable_directory",
                    message,
                    Some(path),
                )),
            }
        }
    }
    let objects = cf.join("objects");
    for subdir in codefire_store::known_object_subdirs() {
        let path = objects.join(subdir);
        if !path.is_dir() {
            let message = format!("missing .codefire/objects/{subdir} directory");
            match object_subdir_requirement(subdir) {
                LayoutRequirement::Required => issues.push(DoctorIssue::error(
                    "missing_object_subdirectory",
                    message,
                    Some(path),
                )),
                LayoutRequirement::AutoCreate => issues.push(DoctorIssue::warning(
                    "missing_repairable_object_subdirectory",
                    message,
                    Some(path),
                )),
            }
        }
    }
}

fn check_object_records(objects: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    if !objects.exists() {
        return Ok(0);
    }
    let mut checked = 0usize;
    codefire_store::scan_object_store_record_documents(objects, |entry| {
        let path = entry.path;
        let record = match entry.document {
            Ok(document) => document.record,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_object_record",
                    error.to_string(),
                    Some(path),
                ));
                return;
            }
        };
        match codefire_store::validate_object_record(&record, None, Some(&path)) {
            Ok(()) => checked += 1,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "object_integrity_error",
                    error.to_string(),
                    Some(path.clone()),
                ));
                return;
            }
        }
        if let Some(expected_subdir) = codefire_store::object_subdir(&record.type_tag) {
            if path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                != Some(expected_subdir)
            {
                issues.push(DoctorIssue::error(
                    "object_type_directory_mismatch",
                    format!(
                        "{} object should be stored under objects/{expected_subdir}",
                        record.type_tag
                    ),
                    Some(path),
                ));
            }
        } else {
            issues.push(DoctorIssue::error(
                "unknown_object_type",
                format!("unknown object type: {}", record.type_tag),
                Some(path),
            ));
        }
    })?;
    if checked == 0 {
        issues.push(DoctorIssue::warning(
            "empty_object_store",
            "object store has no valid object records",
            Some(objects.to_path_buf()),
        ));
    }
    Ok(checked)
}

fn check_branch_heads(
    cf: &Path,
    issues: &mut Vec<DoctorIssue>,
    sealed_validation: &mut DoctorSealedValidation,
) -> Result<(), CliError> {
    let branches = cf.join("branches");
    if !branches.exists() {
        return Ok(());
    }
    for path in collect_json_files(&branches)? {
        let branch = match read_doctor_json(&path, ACTIVE_DATA_FILE_BYTE_LIMIT) {
            Ok(branch) => branch,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_branch_record",
                    error.to_string(),
                    Some(path),
                ));
                continue;
            }
        };
        let head = match branch.get("head").and_then(Value::as_str) {
            Some(head) => head,
            None => {
                issues.push(DoctorIssue::error(
                    "missing_branch_head",
                    "branch record must contain head",
                    Some(path),
                ));
                continue;
            }
        };
        sealed_validation.validate_branch_head(head, path, issues);
    }
    Ok(())
}

fn check_opened_registry(
    cf: &Path,
    issues: &mut Vec<DoctorIssue>,
    sealed_validation: &mut DoctorSealedValidation,
) -> Result<usize, CliError> {
    let opened = cf.join("opened");
    if !opened.exists() {
        return Ok(0);
    }
    let mut checked = 0usize;
    for path in collect_json_files(&opened)? {
        let registry = match read_doctor_json(&path, ACTIVE_DATA_FILE_BYTE_LIMIT) {
            Ok(registry) => registry,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_open_registry",
                    error.to_string(),
                    Some(path),
                ));
                continue;
            }
        };
        let branch_name = match registry
            .get("branch")
            .and_then(|branch| branch.get("name"))
            .and_then(Value::as_str)
        {
            Some(branch) => Some(branch.to_string()),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_branch_name",
                    "opened registry must contain branch.name",
                    Some(path.clone()),
                ));
                None
            }
        };
        let open_instance_id = match registry
            .get("open")
            .and_then(|open| open.get("open_instance_id"))
            .and_then(Value::as_str)
        {
            Some(open_instance_id) => Some(open_instance_id.to_string()),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_instance_id",
                    "opened registry must contain open.open_instance_id",
                    Some(path.clone()),
                ));
                None
            }
        };
        let open_path = match registry
            .get("open")
            .and_then(|open| open.get("path"))
            .and_then(Value::as_str)
        {
            Some(open_path) => Some(PathBuf::from(open_path)),
            None => {
                issues.push(DoctorIssue::error(
                    "missing_open_path",
                    "opened registry must contain open.path",
                    Some(path.clone()),
                ));
                None
            }
        };
        match registry
            .get("open")
            .and_then(|open| open.get("current_base_commit"))
            .and_then(Value::as_str)
        {
            Some(base) => {
                sealed_validation.validate_open_base(base, path.clone(), open_path.clone(), issues)
            }
            None => issues.push(DoctorIssue::error(
                "missing_open_base_commit",
                "opened registry must contain open.current_base_commit",
                Some(path.clone()),
            )),
        }
        if let Some(open_path) = open_path.as_deref() {
            if !open_path.is_dir() {
                issues.push(DoctorIssue::error(
                    "missing_open_path_dir",
                    format!(
                        "opened registry path does not exist: {}",
                        open_path.display()
                    ),
                    Some(path.clone()),
                ));
            } else {
                check_open_marker(
                    cf,
                    &path,
                    &registry,
                    open_path,
                    branch_name.as_deref(),
                    open_instance_id.as_deref(),
                    issues,
                );
            }
        }
        match registry
            .get("open")
            .and_then(|open| open.get("active_state_path"))
            .and_then(Value::as_str)
        {
            Some(active_path) => {
                match crate::active_state::ActiveStateDir::bind(Path::new(active_path)) {
                    Ok(active_state) => {
                        checked += 1;
                        check_verified_open_snapshot(cf, &path, &registry, &active_state, issues);
                    }
                    Err(error) if error.is_not_found() => issues.push(DoctorIssue::error(
                        "missing_active_state_dir",
                        format!("active state directory does not exist: {active_path}"),
                        Some(path),
                    )),
                    Err(error) => issues.push(DoctorIssue::error(
                        "invalid_active_state_dir",
                        error.to_string(),
                        Some(path),
                    )),
                }
            }
            None => issues.push(DoctorIssue::error(
                "missing_active_state_path",
                "opened registry must contain open.active_state_path",
                Some(path),
            )),
        }
    }
    Ok(checked)
}

fn check_verified_open_snapshot(
    cf: &Path,
    registry_path: &Path,
    registry: &Value,
    active_state: &crate::active_state::ActiveStateDir,
    issues: &mut Vec<DoctorIssue>,
) {
    let Some(repo_root) = cf.parent() else {
        return;
    };
    let repository_record_path = cf.join("repo.json");
    let Some(branch_name) = value_string(registry, &["branch", "name"]) else {
        return;
    };
    let branch_record_path = cf
        .join("branches")
        .join(format!("{}.json", crate::ref_file_name(&branch_name)));
    let Some(open_path) = path_string(registry, &["open", "path"]) else {
        return;
    };
    let marker_path = open_path.join(".codefire-open");
    let (repository, branch, marker, active_state_record) = match (
        read_doctor_json(&repository_record_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT),
        read_doctor_json(&branch_record_path, ACTIVE_DATA_FILE_BYTE_LIMIT),
        read_doctor_json(&marker_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT),
        read_active_state_json(active_state, Path::new("state.json"), "state.json"),
    ) {
        (Ok(repository), Ok(branch), Ok(marker), Ok(active_state_record)) => {
            (repository, branch, marker, active_state_record)
        }
        _ => return,
    };
    let Some(open_instance_id) = value_string(registry, &["open", "open_instance_id"]) else {
        return;
    };
    let expected_active_state_path = cf
        .join("active")
        .join(crate::ref_file_name(&open_instance_id));
    let pending_commit_transaction = read_optional_active_transaction(
        active_state,
        "commit_transaction.json",
        crate::COMMIT_TRANSACTION_BYTE_LIMIT,
    );
    let pending_state_transaction = read_optional_active_transaction(
        active_state,
        "state_transaction.json",
        crate::STATE_TRANSACTION_BYTE_LIMIT,
    );

    let values = (
        value_string(&repository, &["repository_id"]),
        path_string(&marker, &["repository", "path"]),
        value_string(&marker, &["repository", "repository_id"]),
        value_string(&marker, &["branch", "name"]),
        value_string(&branch, &["name"]),
        path_string(&marker, &["open", "opened_path"]),
        value_string(&marker, &["open", "open_instance_id"]),
        value_string(&marker, &["branch", "opened_from_commit"]),
        value_string(registry, &["open", "opened_from_commit"]),
        path_string(registry, &["open", "active_state_path"]),
        value_string(&branch, &["head"]),
        value_string(registry, &["open", "current_base_commit"]),
        value_string(&branch, &["state"]),
        value_string(registry, &["state", "last_known"]),
        value_string(&active_state_record, &["state"]),
    );
    let (
        Some(repository_id),
        Some(marker_repository_path),
        Some(marker_repository_id),
        Some(marker_branch),
        Some(branch_record_name),
        Some(marker_open_path),
        Some(marker_open_instance_id),
        Some(marker_opened_from_commit),
        Some(registry_opened_from_commit),
        Some(registry_active_state_path),
        Some(branch_head),
        Some(registry_current_base),
        Some(branch_state),
        Some(registry_state),
        Some(active_state_value),
    ) = values
    else {
        return;
    };

    let active_state_record_path = active_state.display_path(Path::new("state.json"));
    let active_ledger_descriptor =
        crate::fire::active_ledger_transaction::validate_active_ledger_descriptor(active_state)
            .ok()
            .flatten();
    let identity = crate::open_snapshot::OpenSnapshotIdentityAuthorities {
        repo_root,
        repository_record_path: &repository_record_path,
        marker_path: &marker_path,
        branch_record_path: &branch_record_path,
        registry_path,
        expected_repository_path: cf,
        marker_repository_path: &marker_repository_path,
        repository_id: &repository_id,
        marker_repository_id: &marker_repository_id,
        branch: &branch_name,
        marker_branch: &marker_branch,
        registry_branch: &branch_name,
        branch_record_name: &branch_record_name,
        open_dir: &open_path,
        marker_open_path: &marker_open_path,
        registry_open_path: &open_path,
        marker_open_instance_id: &marker_open_instance_id,
        registry_open_instance_id: &open_instance_id,
        marker_opened_from_commit: &marker_opened_from_commit,
        registry_opened_from_commit: &registry_opened_from_commit,
        expected_active_state_path: &expected_active_state_path,
        registry_active_state_path: &registry_active_state_path,
        branch_head: &branch_head,
        registry_current_base: &registry_current_base,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
    };
    let state = crate::open_snapshot::OpenSnapshotStateAuthorities {
        active_state_record_path: &active_state_record_path,
        branch_state: &branch_state,
        registry_state: &registry_state,
        active_state: &active_state_value,
        pending_commit_transaction: pending_commit_transaction.as_ref(),
        pending_state_transaction: pending_state_transaction.as_ref(),
        active_ledger_descriptor: active_ledger_descriptor.as_ref(),
    };
    if let Err(divergence) = crate::open_snapshot::verify_open_snapshot(&identity, &state) {
        issues.push(DoctorIssue::open_snapshot_divergence(*divergence));
    }
}

fn read_optional_active_transaction(
    active_state: &crate::active_state::ActiveStateDir,
    file_name: &str,
    limit: u64,
) -> Option<Value> {
    let relative = Path::new(file_name);
    match active_state.exists(relative) {
        Ok(true) => active_state
            .read_required(relative, limit)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
        Ok(false) | Err(_) => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitSemanticProfile {
    Current,
    LegacyUnversioned,
}

enum DoctorSealedCandidateKind {
    BranchHead {
        record_path: PathBuf,
    },
    OpenBase {
        registry_path: PathBuf,
        open_path: Option<PathBuf>,
    },
}

struct SuccessfulDoctorSealedCandidate {
    commit_id: String,
    issue_index: usize,
    kind: DoctorSealedCandidateKind,
}

struct DoctorSealedValidation {
    session: Option<codefire_store::SealedCommitValidationSession>,
    begin_error: Option<String>,
    successful: Vec<SuccessfulDoctorSealedCandidate>,
}

impl DoctorSealedValidation {
    fn new(objects: &Path) -> Self {
        match codefire_store::begin_sealed_commit_validation_session(objects) {
            Ok(session) => Self {
                session: Some(session),
                begin_error: None,
                successful: Vec::new(),
            },
            Err(error) => Self {
                session: None,
                begin_error: Some(error.to_string()),
                successful: Vec::new(),
            },
        }
    }

    fn validate_branch_head(
        &mut self,
        commit_id: &str,
        record_path: PathBuf,
        issues: &mut Vec<DoctorIssue>,
    ) {
        self.validate(
            commit_id,
            DoctorSealedCandidateKind::BranchHead { record_path },
            issues,
        );
    }

    fn validate_open_base(
        &mut self,
        commit_id: &str,
        registry_path: PathBuf,
        open_path: Option<PathBuf>,
        issues: &mut Vec<DoctorIssue>,
    ) {
        self.validate(
            commit_id,
            DoctorSealedCandidateKind::OpenBase {
                registry_path,
                open_path,
            },
            issues,
        );
    }

    fn validate(
        &mut self,
        commit_id: &str,
        kind: DoctorSealedCandidateKind,
        issues: &mut Vec<DoctorIssue>,
    ) {
        let result = match self.session.as_mut() {
            Some(session) => session
                .validate_commit_recoverable(commit_id)
                .map_err(|error| error.to_string()),
            None => Err(self
                .begin_error
                .clone()
                .unwrap_or_else(|| "sealed validation session is unavailable".to_string())),
        };
        match result {
            Ok(()) => self.successful.push(SuccessfulDoctorSealedCandidate {
                commit_id: commit_id.to_string(),
                issue_index: issues.len(),
                kind,
            }),
            Err(message) => push_doctor_candidate_error(issues, &kind, message),
        }
    }

    fn finish(mut self, issues: &mut Vec<DoctorIssue>) -> usize {
        let finalization_error = self
            .session
            .as_mut()
            .and_then(|session| session.finalize().err())
            .map(|error| error.to_string());
        let mut checked_branches = 0usize;
        let mut inserted = 0usize;
        for candidate in self.successful {
            let diagnostic = match finalization_error.as_ref() {
                Some(message) => Some(DoctorIssue::error(
                    candidate_error_kind(&candidate.kind),
                    message.clone(),
                    Some(candidate_error_path(&candidate.kind).to_path_buf()),
                )),
                None => {
                    let payload = self
                        .session
                        .as_ref()
                        .expect("successful doctor validation retains its session")
                        .validated_commit_payload(&candidate.commit_id);
                    match payload.map_err(Box::new).and_then(commit_semantic_profile) {
                        Ok(CommitSemanticProfile::Current) => {
                            checked_branches += usize::from(matches!(
                                &candidate.kind,
                                DoctorSealedCandidateKind::BranchHead { .. }
                            ));
                            None
                        }
                        Ok(CommitSemanticProfile::LegacyUnversioned) => {
                            checked_branches += usize::from(matches!(
                                &candidate.kind,
                                DoctorSealedCandidateKind::BranchHead { .. }
                            ));
                            Some(legacy_profile_issue(&candidate))
                        }
                        Err(error) => Some(DoctorIssue::error(
                            candidate_error_kind(&candidate.kind),
                            error.to_string(),
                            Some(candidate_error_path(&candidate.kind).to_path_buf()),
                        )),
                    }
                }
            };
            if let Some(diagnostic) = diagnostic {
                issues.insert(candidate.issue_index + inserted, diagnostic);
                inserted += 1;
            }
        }
        checked_branches
    }
}

fn candidate_error_kind(kind: &DoctorSealedCandidateKind) -> &'static str {
    match kind {
        DoctorSealedCandidateKind::BranchHead { .. } => "invalid_branch_head",
        DoctorSealedCandidateKind::OpenBase { .. } => "invalid_open_base_commit",
    }
}

fn candidate_error_path(kind: &DoctorSealedCandidateKind) -> &Path {
    match kind {
        DoctorSealedCandidateKind::BranchHead { record_path } => record_path,
        DoctorSealedCandidateKind::OpenBase { registry_path, .. } => registry_path,
    }
}

fn push_doctor_candidate_error(
    issues: &mut Vec<DoctorIssue>,
    kind: &DoctorSealedCandidateKind,
    message: String,
) {
    issues.push(DoctorIssue::error(
        candidate_error_kind(kind),
        message,
        Some(candidate_error_path(kind).to_path_buf()),
    ));
}

fn legacy_profile_issue(candidate: &SuccessfulDoctorSealedCandidate) -> DoctorIssue {
    match &candidate.kind {
        DoctorSealedCandidateKind::BranchHead { record_path } => DoctorIssue::warning(
            "legacy_branch_head_semantic_profile",
            format!(
                "branch head {} is a valid legacy-unversioned sealed commit, but it does not carry {} semantic proof",
                candidate.commit_id,
                codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
            ),
            Some(record_path.clone()),
        ),
        DoctorSealedCandidateKind::OpenBase { open_path, .. } => DoctorIssue::warning(
            "legacy_open_base_semantic_profile",
            format!(
                "open base {} is a valid legacy-unversioned sealed commit, but it does not carry {} semantic proof; scan this workspace to construct current-profile state",
                candidate.commit_id,
                codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
            ),
            open_path.clone(),
        ),
    }
}

fn commit_semantic_profile(
    commit: &Value,
) -> Result<CommitSemanticProfile, Box<codefire_store::StoreError>> {
    match commit.get("semantic_validation_profile") {
        None => Ok(CommitSemanticProfile::LegacyUnversioned),
        Some(Value::String(profile))
            if profile == codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE =>
        {
            Ok(CommitSemanticProfile::Current)
        }
        Some(Value::String(_)) => Err(Box::new(codefire_store::StoreError::InvalidSealedCommit(
            format!(
                "unsupported commit.semantic_validation_profile; supported value is {}",
                codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE
            ),
        ))),
        Some(_) => Err(Box::new(codefire_store::StoreError::InvalidSealedCommit(
            "commit.semantic_validation_profile must be a string".to_string(),
        ))),
    }
}

fn check_open_marker(
    cf: &Path,
    registry_path: &Path,
    registry: &Value,
    open_path: &Path,
    branch_name: Option<&str>,
    open_instance_id: Option<&str>,
    issues: &mut Vec<DoctorIssue>,
) {
    let marker_path = open_path.join(".codefire-open");
    if !marker_path.exists() {
        issues.push(DoctorIssue::error(
            "missing_open_marker",
            "open path is missing .codefire-open marker",
            Some(marker_path),
        ));
        return;
    }
    let marker = match read_doctor_json(&marker_path, ACTIVE_CONTROL_FILE_BYTE_LIMIT) {
        Ok(marker) => marker,
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_open_marker",
                error.to_string(),
                Some(marker_path),
            ));
            return;
        }
    };
    if path_string(&marker, &["repository", "path"]).as_deref() != Some(cf) {
        issues.push(DoctorIssue::error(
            "open_marker_repository_mismatch",
            "open marker repository.path does not match registry repository",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["branch", "name"]).as_deref() != branch_name {
        issues.push(DoctorIssue::error(
            "open_marker_branch_mismatch",
            "open marker branch.name does not match opened registry branch.name",
            Some(marker_path.clone()),
        ));
    }
    if value_string(&marker, &["open", "open_instance_id"]).as_deref() != open_instance_id {
        issues.push(DoctorIssue::error(
            "open_marker_instance_mismatch",
            "open marker open_instance_id does not match opened registry",
            Some(marker_path.clone()),
        ));
    }
    if path_string(&marker, &["open", "opened_path"]).as_deref() != Some(open_path) {
        issues.push(DoctorIssue::error(
            "open_marker_path_mismatch",
            "open marker opened_path does not match opened registry open.path",
            Some(marker_path),
        ));
    }
    if value_string(registry, &["state", "last_known"]).is_none() {
        issues.push(DoctorIssue::error(
            "missing_open_registry_state",
            "opened registry must contain state.last_known",
            Some(registry_path.to_path_buf()),
        ));
    }
}

fn check_active_state_files(cf: &Path, issues: &mut Vec<DoctorIssue>) -> Result<usize, CliError> {
    let active = cf.join("active");
    let active_root = match crate::secure_worktree::open_absolute_dir_nofollow(&active) {
        Ok(active_root) => active_root,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                format!("active-state root cannot be pinned no-follow: {error}"),
                Some(active),
            ));
            return Ok(0);
        }
    };
    let root_identity = crate::secure_worktree::StableFsIdentity::for_directory(&active_root)?;
    let issue_start = issues.len();
    let mut names = active_root
        .entries()?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut checked = 0usize;
    for name in names {
        let mut state_dir = active.clone();
        state_dir.push(&name);
        let metadata = match active_root.symlink_metadata(&name) {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    format!("active-state entry cannot be inspected no-follow: {error}"),
                    Some(state_dir),
                ));
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                "active-state instance must not be a symlink",
                Some(state_dir),
            ));
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        let pinned_instance = match active_root.open_dir_nofollow(&name) {
            Ok(instance) => instance,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    format!("active-state instance cannot be pinned no-follow: {error}"),
                    Some(state_dir),
                ));
                continue;
            }
        };
        let instance_identity =
            crate::secure_worktree::StableFsIdentity::for_directory(&pinned_instance)?;
        let instance_issue_start = issues.len();
        let instance_checked = check_active_state_dir(&state_dir, issues);
        let rebound = active_root.open_dir_nofollow(&name);
        if rebound
            .and_then(|directory| {
                instance_identity.ensure_directory(&directory, "active-state instance")
            })
            .is_err()
        {
            issues.truncate(instance_issue_start);
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                "active-state instance identity changed during doctor inspection",
                Some(state_dir),
            ));
        } else {
            checked += instance_checked;
        }
    }
    if root_identity
        .ensure_directory(&active_root, "active-state root")
        .and_then(|()| crate::secure_worktree::ensure_absolute_dir_identity(&active_root, &active))
        .is_err()
    {
        issues.truncate(issue_start);
        issues.push(DoctorIssue::error(
            "invalid_active_state_dir",
            "active-state root identity changed during doctor inspection",
            Some(active),
        ));
        return Ok(0);
    }
    Ok(checked)
}

fn check_active_state_dir(state_dir: &Path, issues: &mut Vec<DoctorIssue>) -> usize {
    let active_state = match crate::active_state::ActiveStateDir::bind(state_dir) {
        Ok(active_state) => active_state,
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                error.to_string(),
                Some(state_dir.to_path_buf()),
            ));
            return 0;
        }
    };
    let state_path = active_state.display_path(Path::new("state.json"));
    match active_state.exists(Path::new("state.json")) {
        Ok(true) => {}
        Ok(false) => issues.push(DoctorIssue::error(
            "missing_active_state_file",
            "active state directory is missing required state.json",
            Some(state_path),
        )),
        Err(error) => {
            issues.push(DoctorIssue::error(
                "invalid_active_state_dir",
                error.to_string(),
                Some(state_dir.to_path_buf()),
            ));
            return 0;
        }
    }
    let mut checked = 0usize;
    let mut state_value = None::<String>;
    let mut open_fire_count = None::<usize>;
    for &file_name in ACTIVE_STATE_FILES {
        let relative = Path::new(file_name);
        let path = active_state.display_path(relative);
        match active_state.exists(relative) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(error) => {
                issues.push(DoctorIssue::error(
                    "invalid_active_state_dir",
                    error.to_string(),
                    Some(state_dir.to_path_buf()),
                ));
                break;
            }
        }
        match read_active_state_json(&active_state, relative, file_name) {
            Ok(value) => match validate_active_state_shape(file_name, value.clone()) {
                Ok(()) => {
                    if file_name == "state.json" {
                        state_value = value
                            .get("state")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    } else if file_name == "fires.json" {
                        open_fire_count = value.as_array().map(|fires| {
                            fires
                                .iter()
                                .filter(|fire| {
                                    fire.get("status").and_then(Value::as_str) == Some("open")
                                })
                                .count()
                        });
                    } else if file_name == "commit_transaction.json"
                        && value.get("status").and_then(Value::as_str) == Some("pending")
                    {
                        let phase = value
                            .get("phase")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let (stored_objects, planned_objects) = value
                            .get("objects")
                            .and_then(Value::as_array)
                            .map_or((0, 0), |objects| {
                                objects.iter().fold((0, 0), |(stored, planned), object| {
                                    if object.is_string()
                                        || object.get("state").and_then(Value::as_str)
                                            == Some("stored")
                                    {
                                        (stored + 1, planned)
                                    } else if object.get("state").and_then(Value::as_str)
                                        == Some("planned")
                                    {
                                        (stored, planned + 1)
                                    } else {
                                        (stored, planned)
                                    }
                                })
                            });
                        let pending_authority = pending_commit_authority(&value);
                        issues.push(DoctorIssue::pending_authority(
                            "pending_commit_transaction",
                            format!(
                                "commit transaction is pending at phase {phase} with {stored_objects} stored object(s) and {planned_objects} planned object(s)"
                            ),
                            path.clone(),
                            pending_authority,
                        ));
                    } else if file_name == "state_transaction.json"
                        && value.get("status").and_then(Value::as_str) == Some("pending")
                    {
                        let operation = value
                            .get("operation")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let phase = value
                            .get("phase")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let intended_state = value
                            .get("intended_state")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        let pending_authority = pending_state_authority(&value);
                        issues.push(DoctorIssue::pending_authority(
                            "pending_state_transaction",
                            format!(
                                "{operation} state transaction is pending at phase {phase} for intended state {intended_state}"
                            ),
                            path.clone(),
                            pending_authority,
                        ));
                    } else if file_name
                        == crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE
                    {
                        match crate::fire::active_ledger_transaction::validate_active_ledger_descriptor(&active_state) {
                            Ok(Some(descriptor)) if descriptor.publication_status != crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => {
                                let pending_authority = pending_active_ledger_authority(&descriptor);
                                let message = match descriptor.publication_status {
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::LedgerPending => format!(
                                        "active ledger transaction {} ({}) is pending at phase {}",
                                        descriptor.request_hash, descriptor.operation, descriptor.phase
                                    ),
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::StatePublicationPending => format!(
                                        "active ledger transaction {} ({}) completed ledger phase {} and is awaiting state publication",
                                        descriptor.request_hash, descriptor.operation, descriptor.phase
                                    ),
                                    crate::fire::active_ledger_transaction::ActiveLedgerPublicationStatus::Released => unreachable!("released descriptors were filtered above"),
                                };
                                issues.push(DoctorIssue::pending_authority(
                                    "pending_active_ledger_transaction",
                                    message,
                                    path.clone(),
                                    pending_authority,
                                ));
                            }
                            Ok(Some(_)) | Ok(None) => {}
                            Err(error) => issues.push(DoctorIssue::error(
                                "invalid_active_ledger_transaction",
                                error.to_string(),
                                Some(path.clone()),
                            )),
                        }
                    }
                    checked += 1;
                }
                Err(message) => issues.push(DoctorIssue::error(
                    "invalid_active_state_shape",
                    message,
                    Some(path),
                )),
            },
            Err(error) => issues.push(DoctorIssue::error(
                "invalid_active_state_json",
                error.to_string(),
                Some(path),
            )),
        }
    }
    if matches!(
        state_value.as_deref(),
        Some("open-clean" | "open-consistent")
    ) && open_fire_count.unwrap_or(0) > 0
    {
        issues.push(DoctorIssue::error(
            "active_state_invariant_violation",
            "state is clean/consistent but fires.json contains open fires",
            Some(state_dir.to_path_buf()),
        ));
    }
    checked
}

fn pending_commit_authority(value: &Value) -> DoctorPendingAuthority {
    let phase = value
        .get("phase")
        .and_then(Value::as_str)
        .expect("shared commit transaction validator requires phase");
    let transaction_id = value
        .get("transaction_id")
        .and_then(Value::as_str)
        .expect("shared commit transaction validator requires transaction_id");
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::Commit,
        operation: DoctorPendingOperation::Commit,
        phase: phase.to_string(),
        request_identity: bounded_doctor_identity(transaction_id)
            .map(|transaction_id| DoctorPendingRequestIdentity::TransactionId { transaction_id })
            .unwrap_or(DoctorPendingRequestIdentity::Withheld {
                reason: "identifier_not_safe_for_diagnostics",
            }),
        intended_state: None,
    }
}

fn pending_state_authority(value: &Value) -> DoctorPendingAuthority {
    let operation = value
        .get("operation")
        .and_then(Value::as_str)
        .and_then(DoctorPendingOperation::from_validated)
        .expect("shared state transaction validator requires a supported operation");
    let phase = value
        .get("phase")
        .and_then(Value::as_str)
        .expect("shared state transaction validator requires phase");
    let intended_state = value
        .get("intended_state")
        .and_then(Value::as_str)
        .expect("shared state transaction validator requires intended_state");
    let request_identity = if value.get("version").and_then(Value::as_u64) == Some(2) {
        if let Some(request_hash) = value.get("request_hash").and_then(Value::as_str) {
            DoctorPendingRequestIdentity::RequestHash {
                request_hash: request_hash.to_string(),
            }
        } else {
            let subject = (
                value.pointer("/subject/branch").and_then(Value::as_str),
                value
                    .pointer("/subject/open_instance_id")
                    .and_then(Value::as_str),
                value
                    .pointer("/subject/current_base_commit")
                    .and_then(Value::as_str),
            );
            match subject {
                (Some(branch), Some(open_instance_id), Some(current_base_commit)) => {
                    match (
                        bounded_doctor_identity(branch),
                        bounded_doctor_identity(open_instance_id),
                        bounded_doctor_identity(current_base_commit),
                    ) {
                        (Some(branch), Some(open_instance_id), Some(current_base_commit)) => {
                            DoctorPendingRequestIdentity::StateSubject {
                                branch,
                                open_instance_id,
                                current_base_commit,
                            }
                        }
                        _ => DoctorPendingRequestIdentity::Withheld {
                            reason: "identifier_not_safe_for_diagnostics",
                        },
                    }
                }
                _ => unreachable!("shared version-2 state transaction validator requires subject"),
            }
        }
    } else {
        DoctorPendingRequestIdentity::LegacyUnbound
    };
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::State,
        operation,
        phase: phase.to_string(),
        request_identity,
        intended_state: Some(intended_state.to_string()),
    }
}

fn pending_active_ledger_authority(
    descriptor: &crate::fire::active_ledger_transaction::ValidatedActiveLedgerDescriptor,
) -> DoctorPendingAuthority {
    let operation = DoctorPendingOperation::from_validated(&descriptor.operation)
        .expect("shared active-ledger descriptor requires a supported operation");
    DoctorPendingAuthority {
        authority: DoctorPendingAuthorityKind::ActiveLedger,
        operation,
        phase: descriptor.phase.clone(),
        request_identity: DoctorPendingRequestIdentity::RequestHash {
            request_hash: descriptor.request_hash.clone(),
        },
        intended_state: None,
    }
}

fn bounded_doctor_identity(value: &str) -> Option<String> {
    (!value.is_empty()
        && value.len() <= MAX_DOCTOR_IDENTITY_BYTES
        && !value.chars().any(char::is_control))
    .then(|| value.to_string())
}

fn read_active_state_json(
    active_state: &crate::active_state::ActiveStateDir,
    relative: &Path,
    file_name: &str,
) -> Result<Value, CliError> {
    let limit = match file_name {
        "state.json" => ACTIVE_CONTROL_FILE_BYTE_LIMIT,
        "state_transaction.json" => crate::STATE_TRANSACTION_BYTE_LIMIT,
        "commit_transaction.json" => crate::COMMIT_TRANSACTION_BYTE_LIMIT,
        crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_FILE => {
            crate::fire::active_ledger_transaction::ACTIVE_LEDGER_TRANSACTION_BYTE_LIMIT
        }
        _ => ACTIVE_DATA_FILE_BYTE_LIMIT,
    };
    let bytes = active_state.read_required(relative, limit)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn read_doctor_json(path: &Path, limit: u64) -> Result<Value, CliError> {
    let bytes = crate::secure_worktree::read_file_nofollow_bounded(path, limit)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate_active_state_shape(file_name: &str, value: Value) -> Result<(), String> {
    match file_name {
        "state.json" => {
            let state = value
                .get("state")
                .and_then(Value::as_str)
                .ok_or_else(|| "state.json must contain string field state".to_string())?;
            if !matches!(state, "open-clean" | "open-burning" | "open-consistent") {
                return Err(format!("state.json contains unsupported state: {state}"));
            }
            if value
                .get("pending_merge_parent")
                .is_some_and(|parent| !parent.is_string())
            {
                return Err("state.json pending_merge_parent must be a string".to_string());
            }
            Ok(())
        }
        "fires.json" => serde_json::from_value::<Vec<codefire_core::Fire>>(value)
            .map(|_| ())
            .map_err(|error| format!("fires.json shape is invalid: {error}")),
        "resolutions.json" => serde_json::from_value::<Vec<codefire_core::Resolution>>(value)
            .map(|_| ())
            .map_err(|error| format!("resolutions.json shape is invalid: {error}")),
        "scan.json" => serde_json::from_value::<codefire_core::ScanResult>(value)
            .map(|_| ())
            .map_err(|error| format!("scan.json shape is invalid: {error}")),
        "verification.json" => serde_json::from_value::<codefire_core::Verification>(value)
            .map(|_| ())
            .map_err(|error| format!("verification.json shape is invalid: {error}")),
        "commit_transaction.json" => validate_commit_transaction_shape(value),
        "state_transaction.json" => validate_state_transaction_shape(value),
        _ => Ok(()),
    }
}

fn validate_state_transaction_shape(value: Value) -> Result<(), String> {
    crate::validate_state_transaction_record(&value)
}

fn validate_commit_transaction_shape(value: Value) -> Result<(), String> {
    crate::validate_commit_transaction_record(&value)
}

fn value_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(ToString::to_string)
}

fn path_string(value: &Value, path: &[&str]) -> Option<PathBuf> {
    value_string(value, path).map(PathBuf::from)
}

fn collect_json_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn doctor_sealed_session_attributes_final_subject_change_to_valid_candidate() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let initialized = crate::init_repo(&repo, false).unwrap();
        let cf = repo.join(".codefire");
        let objects = cf.join("objects");
        let commit_path =
            codefire_store::object_record_path(&objects, &initialized.main_commit).unwrap();

        let checked = run_checks_with_sealed_validation_hook(&cf, true, || {
            let mut bytes = fs::read(&commit_path)?;
            bytes.push(b' ');
            fs::write(&commit_path, bytes)?;
            Ok(())
        })
        .unwrap();

        assert_eq!(checked.branches, 0);
        assert!(checked.issues.iter().any(|issue| {
            issue.kind == "invalid_branch_head"
                && issue.message.contains("sealed validation subject changed")
        }));
    }
}
