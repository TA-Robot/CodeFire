# CFR-168 Python HTTP Manager Composition Freeze Task Packet

Status: blocked pending three exact accepted successor bindings

Typed packet:
[`json/cfr168-python-http-composition-freeze-20260730.json`](json/cfr168-python-http-composition-freeze-20260730.json)

Canonical freeze base:
`740ea18eb42630ba265a8aad83c950ceb96aacca`

Role: manager only

No `dev-swarm.v1` manifest exists for this task. Do not create one, launch a
worker, create a product composition branch, or modify product bytes while
this packet remains `blocked`.

## Objective

Compose the accepted Python HTTP Doctor, Branch, Upload, merge-request list,
and merge-request write product deltas into one reviewed request/session/
response spine without importing a rejected branch tip, duplicating an
ownership model, or treating isolated-candidate evidence as composed-tree
evidence.

This packet freezes the manager-controlled order and gates. It is not an
implementation candidate and does not authorize Integration, Close, Install,
Dogfood, Retrospect, or capability re-advertisement.

## Exact authority and blockers

| Surface | Current exact evidence | Required accepted binding |
|---|---|---|
| Doctor + Branch scaffold | product `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`; independent `ACCEPT` report `c57215e243839b224a641251fc9e3bf092f7e116` | fixed and accepted, but only for the Doctor + Branch scope |
| Embedded old Upload slice | `0703f71b9800ff19ce52d01bcd46c8eddb323df2`, representing the earlier `cb13da08…` recovery product/test slice | must be advanced to the accepted Upload successor before the scaffold is landable |
| Upload repair | current candidate `76baff84cb1192e11b26dcc669fb800f7b13fa30`; exact rereview pending | `ACCEPTED_UPLOAD_SHA = UNBOUND`; exact report commit, report SHA-256, decision, and accepted product-delta range are also unbound |
| MR-list repair | `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c` rejected by `80784151e0a42b2f839cdcf01336252319e5830a`; closure repair active | `ACCEPTED_MR_LIST_SHA = UNBOUND`; exact fresh report commit, report SHA-256, and accepted product-delta range are unbound |
| Shared-primitives adversarial preflight | blocked packet `cfr168-python-http-shared-primitives-preflight-20260730`; no manifest while Upload/MR-list are unbound | exact Fugu `READY` report commit and SHA-256 must be bound before MR-write closure implementation starts |
| MR-write repair | `6dd4527f23135388b53fc5c6d637c4105cc5d659` rejected by `b86a782b6ac72cabe684106f969681c5cd56380e`; closure repair prepared | `ACCEPTED_MR_WRITE_SHA = UNBOUND`; exact fresh report commit, report SHA-256, and accepted product-delta range are unbound |

The packet may move from `blocked` to `ready` only after all three `UNBOUND`
values are replaced in both packet files and each replacement is supported
by:

1. an immutable exact-candidate report with decision `ACCEPT`;
2. the report commit and SHA-256;
3. the accepted candidate parent, head, tree, and deterministic patch digest;
4. an exact product/test/source-doc changed-file set inside this packet's
   write set;
5. a disposition for every prior finding and every new finding;
6. proof that no later report supersedes or rejects the candidate.
7. for MR-write, the committed shared-primitives preflight decision and
   SHA-256, with decision `READY`, were bound before the implementation lane
   launched.

A focused test pass, clean worktree, agent success, report draft, accepted
predecessor, or manager inspection does not satisfy an accepted binding.

## Conditional scaffold rule

`701469f…` contains old Upload product bytes from `0703f71…`. Its independent
report deliberately accepts Doctor + Branch only and excludes Upload.
Therefore:

- when Upload is accepted, use `701469f…` only as the isolated composition
  scaffold and advance its embedded Upload slice through the accepted product
  delta;
- when Upload is `REQUEST_CHANGES`, interrupted, unavailable, or still
  pending, neither `701469f…` nor an MR-list descendant carrying `0703f71…`
  may be integrated wholesale;
- reconstructing a no-Upload Doctor/Branch/MR-list tree is a different
  composition subject and requires a separately frozen packet, exact review,
  and broad gates. This packet does not authorize that fallback.

## Frozen composition order

### 1. Doctor + Branch scaffold

