# CFR-168 Python HTTP Upload + Scale Combined Review

Independently review the complete Python HTTP upload ancestry/signature candidate together with the
CFB-138 default-profile scale correction. Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). This is
review-only: do not modify product code, tests, issue state, or management ledgers.

## Exact subject

- upload recovery result: `cb13da08e82c2502920f735d27617e61f978527a`
- CFB-138 task base: `88df84b7fd962cd33f4c897f555d083bc2e58677`
- completed scale target: `dece688afea815d1242e7751367052d1ceac4580`
- target branch:
  `claude-agent/cfr168-python-http-upload-staging-scale-20260730-upload-staging-scale`
- cumulative paths: `codefire`, `tests/test_python_http_upload_accounted_traversal.py`
- contracts:
  `docs/development/ai/swarm-batches/cfr168-python-http-upload-traversal-canonical-20260729.md`
  and
  `docs/development/ai/swarm-batches/cfr168-python-http-upload-staging-scale-20260730.md`

The earlier reviewer report at `985ca9e4bd376614f1510c62100b86af7c640739` is evidence, not an
authoritative verdict for this corrected target. Reproduce or reject each finding independently.
For this review, a live-path defect is not dismissed merely because it is byte-identical at an
earlier baseline: CFR-168 owns the complete request decision-work invariant, not only lines newly
introduced by one recovery commit.

## Required review

1. One exact HTTP request ledger/deadline must own upload body, staging, sealed/signature/ancestry
   validation, publication identity/preflight, response bytes/headers, final fence, effect, replay,
   and cleanup. Reproduce whether nested legacy `validate_sealed_commit` traversals or independent
   local ledgers remain on the live route.
2. `_AccountedUploadCommitSource` and every validation session/list/dict/set/FD owner must reserve
   source-specific retained/temporary capacity before construction or insertion. Reproduce the
   earlier zero-outstanding-reservation allocation probe on the corrected target.
3. Prove `concurrent_retained_upper` is a real upper bound for simultaneous owner lifetime:
   retained canonical documents once, every live traversal frame, one maximum raw/decode record,
   and fixed control/fingerprint terms. No valid staged record shape may make `_claim_memory`
   exceed a supposedly admitted reservation.
4. Reproduce default `http_v1` 1x/3x/10x curves, the 1201-commit reversal, exact/minus-one/plus-one,
   claimed peak, zero debt, unchanged authority, and no raised public limit. Counter slopes and
   diagnostic precedence must be deterministic.
5. Publication identity, preflight, final deadline/subject fence, and durable result selection must
   all occur before the first externally visible effect. Reproduce whether deadline expiry between
   redundant validation/publication phases can still publish.
6. Verify signature policy, fast-forward/non-fast-forward, malformed/corrupt/missing history,
   replay/nonce, response bytes, atomicity, and neighboring Doctor/branch behavior. Run the 13
   focused tests, relevant legacy cases, Python compilation, docs checks, and `git diff --check`.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-upload-scale-combined-review-20260730/review.md`

Include exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, explicit disposition of every earlier finding, verdicts for all six items,
commands/results, and a minimal repair ordering if rejected. Commit only the report and leave the
reviewer worktree clean.
