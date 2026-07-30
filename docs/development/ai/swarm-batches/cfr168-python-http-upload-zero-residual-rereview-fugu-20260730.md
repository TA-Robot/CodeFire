# CFR-168 / CFB-138 Python HTTP Upload Zero-Residual Independent Re-review

Independently re-review the exact Python HTTP upload repair candidate
`76baff84cb1192e11b26dcc669fb800f7b13fa30`. Use Sakana Fugu 1.1 Ultra
(`fugu-ultra-v1.1`). This is a report-only reviewer lane: do not modify product code, tests,
canonical source docs, issue state, management ledgers, or the repair history.

The prior verdict was `REQUEST_CHANGES`. An `ACCEPT` decision is valid only if every prior
Blocker/High/Medium finding, every required boundary/reversal, every upload-local residual, and the
neighboring HTTP call-site audit is independently closed on the exact candidate. Do not infer
closure from commit subjects, source-doc claims, the implementer's tests, or a clean worktree.

## Exact subject and immutable provenance

| Role | Revision / digest |
|---|---|
| Canonical implementation snapshot | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Reviewed scale target | `dece688afea815d1242e7751367052d1ceac4580` |
| Prior report in this ancestry | `173cdf83583f8a677fe63a58c9787e8f8e884e9d` |
| Byte-identical original report commit | `0b1fd6f50fce2126a07238c422b1ff01b575c047` |
| Prior report Git blob | `32e0ead1727d6a0f745616f08149579990c60845` |
| Prior report SHA-256 | `534f098348e72fdc377b56ebf561a931e53284055a1f8a0e37d6d5393d956486` |
| Repair ticket / implementation base | `5f798990c313ea2e1b117bc74824de3f3ce028d2` |
| Repair ticket SHA-256 | `13a785e553545f111e4a0ab51583ff1ff23f6cd14ff60aa09b1f4ce06195db0d` |
| **Exact repair candidate** | **`76baff84cb1192e11b26dcc669fb800f7b13fa30`** |
| Candidate parent | `f6798cbfffda6ae181e934210a465ef7a7373539` |
| Candidate Git tree | `fcc65a4729f5029556e0266971d94b6321421f09` |
| Implementation-base Git tree | `8f6cadbf9457081de27e485dbf00dd78f27293ff` |
| Candidate patch SHA-256 | `0e0a0c35ae8c7e9852775199ade4172b737e6797bcbcc2425b2baba29aadd028` |

The patch digest is the output of:

```bash
git diff --binary --full-index --no-ext-diff --no-renames \
  5f798990c313ea2e1b117bc74824de3f3ce028d2..\
76baff84cb1192e11b26dcc669fb800f7b13fa30 | sha256sum
```

The merge base of the implementation base and candidate must be exactly
`5f798990c313ea2e1b117bc74824de3f3ce028d2`, and the range must contain exactly 11 commits:

```text
600750cc070f076052b46fa39a32ed3eb19a8068  stream canonical encoding with bounded scratch
c33592a40d2099bdb3eea5b82a728ce643110b2d  reserve sealed-session control state
0b288985aa37b23567e8e28afb6505cc54b0ce2a  one request-owned traversal and final effect fence
6abc9916fd3b1f091d0c13e02ea0a4cc21f82206  consume the sealed proof inside its reservation
c5d7e5e88da1dd734455913443b37c3eb68052ca  production prepared-upper invariant
8830358bccbc5837b4874735c0a9805cf26b5c47  reversal and non-vacuity evidence
7d5d219da9ee28f37b151b3265cf55d4475d523f  portable exact-target session probe
49e044a53baabf731b70fd65ce83d0ab8b78445c  canonical source-doc update
668c44fa5871903491043a659ff253943678e908  release detached-document borrow fingerprint
f6798cbfffda6ae181e934210a465ef7a7373539  remove dead publication-root proof plumbing
76baff84cb1192e11b26dcc669fb800f7b13fa30  pin record-count and record-size ceilings
```

