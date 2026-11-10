#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum ExitCode {
    Success = 0,
    GenericFailure = 1,
    InvalidUsageOrConfig = 2,
    OpenFires = 10,
    MissingRequiredLinks = 11,
    StaleResolution = 12,
    DuplicateAtomId = 13,
    VerificationCommandFailed = 14,
    MergeConflict = 15,
    RepositoryCorruption = 20,
    ObjectReferenceInvalid = 21,
    SealedCommitInvalid = 22,
    LockContention = 30,
    RemoteRejected = 31,
    #[allow(dead_code)]
    // Reserved by the v0.6 public taxonomy; signature enforcement lands in CF-213.
    AuthenticationOrSignatureFailure = 32,
    #[allow(dead_code)]
    // Reserved by the v0.6 public taxonomy; idempotency lands in CF-224.
    IdempotencyConflict = 33,
    #[allow(dead_code)]
    // Reserved by the v0.6 public taxonomy; migration checks land in CF-216.
    MigrationIncompatibility = 40,
    #[allow(dead_code)]
    // Reserved by the v0.6 public taxonomy; artifact validation lands in CF-215/CF-226.
    ExternalArtifactInvalid = 50,
    ResourceBudgetExceeded = 60,
}

impl ExitCode {
    pub(crate) fn code(self) -> i32 {
        self as i32
    }
}

pub(crate) fn verification_exit_code(verification: &codefire_core::Verification) -> ExitCode {
    match codefire_core::committable_decision(verification).primary_reason {
        None => ExitCode::Success,
        Some(codefire_core::CommittableReason::UnresolvedMergeConflicts) => ExitCode::MergeConflict,
        Some(codefire_core::CommittableReason::OpenRequiredFires) => ExitCode::OpenFires,
        Some(codefire_core::CommittableReason::MissingRequiredLinks) => {
            ExitCode::MissingRequiredLinks
        }
        Some(codefire_core::CommittableReason::StaleResolutions) => ExitCode::StaleResolution,
        Some(codefire_core::CommittableReason::MissingEvidenceRefs) => {
            ExitCode::ObjectReferenceInvalid
        }
        Some(codefire_core::CommittableReason::DuplicateAtomIds) => ExitCode::DuplicateAtomId,
        Some(codefire_core::CommittableReason::FailedChecks) => ExitCode::VerificationCommandFailed,
    }
}

pub(crate) fn core_error_exit_code(error: &codefire_core::CoreError) -> ExitCode {
    match error {
        codefire_core::CoreError::Config(_) => ExitCode::InvalidUsageOrConfig,
        codefire_core::CoreError::Io(_) => ExitCode::GenericFailure,
        codefire_core::CoreError::Json(_) => ExitCode::RepositoryCorruption,
        codefire_core::CoreError::TraceGraphEndpointIntegrity(_) => {
            ExitCode::ObjectReferenceInvalid
        }
        codefire_core::CoreError::SealedSnapshotSemanticIntegrity(_) => {
            ExitCode::SealedCommitInvalid
        }
    }
}

pub(crate) fn store_error_exit_code(error: &codefire_store::StoreError) -> ExitCode {
    match error {
        codefire_store::StoreError::InvalidSealedCommit(_)
        | codefire_store::StoreError::SealedValidationSubjectChanged { .. } => {
            ExitCode::SealedCommitInvalid
        }
        codefire_store::StoreError::DecisionWorkAccounting(error)
            if error.budget_exceeded().is_some() =>
        {
            ExitCode::ResourceBudgetExceeded
        }
        codefire_store::StoreError::DecisionWorkAccounting(_) => ExitCode::GenericFailure,
        codefire_store::StoreError::ValidationCancelled => ExitCode::ResourceBudgetExceeded,
        codefire_store::StoreError::ObjectPublicationRetryRequired => ExitCode::GenericFailure,
        codefire_store::StoreError::ObjectIdMismatch { .. }
        | codefire_store::StoreError::ObjectHashMismatch { .. }
        | codefire_store::StoreError::ObjectCollision { .. }
        | codefire_store::StoreError::ObjectRecordTooLarge { .. }
        | codefire_store::StoreError::FilenameMismatch { .. }
        | codefire_store::StoreError::ObjectNotFound(_)
        | codefire_store::StoreError::UnknownObjectType(_)
        | codefire_store::StoreError::IncompleteRecord(_) => ExitCode::ObjectReferenceInvalid,
        codefire_store::StoreError::Json(_) => ExitCode::RepositoryCorruption,
        codefire_store::StoreError::Io(_) => ExitCode::GenericFailure,
    }
}

