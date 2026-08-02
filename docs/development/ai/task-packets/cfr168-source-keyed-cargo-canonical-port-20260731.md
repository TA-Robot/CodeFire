# CFR-168 Source-Keyed Cargo Gate Canonical Port

Status: source and port both independently accepted; canonical integration is
held pending a fresh exact review of the 2026-08-02 O1 evidence-authority
repair. Not integrated.

Owner: portfolio manager  
Execution role: Claude Opus 5 implementer, fresh Claude Opus 5 reviewer  
Canonical packet base: `c273a0c6cb231d000d6289f6de5615fb9f1320be`  
Codex worker fallback: forbidden  
Product mutation: forbidden

## Objective

Port the independently accepted source-keyed Cargo-gate implementation onto a
fresh then-current canonical head, obtain an exact review of that port, and
integrate it without waiting for the independent KGI/KPI candidate.

An accepted canonical port releases the source-bound Rust validation critical
path. KGI/KPI is deliberately a later composition dependency, not a
prerequisite for this port.

## Scope

- Source range
  `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6..1bbcfde2acc60476b5e29136a30f7c808fb6bb4e`
  (5 commits: `bfbbc2b`, `aaa39b6`, `4e5c0e7`, `74cd37d`, `1bbcfde`).
- Source head tree `753c44216e0c1145fd97f33d3863c111f49eff2c`; source-range
  binary patch SHA-256
  `03b54705f09cc83b75598560b42fecd972650e2d2e08de0f0d25c152547c2e56`.
- The exact 20 source-range paths plus one port result, i.e. 21 port paths.
- Semantic port from a fresh canonical worktree, complete Python-only
  validation, and a fresh independent exact-port review.
- Preservation of all canonical provider incidents, review-root-cause gates,
  KGI/KPI work already present in canonical, and current routing policy.

## Non-Goals

- No KGI/KPI candidate port; it has its own later composition packet.
- No Cargo, Rust, CodeFire product, install, dogfood, packaging, capability, or
  issue-closure mutation.
- No whole-branch merge or complete-history cherry-pick.
- No use of an implementer result, worker gate, or clean tree as an acceptance
  verdict.
- No automatic Codex fallback.

## Accepted Authority

The former admission blocker is discharged. Both halves of the chain now carry
their own fresh independent exact `ACCEPT`.

### Source `ACCEPT` — `1bbcfde`

| Field | Value |
|---|---|
| Source head | `1bbcfde2acc60476b5e29136a30f7c808fb6bb4e` |
| Source tree | `753c44216e0c1145fd97f33d3863c111f49eff2c` |
| Source parent | `74cd37d5e9204c45ac23bf31aa6973d6004cc0f0` |
| Two-path `parent..head` binary patch SHA-256 | `de42c57eaad99307fef9ccb133d262123307aa5519ef53918cf4460dafc145c8` |
| Reviewer session | `9fac2a66-27ff-4be6-9551-b83e662fd31c`, agent `aia-cargo-final-oracle-rereview-claude-20260802`, `2026-08-02T14:40:04.301989+00:00`, `is_error` `false` |
| Verdict | `ACCEPT` |

That verdict is now durable in canonical at
`docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-fourth-closure-rereview-claude-20260802/review.md`,
published 2026-08-02 to close observation `O1` of the exact port review:

| Digest | SHA-256 |
|---|---|
| Published file | `624d2d0c40530a5c2fd823a21f6c47a8508d220fc068c298631cf36581038481` |
| Verbatim report body inside it | `7b81cf2ab150d553cbd80a74c151fbaefcb5d90ec3201fd7d7b25da1e04dd9bd` |
| Capture `transcript.jsonl` | `2ec8b17a4f4bf33a2b5c71c5d335e5291a6d61a998f3371bc759b3bbc80acd0a` |
| Capture `events.jsonl` | `18d89bfb1647a4ce71b3d79aa7819d237ac50857dbb6a28a5a83861b3714603b` |
| `jq -j .response` bytes | `69bc03a6270e38a960d0e6191186faab2ec7e38c5e4d4d78efeb3620462f58f3` |

### Canonical-port `ACCEPT` — `fb891b0`

