# CFR-168 C14-LIVE-A1 C4-Bound Adjacent-Oracle Amendment/FREEZE

Status: `ready_for_fresh_independent_preflight`

Packet ID:
`cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803`

Lifecycle position: exact C4 is frozen; this F5 packet authorizes no test edit
until one fresh independent PF5 returns the exact verdict required below

Execution shape: one fresh independent report-only PF5, one bounded test-only
O1 writer, one result-only evidence writer, and one fresh independent exact
reviewer; the manager retains candidate, integration, and lifecycle authority

Governing packets:

- [`cfr168-c14-live-a1-plan-freeze-20260802.md`](cfr168-c14-live-a1-plan-freeze-20260802.md)
- [`cfr168-c14-live-a1-protected-exception-traceback-reuse-repair-20260803.md`](cfr168-c14-live-a1-protected-exception-traceback-reuse-repair-20260803.md)

This packet is a minimal additive amendment. It freezes one later test-only
oracle correction and nothing else. It changes no product byte, test byte,
canonical source document, result, review, issue state, or lifecycle state.

## Objective

Freeze the exact C4-bound, two-literal adjacent-oracle correction, require a
fresh independent PF5 before O1, and preserve C4 and every behavior outside
those two later expected-literal tokens byte-for-byte.

## 0. Frozen Decision

The adjacent mismatch is a stale test oracle, not a new product defect.

C4 preserves A1's accepted error-precedence invariant:

```text
outer / primary: exact first construction or operation exception
retained cleanup evidence: exact first cleanup-close exception
```

The two adjacent tests named below inject both a construction failure and a
cleanup-close failure, but their outer `assertRaisesRegex` still expects the
cleanup-close text. That expectation conflicts with the accepted invariant and
with C4's intended behavior. Correcting only those two expected literals makes
the oracle semantically accurate; it does not repair or change production
behavior.

The complete authorization-changing inventory is:

1. C4 remains the immutable product candidate for this episode.
2. The mismatch affects test-oracle correctness only.
3. Exactly two literal replacements in one existing test file may be made by
   a later O1 commit.
4. Both injected cleanup-close literals and callbacks remain byte-identical.
5. No assertion may be deleted, weakened, rewritten, reordered, or added.
6. No product source, fault injection, cleanup retry, object identity,
   accounting, authority, API, wire, or persisted-data behavior may change.
7. PF5 must independently accept this contract before O1 exists.
8. PF5 is sibling report evidence and is never inserted into O1 ancestry.
9. O1 descends directly from F5 and changes no byte except the two literal
   tokens frozen in section 3.
10. Result-only evidence and one fresh exact review bind the exact tuple
    `C4 + F5 + PF5 + O1`.
11. The C4 author's candidate-worktree mutation logs are inadmissible; any
    final reversal evidence must be recreated in a real throwaway export.
12. This packet claims no acceptance, integration, A2/B3 start, CFR-168
    closure, CFB-138 closure, Install, Dogfood, Evaluate, Close, or
    Retrospect.

## 1. Immutable C4 Subject

### 1.1 Commit identity

| Item | Exact value |
|---|---|
| C4 commit | `19010f742d358c5c03e19d091c4249420f9e53b9` |
| C4 tree | `a0ec4ee256f63c7fe7de438eb5c978c918b3d22f` |
| C4 parent / F4 | `8ff33fd9b354a169380b9401f342b96cc6785ad7` |
| F4 tree | `7b76202ce9f8bc684bd4cf0de8310ef56c754038` |
| F4 parent / C3 | `46ed2a957fcf66821387b3414e075c98e31209be` |
| C4 commit count over F4 | exactly `1` |
| C4 merge count over F4 | exactly `0` |

F5 authoring must start from exact clean C4. A mismatch in commit, tree,
parent, or status aborts without edits.

### 1.2 Exact two-path C4 write set

`git diff --name-status F4..C4` is exactly:

```text
M	codefire
M	tests/test_python_http_upload_accounted_traversal.py
```

| Path at C4 | Mode | Git blob | SHA-256 |
|---|---|---|---|
| `codefire` | `100755` | `15c7ebf5e2c53216c1aaa1add12070cffc12cbdd` | `dc40d2c02c343c076204ce680018c33871c43b19cccf45d274aa0ac960af1124` |
| `tests/test_python_http_upload_accounted_traversal.py` | `100644` | `c609e2a84fbcd5f3dca3791cfa106abca369a374` | `4905784910368bfa9cd16d51eaf8a7e1d454d2bee52f6b9bbe3c27b0c96be054` |

