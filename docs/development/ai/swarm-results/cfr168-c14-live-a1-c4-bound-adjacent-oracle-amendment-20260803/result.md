# CFR-168 C14-LIVE-A1 C4-Bound Adjacent-Oracle W5 Result Evidence

Role: fresh result-only evidence writer (`validator`).

Task packet:
`docs/development/ai/task-packets/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803.md`.

Report mode: one-path, report-only W5 evidence. This report changes no product,
test, packet, canonical source document, PF5 report, issue state, or lifecycle
state. It does not accept or integrate C4/A1, authorize A2/B3, close CFR-168 or
CFB-138, or authorize Install, Dogfood, Evaluate, Close, or Retrospect.

## 1. Evidence Attribution and Closed Scope

| Evidence class | Attribution | Treatment |
|---|---|---|
| Original ordered O1 gates and original static gates | Immutable artifacts produced by the stopped original result session under `/workspace/.cache/cfr168-c14-a1-c4-adjacent-oracle-w5-result-20260803-evidence` | Independently read, re-hashed, and checked in this lane; not claimed as newly executed here |
| Paired C4/O1 full discovery | Manager-owned immutable artifacts under `/workspace/.cache/cfr168-c14-a1-c4-o1-paired-full-20260803-evidence` | Independently read, re-hashed, raw-log inspected, and compared with the exact read-only comparator in this lane |
| Fresh AO-01/AO-02 reversals | Executed by this W5 lane in two separate non-Git exports materialized from exact O1 | Writer-executed; these are the only fresh product-test commands run by this lane |
| Post-report docs/AI checks | Executed by this W5 lane after this report was first written | Writer-executed; recorded in section 11 |

No 2-test, 81-test, 184-test, compile, 558-test, or full-discovery product gate
was rerun. No Cargo, Rust, demo, install, packaging mutation, environment repair,
or product mutation was run. C4 and O1 were never used as mutation worktrees.

## 2. Immutable Subject and Topology

### 2.1 Full committed identities

| Node | Commit | Tree | Exact parent | Subject |
|---|---|---|---|---|
| C4 | `19010f742d358c5c03e19d091c4249420f9e53b9` | `a0ec4ee256f63c7fe7de438eb5c978c918b3d22f` | `8ff33fd9b354a169380b9401f342b96cc6785ad7` | `fix: release reused cleanup tracebacks` |
| F5 | `f29019ad97308f8ac58e47ce57516287594b12bc` | `1c0ef6ec64fe7773f2f2e7f123eacb12b03df6c5` | exact C4 | `docs(cfr168): freeze C4-bound adjacent oracle amendment` |
| PF5 | `4e3cf33a3cd88f7ba71983bc1a4046af43e136e4` | `6adb6c3183e9753ed7c39b4e959f4fc3110fdcc5` | exact F5 | `docs(cfr168): record C4-bound adjacent oracle contract preflight` |
| O1 | `46d98466c702f111b58a3c1ea080518f75a59580` | `626b28d1989ef21bbc23f94288845be5b0082c90` | exact F5 | `test(cfr168): correct adjacent primary-error oracles` |
| W5 | this report-only commit | fixed by the commit containing this file | required parent exact O1 | one added report path only |

W5 cannot contain its own commit or tree hash without a cryptographic
self-reference. It therefore binds its exact required parent and sole path
here; the final handoff binds the created W5 commit/tree, and fresh R5 must
independently recompute that identity. W5 does not claim that its own identity
was part of O1.

### 2.2 Recomputed topology

```text
C4..F5 commit count = 1; merges = 0
F5..O1 commit count = 1; merges = 0
PF5^ = F5
O1^  = F5
merge-base(PF5, O1) = F5
PF5 is not an ancestor of O1
F5 is an ancestor of both PF5 and O1
```

PF5 is a report-only sibling and is absent from O1/W5 ancestry. No merge,
cherry-pick, squash, or copied PF5 report byte appears in O1.

## 3. F5 and PF5 Bindings

### 3.1 F5 packet

| Item | Exact value |
|---|---|
| `C4..F5` name-status | exactly `A docs/development/ai/task-packets/cfr168-c14-live-a1-c4-bound-adjacent-oracle-amendment-20260803.md` |
| Numstat | `664` insertions, `0` deletions |
| Mode / blob | `100644` / `17f50f6b4f9e9c30b2a7f9d108c8aa58d9ce2dd2` |
| SHA-256 | `1fcdaccbbda7936a973e6f9fe4e58b1ed4bbe6f78c5c270e16d5d8da0541d012` |
| Commit count / merges | `1` / `0` |

