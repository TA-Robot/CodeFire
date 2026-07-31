# CFR-168 / CFB-138 Python HTTP Upload Review-Closure Repair

Status: ready for exact original-implementer continuation

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: implementation repair

Resume only the original implementer session
`7baa1275-0ee2-476a-af46-1907386ebceb` in its existing isolated worktree.
Do not start a replacement implementation lane while that exact session and
worktree remain recoverable.

## Exact immutable subject

| Item | Exact value |
|---|---|
| Rejected product candidate | `76baff84cb1192e11b26dcc669fb800f7b13fa30` |
| Candidate parent | `f6798cbfffda6ae181e934210a465ef7a7373539` |
| Candidate tree | `fcc65a4729f5029556e0266971d94b6321421f09` |
| Candidate repair-range base | `5f798990c313ea2e1b117bc74824de3f3ce028d2` |
| Candidate binary/full-index patch SHA-256 | `0e0a0c35ae8c7e9852775199ade4172b737e6797bcbcc2425b2baba29aadd028` |
| Independent-review taskbase | `de63a148c199ade7a05d3d7b68b60ef6af0d576a` |
| Independent-review commit | `35983261e1abd32045e1ff6cbd7dd7dfff5e806d` |
| Independent-review tree | `c95ec4ef04cbcc1da8bbd8a6762807b8555783ca` |
| Canonical report commit | `f835f64abb50fecaea28673ffa5dd829f1d3fc70` |
| Candidate-rooted report copy | `fe7cf94388393bd9b0457564d0116406f2c8844e` |
| Report SHA-256 | `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` |
| Review binary/full-index patch SHA-256 | `30529b8c321329323bfb30799a129905de679206742e8cde1f6778116dc4aa0` |
| Review verdict | `REQUEST_CHANGES` |
| Original implementer session | `7baa1275-0ee2-476a-af46-1907386ebceb` |

The immutable report is already present on this product lineage at:

`docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-rereview-fugu-20260730/review.md`

Its Git blob is `1fd3de9b48e04d29b56f36d79584a0a03c22bcff`. Verify the
report, commit, tree, and SHA-256 before changing product bytes. The supplied
review patch digest is defined by:

```bash
git diff --binary --full-index --no-ext-diff \
  de63a148c199ade7a05d3d7b68b60ef6af0d576a \
  35983261e1abd32045e1ff6cbd7dd7dfff5e806d | sha256sum
```

The reviewed candidate and report are immutable inputs. Add repair commits
above this packet commit. Do not amend, rebase, squash, reset, merge, replace,
or rewrite `76baff84`, `fe7cf943`, the report, or this packet.

## Objective

Close all four independent-review findings without weakening the accepted
ordinary-upload behavior:

- the default profile admits the proved record sizes and count ceiling;
- the 1,201-commit divergent history reaches the exact 409 non-fast-forward
  decision;
- normal HTTP upload uses one request-owned sealed snapshot and no foreign
  sealed-validation ledger;
- the canonical streaming encoder remains byte-identical and bounded;
- the existing-project and missing-project final deadline fences remain
  pre-effect;
- descriptor registry/reuse bounds, detached-document fingerprint release,
  exact response bytes, nonce/idempotency precedence, and zero terminal
  ownership remain intact.

Do not redesign the already accepted streaming, formula, descriptor, or
ordinary-upload portions merely because this is a continuation. Repair the
four exact gaps below and any newly discovered route-local consequence of
those changes.

## Required closure

### BLOCKER-1 — completed-quarantine recovery must retain the request ledger and deadline

The exact HTTP upload retry currently reaches:

```text
handle_http_upload
  -> execute_remote_json_publication
  -> recover_remote_pending_quarantine
  -> validate_sealed_commit
```

The last call receives neither the upload request's `DecisionWorkLedger` nor
its `HTTPRequestDeadline`, creates a fresh `local_v1()` `local_graph` ledger,
and can remove the completed quarantine and select 200 after the request
deadline has expired.

Repair this boundary so:

1. `handle_http_upload` transfers the exact request ledger and deadline
   through `execute_remote_json_publication` and
   `recover_remote_pending_quarantine` for an upload-owned recovery.
2. Every recovered durable root is validated on that same ledger and deadline
   with an exact upload-recovery phase. No fresh `sealed validation` ledger is
   minted inside the HTTP request.
3. Poll the request deadline before each recovered-root validation and again
   immediately before quarantine cleanup, receipt/recovery transition, or
   response selection. No fallible or input-proportional gap may follow the
   last poll and precede that effect.
4. A clock advance from 0.0 seconds to 1.0 seconds against a 100 ms request
   returns the exact typed deadline response. The pre-call objects, branches,
   generations, nonce, pending transaction, receipt, audit, quarantine, and
   response authority remain byte-identical. In particular, the completed
   quarantine is not removed.
