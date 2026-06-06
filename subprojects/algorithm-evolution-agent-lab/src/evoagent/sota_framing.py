from __future__ import annotations

from dataclasses import dataclass

from evoagent.claims import ClaimDisciplineClassifier, ClaimType
from evoagent.safety import ClaimEvidenceSummary, ClaimReadiness, claim_readiness


@dataclass(frozen=True)
class SotaPursuitFrame:
    label: str
    claim_type: ClaimType
    readiness: ClaimReadiness
    external_claim_allowed: bool
    missing_evidence: tuple[str, ...]
    recommended_wording: str


# cf-atom: CODE-SotaPursuitFramer
class SotaPursuitFramer:
    def frame(
        self,
        *,
        requested_claim: ClaimType,
        evidence_tokens: tuple[str, ...],
        evidence_summary: ClaimEvidenceSummary,
    ) -> SotaPursuitFrame:
        discipline = ClaimDisciplineClassifier().classify(requested_claim, evidence_tokens)
        readiness = claim_readiness(evidence_summary)
        external_allowed = (
            requested_claim == ClaimType.EXTERNAL_SOTA
            and discipline.allowed
            and readiness == ClaimReadiness.EXTERNAL_SOTA_CLAIM
        )
        label = "evidence_backed_claim" if external_allowed else "hypothesis"
        wording = (
            "External SOTA claim is evidence-backed."
            if external_allowed
            else "Treat as a SOTA-seeking hypothesis until benchmark, baseline, and reproducibility evidence mature."
        )
        return SotaPursuitFrame(
            label=label,
            claim_type=requested_claim,
            readiness=readiness,
            external_claim_allowed=external_allowed,
            missing_evidence=discipline.missing_evidence,
            recommended_wording=wording,
        )