F5 changes no product or test byte and preserves the O1 target preimage.

### 3.2 PF5 report and provenance

| Item | Exact value |
|---|---|
| Report path | `docs/development/ai/swarm-results/cfr168-c14-live-a1-c4-bound-adjacent-oracle-preflight-20260803/preflight.md` |
| Mode / blob | `100644` / `aa904431f717868f4cbf489336c9386283bf5189` |
| SHA-256 | `a630edc3e11592804636449fb51afdf508be0649140728452e2bb814b8ed635b` |
| `F5..PF5` name-status | exactly one added report path above |
| Provider/runtime | Anthropic Claude Code `claude-code_2-1-220_agent`, entrypoint `sdk-cli` |
| Model / effort | `claude-opus-5` / `high` |
| Session | `efe22d7d-46bb-4379-b3a2-e8644b599535`, fresh child session |
| Batch / task | `cfr168-c14-a1-c4-adjacent-oracle-preflight-20260803` / `c14-a1-c4-adjacent-oracle-preflight` |
| Exact terminal line | `PREFLIGHT_VERDICT: ACCEPT_CONTRACT` |

PF5 authorizes O1 only. It makes no acceptance, integration, or lifecycle
decision.

## 4. C4 and O1 Byte Identity

### 4.1 C4 two-path implementation candidate

`git diff --name-status 8ff33fd9b354a169380b9401f342b96cc6785ad7..C4`
is exactly:

```text
M	codefire
M	tests/test_python_http_upload_accounted_traversal.py
```

| C4 path | Mode | Git blob | Content SHA-256 |
|---|---:|---|---|
| `codefire` | `100755` | `15c7ebf5e2c53216c1aaa1add12070cffc12cbdd` | `dc40d2c02c343c076204ce680018c33871c43b19cccf45d274aa0ac960af1124` |
| `tests/test_python_http_upload_accounted_traversal.py` | `100644` | `c609e2a84fbcd5f3dca3791cfa106abca369a374` | `4905784910368bfa9cd16d51eaf8a7e1d454d2bee52f6b9bbe3c27b0c96be054` |

The ordinary `git diff --binary F4 C4 | sha256sum` binding is
`ae001ae07eb9e3a7be5eb9a3e7c38474abb762d4ae29e9bd1d820c3cf7bb09f2`.

### 4.2 O1 exact one-path, two-token change

| Item | Exact value |
|---|---|
| `F5..O1` name-status | exactly `M tests/test_python_existing_terminal_owner.py` |
| Numstat | exactly `2` insertions and `2` deletions |
| Commit count / merges | `1` / `0` |
| Before mode / blob | `100644` / `cc51e5ebcc4f432f19a5b235c92b6bcae3b69215` |
| Before SHA-256 | `0ccbe0a9798c795ae4c5414ab3fb3b193d6558305450f40f919e115c0b6069ff` |
| After mode / blob | `100644` / `e5763235f9c146d09aa57237c11bf84b78662f0c` |
| After SHA-256 | `9c8bf1154ebc567ed7d96f1319b2a4bef26016d6fa2c25f9bd201c288258b4e4` |
| Binary patch SHA-256 | `9a1f8b9d5bdebe7d8819ab3652ba02cf94db8c339428d091d301943e778e0de2` |

The complete zero-context semantic diff is:

```diff
-                KeyboardInterrupt, "partial verification close"
+                KeyboardInterrupt, "partial verification construction"
-                KeyboardInterrupt, "partial ancestry close"
+                KeyboardInterrupt, "partial ancestry construction"
```

`git diff --exit-code C4..O1 -- codefire` is empty. `F5..O1` changes no path
other than the named test file, and the zero-context diff changes no line other
than the two expected literals. Therefore all production bytes, both injected
construction failures, both injected cleanup-close failures, both cleanup
callbacks, callback installation/restoration, publication, retry, close-call,
closed-child, cleared-owner, abort-only, zero-ledger, and post-auto-close
assertions are byte-identical.

### 4.3 Impact-axis negative space

