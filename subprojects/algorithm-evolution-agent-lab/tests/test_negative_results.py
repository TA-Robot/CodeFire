import unittest

from evoagent.negative_results import NegativeResultArchive


# cf-atom: TEST-negative-result-archive-retains-failed-paths
class NegativeResultArchiveTests(unittest.TestCase):
    def test_negative_result_archive_retains_failed_paths(self):
        archive = NegativeResultArchive()
        result = archive.add(
            hypothesis_id="hyp-1",
            failure_mode="instability",
            summary="high variance across seeds erased the apparent gain",
            avoid_conditions=("single-seed promotion", "unregularized feature filter"),
            evidence_refs=("run-1",),
        )

        self.assertEqual(archive.by_failure_mode("instability"), (result,))
        self.assertTrue(archive.should_avoid("single-seed"))
        self.assertFalse(archive.should_avoid("calibrated ensemble"))

    def test_negative_result_archive_requires_identity(self):
        with self.assertRaises(ValueError):
            NegativeResultArchive().add(
                hypothesis_id="",
                failure_mode="instability",
                summary="missing id",
                avoid_conditions=(),
            )


if __name__ == "__main__":
    unittest.main()
