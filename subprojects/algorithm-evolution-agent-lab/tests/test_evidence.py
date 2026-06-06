import unittest

from evoagent.evidence import EvidenceLedger


# cf-atom: TEST-evidence-ledger-links-claims-to-runs
class EvidenceLedgerTests(unittest.TestCase):
    def test_evidence_ledger_links_claims_to_runs(self):
        ledger = EvidenceLedger()

        record = ledger.add(
            claim_id="claim-sample-efficiency",
            source_type="experiment_run",
            source_ref="run_123",
            summary="Candidate improved validation score by 0.03 over baseline.",
            confidence=0.8,
            tags=("tabular", "baseline-comparison"),
        )

        self.assertEqual(record.claim_id, "claim-sample-efficiency")
        self.assertEqual(ledger.for_claim("claim-sample-efficiency"), [record])
        self.assertEqual(ledger.claim_confidence("claim-sample-efficiency"), 0.8)

    def test_evidence_ledger_rejects_invalid_confidence(self):
        with self.assertRaises(ValueError):
            EvidenceLedger().add(
                claim_id="claim",
                source_type="experiment_run",
                source_ref="run_123",
                summary="bad confidence",
                confidence=1.5,
            )


if __name__ == "__main__":
    unittest.main()