The exact cumulative diff is six modified files, `+2,227/-120`:

| Path | Added | Deleted | Candidate Git blob |
|---|---:|---:|---|
| `codefire` | 1,151 | 97 | `cc2c52aff4a11ebd80b2f22e37482b1700c0d738` |
| `tests/test_python_http_upload_accounted_traversal.py` | 1,006 | 20 | `ceeeebe8be2b581b2ba216dc349ced12624ad4c3` |
| `docs/development/change-playbook.md` | 17 | 0 | `fcb1514713644ae346e367358220852c64280725` |
| `docs/development/command-source-trace.md` | 13 | 2 | `847d19441cac15bc89217a002d9cab67891555ec` |
| `docs/development/source-architecture.md` | 37 | 0 | `6a4f8d4afb5312d18029ef0754fcedc16b6bab77` |
| `docs/development/source-data-model-catalog.md` | 3 | 1 | `ea357917bfa18e13e3a58fb3c4ce0092cb8a0241` |

Reject the candidate if any reviewed product/test/source-doc byte differs, the commit range is not
linear, the patch digest differs, or the changed-file set grows.

## Required context

Read in full before deciding:

- `AGENTS.md`
- `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-repair-20260730.md`
- `docs/development/ai/swarm-results/cfr168-python-http-upload-scale-review-claude-fallback-20260730/review.md`
- the exact cumulative diff from `5f798990…` through `76baff84…`
- all six final candidate files listed above
- the four canonical source docs named in the candidate diff
- upload-neighboring HTTP implementations and tests reached by every production
  `validate_sealed_commit` call site

The implementer reported an earlier 443/443 Python-pattern pass before the last cleanup commits.
Its final broad run was still in a background process when the implementation session ended and
that process was stopped. Therefore there is **no accepted final-head broad result**. Re-run the
gate on `76baff84…`; do not cite the interrupted run as passing evidence.

## Prior finding disposition — every row is mandatory

### F1 / Blocker — duplicate request-bypassing traversals and the missing final deadline fence

The prior live upload validated the publication root twice through fresh `local_v1()` ledgers, did
not poll the request deadline across those traversals, and could publish after 1.0 seconds elapsed
against a 100 ms request. Independently prove all of the following:

1. one upload uses the exact handler ledger and deadline for body through cleanup;
2. no live upload creates another sealed-validation ledger;
3. each publication root closure is traversed exactly once;
4. `RemoteJSONPublicationRootProof` is derived only from the request-owned validated snapshot,
   is single-use/revocable, and rejects a foreign store, root, operation, actor, key, or stale
   identity;
5. publication preflight reuses `RemoteJSONPublicationIdentity` without skipping a required
   identity/final-subject check;
6. the exact request deadline is polled after the last identity/subject check and immediately
   before each first effect: missing-project layout bootstrap, object copy, and
   branch/generation publication;
7. an already-expired request returns the typed deadline result with byte-identical objects,
   branch, generation, nonce, pending transaction, receipt, audit, and response authority;
8. the response slot cannot mark an effect started before the corresponding deadline fence.

Reproduce the exact elapsed `1.0 s` versus limit `100 ms` case and cover both existing-project and
missing-project paths. Record session identities, ledger identities, traversal counts, deadline
poll deltas, result bytes, and before/after durable-state digests.

### F2 / High — false per-record retained/temporary admission cliff

The prior formula charged `128×`/`129× maximum_record_bytes` without matching simultaneous live
owners, refusing a valid record around 128 KiB. Review the final
`concurrent_retained_upper`, `concurrent_temporary_upper`, `_stream_scratch_upper`,
`canonical_json_stream`, and `canonical_json_stream_digest` from claim sites and lifetimes, not
from their comments.

Prove:

- each retained and temporary term maps to an actual simultaneously-live owner and dimension;
- non-overlapping scratch maxima are not summed;
- streaming output is byte-identical to `canonical_json` for every supported value and text shape;
- bounded streaming never retains a whole-record duplicate;
- no public limit was raised and no record/file/aggregate validation was skipped;
- the real maximum admissible record size and record count are measured under unchanged
  `DecisionWorkProfile.http_v1()`;
