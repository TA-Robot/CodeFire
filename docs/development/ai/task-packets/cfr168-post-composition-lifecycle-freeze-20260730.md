# CFR-168 Post-Composition Lifecycle Freeze Task Packet

Status: blocked pending accepted Python and Rust compositions, accepted
successor surfaces, and an exact final integrated revision

Typed packet:
[`json/cfr168-post-composition-lifecycle-freeze-20260730.json`](json/cfr168-post-composition-lifecycle-freeze-20260730.json)

## Objective

Run the manager-owned CFR-168 lifecycle from one frozen post-composition
subject through Close, isolated Install, canonical and real-repository
Dogfood, and Retrospect. The result is acceptable only when every claim,
review, release artifact, installed reversal, dogfood result, issue
transition, and KGI/KPI record is bound to one exact terminal revision.

This packet does not authorize more product implementation. It freezes the
evidence state machine that may run after the Python composition, Rust
composition, and all predecessor-gated CFR-168 / CFB-136 / CFB-138 successor
surfaces have independent acceptance.

## Exact Freeze Basis

| Subject | Exact value at packet freeze |
|---|---|
| Control-plane base revision | `740ea18eb42630ba265a8aad83c950ceb96aacca` |
| Control-plane base tree | `40ec9ebe9ab30fa63858ee3d4dc4b17f38e67fe8` |
| Immutable pre-recovery product snapshot | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Immutable product-snapshot tree | `9be2b8aca8a8538d12852b835b95aafb8601f4a4` |
| Recovery ledger SHA-256 | `af8d5dccd9993d1d8e16f481a4ba74996e356641d9c1bbe06a82cc85bec11422` |
| Review-disposition index SHA-256 | `76de456efc526b53086878b958f69b6c10ddc2aee970bab89b8323c403a21294` |

At this freeze, `740ea18` is byte-identical to `e2a218e` across `codefire`,
`crates/`, `tests/`, `Cargo.toml`, and `Cargo.lock`. No post-composition
product SHA is accepted or named. That absence is an intentional launch
blocker, not a value the executor may infer from a branch tip, worktree, WIP
commit, or passing candidate log.

No `dev-swarm` manifest or launch is authorized for this blocked packet.

## Manager-Only Authority

Only the manager may:

- bind the accepted Python and Rust composition SHAs;
- bind each accepted successor-surface SHA and independent decision;
- choose and create the integrated revision;
- resolve integration conflicts and request a fresh composition review;
- move this packet from `blocked` to `ready`;
- recognize a Close, Install, Dogfood, or Retrospect exit;
- mutate authoritative issue, map, history, capability, or KGI/KPI state; or
- declare the batch terminal.

A validation runner, dogfood runner, or independent reviewer may produce
read-only evidence under an exact manager-bound subject. It cannot select a
candidate, waive a finding, update issue state, advertise a capability, or
approve its own implementation.

## Launch Blocker

The manager must amend and commit both packet files before changing the
status to `ready`. The amendment must contain all of the following exact,
verified values:

1. the accepted Python composition SHA, tree, selected commit ranges,
   changed-path set, patch digest, and independent exact-candidate decision;
2. the accepted Rust composition SHA, tree, selected commit ranges,
   changed-path set, patch digest, and independent exact-candidate decision;
3. the accepted SHA and decision for every successor surface below;
4. the integrated SHA and tree containing only those accepted ranges, plus a
   clean tracked and untracked status;
5. the exact versioned Rust/Python conformance corpus and the exact installed
   reversal runner with repository path and command line;
6. the accepted KGI/KPI validator and committed-final provenance behavior;
7. the exact capability re-advertisement commit, changed paths, and
   independent decision, held back as the last product mutation; and
8. the exact canonical Algorithm subject plus every read-only real-repository
   dogfood subject and its pre-run authority/hash inventory.

The successor-surface set is:

1. Python `POST …/gc` request ledger, deadline, and pre-effect response;
2. Python local `common_ancestor` / `ancestor_distances` parity;
3. Rust local `cf merge` ancestry;
4. Rust `cf scan` validation ledger/session;
5. Rust local Doctor open-summary, evidence, and active-state ownership;
6. file clone, storage-report, and GC graph walks;
7. accepted
   `cfr168-final-decision-work-metrics-projection-freeze-20260730` successor
   closing cache invalidation/entry metrics, independent metrics
   orthogonality, command-wide decision-work projection, and the final CLI
   envelope/text bytes;
