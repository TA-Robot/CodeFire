import unittest

from evoagent.overrides import HumanOverrideLedger


# cf-atom: TEST-human-override-ledger-preserves-audit-trail
class HumanOverrideLedgerTests(unittest.TestCase):
    def test_human_override_ledger_preserves_audit_trail(self):
        ledger = HumanOverrideLedger()
        freeze = ledger.add(
            target_id="hypothesis-1",
            action="freeze",
            rationale="Pause this direction until baseline reproduction is complete.",
            operator="researcher",
            evidence_refs=("evidence-1",),
        )
        priority = ledger.add(
            target_id="candidate-2",
            action="priority_override",
            rationale="Run this cheap uncertainty reducer before the expensive candidate.",
            operator="researcher",
            priority=0.9,
        )

        self.assertEqual(ledger.for_target("hypothesis-1"), (freeze,))
        self.assertEqual(ledger.active_freezes(), (freeze,))
        self.assertEqual(ledger.all(), (freeze, priority))

    def test_human_override_ledger_requires_rationale(self):
        with self.assertRaises(ValueError):
            HumanOverrideLedger().add(
                target_id="candidate",
                action="reject",
                rationale="",
                operator="researcher",
            )


if __name__ == "__main__":
    unittest.main()
