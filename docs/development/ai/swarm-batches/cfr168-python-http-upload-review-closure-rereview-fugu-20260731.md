# CFR-168 / CFB-138 Python HTTP Upload Review-Closure Independent Rereview

Status: ready for exact independent review

Provider/model: fresh Sakana Fugu 1.1 Ultra (`fugu-ultra-v1.1`)

Role: independent reviewer

Mode: report-only

All candidate fields are bound to the immutable manager-recomputed subject
below. The runnable manifest is created only above this ticket and the
candidate card.

## Objective

Independently review the exact three-commit successor that claims to close
the prior Fugu `REQUEST_CHANGES`. Reproduce the completed-quarantine
request-budget escape, the caller-owned API failure, the reusable
publication-root proof, and the vacuous reversal matrix on the rejected
candidate. Then re-falsify every claimed closure on the exact successor and
return one immutable report.

The reviewer uses a new Fugu agent, session, and worktree. It inherits no
prior reviewer session, Claude implementer conversation, WIP, conclusion, or
mutable `/tmp` evidence.

## Exact subject

| Item | Exact value |
|---|---|
| Rejected predecessor | `76baff84cb1192e11b26dcc669fb800f7b13fa30`; tree `fcc65a4729f5029556e0266971d94b6321421f09` |
| Prior independent-review commit | `35983261e1abd32045e1ff6cbd7dd7dfff5e806d` |
| Prior report in candidate lineage | commit `fe7cf94388393bd9b0457564d0116406f2c8844e`; blob `1fd3de9b48e04d29b56f36d79584a0a03c22bcff` |
| Prior report SHA-256 / verdict | `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` / `REQUEST_CHANGES` |
| Continuation base / merge base | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` / `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Candidate SHA / parent / tree | `467b0c27715e424ae18ffb5664da053b55915e2d` / `f0ecd25ed18005d3f50b54dd254b7d05e678afb3` / `3b129b9e41ae45defe4d6e7c43da04b016f07404` |
| Additive commits | `f2ebb32c26629cadca6a8c0967a212c1a91ff688`, `f0ecd25ed18005d3f50b54dd254b7d05e678afb3`, `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Additive count / merges | `3` / `0` |
| Binary full-index patch SHA-256 | `a71ea9276361cd426c0eecbf06c900034b5129849df281c8b8f881ffa0f63dc1` |
| Original/resumed Claude session | `7baa1275-0ee2-476a-af46-1907386ebceb` |
| Implementer result blob / SHA-256 | `7100966bf76b1ada44d797e8787eb8377a18f6fd` / `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361` |
| Terminal state | implementer completed; provider/gate processes terminal; candidate worktree clean |

The complete manager card is:

`docs/development/ai/swarm-results/cfr168-python-http-upload-review-closure-repair-candidate-ready-20260731/candidate.md`

Before reviewing behavior, create a new clean detached scratch checkout of
the exact candidate and recompute:

- checkout HEAD, parent, tree, merge base, linear ordered commits, and merge
  count;
- binary/full-index/no-renames patch SHA-256;
- exact changed paths, numstat, modes, blobs, byte sizes, and content SHA-256;
- the prior report, continuation packet, and implementer-result
  blob/SHA-256; and
- the original candidate worktree's clean state without modifying it.

Do not follow a mutable branch tip and do not use the implementer's worktree
for mutations. Any identity mismatch is immediate `REQUEST_CHANGES`.

## Scope and write isolation

Read:

- `AGENTS.md`;
- the exact candidate range and live Python HTTP upload dispatcher;
- the immutable prior report;
- the continuation packet and implementer result;
- the manager candidate card;
- the focused upload tests and selected neighboring Python suites;
- the four changed source-contract documents; and
- earlier reports/tests necessary to re-falsify inherited upload behavior.

The only authorized repository write is:

`docs/development/ai/swarm-results/cfr168-python-http-upload-review-closure-rereview-fugu-20260731/review.md`

Product code, tests, source docs, packets, cards, helpers, existing reports,
issues, ledgers, disposition indexes, lifecycle state, Rust, devcontainer,
packaging, Install, and Dogfood are read-only or out of scope. Create
reviewer-specific scratch checkouts and logs outside the review worktree.
Remove temporary mutations when no longer needed and leave no untracked
artifact in the reviewer worktree.

## Required independent falsification

### 1. BLOCKER-1 — completed-quarantine recovery request ownership

Drive the production upload retry through:

```text
handle_http_upload
  -> execute_remote_json_publication
  -> recover_remote_pending_quarantine
  -> validate_sealed_commit
  -> receipt replay / quarantine cleanup / response selection