| Axis | O1 result | Proof |
|---|---|---|
| Product correctness and cleanup precedence | unchanged | `codefire` byte-identical C4→O1; only test expected literals changed |
| State and cleanup retry | unchanged | retry callbacks and every state/close assertion are outside both diff hunks |
| Object identity/evidence chain | unchanged | no product, helper, owner, or identity byte changed |
| Accounting | unchanged | zero-ledger assertions and accounting implementation unchanged |
| Authority/security | unchanged | no capability, validation, policy, or authority path changed |
| API/wire | unchanged | no command, response, status, payload, or serialization path changed |
| Persisted data | unchanged | no object, schema, transaction, receipt, or repository layout path changed |
| Test-oracle correctness | corrected | exact stale cleanup-first regexes now name the construction primary |

## 5. Immutable Evidence Bundles

### 5.1 Original ordered O1 evidence

Root:
`/workspace/.cache/cfr168-c14-a1-c4-adjacent-oracle-w5-result-20260803-evidence`.

| Artifact | SHA-256 / fact |
|---|---|
| `run-ordered-gates.sh` | `cfa7ed0a4a396d084f7a9473037dd82cbe51a5348414f3497e68be52aa5930f1` |
| `run-static-gates.sh` | `be1262de8b179f94749bd9643781c973ca9eb834474ce915c49158cb9f0056cf` |
| Pre-run identity record | SHA-256 `23124a055fba75db67edfca636573d6dd94bfba3f97d820a77d4c31d5d854f9e`; exact O1/tree/parent, uid `1000`, Python `3.10.12`, zero status lines |
| Ordered sequence | start `2026-08-03T03:48:41Z`; terminal marker identifies failed gate `06_stable_full_discovery`; finish `2026-08-03T04:13:27Z` |
| Static sequence | start `2026-08-03T04:14:17Z`; exit `0`; finish `2026-08-03T04:16:16Z` |

Every `.command`, `.meta`, and `.log` file in this bundle was read. Every
recorded log digest equals a fresh SHA-256 of the complete log bytes.

### 5.2 Manager-owned paired full-discovery evidence

Root:
`/workspace/.cache/cfr168-c14-a1-c4-o1-paired-full-20260803-evidence`.

The required `paired.finished`, `c4.meta`, and `o1.meta` files exist.

| Artifact | SHA-256 |
|---|---|
| Immutable runner `run-paired-full.sh` | `548e433203bd7ae1894e2cdf2f65d92796959163ecdfadfeba494156c59297d2` |
| Read-only comparator `compare_paired_full.py` | `23e0ea0a9ccc8eb29779cb0e0277ad11fd3ebc30d88bd69542daa1d5b4365065` |
| Environment record | `909b25d97aae226bdb573952f374558f1d7393788c0ce8100c2114eb994a54bb` |
| C4 log | `3742cf97c9205267d091405aef7a9b08481bfe7a3c33cef87c203365a2593338` |
| O1 log | `3ac510bdc32ca4955bba14e8b02002812765e3a1684e63bf3f5cfd668dee452d` |
| Comparator output captured by W5 | `2eb2012c2f512090767dc1d388da1fb330d96a544489e014a67f01afad8684d0` |

Every command/script, metadata file, sentinel, environment record, and raw log
in this bundle was independently inspected. The comparator is an aid; the raw
logs and metadata were separately checked.

## 6. Original Ordered O1 Gate Evidence

All rows bind revision `46d98466c702f111b58a3c1ea080518f75a59580`.
Counts are selected/passed/failed/errors/skipped.

### 6.1 Exact command registry

```text
G01 timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_verification_child_is_published_and_retried tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_ancestry_construction_and_auto_close_are_reconciled
G02 timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v tests.test_python_signature_terminal_freeze tests.test_python_sealed_signature_composite tests.test_python_existing_terminal_owner tests.test_python_existing_consumer_terminal_freeze tests.test_python_sealed_owner tests.test_python_sealed_borrow
G03 timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v tests.test_python_http_upload_accounted_traversal tests.test_python_ancestry_owner tests.test_python_ancestry_borrow_handoff tests.test_python_signature_authority tests.test_python_signature_owner tests.test_python_external_verification tests.test_python_external_production tests.test_python_signature_verification_handoff tests.test_python_sealed_production
G04 timeout 900s env PYTHONDONTWRITEBYTECODE=1 PYTHONPYCACHEPREFIX=/workspace/.cache/cfr168-c14-a1-c4-adjacent-oracle-w5-result-20260803-evidence/pycache /usr/bin/python3.10 -B -m py_compile codefire tests/test_codefire_cli.py tests/test_python_http_upload_accounted_traversal.py tests/test_python_ancestry_owner.py tests/test_python_ancestry_borrow_handoff.py tests/test_python_signature_authority.py tests/test_python_signature_owner.py tests/test_python_external_verification.py tests/test_python_external_production.py tests/test_python_signature_verification_handoff.py tests/test_python_sealed_production.py tests/test_python_existing_terminal_owner.py
G05 timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest discover -s tests -p 'test_python_*.py' -v
G06 timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest discover -s tests -v
G07a timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_source_docs.py
G07b timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_project_docs.py
G07c timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_contracts.py
G07d timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_acceleration_docs.py
G07e timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_development.py --timeout 600 --skip-cli
G07f timeout 900s git diff --check 19010f742d358c5c03e19d091c4249420f9e53b9..46d98466c702f111b58a3c1ea080518f75a59580
G07g timeout 900s git status --porcelain
```

