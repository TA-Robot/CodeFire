from __future__ import annotations

import subprocess
from pathlib import Path

from evoagent.models import ExperimentGoal, Hypothesis
from evoagent.planner import parse_hypotheses_json


# cf-atom: CODE-CodexPlanner
class CodexPlanner:
    def __init__(
        self,
        *,
        command: tuple[str, ...] = ("codex", "exec"),
        model: str | None = None,
        timeout_seconds: float = 120.0,
        cwd: Path | None = None,
    ):
        self.command = command
        self.model = model
        self.timeout_seconds = timeout_seconds
        self.cwd = cwd

    def propose(self, goal: ExperimentGoal, count: int = 3) -> list[Hypothesis]:
        prompt = self.build_prompt(goal, count)
        output = self.run_codex(prompt)
        return parse_hypotheses_json(output)

    def build_prompt(self, goal: ExperimentGoal, count: int) -> str:
        return (
            "You are a machine learning research planner. "
            "Propose candidate algorithm hypotheses for an experiment automation agent.\n\n"
            "Return only JSON with this shape:\n"
            '{"hypotheses":[{"title":"...","rationale":"...","expected_gain":0.05,"novelty":0.7}]}\n\n'
            f"Goal: {goal.objective}\n"
            f"Target metric: {goal.target_metric}\n"
            f"Maximize metric: {goal.maximize}\n"
            f"Budget hours: {goal.budget_hours}\n"
            f"Number of hypotheses: {count}\n\n"
            "Rules:\n"
            "- expected_gain and novelty must be numbers between 0 and 1.\n"
            "- Prefer ideas that can be tested with small, reproducible experiments.\n"
            "- Do not claim SOTA; these are hypotheses only.\n"
        )

    def run_codex(self, prompt: str) -> str:
        command = list(self.command)
        if self.model:
            command.extend(["--model", self.model])
        command.extend(
            [
                "--skip-git-repo-check",
                "--ephemeral",
                "--sandbox",
                "read-only",
                "-",
            ]
        )
        proc = subprocess.run(
            command,
            input=prompt,
            cwd=self.cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=self.timeout_seconds,
            check=False,
        )
        if proc.returncode != 0:
            raise RuntimeError(f"codex planner failed: {proc.stderr.strip() or proc.stdout.strip()}")
        return proc.stdout
