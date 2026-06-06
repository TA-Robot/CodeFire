import unittest

from evoagent.review import ReviewLedger


# cf-atom: TEST-review-ledger-tracks-open-objections
class ReviewLedgerTests(unittest.TestCase):
    def test_review_ledger_tracks_open_objections(self):
        ledger = ReviewLedger()
        objection = ledger.add_objection(
            claim_id="claim-sota",
            summary="baseline comparison may be too weak",
            severity="blocking",
            evidence_refs=("evidence-1",),
        )

        self.assertEqual(ledger.open_objections("claim-sota"), (objection,))
        self.assertEqual(ledger.blocking_objections("claim-sota"), (objection,))

        resolved = ledger.resolve(objection.objection_id, resolution="Added reproduced external baseline comparison.")

        self.assertTrue(resolved.resolved)
        self.assertEqual(ledger.open_objections("claim-sota"), ())
        self.assertEqual(ledger.blocking_objections("claim-sota"), ())

    def test_review_ledger_rejects_unknown_severity(self):
        with self.assertRaises(ValueError):
            ReviewLedger().add_objection(claim_id="claim", summary="bad severity", severity="urgent")


if __name__ == "__main__":
    unittest.main()
