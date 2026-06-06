from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from evoagent.challenge import SotaChallengeDefinition
from evoagent.program import ResearchProgram
from evoagent.reporting import LivingResearchReport, ReportOptions


@dataclass(frozen=True)
class SotaReportBundleManifest:
    bundle_dir: Path
    files: tuple[Path, ...]


# cf-atom: CODE-SotaReportBundle
class SotaReportBundleGenerator:
    def __init__(self, report: LivingResearchReport | None = None):
        self.report = report or LivingResearchReport(ReportOptions(title="SOTA Challenge Report"))

    def write(
        self,
        *,
        program: ResearchProgram,
        challenge: SotaChallengeDefinition,
        output_dir: str | Path,
    ) -> SotaReportBundleManifest:
        bundle_dir = Path(output_dir)
        bundle_dir.mkdir(parents=True, exist_ok=True)

        files = (
            write_text(bundle_dir / "report.md", self.report.render(program)),
            write_text(bundle_dir / "challenge.md", render_challenge(challenge)),
            write_text(bundle_dir / "reproduction.md", render_reproduction(program, challenge)),
            write_text(bundle_dir / "limitations.md", render_limitations(program, challenge)),
        )
        manifest = SotaReportBundleManifest(bundle_dir=bundle_dir, files=files)
        write_text(bundle_dir / "manifest.json", render_manifest(manifest, challenge))
        return SotaReportBundleManifest(bundle_dir=bundle_dir, files=files + (bundle_dir / "manifest.json",))


def render_challenge(challenge: SotaChallengeDefinition) -> str:
    lines = [
        f"# Challenge: {challenge.challenge_id}",
        "",
        f"- Task: {challenge.task}",
        f"- Dataset: {challenge.dataset}",
        f"- Metric: {challenge.metric}",
        f"- Split: {challenge.split}",
        f"- Target improvement: {challenge.target_improvement:.4f}",
        f"- Compute: {challenge.allowed_compute.max_hours:g} hours, {challenge.allowed_compute.max_trials} trials",
        "",
        "## Baselines",
        "",
        "| Baseline | Metric | Source |",
        "|---|---:|---|",
    ]
    for baseline in challenge.baselines:
        lines.append(f"| {baseline.name} | {baseline.metric_value:.4f} | {baseline.source} |")
    return "\n".join(lines) + "\n"


def render_reproduction(program: ResearchProgram, challenge: SotaChallengeDefinition) -> str:
    commands = [candidate.plan.command for candidate in program.candidates if candidate.plan.command]
    lines = [
        f"# Reproduction: {challenge.challenge_id}",
        "",
        "## Budget",
        "",
        f"- Max hours: {challenge.allowed_compute.max_hours:g}",
        f"- Max trials: {challenge.allowed_compute.max_trials}",
        "",
        "## Commands",
        "",
    ]
    if commands:
        lines.extend(f"```bash\n{command}\n```" for command in commands)
    else:
        lines.append("No runnable candidate commands recorded yet.")
    return "\n".join(lines) + "\n"


def render_limitations(program: ResearchProgram, challenge: SotaChallengeDefinition) -> str:
    failed_runs = [run for run in program.runs if not run.succeeded]
    low_confidence = [record for record in program.evidence.all() if record.confidence < 0.5]
    lines = [
        f"# Limitations: {challenge.challenge_id}",
        "",
        "## Disallowed Shortcuts",
        "",
        *[f"- {shortcut}" for shortcut in challenge.disallowed_shortcuts],
        "",
        "## Current Gaps",
        "",
    ]
    if failed_runs:
        lines.append(f"- {len(failed_runs)} failed run(s) require analysis before promotion.")
    if low_confidence:
        lines.append(f"- {len(low_confidence)} low-confidence evidence record(s) require replication.")
    if not failed_runs and not low_confidence:
        lines.append("- No failed runs or low-confidence evidence records are currently recorded.")
    return "\n".join(lines) + "\n"


def render_manifest(manifest: SotaReportBundleManifest, challenge: SotaChallengeDefinition) -> str:
    data = {
        "challenge_id": challenge.challenge_id,
        "files": [path.name for path in manifest.files] + ["manifest.json"],
    }
    return json.dumps(data, indent=2, sort_keys=True) + "\n"


def write_text(path: Path, text: str) -> Path:
    path.write_text(text, encoding="utf-8")
    return path
