use super::CliError;
use serde_json::{json, Value};

pub(crate) const SUPPORT_PROFILE: &str = "codefire.support.v1";
pub(crate) const COMMAND_RESULT_SCHEMA: &str = "codefire.command_result.v1";
pub(crate) const CAPABILITIES_SCHEMA: &str = "codefire.capabilities.v1";
pub(crate) const REMOTE_DOCTOR_SCHEMA: &str = "codefire.remote_doctor.v1";
pub(crate) const PAGE_SCHEMA: &str = "codefire.page.v1";
pub(crate) const PAGE_CURSOR_SCHEMA: &str = "codefire.page_cursor.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionSupport {
    Legacy,
    Current,
}

#[derive(Debug, Clone, Copy)]
struct FormatContract {
    id: &'static str,
    current: u64,
    readable: &'static [u64],
    missing_is_legacy: bool,
    writer: bool,
}

const FORMAT_CONTRACTS: &[FormatContract] = &[
    format("object_record", 1, &[1], true, true),
    format("commit_payload", 1, &[1], false, true),
    format("commit_root_schema", 2, &[1, 2], false, true),
    format("remote_project_descriptor", 1, &[1], false, true),
    format("remote_branch", 1, &[1], false, true),
    format("remote_merge_request", 1, &[1], false, true),
    format("remote_publication_transaction", 1, &[1], false, true),
    format("remote_idempotency_receipt", 2, &[1, 2], false, true),
    format("command_result", 1, &[1], false, true),
    format("capabilities", 1, &[1], false, true),
    format("remote_doctor", 1, &[1], false, true),
];

const fn format(
    id: &'static str,
    current: u64,
    readable: &'static [u64],
    missing_is_legacy: bool,
    writer: bool,
) -> FormatContract {
    FormatContract {
        id,
        current,
        readable,
        missing_is_legacy,
        writer,
    }
}

pub(crate) fn validate_record_version(
    value: &Value,
    contract_id: &str,
) -> Result<VersionSupport, CliError> {
    let contract = FORMAT_CONTRACTS
        .iter()
        .find(|contract| contract.id == contract_id)
        .ok_or_else(|| {
            CliError::InvalidRepository(format!("unknown internal format contract: {contract_id}"))
        })?;
    let Some(raw_version) = value.get("version") else {
        return if contract.missing_is_legacy {
            Ok(VersionSupport::Legacy)
        } else {
            Err(CliError::InvalidRepository(format!(
                "{contract_id}.version is required"
            )))
        };
    };
    let version = raw_version.as_u64().ok_or_else(|| {
        CliError::InvalidRepository(format!("{contract_id}.version must be an unsigned integer"))
    })?;
    if !contract.readable.contains(&version) {
        return Err(CliError::InvalidRepository(format!(
            "unsupported {contract_id}.version {version}; readable versions are {}",
            contract
                .readable
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(if version == contract.current {
        VersionSupport::Current
    } else {
        VersionSupport::Legacy
    })
}

pub(crate) fn support_manifest_json() -> Value {
    json!({
        "type": "codefire_support_manifest",
        "version": 1,
        "profile": SUPPORT_PROFILE,
        "unknown_version": "deny",
        "unknown_critical_extension": "deny",
        "unknown_noncritical_extension": "preserve",
        "canonical_json_profile": "serde-json-compact-v1",
        "public_object_id": {
            "writer_digest_hex_length": codefire_store::OBJECT_ID_DIGEST_HEX_LENGTH,
            "reader_digest_hex_lengths": [12, codefire_store::OBJECT_ID_DIGEST_HEX_LENGTH],
        },
        "semantic_validation_profile": codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE,
        "runtimes": [
            {"id": "rust-default", "role": "default", "read": true, "write": true},
            {"id": "python-fallback", "role": "reference-fallback", "read": true, "write": true, "command_manifest_authority": false},
            {"id": "independent-python-stdlib-v1", "role": "read-only-conformance", "read": true, "write": false}
        ],
        "formats": FORMAT_CONTRACTS.iter().map(|contract| json!({
            "id": contract.id,
            "current": contract.current,
            "readable": contract.readable,
            "missing": if contract.missing_is_legacy { "legacy" } else { "deny" },
            "writer": contract.writer,
        })).collect::<Vec<_>>(),
        "schemas": [
            {"id": COMMAND_RESULT_SCHEMA, "path": "schemas/public/command-result-v1.schema.json"},
            {"id": CAPABILITIES_SCHEMA, "path": "schemas/public/capabilities-v1.schema.json"},
            {"id": REMOTE_DOCTOR_SCHEMA, "path": "schemas/public/remote-doctor-v1.schema.json"},
            {"id": PAGE_SCHEMA, "path": "schemas/public/page-v1.schema.json"},
            {"id": PAGE_CURSOR_SCHEMA, "path": "schemas/public/page-cursor-v1.schema.json"},
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gate_distinguishes_legacy_current_and_future() {
        assert_eq!(
            validate_record_version(&json!({}), "object_record").unwrap(),
            VersionSupport::Legacy
        );
        assert_eq!(
            validate_record_version(&json!({"version": 1}), "remote_branch").unwrap(),
            VersionSupport::Current
        );
        let error = validate_record_version(&json!({"version": 2}), "remote_branch").unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported remote_branch.version 2"));
    }
}