```

Independently prove:

1. the exact handler `DecisionWorkLedger` and `HTTPRequestDeadline` are
   transferred through every upload-owned call;
2. each recovered durable root is validated on that same ledger/deadline,
   with no fresh `local_v1()` or `sealed validation` ledger;
3. deadline polls occur before every recovered-root validation and
   immediately before receipt replay, quarantine cleanup, and recovered
   response authority;
4. no fallible, callback-capable, input-proportional, or authority-dependent
   work occurs between the last required poll and its effect;
5. advancing 0.0 seconds to 1.0 seconds against a 100 ms deadline returns the
   typed deadline outcome, preserves the completed quarantine, and leaves
   objects, branches, generation, nonce, pending transaction, receipts,
   audit, response authority, debt, and permits unchanged;
6. the live retry still validates each intended root once, clears only the
   intended quarantine, returns byte-identical 200, and replays
   idempotently; and
7. ordinary upload retains one request ledger, one deadline, one prepared
   sealed snapshot, one terminal response decision, and no foreign
   validation session.

Run the same reviewer-authored defect probe on exact rejected predecessor
`76baff84…` and require the original escaped-ledger/expired-200 behavior.
Silencing only the new handoff or its last poll in a scratch successor
mutation must restore the defect for the named reason.

### 2. HIGH-2 — usable caller-owned sealed-validation APIs

Audit `_AccountedUploadSourceMemory`, ordered projection construction, and
`SealedCommitValidationSession.finalize` from actual claim sites and live
owner lifetimes.

Prove:

- the exact `temporary_bytes` projection claim is acquired before list
  allocation and sorting;
- the formula is derived from the actual document count and runtime
  collection footprint rather than hard-coded observations;
- empty, one-record, 27-record, and a large multi-record closure reproduce
  the exact projection size and terminal release;
- exact-minus-one, exact, peak-minus-one, and peak capacity fail or complete
  at the claimed owner and phase;
- constructor, allocation, sort, deadline, cancellation, publication,
  release-failure, and retry paths preserve one owner and zero terminal
  debt/permits;
- all three public paths complete both with a caller ledger and a caller
  ledger plus deadline:
  `validate_sealed_commit`,
  `validate_remote_json_publication_identity`, and
  `preflight_remote_json_publication`; and
- the no-ledger compatibility path and diagnostic precedence remain
  unchanged.

Delete only the ordered-projection claim and, separately, the source owner's
temporary dimension in fresh scratch mutations. The candidate's focused
oracle must fail before an unowned allocation; a missing symbol, forced
raise, or fixture failure is not proof.

### 3. HIGH-3 — atomically single-use publication-root proof

Trace every transition among `available`, `consuming`, `consumed`, and
`revoked`. Prove:

- state is claimed before the callback-capable deadline poll;
- exactly one successful use returns roots;
- alias use, callback reentry, and ordinary second use cannot consume again;
- foreign object store, uncovered root, stale identity, expired deadline,
  and operation/actor/key mismatch retain the documented retry or terminal
  state and diagnostic precedence;
- revoke-before-use, revoke-after-use, repeated revoke, cleanup failure, and
  owner drop release exactly once; and
- normal upload consumes once, later revokes/cleans, traverses once, and
  preserves wire bytes and publication ordering.

Remove only the reuse guard in a scratch mutation and require the direct
state oracle to reproduce a second successful consumption and callback
reentry. Compare with the exact rejected predecessor.

### 4. MEDIUM-4 — non-vacuous reversal matrix

Recreate every mutation independently, one at a time, against a clean
scratch export. Record the exact patch, baseline and mutated SHA-256,
command, expected assertion, observed failure, monotonic duration, and proof
of restoration before the next mutation.

The matrix must cover:

1. ordered-projection claim removal while list allocation/sort remain;
2. replacing `_assert_prepared_coverage` with a no-op;
3. removing its debt predicate;
4. removing its peak-versus-upper predicate;
5. removing its pool-was-used predicate;
6. removing only the layout-bootstrap deadline poll;
7. dropping recovery ledger/deadline handoff;
8. removing publication-root proof reuse protection; and
9. removing the source owner's temporary dimension.

Every row must be non-green for the intended behavioral reason. A candidate
symbol absent on `76baff84…`, import error, line-number mismatch, or
reviewer-forced exception is vacuous.

### 5. Preserve accepted upload properties

Re-falsify:

- the default-profile 1x/3x/10x curves and 1,201-commit exact 409
  non-fast-forward decision;
- admitted records above 3 MiB and the exact record-size/count ceiling under
  unchanged `DecisionWorkProfile.http_v1()`;
- canonical streaming byte identity for ASCII, CJK, emoji, escape-heavy,
  control-character, and mixed Unicode shapes;
- bounded scratch with no whole-record duplicate;
- descriptor registry/reuse bounds across wide fan-in, deep chain, shared
  DAG, signed history, and stacked diamonds;
- detached-document fingerprint release;
- existing-project and missing-project pre-effect deadline fences;
- exact 200/409/error/retry bytes, authentication, signature, nonce,
  idempotency, receipt, recovery, and fast-forward precedence; and
- zero current retained/temporary debt, zero permits, and one supported
  terminal observation on success and every failure.

No raised public limit, reduced fixture, skipped validation, weakened
identity, or changed error precedence is admissible.

### 6. Call-site and composition residual audit

Enumerate all production `validate_sealed_commit` occurrences and every
shortest route from HTTP/CLI dispatch. The candidate result reports 29
occurrences.

Confirm:

- upload-owned recovery uses the request ledger/deadline;
- the preserved no-budget recovery call is reachable only from a compatible
  non-request owner;
- `handle_http_list`, `http_remote_branch`, and
  `validate_http_merge_request_record` remain separately owned and the
  reusable API is sufficient for their separately tracked repair;
- no second proof, traversal, ledger, response-owner, or effect path bypasses
  the repaired protocol;
- no exception arm swallows a deadline/accounting failure or revives a stale
  prepared success;
- owner state is never erased before successful release; and
- the shared `execute_remote_json_publication` and
  `recover_remote_pending_quarantine` signature changes do not regress MR
  merge/review/apply, remote GC, CLI publication, or no-ledger compatibility.

Identify exact textual/semantic overlap with the active MR-list candidate.
An `ACCEPT` here is Upload-scoped and cannot substitute for later Python
composition review.

## Required validation

Use `PYTHONDONTWRITEBYTECODE=1`, a reviewer-specific `/tmp` prefix, unique log
paths, and a monotonic/performance-counter-compatible wrapper. Record exact
command, candidate/export identity, count, monotonic duration, exit status,
runtime/toolchain, log bytes, and log SHA-256.

Run at least:

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

git diff --check \
  b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b..\
467b0c27715e424ae18ffb5664da053b55915e2d
```

