# AIA-048 Source-Keyed Live-Sampler Churn Repair Exact Review

Reviewed implementation: `98809413cef7df25d56e985944c3701f48635fb2`

Reviewed result subject: `bc37b640632f9a4e2df187dd65a7d53863432d3c`

Report mode: fresh independent report-only review. No Cargo, Rust, product,
install, dogfood, network, or product-candidate mutation was performed by this
reviewer.

This review decides only the AIA-048 validation-control repair. It does not
accept the File Ancestry R1g product candidate, AIA-046 as a whole, File
Ancestry as a whole, or CFR-168, and it does not authorize integration. The
manager remains responsible for those separate decisions.

## 1. Immutable subject

| Field | Declared | Recomputed | Result |
|---|---|---|---|
| packet base | `54028a136943a9d5f9608e19d3855c0edae46adf` / tree `e4057e1b6f16b12fed45ad0b50bc4c6014ca7ebd` | same commit and tree | MATCH |
| packet commit `P` | `5b283ccbc8d0bd7dcf166939a8d5f24dcf997d29` | parent `54028a1`, tree `a9f554460361929991573c13fcf2077226198fcc` | MATCH |
| implementation `I` | `98809413cef7df25d56e985944c3701f48635fb2` | parent `P`, tree `cd0014c85625404767554512de0af5bea8851ea0` | MATCH |
| result `W` | `bc37b640632f9a4e2df187dd65a7d53863432d3c` | parent `I`, tree `047858e68756fc0e2f1385e4c8843cc636e51d19` | MATCH |
| merge base `P` / `W` | `P` | `P` | MATCH |
| additive commits / merges in `P..W` | 2 / 0 | 2 / 0 | MATCH |
| implementation binary-patch SHA-256, `P..I` | not predeclared | `0287010d6755f97e4902942725213a85d5b7068d48fc1414ff19f00ecfe2ba0c` | BOUND |
| complete binary-patch SHA-256, `P..W` | not predeclared | `1abb9cd16613c69744aa969c62b2f8492eee2b95c60e052a4e2bb683de65e3e7` | BOUND |
| clean tracked/untracked at review start | true | empty porcelain, empty digest `e3b0c442…b855` | MATCH |

`I` changes exactly the three authorized implementation paths:

- `docs/development/tools/run_source_keyed_cargo_gate.py`
- `docs/development/tools/test_ai_tools.py`
- `docs/development/ai/source-keyed-cargo-gates.md`

`W` adds exactly the authorized result path. The three implementation blob
hashes independently reproduce the result:

| Path | SHA-256 |
|---|---|
| `run_source_keyed_cargo_gate.py` | `821a90604521904e053e406dc59c6155f362711ff755f316192407c889314dcc` |
| `test_ai_tools.py` | `f588b50fd81f62c703c4115fdb2f974bb15d87e76b5715c9b249868012172bdd` |
| `source-keyed-cargo-gates.md` | `7f2be7289772a61f0da7524eecbbe44abe2eda6782bdf78b905f7e73173dac8e` |

The result blob SHA-256 is
`75e218e53fd9eabad4b8d6264d6b3b163f043a81d0c13d5dce2534e29c7321f3`.

## 2. Scope and production reachability

The live route is `run_source_keyed_gate` -> `runtime_resource_guard` ->
`live_namespace_sample` -> `namespace_usage_at`. The repair selects the named
`LIVE_BEST_EFFORT_NAMESPACE_SCAN` policy only at that one-second supervisor
route. `namespace_usage`, admission, registry inspection, snapshot accounting,
post-gate measurement, and retention remain strict. A traversal dictionary
without a named policy also defaults fail-closed.

The live policy tolerates only descriptor-relative `FileNotFoundError` after a
name was enumerated or a bound internal directory disappeared before reopen.
The name has already consumed the shared entry budget. The walk binds every
absolute namespace component from `/`, every queued internal directory
component, revalidates the current directory after its scan, and revalidates
the complete namespace-root chain after pending children drain. Wrong
device/inode, same-name replacement, ancestor or root replacement, symlinked
ancestry, malformed entries, count excess, and deadline excess do not enter the
tolerant branch.

The final strict scan remains the retention authority. The independent
filesystem-free sample still runs after observed churn; the per-namespace and
aggregate allocation checks are unchanged. The documentation accurately calls
the live scan a non-atomic best-effort reaction boundary rather than a quota or
snapshot.

