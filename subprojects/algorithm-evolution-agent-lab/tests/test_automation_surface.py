import unittest

from evoagent.automation_surface import ExperimentAutomationSurfaceGuard, ProductSurface, ProductWorkItem


# cf-atom: TEST-experiment-automation-surface-guard-prioritizes-automation
class ExperimentAutomationSurfaceGuardTests(unittest.TestCase):
    def test_experiment_automation_surface_guard_prioritizes_automation(self):
        automation = ProductWorkItem("a", "queue runner", ProductSurface.EXPERIMENT_AUTOMATION, True, 0.4, 0.2)
        model_idea = ProductWorkItem("b", "new layer", ProductSurface.MODEL_IDEA, False, 10.0, 1.0)
        reporting = ProductWorkItem("c", "report", ProductSurface.REPORTING, True, 0.3, 0.3)

        guard = ExperimentAutomationSurfaceGuard()
        findings = guard.findings((model_idea,))

        self.assertEqual(guard.prioritize((model_idea, reporting, automation))[0], automation)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].kind, "missing_experiment_plan")


if __name__ == "__main__":
    unittest.main()