| Field | Value |
|---|---|
| Canonical base / candidate parent | `41dea1488bf04c8a2bc1c1d1ad3df7ac9b2570f6` |
| Candidate head | `fb891b01f45e4d1963c48f06b827f8335150dace` |
| Candidate tree | `2dadc3088c14fb624fb30d8060eba7466297959a` |
| Binary patch SHA-256 (`git diff --binary base candidate`) | `ba67a8278a20e3624b5cb98a99a1725456b92e112ef3b6a34f7b118afa3c6b1f` |
| Changed paths | 21 = the 20 source-range paths plus the port result |
| Exact port review | `8316956` → `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-rereview-claude-20260802/review.md` |
| Port review report SHA-256 | `4dc9cd3c5b1cdbfb8938b744ac06730e56d99aafa832a12d7dab70cdf6b51eea` |
| Verdict | `ACCEPT` |

### Integration state

The accepted chain is **not integrated**. Canonical does not yet contain
`fb891b0`. This packet records accepted evidence only; it is not an
integration record and closes no issue.

The chain becomes manager-fast-forward-ready **only after this 2026-08-02 O1
evidence-authority repair itself receives its own fresh exact review**. That
repair is a separate exact subject and may not be self-certified by the same
authority it publishes.

No Rust lane release, and no held Rust review may cite this Cargo evidence,
until canonical contains the **entire** accepted chain — the source `ACCEPT`
publication, the accepted port `fb891b0`, its exact review `8316956`, and the
reviewed O1 repair.

### Superseded history — the 2026-07-31 quota incident

Retained as history, not as a current blocker. The original admission blocker
named exact candidate
`4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` and recorded that reviewer session
`d089c8ae-8364-49af-bcd1-b9bf4b741ea1` stopped on Claude quota before writing
any report or verdict, with the instruction to rebind this packet to the
accepted successor if that line did not clear. That is exactly what happened:
`4e5c0e7` never received a verdict, the work advanced through `74cd37d` to
`1bbcfde`, and `1bbcfde` carries the accepted authority recorded above. The
`d089c8ae` session is closed and superseded; it is not to be resumed, and the
stale `4e5c0e7`/19-path binding is superseded everywhere in this packet.

## Why A Port Is Required

Accepted source `1bbcfde` is five commits beyond historical base
`893cbcad2b1f2a8455a5c256246c8110d3fcbdd6`:

```text
bfbbc2b2bdc22f88679e5e1dc794665d863304a1
aaa39b61b37df273fe4d57e274ed8e1321e788ad
4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912
74cd37d5e9204c45ac23bf31aa6973d6004cc0f0
1bbcfde2acc60476b5e29136a30f7c808fb6bb4e
```

The current canonical line has newer management, routing, provider-incident,
and review-method evidence. Cherry-picking or merging the stale history would
overwrite or conflict with that evidence and would not prove the combined
tree.

## Manager Preflight

Read-only evidence
`docs/development/ai/swarm-results/cfr168-source-keyed-cargo-canonical-port-preflight-20260731/evidence.md`
at SHA-256
`90449137903e2b15f0fd780cee7a3e9809cb54e7d1b7da18cce885282579ba34`
binds the source-range patch digest, path status, comparison-snapshot blob
collisions, and normalized test-name-set digests.

At canonical snapshot `9c1eefd`, only the File Doctor and Rust HTTP packet
paths had diverged from the historical base among the 19 source-range paths.
The source candidate has 104 test names absent from the historical base and
replaces one permissive shared-root test with separate fail-closed and admitted
source-keyed cases.

That evidence is immutable and is retained as written. It predates the
accepted successor: it was computed against the `4e5c0e7` range of 19 paths,
whereas the accepted `893cbcad..1bbcfde` range is 20 paths and 110 added test
names. The superseding figures are the exact port review's, recomputed
independently against `1bbcfde` and the candidate.

These are preflight observations, not frozen port outputs. Recompute every
path blob and test-name set from the then-current canonical base after
admission. A changed observation requires explicit disposition; it does not
authorize editing the preflight or copying a candidate blob wholesale.

## Exact Write Set

The 20 paths of the accepted `893cbcad..1bbcfde` range:

