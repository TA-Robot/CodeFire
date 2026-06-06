from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ProbeRecord:
    target: str
    used_validation_control: bool
    notes: str = ""


@dataclass(frozen=True)
class OverfittingFinding:
    target: str
    kind: str
    severity: str
    detail: str


# cf-atom: CODE-AntiOverfittingMonitor
class AntiOverfittingMonitor:
    def check(self, probes: list[ProbeRecord], *, max_probes_per_target: int) -> list[OverfittingFinding]:
        if max_probes_per_target <= 0:
            raise ValueError("max_probes_per_target must be positive")
        findings: list[OverfittingFinding] = []
        by_target: dict[str, list[ProbeRecord]] = {}
        for probe in probes:
            by_target.setdefault(probe.target, []).append(probe)

        for target, target_probes in by_target.items():
            if len(target_probes) > max_probes_per_target:
                findings.append(
                    OverfittingFinding(
                        target=target,
                        kind="excessive_probe_count",
                        severity="blocking",
                        detail=f"{len(target_probes)} probe(s) exceeds limit {max_probes_per_target}",
                    )
                )
            uncontrolled = [probe for probe in target_probes if not probe.used_validation_control]
            if uncontrolled:
                findings.append(
                    OverfittingFinding(
                        target=target,
                        kind="missing_validation_control",
                        severity="high",
                        detail=f"{len(uncontrolled)} probe(s) used target feedback without validation control",
                    )
                )
        return findings
