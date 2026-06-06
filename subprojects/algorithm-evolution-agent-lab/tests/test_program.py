import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentPlan, Hypothesis
from evoagent.program import DecisionRecordValidator, ResearchProgram
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-research-program-records-runs-and-evidence
class ResearchProgramTests(unittest.TestCase):
    def test_research_program_records_runs_and_evidence(self):
        program = ResearchProgram(ExperimentGoal("Improve sample efficiency"))
        hypothesis = Hypothesis("stability filter", "reduce noisy feature choices", 0.04, 0.6)
        plan = ExperimentPlan(
            hypothesis,
            "toy-tabular",
            "baseline",
            "accuracy",
            0.1,
            artifact_paths=("metrics.json",),
        )
        candidate = CandidateAlgorithm("candidate-1", hypothesis, plan)
        program.add_candidate(candidate)

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=f"{sys.executable} -c \"from pathlib import Path; Path('metrics.json').write_text('{{}}')\"",
            )

        evidence = program.add_run_evidence(
            claim_id="claim-sample-efficiency",
            run=run,
            summary="Initial toy run completed.",
            confidence=0.4,
        )
        decision = program.record_decision(
            summary="Keep candidate for one more probe",
            rationale="Evidence is weak but run completed cleanly.",
            evidence_refs=(evidence.evidence_id,),
            rejected_alternatives=("reject immediately",),
            follow_up_work=("run another seed",),
        )

        self.assertEqual(program.hypotheses, [hypothesis])
        self.assertEqual(program.candidates, [candidate])
        self.assertEqual(program.runs, [run])
        self.assertEqual(program.evidence.for_claim("claim-sample-efficiency"), [evidence])
        self.assertEqual(decision.evidence_refs, (evidence.evidence_id,))
        self.assertEqual(DecisionRecordValidator().validate(decision), ())

    def test_decision_record_validator_requires_evidence_alternatives_and_followup(self):
        program = ResearchProgram(ExperimentGoal("Improve sample efficiency"))

        decision = program.record_decision(summary="Promote candidate", rationale="Single run looked good.")
        findings = DecisionRecordValidator().validate(decision)

        self.assertEqual(
            {finding.kind for finding in findings},
            {"missing_evidence", "missing_rejected_alternatives", "missing_follow_up"},
        )


if __name__ == "__main__":
    unittest.main()