8. accepted `cfr168-final-complexity-conformance-freeze-20260730` successor
   binding the canonical asymptotic/order table and shared cross-runtime
   corpus;
9. file transport branch/generation/plan authority;
10. HTTP-client Upload accounting;
11. `remote::publication::validate_object_roots`; and
12. accepted declaration-only
    `cfr168-final-capability-readvertisement-freeze-20260730` successor,
    applied as the truthful final capability re-advertisement.

The disposition index must also have a terminal, exact disposition for every
formal review report and finding. An accepted nucleus with unresolved
Should/Could findings, an interrupted transcript, a candidate-only broad
gate, or an unreviewed conflict-resolution commit does not satisfy this
blocker.

## Lifecycle State Machine

| Stage | Exact entry evidence | Exact exit evidence |
|---|---|---|
| Close | accepted Python/Rust compositions and every successor are integrated; exact SHA/tree/patch digest are frozen; tree is clean; capability is not newly advertised | full Python/Rust/conformance/docs/diff gates pass on that SHA; independent High review accepts that exact SHA and evidence; source is only a `source-close candidate` |
| Install | accepted source-close SHA is unchanged and clean; release inputs and commands are frozen | two isolated release builds agree; provenance and hashes are recorded; temp-prefix active binary is exact; installed boundary/scale reversals pass |
| Dogfood | installed binary path, digest, provenance, source SHA, and temp prefix are unchanged | canonical Algorithm tests and installed workflow pass; real-repository read-only matrices show no authority drift; every finding is routed |
| Retrospect | Dogfood passes with no unresolved same-root blocker | issue/map/history/index/ledger and final KGI/KPI record agree; final capability mutation is reviewed and fully revalidated; terminal SHA/tree/install/dogfood evidence is fresh and clean |

No stage may be skipped or credited from a different SHA. A source change
returns the task to Close. A release-binary change returns it to Install. A
dogfood-subject or command change returns it to the affected Dogfood entry.

## Close Contract

### Entry

The manager records:

- exact integrated commit and tree;
- both composition heads and all selected accepted ranges;
- `git status --porcelain=v1 --untracked-files=all` as empty;
- complete `e2a218e..HEAD` name-status and binary patch SHA-256;
- exact issue/finding-to-commit disposition;
- no unreviewed merge, branch-tip import, duplicate recovery commit, or
  candidate worktree residue; and
- a capability probe proving no new final coverage claim has been exposed.

### Required source gates

Run in this order on the frozen integrated SHA:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -v
./demo.sh

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

PYTHONDONTWRITEBYTECODE=1 python3 schemas/conformance/minimal_consumer.py \
  schemas/conformance/manifest-v1.json
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_checked_conformance_manifest_matches_python_support_decisions
cargo test --locked -p codefire-cli --bin codefire-rs \
  checked_conformance_manifest_matches_rust_support_decisions

PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_source_docs.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_ai_development.py --timeout 120
git diff --check e2a218e015953f082245f36dbc6aee906b837e5a...HEAD
```

The accepted successor must extend the versioned shared corpus so the
conformance commands cover common-ancestor selection, required-link
diagnostics, sealed/signature traversal, object transfer, every selected
local/HTTP typed budget field, metrics on/off equivalence, and capability
coverage in both runtimes. Passing the older generic corpus alone is
insufficient.

Focused production-path evidence must additionally cover every selected
surface at limit minus one, exact limit, and limit plus one; 1x/3x/10x
counter curves; stable first-error order; no partial response or mutation;
and explicit time/memory complexity. Focused logs name the exact production
entry point and may not substitute helper-only or test-observer behavior.

### Independent High review

After all source gates pass, an independent reviewer who did not author or
compose the product revision reviews the exact integrated SHA. The committed
review packet must bind:

- base and head commit/tree;
- exact changed paths and canonical patch digest;
- every required validation command, exit status, count, duration, and log
  digest;
- composition conflict resolutions and all disposition-index findings;
- Rust/Python parity, order, counter, memory, deadline, cleanup, and
  authority-safety arguments;
- the capability non-claim before final re-advertisement; and
- one decision of `ACCEPT` or `REQUEST_CHANGES`.

Anything other than exact `ACCEPT` returns to Freeze/Plan. Review freshness
is lost after any source, test, corpus, schema, documentation, or
capability-declaration change.

### Exit

Close exits only as an immutable `source-close candidate`. CFR-168, CFB-136,
and CFB-138 remain `investigating`; no final capability coverage is newly
advertised; no later-root task is authorized.

## Install Contract

### Entry

The source-close SHA and tree equal the independently accepted subject, the
worktree is clean, all source gates remain green, and no build input differs
from HEAD.

### Release provenance and reproducibility

Run the source-bound release builder in two disposable clean worktrees at
the same exact SHA, with the same locked dependency graph and recorded
toolchain. Each run must produce:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/build_source_bound_rust.py \
  --output target/codefire-ai/rust-build-provenance.json
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/probe_validation_capabilities.py
```

