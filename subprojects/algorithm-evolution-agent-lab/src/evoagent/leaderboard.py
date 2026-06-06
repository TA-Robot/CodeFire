from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class LeaderboardSubmission:
    submission_id: str
    benchmark: str
    submitted_at_day: int
    local_evidence_refs: tuple[str, ...]


@dataclass(frozen=True)
class LeaderboardSubmissionFinding:
    kind: str
    detail: str


# cf-atom: CODE-LeaderboardSubmissionLedger
class LeaderboardSubmissionLedger:
    def __init__(self):
        self._submissions: list[LeaderboardSubmission] = []

    def add(self, submission: LeaderboardSubmission) -> LeaderboardSubmission:
        if any(existing.submission_id == submission.submission_id for existing in self._submissions):
            raise ValueError(f"duplicate submission_id: {submission.submission_id}")
        self._submissions.append(submission)
        return submission

    def findings(
        self,
        *,
        benchmark: str,
        max_submissions: int,
        cooldown_days: int,
        require_local_evidence: bool = True,
    ) -> tuple[LeaderboardSubmissionFinding, ...]:
        submissions = sorted(
            (submission for submission in self._submissions if submission.benchmark == benchmark),
            key=lambda submission: submission.submitted_at_day,
        )
        findings: list[LeaderboardSubmissionFinding] = []
        if len(submissions) > max_submissions:
            findings.append(
                LeaderboardSubmissionFinding(
                    "excessive_submissions",
                    f"{len(submissions)} submissions exceeds limit {max_submissions}",
                )
            )
        for previous, current in zip(submissions, submissions[1:]):
            if current.submitted_at_day - previous.submitted_at_day < cooldown_days:
                findings.append(
                    LeaderboardSubmissionFinding(
                        "cooldown_violation",
                        f"{current.submission_id} was submitted before cooldown elapsed",
                    )
                )
        if require_local_evidence:
            for submission in submissions:
                if not submission.local_evidence_refs:
                    findings.append(
                        LeaderboardSubmissionFinding(
                            "missing_local_evidence",
                            f"{submission.submission_id} has no local evidence refs",
                        )
                    )
        return tuple(findings)

    def for_benchmark(self, benchmark: str) -> tuple[LeaderboardSubmission, ...]:
        return tuple(submission for submission in self._submissions if submission.benchmark == benchmark)
