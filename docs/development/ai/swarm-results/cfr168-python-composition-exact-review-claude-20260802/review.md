# CFR-168 Python accepted-input composition exact review

Date: 2026-08-02

Review kind: fresh exact, report-only

Terminal reviewer: Codex takeover after the Claude Opus 5 evidence run reached its provider execution limit

Decision subject: immutable candidate `C` only

## 1. Verdict

**ACCEPT** — scoped exactly to candidate
`cf1ac6a7d61d32cb8138c127e649151c5c545196`, tree
`531d07fe121a1ac49c5a49462f0ce99f4270c595`, over frozen launch commit
`6db7944b26399c3bc43867f12ee9b3f260988621`.

The accepted subject is the 18-path candidate delta `L..C`, whose binary patch
SHA-256 is
`ca496cbe48714c4f97f35c2ad30d01f9c07e82b670d0db315349a8e86f53c68a`.
The aggregate writer report commit
`b7aadd591a4ad494668cd671aa4369b368879ce1` is evidence, not part of the
product subject.

There are **zero candidate-created required corrections**, zero undecided
candidate observations, and no implementation variance requiring manager
adjudication before this verdict. The candidate preserves all accepted input
behavior, composes the disjoint accepted deltas once, and implements exactly the
frozen `C18` cleanup-origin bridge needed to make the accepted inputs coexist.

This verdict does **not** close CFR-168, CFB-136, CFB-138, all of Upload, global
`C14`, repository-wide `C18`, the original shared-primitives preflight, `B1`,
`B2`, `B3`, MR-write, integration, Install, Dogfood, or any predecessor-gated
successor. It does not authorize moving root row 6 away from
`Implement in progress`. The three registered `C14` successor handles remain
OPEN and unimplemented.

## 2. Review independence and evidence boundary

The original Claude Opus 5 review execution completed the expensive evidence
collection, including the 228-test CLI gate and the 767-test full suite, but
stopped before authoring a terminal report because the provider limit was
reached. The terminal takeover did not inherit or assert an unpublished verdict.
It reread the governing protocol, launch packet, typed companion, both
preflights, all three accepted-input reports, the writer result, and the frozen
candidate bytes; then independently recomputed the decisive topology, blobs,
patch identity, duplicate proof, three-stage composition, AST disjointness,
bridge surface, negative space, live probes, and required bridge reversals.

Evidence labels used below:

- **reviewer-executed**: executed in the fresh exact review lane against an
  extracted or cloned immutable `C` tree.
- **independently recomputed**: derived again from Git objects or candidate
  bytes by the terminal reviewer, rather than copied from the writer report.
- **writer/manager evidence audited**: durable evidence in `W` was checked for
  identity, command scope, result, and candidate binding; the terminal reviewer
  did not relabel it as its own run.

The reviewer used scratch outside the repository and wrote no candidate,
product, test, governance, tool, or accepted-input file. This report is the only
authorized repository write.

## 3. Subject identity and topology

| Item | Independently verified value | Result |
|---|---|---|
| `L` | `6db7944b26399c3bc43867f12ee9b3f260988621`, tree `69b9f7414ef0104a5782493f8731e1cd3636795f` | PASS |
| `C` | `cf1ac6a7d61d32cb8138c127e649151c5c545196`, tree `531d07fe121a1ac49c5a49462f0ce99f4270c595`, parent exactly `L` | PASS |
| `W` | `b7aadd591a4ad494668cd671aa4369b368879ce1`, tree `f00dd97a5af1e35a9998fbbbfe94b16329c491e0`, parent exactly `C` | PASS |
| `L..C` | 1 commit, 0 merges | PASS |
| `C..W` | 1 commit, 0 merges | PASS |
| `L..W` | 2 commits, 0 merges | PASS |
| `A..L` | 6 linear docs-only commits, 0 product/test/packaging paths | PASS |
| Candidate write set | exactly the frozen 18 paths; no deletion | PASS |
| Writer write set | exactly one report `ADD`; candidate-path diff `C..W` empty | PASS |
| Candidate self-reference | exact `C` SHA, tree and patch digest occur 0 times in the 18 candidate paths | PASS |