### 6.2 Results

| ID | UTC start → end | Duration ms | Exit | Counts/result | Complete log SHA-256 | Evidence state |
|---|---|---:|---:|---|---|---|
| G01 | `2026-08-03T03:48:41Z` → `03:48:42Z` | 1,451 | 0 | `2/2/0/0/0`; `Ran 2`; `OK` | `ad24c6186961aa75f64fd051d97479786c1af4347e2586c64a15e0d4e1741e63` | PASS |
| G02 | `2026-08-03T03:48:42Z` → `03:49:26Z` | 43,495 | 0 | `81/81/0/0/0`; `Ran 81`; `OK` | `e59ecc2409416fff6eef4273be72ce967b377992a9298f381dc1237b5b251ae6` | PASS |
| G03 | `2026-08-03T03:49:26Z` → `03:51:29Z` | 123,275 | 0 | `184/184/0/0/0`; `Ran 184`; `OK` | `4d6e195b4eefb6d48b7df18e3f1e9ee87da7d7df72f3594eb8ad9a05a11d0a07` | PASS |
| G04 | `2026-08-03T03:51:29Z` → `03:51:30Z` | 811 | 0 | exact 12 paths compiled; empty log; bytecode redirected outside repository | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | PASS |
| G05 | `2026-08-03T03:51:30Z` → `03:58:27Z` | 416,620 | 0 | `558/558/0/0/0`; `Ran 558`; `OK` | `f6948b6db5bafd1f77ae49f8943c6f5dbe2814efb9699059a8d97858cdd04b7b` | PASS |
| G06 | `2026-08-03T03:58:27Z` → `04:13:27Z` | 900,028 | 124 | incomplete: 199 verbose status lines; no `Ran 786`, no terminal summary, no complete failure blocks | `babcd85d79fb43db6b943c70e42cf7972805ca72d618fc524675bc1aba87a897` | TIMEOUT, NOT PASS |
| G07a | `2026-08-03T04:14:17Z` → `04:14:18Z` | 373 | 0 | source docs passed: 44 modules, 34 commands, 12 object kinds | `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` | PASS |
| G07b | `2026-08-03T04:14:18Z` → `04:14:19Z` | 1,539 | 0 | project docs passed; 708 Markdown files inventoried | `3ad2517901ffdf27ec185440d9d7141fdf59fcffe7ace1d2181369ee6babeb79` | PASS |
| G07c | `2026-08-03T04:14:19Z` → `04:14:19Z` | 396 | 0 | AI contracts passed | `b04cb4dd38aa88e638a9d42aadfbf29798298711a3551218268736c3383d3e85` | PASS |
| G07d | `2026-08-03T04:14:20Z` → `04:14:20Z` | 511 | 0 | AI acceleration docs passed | `c5acd9965199dc37c85c9e09d86d88a7371d733d2cc0ede29d531e78bd62a33d` | PASS |
| G07e | `2026-08-03T04:14:20Z` → `04:16:16Z` | 116,012 | 0 | all seven aggregate components passed | `4fbf60b5e5556dbc658be8648b5dd0749710f4e01d3c6993ae77f5852a461ec6` | PASS |
| G07f | `2026-08-03T04:16:16Z` → `04:16:16Z` | 16 | 0 | empty diff-check log | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | PASS |
| G07g | `2026-08-03T04:16:16Z` → `04:16:16Z` | 27 | 0 | empty porcelain log | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | PASS |

