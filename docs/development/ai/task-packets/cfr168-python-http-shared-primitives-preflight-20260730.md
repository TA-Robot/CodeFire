# CFR-168 Python HTTP Shared-Primitives Adversarial Preflight

Status: blocked pending exact accepted Upload and MR-list bindings

Typed packet:
[`json/cfr168-python-http-shared-primitives-preflight-20260730.json`](json/cfr168-python-http-shared-primitives-preflight-20260730.json)

Freeze base: `c97d6488b6349ae46d4f4887a2630b2f9ff59e02`

Provider/model when unblocked: Fugu 1.1 Ultra (`fugu-ultra-v1.1`)

Role: independent architecture reviewer

No `dev-swarm.v1` manifest exists for this task. Do not launch it or begin the
MR-write closure repair while either predecessor binding remains unbound.

## Objective

Before another large MR-write implementation, determine whether the accepted
Doctor/Branch, Upload, and MR-list contracts can share one request/session/
response/publication lifecycle. Produce one exact `READY` or `BLOCK` report
whose counterexamples become the implementation oracle.

This task does not modify product code or approve a candidate. It shifts
physical ownership, effect, cleanup, authority, response, and mutation
counterexamples before implementation so the next Claude lane does not
rediscover them after a multi-thousand-line patch.

## Exact blocked inputs

| Input | Current evidence | Required binding |
|---|---|---|
| Doctor + Branch | product `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`; report `c57215e243839b224a641251fc9e3bf092f7e116` | fixed scoped input |
| Upload | candidate `76baff84cb1192e11b26dcc669fb800f7b13fa30`; exact review active | `ACCEPTED_UPLOAD_SHA`, tree, parent, patch digest, report commit and report digest are unbound |
| MR-list | rejected `38da8988ac8bc1fcdfcb82c62e7aa0d28791500c`; closure repair active | `ACCEPTED_MR_LIST_SHA`, tree, parent, patch digest, report commit and report digest are unbound |
| MR-write findings | candidate `6dd4527f23135388b53fc5c6d637c4105cc5d659`; rejecting report `b86a782b6ac72cabe684106f969681c5cd56380e` | fixed problem statement; no successor implementation is authorized yet |

The manager may change this packet to `ready` only after both predecessor
successors have immutable exact-candidate `ACCEPT` reports and every new
review finding is included. A green worker gate, draft report, predecessor
acceptance, or manager expectation is not a binding.

## Scope

Read the live Python HTTP dispatcher, the five focused Doctor/Branch/Upload/
MR-list/MR-write modules, accepted predecessor product trees and exact reports,
the rejected MR-write candidate/report, the four canonical source documents,
ADR-020, and the Python composition freeze packet.

The only authorized write is:

`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-fugu-20260730/preflight.md`

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

- all accepted predecessor identities and reports recompute exactly;
- every shared primitive has one compatible ownership/lifetime contract;
- each F1–F7 MR-write finding maps to a production counterexample and expected
  repair seam;
- every material property has a non-vacuous oracle and deliberate regression;
- the proposed MR-write product/test/source-doc write set is bounded;
- no unresolved predecessor finding or incompatible accepted behavior must be
  guessed by the implementer.

Return `BLOCK` with exact symbols, revisions, and conflicting semantics if any
condition is missing. Do not invent a weaker third model to force `READY`.

## Non-Goals

- Repair or accept any isolated candidate.
- Compose or integrate the Python HTTP product tree.
- Resolve manager-owned product conflicts.
- Design Python GC, Rust, file transport, packaging, install, or dogfood work.
- Run broad suites whose result cannot change the preflight decision.
- Infer future accepted predecessor identities before exact reports exist.

## Validation

1. Recompute accepted Upload and MR-list parent, head, tree, patch, changed
   paths, report commit, report SHA-256, and clean-state evidence.
2. Trace every claimed primitive through the production dispatcher and
   distinguish test seams from live reachability.
3. Bind every material claim to an independent observable oracle and a
   deliberate regression, plus exact boundaries and scaling where applicable.
4. Reconcile every MR-write F1–F7 finding and every newer predecessor finding.
5. Before handoff, require the launch-to-head diff and worktree status to
   contain only the declared report.

## Handoff

When unblocked, write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-shared-primitives-preflight-fugu-20260730/preflight.md`

The report must contain:

- recomputed predecessor identity table;
- `READY` or `BLOCK`;
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