The cumulative C4 binary patch binding uses the repository's ordinary binary
diff form, without `--full-index`:

```bash
git diff --binary \
  8ff33fd9b354a169380b9401f342b96cc6785ad7 \
  19010f742d358c5c03e19d091c4249420f9e53b9 | sha256sum
```

Required SHA-256:
`ae001ae07eb9e3a7be5eb9a3e7c38474abb762d4ae29e9bd1d820c3cf7bb09f2`.
Another diff option set is not a substitute for this binding.

### 1.3 O1 target preimage

The later O1 target is unchanged by C4 and is bound before editing:

| Path at C4 and F5 | Mode | Git blob | SHA-256 |
|---|---|---|---|
| `tests/test_python_existing_terminal_owner.py` | `100644` | `cc51e5ebcc4f432f19a5b235c92b6bcae3b69215` | `0ccbe0a9798c795ae4c5414ab3fb3b193d6558305450f40f919e115c0b6069ff` |

If F5 changes that blob, or O1 does not begin from that exact preimage, the
subject is invalid and returns to manager Freeze.

## 2. PF4 Sibling Evidence Binding

PF4 is prior report-only evidence for the C4 repair contract. It is not an
ancestor of C4 and is not inserted into this packet or O1 ancestry.

| Item | Exact value |
|---|---|
| PF4 report commit | `6e3dc1cef9223981c4f8a2f7d67108bbaefd2eab` |
| PF4 tree | `38ba565a841bee7db9b7dbef2a61583e682b98d9` |
| PF4 parent | `8ff33fd9b354a169380b9401f342b96cc6785ad7` (= F4) |
| PF4 report path | `docs/development/ai/swarm-results/cfr168-c14-live-a1-protected-exception-traceback-reuse-preflight-20260803/preflight.md` |
| PF4 report blob | `ef0e46b83823a7a66035be303d9763f4b30de5b1` |
| PF4 report SHA-256 | `d7f46d4758901d20c40236cb0bb71e4b1183b90d159b3f24136cb01b4dcb1844` |
| PF4 terminal verdict | `PREFLIGHT_VERDICT: ACCEPT_CONTRACT` |

The independently recomputed topology facts are:

```text
PF4^ = F4
C4^  = F4
merge-base(PF4, C4) = F4
PF4 is not an ancestor of C4
```

PF4 supports the bounded C4 repair contract. It does not preflight this F5
amendment, accept C4, or authorize O1. That is PF5's separate responsibility.

## Scope

The only later implementation scope is the one-path, two-token O1 change in
section 3. F5 itself changes only this packet. PF5, W5, and R5 are report-only
evidence nodes with the topology and write boundaries frozen below.

## 3. Exact O1 Authorization

O1 may modify exactly:

```text
tests/test_python_existing_terminal_owner.py
```

The complete authorized semantic diff is:

| ID | Exact method | Exact expected literal before | Exact expected literal after |
|---|---|---|---|
| `C14-A1-C4-AO-01` | `PythonExistingTerminalOwnerTests.test_partial_verification_child_is_published_and_retried` | `partial verification close` | `partial verification construction` |
| `C14-A1-C4-AO-02` | `PythonExistingTerminalOwnerTests.test_partial_ancestry_construction_and_auto_close_are_reconciled` | first outer `partial ancestry close` | `partial ancestry construction` |

Only the string token passed as the named outer `assertRaisesRegex` expected
literal changes. The intended token-level replacements are:

```diff
-                KeyboardInterrupt, "partial verification close"
+                KeyboardInterrupt, "partial verification construction"
```

and:

```diff
-                KeyboardInterrupt, "partial ancestry close"
+                KeyboardInterrupt, "partial ancestry construction"
```

The following bytes remain unchanged:

- `raise KeyboardInterrupt("partial verification construction")`;
- `raise KeyboardInterrupt("partial verification close")`;
- `raise KeyboardInterrupt("partial ancestry construction")`;
- `raise KeyboardInterrupt("partial ancestry close")`;
- both cleanup callbacks and all callback installation/restoration code;
- child publication to the terminal owner and policy authority;
- every retry and close-call check, including `len(close_calls) >= 2`;
- every closed-child, cleared-owner, and abort-only assertion;
- every zero-ledger assertion;
- the later `post ancestry auto-close` expectation;
- every other assertion, call, method name, import, fixture, helper, local,
  quote style, line break, indentation byte, and whitespace byte outside the
  two replaced literal tokens; and
- every other repository path.

No assertion may be deleted, weakened, combined, generalized, replaced by a
type-only check, or made conditional. No new test or comment is authorized. If
the exact two replacements are insufficient, O1 stops and returns evidence to
manager adjudication without another edit.

## 4. Why This Is an Oracle Correction

The two tests each create this sequence:

1. construction begins and publishes a partially initialized child;
2. construction raises the named construction exception;
3. cleanup closes or retries the published child;
4. the first cleanup-close attempt raises the named cleanup exception; and
5. A1 retains the construction exception as primary while attaching cleanup
   failure as cleanup evidence.

The existing outer regex expects step 4 to replace step 2. That is the stale
pre-A1 cleanup-first oracle. C4 correctly preserves step 2 at the outer
boundary. The remaining assertions continue to prove publication, retry,
closure, abort-only transition, and zero accounting.

| Impact axis | Frozen adjudication |
|---|---|
| Product correctness | unaffected; C4 follows the accepted precedence contract |
| Test-oracle correctness | affected; two outer message literals are stale |
| State and cleanup retry | unaffected; all state and retry assertions remain |
| Object identity and evidence chain | unaffected; no identity or link behavior changes |
| Accounting | unaffected; all zero-ledger checks remain |
| Authority/security | unaffected; no capability or validation authority changes |
| API and wire | unaffected; no command, status, response, or payload changes |
| Persisted data | unaffected; no object, schema, transaction, or receipt changes |
| Actionability | required correction, exactly the two test literal tokens |

Changing `codefire` to satisfy the old literals would reverse A1's accepted
error precedence and would be outside this packet.

## Non-Goals

This packet does not authorize product repair, a third literal, a new test,
assertion weakening, callback or fault-injection changes, cleanup behavior
changes, canonical-doc synchronization, installer or environment repair,
integration, A2/B3 work, or any lifecycle acceptance or closure.

## 5. Frozen Topology

Use the following no-merge topology:

```text
F4  8ff33fd9b354a169380b9401f342b96cc6785ad7
|\
| PF4 6e3dc1cef9223981c4f8a2f7d67108bbaefd2eab  prior sibling evidence
|
C4  19010f742d358c5c03e19d091c4249420f9e53b9  immutable product candidate
|
F5  this packet-only commit
|\
| PF5 fresh independent report-only preflight sibling
|
O1  one test-only commit, parent exactly F5
|
W5  one result-only evidence commit
|
R5  one fresh independent exact-review report-only commit
```

F5 adds exactly:

```text
docs/development/ai/task-packets/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803.md
```

PF5 is created from F5 and adds exactly one preflight report path. O1 is also
created from F5, but only after the committed PF5 report ends with the exact
line:

```text
PREFLIGHT_VERDICT: ACCEPT_CONTRACT
```

PF5 is sibling evidence, not an ancestry node. It must not be cherry-picked,
merged, rebased, squashed, copied, or otherwise inserted into O1, W5, or R5
ancestry. O1's parent is exactly F5. Evidence records bind PF5 by its separate
commit, tree, parent, report blob, and report SHA-256.

W5 descends from O1 and adds exactly one result report. R5 descends from W5 and
adds exactly one exact-review report. Both reports bind exact C4, F5, PF5, and
O1 identities. Any other byte change, merge, restoration commit, or extra path
invalidates the subject.

## 6. PF5 Contract

PF5 is a fresh independent contract preflight. It must not inherit the F5
author's session, the C4 implementation session, the PF4 session, or any
candidate-mutation session. Provider diversity alone is not independence.

PF5 independently verifies:

1. every C4 commit/tree/parent, path, blob, content digest, and binary patch
   binding in sections 1 and 2;
2. PF4 is the bound report-only sibling and is not a C4 ancestor;
3. the two current test contexts inject both construction and close failures;
4. A1's accepted primary/cleanup precedence makes the two outer literals stale;
5. the two token replacements are sufficient and do not weaken any independent
   publication, retry, closure, state, or accounting oracle;
