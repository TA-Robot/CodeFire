# CFR-168 Python HTTP Upload + Scale Review — Claude Fallback

Review date: 2026-07-30

Reviewer: Claude Opus 5 (`claude-opus-5`)

Reviewer lane: `cfr168-python-http-upload-scale-review-claude-fallback-20260730` / task
`upload-scale-review-claude-fallback`

Reviewer worktree base: `9f1fe186df3002198a184fdced4691123037d71f`

Continuation of the Fugu combined-review packet
`docs/development/ai/swarm-batches/cfr168-python-http-upload-scale-combined-review-20260730.md`,
which ended without a report. This is an independent review, not a transcription: every earlier
finding was reproduced or rejected against the corrected target from first principles, and the new
retained upper was stressed with reviewer-authored probes rather than with the target's own tests.

## Decision

**`REQUEST_CHANGES`**

The CFB-138 correction itself is real and its central claim holds. The 512 MiB retained override is
gone; the 1201-commit divergent fixture reaches the existing 409 non-fast-forward decision under an
unmodified `DecisionWorkProfile.http_v1()`; no public default was raised; and across six
adversarial staged shapes that reach the owner, no `_claim_memory` call ever exceeded an admitted
reservation. The single `aggregate_bytes` term is a sound upper on the retained documents: for every
staged record the claimed canonical measure is exactly `physical − 1`.