Exact writable review path:
`docs/development/ai/swarm-results/aia048-source-keyed-live-sampler-churn-repair-20260802/review.md`.
No other path is writable in this report commit.

## 3. Frozen obligation coverage

| ID | Frozen obligation | Disposition | Production proof / deterministic oracle |
|---|---|---|---|
| D1 | Separate live and strict policies | CLOSED | Named policies and separate `live_namespace_sample` / `namespace_usage` entry points; strict-default test |
| D2 | Tolerate only proven live disappearance | CLOSED | Exact file-stat and directory-reopen disappearance fixtures complete and preserve survivor accounting |
| D3 | Replacement, symlink, identity, and containment remain fail-closed or safely measured | CLOSED | Direct, nested-parent, file-parent, post-scan, root, registry-root, intermediate-symlink, and late-root controls are rejected |
| D4 | No unbounded retry or delay; deadline remains authoritative | CLOSED | No new sleep/loop; checks surround directory work, entries, disappearance handling, post-scan, and final-root validation |
| D5 | Cadence and all resource fences remain | CLOSED | Exact runtime guard route preserves namespace, aggregate, and free-floor checks; focused controls make each blocker reachable |
| D6 | State the real best-effort guarantee | CLOSED | Contract explicitly says non-atomic reaction boundary and assigns retention to the strict post-gate scan |
| D7 | Preserve evidence/source/failure predicates except the false churn block | CLOSED | Existing publication topology suite passes; exact R1g gate binds the implementation supervisor hash and publishes complete evidence |
| T1 | File disappears after enumeration | CLOSED | `test_live_scan_tolerates_a_file_vanishing_after_enumeration` |
| T2 | Directory disappears before reopen | CLOSED | `test_live_scan_tolerates_a_directory_vanishing_before_reopen` |
| T3 | Replacement/identity is not skipped | CLOSED | `test_live_scan_refuses_replacement_and_symlink_as_churn` |
| T4 | Strict controls remain fail-closed | CLOSED | `test_strict_measurement_still_fails_closed_on_those_controls` |
| T5 | Stable namespace over-limit blocks | CLOSED | `test_live_guard_still_blocks_stable_over_limit_allocation` |
| T6 | Aggregate and free-floor blockers remain reachable | CLOSED | Separate aggregate test plus churn-and-free-floor test |
| T7 | Deadline and entry limits remain | CLOSED | Guard controls plus empty-directory deadline control |
| T8 | Bounded repeated churn makes progress and predecessor fails behaviorally | CLOSED | Five repaired rounds plus two-attempt predecessor reversal reaching `live-target-usage-unavailable` |
| T9 | PASS/BLOCK cleanup and candidate invariants remain | CLOSED | Real gate topology control checks durable result, log, candidate, snapshot, quarantine, and runtime scratch on both decisions |

The 12 added test methods are not twelve shallow assertions: several contain
multiple independently scheduled filesystem mutations. They close every D/T
row above without relying on a background-thread race or on a symbol-presence
failure.

## 4. Owner, effect, and fence summary

| Phase | Physical owner | Permitted effect | Final fence | Terminal outcome |
|---|---|---|---|---|
| live supervisor | gate-owned Cargo namespace | omit only a descriptor-proven vanished name/subtree from one sample | deadline, entry count, component identity, namespace/aggregate/free-space bounds | continue or typed `BLOCK` |
| admission / registry / recovery | manager under registry/global locks | no churn tolerance | strict descriptor walk and identity checks | admit or fail closed |
| post-gate / retention | manager after Cargo quiesces | exact strict measurement and retention write | strict scan, aggregate/free floor, committed run state | persist, clean, or `BLOCK` |
| result publication | evidence publisher | publish canonical terminal JSON/log | digest, mode, link count, candidate recapture, cleanup evidence | durable `PASS` or `BLOCK` |

## 5. Mutation and non-vacuity matrix

| Mutation | Focused oracle | Expected | Observed in 266-test run |
|---|---|---|---|
| enumerated file unlinked before no-follow stat | live versus strict pair | live progresses; strict raises | KILLED |
| queued directory removed before reopen | live versus strict pair | live progresses; strict raises | KILLED |
| directory or ancestor replaced with another inode | replacement controls | identity block | KILLED |
| bound ancestor replaced with a symlink | ancestry controls | non-ENOENT failure | KILLED |
| namespace root replaced during root or child scan | early/late root controls | identity block | KILLED |
| allocation/aggregate/free floor forced past bound | three live-guard controls | respective typed blocker | KILLED |
| entry bound or live deadline exhausted | bound controls, including empty directory | typed blocker | KILLED |
| predecessor uses two strict retries under two exact stale-name windows | behavioral predecessor simulation | `live-target-usage-unavailable` | KILLED |
| terminal PASS or replacement-driven BLOCK | real gate topology fixture | complete publication and cleanup invariants | KILLED |

