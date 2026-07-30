# CFR-168 Python HTTP MR Write Review-Closure Repair

Repair the exact Python HTTP merge-request write candidate
`6dd4527f23135388b53fc5c6d637c4105cc5d659` against every finding in the
independent review committed as
`b86a782b6ac72cabe684106f969681c5cd56380e`:

`docs/development/ai/swarm-results/cfr168-python-http-mr-mutation-zero-residual-rereview-fugu-20260730/review.md`

The report SHA-256 is
`214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and
write set. Add new commits on top of the frozen task revision; do not amend,
rewrite, merge, or replace the reviewed candidate or report.

The verdict is `REQUEST_CHANGES`. F1 through F7 and every newly discovered
route-local residual are required closure work. Preserve the accepted parts of
the candidate: removal of the nested compatibility ledger, one handler ledger,
deterministic legacy preflight before escrow, typed same-path growth handling,
goal-directed apply admission, and the existing predecessor gate for GC.

## Exact ancestry and authority

- Immutable product baseline:
  `e2a218e015953f082245f36dbc6aee906b837e5a`
- Previously rejected repair:
  `21fe75175e997cc7c6f8c3ef2a7558144d850f94`
- Reviewed candidate:
  `6dd4527f23135388b53fc5c6d637c4105cc5d659`
- Independent review:
  `b86a782b6ac72cabe684106f969681c5cd56380e`
- Independent report SHA-256:
  `214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef`

The repair must remain an auditable additive successor of the frozen task
revision. This lane does not authorize canonical integration, issue closure,
installation, dogfood, or retrospective completion.

## Required architecture and behavior

### F1 — one whole-request ownership domain

1. The handler-created `http_v1` ledger and its request deadline must own all
   create, review, and apply decision work until terminal cleanup.
2. Account before reading or allocating every live owner for:
   - project descriptors and project identity;
   - branch, merge-request, generation, policy, signature, and nonce authority;
   - recovery, pending, receipt, audit, and replay records;
   - publication preimages, canonical encodings, hashes, copied buffers, and
     response projections;
   - object paths, descriptors, raw bytes, parsed values, retained collections,
     and temporary iteration state;
   - nonce postimages before postimage allocation.
3. Reuse charged owners instead of materializing the same authority or
   projection twice. Poll the request deadline in every input-proportional
   loop.
4. The review's concrete 2,098,457-byte merge-request read must no longer
   finish with `aggregate_bytes=0`; it must either succeed within the exact
   owned limit or return the typed boundary outcome with non-zero, formula-
   consistent accounting.
5. Do not raise a public profile limit, reserve an arbitrary blanket maximum,
   hide work from instrumentation, or move a charge after its allocation.

### F2 — apply reuses the sealed request snapshot

1. Eliminate physical rereads of sealed commits during apply. Authorization,
   validation, collection, planning, and publication must consume the same
   request-owned `SealedCommitValidationSession` or an explicitly transferred
   request-owned snapshot.
2. `AccountedRemoteCommitBorrow` must not reopen or reread an object already
   retained by the sealed session merely to satisfy a later apply phase.
3. For the review fixture with ten object records, the repaired route must not
   perform thirteen physical reads. Report exact logical-record and physical-
   read counts.
4. At 1x, 3x, and 10x, the apply reread gap must remain zero rather than grow
   1, 3, and 10. A deliberate regression that restores the second physical
   read path must fail the focused evidence.
5. Ownership transfer, discard, early rejection, and cleanup must release each
   descriptor, buffer, record, and permit exactly once.

### F3 — complete and non-vacuous boundary evidence

1. Cover exact, exact-minus-one, and exact-plus-one for every charged dimension,
   including project identity, policy, nonce, authority records, descriptors,
   object bytes, retained records, canonicalization, copying, response
   projection, publication, and cleanup.
2. Prove each boundary independently. Do not let a different earlier limit
   make a later boundary case vacuous.
3. Add deliberate-regression checks that fail when:
   - a policy, nonce, descriptor, or projection owner is omitted;
   - apply restores a physical reread;
   - publication begins before the prepublication seal;
   - cleanup forgets a releasable amount;
   - nonce state is committed before reducer confirmation.
4. Each new check must fail against
   `6dd4527f23135388b53fc5c6d637c4105cc5d659` for the intended reason. Record
   the observed reversal rather than only asserting source shape.
5. Preserve the earlier focused suite's non-vacuous failure against
   `e2a218e015953f082245f36dbc6aee906b837e5a`.

### F4 — seal all publication work before the first authoritative effect

1. All object-copy plans, canonical bytes, hashes, branch/MR/generation
   postimages, recovery state, response preimage, and required owner formulas
   must be complete before `seal_prepublication`.
2. No escrow object copy or other authoritative filesystem write may occur
   before the reducer/pending transaction can either complete atomically or
   recover deterministically.
3. If the second of four planned object writes fails, the full authority
   snapshot must remain byte-identical to the pre-call snapshot, including
   objects, branches, generations, merge-request records, nonce state,
   pending/recovery/quarantine/receipt/audit state, and response bytes.
4. If the product deliberately supports a durable recovery boundary instead
   of rollback, the durable recovery record must exist before the first
   authoritative effect and exact retry must converge to the same result.
   A newly added object without its recovery authority is forbidden.
5. Preserve typed permanent-versus-retryable precedence and legacy response
   bytes whenever the repaired boundary is not crossed.

### F5 — retryable cleanup and stable terminal outcome

1. Do not zero or forget a retained/temporary scalar before its release
   succeeds. A failed release must leave enough exact state for bounded retry
   or compensating cleanup.
2. Cleanup of sessions, copied owners, projections, response slots, observers,
   and permits must be idempotent. Double-close must not double-release.
3. A cleanup failure after durable publication must not replace or suppress the
   already selected primary response. Record the cleanup condition through the
   supported terminal observation while preserving one response document.
4. Success and every failure must end with zero retained bytes, zero temporary
   bytes, zero open permits, and exactly one terminal observation.
5. Add deterministic first-release-fails/second-release-succeeds fixtures for
   each affected owner and prove exact terminal state.

### F6 — commit replay nonce state with reducer confirmation

1. Do not persist a replay nonce before the reducer has confirmed the
   authoritative replay result.
2. A reducer failure returning the supported retry response must leave the
   request retryable with the same signed request; it must not consume the
   nonce and then reject the exact retry as already used.
3. On success, nonce acceptance and authoritative replay publication must
   converge atomically or through one durable recovery transaction.
4. Preserve key-window, clock-skew, nonce-capacity, idempotency, and exact
   replay-response semantics. A captured request must not become reusable
   without bound.
5. Cover reducer failure before confirmation, durable recovery, exact retry,
   different-payload retry, and successful replay with full authority
   snapshots.

### F7 — canonical source truth

1. Update `docs/development/source-data-model-catalog.md`: the HTTP response
   owner is used by all covered MR write routes, not upload only.
2. Keep `source-architecture.md`, `command-source-trace.md`,
   `source-data-model-catalog.md`, and `change-playbook.md` synchronized with
   every changed owner, phase boundary, recovery rule, and validation route.
3. Name the single request ledger, sealed session/snapshot transfer,
   prepublication seal, nonce/reducer transaction, publication state, cleanup
   state, and exact terminal guarantees.

## Required evidence

### Ownership and complexity

Provide:

- the exact live call chain for create, review, and apply;
- a retained/temporary formula table for every owner;
- simultaneous-owner peak formulas for authorization, validation, apply,
  projection, publication, response, and cleanup;
- time and memory complexity for every input-proportional traversal;
- logical-record, physical-read, descriptor, byte, retained, temporary, and
  permit curves at 1x, 3x, and 10x;
- exact/minus-one/plus-one observations for every charged dimension;
- proof that no nested ledger or unowned collection/copy/JSON read remains.

### Authority and terminal snapshots

Before and after every injected failure, compare exact bytes and membership for:

- object records;
- branches and generations;
- merge-request records;
- policy, signature, and nonce state;
- transaction, pending, recovery, quarantine, receipt, and audit state;
- selected response bytes;
- retained/temporary counters, permits, and terminal observations.

### Compatibility

Preserve exact create/review/apply success, typed error, durable retry,
idempotency, cross-runtime, signature, nonce, branch-protection, legacy escrow,
and response semantics except for the explicitly required corrections above.
`POST .../gc` remains predecessor-gated and outside this lane.

## Authorized validation ladder

Run narrow syntax and focused evidence first:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_mutation_decision_work.py \
  tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_mutation_decision_work -v
```

Then run affected neighboring Python HTTP/MR/signature/nonce/replay/escrow/
branch-protection cases and:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

The manager serializes the full Python suite, composition gates, and install
checks after all predecessor candidates are accepted. Do not run Docker,
install, dogfood, unrestricted all-language discovery, or unrelated Cargo
work in this lane.

Create a durable WIP commit before any long validation. Commit only intended
product, focused tests, and canonical source documents; leave the worktree
clean.

## Handoff

Report:

- final commit and exact parent/task revision;
- exact changed and intentionally untouched files;
- F1 through F7 closure evidence;
- formulas, owner peaks, counter curves, and physical-read curves;
- exact/minus-one/plus-one and deliberate-regression results;
- full snapshot and zero-debt/zero-permit evidence;
- commands, counts, durations, and hashes;
- any exact residual that is genuinely predecessor-gated;
- expected composition conflicts;
- a request for fresh independent review of the exact repaired commit.

Do not claim complete closure merely because focused tests pass. Closure
requires the fresh independent review and manager composition gates.
