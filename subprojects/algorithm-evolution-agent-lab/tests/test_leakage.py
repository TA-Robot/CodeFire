import unittest

from evoagent.leakage import DatasetSplitAudit, LeakageChecker


# cf-atom: TEST-leakage-checker-blocks-split-overlap
class LeakageCheckerTests(unittest.TestCase):
    def test_leakage_checker_blocks_split_overlap(self):
        audit = DatasetSplitAudit(
            train_ids=frozenset({"a", "b", "c"}),
            validation_ids=frozenset({"c", "d"}),
            test_ids=frozenset({"e"}),
            observed_metric="validation_score",
            expected_metric="validation_score",
        )

        findings = LeakageChecker().check(audit)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].kind, "split_overlap")
        self.assertEqual(findings[0].severity, "blocking")

    def test_leakage_checker_blocks_metric_misuse(self):
        audit = DatasetSplitAudit(
            train_ids=frozenset({"a"}),
            validation_ids=frozenset({"b"}),
            test_ids=frozenset({"c"}),
            observed_metric="test_score",
            expected_metric="validation_score",
        )

        findings = LeakageChecker().check(audit)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].kind, "metric_misuse")


if __name__ == "__main__":
    unittest.main()