G06 is retained visibly as the honest initial full-discovery timeout. It is not
converted into a pass by later evidence and is not evidence of 786/786.

## 7. Paired Complete Full Discovery

### 7.1 Exact environment and command

Both runs started at `2026-08-03T04:16:23Z` under identical:

| Field | Exact value |
|---|---|
| uid / gid / umask | `1000` / `1000` / `0077` |
| `HOME` | `/home/devuser` |
| locale | `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`, `LC_CTYPE=C.UTF-8` |
| `TMPDIR` | unset/empty |
| Python | `/usr/bin/python3.10`, `Python 3.10.12`, `-B` |
| Cargo / rustc | absent from `PATH` |
| timeout | `3600s` per subject |
| unittest command | `timeout 3600s /usr/bin/python3.10 -B -m unittest discover -s tests -v` |

The recorded `PATH` is exactly:

```text
/opt/devcontainer-ai-cli/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path:/home/devuser/.codex/tmp/arg0/codex-arg0XJ2XZv:/opt/devcontainer-ai-cli/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path:/opt/devcontainer-ai-cli/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path:/home/devuser/.codex/tmp/arg0/codex-arg0gjCRP1:/opt/devcontainer-ai-cli/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path:/home/devuser/.vscode-server/data/User/globalStorage/github.copilot-chat/debugCommand:/home/devuser/.vscode-server/data/User/globalStorage/github.copilot-chat/copilotCli:/vscode/vscode-server/bin/linux-x64/e4c7e7b1d6d060162f4aa7f8225271b67ce1df75/bin/remote-cli:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
```

### 7.2 Subject results

| Subject | Revision / tree / parent | UTC start → end | Duration ms | Exit | Complete terminal | Log SHA-256 |
|---|---|---|---:|---:|---|---|
| C4 | `19010f74…` / `a0ec4ee2…` / exact F4 | `2026-08-03T04:16:23Z` → `04:50:44Z` | 2,061,035 | 1 | `Ran 786 tests in 2052.655s`; `FAILED (failures=4)` | `3742cf97c9205267d091405aef7a9b08481bfe7a3c33cef87c203365a2593338` |
| O1 | `46d98466…` / `626b28d1…` / exact F5 | `2026-08-03T04:16:23Z` → `04:50:48Z` | 2,065,274 | 1 | `Ran 786 tests in 2056.961s`; `FAILED (failures=2)` | `3ac510bdc32ca4955bba14e8b02002812765e3a1684e63bf3f5cfd668dee452d` |

Both logs contain 786 verbose method IDs and 786 unique method IDs. Their
ordered method lists are byte-equivalent after parsing, with ordered method
SHA-256 `e1ca2f1c6818bb82dc6312c2923ad8d34585e61edc667dcbb6a801e9f362928c`.
The recorded log digests match metadata. Both exact worktrees were clean before
the runner began and in `paired.finished` after it ended.

### 7.3 Exact failure comparison

The only status transitions are:

| Method | C4 | O1 |
|---|---|---|
| `test_partial_ancestry_construction_and_auto_close_are_reconciled (test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests)` | `FAIL` | `ok` |
| `test_partial_verification_child_is_published_and_retried (test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests)` | `FAIL` | `ok` |

C4-only normalized failure fingerprints:

| Method | Exact failure | Normalized block SHA-256 |
|---|---|---|
| partial ancestry | `KeyboardInterrupt: partial ancestry construction`; `AssertionError: "partial ancestry close" does not match "partial ancestry construction"` | `b07f50d9930e9731e94446bc60495da907caca8646924470f5d6f15446a7bbca` |
| partial verification | `KeyboardInterrupt: partial verification construction`; `AssertionError: "partial verification close" does not match "partial verification construction"` | `0c713757065c8f60ce6dd7e1f401321d29f9e1d134dde6fda6e0522c72807ca9` |

Common C4/O1 environmental failures are byte-equivalent after replacing the
subject root with `<SUBJECT_ROOT>`:

| Failure ID | Exact normalized signature | Normalized block SHA-256 |
|---|---|---|
| `test_completion_command_and_install_completion` | assertion return code `2`; `error: cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt binary` | `8ae8b13263a1c0a653997514367bb778c74ecf171a7a303db7d5ccbbe0bdd32d` |
| `test_install_script_installs_codefire` | assertion return code `2`; same absent-Cargo diagnostic | `ca447215409287d0ecb9277a545c2f38e60c96dd606d893b8abe9eb98bb04d6b` |

