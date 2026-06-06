import unittest

from evoagent.leaderboard import LeaderboardSubmission, LeaderboardSubmissionLedger


# cf-atom: TEST-leaderboard-submission-ledger-flags-excessive-probing
class LeaderboardSubmissionLedgerTests(unittest.TestCase):
    def test_leaderboard_submission_ledger_flags_excessive_probing(self):
        ledger = LeaderboardSubmissionLedger()
        ledger.add(LeaderboardSubmission("s1", "hidden-tabular", 1, ("ev_1",)))
        ledger.add(LeaderboardSubmission("s2", "hidden-tabular", 2, ()))
        ledger.add(LeaderboardSubmission("s3", "hidden-tabular", 10, ("ev_2",)))

        findings = ledger.findings(
            benchmark="hidden-tabular",
            max_submissions=2,
            cooldown_days=3,
        )

        self.assertEqual(
            {finding.kind for finding in findings},
            {"excessive_submissions", "cooldown_violation", "missing_local_evidence"},
        )


if __name__ == "__main__":
    unittest.main()