Start the isolated manager composition from exact `701469f…`. Preserve the six
combined `read_route_phase` conflict resolutions, the corrected branch-list
deadline expectation, and per-request reset of bundle, Doctor, list, and
mutation flags.

The scaffold's 42/42 combined focused, 463/463 Python-pattern, and 691/691
unrestricted evidence is baseline evidence only. It does not prove any later
product tree.

### 2. Accepted Upload successor

Advance the embedded old Upload slice to `ACCEPTED_UPLOAD_SHA`. Do not replay
`3153d75…` or `cb13da08…`; their recovery product/test patch is already
represented by `0703f71…`. Do not merge or cherry-pick the Upload agent branch
tip wholesale.

Port only the accepted scale and zero-residual product/test/source-doc deltas.
The result must retain one handler ledger/deadline, one request-owned sealed
snapshot, bounded canonical streaming, one publication identity/proof path,
and the final pre-effect fences accepted by the Upload review.

### 3. Accepted MR-list successor

Port only the accepted MR-list product delta onto the composed
Doctor+Branch+Upload tree. The MR-list lineage descends from `701469f…`, but
its agent branch also carries reports, task registration, rejected
intermediate commits, and the old Upload scaffold. Never land that branch
tip.

Reconcile GET and POST dispatch through:

- one origin-aware filesystem-versus-transport error taxonomy;
- one retryable/idempotent cleanup state machine;
- one prepared-response owner and first-byte publication point;
- one authority/session/project final-fence protocol;
- one request-flag reset and mixed-route isolation model.

### 4. Accepted MR-write successor

Port `ACCEPTED_MR_WRITE_SHA` last. The MR-write lineage diverges at
`cab3126f216c595c0d2211174b364e8532a310e2`; blind merge, rebase, or wholesale
cherry-pick is forbidden.

The successor must descend from an implementation packet that binds the exact
`READY` report produced by
`cfr168-python-http-shared-primitives-preflight-20260730`. A preflight
`BLOCK`, missing report, later predecessor finding, or unbound report digest
keeps the MR-write implementation and this composition packet blocked.

The port must reuse, not fork:

- Upload's response owner, canonical streaming, sealed-snapshot borrowing,
  publication preflight, and reducer seams;
- MR-list's authority reader, error-origin classification, retryable cleanup,
  final fence, and dispatcher state;
- Doctor/Branch's exact request ledger/deadline and read-route phase model.

Nonce/replay, escrow, pending/recovery/receipt, object copy, and authoritative
publication must converge on one prepublication-sealed transaction model.

### 5. Manager conflict resolution and final spine

Only the manager may resolve overlapping `codefire`,
`tests/test_codefire_cli.py`, dispatcher, response, session, reducer, or
canonical source-doc hunks. A conflict resolution may select and combine
already accepted contracts; it may not invent an unreviewed weaker third
model. If accepted inputs are semantically incompatible, stop and freeze a
new repair packet rather than masking the incompatibility.

Record every input base/head, selected product commit, skipped management
commit, resolved hunk, before/after blob, and cumulative patch digest.

## Scope

### Product-delta-only rule

Each input must be materialized as an auditable product delta, not as an agent
worktree or branch-tip merge.

Allowed composed paths are limited to:

- `codefire`;
- `tests/test_codefire_cli.py`;
- `tests/test_python_http_doctor_decision_work.py`;
- `tests/test_python_http_branch_decision_work.py`;
- `tests/test_python_http_upload_accounted_traversal.py`;
- `tests/test_python_http_mr_list_decision_work.py`;
- `tests/test_python_http_mr_mutation_decision_work.py`;
- `docs/development/source-architecture.md`;
- `docs/development/command-source-trace.md`;
- `docs/development/source-data-model-catalog.md`;
- `docs/development/change-playbook.md`.

Reports, task packets, manifests, recovery ledgers, issue state, history,
agent state/logs, and unrelated source files are not product-delta inputs.
Intermediate rejected commits may appear in the accepted candidate ancestry,
but the composed final blobs must equal the independently accepted successor
semantics and the cumulative review must cover the whole selected range.

## Shared ownership contract

The composed tree must have:

1. exactly one handler-created `http_v1` ledger and one deadline per request;
2. no request-reachable fresh `local_v1` validation ledger;
3. one reusable sealed session/snapshot per store, with no physical reread by
   signature, ancestry, MR-list, or MR-write consumers;