There is no other status or failure difference.

### 7.4 Exact comparator execution

Executed in this W5 lane exactly as required:

```text
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B /workspace/.cache/cfr168-c14-a1-c4-o1-paired-full-20260803-evidence/compare_paired_full.py
```

Exit was `0`; `errors` was empty; all four comparisons were `true`; final
disposition was exactly:

```text
MATCH_ENVIRONMENTAL_UNAVAILABLE
```

This is complete, same-environment comparative evidence that O1 removes only
the two stale-literal failures. Because both runs retain the two identical
installer failures, full discovery remains environmental/unavailable and is
never represented as 786/786 PASS.

## 8. Fresh One-at-a-Time Throwaway Reversals

### 8.1 Export identity

A single archive was created outside every Git worktree with
`git archive --format=tar 46d98466c702f111b58a3c1ea080518f75a59580`.

| Item | Exact value |
|---|---|
| Archive commit ID | exact O1 `46d98466c702f111b58a3c1ea080518f75a59580` |
| O1 tree | `626b28d1989ef21bbc23f94288845be5b0082c90` |
| Archive SHA-256 / bytes | `f1ec9c30c636776f924937c1b8384ff3e18af83032416a2b7e43bf7ceefba8be` / `21,688,320` |
| Exports | separate `ao01/export` and `ao02/export` under `/workspace/.cache/cfr168-c14-a1-c4-adjacent-oracle-w5-result-retry1-20260803-reversals` |
| Git metadata | absent in both exports |
| Initial target blob / SHA-256 | `e5763235f9c146d09aa57237c11bf84b78662f0c` / `9c8bf1154ebc567ed7d96f1319b2a4bef26016d6fa2c25f9bd201c288258b4e4` in each export |

### 8.2 AO-01 verification reversal

`apply_patch` changed only the expected literal
`partial verification construction` back to stale
`partial verification close` in the AO-01 export.

| Item | Exact value |
|---|---|
| Mutated target blob / SHA-256 | `dfa804a6cea3f4ad0ea456e543d85fe509b0672e` / `4ae27cc37d623a83ac101367d8b532adcaa28310ff8768fa8d7e5de296e50bc9` |
| Exact command | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_verification_child_is_published_and_retried` |
| UTC start / end | `2026-08-03T05:10:13Z` / `2026-08-03T05:10:14Z` |
| Duration / exit | `1,109 ms` / `1` |
| Counts | selected `1`, passed `0`, failed `1`, errors `0`, skipped `0` |
| Terminal | `FAILED (failures=1)` |
| Complete log SHA-256 / bytes | `6bd47fefa5fe0577e7476b6e66239ad6e006e9eca4b0cb072610be4f58e76884` / `1,051` |
| Exact signature | `KeyboardInterrupt: partial verification construction`; `AssertionError: "partial verification close" does not match "partial verification construction"` |
| Failure block SHA-256 / normalized SHA-256 | `d6b45f3aa568e7fe121c6b31466544d832e2aba443797084e857152eacd0bd9d` / `95f45ac659c0345297acffb430e4bf7b087f56d4fd30c760917f067aee65639a` |

There was no other failure or error. The mutated manifest was byte-identical
before and after the test, proving the test command made no source change.
`apply_patch` then restored the corrected literal. The final target blob/SHA-256
returned to exact O1, and the complete final export manifest equaled its initial
manifest (`fd00bd4a5bd1c1cd330cc6d07a18a86cb08405b2f4d88b6e3ea109594bca706c`).

### 8.3 AO-02 ancestry reversal

`apply_patch` changed only the expected literal
`partial ancestry construction` back to stale `partial ancestry close` in the
separate AO-02 export.

| Item | Exact value |
|---|---|
| Mutated target blob / SHA-256 | `a546396e9c00164cdd94d859c5b6f54c5163145c` / `0c27ca41680de8381205f4bbfa4c2f1b8aef675d86b8f330842ec8b1d3d46d7b` |
| Exact command | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B -m unittest -v tests.test_python_existing_terminal_owner.PythonExistingTerminalOwnerTests.test_partial_ancestry_construction_and_auto_close_are_reconciled` |
| UTC start / end | `2026-08-03T05:10:47Z` / `2026-08-03T05:10:48Z` |
| Duration / exit | `1,612 ms` / `1` |
| Counts | selected `1`, passed `0`, failed `1`, errors `0`, skipped `0` |
| Terminal | `FAILED (failures=1)` |
| Complete log SHA-256 / bytes | `ae9b907abdfac4547d9864d865b214094c3503d3448e9f5caf79744130dc1412` / `1,063` |
| Exact signature | `KeyboardInterrupt: partial ancestry construction`; `AssertionError: "partial ancestry close" does not match "partial ancestry construction"` |
| Failure block SHA-256 / normalized SHA-256 | `38f37167247bee951489e6633670b2180f6011b92a374ad5e556d11a519fb93f` / `59ef1f4afb0c1adca0de226a65bc8409c648651e3e598406feeb9ad0638226ba` |