6. the O1 write set and byte-level negative space are closed and executable;
7. the topology keeps PF5 out of candidate ancestry; and
8. the validation sequence and evidence requirements below are complete.

PF5 is report-only. It may use Git/object/digest inspection, static source/test
analysis, and static docs/AI checks. It does not edit or restore C4/F5, author
O1, run a mutation in candidate bytes, claim implementation success, or claim
acceptance. A verdict other than the exact terminal line above returns to
manager Freeze and leaves O1 unauthorized.

## Validation

Sections 7 and 8 define the exact O1 identity proof and ordered post-O1 gates;
section 12 defines the strictly static F5 authoring-lane validation.

## 7. O1 Identity and Negative-Space Proof

After PF5 accepts the contract, the O1 writer starts from exact clean F5 and
commits one non-merge test-only change. The result evidence must prove:

```bash
git rev-parse C4 C4^{tree} C4^ F5 F5^{tree} F5^ O1 O1^{tree} O1^
test "$(git rev-parse O1^)" = "$(git rev-parse F5)"
test "$(git rev-list --count F5..O1)" -eq 1
test -z "$(git rev-list --merges F5..O1)"
git diff --name-status C4..F5
git diff --name-status F5..O1
git diff --numstat F5..O1
git diff --unified=0 F5..O1 -- \
  tests/test_python_existing_terminal_owner.py
git diff --binary F5..O1 | sha256sum
git diff --exit-code C4..O1 -- codefire
git diff --check C4..O1
git status --porcelain
```

Required results:

- `C4..F5` adds only this packet;
- `F5..O1` modifies only
  `tests/test_python_existing_terminal_owner.py`;
- `F5..O1` is exactly two insertions and two deletions;
- the unified-zero-context diff contains exactly the two replacements in
  section 3;
- `O1:codefire` is byte-identical to `C4:codefire`;
- every C4 path except the adjacent target is byte-identical in O1;
- the adjacent target before blob/SHA-256 matches section 1.3;
- the result records the after blob/SHA-256 and O1 binary patch SHA-256;
- no merge or extra commit exists; and
- status and whitespace checks are clean.

## 8. Post-O1 Validation Sequence

All Python commands use exactly the Python 3.10 interpreter form
`PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B`. Ordinary `-I -S` is
forbidden. Every command receives an outer timeout of at least 600 seconds;
900 seconds is the default bound below.

For every gate, W5 records the exact revision, command, UTC start/end,
wall-clock duration, exit status, selected/passed/failed/error/skipped counts
when applicable, and SHA-256 of the complete stdout/stderr log. A timeout,
killed process, missing terminal summary, or unavailable capability is never
reported as PASS.

The order is mandatory. A failed earlier stage stops later product gates and
returns the exact evidence to manager adjudication.

### 8.1 Two exact corrected methods

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v \
  tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_verification_child_is_published_and_retried \
  tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_ancestry_construction_and_auto_close_are_reconciled
```

Required result: exactly 2 selected, 2 passed, 0 failed, 0 errors, 0 skipped.

### 8.2 Six adjacent modules

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_signature_composite \
  tests.test_python_existing_terminal_owner \
  tests.test_python_existing_consumer_terminal_freeze \
  tests.test_python_sealed_owner \
  tests.test_python_sealed_borrow
```

Required result: exactly 81 selected, 81 passed, 0 failed, 0 errors, 0
skipped. O1 adds no test method, so the count must remain 81.

### 8.3 Nine unchanged A1 modules

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v \
  tests.test_python_http_upload_accounted_traversal \
  tests.test_python_ancestry_owner \
  tests.test_python_ancestry_borrow_handoff \
  tests.test_python_signature_authority \
  tests.test_python_signature_owner \
  tests.test_python_external_verification \
  tests.test_python_external_production \
  tests.test_python_signature_verification_handoff \
  tests.test_python_sealed_production
```

Required result: exactly 184 selected, 184 passed, 0 failed, 0 errors, 0
skipped. O1 does not touch any of these nine modules.

### 8.4 Separately authorized compile gate

Only after sections 8.1-8.3 pass, run:

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m py_compile \
  codefire \
  tests/test_codefire_cli.py \
  tests/test_python_http_upload_accounted_traversal.py \
  tests/test_python_ancestry_owner.py \
  tests/test_python_ancestry_borrow_handoff.py \
  tests/test_python_signature_authority.py \
  tests/test_python_signature_owner.py \
  tests/test_python_external_verification.py \
  tests/test_python_external_production.py \
  tests/test_python_signature_verification_handoff.py \
  tests/test_python_sealed_production.py \
  tests/test_python_existing_terminal_owner.py
```

