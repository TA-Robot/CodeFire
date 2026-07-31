# CFR-168 Review Recurrence v2 Design Adjudication

Status: `PROCEED_WITH_MEASUREMENT_CORRECTIONS`; implementation remains
experimental and no live arm or integration is authorized by this decision.

## Exact subject

| Item | Identity |
|---|---|
| Accepted implementation contract | commit `41824afaace1d172ec12f26d98fa0ae306b4ddcd`; packet SHA-256 `e08e736cbdb7f48fc131ed2b542db951fbff32ebe1b29d399bfe83172bdea6f6` |
| Independent contract acceptance | canonical commit `1c8a95fcf46cd3d1cec91f83e1ede5761ceb36bc`; report SHA-256 `fdc0df8111da1bea50737563d5ed3dea1ed989f95c1c728f30fe406edfff336c` |
| Pre-implementation design checkpoint | commit `a86926751d929cb2c809a16eac65646f77a7bc53`; path `docs/development/ai/swarm-results/cfr168-review-recurrence-v2-repair-claude-20260731/design.md`; SHA-256 `949d4904a452e42cdf0a8a78ab156b2ce32085f53847c5e92b153cf72c3599b5` |

This is a manager adjudication of the design checkpoint's explicit burden
measurement and extension points. It does not amend the accepted packet or
pre-accept an implementation.

## Decision

Implementation may proceed because the minimal-record increase is caused by
exact Git artifact identity replacing free text, while the design's realistic
record shape is not larger than the published v1 comparison. This is only a
live-experiment hypothesis. It is not evidence that review lead time, author
burden, or rereview count improved.

The following corrections are mandatory in the implementation result and its
independent review:

1. Report required leaves for the **record and pre-repair manifest separately
   and together**. Do not compare a record-only v2 number with a v1 number while
   omitting a new separately authored manifest.
2. Split verification into at least:
   - structural/schema consistency;
   - Git/evidence identity binding;
   - semantic human judgement.
   Set equality with a manager-authored manifest proves consistency with that
   manifest, not completeness of its obligations, parity claims, validation
   selection, oracle quality, or closure meaning. Do not count those semantic
   decisions as machine-proved merely because their bytes are bound.
3. Retain the design's `109` minimal-record and approximately `301`
   realistic-record measurements as provisional. Recompute them from the final
   schema, add manifest leaves, and report author/manager/reviewer minutes,
   unavailable values, and the counting program or reproducible method.
4. The future live-arm anchor is manager-owned authority. It must be loaded
   from the clean committed **validator source repository**, not from the
   analysed candidate `--git-root`; the result must bind its validator commit,
   path, mode, and SHA-256. A missing anchor, a candidate-authored anchor, an
   uncommitted anchor, or an identity mismatch is `BLOCK`. No Cargo/KGI anchor
   is created in this implementation lane.
5. Preserve the disclosed second-generation sequencing blocker. A non-empty
   repair surface must correspond to real remaining work; never create an
   artificial byte change to exercise `ALLOW`.
6. Gate-identity differences across rooted historical links must remain
   explicit evidence. The implementation and reviewer must state whether each
   difference is blocked, compatibility-authorized, or retained without a
   semantic re-proof; an unknown gate identity may not silently inherit current
   authority.

## Required next steps

1. Claude Opus 5 completes the isolated v2 schemas, tools, tests, and docs in
   bounded commits.
2. The implementation result dispositions every F-1 through F-14, C-1 through
   C-16, R-1 through R-10, N-1 through N-6, and all six corrections above.
3. The manager integrates only a clean candidate that passes every command in
   the accepted packet.
4. A fresh Claude Opus 5 reviewer attacks the exact integrated candidate,
   including manifest-burden accounting, self-authored live anchors, gate
   identity migration, and all prior counterexamples.
5. Every review finding is returned to the original implementer session and
   independently rereviewed before live-arm preparation.
6. A separate manager packet later creates and independently preflights exact
   Cargo/KGI manifests and the validator-owned anchor.

## Adoption evidence still unavailable

- final v2 schemas, implementation, tests, result identity, and independent
  implementation verdict;
- final record-plus-manifest leaf counts and actual author/reviewer minutes;
- Cargo/KGI first and second v2 generations;
- later-finding capture, first-pass decision, rework, oracle failure,
  repair-introduced defect, and product-repair-generation results;
- Fugu blind classification scheduled on or after 2026-08-03; and
- Close, Install, Dogfood, Retrospect, and final KGI/KPI evidence.

Until those exist, the review bottleneck and the user's full completion goal
remain open.