There was no other failure or error. The mutated manifest was byte-identical
before and after the test. `apply_patch` restored the corrected literal; the
target blob/SHA-256 returned to exact O1, and the complete final manifest again
equaled the initial manifest
(`fd00bd4a5bd1c1cd330cc6d07a18a86cb08405b2f4d88b6e3ea109594bca706c`).

Both negative controls are killed one at a time for the exact stale-regex
reason. Neither reversal touched or restored C4 or O1.

## 9. Worktree Cleanliness and C4 Procedure Observation

### 9.1 Clean frozen worktrees

The paired runner required empty C4/O1 porcelain before starting and recorded
zero-byte porcelain for both after completion. After both fresh reversals, W5
performed a separate read-only identity/status check:

| Item | Exact value |
|---|---|
| UTC / duration / exit | `2026-08-03T05:19:23Z` / `72 ms` / `0` |
| C4 status bytes | `0`; exact C4/tree/parent unchanged |
| C4 target blob / SHA-256 | `cc51e5ebcc4f432f19a5b235c92b6bcae3b69215` / `0ccbe0a9798c795ae4c5414ab3fb3b193d6558305450f40f919e115c0b6069ff` |
| O1 status bytes | `0`; exact O1/tree/parent unchanged |
| O1 target blob / SHA-256 | `e5763235f9c146d09aa57237c11bf84b78662f0c` / `9c8bf1154ebc567ed7d96f1319b2a4bef26016d6fa2c25f9bd201c288258b4e4` |
| Complete cleanliness log SHA-256 | `2971cdf63353541671a096c051e78db27c4f3c2359694a8d2ebf3e200931a31d` |

### 9.2 C4 implementation-procedure observation

C4's author temporarily mutated its candidate worktree when attempting
reversal evidence, contrary to the F4 throwaway-only rule. Those mutation logs
remain inadmissible. The frozen packet records that exact C4 ran the nine-module
gate 184/184 before mutation and a combined focused/inherited 10/10 after
restoration; those restoration facts do not launder the mutation logs.

Committed C4 is not contradicted: its exact two blobs, cumulative patch digest,
and clean status match their frozen bindings. No C5, restoration commit, product
repair, or extra test edit is authorized. Final reversal evidence in this W5 is
instead the two new real-export controls in section 8.

## 10. Discrepancy and Finding Classification

`affected impact` names the affected evidence/product axis, not severity alone.
Only a current `required_correction` would receive repair authority.

| ID | Observation | Affected impact | Evidence state | Must-fix | Actionability/disposition | Causal class |
|---|---|---|---|---|---|---|
| W5-O1 | C4 retained two stale cleanup-first expected regexes | test-oracle correctness affected at C4; product behavior unaffected | available: paired C4 log and two exact failure blocks | historical correction required; closed by exact O1 | frozen two-token oracle correction completed; no open repair | null for current O1 subject; pre-existing A1-era stale oracle, not C4 product causation |
| W5-O2 | Original O1 full discovery hit exact 900-second timeout with no terminal summary | evidence completeness/development flow affected; product conclusion unknown from this run | available and complete as timeout evidence | no candidate byte fix | environmental/unavailable; retained as NOT PASS | null |
| W5-O3 | Complete paired full discovery has two common absent-Cargo installer failures on C4 and O1 | installation capability/release evidence affected in this environment; O1 comparative correctness unaffected | available: byte-equivalent normalized failure blocks | no O1 must-fix | out of O1 scope; `MATCH_ENVIRONMENTAL_UNAVAILABLE` | null |
| W5-O4 | C4 author mutated the candidate worktree while producing reversal evidence | evidence procedure and development flow affected; committed product bytes unaffected | available from frozen F5/PF5 observation plus exact committed-byte proof | evidence-package correction required; closed by section 8 | historical evidence-only required correction completed; old logs remain inadmissible | `implementation_nonconformance` |
| PF5-F1 | 558/0-skipped claim has a latent root-uid precondition | operations/test-environment robustness; actual uid 1000 run unaffected | available; actual 558/558/0 skipped | no | nonblocking improvement only | null |
| PF5-F2 | Full discovery command relies on default `test*.py` while count prose says `test_*.py` | specification precision; current 35-module sets identical | available static/runtime evidence | no | nonblocking improvement only | null |
| PF5-F3 | AO-02 wording says “first outer” although only one `partial ancestry close` outer literal exists | documentation clarity only | available source occurrence proof | no | nonblocking improvement only | null |
| PF5-F4 | “Adjacent mismatch” might be misread as C4-induced | no actual defect: C4's traceback-only line cannot change messages | available source/diff proof | no | invalid as a C4-causation claim; retain as clarification | null |
| PF5-F5 | O1 makes later assertions reachable for the first time | preflight risk to state/retry/accounting assertions | available: G01 2/2 plus G02/G03 broad focused passes | no remaining must-fix | risk falsified by runtime evidence | null |