- the first refusal is a named, observed owner rather than formula slack;
- the candidate's source-doc claim that a single record is admitted above 3 MiB and first refuses
  in staging is either reproduced with exact values or rejected.

### F3 / High — commit-source session constructed without a control owner

Independently instrument `_AccountedUploadSourceMemory` and every
`SealedCommitValidationSession` construction/insertion. Before allocation, an outstanding matching
reservation must exist for the absolutized `Path`, document memo, validated set, descriptor
registry, descriptor reuse pool, subdirectory map, every map/list insertion, every descriptor
owner, and the final ordered-id projection. A scalar already consumed into current usage is not
evidence of an outstanding construction reservation.

Verify acquisition publication before fallible work, ledger/deadline identity, exact dimension and
phase, release order, retry behavior after injected release failure, and zero retained/temporary
debt and permits on success and every failure.

### F4 / Medium — per-read descriptor retention and unclaimed ordered projection

Show that closed transient descriptor owners are recycled or removed and the registry is bounded
by simultaneously-open descriptors, not total record reads. Measure the registry/reuse-pool peak
at 1x/3x/10x and at wide-fan-in/deep-chain shapes. Prove `finalize` reserves the ordered-id
projection before list allocation and sorting. Exercise descriptor open/read/close/recycle faults;
cleanup must not double-close, reuse a stale numeric descriptor, erase retryable debt, or leave
ledger state behind.

## Candidate-specific closure claims to falsify

Review all new or materially changed surfaces:

- `_canonical_escaped_text`, `canonical_json_stream`, and `canonical_json_stream_digest`;
- `RemoteJSONPublicationRootProof` and `RemoteJSONPublicationIdentity`;
- `_AccountedUploadSourceMemory`;
- upload `fence_first_effect` and request-owned proof/identity wiring;
- `SealedCommitValidationSession.control_state_upper`, control claims/releases, descriptor reuse,
  ordered projection, and close;
- `AccountedHTTPUploadSealedValidation` retained/temporary formulas, publication-proof ownership,
  production `_assert_prepared_coverage`, and streaming fingerprint paths;
- caller-supplied ledger/deadline support in `validate_sealed_commit`;
- detached-document borrow-fingerprint release;
- removal of dead proof plumbing;
- the nine new focused tests plus the final record-count ceiling update.

Check parse/validate/plan/preflight/apply/render/write separation, exception precedence, callback
reentry, single-use state transitions, exact cleanup retry, alias/stale-reference rejection, and
that a coverage assertion cannot itself leave terminal debt or suppress the canonical response
after a durable effect.

## Required scale, boundary, and durable-state evidence

Use reviewer-authored fixtures in addition to executing candidate tests:

1. Preserve the default-profile 1x/3x/10x curves and the 1,201-commit 409 non-fast-forward
   reversal. Record counts, aggregate/max-record bytes, nodes, edges, reads, depth, frontier,
   reservation, claimed peak, ledger peak, slopes, outcome bytes, and zero-debt cleanup.
2. Exercise record sizes at 64 KiB, 126,976 B, 128 KiB, 256 KiB, 1 MiB, at least one admitted
   size above 3 MiB, and a practical near-request-body boundary.
3. Exercise ASCII, CJK, emoji, escape-heavy, control-character, and mixed Unicode records plus
   wide fan-in, signed history, stacked diamonds, shared DAG, and deep chain.
4. Bind exact, minus one, and plus one at request-body bytes, record/file bytes, aggregate bytes,
   retained bytes, temporary bytes, graph nodes/edges, depth, frontier, response body, response
   headers, session control, publication proof, and publication-response owner boundaries.
5. Snapshot objects, branches, generations, nonce, pending transaction, receipt, audit, response
   bytes, ledger current/peak, permits, and terminal state before every budget, deadline, tamper,
   encoding, validation, cleanup, preflight, and publication failure.