Record each source revision, source-manifest SHA-256, source-file count,
Cargo and rustc version, build command, build-log SHA-256, binary byte count,
binary SHA-256, provenance SHA-256, duration, and worktree status. The two
binary digests must agree. The local provenance helper alone is not a claim
of trusted or supply-chain-reproducible build output; disagreement is a
release blocker, not evidence to average or select away.

Use sequential builds and remove only manager-created disposable build
outputs after their compact evidence is committed. Do not overlap full
builds or retain duplicate `target/` trees when resource guardrails would be
crossed.

### Isolated install and active-binary identity

Install one verified build into a manager-created temporary prefix:

```bash
CFR168_PREFIX="$(mktemp -d /tmp/codefire-cfr168-final.XXXXXX)"
./install.sh --prefix "$CFR168_PREFIX" \
  --binary "$PWD/target/release/codefire-rs"
test "$(sha256sum "$PWD/target/release/codefire-rs" | awk '{print $1}')" = \
  "$(sha256sum "$CFR168_PREFIX/bin/codefire" | awk '{print $1}')"
test "$(PATH="$CFR168_PREFIX/bin:$PATH" command -v codefire)" = \
  "$CFR168_PREFIX/bin/codefire"
"$CFR168_PREFIX/bin/codefire" --version
"$CFR168_PREFIX/bin/codefire" --help
"$CFR168_PREFIX/bin/codefire-py" --help
"$CFR168_PREFIX/bin/codefire" capabilities --json
```

The evidence records the prefix, resolved active path, real path, source and
installed hashes, permissions, version/help fingerprints, and capability
payload digest. Every later installed command rechecks the active path and
binary digest immediately before execution.

### Installed reversal matrix

The exact accepted installed-runner path and command are a launch-blocker.
It must exercise the installed binary, not source helpers, and cover:

- limit minus one, exact limit, and limit plus one for every advertised
  dimension and selected production surface;
- 1x/3x/10x stable counters and the frozen asymptotic/order table;
- CFB-136 corruption/cache/final-revalidation reversals;
- the CFB-138 default-profile root-plus-1200-descendant upload;
- local and HTTP diagnostic parity, first-error order, and exit 60 / HTTP
  413;
- deadline/connection admission before effect;
- response exact/+1 with no partial success header/body;
- failed upload/MR/GC with no nonce, branch, object, transaction, or GC
  authority drift;
- metrics enabled/disabled semantic identity and zero unclosed debt; and
- the pre-advertisement capability non-claim.

Each case stores exact argv, fixture digest, expected literal oracle, exit
status, normalized output digest, before/after authority hashes, counters,
peak retained work, and duration.

### Exit

Install exits with one immutable install record bound to the accepted source
SHA, matching two-build binary digest, provenance digests, exact prefix and
active path, complete passing reversal matrix, and no source or authority
drift.

## Dogfood Contract

### Canonical Algorithm

First run all current Algorithm tests and record the exact discovered count:

```bash
(cd subprojects/algorithm-evolution-agent-lab && \
  PYTHONPATH=src python3 -m unittest discover -s tests -v)
```

Then use only the installed binary whose path and digest passed Install:

```bash
"$CFR168_PREFIX/bin/codefire" status \
  --path subprojects/algorithm-evolution-agent-lab --json
"$CFR168_PREFIX/bin/codefire" scan \
  --path subprojects/algorithm-evolution-agent-lab --json
"$CFR168_PREFIX/bin/codefire" verify \
  --path subprojects/algorithm-evolution-agent-lab --json
"$CFR168_PREFIX/bin/codefire" doctor \
  --path subprojects/algorithm-evolution-agent-lab --json
```

Before and after the sequence, record Git HEAD/tree/status plus CodeFire
commit, branch, object, open-state, and active-state inventories and hashes.
The Algorithm source/tests/configuration may not be edited to make CodeFire
pass. The expected CodeFire authority transition must be frozen before the
mutating commands; any other drift blocks Dogfood.

### Real-repository matrix

The unblock amendment names every configured and plain real repository by
canonical path, immutable Git subject, and pre-run authority inventory.
Only installed read-only commands are permitted. For every subject:

- record active binary path/digest and exact argv;
- run the frozen status, storage, Doctor, ancestry, and remote-read probes;
- record stdout/stderr/exit/duration digests;
- re-hash Git refs/index/worktree plus CodeFire branch/object/open/active
  authority; and
- require byte-identical before/after authority state.

No network mutation, remote publication, cleanup, migration, scan, verify,
or fixture rewrite belongs to the real-repository read-only matrix.

### Exit

Dogfood exits only when the Algorithm tests and installed workflow pass,
every real-repository probe is read-only and drift-free, the installed
binary identity remains exact, and every observation has a manager
disposition.

## New-Finding Routing

Every new observation receives a stable ID, exact subject, reproducer,
severity, invariant, evidence digest, owner, and disposition before the
lifecycle can advance.

- A CFR-168 / CFB-136 / CFB-138 same-root defect, acceptance-gate defect,
  evidence-integrity defect, capability overclaim, data/authority mutation,
  or flaky release gate is blocking. Return to Discover/Freeze/Plan, create
  or amend the typed repair packet, and invalidate all downstream evidence.
- A distinct product defect receives a canonical CFB/CFR issue, root-cause
  mapping, dependency placement, and explicit manager decision. It may be
  non-blocking for this root only when the frozen invariants and all
  acceptance gates remain true.
- Environment, provider, memory, disk, timeout, or tooling incidents go to
  the flow record with lost-work and recovery measurements. They never
  become a product pass or a reason to weaken a gate.
- “No new finding” is an explicit evidence-backed Retrospect result, not the
  absence of a note.

## Scope

The manager may read the complete product, lifecycle, conformance, issue,
release, and dogfood subjects needed to verify the exact terminal revision.
Writes are limited to:

- this packet's blocked-to-ready exact binding;
- committed review and flow-metric evidence;
- the CFR-168 / CFB-136 / CFB-138 authoritative lifecycle documents;
- the separately reviewed capability re-advertisement paths named in the
  typed packet; and
- expected CodeFire state beneath the canonical Algorithm subject during its
  explicitly authorized mutating Dogfood sequence.

All remaining product implementation, installer/build scripts, Cargo
identity, Algorithm source/tests/docs/configuration, other review reports,
devcontainer/CI files, and real-repository subjects are read-only. If the
accepted capability commit needs a path outside the typed write set, the
packet stays blocked until the manager amends and revalidates the write set.

## Validation

The typed packet freezes every required command and evidence output. While
the packet is blocked, its installed-reversal and real-repository rows
deliberately state that no accepted command is bound; those rows must be
replaced with exact repository-local commands and subjects in the committed
blocked-to-ready amendment.

Before that amendment or any lifecycle execution, validate this packet
itself:

```bash
python3 -m json.tool \
  docs/development/ai/task-packets/json/cfr168-post-composition-lifecycle-freeze-20260730.json
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_ai_acceleration_docs.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_project_docs.py
git diff --check
```

Passing these documentation/contract checks validates the freeze packet
only. It is not CFR-168 product, Close, Install, Dogfood, or Retrospect
evidence.

## Retrospect Contract

### Authoritative synchronization

Only after Dogfood passes, synchronize:

- `docs/development/code-review-issues/cfr-168.md`;
- `docs/development/bug-issues/cfb-136.md` and `cfb-138.md`;
- the CFR summary ledger, bug backlog, root-cause map, current-cycle plan,
  known limitations, history, recovery ledger, and review-disposition
  index;