Required result: 12 exact paths compiled, exit 0, and no repository bytecode
artifact.

### 8.5 Separately authorized broad Python gates

First run the Python-only discovery:

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest \
  discover -s tests -p 'test_python_*.py' -v
```

C4 contains exactly 558 `test_` methods in these modules, and O1 adds or
removes none. Required result: exactly 558 selected, 558 passed, 0 failed, 0
errors, 0 skipped.

Then run one stable full discovery:

```bash
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest \
  discover -s tests -v
```

C4 contains exactly 786 `test_` methods across `tests/test_*.py`, and O1 adds
or removes none. W5 records the exact 786-test terminal summary, duration,
exit, and log digest. Any environment-dependent installer failure must be
compared with a manager-bound exact-C4 same-environment baseline by complete
method set and traceback signature. It remains
`unavailable/environmental`, never PASS. Any additional or changed failure is
candidate-created evidence and returns to manager adjudication.

This packet does not authorize `./demo.sh`, Cargo/Rust, installation, package
mutation, or an environment repair.

### 8.6 Separately authorized docs/AI gates

Run each with an outer 900-second timeout and the exact interpreter form:

```bash
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_source_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_project_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_contracts.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_acceleration_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_development.py --timeout 600 --skip-cli
git diff --check C4..O1
git status --porcelain
```

Required result: every docs/AI gate exits 0; diff and status are clean. W5
records each command separately with duration, exit, and complete-log digest.

## 9. C4 Implementation-Procedure Observation

C4's author temporarily mutated the candidate worktree while producing
reversal evidence, despite F4 requiring every mutation to occur only in a
throwaway export. This is an implementation-procedure nonconformance, not a
new product defect.

The adjudication is exact:

- every mutation log produced while C4 candidate bytes were edited is
  inadmissible as final reversal evidence;
- committed C4 is clean and its two product/test blobs are byte-identical to
  the pre-mutation blob and SHA-256 bindings in section 1.2;
- exact C4 ran the nine-module gate 184/184 before mutation;
- after restoration, exact C4 ran the combined focused/inherited gate 10/10;
- those restoration facts do not make the mutation logs admissible, but they
  do mean the committed product bytes are not contradicted; and
- no product repair, C5, restoration commit, or extra test edit is authorized
  by this observation.

Any final mutation/reversal claim carried into W5 must be recreated from exact
C4/O1 source in a real throwaway export, for example an isolated directory
materialized from `git archive O1`. The result records the export source
identity, initial path digests, each one-at-a-time reversal, expected failing
oracle, actual failure, duration, exit, log digest, and restored export digest.
The export may be discarded after capture. C4 and O1 must never be edited,
restored, or used as mutation worktrees.

## 10. W5 Result-Only Evidence

W5 is one report-only commit descending from O1. It binds and records:

- full commit/tree/parent identity for C4, F5, PF5, O1, and W5;
- proof PF5 is a report-only child of F5 and is not in O1 ancestry;
- F5 packet path, Git blob, SHA-256, and one-added-path proof;
- PF5 report path, Git blob, SHA-256, provider/model/session provenance, and
  exact terminal verdict;
- C4's exact two-path name-status, blob map, content digests, mode, and
  cumulative binary patch SHA-256;
- O1's exact one-path, two-token diff, commit count, zero-merge proof, before
  and after target blob/SHA-256, and binary patch SHA-256;
- exact results for sections 8.1-8.6 with counts, durations, exits, and
  complete-log SHA-256 values;
- honest baseline treatment for any unavailable full-discovery evidence;
- the implementation-procedure observation and newly recreated throwaway
  reversal evidence from section 9;
- proof no product, authority, API, wire, persisted, accounting, identity, or
  cleanup-retry behavior changed in O1; and
- remaining risks without an acceptance, integration, or closure claim.

W5 modifies no candidate, packet, test, canonical source document, or PF5
report byte. It does not claim its own commit identity was part of O1.

## 11. Fresh Exact Review

R5 is one fresh independent report-only review after W5 exists. Exclude:

- the F5 packet author/session;
- the PF5 author/session;
- the C4 implementation and candidate-mutation session;
- the PF4 author/session;
- the O1 writer/session;
- the W5 evidence writer/session;
- any session that edited, restored, amended, rebased, or mutated C4/O1; and
- the manager adjudication session as the independent reviewer.

R5 independently recomputes and reviews exact `C4 + F5 + PF5 + O1`, uses W5
only as claimed evidence, and answers:

1. Are all C4, F5, PF5, and O1 object and digest bindings exact?
2. Is PF5 sibling report evidence, absent from O1 ancestry, with exact
   `PREFLIGHT_VERDICT: ACCEPT_CONTRACT`?
3. Does O1 contain only the two authorized expected-literal replacements?
4. Are both cleanup-close injections, callbacks, and every independent
   assertion byte-identical?
5. Are 2/2, 81/81, and 184/184 exact and ordered before broader gates?
6. Are compile, 558-test Python discovery, stable full discovery, and docs/AI
   evidence complete with counts, durations, exits, and log digests?
7. Were all final reversals recreated in a real throwaway export without
   editing C4/O1?
8. Did any product, state, identity, accounting, authority, API, wire, or
   persisted behavior change?
9. Does the subject remain bounded to A1's two stale adjacent literals?
10. Are all acceptance, integration, A2/B3, issue-closure, and lifecycle
    decisions still manager-owned?

R5 may approve or request changes for this exact tuple only. It may not repair
a file, widen the oracle, insert PF5 into ancestry, integrate, or close an
issue.

## 12. F5 Authoring-Lane Boundary and Validation

The F5 author changes only this packet path. Freeze-lane validation is limited
to Git/object/digest inspection and static docs/AI checks. The author runs no
product test, adjacent gate, broad discovery, demo, Cargo/Rust, install,
mutation, or candidate-changing command.

Before the commit:

```bash
git rev-parse HEAD HEAD^{tree} HEAD^
git status --porcelain
git diff --check
```

Run each applicable static check with an outer timeout of at least 600 seconds
and the exact Python 3.10 form:

```bash
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_source_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_project_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_contracts.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_acceleration_docs.py
timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B \
  docs/development/tools/check_ai_development.py --timeout 600 --skip-cli
