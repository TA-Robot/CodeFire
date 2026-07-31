# CFR-168 Python HTTP Shared-Primitives Adversarial Preflight

Status: logically ready; provider launch held by Claude backoff and live admission

Typed packet:
[`json/cfr168-python-http-shared-primitives-preflight-20260730.json`](json/cfr168-python-http-shared-primitives-preflight-20260730.json)

Freeze base: `2c439d8b483c14919633036741af43be221e98cc`

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: independent architecture reviewer

No `dev-swarm.v1` manifest exists for this task. The report-only preflight may
run now. The MR-write closure repair remains blocked until the preflight
commits an exact `READY` decision and the manager binds its commit and digest.

## Objective

Before another large MR-write implementation, determine whether accepted
Doctor/Branch and MR-list behavior plus the immutable counterexamples from
rejected Upload, MR-write, and Storage candidates define one compatible
request/session/response/publication lifecycle. Produce one exact `READY` or
`BLOCK` report whose counterexamples become the implementation oracle.

This task does not modify product code or approve a candidate. It shifts
physical ownership, effect, cleanup, authority, response, and mutation
counterexamples before implementation so the next Claude lane does not
rediscover them after a multi-thousand-line patch.

Rejecting reports are specifications only. They authorize no product byte.
Every primitive clause must cite the source report SHA-256, record whether the
finding is behavior-open, behavior-closed, evidence-unproven, or claim-only,
and mark a primitive `shape-pending` when its exact implementation still
depends on an open residual.

## Exact bound inputs

| Input | Bound evidence | Authority in this preflight |
|---|---|---|
| Doctor + Branch | product `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`; accepted report `c57215e243839b224a641251fc9e3bf092f7e116` | fixed scoped product contract |
| MR-list | candidate `b16404676639336f705feffd9af4857d4a1fcc69`; accepted report `ee6517a`, report SHA-256 `5c952a9a5c162bfbb93522bb644fcba44ce32ddf3628501e6f94308899fe760e` | fixed scoped product contract plus four manager-routed observations |
| Upload generation 1 | rejected candidate `76baff84cb1192e11b26dcc669fb800f7b13fa30`; report `f835f64`, SHA-256 `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` | immutable negative cases and closed/open disposition history only |
| Upload generation 2 | rejected candidate `467b0c27715e424ae18ffb5664da053b55915e2d`; report `8bdc076`, SHA-256 `57d6792106f09c17a1d154aedb5a64d342e5d19c23db7e9963c46061c868c132` | immutable negative cases and closed/open disposition history only |
| Upload generation 3 | rejected candidate `02e04c7e6458035d9c7d58de4d30b4a12e7cd594`; report `b3de0d7`, SHA-256 `3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f` | latest immutable Upload specification; retryable cleanup remains `shape-pending` |
| MR-write | rejected candidate `6dd4527f23135388b53fc5c6d637c4105cc5d659`; report `84175bd`, SHA-256 `214e42fc8f36108f45e17b13e4343bfc2b56cc89b72be5c1258f5b497c9ec6ef` | F1–F7 fixed problem statement; no successor product authority |
| Storage | rejected candidate `cdcb9bd`; report `8ac6741`, SHA-256 `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` | cross-runtime invariant-family counterexamples only; no Rust design import |

The currently reviewed Upload repair may be inspected only as a hypothesis.
It cannot freeze a primitive shape or supply an accepted product binding.
The preflight no longer waits for Upload acceptance because it writes no
product byte and needs negative specifications, not an integration candidate.

## Scope

Read the live Python HTTP dispatcher, the five focused Doctor/Branch/Upload/
MR-list/MR-write modules, accepted predecessor product trees and exact reports,
the rejected MR-write candidate/report, the four canonical source documents,
ADR-020, and the Python composition freeze packet.

The only authorized write is:

`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`

Product code, tests, source documents, issue/lifecycle state, task packets,
manifests, existing reports, Rust, file transport, packaging, devcontainer,
and dogfood source are read-only or out of scope.

## Required preflight model

### Physical owner and lifetime table

For every live production entry point and resource, name:

- creating function and exact request ledger;
- allocation/read/open/acquire point;
- retained and temporary formula;
- permit or descriptor identity;
- borrowers and physical rereads;
- transfer, selection, publication, release, retry, and final terminal owner;
- error precedence when release or terminalization also fails.

At minimum include project descriptors, branch/MR/generation authority,
sealed object snapshots, signature and nonce records, response bodies and
headers, publication preimages/postimages, object-copy plans, reducers,
pending/recovery/receipt/audit state, and cleanup debt.

### Effect and response timeline

Build one event sequence for create, review, and apply:

```text
request parse
→ authority/session capture
→ response pair preparation
→ publication plan/postimages
→ prepublication seal
→ final authority fence
→ first authoritative effect
→ reducer confirmation/recovery
→ response selection
→ first byte
→ cleanup/terminal observation
```

For every fallible step, state whether authority must be byte-identical,
durably recoverable, already committed, or transport-abandoned. No response
may be selected after an effect in a way that replaces the committed outcome.

### Shared primitive decision

Decide `reuse`, `adapt`, or `incompatible` for:

- `RequestContext`: one handler ledger plus deadline and route phase;
- `SealedSnapshotSession`: one physical capture with purpose-bounded borrows;
- `AuthorityFence`: last complete project/session/subject observation;
- `PreparedResponse`: success/durable pair, transactional selection, one
  first-byte publication point;
- `RetryableCleanup`: an unreleased amount remains owned until retry succeeds;
- `PublicationTxn`: nonce, object, escrow, pending, recovery, receipt, audit,
  reducer confirmation, and replay;
- `ErrorOrigin`: filesystem, transport, deadline, budget, integrity, and
  postcommit cleanup precedence.

Surface-local replacement is `BLOCK` unless an exact semantic incompatibility
is demonstrated.

## Mandatory counterexample matrix

Every row must identify a real production entry point, independent observable
oracle, and deliberate regression. Source-shape or helper-call counters alone
are insufficient.

1. Omit each physical owner before its first allocation/read/open.
2. Restore one second physical read after sealed snapshot capture.
3. Move response preparation after escrow or publication.
4. Move one object/postimage allocation before its reservation.
5. Mutate branch, MR, generation, policy, signature, object, project
   descriptor, mode/link/inode, or readable state after the last observation.
6. Fail every first cleanup release, then retry the same exact owner.
7. Partially consume response/header/body permits and attempt durable fallback.
8. Fail the second of several object writes and then exact-retry.
9. Fail reducer confirmation before and after nonce persistence.
10. Alternate Doctor, Branch, Upload, MR-list, and MR-write on one server and
    prove request flags, ledgers, phases, debt, and responses do not leak.
11. Exercise exact, minus-one, and plus-one for every charged dimension while
    proving the intended owner—not an earlier limit—fails.
12. Run 1x/3x/10x logical-record, physical-read, descriptor, byte, retained,
    temporary, permit, and deadline-poll curves.

Deleted whole helpers, constant labels, callbacks that skip the production
route, incomplete snapshots, or blanket terminal sweeps do not count as
deliberate-regression proof.

## READY / BLOCK decision

Return `READY` only when:

- the accepted Doctor/Branch and MR-list identities and every specification
  report commit/SHA-256 recompute exactly;
- every shared primitive has one compatible ownership/lifetime contract;
- every clause identifies its source report digest and evidence state;
- any exact shape depending on the open Upload cleanup residual is explicitly
  `shape-pending`, while its invariant and rejecting counterexample are fixed;
- each F1–F7 MR-write finding maps to a production counterexample and expected
  repair seam;
- every material property has a non-vacuous oracle and deliberate regression;
- the proposed MR-write product/test/source-doc write set is bounded;
- no unresolved specification conflict or incompatible accepted behavior must
  be guessed by the implementer; and
- the report states explicitly that `READY` authorizes only the MR-write
  repair packet, not Upload integration or any rejected product bytes.

Return `BLOCK` with exact symbols, revisions, and conflicting semantics if any
condition is missing. Do not invent a weaker third model to force `READY`.

## Non-Goals

- Repair or accept any isolated candidate.
- Compose or integrate the Python HTTP product tree.
- Resolve manager-owned product conflicts.
- Design Python GC, Rust, file transport, packaging, install, or dogfood work.
- Run broad suites whose result cannot change the preflight decision.
- Infer a future accepted Upload identity or freeze its pending cleanup shape.

## Validation

1. Recompute accepted Doctor/Branch and MR-list identities plus every rejecting
   specification report commit and SHA-256. Recompute candidate identities
   only to bind the counterexample subject; never treat them as accepted.
2. Trace every claimed primitive through the production dispatcher and
   distinguish test seams from live reachability.
3. Bind every material claim to an independent observable oracle and a
   deliberate regression, plus exact boundaries and scaling where applicable.
4. Reconcile every MR-write F1–F7 finding and every newer predecessor finding.
5. Before handoff, require the launch-to-head diff and worktree status to
   contain only the declared report.

## Handoff

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md`

The report must contain:

- recomputed accepted-input and rejecting-specification identity table;
- `READY` or `BLOCK`;
- per-clause report SHA-256, evidence state, and `frozen`/`shape-pending`
  marker;
- production call graphs and owner/lifetime table;
- effect/response timelines;
- shared primitive decisions and incompatible semantics;
- exact counterexample/mutation matrix;
- focused validation plan and expected failure reasons on `6dd4527`;
- bounded MR-write implementation write set and conflict map;
- residual predecessor gates and next action.

Commit only the report. The manager must bind its commit and SHA-256 into the
MR-write closure packet before creating or starting that implementation
manifest.

## Launch Hold

Do not start merely because this packet is logically ready. Before launch the
manager must:

1. obtain one successful bounded Claude Opus 5 health probe after the recorded
   provider backoff;
2. preserve priority for the exact Cargo, Upload, and KGI/KPI review resumes
   that release existing candidates;
3. rerun live provider-window and
   `docs/development/tools/check_swarm_admission.py` evidence at the actual
   number of outstanding Claude lanes;
4. verify the exact freeze base, declared write set, and absence of a competing
   shared-primitives session; and
5. record that this new report-only lane fits the current adaptive window and
   resource floor.

Provider recovery alone is not admission. If any check fails, preserve the
manifest and leave the task held without falling back to Codex.