## 6. Validation

| Class | Exact command / subject | Result | Duration / digest |
|---|---|---|---|
| independent full Python suite | `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py` at `W` | 266/266, `OK` | 139.286 s; foreground output not retained as a file |
| syntax | `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/run_source_keyed_cargo_gate.py docs/development/tools/test_ai_tools.py` | rc 0 | direct reviewer run |
| AI docs | `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_acceleration_docs.py` | rc 0 | `AI acceleration docs check passed.` |
| project docs | `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py` | rc 0 | 690 Markdown files inventoried |
| implementation whitespace | `git diff --check P..I` | rc 0, empty output | exact implementation range |
| live source-keyed Rust evidence | logical argv `cargo test -p codefire-cli --bin codefire-rs remote::file_remote_ancestry_tests --locked` | `PASS`; standard `codefire-rs` harness 40/40, 0 failed, 768 filtered | 596.071769305 s monotonic; result `cc8814ba…59cc4`; log `d8c525fd…2fbc1` |

The manager-only R1g gate used candidate
`fb1396df69949ea995debba19557ef9661cdfa78`, tree
`7004395611abe68d7f7da7a669aa7b845087d28a`. Its before/after candidate
records are byte-equal, clean, and untracked-free. The result directly binds
the AIA-048 `policy-supervisor` SHA-256
`821a90604521904e053e406dc59c6155f362711ff755f316192407c889314dcc`.

The result is 36,901 bytes, SHA-256
`cc8814ba73b3d73c4e8d37613691ad30db486dc3cd1ad4d79adf840e89e59cc4`,
mode `0400`, link count 1. Its sibling log is 8,061 bytes, SHA-256
`d8c525fd757a7dde2005f60dffdef9168558e2d9820e26aa0b827b8b0c82fbc1`,
mode `0400`, link count 1. Both actual files reproduce the recorded digests.

The runtime guard reports `PASS`, 569 invocations, maximum 0.147380333 s,
total 27.684850479 s, no unfinished invocation. Run state committed as
`081982a2ad6772cb60325ea599cc958288c11bd3f438bcf1ca879d2e963b1e50`;
retention is `PERSISTED`; runtime scratch is removed; the exact source snapshot
is removed marker-last; and both registry and snapshot quarantine scans are
empty.

The evidence records a 15.548115083 s wall/monotonic discontinuity
(`wall_clock_discontinuity: true`). This is retained rather than hidden. The
authoritative duration source is `time.monotonic_ns`, the gate did not time
out, and neither the packet nor the source-gate decision contract requires a
continuous wall clock for this local test result.

The earlier R1f evidence is intentionally not treated as this PASS. R1f used
the same exact supervisor bytes and its runtime guard passed, but its product
suite was 38/40 and the overall result was `BLOCK`. R1g is a different clean
candidate and is the evidence that closes the packet's overall-PASS condition.

## 7. New findings and integration overlap

No material or acceptance-critical candidate-created finding was found.

R1g product semantics are outside this report: the R1g run is accepted only as
a compliant live workload and exact validation artifact for AIA-048. Any
review, integration, or lifecycle decision for R1g remains separately owned.

The Markdown packet and result use conventional two-space hard breaks. Exact
`P..I` implementation whitespace is clean; those report-format spaces do not
alter any implementation byte or violate the frozen three-path implementation
validation boundary.

## 8. Unavailable evidence

- No immutable file log was created for this reviewer's independent Python
  266-test rerun; the terminal result and duration were observed directly.
  This does not substitute for the separately durable manager source-keyed
  result/log pair described above.
- Install and dogfood are not applicable to this report-only validation-control
  review and make no product-readiness claim.

## 9. Decision

- local frozen obligations closed: 16/16
- deterministic mutation groups killed: 9/9
- required manager live gate: 1/1
- exact live harnesses passing: 1/1, 40/40 tests
- unowned material findings: 0
- integration authorization: no; manager-only

OVERALL_VERDICT: ACCEPT
