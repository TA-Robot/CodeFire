# CFR-168 Python HTTP MR-List Review-Closure Repair

Repair exact Python HTTP merge-request list candidate
`38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` against every finding in the
independent rereview committed as
`80784151e0a42b2f839cdcf01336252319e5830a`:

`docs/development/ai/swarm-results/cfr168-python-http-mr-list-zero-residual-rereview-fugu-20260730/review.md`

The report SHA-256 is
`6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and
write set. Add new commits; do not amend, rewrite, merge, or replace the
reviewed candidate, its report, or any adjacent rejected candidate.

The verdict is `REQUEST_CHANGES`. MUST-1 through MUST-4, SHOULD-1, COULD-1,
COULD-2, the unexplained-run evidence requirement, and the shared GET/POST
composition handoff are all mandatory. Preserve the accepted parts of
`38da898`: deferred filename-order fault precedence, direct leaf-symlink typed
failure, operation-local escrow memoization, one sealed session, one
`stable_project_id` owned-byte read, affine poll/counter curves, exact response
shape, and production-socket reachability.

## Exact ancestry and authority

- Legacy behavior baseline:
  `7327e2a8e72f6af84b0eadd8bb6b16233f86882a`
- Repair task base:
  `1695cc86664640cd3df94d0786839fdf3cff196f`
- Prior reviewed target:
  `37f7427d46d2cf3ad66de3c1885ef4c0527cae70`
- Rejected reviewed candidate:
  `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c`
- Independent rereview:
  `80784151e0a42b2f839cdcf01336252319e5830a`
- Rereview SHA-256:
  `6840035553871f5251e32d67bb5b2f9d6c6ee061b7ab3106e94de373ec97b20d`
- Adjacent reviewed MR-write candidate:
  `6dd4527f23135388b53fc5c6d637c4105cc5d659`
- Adjacent MR-write rereview:
  `b86a782b6ac72cabe684106f969681c5cd56380e`

The repaired MR-list candidate remains an additive successor of the frozen
task revision. It must not import either the older adjacent candidate
`b1c29fc97a9f6190c726b31d465b872558ce2808` or the rejected MR-write candidate
wholesale. Shared-dispatcher composition remains a manager-owned successor
step after both repaired candidates exist. This lane must still design its
authority, error-origin, cleanup, and prepublication interfaces so that one
composed model can replace both isolated variants.

This lane does not authorize canonical product integration, issue Close,
Install, Dogfood, or Retrospect.

## Required closure

### MUST-1 — distinguish leaf absence from unavailable parent authority

1. Pin and validate the authoritative parent directory capability before a
   missing leaf can be recorded as absence.
2. A leaf may be absent only when the exact pinned parent is an available
   directory and the leaf lookup itself returns the supported absence result.
3. Missing, replaced, symlinked, denied, unreadable, or I/O-failing parent
   authority must return the typed filesystem-authority response; it must not
   silently change a stale MR to open.
4. The final absent-state fence must reprove both parent identity/state and
   leaf absence. An unreadable parent is never evidence of continued absence.
5. Cover live GET cases for missing parent, parent symlink, permission denial,
   metadata failure, injected parent-open/read I/O failure, and a stale MR
   whose status would otherwise be downgraded.
6. Compare complete authority and response snapshots against the legacy
   baseline and exact reviewed candidate.

### MUST-2 — classify failure origin at the actual boundary

1. Do not classify every `TimeoutError` or `ConnectionError` as client
   transport solely by Python exception type.
2. Filesystem-origin `OSError`, including `ETIMEDOUT`, must enter the typed
   filesystem-authority family on both GET and POST dispatch.
3. Actual socket read/write timeout, reset, broken pipe, or disconnect at the
   transport boundary must continue to abandon the connection without
   fabricating a second response.
4. Preserve the existing typed request-deadline outcome and allow
   `MemoryError`/process-fatal conditions to remain outside the filesystem
   response arm.
5. Add independent GET and POST cases for filesystem timeout, generic
   filesystem error, real transport timeout/reset/disconnect, request
   cancellation, memory exhaustion, and primary-error-plus-cleanup-error.
6. The origin proof must survive later shared-dispatcher composition; a
   string or exception-class heuristic is insufficient.

### MUST-3 — cleanup retains retry ownership until release succeeds

1. Never zero a scalar, remove a session, detach an owner, or clear a map
   before its exact ledger release/close succeeds.
2. Cleanup must be idempotent and retryable. A second call after partial
   cleanup releases only remaining owners and cannot double-release.
3. Do not swallow `BaseException`. Preserve the selected primary diagnostic
   while recording and completing cleanup through the supported terminal
   protocol.
4. A success response cannot publish until cleanup proves the required
   request-header-only/open-zero state. A post-publication cleanup failure
   must not leave a published 200 with hidden current work.
5. Inject the first failure and retry for every retained/temporary scalar,
   session close, escrow/frontier close, projection close, prepared-response
   close, and request terminalization path.
6. Success and every failure must finish at zero retained bytes, zero
   temporary bytes, zero permits, and exactly one terminal observation.

### MUST-4 — terminal metadata remains bound through read and digest

1. Reprove every captured metadata field after the final byte read, EOF probe,
   and all interruptible digest work: device, inode, regular-file type, size,
   mode, link count, and every contractually captured field.
2. Bind content digest and the final metadata observation into one safe
   protocol; no mode/link change after the last `fstat` may escape.
3. Preserve the documented compatibility rule for stable pre-existing links.
   If exactly-one-link becomes the contract, make that a deliberate documented
   compatibility change with explicit migration evidence rather than an
   incidental rejection.
4. Add deterministic changes for mode, hardlink add/remove, symlink
   replacement, size, identity, and content at each boundary: before open,
   during read, after EOF, during digest, after digest, and immediately before
   publication.
5. Every detected change returns the exact typed authority-change response and
   leaves complete authority bytes and ownership in the required state.

### SHOULD-1 — final fences immediately precede the first response byte

1. Prepare and encode the complete accounted response before the terminal
   authority/session/project fence.
2. Keep the required pinned owners and sealed session alive through that last
   fence.
3. After the fence, perform no fallible, authority-dependent, callback-capable,
   or input-proportional work before the first response byte.
4. Define the precise publication point and transfer the prepared response
   owner without a second encoding or ledger.
5. Add changes after projection, after encoding, after each current finalizer,
   inside the final fence, and immediately before `prepared.publish`.
6. Prove success bytes, typed failure bytes, authority snapshots, deadline
   precedence, and cleanup state at each boundary.

### COULD-1 — make descriptor evidence state its true scope

1. Either rename/narrow the one-read oracle to the
   `stable_project_id` owned-byte boundary, or instrument every descriptor
   open/read and assert the exact route-wide protocol.
2. Do not use one `read_json_accounted` call count to claim one physical
   descriptor read while project-gate acquisition/recheck/release perform
   additional required reads.
3. Document each physical descriptor read's owner, purpose, phase, and reuse
   rule. Eliminate a read only when the corresponding authority proof remains
   complete.
4. Add a deliberate regression that duplicates the scoped read or bypasses
   its owner and fails for the intended reason.

### COULD-2 — prove each production poll or remove redundancy

1. Enumerate production poll sites and admitted-work-quantum boundaries
   independently from the helper that implements them.
2. Silence one site at a time. Each retained poll must either provide the
   unique cancellation point for a bounded quantum or enforce the documented
   maximum quantum.
3. Remove genuinely redundant polls and update the phase/poll formula.
4. Preserve affine 1/8/24/80/200 curves, long-traversal cancellation, one
   sealed session, constant escrow traversal, and exact phase diagnostics.

## Prior-finding preservation

- Deferred per-record authority capture must retain filename-order legacy
  first-error precedence under both fault orderings.
- Direct MR, branch, and descriptor leaf symlinks must return typed non-empty
  responses.
- Escrow memo keys remain operation-local and bind the absolute pinned object
  root plus source head; no cross-request reuse is allowed.
- Reused session documents are finalized once per pinned root after
  projection.
- `stable_project_id` remains derived from the exact bytes charged by its
  owner, without the old stat/reopen gap.
- Object records, memo misses, unique nodes, semantic validations, graph
  nodes, and graph edges remain constant on shared-head growth; per-record
  reads/fences/hashes/projection/polls remain affine.

## Non-vacuity and evidence

1. Every new regression must fail against
   `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` for its intended finding.
2. Preserve the prior candidate and baseline reversals for multi-fault
   precedence, direct leaf symlink, memo removal, and helper-owned reads.
3. Exact/minus-one/plus-one boundaries must independently reach every
   retained, temporary, response, header, descriptor, authority-record,
   projection, and cleanup owner.
4. Record 1/8/24/80/200 counters, owner formulas, time/memory complexity,
   physical reads, fence records, bytes hashed, and poll counts.
5. Full pre/post snapshots cover project descriptor, parent directories,
   branches, MR records, objects, locks, nonce/recovery/pending/receipt/audit
   state, prepared/wire response bytes, ledger current, permits, and terminal
   observation.
6. If the focused module fails once, preserve the failing test ID, traceback,
   exact wire bytes, observer snapshot, phase/poll trace, and enough fixture
   identity to reproduce it. Later green reruns cannot erase an unidentified
   failure.

## GET/POST composition contract

The final accepted Python spine must reconcile MR-list with the accepted
successor of MR-write and Upload. This lane must provide exact shared-helper
handoffs for:

- one filesystem-authority error taxonomy with origin-sensitive transport
  handling;
- one retryable cleanup state machine;
- one prepared-response owner and prepublication point;
- one authority/session/project final-fence protocol;
- one GET/POST flag-reset and route-isolation model.

Do not duplicate incompatible helpers merely to keep the isolated patch
conflict-free. Do not import rejected adjacent product bytes. Report exact
symbols and conflicts so the manager can compose the accepted ranges and run
mixed GET/POST isolation, filesystem, cleanup, fence, response, and reversal
evidence before independent composed review.

## Authorized validation

Run narrow syntax and individual regressions first:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_list_decision_work.py \
  tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_list_decision_work -v
```

Then run affected legacy MR-list, branch, project-descriptor, filesystem,
deadline, transport, cleanup, response, and adjacent GET/POST cases plus:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

The manager serializes full Python, composition, install, and dogfood gates.
Do not run Cargo, Docker, install, dogfood, unrestricted all-language
discovery, or unrelated product gates in this lane.

Create a durable WIP before long validation. Commit only intended product,
focused tests, and canonical source documents; leave the worktree clean.

## Handoff

Report:

- final commit and exact parent/task revision;
- changed and intentionally untouched files;
- MUST-1 through MUST-4, SHOULD-1, COULD-1, and COULD-2 closure;
- exact formulas, physical-read protocol, boundaries, curves, and snapshots;
- candidate reversal and prior-regression preservation results;
- commands, counts, durations, and hashes;
- complete GET/POST/MR-write/Upload symbol and conflict handoff;
- any exact predecessor-gated residual;
- a request for manager composition followed by fresh independent review.

Do not claim zero-residual final acceptance from this isolated candidate.
Closure requires the composed Python spine and its independent review.
