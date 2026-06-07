import unittest

from evoagent.risk import ExperimentRisk, ExperimentRiskRegister, RiskSeverity, RiskStatus


# cf-atom: TEST-experiment-risk-register-prioritizes-open-risks
class ExperimentRiskRegisterTests(unittest.TestCase):
    def test_experiment_risk_register_prioritizes_open_risks(self):
        register = ExperimentRiskRegister()
        risks = (
            ExperimentRisk(
                risk_id="closed",
                plan_id="p0",
                category="reporting",
                severity=RiskSeverity.CRITICAL,
                probability=1.0,
                impact=10.0,
                status=RiskStatus.CLOSED,
                mitigation="already handled",
            ),
            ExperimentRisk(
                risk_id="leakage",
                plan_id="p1",
                category="evaluation",
                severity=RiskSeverity.CRITICAL,
                probability=0.6,
                impact=0.9,
                mitigation="audit dataset overlap before promotion",
            ),
            ExperimentRisk(
                risk_id="compute",
                plan_id="p2",
                category="budget",
                severity=RiskSeverity.MEDIUM,
                probability=0.9,
                impact=0.5,
                mitigation="cap repeated runs",
            ),
        )

        prioritized = register.prioritize(risks)
        self.assertEqual(tuple(item.risk_id for item in prioritized), ("leakage", "compute"))
        self.assertEqual(tuple(item.risk_id for item in register.blockers(risks)), ("leakage",))
        self.assertEqual(
            register.mitigation_checklist(risks),
            ("audit dataset overlap before promotion", "cap repeated runs"),
        )

    def test_experiment_risk_register_validates_probability(self):
        register = ExperimentRiskRegister()

        with self.assertRaises(ValueError):
            register.prioritize(
                (
                    ExperimentRisk(
                        risk_id="bad",
                        plan_id="p1",
                        category="evaluation",
                        severity=RiskSeverity.HIGH,
                        probability=1.5,
                        impact=1.0,
                    ),
                )
            )


if __name__ == "__main__":
    unittest.main()