- the committed final review packet and evidence digests; and
- the KGI/KPI flow record.

All three issue statuses remain open until the final capability revision has
fresh Close, Install, and Dogfood evidence. Counts, status, root, evidence,
and dependency fields must agree across every canonical owner.

### KGI/KPI final record

Create a new immutable final version; do not rewrite an earlier final
record. It must bind:

- exact source and terminal revision;
- append-only event-prefix bytes and SHA-256;
- stage entry/exit instants and lead times;
- trusted-signal, candidate, review, integration, Close, Install, Dogfood,
  and Retrospect events;
- first-pass rate, correction/rework, queue and integration time, useful
  parallelism, validator time, resource incidents, lost work, recovery time,
  provider routing/cost, and Codex coordination usage;
- `KGI-DEV-001` and every applicable KPI as achieved, missed, or
  inconclusive with numerator, denominator, uncertainty, and evidence;
- management decisions and the next review due date; and
- committed-final Git provenance accepted by the repaired KGI/KPI
  validator.

Schema, semantic, lineage, due-date, evidence, digest, and Git-provenance
checks must pass on the committed record. Generated files, commit count,
agent count, or model approvals never count as progress.

### Final capability re-advertisement

Capability re-advertisement is the last product mutation. It is permitted
only after every prior Close, Install, Dogfood, issue-sync, and KGI/KPI
precondition passes, and only by applying the separately accepted exact
capability commit.

Because that mutation creates a new SHA, no earlier source, review, release,
install, reversal, or dogfood evidence may be transferred to it. On the new
terminal SHA the manager must rerun:

1. the complete Close source/conformance/docs/diff gate;
2. a fresh independent High exact-SHA review;
3. both release builds and provenance comparison;
4. a fresh isolated temp-prefix Install and hash/active-path proof;
5. the complete installed reversal matrix;
6. canonical Algorithm and real-repository Dogfood; and
7. capability schema, help, manifest, cross-runtime conformance, and
   truthful-coverage checks.

Validation after this last product mutation is allowed; another product
mutation is not. Any failure withdraws terminal status and returns to
Freeze/Plan. Only after the rerun passes may the manager mark CFR-168,
CFB-136, and CFB-138 fixed, finalize the KGI/KPI record, and authorize the
next frozen root.

### Terminal exit

Retrospect exits with:

- exact terminal commit/tree and empty worktree status;
- complete accepted ancestry from `e2a218e`;
- fresh independent review for that terminal SHA;
- matching two-build, source, and installed binary hashes;
- passing installed reversals and Dogfood on the same binary;
- truthful final capability output and conformance;
- zero unresolved CFR-168 / CFB-136 / CFB-138 finding;
- synchronized issue/map/history/index/ledger state;
- one valid committed final KGI/KPI record; and
- either routed new findings or an explicit evidence-backed no-new-finding
  result.

## Non-Goals

- Do not implement, refactor, or opportunistically repair product behavior
  under this lifecycle packet.
- Do not review the devcontainer, CI, host setup, agent wrappers, packaging
  design, or unrelated later roots.
- Do not merge whole candidate branches or recovery worktrees.
- Do not edit Algorithm source/tests/configuration to make Dogfood pass.
- Do not turn local build provenance into a supply-chain attestation claim.
- Do not close issues, advertise coverage, or declare KGI achievement from a
  source-close candidate.
- Do not reuse candidate, predecessor, stale-review, stale-install, or
  pre-capability evidence for a changed SHA.

## Handoff

The manager commits a compact evidence index containing:

- freeze/unblock revision table and selected ranges;
- issue/finding disposition matrix;
- source gate and independent-review table;
- release reproducibility and provenance table;
- source/installed/active binary identity table;
- installed boundary/scale reversal table;
- canonical and real-repository dogfood table;
- new-finding routing;
- issue/map/history/index/ledger synchronization diff;
- final KGI/KPI record and validator results;
- final capability payload and conformance digest;
- command counts, durations, log digests, peak resource samples, and cleanup
  record; and
- the exact terminal readiness decision.

Any missing exact value, stale subject, unresolved finding, failed required
command, unexplained authority drift, non-matching binary hash, or
non-decision-grade KGI/KPI record means `REQUEST_CHANGES`.