The target is nevertheless not acceptable. Two earlier live-path blockers are byte-identical and
still reproduce on the corrected target, and both are in CFR-168 scope per this packet:
publication-root validation still runs two sealed traversals on fresh `local_v1()` ledgers that
never poll the request deadline — an already-expired request published its branch and object and
returned 200 — and the new traversal commit source still builds a validation session with no memory
owner and therefore no reservation for any of its collections or descriptor owners. A third,
previously unreported defect surfaced under independent stress: the reservation still carries a
per-record *heap* upper (`128×`/`129×` one record's canonical bytes) instead of an owner lifetime,
so the default profile refuses any staged bundle containing a single object record above
**~127 KiB** in `semantic_validation`, before either traversal — the same root cause CFB-138 named,
relocated from record *count* to record *size*.

## Exact subject

| Role | Revision |
|---|---|
| Canonical implementation snapshot | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Traversal task registration / product baseline | `5254e3dc8e3cbab3330c2a34469eb56750edf626` |
| Interrupted upload production commit | `3153d75a20414a12a37fcce847b79e13d0616c75` |
| Upload recovery result | `cb13da08e82c2502920f735d27617e61f978527a` |
| CFB-138 task base | `88df84b7fd962cd33f4c897f555d083bc2e58677` |
| **Completed scale target (reviewed)** | **`dece688afea815d1242e7751367052d1ceac4580`** |
| Earlier reviewer report (evidence only) | `985ca9e4bd376614f1510c62100b86af7c640739` |

Target branch:
`claude-agent/cfr168-python-http-upload-staging-scale-20260730-upload-staging-scale`

Ancestry of the reviewed target (linear):

```text
5254e3dc8e3cbab3330c2a34469eb56750edf626  chore: add canonical HTTP upload traversal task
3153d75a20414a12a37fcce847b79e13d0616c75  wip(recovery): preserve interrupted HTTP upload traversal
cb13da08e82c2502920f735d27617e61f978527a  test(recovery): make the three interrupted upload traversal tests decisive
88df84b7fd962cd33f4c897f555d083bc2e58677  docs: admit CFB-138 upload scale blocker
dece688afea815d1242e7751367052d1ceac4580  fix(http-upload): reserve sealed validation's concurrent retained bytes
```

Cumulative product/test diff `5254e3dc..dece688a` — 2 files, +1,829/−11:

| Path | Change |
|---|---|
| `codefire` | +249/−11 |
| `tests/test_python_http_upload_accounted_traversal.py` | +1,580/−0 (new) |

The CFB-138 commit `dece688a` alone touches `codefire` in five hunks, all inside
`AccountedHTTPUploadSealedValidation` (target lines 21916–21936, 22377–22427, 22451–22453,
22499–22502, 22674–22675). **Everything in `codefire` below line 21912 is byte-identical to
`cb13da08`**, which is why the earlier live-path findings can be re-cited at unchanged line numbers
and why "already reviewed at an earlier baseline" is not a disposition here.

Contracts applied:
`docs/development/ai/swarm-batches/cfr168-python-http-upload-traversal-canonical-20260729.md` and
`docs/development/ai/swarm-batches/cfr168-python-http-upload-staging-scale-20260730.md`.

All probes and tests below ran against a `git archive dece688a…` export whose `codefire` and
`tests/test_python_http_upload_accounted_traversal.py` are the target Git objects.

## Findings, severity-ordered

### BLOCKER-1 — Publication root validation runs two request-bypassing sealed traversals, and an expired request deadline still publishes

**Locations:** `codefire:12297`, `codefire:12487`, `codefire:3237`, `codefire:3171`,
`codefire:29017`–`codefire:29019`, `codefire:12510`–`codefire:12516`.

`handle_http_upload` validates the publication root twice. First directly at `codefire:12297`
(`validate_remote_json_publication_identity`), then again at `codefire:12487` inside
`preflight_remote_json_publication`, whose own identity validation at `codefire:3237` repeats it.
Both reach `codefire:3171`, which calls `validate_sealed_commit`. That function
(`codefire:29017`) constructs its own ledger —

```python
ledger = DecisionWorkLedger(DecisionWorkProfile.local_v1(), "sealed validation")   # 29018
session = SealedCommitValidationSession(objects_root, ledger)                      # 29019
```

— and accepts no deadline parameter at all. `local_v1()` allows 1 GiB retained and 1 GiB aggregate
against the request's `http_v1` 32 MiB / 1 GiB, so this work is neither charged to the request nor
bounded by it.

Reviewer probe, live route, 20-commit fast-forward upload returning 200
(`/tmp/probe_nested.py`): **four** `SealedCommitValidationSession` constructions in one request —

| # | ledger | command | memory owner | retained limit |
|---|---|---|---|---|
| 1 | request ledger | `nested probe` | `AccountedHTTPUploadSealedValidation` | 33,554,432 |
| 2 | fresh | `sealed validation` | none | 1,073,741,824 |
| 3 | request ledger | `nested probe` | none | 33,554,432 |
| 4 | fresh | `sealed validation` | none | 1,073,741,824 |

Across each nested `validate_sealed_commit` the request ledger's `unique_nodes_discovered`,
`record_reads` and `graph_edges` were unchanged (27/27/146, then 68/47/166) and the request
deadline's `poll_count` was unchanged (19,625 → 19,625; 28,010 → 28,010).

Off-budget cost is not marginal. On a 400-commit successful upload
(`/tmp/probe_amplify.py`) the request ledger charged 802 graph nodes, 3,206 edges, 807 record reads
and 968,876 file bytes; the two nested `local_v1` ledgers together performed **802 graph nodes,
5,612 edges, 814 record reads, 1,294,772 file bytes and 1,628 `_read_exact_bounded` file opens** —
more record reads and 1.34× more file bytes than the request paid for, all uncharged and unpolled.

The atomicity consequence is directly observable (`/tmp/probe_deadline.py`). The injected clock was
advanced from 0.0 s to 1.0 s against a 100 ms limit immediately before `codefire:12487` — i.e.
after response preflight and before the first externally visible effect:

```text
enter validate_sealed_commit        poll_count=19624  expired_phase=None
leave validate_sealed_commit        poll_count=19624  expired_phase=None
clock advanced to 1.0s              poll_count=28009  expired_phase=None
enter validate_sealed_commit        poll_count=28009  expired_phase=None
leave validate_sealed_commit        poll_count=28009  expired_phase=None
EFFECT: write_object_records        poll_count=28009  expired_phase=None
EFFECT: execute_remote_json_publication  poll_count=28009  expired_phase=None
```

Result: status 200, `branches/main.json` moved to the uploaded tip, the uploaded object durable in
the remote store, `expired_phase` never set. From the moment the deadline elapsed to the moment the
branch moved, the request polled its deadline **zero** times. The "every budget/deadline failure
precedes publication" invariant in both contracts is false on the live route.

### HIGH-2 — The reservation still charges a per-record heap upper, so the default profile refuses any bundle holding one record above ~127 KiB before either traversal

**Locations:** `codefire:21927`, `codefire:22412`–`codefire:22416`, `codefire:22425`,
`codefire:22454`–`codefire:22464`, `codefire:22795`.

CFB-138's stated root cause was reserving "every record's heap upper as if all records were
simultaneously live". `dece688a` removed that from the *aggregate*, but both dimensions still keep a
`128×` per-record heap factor against `maximum_record_bytes`:

```python
+ cls.RETAINED_BYTES_MULTIPLIER * maximum_record_bytes            # 22425, retained,  128x
...
+ (cls.TEMPORARY_BYTES_MULTIPLIER + cls.PAIR_PROBE_BYTES_MULTIPLIER) * maximum_record_bytes
                                                                  # 22459-22463, temporary, 129x
```

Reviewer bisect on the live route with the unmodified `http_v1` profile, one commit record whose
message is the only variable, plus 100 small records and the bootstrap closure
(`/tmp/probe_maxterm.py`):

| single-record message | outcome |
|---|---|
| 65,536 B | admitted (200) |
| 98,304 B | admitted (409 ancestry) |
| 126,976 B | admitted (409 ancestry) |
| 131,072 B | **refused** `temporary_bytes` / `semantic_validation`, observed 17,095,477 vs 16,777,216 |
| 262,144 B | **refused** `retained_bytes` / `semantic_validation`, observed 34,973,356 vs 33,554,432 |
| 524,288 B | **refused** `retained_bytes` / `semantic_validation`, observed 69,052,076 vs 33,554,432 |
| 1,048,576 B | **refused** (round 4b case G) `retained_bytes`, observed 137,209,516 vs 33,554,432 |

Largest admitted ≤ 126,976 B; smallest refused ≥ 131,072 B. `33554432 / 128 = 262144` and
`16777216 / 129 ≈ 130056` reproduce both ceilings exactly. For comparison, `http_v1`
`request_body_bytes` is 16,777,216 and `MAX_OBJECT_RECORD_BYTES` is 67,108,864, so the live default
route now rejects records two orders of magnitude below its own declared record limit — in
`semantic_validation`, before signature and before ancestry, which is precisely the CFB-138
symptom.

The retained half of this is additionally unbacked. `RETAINED_BYTES_MULTIPLIER` appears in exactly
two places in `codefire` — its definition at `codefire:21927` and the reservation term at
`codefire:22425`. **No claim anywhere multiplies a record's bytes into `retained_bytes`.** Every
retained claim in the covered pool is one of: `1 ×` the record's canonical measure
(`codefire:21404`, `codefire:21343`), a `256 + len(object_id)` frame (`codefire:21284`), a
1,024-byte borrow fingerprint (`codefire:22831`), or the pair-probe plan (`codefire:23119`). The
`128×` decode/encode residue the docstring at `codefire:22412`–`22416` invokes is charged to
*`temporary_bytes`* at `codefire:22795` — and the temporary upper already budgets `129 ×
maximum_record_bytes` for exactly that. The term is therefore double-counted across dimensions, yet
it dominates the retained reservation as soon as one record is large — 40 % at case H (1,311,232 of
3,248,946) and 98 % on the 1 MiB bundle (134,228,096 of 137,209,516).

Either that term is wrong, or the `1 × aggregate_bytes` documents term is: if `128` models a parsed
record's real Python heap, then reserving `1 ×` for a memo that keeps *every* record live
under-reserves the memo by the same factor. Neither `codefire` nor the new tests state what unit
`retained_bytes` denotes, so the two terms cannot both be right. This ambiguity is the residual of
CFB-138, not a new design question.

Severity is HIGH rather than BLOCKER: every refusal above is pre-effect, carries the complete typed
diagnostic, leaves authority byte-identical, and returns current/permits to zero.

### HIGH-3 — The traversal commit source builds a validation session with no memory owner, so its collections and descriptor owners allocate against no reservation

**Locations:** `codefire:6647`–`codefire:6655`, `codefire:20923`–`codefire:20940`,
`codefire:20953`–`codefire:20956`, `codefire:21038`–`codefire:21050`.

`_AccountedUploadCommitSource._open` constructs the session positionally:

```python
session = SealedCommitValidationSession(self._objects_root, self._ledger)   # 6653
```

`memory_owner` defaults to `None`, so `_memory_apply`/`_memory_acquire` fall through to the raw
ledger and the session's own control state is never reserved *or* charged: the absolutised
`objects_root` `Path`, the `documents` dict, the `validated_commits` set, the `_fd_owners` list, the
`_subdir_fds` dict and the root `_SealedValidationFileDescriptor`
(`codefire:20923`–`codefire:20940`), plus every later owner appended by `_new_fd_owner`
(`codefire:20955`) and inserted by `_subdir_fd` (`codefire:21049`).

Reviewer probe on the live 200 upload (`/tmp/probe_nested.py`) instrumented every such site on the
request ledger and recorded outstanding reservations at each — 109 sites in total, all reporting
`reserved_count = 0`. Attributing them to their session matters, so the split is stated explicitly:

| session | sites | coverage |
|---|---|---|
| covered sealed-validation session (`memory_owner` set) | 85 — 78 `_fd_owners.append`, 7 `_subdir_fds` insertions | reservation already **consumed** into `_current` (348,578 retained), so `reserved_count = 0` there is expected; the allocations are nevertheless unclaimed → MEDIUM-4 |
| `_AccountedUploadCommitSource` ancestry session (`memory_owner=None`) | 24 — 1 construction, 22 `_fd_owners.append`, 1 `_subdir_fds` insertion | **no reservation and no charge at all**, by construction → this finding |

The uncovered group's first site is the source session's own construction:

```json
{"site": "before SealedCommitValidationSession construction", "phase": "upload ancestry",
 "request_ledger": true, "reserved_count": 0,
 "current": {"retained_bytes": 1810, "temporary_bytes": 0}, "open_permits": false}
```

The 1,810 retained bytes present belong to the already prepared `AccountedHTTPUploadAncestry`
owner, not to a source-session permit. The static fact is decisive independently of the counters:
`memory_owner` is omitted at `codefire:6653`, so this session's control state can never route
through `_claim_memory`. The upload still returned 200, so this is the production path.
`tests/test_python_http_upload_accounted_traversal.py:1093` covers only
`AccountedHTTPUploadAncestry`'s `seen`/`frontier` allocators; the source session's structures are
untested.

The traversal algorithm itself does reserve its BFS collections before allocating them. The
complete new traversal owner stack does not, so the canonical packet's charge-before-allocation
rule is not met.

### MEDIUM-4 — Inside the covered session, `_fd_owners` grows one never-removed descriptor owner per read, and `finalize` allocates a per-record id list, neither claimed

**Locations:** `codefire:21152`, `codefire:20953`–`codefire:20956`, `codefire:21728`–`codefire:21729`,
`codefire:21939`–`codefire:21948`.

`_read_exact_bounded` allocates a fresh `_SealedValidationFileDescriptor` per record read
(`codefire:21152`) and `_new_fd_owner` appends it to `self._fd_owners`. The descriptor is closed in
the `finally` at `codefire:21261` but the object is never removed from the list, so `_fd_owners`
grows monotonically with reads. The reviewer probe observed **78 appends** on the covered
sealed-validation session of a 27-record upload. The reservation's only descriptor allowance,
`_pair_plan_upper` (`codefire:21939`–`codefire:21948`), budgets
`min(record_count, len(OBJECT_PREFIXES)) + 2` `_CallbackFreeFD` slots — a small constant, not one
per read. Similarly `finalize` builds `ordered_object_ids = list(self.documents)` and sorts it
(`codefire:21728`–`codefire:21729`) with no claim; that is one pointer per staged record.

Neither routes through `_claim_memory`, so neither can trip the coverage check — they are silently
outside the accounted model rather than over it. On the shapes measured they fit inside
`RETAINED_BASE_BYTES` slack, which is why this is MEDIUM and not a reproduced budget breach. It is
the same class as HIGH-3 and should be repaired with it.

## Disposition of every earlier finding

Earlier report: `985ca9e4bd376614f1510c62100b86af7c640739`,
`docs/development/ai/swarm-results/cfr168-python-http-upload-independent-review-20260730/review.md`.

| Earlier finding | Disposition on `dece688a` | Basis |
|---|---|---|
| **BLOCKER-1** — nested `validate_sealed_commit` traversals with fresh local ledgers; expired deadline can still publish | **CONFIRMED — reproduced independently, still blocking** | Code byte-identical (only `codefire` ≥ 21912 changed). Reproduced from scratch: 4 sessions / 2 fresh `sealed validation` `local_v1` ledgers on one 200 upload; request counters and deadline `poll_count` frozen across both; 1.0 s elapsed against a 100 ms limit immediately before `codefire:12487` still returned 200, moved the branch and made the object durable. New quantification: 814 off-budget record reads and 1,294,772 off-budget file bytes on a 400-commit upload. Earlier line citations `28957`/`28958` are now `29018`/`29019` (+61 from this commit's insertions); `12297`, `12487`, `3171`, `3237`, `12515`–`12516` are unchanged. |
| **HIGH-2** — deep recovery test raises retained to 512 MiB to bypass a default-profile predecessor failure | **RESOLVED by `dece688a`** | The override is gone; `tests/test_python_http_upload_accounted_traversal.py:487` asserts `retained_limit == 32 * 1024 * 1024` and `peak < limit`. Reproduced independently on the same fixture shape (probe 4b case E′): 1,208 staged records → **409** with the exact non-fast-forward body, reservation 8,307,300 vs limit 33,554,432, measured concurrent peak 1,314,161, `authority changed: []`, current retained/temporary zero, no open permits. No public default was raised: `git diff 5254e3dc dece688a -- codefire` touches neither `local_v1` nor `http_v1` limits; the target's `http_v1` retained is still `32 * 1024 * 1024`. |
| **HIGH-3** — commit source allocates session collections and FD owners before a source-specific reservation | **CONFIRMED — reproduced independently, still open** | Code byte-identical. Reproduced with a labelled live probe: of 109 instrumented allocation sites, 24 belong to the `_AccountedUploadCommitSource` ancestry session, which is constructed with `memory_owner` omitted at `codefire:6653` and therefore has no reservation and no charge at all. See HIGH-3, which also separates the 85 sites belonging to the covered session (MEDIUM-4). |
| Earlier item-1 verdict FAIL | Upheld | Same evidence as BLOCKER-1. |
| Earlier item-2 verdict FAIL | Upheld | Same evidence as HIGH-3. |
| Earlier item-3 verdict FAIL | Upheld | Same evidence as BLOCKER-1 (atomicity half). |
| Earlier item-4 verdict FAIL | **Reversed to PASS** | The FAIL rested entirely on the 512 MiB override; the override is gone and the default fixture is non-vacuous. See item 4 below. |
| Earlier item-5 verdict PASS | Upheld | 13/13 focused, 59/59 neighbouring owner cases and 22/22 selected legacy CLI cases pass on the target; response bytes, signature semantics, diagnostic precedence, replay/nonce and zero-debt cleanup all preserved. |
| Earlier item-6 verdict FAIL ("recovery tests relax product requirements") | **Reversed to resolved** | Same basis as HIGH-2. Note the earlier report numbered its verdicts against the *traversal* packet; the six items decided below are the *combined* packet's. |
| Earlier harness note: `CodeFireCLITests` vs `CodeFireCliTests` | Not reproduced | Reviewer used `CodeFireCliTests` and loaded all 22 selected cases on the first attempt. |

## Verdicts on all six required-review items

| # | Item | Verdict |
|---|---|---|
| 1 | One exact request ledger/deadline owns body → cleanup; no nested legacy `validate_sealed_commit` traversals or independent local ledgers on the live route | **FAIL** |
| 2 | `_AccountedUploadCommitSource` and every validation session/list/dict/set/FD owner reserves source-specific capacity before construction/insertion | **FAIL** |
| 3 | `concurrent_retained_upper` is a real upper bound derived from genuinely concurrent owner lifetimes; no valid staged shape makes `_claim_memory` exceed an admitted reservation | **FAIL** (safety half passes) |
| 4 | Default `http_v1` 1x/3x/10x curves, 1201-commit reversal, exact/±1, claimed peak, zero debt, unchanged authority, no raised public limit, deterministic slopes and precedence | **PASS** |
| 5 | Publication identity, preflight, final deadline/subject fence and durable result selection all occur before the first externally visible effect | **FAIL** |
| 6 | Signature policy, fast-forward/non-fast-forward, malformed/corrupt/missing history, replay/nonce, response bytes, atomicity, neighbouring Doctor/branch; 13 focused tests, legacy cases, compilation, docs checks, `git diff --check` | **FAIL** (atomicity only; every other clause passes) |

**Item 1 — FAIL.** `is_ancestor_from_objects_accounted` and
`enforce_remote_commit_signature_policy_accounted` do receive the exact request ledger and deadline
(`codefire:12379`–`12385`, `codefire:12279`–`12285`), and
`tests/test_python_http_upload_accounted_traversal.py:384` proves object identity for both. But two
publication-root validations still reach `validate_sealed_commit`, which mints a `local_v1()` ledger
and polls no deadline. Measured: 2 fresh `sealed validation` ledgers per successful upload; request
counters and deadline polls frozen across both. See BLOCKER-1.

**Item 2 — FAIL.** The ancestry owner is goal-directed and iterative and reserves its own
`seen`/`frontier` before allocation; the shared-DAG fixture is deterministic and charged once
(reproduced: probe 4 case D, 187 records, 200, zero debt). But
`_AccountedUploadCommitSource._open` omits `memory_owner`, so the source session's path, dict, set,
list and every descriptor owner allocate against nothing (24 measured sites, none reservable), and
even the *covered* session grows `_fd_owners` per read (78 appends measured) and builds `finalize`'s
ordered-id list unclaimed. See HIGH-3 and MEDIUM-4.

**Item 3 — FAIL, with the safety half explicitly passing.** The bound is never exceeded. Six
reviewer-authored staged shapes reached the owner — 400-commit linear chain, 300-parent fan-in
merge, 60 stacked diamonds, the 1,201-commit divergent fixture, a `require_history` signed
400-commit chain, and a 250-parent fan-in under `require_history`. In all six `_claim_memory` never
refused, `_claimed_memory_peak` never reached the reservation, and
`concurrent_retained_upper(count, aggregate, maximum)` recomputed from the class constants equalled
`_coverage_retained_upper`:

| shape | records | bounds (count, aggregate, maximum) | reservation | measured peak | used |
|---|---|---|---|---|---|
| A 400-commit chain | 407 | 407 / 324,093 / 805 | 2,912,530 | 439,976 | 15.1 % |
| B 300-parent fan-in | 308 | 308 / 255,463 / 11,870 | 3,683,126 | 256,025 | 7.0 % |
| D 60 diamonds | 187 | 187 / 148,933 / 840 | 1,436,770 | 149,616 | 10.4 % |
| E′ CFB-138 1201 | 1,208 | 1,207 / 967,078 / 805 | 8,307,300 | 1,314,161 | 15.8 % |
| F signed chain, `require_history` | 407 | 407 / 414,493 / 1,031 | 3,122,258 | 530,376 | 17.0 % |
| H 250-parent fan-in, `require_history` | 258 | 258 / 270,837 / 10,244 | 3,248,946 | 271,449 | 8.4 % |

The crux term is sound: the retained claim per document is the canonical measure while the bound is
the *physical* staged size, and probe 6 (`/tmp/probe_aggregate.py`) confirms over ASCII, CJK, emoji,
escape, control-character and Latin-1 record shapes that canonical is exactly `physical − 1` for
every staged record (81 documents, Σcanonical 34,436 vs Σphysical 34,517, per-record delta uniformly
`−1`), because staged records are written compact-canonical plus a newline. Live traversal frames
(`256 + len(object_id)` ≈ 290 B; from `validate_commit`, `codefire:21609`–`codefire:21691`, one
charged frame per DFS-active node plus at most one pending, so at most `record_count + 1`
concurrent) and the per-record borrow
fingerprint (1,024 B) both sit inside the `RETAINED_RECORD_BYTES + BORROW_FINGERPRINT_BYTES = 5,120`
per-record allowance.

The item nevertheless fails its stated requirement that every term be derived from a genuinely
concurrent owner lifetime: `RETAINED_BYTES_MULTIPLIER * maximum_record_bytes` corresponds to no
retained claim in the covered pool and duplicates a term the temporary upper already carries, while
reaching 98 % of the whole retained reservation on a 1 MiB-record bundle. See HIGH-2.

**Item 4 — PASS.** Reproduced independently on the default profile, no overrides:

| factor | depth | staged records | result | reservation | measured peak | ledger retained peak | limit |
|---|---|---|---|---|---|---|---|
| 10x | 1,200 | 1,208 | 409 non-fast-forward | 8,307,300 | 1,314,161 | 8,307,300 | 33,554,432 |

`authority changed: []`, tip absent from the remote store, `current` retained/temporary zero, no
open permits. The target's own curve test (`tests/test_python_http_upload_accounted_traversal.py:552`)
pins 1x/3x/10x at records 127/367/1,207, nodes/reads 121/361/1,201, depth 120/360/1,200, cumulative
retained 1,600,708/4,292,260/13,743,133 and concurrent peak 1,025,102/2,646,062/8,322,480, and it
passes; my 10x reservation of 8,307,300 differs from its 8,322,480 only through fixture message
bytes (`aggregate` 967,078 here), and the recomputed-from-constants identity held in every case, so
the curves are reproducible rather than hard-coded. The reversal is genuine: the restored
all-records-live formula gives 130,978,456 for the 10x bounds, 3.9× over the ceiling, and
`tests/…:842` proves the pre-fix reservation fails the default profile on the same fixture. Slopes
are deterministic — the aggregate slope is exactly 2 (verified at `tests/…:702`) against 128 before.
Counter and diagnostic precedence were stable across every repeated probe run. No public limit was
raised (`git diff 5254e3dc dece688a -- codefire` touches no profile definition; `http_v1` retained
is still `32 * 1024 * 1024`, temporary `16 * 1024 * 1024`).

Scope of independent reproduction for this item, stated plainly: I reproduced the 10x point and five
further shapes end-to-end with my own fixtures, and the aggregate-slope, canonical-versus-physical
and never-exceeded properties analytically and by measurement. The 1x and 3x rows, and the retained
exact / minus-one / plus-one boundaries (`tests/…:722`, `tests/…:982`), rest on the target's own
tests, which I executed and which pass — I did not rebuild those two fixtures. The minus-one case is
the one that most matters and its assertions are non-vacuous: it lands the typed
`retained_bytes`/`semantic_validation` diagnostic on the corrected owner before the session
allocator and before either traversal, which my `_claim_memory` instrumentation independently
confirms is the enforcement point.

**Item 5 — FAIL.** `preflight_remote_json_publication` (`codefire:12487`),
`publication_preflight.seal_for_effect` (`codefire:12500`) and
`response_slot.seal_for_effect()`/`mark_effect_started()` (`codefire:12510`–`12512`) do all precede
`write_object_records` (`codefire:12515`) and `execute_remote_json_publication`
(`codefire:12516`), and the selected legacy pre-effect cases pass. The final *deadline* fence,
however, does not exist: no deadline poll occurs anywhere between `codefire:12487` and the effect.
Reproduced — deadline elapsed 10× over, zero polls, status 200, branch moved, object durable. See
BLOCKER-1.

**Item 6 — FAIL, on the atomicity clause only.** Everything else in the clause passes on the
target. Signature policy required/optional decisions and first-error precedence, fast-forward and
non-fast-forward, malformed/corrupt/missing-history rejection, replay/nonce, exact response bytes
and typed 413, and neighbouring Doctor/branch behaviour are all green (22/22 selected legacy CLI
cases, 13/13 focused, 59/59 neighbouring owner cases, 434/434 on the full
`discover -p 'test_python_*.py'` sweep, `py_compile`, both docs checks and `git diff --check` clean —
see the command table). Atomicity fails: an expired request published. Note that no automated test
in the target detects that failure, which is why repair step 5(a) is not optional.

## Commands and results

All executions used the exact `dece688afea815d1242e7751367052d1ceac4580` export. Python 3.10.12.

| Command | Result |
|---|---|
| `git diff --numstat 5254e3dc…dece688a -- codefire tests/test_python_http_upload_accounted_traversal.py` | PASS — 2 files, +1,829/−11 |
| `git diff --numstat cb13da08…dece688a -- codefire` | PASS — +67/−7, five hunks, all ≥ line 21912 |
| `git diff --check cb13da08…dece688a` / `git diff --check 5254e3dc…dece688a` | PASS (both) |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py` | PASS |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | PASS — 13/13, 141.162 s |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | PASS — 59/59, 66.860 s |
| 22 selected `tests.test_codefire_cli.CodeFireCliTests` cases (non-fast-forward, inconsistent/wrong-root/malformed/invalid-parent sealed commits, server verification policy, required commit signature, receipt replay across transports, response preflight/durable retry, nonce race, all-or-typed response body, compact unicode response boundaries, closure missing/corrupt rejection, closure fast-forward + rejected-stage cleanup, publication identity and plan pre-effect, http upload/list/clone, branch protection, two Doctor sealed/integrity cases, remote list+doctor one-ledger, remote doctor missing project) | PASS — 22/22, 424.551 s |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'` | PASS — 434/434, 367.671 s |
| `python3 docs/development/tools/check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` | PASS — 526 Markdown files inventoried |
| `git diff --check` on the target export | PASS |
| `python3 /tmp/probe_nested.py` (BLOCKER-1 + HIGH-3) | Reproduced 2 fresh `sealed validation` `local_v1` ledgers with frozen request counters/polls, and 109 instrumented allocation sites at `reserved_count = 0` (24 on the uncovered ancestry source session, 85 on the covered sealed-validation session) |
| `python3 /tmp/probe_deadline.py` (BLOCKER-1 atomicity) | Reproduced 1.0 s elapsed vs 100 ms limit → 0 polls → status 200, branch moved, object durable |
| `python3 /tmp/probe_amplify.py` (BLOCKER-1 cost) | 814 off-budget record reads / 1,294,772 off-budget file bytes / 1,628 file opens vs 807 charged reads on a 400-commit upload |
| `python3 /tmp/probe_upper.py`, `/tmp/probe_upper2.py` (item 3) | 8 staged shapes attempted, 6 reached the owner; no `_claim_memory` refusal; peak 7.0 %–17.0 % of reservation; recomputed formula equalled the reservation in every case. The two that did not reach it are the large-record cases: a 4 MiB record is refused earlier at `staging`/`temporary_bytes` (20,976,081 vs 16,777,216) and a 1 MiB record at the sealed reservation (HIGH-2) |
| `python3 /tmp/probe_maxterm.py` (HIGH-2) | Default-profile single-record admission bisect: ≤126,976 B admitted, ≥131,072 B refused `temporary_bytes`, ≥262,144 B refused `retained_bytes` |
| `python3 /tmp/probe_aggregate.py` (item 3 crux) | canonical = physical − 1 for all 81 retained documents across ASCII/CJK/emoji/escape/control/Latin-1 shapes |
| Reviewer worktree `git status` / `git diff --check` after writing this report | PASS — only this report added |

The docs checks require a Git work tree; the target export was committed into a throwaway
repository (`git init` in the export directory) so both tools could resolve tracked files. No
command in this review wrote to the reviewer worktree except this report.

## Minimal repair order

1. **BLOCKER-1, ledger/deadline ownership.** Make publication-root validation consume the
   already-prepared request-owned sealed snapshot/owner instead of calling `validate_sealed_commit`,
   or thread the exact request `DecisionWorkLedger` and `HTTPRequestDeadline` through
   `validate_remote_json_publication_identity` → `validate_sealed_commit` and charge/poll them.
   Collapse the duplicate identity validation at `codefire:12297` and `codefire:3237` to one.
2. **BLOCKER-1, atomicity fence.** Poll the exact request deadline once immediately before the first
   externally visible effect (between `codefire:12512` and `codefire:12515`), so an already-elapsed
   deadline can never publish even if step 1 regresses.
3. **HIGH-3 / MEDIUM-4, charge-before-allocate.** Pass a memory owner (or acquire a source-specific
   reservation) at `codefire:6653` covering the session `Path`, `documents`, `validated_commits`,
   `_fd_owners`, `_subdir_fds` and each descriptor owner before construction and before each
   insertion; bound `_fd_owners` growth per read (reuse or evict the closed owner) and claim
   `finalize`'s ordered-id list.
4. **HIGH-2, reservation units.** Decide and document what `retained_bytes` denotes, then either
   delete `RETAINED_BYTES_MULTIPLIER * maximum_record_bytes` from `concurrent_retained_upper` (if
   the unit is canonical bytes, which every claim implies) or apply the heap factor consistently to
   the documents term. Re-derive `_canonical_borrow_digest`'s `128 ×` temporary scratch from the
   encoder's real 1–2× canonical output and re-measure the single-record admission ceiling.
5. **Tests, live route only.** (a) an already-expired deadline immediately before
   `preflight_remote_json_publication` must fail pre-effect with authority, objects, generation,
   nonce and receipt unchanged; (b) zero `SealedCommitValidationSession` instances on any ledger
   other than the request's during one upload; (c) a source-session allocation probe asserting an
   outstanding reservation at every list/dict/set/FD insertion; (d) a single-record size curve
   pinning the largest record the default profile admits, so any future change to the multipliers
   moves a recorded number.
6. Re-review independently before integration. Do not treat CFB-138 as closed while item 4 of the
   packet passes but items 1, 2, 3, 5 and 6 do not.

## Residual risk

- **Record-count ceiling.** With the corrected formula the reservation is still linear at roughly
  `5,120 + 2 × mean_record_bytes` per record — 6,883 B/record on the measured 1201-commit fixture —
  so the default 32 MiB retained limit is reached at roughly 4,900 staged records of that size. The
  contracts require 1,201; nothing pins the ceiling, and a future fixture between 1,201 and ~4,900
  will pass while one just above it will fail in `semantic_validation` with no test naming the
  boundary.
- **Reservation slack does not shrink with scale but is large.** Measured utilisation is 7 %–17 %
  across every shape, i.e. the bound over-reserves by 6×–14×. That is safe but it means the profile
  refuses uploads roughly an order of magnitude before real pressure, and the CFB-138 class of
  report will recur at each new scale target.
- **`_claimed_memory_peak` is observation-only.** It is written at `codefire:22674`–`22675` and read
  by tests, but nothing in production compares it to the reservation, so an over-reservation is
  invisible at runtime; only an *under*-reservation raises.
- **Unclaimed structures are invisible to coverage.** MEDIUM-4's allocations do not route through
  `_claim_memory`, so they can never trip the prepared-upper check. Today they fit in
  `RETAINED_BASE_BYTES` slack; a larger `OBJECT_PREFIXES` set, a deeper closure or more reads per
  record would change that silently.
- **Nested-traversal blast radius beyond this route.** `validate_sealed_commit` has 27 call sites in
  `codefire`, several on remote branch/MR/escrow paths (`codefire:11654`, `11747`, `32417`–`32418`,
  `32540`, `32803`–`32804`). This review reproduced the ledger/deadline escape only on the HTTP
  upload route; the same fresh-`local_v1()` pattern very likely bypasses other request budgets, and
  that is outside this packet's subject.
- **Not assessed here.** Rust local/file transport, GC/metrics, and every other frozen CFR-168 or
  CFB-136 surface outside `codefire` and
  `tests/test_python_http_upload_accounted_traversal.py`. Product, tests, issue state and management
  ledgers were read-only for this review; no conclusion is offered about them.