5. A live exact retry still validates each root once, clears only the intended
   completed quarantine, returns the original byte-exact 200 result, and
   preserves idempotent replay.
6. Success and every failure end with the request ledger at zero current
   retained/temporary debt, zero permits, and one supported terminal
   observation. Proving a separately minted ledger also reaches zero is not
   closure.

Add deterministic failures before and after recovered-root validation, at the
last deadline poll, and immediately before cleanup/response publication.
Prove primary error precedence and retryability at each state.

### HIGH-2 — caller-owned sealed-validation APIs must be usable

`_AccountedUploadSourceMemory` currently reserves retained control ownership
but zero `temporary_bytes`. `SealedCommitValidationSession.finalize` then
correctly claims an ordered object-ID projection before list allocation and
sort, so every caller-ledger finalization rejects at that claim.

Repair the owner formula and transition so:

1. The exact ordered-projection temporary upper is reserved or acquired on
   the caller's ledger before allocating the list or sort scratch.
2. The upper is derived from the actual prepared document count, object-ID
   bytes, collection growth, and simultaneously live sort/projection state.
   Do not add a blanket profile-sized reservation.
3. Publication, consumption, cancellation, failed acquisition, and retryable
   release preserve one owner and cannot double-charge or detach debt.
4. All three direct paths complete with a caller ledger and with a caller
   ledger plus deadline:
   - `validate_sealed_commit`;
   - `validate_remote_json_publication_identity`; and
   - `preflight_remote_json_publication`.
5. Cover empty, one-record, and multi-record closures; exact and
   exact-minus-one temporary capacity; constructor/list/sort/deadline faults;
   release failure and retry; foreign-ledger cleanup; and zero debt/permits on
   every exit.
6. Preserve the no-ledger compatibility path and its existing error
   precedence. Do not make the ordinary upload allocate or traverse its
   sealed closure a second time.

Record the exact old and new retained/temporary formulas, simultaneous peaks,
and the observed minimum projection ownership. The review measured 56 bytes
for an empty projection, 64 bytes for one record, and 272 bytes for its
27-record fixture; recompute rather than hard-code those runtime observations
as a universal formula.

### HIGH-3 — `RemoteJSONPublicationRootProof` must be atomically single-use

The current proof rejects revocation, a foreign object store, an uncovered
root, and an expired deadline, but accepts two successful calls to
`validated_roots` before revocation.

Implement an explicit single-use state transition:

1. A successful consumption changes `available -> consumed` before returning
   the borrowed/validated roots.
2. An alias, callback reentry, or ordinary second call cannot consume the same
   proof again and receives the exact typed state error.
3. A failed deadline, object-store, root-coverage, or identity check must have
   an explicitly tested state disposition. Preserve the existing diagnostic
   precedence and allow retry only where the contract says no consumption
   occurred.
4. `revoke` before use, revoke after use, repeated revoke, cleanup failure,
   and owner drop release every resource exactly once.
5. Normal upload still consumes the proof once, performs one sealed traversal,
   and later revokes/cleans it without changing wire bytes or publication
   ordering.

Add direct alias, callback-reentry, second-use, failure-before-consumption,
failure-after-transition, revoke-before-use, revoke-after-use, stale snapshot,
foreign store/root, and construction-then-deadline-expiry tests.

### MEDIUM-4 — mandatory reversal evidence must become non-vacuous

The product contains the ordered-projection claim and
`_assert_prepared_coverage`, but the review proved that either can be deleted
while both the focused 22-test suite and broader 33-owner suite remain green.
The distinct layout-bootstrap deadline poll also lacks its required removal
mutation.

Add focused production-path oracles that fail for the intended reason when:

1. the ordered-projection temporary claim is removed while list allocation
   and sorting remain;
2. `_assert_prepared_coverage` is replaced by a no-op;
3. any newly relied-on prepared-coverage predicate is removed independently;
4. the `upload layout bootstrap` deadline poll is removed while the
   publication-effect poll remains; and
5. the BLOCKER-1 request-ledger/deadline handoff is dropped from completed-
   quarantine recovery.

The projection oracle must assert acquisition-before-allocation, the exact
dimension/formula, peak use, release, and terminal zero. The coverage oracle
must compare production high-water/use to the prepared upper; a forced raise
only proves reachability and is not sufficient. The layout-poll mutation must
create a behavioral difference—layout/effect or response—not a missing symbol,
fixture import failure, line-number mismatch, or helper-only counter change.

Run every deliberate regression against exact rejected candidate `76baff84`
or an exact repaired-tree mutation as appropriate. Preserve the portable
deadline reversal against `dece688afea815d1242e7751367052d1ceac4580`,
but do not count failures caused only by candidate-only symbols as old-target
evidence.

## Required state, formula, and call-site evidence

For the repaired route, provide:

