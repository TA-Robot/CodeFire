use crate::{CoreError, VerificationPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const EFFECTIVE_POLICY_PROFILE: &str = "effective-verification-policy-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyRuleSource {
    BuiltInDefault,
    RepositoryConfig,
    CliOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySafetyClass {
    SafetyFloor,
    OptionalHardening,
    Relaxable,
    Traceability,
    Execution,
    Disclosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicyRule {
    pub id: String,
    pub source: PolicyRuleSource,
    pub effective_value: Value,
    pub applicability: String,
    pub safety_class: PolicySafetyClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicy {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub profile: String,
    pub rules: Vec<EffectivePolicyRule>,
    pub digest: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyProvenance {
    repository_rules: BTreeSet<String>,
    repository_rule_prefixes: BTreeSet<String>,
    cli_override_rules: BTreeSet<String>,
}

impl PolicyProvenance {
    pub fn record_repository_rule(&mut self, id: impl Into<String>) {
        self.repository_rules.insert(id.into());
    }

    pub fn record_cli_override(&mut self, id: impl Into<String>) {
        self.cli_override_rules.insert(id.into());
    }

    pub fn record_repository_prefix(&mut self, prefix: impl Into<String>) {
        self.repository_rule_prefixes.insert(prefix.into());
    }

    fn source_for(&self, id: &str) -> PolicyRuleSource {
        if self.cli_override_rules.contains(id) {
            PolicyRuleSource::CliOverride
        } else if self.repository_rules.contains(id)
            || self
                .repository_rule_prefixes
                .iter()
                .any(|prefix| id.starts_with(prefix))
        {
            PolicyRuleSource::RepositoryConfig
        } else {
            PolicyRuleSource::BuiltInDefault
        }
    }
}

pub fn effective_policy(
    policy: &VerificationPolicy,
    provenance: &PolicyProvenance,
) -> Result<EffectivePolicy, CoreError> {
    let mut rules = Vec::new();
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.require_no_required_fires",
        policy.require_no_required_fires,
        "commit",
        PolicySafetyClass::SafetyFloor,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.require_no_stale_resolutions",
        policy.require_no_stale_resolutions,
        "commit",
        PolicySafetyClass::SafetyFloor,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.require_trace_completeness",
        policy.require_trace_completeness,
        "commit",
        PolicySafetyClass::Relaxable,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.require_verification_success",
        policy.require_verification_success,
        "commit",
        PolicySafetyClass::SafetyFloor,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.reject_duplicate_atom_ids",
        policy.reject_duplicate_atom_ids,
        "commit",
        PolicySafetyClass::SafetyFloor,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "commit.require_commit_signature",
        policy.require_commit_signature,
        "commit",
        PolicySafetyClass::OptionalHardening,
    );
    push_bool_rule(
        &mut rules,
        provenance,
        "extinguish.no_change_required.requires_rationale",
        policy.no_change_required_requires_rationale,
        "extinguish:no-change-required",
        PolicySafetyClass::Relaxable,
    );

    for (kind, required) in &policy.required_links {
        let mut normalized = required.clone();
        normalized.sort_by(|left, right| {
            left.link_type
                .cmp(&right.link_type)
                .then_with(|| left.target_kind.cmp(&right.target_kind))
                .then_with(|| left.min.cmp(&right.min))
        });
        let mut occurrences = BTreeMap::<String, usize>::new();
        for rule in &normalized {
            let base_id = format!(
                "trace.required_links.{kind}.{}.{}.{}",
                rule.link_type, rule.target_kind, rule.min
            );
            let occurrence = occurrences.entry(base_id.clone()).or_default();
            *occurrence += 1;
            let id = if *occurrence == 1 {
                base_id
            } else {
                format!("{base_id}#{}", *occurrence)
            };
            rules.push(EffectivePolicyRule {
                source: provenance.source_for(&id),
                id,
                effective_value: json!({
                    "type": rule.link_type,
                    "target_kind": rule.target_kind,
                    "min": rule.min,
                }),
                applicability: format!("atom_kind:{kind}"),
                safety_class: PolicySafetyClass::Traceability,
            });
        }
    }

    for command in &policy.verification {
        let id = format!("verification.command.{}", command.id);
        rules.push(EffectivePolicyRule {
            source: provenance.source_for(&id),
            id,
            effective_value: serde_json::to_value(command)?,
            applicability: "verification:local".to_string(),
            safety_class: PolicySafetyClass::Execution,
        });
    }

    // This is an explicit description of the current safety behavior, not a claim that
    // file-only evidence proves byte/span coverage.
    rules.push(EffectivePolicyRule {
        id: "coverage.unclassified_changes".to_string(),
        source: PolicyRuleSource::BuiltInDefault,
        effective_value: Value::String("allow_with_disclosure".to_string()),
        applicability: "changed_file_without_proven_span_coverage".to_string(),
        safety_class: PolicySafetyClass::Disclosure,
    });

    rules.sort_by(|left, right| left.id.cmp(&right.id));
    for pair in rules.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(CoreError::Config(format!(
                "duplicate effective policy rule id {}; verification command ids must be unique",
                pair[0].id
            )));
        }
    }
    let digest = effective_policy_digest(&rules)?;
    Ok(EffectivePolicy {
        type_tag: "effective_policy".to_string(),
        version: 1,
        profile: EFFECTIVE_POLICY_PROFILE.to_string(),
        rules,
        digest,
    })
}

pub fn validate_effective_policy(contract: &EffectivePolicy) -> Result<(), CoreError> {
    if contract.type_tag != "effective_policy"
        || contract.version != 1
        || contract.profile != EFFECTIVE_POLICY_PROFILE
    {
        return Err(CoreError::Config(
            "unsupported effective policy contract".to_string(),
        ));
    }
    let mut normalized = contract.rules.clone();
    normalized.sort_by(|left, right| left.id.cmp(&right.id));
    if normalized != contract.rules {
        return Err(CoreError::Config(
            "effective policy rules must be ordered by stable rule id".to_string(),
        ));
    }
    for pair in contract.rules.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(CoreError::Config(format!(
                "duplicate effective policy rule id {}",
                pair[0].id
            )));
        }
    }
    let computed = effective_policy_digest(&contract.rules)?;
    if computed != contract.digest {
        return Err(CoreError::Config(format!(
            "effective policy digest mismatch: computed {computed}, recorded {}",
            contract.digest
        )));
    }
    Ok(())
}