4. one prepared response owner whose terminal selection survives cleanup and
   durable publication;
5. one origin-sensitive filesystem/transport boundary shared by GET and POST;
6. retryable cleanup that never erases an unreleased amount or replaces the
   selected primary outcome;
7. complete response encoding before the final authority/session/project
   fence and no fallible input-dependent work before the first byte/effect;
8. one prepublication-sealed reducer path for Upload and MR writes, including
   nonce, object, escrow, pending, recovery, receipt, and audit authority;
9. all bundle/Doctor/list/mutation flags reset before every request and only
   the dispatched route armed;
10. synchronized source architecture, command trace, data model, and change
    playbook descriptions.

## Review boundaries

Isolated candidate acceptance is necessary but never sufficient for
composition.

- Every manager-authored conflict-resolution series must be included in a
  fresh exact report-only composition review.
- If Doctor+Branch+Upload becomes a durable next-wave base before MR-list is
  ready, that exact head needs its own composition review.
- If Upload and MR-list are both ready, the manager may apply them in the
  frozen order, run focused checks after each, and request one cumulative
  fresh review of the final Doctor+Branch+Upload+MR-list head.
- Adding MR-write always creates a new final full-spine review subject.
- Any product/test/source-doc change after an `ACCEPT` report invalidates that
  report for the new tree. Report-only descendants do not change the product
  tree, but the report must bind its exact product tree and cumulative patch
  digest.

No reviewer may also author the product conflict resolution it reviews.

## Validation boundaries

### After every product delta or conflict-resolution commit

Run syntax, the directly affected focused modules, neighboring mixed GET/POST
and response/session cases, source-doc checks, and `git diff --check`. Stop at
the first failure. Do not spend a broad gate on a rejected or unresolved
intermediate tree.

### At every stable composition head used as a later base

Run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s tests -p 'test_python_*.py'
```

Bind the exact product head/tree, command, count, duration, and environment.
If Upload and MR-list are composed in one wave, run this once after both,
provided focused checks passed after each delta.

### After MR-write and the final conflict resolution

Run all five focused HTTP modules, mixed dispatcher/filesystem/transport/
cleanup/fence/response/reducer/nonce reversals, Python-pattern discovery, and:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -v
./demo.sh
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Any later product or conflict-resolution byte invalidates these results and
requires the affected focused checks plus both broad gates again. The final
independent full-spine review follows the exact green composition head.

## GC successor gate

Python `POST …/gc` is outside this packet. Do not import its legacy
authorization, nonce, response, or publication timing into the composed
spine, and do not claim uniform POST closure.

GC may receive its own typed successor packet only after the final composed
spine is independently accepted and exposes one reusable ledger/deadline,
response/session, prepublication reducer, retryable cleanup, and exact
terminal interface. Its independent focused, boundary, broad, Install, and
Dogfood evidence remains mandatory.

## Non-Goals

- Repair or re-review an isolated Upload, MR-list, or MR-write candidate.
- Integrate a rejected/superseded candidate or an entire agent worktree.
- Reconstruct a no-Upload alternative under this packet.
- Change Rust, file transport, GC, install/packaging, devcontainer, CI, or the
  dogfood project.
- Update CFR-168/CFB issue state, history, review reports, or recovery ledgers.
- Perform Install, Dogfood, Close, Retrospect, or capability
  re-advertisement.

## Handoff

The manager handoff must include:

- all accepted candidate/report/digest bindings;
- exact composition base, final head/tree, linear commit list, and cumulative
  patch digest;
- product-delta provenance and excluded branch/management commits;
- conflict-resolution table and shared-owner decision table;
- exact changed and intentionally untouched paths;
- per-delta focused results and stable-head broad results;
- fresh composition-review commits, report hashes, and decisions;
- zero-debt/zero-permit/one-terminal and full authority snapshots;
- unresolved predecessor gates, especially GC;
- an explicit statement that this packet still does not authorize Install,
  Dogfood, Close, or capability re-advertisement.

Any unresolved accepted-SHA placeholder, `REQUEST_CHANGES`, unexplained gate
failure, unreviewed conflict byte, or imported rejected product behavior keeps
the task blocked.