`A = a972cde` is an ancestor of `L`. The six `A..L` commits are linear and touch
only the two preflights, launch packet, typed companion, and burn-down control
documents. They do not alter the product base used by the composition.

The exact `L..C` write set is one executable, five tests, four canonical source
documents, `cfr-168.md`, and seven accepted Upload evidence files. There is no
agent state, log, worktree metadata, package surface, Rust source, or management
closure document in the delta.

The final report commit `R` cannot contain its own SHA without circularity.
Direct-parent and `L..R` topology are therefore post-commit handoff checks: `R`
must be a direct child of `W`, change this one path only, and make `L..R` exactly
3 commits with 0 merges.

## 4. Exact blobs and accepted-input binding

### 4.1 Candidate product and test blobs

| Path | Mode / blob at `C` | Result |
|---|---|---|
| `codefire` | `100755` / `73b1f368f9c1591be6fd9219b839351878b82b9a` | exact |
| `tests/test_codefire_cli.py` | `b23c43ced6ccb69c0bb553f9856c1a7b2145e770` | exact |
| `tests/test_python_http_doctor_decision_work.py` | `0f993edd6c6a57ee5ca0d15abc04e2260c3d48cd` | exact |
| `tests/test_python_http_branch_decision_work.py` | `c0269ee80e00265d0144520ab975ce7e4bc9b7e3` | exact |
| `tests/test_python_http_mr_list_decision_work.py` | `0d49841a2e682e661d169274e571e198f3408aa7` | exact |
| `tests/test_python_http_upload_accounted_traversal.py` | `aba7ac9fb4f73bfc1257da00d2a4ddbcbf6f55a4` | exact |

### 4.2 Final canonical source-document blobs

