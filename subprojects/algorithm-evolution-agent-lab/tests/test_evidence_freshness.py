import unittest

from evoagent.evidence import EvidenceRecord
from evoagent.evidence_freshness import EvidenceFreshnessAudit, VersionedEvidenceRef


# cf-atom: TEST-evidence-freshness-audit-flags-stale-versioned-evidence
class EvidenceFreshnessAuditTests(unittest.TestCase):
    def test_evidence_freshness_audit_flags_stale_versioned_evidence(self):
        evidence = EvidenceRecord(
            evidence_id="ev_1",
            claim_id="claim",
            source_type="experiment_run",
            source_ref="run_1",
            summary="old benchmark run",
            confidence=0.8,
            created_at="2026-06-05T00:00:00Z",
        )
        ref = VersionedEvidenceRef(
            evidence=evidence,
            benchmark_version="bench-v1",
            dataset_version="data-v1",
            protocol_version="protocol-v1",
        )

        findings = EvidenceFreshnessAudit().audit(
            (ref,),
            current_benchmark_version="bench-v2",
            current_dataset_version="data-v1",
            current_protocol_version="protocol-v2",
        )

        self.assertEqual({finding.kind for finding in findings}, {"stale_benchmark", "stale_protocol"})


if __name__ == "__main__":
    unittest.main()
