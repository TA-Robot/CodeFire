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
    #[allow(dead_code)]
    // Reserved by the v0.6 public taxonomy; merge blocking lands in CF-222/CF-232 follow-ups.
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
}

impl ExitCode {
    pub(crate) fn code(self) -> i32 {
        self as i32
    }
}

pub(crate) fn verification_exit_code(verification: &codefire_core::Verification) -> ExitCode {
    if verification.result == "passed" {
        return ExitCode::Success;
    }
    if verification.open_required_fires > 0 {
        return ExitCode::OpenFires;
    }
    if verification.trace_completeness_required && !verification.missing_required_links.is_empty() {
        return ExitCode::MissingRequiredLinks;
    }
    if !verification.stale_resolutions.is_empty() {
        return ExitCode::StaleResolution;
    }
    if !verification.missing_evidence_refs.is_empty() {
        return ExitCode::ObjectReferenceInvalid;
    }
    if !verification.duplicate_atom_ids.is_empty() {
        return ExitCode::DuplicateAtomId;
    }
    if !verification.failed_checks.is_empty() {
        return ExitCode::VerificationCommandFailed;
    }
    ExitCode::GenericFailure
}

pub(crate) fn core_error_exit_code(error: &codefire_core::CoreError) -> ExitCode {
    match error {
        codefire_core::CoreError::Config(_) => ExitCode::InvalidUsageOrConfig,
        codefire_core::CoreError::Io(_) => ExitCode::GenericFailure,
        codefire_core::CoreError::Json(_) => ExitCode::RepositoryCorruption,
    }
}

pub(crate) fn store_error_exit_code(error: &codefire_store::StoreError) -> ExitCode {
    match error {
        codefire_store::StoreError::InvalidSealedCommit(_) => ExitCode::SealedCommitInvalid,
        codefire_store::StoreError::ObjectIdMismatch { .. }
        | codefire_store::StoreError::ObjectHashMismatch { .. }
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
            }],
            missing_evidence_refs: vec![codefire_core::MissingEvidenceRef {
                resolution_uid: "resolution_1".to_string(),
                evidence_id: "CF-EVIDENCE-missing".to_string(),
            }],
            duplicate_atom_ids: vec!["REQ-session".to_string()],
            verified_at: "2026-06-04T00:00:00Z".to_string(),
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
            duplicate_atom_ids: Vec::new(),
            verified_at: "2026-06-04T00:00:00Z".to_string(),
        };
        assert_eq!(
            verification_exit_code(&missing_evidence_only),
            ExitCode::ObjectReferenceInvalid
        );
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
    }
}