| Path | Blob at `C` | Result |
|---|---|---|
| `docs/development/source-architecture.md` | `fe2b3b74f2beaf30218333f14e05b6e4e6d4e699` | exact |
| `docs/development/command-source-trace.md` | `361a5de8f9f881fbedf35ed5a833baade9529e38` | exact |
| `docs/development/change-playbook.md` | `18d882813e02ed8cd795aa1e46d8cac6a09be4ef` | exact |
| `docs/development/source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | exact |

### 4.3 Accepted report and duplicate proof

- The terminal Upload ACCEPT report SHA-256 is
  `46e7c3e41ac1b132766657e445e017b392ce6a0b72eb46f909b13755fc8a9b3c`.
- Its bound repair packet SHA-256 is
  `756bf200138f6066dc1f2ac13d4933cf3e4a4228742bfc079d0c1ce3da2dabe9`.
- The Doctor/Branch and MR-list ACCEPT reports remain the exact accepted blobs
  and digests identified by the launch packet.
- `merge-base(701469f, b164046) = 701469f`; the Doctor/Branch accepted tree is
  already contained in the MR-list lineage.
- `git show 0703f71 | git patch-id --stable` and
  `git diff 5254e3d cb13da0 | git patch-id --stable` both produce
  `9889888cedeba7344fb987ff5891cf728d9c7e36`.

The initial Upload patch is therefore present exactly once through the
MR-list lineage; the Upload replay correctly begins after its duplicate. The
accepted Upload test module had 36 unique test methods. Candidate `C` has 37,
and the only added method is
`test_c18_cleanup_origin_joins_the_one_shared_error_taxonomy`. The MR-list test
module is byte-identical to its accepted source.

## 5. Three-stage composition reconstruction

### 5.1 Stage one: accepted-input union

The terminal reviewer reconstructed the five merge outputs directly from base,
OURS, and THEIRS blobs with `git merge-file -p --diff3`. Every command exited 0.

| Output | Reconstructed blob | Expected | Result |
|---|---|---|---|
| pre-bridge `codefire` | `f5e0b6db7c15aeadd3025ae4f038e301a7634caf` | same | PASS |
| `source-architecture.md` | `d8d7aa9881b5f96bcfa13b78168403be9ae62512` | same | PASS |
| `command-source-trace.md` | `0a8d2a886c883a051fe8c0e74bf550da3821b977` | same | PASS |
| `change-playbook.md` | `9a3afd7dd932bd3db7f33909e3cda40fb867abfc` | same | PASS |
| `source-data-model-catalog.md` | `2b68347eaa357f187f794c2a76992401743929eb` | same | PASS |

The pre-bridge AST proof was independently recomputed over qualified definitions
and ordered module statements with multiplicity:

| Measure | Base | OURS | THEIRS | Merged |
|---|---:|---:|---:|---:|
| qualified units | 1088 | 1163 | 1136 | 1211 |
| module statements | 84 | 94 | 92 | 102 |

OURS changes 101 units and THEIRS changes 78; their intersection is **0**.
Every side-changed unit is byte/AST exact in the merged output, all 1034 units
untouched by either side have zero drift, and there are zero duplicate qualified
definitions or qualified-name collisions. No module statement is removed;
merged additions equal the OURS+THEIRS multiset exactly; and the base, OURS, and
THEIRS module orders are each preserved as subsequences of the merged order.

### 5.2 Stage two: existing-authority citation

The reconstructed stage-one documents were compared with the frozen cited
stage:

- architecture becomes blob `5a1755ebd7e0eff8f55f92ae940557badb3ca567`;
- command trace becomes blob `72953356fca0982596bbb81a939fd4e2f2584a75`;
- playbook and catalog are byte-identical to stage one.

The only semantic addition is the preflight-prescribed relative citation to the
existing shared-primitives authority at the `C14`/`B2` mentions, with minimal
line reflow. No new authority was invented and no accepted claim was changed.

### 5.3 Stage three: frozen `C18` bridge

The stage-two-to-final product delta adds exactly two qualified units and
modifies exactly three:

- added: `mark_commit_cleanup_origin` and `is_commit_cleanup_os_error`;
- modified: `_drained_commit_source_cleanup_error`,
  `classify_route_os_error`, and `os_error_origin`;
- module addition: exactly
  `COMMIT_CLEANUP_ORIGIN_ATTRIBUTE = "codefire_commit_cleanup_origin"`.

There are no deleted qualified units, duplicate definitions, module removals,
or unrelated product changes. The original module order remains a subsequence
of the final order.

The bridge marks the first real cleanup `OSError` at the cleanup owner and has
the shared classifier refuse that origin, just as it refuses transport origin.
It does not use exception subclass, errno, message, filename, route, dispatcher,
or a guessed phase index as origin evidence. A generic unmarked route `OSError`
still follows typed filesystem classification.

The final document delta is equally bounded: architecture, trace, and playbook
gain only the three-origin `C18` truth and cleanup-owner/classifier-refusal
description; the catalog is unchanged.

Line-anchored conflict-marker counts are zero in `codefire` and all four source
documents at stage one, stage two, and final `C`.

## 6. Critical invariant review

### 6.1 Pre-bridge defect and bridge sufficiency

Against the reconstructed pre-bridge executable and candidate test module,
`test_request_loop_accounts_a_drained_cleanup_fault` fails on both
`ancestry` and `signature-history` with `0 != 1`. This reproduces the exact
accepted-input incompatibility: the cleanup fault is drained but the unmarked
`OSError` is incorrectly translated rather than escaping after accounting.

The final candidate passes the same live oracle on both routes. The two-part
repair is necessary: removing the owner mark (`X10`) or restoring the old
classifier with the mark retained (`X11`) independently recreates the same two
failures. The candidate therefore repairs the measured incompatibility, not an
adjacent hypothetical path.

### 6.2 `INV-B3` unchanged surface

AST source segments independently hash identically between accepted sources and
candidate:

| Surface | Accepted source | SHA-256 | Bytes | Result |
|---|---|---|---:|---|
| `_PreparedHTTPMutationResponses` | `0f43415` | `16a4501d35455d482e1b1e6bd4b16bd661b0c18cdb4afb86c7afcef5353cf25c` | 5769 | identical |
| `make_http_handler.CodeFireHTTPHandler.do_POST` | `b164046` | `ff31cb1ffc9888836553de36cab1b340b43f04f7038b49b1d581e85bae301e1a` | 11265 | identical |
| `make_http_handler.CodeFireHTTPHandler.handle_one_request` | `b164046` | `af8a6a832d3845a236e1f983107dc72b5dec886c8babc90e026b351593eaf007` | 7834 | identical |

The bridge changes one classifier decision before those surfaces. It does not
change the prepared-response contract, POST arm, request-loop accounting, or
wire publication window.

### 6.3 Precedence, exact error identity, and non-vacuity

Reviewer-owned production-only probes established:

- `X2`: a real socket POST with one real `close(2)` fault releases all current
  accounting, leaves permit count zero, records terminal completion, emits no
  response, creates no durable repository diff, leaves the branch unmoved, and
  escapes exactly the one cleanup `OSError`.
- `X3`: with eight faults armed, production traversal fires two faults at the
  faulting source, returns the **first** error by object identity, marks it as
  commit-cleanup origin, and closes three non-faulting sources exactly once.
- `X4`: real traversal followed by an operation failure preserves the exact
  operation object as the outer exception and the exact first cleanup error as
  its `__cause__`; only the cause carries cleanup origin and the outer decision
  is not reclassified.
- `X9`: the three new Upload live-route tests pass together on three consecutive
  runs (`Ran 3 tests`, `OK` each time).

The inversion census finds exactly one
`raise cleanup_error from run_error`, in `run_server_verification`, and zero in
`enforce_remote_commit_signature_policy_accounted`. The drain helper appears
exactly five times, preserving both accepted live call sites.

## 7. Test and checker gates

### 7.1 Python and demo

| Gate | Result | Provenance |
|---|---|---|
| `V1` compile of executable and five bound test modules | PASS | writer evidence audited; final bytes also parsed successfully during AST review |
| `V2` Upload module | 37/37, OK | writer evidence audited |
| `V3` MR-list module | 49/49, OK | writer evidence audited |
| `V4` Doctor + Branch | 32/32, OK | writer evidence audited |
| `V5` accepted Upload neighbour set | 59/59, OK | writer evidence audited |
| `V6` `test_python_*.py` | **539/539**, 34 modules, `530.828s`, OK | reviewer-executed; log SHA-256 `eb5cc42634071190de2a3068a86ff0d1e266a1599faca5ab4bd09657a9d797a7` |
| `V7` `tests.test_codefire_cli -v` | **228/228**, `1273.707s`, OK | reviewer-executed; log SHA-256 `08cd78bd403f83ae90e4f99d3c7ae612820a88180a5e4b263d3ffe58ee1dd32b` |
| `V8` full discovery | **767/767**, `1649.665s`, OK | reviewer-executed; same suite/count as the required verbose run; log SHA-256 `6d40941f2c775484fde2666615330b2a732fc09a13944ac5c021783376eeb78d` |
| exact verbose `V8` | 767/767, `1804.878s`, OK | manager evidence audited and bound in `W` to exact `C` and the same source-bound binary |
| `V9` import-time execution | PASS through `V6`/`V7`/`V8` | reviewer-executed |
| demo | PASS, `CodeFire demo completed` | reviewer-executed; log SHA-256 `27e757e2455a0435c00a651c47859be0aaf64317f70995135ce42072442a66ca` |

The review clone was at exact `C`, exact tree `531d07fe…`, and clean when these
logs were inspected. The expected arithmetic is independently consistent:
421 base Python tests + 19 Doctor + 13 Branch + 49 MR-list + 36 accepted Upload
= 538 pre-bridge; the single `C18` oracle makes 539; adding 228 CLI tests makes
767.

The full-suite log includes expected exception output from fault-injection tests
but terminates `Ran 767 tests` / `OK`; it is not a hidden suite failure.

### 7.2 Documentation and development gates

| Gate | Result | Evidence |
|---|---|---|
| source docs | PASS: 44 modules, 34 commands, 12 object kinds | reviewer-executed; SHA-256 `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` |
| project docs at `C` | PASS: 694 Markdown files | reviewer-executed; SHA-256 `15f79fbe0aa3bd2aed3849fb75a2c6aadc813a4056e6a44487686569c1aa2635` |
| AI contracts | PASS | reviewer-executed; SHA-256 `b04cb4dd38aa88e638a9d42aadfbf29798298711a3551218268736c3383d3e85` |
| AI acceleration docs | PASS | reviewer-executed; SHA-256 `c5acd9965199dc37c85c9e09d86d88a7371d733d2cc0ede29d531e78bd62a33d` |
| AI development gate, timeout 180 | PASS 7/7 including `tool_tests` | reviewer-executed; SHA-256 `4fbf60b5e5556dbc658be8648b5dd0749710f4e01d3c6993ae77f5852a461ec6` |
| Cargo/AIA negative space | `L..C` diff empty on the two governed Cargo gate paths | independently recomputed |
| authority citations | every changed `C14`/`B2` citation resolves in-tree | independently inspected |
| three-origin truth | architecture, trace, and playbook name transport, filesystem, and commit-cleanup without class/errno inference | independently inspected |

One earlier AI-development attempt used the known-too-short timing path and
failed only `tool_tests`; it is not evidence for the verdict. The required
timeout-180 rerun passed all seven checks and is the retained result.

## 8. Reversal matrix

The writer executed every frozen reversal on pristine snapshots and restored
byte identity after each. The terminal reviewer audited that complete matrix,
then independently re-falsified the new bridge, selected accepted-input
mechanisms, and the corrected branch expectation.

| ID | Required observation | Review result |
|---|---|---|
| `X1 r1` | remove bounded retry | independently reproduced: the three target tests fail on both routes, 6 failures total |
| `X1 r2`–`r4`, `r5b`, `r6a`, `r6b`, `m2` | accepted Upload mechanisms become observable when reversed | complete writer matrix audited; each failed the named oracle and pristine digest restoration was recorded |
| `X1 r5a` | restore ancestry precedence inversion | independently reproduced: C14 live test errors on `route='ancestry'` |
| `X2` | real socket/accounting checkpoint | PASS, independently reproduced |
| `X3` | production drain identity and bounded retry | PASS, independently reproduced |
| `X4` | operation decision remains outer; cleanup is cause | PASS, independently reproduced |
| `X5a` / `X5b` | publication proof and exact release remain load-bearing | writer reversals audited; named tests fail as required |
| `X5c` | classify by exception class | independently reproduced: 6 failures across request-list, request-merge, and Upload routes |
| `X6` | restore pre-fix branch-list `200`/zero-poll expectation | independently reproduced: exact test fails because live response is not `200 OK` |
| `X7` | inversion census | PASS: exactly one surviving inherited inversion |
| `X8` | drain-helper census | PASS: exactly 5 mentions |
| `X9` | repeat new live-route tests | PASS three consecutive times |
| `X10` | remove cleanup mark at owner | independently reproduced: both live routes fail `0 != 1` |
| `X11` | keep mark but restore old classifier | independently reproduced: both live routes fail `0 != 1` |
| `X12` | refuse generic unmarked `OSError` as though cleanup | independently reproduced: correct MR-list oracle yields 9 subtest failures, including `handle_http_upload`; filesystem classification remains load-bearing |
| `X13` | replace origin marker with errno check | independently reproduced: cross-input `C18` oracle fails |

The first attempted standalone `X4` probe was invalid because it raised the
operation error before real traversal created a cleanup owner. It was discarded
and is not used to support any claim. The corrected probe invoked real traversal
first and produced the identity result above. Likewise, an initial `X12`
invocation named the wrong unittest class and produced only a loader error; it
was discarded before the exact class was run. These were probe-construction
errors, not candidate observations.

## 9. Negative space and residual ownership

Independently verified unchanged across `L..C`:

- the AIA-046 Cargo/O1 implementation and governance surface;
- `docs/development/README.md`, `current-cycle-burn-down-plan.md`, `history.md`,
  `codefire-improvement-cycle.md`, and the recovery execution ledger;
- both exact preflights, the launch packet, and typed companion;
- the superseded Upload `REQUEST_CHANGES` report and stale audit artifacts;
- all package, install, Docker, network, Rust source, and agent-state surfaces.

Root row 6 remains `Implement in progress`. `C14-LIVE-A1`, `C14-LIVE-A2`, and
`C14-SUCCESSOR-OWNER` are uniquely registered OPEN. No candidate wording claims
that they are implemented or closed. The original shared-primitives verdict
remains `BLOCK`; its bytes are unchanged.

The following are reviewed residuals, not candidate-created findings:

| Residual | Impact screen | Reachability / blast radius | Actionability disposition |
|---|---|---|---|
| `C14-LIVE-A1` inherited pre-effect cleanup debt, including the surviving `run_server_verification` inversion | correctness and recoverability can be affected at its named owners; no security, data-format, API, performance, or compatibility change introduced by `C` | reachable only at the registered inherited owners; unchanged from accepted inputs | `out_of_scope_residual`; successor repair required, not a correction to this composition |
| `C14-LIVE-A2` post-effect/wire contract and durable-authority work | reliability, API behavior, and governance remain intentionally incomplete; no new candidate regression | bounded to prepared-response/handler/request-loop and contract-promotion loci | `out_of_scope_residual`; remains an MR-write/global-closure gate |
| `C14-SUCCESSOR-OWNER` deferred global loci | future correctness and operability work; candidate leaves behavior unchanged | production-unwired or explicitly deferred owners only | `out_of_scope_residual`; separately predecessor-gated |
| original `B1` MR-write base condition | integration sequencing only; no candidate product effect | MR-write base remains unchanged | `out_of_scope_residual`; still blocked |
| original `B2` record | candidate `C` measurably eliminates the product contradiction, but the authoritative record is not mutated by a report-only review | candidate-scoped evidence only | this ACCEPT may support later manager adjudication; it does not itself rewrite or globally close `B2` |
| `B3` | post-effect response/wire correctness remains open | named `INV-B3` surfaces are byte-identical, so no regression is introduced | `out_of_scope_residual`; continues to block MR-write |

For each row, evidence is available and the causal relation to `C` was checked:
the residual is inherited, preserved, or merely routed, not newly caused. Repair
inside this 18-path accepted-input composition would widen scope and invalidate
the frozen proof. None is an acceptance-blocking observation against `C`.

## 10. Actionability and verdict rationale

The review protocol requires findings to be screened across impact,
reachability, determinism, blast radius, recoverability, evidence quality,
candidate causality, and repair scope before asking for a correction.

No candidate-created observation survived that screen:

- functional correctness: the exact pre-bridge defect is fixed and both live
  routes pass;
- reliability/recoverability: bounded retry, ledger accounting, exact error
  identity, and decision precedence are preserved and reversal-proven;
- API/wire behavior: `INV-B3` surfaces are byte-identical and generic filesystem
  failures retain typed responses;
- compatibility/data: accepted inputs and duplicate omission are exact; no
  schema, package, or migration surface changes;
- performance/resource behavior: no new loop or unbounded retry; the owner has
  one bounded retry and full-suite behavior is stable;
- security/privacy: no auth, trust, secret, network-policy, or data-exposure
  surface is added;
- maintainability/governance: bridge scope is five qualified units plus one
  constant, documents resolve to existing authority, and residual owners remain
  explicit and OPEN;
- test quality: the new oracle distinguishes equal class/errno/message/filename
  inputs solely by production origin, and `X10`–`X13` prove all bridge limbs are
  load-bearing without weakening filesystem classification.

The two discarded probe attempts are classified
`invalid_or_unsubstantiated`: they have no candidate causality, no product
impact, and require no candidate repair. All inherited residuals are classified
`out_of_scope_residual`. There is no `required_correction` and no
`accepted_implementation_variance` in the decision subject.

## 11. Explicitly not run or changed

- No Cargo build, Cargo test, Cargo lint, Rust edit, install, packaging, Docker,
  network, or dogfood action was performed by this review lane.
- The source-bound Rust binary evidence recorded in `W` was audited by identity;
  it was not rebuilt here.
- No merge, cherry-pick, rebase, amend, reset, integration, or canonical
  fast-forward was performed.
- No root status, issue status, predecessor gate, or OPEN handle was changed.
- No candidate byte was edited to make a test pass.

## 12. Final decision statement

Candidate `cf1ac6a7d61d32cb8138c127e649151c5c545196` is **ACCEPTED as the exact
Python accepted-input composition candidate only**. Its accepted-input union is
exact, its duplicate is omitted once, its one prescribed incompatibility bridge
is bounded and independently reversal-proven, its integrated Python suite is
767/767, and no candidate-created required correction remains.

This is a terminal review decision for that immutable tree, not a project
closure event. CFR-168 and its registered successors remain under the existing
burn-down and predecessor gates.