```

Commit with the exact message:

```text
docs(cfr168): freeze C4-bound adjacent oracle amendment
```

After the commit, prove:

```bash
git rev-parse HEAD HEAD^{tree} HEAD^
test "$(git rev-parse HEAD^)" = \
  "19010f742d358c5c03e19d091c4249420f9e53b9"
test "$(git rev-list --count \
  19010f742d358c5c03e19d091c4249420f9e53b9..HEAD)" -eq 1
test -z "$(git rev-list --merges \
  19010f742d358c5c03e19d091c4249420f9e53b9..HEAD)"
git diff --name-status \
  19010f742d358c5c03e19d091c4249420f9e53b9..HEAD
git diff --check \
  19010f742d358c5c03e19d091c4249420f9e53b9..HEAD
git ls-tree HEAD -- \
  docs/development/ai/task-packets/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803.md
git show HEAD:docs/development/ai/task-packets/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803.md | sha256sum
git diff --binary \
  8ff33fd9b354a169380b9401f342b96cc6785ad7 \
  19010f742d358c5c03e19d091c4249420f9e53b9 | sha256sum
git status --porcelain
```

Required result: exactly one non-merge commit, exactly one added path (this
packet), the bound C4 patch digest, clean diff checks, and empty status.

## Handoff

The F5 author returns:

```text
Role: bounded Freeze/Plan author
Objective: freeze the C4-bound two-literal adjacent-oracle amendment
Files changed: this packet only
Commits: exact F5 commit/tree/parent and one-non-merge proof
Bindings: packet blob/SHA-256 and C4 binary patch SHA-256
Validation: static docs/AI command, timeout, duration, exit, and path proof
Risks: no product evidence rerun here; PF5, O1, W5, and R5 remain pending
Next: fresh independent PF5; do not author PF5 in this lane
```

This packet does not accept or integrate C4/A1, authorize A2/B3, close CFR-168
or CFB-138, or authorize Install, Dogfood, Evaluate, Close, or Retrospect.