6. Preserve exact 200/409/error/retry bytes, signature required/optional precedence,
   fast-forward/non-fast-forward decisions, malformed/corrupt/missing history behavior,
   replay/idempotency semantics, and final-subject detection.

No result obtained by raising a public profile, reducing the scale fixture, bypassing sealed
validation, changing diagnostic precedence, or weakening subject identity is admissible.

## Required reversal and non-vacuity matrix

In temporary exports or runtime probes that leave the reviewer Git tree unchanged, prove the
focused evidence detects at least:

1. restoring either nested publication-root traversal;
2. removing each final effect deadline poll, including the layout-bootstrap poll;
3. omitting the source-session owner or any control reservation;
4. restoring the prior per-record multiplier;
5. retaining one descriptor owner per read;
6. allocating/sorting the ordered-id projection without its claim;
7. replacing bounded streaming with a whole-record encode or changing one emitted canonical byte;
8. disabling production `_assert_prepared_coverage`;
9. accepting a reused/revoked/foreign publication proof or identity;
10. omitting detached-document borrow-fingerprint release.

Run repair-specific tests against `dece688afea815d1242e7751367052d1ceac4580` as well as the exact
candidate. A baseline failure caused only by a missing new symbol, fixture import failure, or
different line number is vacuous. Report the exact counterexample and the assertion that changes
from failing to passing.

## Neighboring HTTP audit

Enumerate every production `validate_sealed_commit` call site and its shortest route from HTTP or
CLI dispatch. Confirm the upload route has no fresh-ledger escape. The candidate source docs claim
four HTTP-reachable owners remain outside this repair:

- `recover_remote_pending_quarantine`;
- `handle_http_list`;
- `http_remote_branch`;
- `validate_http_merge_request_record`.

Verify that this list is exhaustive, that none currently receives an upload request
ledger/deadline that the candidate silently drops, that the new reusable accounted API is adequate
for their separately tracked repair, and that the upload change does not regress their behavior.
Any fifth untracked escape, upload-reachable fresh ledger, lost deadline, or behavior regression is
`REQUEST_CHANGES`.

Run the focused neighboring ancestry/signature/sealed/external suites and selected legacy upload,
nonce, receipt, Doctor, branch, MR list/review/apply, recovery, and publication cases. State exact
test names/counts rather than citing a general suite label.

## Required validation

Run on the exact candidate tree (a temporary clean export is acceptable). Set
`PYTHONDONTWRITEBYTECODE=1` so the report-only worktree remains clean:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_upload_accounted_traversal.py tests/test_codefire_cli.py
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
  5f798990c313ea2e1b117bc74824de3f3ce028d2..\
76baff84cb1192e11b26dcc669fb800f7b13fa30
```

Also run the exact selected legacy methods and reviewer-authored probes required above. Record
command, exact revision/export, result, count, elapsed time, and what each command proves. If the
full `AGENTS.md` all-test/demo gate is not run in this lane, mark it explicitly pending for the
manager integration gate; do not imply that the Python-pattern sweep proves the all-language
suite.

## Decision rule and output

Return `ACCEPT` only when every mandatory item above is proved on the exact candidate and no new
Blocker/High/Medium residual remains. Otherwise return `REQUEST_CHANGES`; list each finding with
severity, exact location, reproducer, state impact, and minimal closure requirement. Do not use a
conditional acceptance.

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-rereview-fugu-20260730/review.md`

The report must include:

- exact base/head/tree/patch/report/ticket hashes;
- reviewer/model independence;
- decision and strongest correctness/maintainability concerns;
- one disposition row for F1-F4 and every prior residual risk;
- candidate-surface and commit-series disposition;
- formulas, owner-lifetime table, scale/boundary measurements, and durable-state snapshots;
- reversal/non-vacuity matrix;
- neighboring HTTP call-site audit;
- exact commands, counts, timings, skipped/pending gates, and evidence limits;
- new findings and residual risk.

Commit only that report and leave the reviewer worktree clean.