Also run:

- every new completed-quarantine, caller-owned API, proof-state, projection,
  coverage, and layout-poll regression individually;
- all nine deliberate mutations;
- the exact 13 selected legacy
  upload/nonce/receipt/Doctor/branch cases named by the immutable prior
  report;
- reviewer-authored raw production probes for each finding; and
- focused MR merge/review/apply, remote GC, recovery, signature, and
  publication cases reached by the changed shared primitives.

The worker recorded 29/29 focused, 59/59 neighboring, 450/450 broad Python,
13/13 selected legacy, and both source/project-doc checks green. These are
claims to reproduce, not review evidence.

Cargo, Rust, Docker, Install, Dogfood, `./demo.sh`, unrestricted all-language
discovery, repository-wide integration, and Python composition are forbidden
in this lane. Do not install a toolchain or mutate the environment. Mark
manager-owned gates explicitly unavailable/pending; do not imply the Python
pattern sweep proves them.

## Decision and handoff

Write and commit exactly the declared report. Include:

- recomputed immutable identity and provenance;
- literal dispositions for BLOCKER-1, HIGH-2, HIGH-3, and MEDIUM-4;
- production call graph, owner/deadline/effect/state timelines;
- formulas, exact/minus-one boundaries, scale curves, and durable-state
  snapshots;
- mutation/reversal and rejected-predecessor evidence;
- full call-site and composition residual audit;
- exact validation commands, counts, durations, log hashes, skipped gates,
  and evidence limits;
- strongest correctness and maintainability concerns; and
- one literal verdict.

`ACCEPT` requires exact identity, all four prior findings closed,
production-reachable non-vacuous counterexamples, every in-scope gate
independently green, zero upload-local Blocker/High/Medium residual, and no
new same-root residual. `ACCEPT_WITH_NOTES` is forbidden. Otherwise return
`REQUEST_CHANGES` with exact location, reproducer, state impact, and minimal
closure requirement.

Acceptance is scoped only to immutable candidate
`467b0c27715e424ae18ffb5664da053b55915e2d`. It does not authorize
integration, composition, issue closure, Install, Dogfood, Close, release, or
any successor protocol.