```text
docs/development/ai/README.md
docs/development/ai/acceleration-master-plan.md
docs/development/ai/failure-cookbook.md
docs/development/ai/source-keyed-cargo-gates.md
docs/development/ai/stage5-manual-control-runbook.md
docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md
docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md
docs/development/history.md
docs/development/tools/ai_contracts.py
docs/development/tools/build_source_bound_rust.py
docs/development/tools/check_ai_acceleration_docs.py
docs/development/tools/check_rust_execution_profile.py
docs/development/tools/pdeath_exec.py
docs/development/tools/probe_validation_capabilities.py
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/run_timed_gate.py
docs/development/tools/test_ai_tools.py
```

The implementer additionally created exactly one result, giving the accepted
candidate its 21 paths:

```text
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-20260802/result.md
```

The reviewer created exactly one report:

```text
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-canonical-port-rereview-claude-20260802/review.md
```

## Port Contract

1. Start from the then-current clean canonical HEAD after the candidate
   acceptance report is integrated.
2. Use the exact source range as reference; do not cherry-pick it.
3. For canonical files already changed since the historical base, apply the
   smallest semantic delta and retain all newer unrelated content.
4. Mechanically extract every test method added by the accepted source range.
   Every name must exist exactly once and execute on the port.
5. Preserve all accepted fail-closed behavior:
   - exact source/worktree/dirty-state key;
   - pinned toolchain and executable identities;
   - private namespace and propagation proof;
   - complement-based descriptor closure;
   - nonzero exact test selection and bounded parser state;
   - source/candidate immutability across execution;
   - durable bounded result/log/artifact publication;
   - bounded persistent storage and crash cleanup;
   - no false hardlink-exact reclaimed-byte or charge-back claims;
   - finite timeout and process-tree termination; and
   - refusal of direct and known indirect unkeyed Rust execution.
6. Preserve canonical KGI/KPI files and tests already present. The later
   `753c8db` semantic KGI/KPI port remains separate.
7. Record the source range, canonical base, composed tree, and every manual
   conflict disposition.

## Validation

Run only Python/docs/control-plane gates:

```bash
python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/test_ai_tools.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_ai_development.py --timeout 180
git diff --check
```

Additional mandatory evidence:

1. accepted-source test-name set equals the canonical-port test-name set for
   every source-range addition;
2. source-gate tests execute separately with nonzero selection;
3. the accepted candidate's manager counterexamples pass on the port;
4. at least one deliberate descriptor-leak regression, one propagation-claim
   regression, and one hardlink-accounting overclaim are rejected;
5. a pre-existing canonical KGI/KPI test subset still runs and passes;
6. all gates bind the exact port HEAD with monotonic duration and complete-log
   digest; and
7. both source and port worktrees remain clean.

No Cargo or Rust process may start.

## Release After Acceptance

Only a fresh `ACCEPT` on the exact canonical-port commit authorizes manager
integration. `fb891b0` has that `ACCEPT` at `8316956`. Its one manager-owned
precondition `O1` is the subject of the 2026-08-02 evidence-authority repair,
which must itself pass a fresh exact review before the chain is
fast-forward-ready. Nothing here is integrated yet.

Remaining manager actions, in order:

1. fresh-review the 2026-08-02 O1 evidence-authority repair as its own exact
   subject;
2. on `ACCEPT`, fast-forward canonical over the entire accepted chain; and
3. run the post-integration gate on the resulting canonical head.

Only after step 3 may the source-keyed validation scheduler admit Rust
candidate gates independently of KGI/KPI review state, subject to live
resource admission and Rust-heavy serialization. No Rust lane release may
precede canonical containing the entire accepted chain.

KGI/KPI then ports onto that Cargo-enabled canonical head through
`cfr168-control-plane-composition-20260731.md`.

## Handoff

The implementer returns:

- accepted candidate report identity and digest;
- source range, canonical base, port commit/tree, branch, and clean state;
- exact changed paths and per-file port disposition;
- source/port test-name-set comparison;
- counterexample and deliberate-regression evidence;
- every validation command, return code, monotonic duration, log digest, and
  unavailable row; and
- result commit and SHA-256.

The reviewer returns:

- exact port identity and result digest;
- one terminal `ACCEPT` or `REQUEST_CHANGES`;
- explicit disposition of every accepted source-gate invariant;
- severity-ordered findings with reproduced or object-level evidence; and
- report commit, report SHA-256, and clean-worktree proof.