pub fn effective_policy_matches_policy(
    policy: &VerificationPolicy,
    contract: &EffectivePolicy,
) -> Result<bool, CoreError> {
    validate_effective_policy(contract)?;
    let expected = effective_policy(policy, &PolicyProvenance::default())?;
    Ok(expected.rules.len() == contract.rules.len()
        && expected
            .rules
            .iter()
            .zip(&contract.rules)
            .all(|(expected, recorded)| {
                expected.id == recorded.id
                    && expected.effective_value == recorded.effective_value
                    && expected.applicability == recorded.applicability
                    && expected.safety_class == recorded.safety_class
            }))
}

fn push_bool_rule(
    rules: &mut Vec<EffectivePolicyRule>,
    provenance: &PolicyProvenance,
    id: &str,
    value: bool,
    applicability: &str,
    safety_class: PolicySafetyClass,
) {
    rules.push(EffectivePolicyRule {
        id: id.to_string(),
        source: provenance.source_for(id),
        effective_value: Value::Bool(value),
        applicability: applicability.to_string(),
        safety_class,
    });
}

fn effective_policy_digest(rules: &[EffectivePolicyRule]) -> Result<String, CoreError> {
    #[derive(Serialize)]
    struct Subject<'a> {
        profile: &'a str,
        rules: &'a [EffectivePolicyRule],
    }
    Ok(codefire_util::sha256_hex(&serde_json::to_vec(&Subject {
        profile: EFFECTIVE_POLICY_PROFILE,
        rules,
    })?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        default_verification_policy, parse_effective_verification_policy, RequiredLinkRule,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn effective_policy_digest_is_independent_of_required_link_order() {
        let mut left = default_verification_policy();
        left.required_links.insert(
            "requirement".to_string(),
            vec![
                RequiredLinkRule {
                    link_type: "verified_by".to_string(),
                    target_kind: "test".to_string(),
                    min: 1,
                },
                RequiredLinkRule {
                    link_type: "refined_by".to_string(),
                    target_kind: "design".to_string(),
                    min: 1,
                },
            ],
        );
        let mut right = left.clone();
        right
            .required_links
            .get_mut("requirement")
            .unwrap()
            .reverse();
        let left = effective_policy(&left, &PolicyProvenance::default()).unwrap();
        let right = effective_policy(&right, &PolicyProvenance::default()).unwrap();
        assert_eq!(left.digest, right.digest);
        assert_eq!(left.rules, right.rules);
    }

    #[test]
    fn cli_override_has_precedence_over_repository_source() {
        let mut provenance = PolicyProvenance::default();
        provenance.record_repository_rule("commit.require_trace_completeness");
        provenance.record_cli_override("commit.require_trace_completeness");
        let contract = effective_policy(&default_verification_policy(), &provenance).unwrap();
        let rule = contract
            .rules
            .iter()
            .find(|rule| rule.id == "commit.require_trace_completeness")
            .unwrap();
        assert_eq!(rule.source, PolicyRuleSource::CliOverride);
    }

    #[test]
    fn parser_distinguishes_default_and_repository_rule_sources() {
        let temp = tempdir().unwrap();
        let default = parse_effective_verification_policy(temp.path()).unwrap();
        let default_rule = default
            .effective
            .rules
            .iter()
            .find(|rule| rule.id == "commit.require_trace_completeness")
            .unwrap();
        assert_eq!(default_rule.source, PolicyRuleSource::BuiltInDefault);
        assert_eq!(default_rule.effective_value, Value::Bool(true));

        fs::write(
            temp.path().join("codefire.policy.yaml"),
            "version: 1\ncommit_policy:\n  require_trace_completeness: false\n",
        )
        .unwrap();
        let repository = parse_effective_verification_policy(temp.path()).unwrap();
        let repository_rule = repository
            .effective
            .rules
            .iter()
            .find(|rule| rule.id == "commit.require_trace_completeness")
            .unwrap();
        assert_eq!(repository_rule.source, PolicyRuleSource::RepositoryConfig);
        assert_eq!(repository_rule.effective_value, Value::Bool(false));
        assert_ne!(default.effective.digest, repository.effective.digest);
    }
}
