from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class TraceLink:
    source: str
    target: str
    link_type: str


@dataclass(frozen=True)
class TraceAuditFinding:
    kind: str
    atom_id: str
    detail: str


@dataclass(frozen=True)
class TraceAuditReport:
    requirements: tuple[str, ...]
    implementations: tuple[str, ...]
    tests: tuple[str, ...]
    findings: tuple[TraceAuditFinding, ...]

    @property
    def passed(self) -> bool:
        return not self.findings


# cf-atom: CODE-CodeFireTraceAudit
class CodeFireTraceAudit:
    def audit(self, links: tuple[TraceLink, ...]) -> TraceAuditReport:
        requirements = tuple(sorted({link.source for link in links if link.source.startswith("REQ-")}))
        implementations = tuple(sorted({link.target for link in links if link.target.startswith("CODE-")}))
        tests = tuple(sorted({link.target for link in links if link.target.startswith("TEST-")}))
        design_links = {link.source: link.target for link in links if link.source.startswith("DES-")}
        findings: list[TraceAuditFinding] = []

        for requirement in requirements:
            targets = tuple(link.target for link in links if link.source == requirement)
            if not reaches_prefix(targets, design_links, "CODE-"):
                findings.append(
                    TraceAuditFinding(
                        "missing_implementation",
                        requirement,
                        "requirement has no reachable CODE atom",
                    )
                )
            if not any(target.startswith("TEST-") for target in targets):
                findings.append(
                    TraceAuditFinding(
                        "missing_test",
                        requirement,
                        "requirement has no direct TEST atom",
                    )
                )

        for link in links:
            if not link.source or not link.target or not link.link_type:
                findings.append(TraceAuditFinding("malformed_link", link.source or "<empty>", "link is incomplete"))

        return TraceAuditReport(
            requirements=requirements,
            implementations=implementations,
            tests=tests,
            findings=tuple(findings),
        )


def reaches_prefix(targets: tuple[str, ...], design_links: dict[str, str], prefix: str) -> bool:
    for target in targets:
        if target.startswith(prefix):
            return True
        if target.startswith("DES-") and design_links.get(target, "").startswith(prefix):
            return True
    return False
