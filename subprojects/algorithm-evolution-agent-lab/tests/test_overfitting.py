import unittest

from evoagent.overfitting import AntiOverfittingMonitor, ProbeRecord


# cf-atom: TEST-anti-overfitting-monitor-limits-target-probes
class AntiOverfittingMonitorTests(unittest.TestCase):
    def test_anti_overfitting_monitor_limits_target_probes(self):
        probes = [
            ProbeRecord("hidden-leaderboard", used_validation_control=True),
            ProbeRecord("hidden-leaderboard", used_validation_control=True),
            ProbeRecord("hidden-leaderboard", used_validation_control=True),
        ]

        findings = AntiOverfittingMonitor().check(probes, max_probes_per_target=2)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].kind, "excessive_probe_count")
        self.assertEqual(findings[0].severity, "blocking")

    def test_anti_overfitting_monitor_flags_missing_validation_control(self):
        probes = [ProbeRecord("public-leaderboard", used_validation_control=False)]

        findings = AntiOverfittingMonitor().check(probes, max_probes_per_target=2)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].kind, "missing_validation_control")


if __name__ == "__main__":
    unittest.main()