No new product, authority, state, accounting, identity, API, wire, persisted,
or cleanup-retry discrepancy was found. There is no open W5-authorized repair.

## 11. Post-Report Authoring Validation

The following commands were run after the initial report bytes existed, with
ordinary Python 3.10 and outer 900-second bounds. The results below are
authoring evidence; a final no-edit rerun is recorded in the commit handoff.

| ID | Exact command | UTC start → end | Duration ms | Exit | Complete log SHA-256 | Result |
|---|---|---|---:|---:|---|---|
| P01 | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_source_docs.py` | `2026-08-03T05:25:39Z` → `2026-08-03T05:25:40Z` | `270` | `0` | `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` | passed: 44 modules, 34 commands, 12 object kinds |
| P02 | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_project_docs.py` | `2026-08-03T05:25:40Z` → `2026-08-03T05:25:41Z` | `1,354` | `0` | `2d844cad1be6599d66b2a880d284e94bbd5ea2041e5920f301a34ffb65c2893f` | passed: 709 Markdown files inventoried |
| P03 | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_contracts.py` | `2026-08-03T05:25:41Z` → `2026-08-03T05:25:41Z` | `417` | `0` | `b04cb4dd38aa88e638a9d42aadfbf29798298711a3551218268736c3383d3e85` | AI contract check passed |
| P04 | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_acceleration_docs.py` | `2026-08-03T05:25:41Z` → `2026-08-03T05:25:42Z` | `532` | `0` | `c5acd9965199dc37c85c9e09d86d88a7371d733d2cc0ede29d531e78bd62a33d` | AI acceleration docs check passed |
| P05 | `timeout 900s env PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3.10 -B docs/development/tools/check_ai_development.py --timeout 600 --skip-cli` | `2026-08-03T05:25:42Z` → `2026-08-03T05:27:26Z` | `104,171` | `0` | `4fbf60b5e5556dbc658be8648b5dd0749710f4e01d3c6993ae77f5852a461ec6` | all seven aggregate components passed |
| P06 | `timeout 900s git diff --check` | `2026-08-03T05:27:26Z` → `2026-08-03T05:27:26Z` | `15` | `0` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | clean; empty log |
| P07 | `timeout 900s git status --porcelain` | `2026-08-03T05:27:26Z` → `2026-08-03T05:27:26Z` | `21` | `0` | `dcdd1b9f50618ae46df86b0489042027072cb424adde71a4b8c8cc0692b19e2f` | only the authorized untracked report directory |

## 12. Remaining Risks and Authority Boundary

- Full repository discovery is environmental/unavailable because Cargo/rustc
  are absent; neither the initial timeout nor the paired failures are a full
  pass.
- The two installer failures remain outside this test-only amendment and
  require a capable release/install environment for independent release
  evidence.
- W5 is result evidence, not exact review. Fresh independent Claude Opus 5 R5
  remains mandatory and must bind the final W5 commit/tree/path, independently
  recompute C4/F5/PF5/O1, and review the claimed evidence.
- Integration, source acceptance, A2/B3 authorization, issue closure, and all
  later lifecycle decisions remain manager-owned.

RESULT_DISPOSITION: READY_FOR_R5