pub(crate) fn usage_exit_code(message: &str) -> ExitCode {
    if message.starts_with("HTTP remote error:") {
        ExitCode::RemoteRejected
    } else {
        ExitCode::InvalidUsageOrConfig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_exit_code_uses_blocker_priority_order() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            trace_completeness_required: true,
            open_required_fires: 0,
            failed_checks: vec![codefire_core::FailedCheck {
                id: "unit".to_string(),
                command: "cargo test".to_string(),
                output: "failed".to_string(),
            }],
            missing_required_links: vec![codefire_core::MissingRequiredLink {
                atom_id: "REQ-session".to_string(),
                required_type: "refined_by".to_string(),
                target_kind: "design".to_string(),
                min: 1,
                found: 0,
            }],
            stale_resolutions: vec![codefire_core::StaleResolution {
                resolution_uid: "resolution_1".to_string(),
                reason: "source changed".to_string(),
                ..Default::default()
            }],
            missing_evidence_refs: vec![codefire_core::MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            unresolved_merge_conflicts: Vec::new(),
            duplicate_atom_ids: vec!["REQ-session".to_string()],
            verified_at: "2026-06-04T00:00:00Z".to_string(),
            subject: None,
            execution: Vec::new(),
            effective_policy: None,
            change_coverage: None,
        };

        assert_eq!(
            verification_exit_code(&verification),
            ExitCode::MissingRequiredLinks
        );

        let missing_evidence_only = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "failed".to_string(),
            trace_completeness_required: true,
            open_required_fires: 0,
            failed_checks: Vec::new(),
            missing_required_links: Vec::new(),
            stale_resolutions: Vec::new(),
            missing_evidence_refs: vec![codefire_core::MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            unresolved_merge_conflicts: Vec::new(),
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-06-04T00:00:00Z".to_string(),
            subject: None,
            execution: Vec::new(),
            effective_policy: None,
            change_coverage: None,
        };
        assert_eq!(
            verification_exit_code(&missing_evidence_only),
            ExitCode::ObjectReferenceInvalid
        );
    }

    #[test]
    fn verification_exit_code_uses_typed_facts_not_recorded_result() {
        let verification = codefire_core::Verification {
            type_tag: "verification".to_string(),
            version: 1,
            result: "passed".to_string(),
            trace_completeness_required: false,
            open_required_fires: 1,
            failed_checks: Vec::new(),
            missing_required_links: Vec::new(),
            stale_resolutions: Vec::new(),
            missing_evidence_refs: Vec::new(),
            unresolved_merge_conflicts: Vec::new(),
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-07-12T00:00:00Z".to_string(),
            subject: None,
            execution: Vec::new(),
            effective_policy: None,
            change_coverage: None,
        };

        assert_eq!(verification_exit_code(&verification), ExitCode::OpenFires);
    }

    #[test]
    fn store_error_exit_code_classifies_object_and_commit_failures() {
        assert_eq!(
            store_error_exit_code(&codefire_store::StoreError::ObjectNotFound(
                "CF-BLOB-missing".to_string()
            )),
            ExitCode::ObjectReferenceInvalid
        );
        assert_eq!(
            store_error_exit_code(&codefire_store::StoreError::InvalidSealedCommit(
                "missing root".to_string()
            )),
            ExitCode::SealedCommitInvalid
        );
        assert_eq!(
            store_error_exit_code(
                &codefire_store::StoreError::SealedValidationSubjectChanged {
                    object_id: "CF-COMMIT-changed".to_string(),
                    path: std::path::PathBuf::from("objects/commits/CF-COMMIT-changed.json"),
                    kind: codefire_store::SealedValidationSubjectChangeKind::DigestChanged,
                    expected_size: 64,
                    actual_size: Some(64),
                }
            ),
            ExitCode::SealedCommitInvalid
        );
        assert_eq!(
            store_error_exit_code(&codefire_store::StoreError::ObjectRecordTooLarge {
                path: std::path::PathBuf::from("objects/blob/oversized.json"),
                size: 65,
                limit: 64,
            }),
            ExitCode::ObjectReferenceInvalid
        );
    }

    #[test]
    fn resource_budget_exit_code_is_stable() {
        assert_eq!(ExitCode::ResourceBudgetExceeded.code(), 60);
        assert_eq!(
            ExitCode::ResourceBudgetExceeded.code(),
            codefire_core::RESOURCE_BUDGET_EXIT_CODE as i32
        );
    }
}