- the exact ordinary and completed-quarantine HTTP call chains;
- every ledger/deadline transfer and proof/session state transition;
- retained and temporary owner formulas, acquisition order, simultaneous
  peaks, exact/minus-one observations, and terminal releases;
- root traversal, physical read, descriptor registry/reuse, canonical byte,
  graph node/edge/frontier, poll, and response counters;
- 1x/3x/10x scale curves plus the 1,201-commit and record-size/count
  boundaries;
- complete before/after snapshots of objects, branches, generations, nonce,
  pending/recovery/quarantine/receipt/audit state, layout/directories, response
  bytes, ledger current, permits, and terminal observation;
- the 28-site production `validate_sealed_commit` inventory, updated for any
  signature change, with upload-reachable recovery distinguished from
  unrelated local/file/HTTP owners; and
- a mutation table with exact patch, command, expected failure, observed
  assertion, duration, and proof that each mutation was removed afterward.

Any source-document statement that the upload request ledger is the only
sealed-validation ledger, that the reusable caller-owned API works, or that
the proof is single-use must match production behavior before handoff.

## Scope

Authorized implementation paths:

- `codefire`;
- direct Python tests under `tests/` for HTTP upload, completed-quarantine
  recovery, sealed validation, proof/identity state, accounting, deadline,
  and selected neighboring regressions; and
- the four canonical source documents, only where the repaired production
  contract or exact formula changes:
  - `docs/development/source-architecture.md`;
  - `docs/development/command-source-trace.md`;
  - `docs/development/source-data-model-catalog.md`;
  - `docs/development/change-playbook.md`.

This packet and the immutable review are read-only inputs after this commit.
Do not edit Rust product/tests, storage/file-ancestry/File Doctor work,
unrelated HTTP routes, issue or lifecycle state, capability claims, management
history, devcontainer, packaging, install, or subprojects.

## Compatibility and invariants

- Do not raise a public profile limit, shrink the 1,201-commit fixture, skip
  sealed/signature/ancestry validation, or weaken same-subject detection.
- Preserve authentication, nonce, signature policy, fast-forward/non-fast-
  forward precedence, idempotency, replay, recovery, receipt, and exact
  200/409/error response bytes.
- Account before every allocation, read, open, hash, sort, collection growth,
  and effect. No test-only observer may stand in for a production owner.
- Keep parse, validate, plan, preflight, apply, render, and write boundaries
  explicit and iterative.
- One upload request owns one ledger, one deadline, one sealed snapshot, and
  one response decision. Compatibility fallbacks outside a live request must
  not become an upload escape.
- Success and every failure finish with zero retained bytes, zero temporary
  bytes, zero open permits, and exactly one terminal observation.
- Preserve the accepted candidate bytes unless changing them is necessary for
  one of the four exact findings. Record every changed and intentionally
  untouched symbol.

## Validation

Run individual new regressions and deliberate mutations first. Use
`PYTHONDONTWRITEBYTECODE=1` and serialize long Python suites:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire \
  tests/test_python_http_upload_accounted_traversal.py \
  tests/test_python_sealed_production.py \
  tests/test_codefire_cli.py

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s tests -p 'test_python_*.py'

python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Also run the exact 13 selected legacy upload/nonce/receipt/Doctor/branch cases
listed in the immutable review, plus every new completed-quarantine,
caller-owned API, proof-state, projection, coverage, and layout-poll test
individually. Record exact commands, test names, counts, durations, runtime,
candidate/export identities, and unavailable evidence.

Create an additive WIP commit before long discovery or mutation runs. Apply
one mutation at a time in a clean isolated tree, preserve its output, reverse
it, and prove the intended tree is restored before the next mutation. Do not
run Cargo, Docker, install, dogfood, unrestricted all-language discovery, or
unrelated broad gates in this lane. The manager owns final composition and
repository-wide validation.

## Handoff

Before handing off:

- leave the implementation worktree clean;
- keep every commit additive above this packet commit;
- record exact parent, commit list, head, tree, binary/full-index patch digest,
  changed paths, and per-file blobs/SHA-256 values;
- report each of BLOCKER-1, HIGH-2, HIGH-3, and MEDIUM-4 separately with
  production reachability, closure evidence, and deliberate-regression
  outcome;
- record accepted-property preservation, formulas, peaks, boundaries, state
  transitions, snapshots, commands, counts, durations, and unavailable
  evidence;
- identify exact conflicts with active Python HTTP composition work; and
- request a fresh independent exact-candidate rereview.

Do not claim acceptance, integration, issue closure, capability completion, or
zero residual from implementer evidence. The manager must independently bind
the new candidate and launch a fresh report-only review.

## Non-goals

- Do not rewrite the rejected candidate, report, packet, or history.
- Do not replace the original implementer with a new architecture lane.
- Do not repair unrelated local/file/Rust surfaces or final composition here.
- Do not integrate, install, dogfood, close issues, or advertise capability.
