import unittest

from evoagent.runlabel import RunLabeler


# cf-atom: TEST-run-labeler-formats-label
class RunLabelerTests(unittest.TestCase):
    def test_run_labeler_pads_seed(self):
        self.assertEqual(RunLabeler.format_run_label("exp", 7), "exp-seed0007")
        self.assertEqual(RunLabeler.format_run_label("exp", 12345), "exp-seed12345")


if __name__ == "__main__":
    unittest.main()
