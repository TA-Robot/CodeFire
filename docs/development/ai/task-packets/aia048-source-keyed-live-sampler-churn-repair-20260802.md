# AIA-048 Source-keyed Cargo Live-Sampler Churn Repair

Status: Frozen for implementation
Packet ID: `aia048-source-keyed-live-sampler-churn-repair-20260802`
Base revision: `54028a136943a9d5f9608e19d3855c0edae46adf`
Base tree: `e4057e1b6f16b12fed45ad0b50bc4c6014ca7ebd`
Risk: High — required correction in the Rust-validation control plane
Lifecycle: newly observed failure -> Freeze / Plan / Preflight

## Objective

Make AIA-046's one-second live target sampler tolerate ordinary file and
directory disappearance caused by the gate-owned Cargo process, without
weakening the strict final retention measurement, target containment, identity
checks, allocation limits, aggregate budget, or filesystem free-space floor.

## Observed failure and admission

The manager ran the source-keyed focused File Ancestry gate on the clean exact
candidate below:

| Field | Value |
|---|---|
| candidate | `44b66efbc1776d2299b8c9f8c53f95a980e6aa5e` |
| tree | `68a93e3b8e6cd123d272603975920361ce506db8` |
| argv | `cargo test -p codefire-cli --bin codefire-rs remote::file_remote_ancestry_tests --locked` |
| result | `/workspace/.cache/codefire-gate-evidence/cfr168-file-ancestry-r1e-focused-44b66ef-20260802.json` |
| result SHA-256 | `5a921cc299b302dd3d67e74625a7c43af1ae215fab784bd14f355978ed0e37f6` |
| log | sibling `.log` |
| log SHA-256 | `b017a5319086a94dc35032b19393ccafeb3d27368de7a0ed0d34b6d34355d55b` |
| gate result | `BLOCK`, process exit `126` |
| primary blocker | `runtime-resource-guard-not-passed` / `live-target-usage-unavailable` |

The build completed and the standard libtest harness printed `running 40
tests`. Multiple tests passed before the supervisor terminated the process.
The log contains no test failure. The runtime guard failed because
`namespace_usage_at` performs `scandir` followed by path-based `stat` and
descriptor reopen while Cargo concurrently creates, renames, and removes
entries in its own target namespace. A vanished entry raises
`FileNotFoundError`; two immediate whole-scan retries can both observe ordinary
churn and then convert it into `live-target-usage-unavailable`.

This is a reachable validation-infrastructure failure on a compliant actor:
the writer is the gate-owned Cargo command under the global execution lock, not
an outside or hostile process. It blocks acceptance-required Rust evidence and
therefore meets `MF-4` and `MF-5`. Disposition:
`required_correction`; causal class: `implementation_nonconformance` because
the live-sampler contract predates this successor.

The secondary zero-test, retention, and run-state blockers are consequences of
the supervisor kill. They are not separate product defects.

## Scope

Implementation candidate may change exactly these three paths:

- `docs/development/tools/run_source_keyed_cargo_gate.py`
- `docs/development/tools/test_ai_tools.py`
- `docs/development/ai/source-keyed-cargo-gates.md`

The implementer result is a separate report-only commit adding exactly:

- `docs/development/ai/swarm-results/aia048-source-keyed-live-sampler-churn-repair-20260802/result.md`

No other path is authorized. In particular, do not change Cargo/Rust product
sources, `readonly_source_exec.py`, the release builder, evidence schemas,
retention budgets, global-lock policy, the File Ancestry candidate, or the
existing AIA-046 review report.

## Non-Goals

- Do not make live sampling an atomic snapshot, quota, or complete ENOSPC/OOM
  prevention mechanism.
- Do not remove or relax the live allocation, aggregate, entry-count,
  deadline, or free-space checks.
- Do not tolerate hostile replacement, symlink traversal, identity mismatch,
  device escape, or a malformed target namespace as ordinary Cargo churn.
- Do not change target-retention sizing, cache-key construction, artifact
  export, provenance publication, test-log correlation, or Cargo scheduling.
- Do not repair File Ancestry product code or use a retry-only workaround to
  turn the recorded BLOCK into PASS.

## Required design properties

1. Separate the **live best-effort reaction sample** from the **quiescent final
   retention measurement** explicitly in code. The final and admission scans
   remain strict and must still reject missing/replaced entries, symlinks,
   identity changes, device escape, entry-limit excess, deadline excess, and
   malformed namespaces.
2. During a live sample only, an entry that was returned by `scandir` and is
   proven absent by descriptor-relative lookup may be classified as ordinary
   gate-owned churn. It must not become a fatal sampling error.
3. A same-name replacement, symlink, wrong inode/device, or containment escape
   is not equivalent to disappearance. It remains fail-closed or causes a
   bounded resample that can only accept after the replacement itself is
   safely measured.
4. Bound all resampling by the existing live-scan deadline. Do not add an
   unbounded retry loop, sleep-until-quiet policy, process suspension, signal
   race, shell command, or a fixed delay that can make an active build
   unmonitorable.
5. Preserve the one-second supervisor cadence, per-namespace growth bound,
   32-GiB aggregate allocation budget, and 100-GiB free-space floor. The
   filesystem free-space sample remains independent and must still run when
   namespace churn was observed.
6. State the real guarantee: live sampling is a best-effort reaction boundary
   and may miss a fast burst between samples; the strict post-gate scan decides
   retention. Do not claim a filesystem quota or atomic namespace snapshot.
7. Keep every result/log/candidate/source-key predicate and failure code stable
   except for the false `live-target-usage-unavailable` outcome on ordinary
   gate-owned disappearance.

The implementer may choose the exact internal representation. Prefer a named
live-scan policy/result over a broad `except FileNotFoundError: pass` so the
strict callers cannot accidentally inherit tolerant semantics.

## Required deterministic tests

Add non-vacuous tests covering all of the following:

1. a live scan sees a file disappear between enumeration and no-follow stat and
   completes without `live-target-usage-unavailable`;
2. a live scan sees a directory disappear before descriptor reopen and
   completes safely;
3. a same-name replacement or identity mismatch is not silently skipped;
4. strict admission/final measurement retains the old fail-closed behavior for
   the same disappearance and replacement controls;
5. stable over-limit namespace allocation still blocks;
6. aggregate-budget and filesystem-free-floor blockers remain reachable;
7. the live-scan deadline and entry-count limits remain enforced;
8. a bounded concurrent-churn fixture repeatedly creates/removes entries while
   the sampler runs and proves the sampler makes progress; the control without
   the repair must fail for the intended behavioral reason, not because a
   symbol is absent;
9. result/log cleanup and candidate unchangedness remain true after both PASS
   and BLOCK paths.

Do not weaken assertions merely to accommodate nondeterminism. Use hooks or a
controlled temporary directory to make the disappearance windows exact.

## Validation

The implementation lane runs no Cargo, Rust, install, Docker, network, or
dogfood command. It must run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

After the clean implementation candidate is committed, the manager must rerun
the exact failed source-keyed File Ancestry command. Acceptance requires:

- result `PASS`;
- one `codefire-rs` harness with 40 or more selected tests and zero failures;
- runtime guard `PASS`;
- committed target run state;
- candidate HEAD/tree unchanged and clean;
- complete log/result digests; and
- no residual namespace or snapshot quarantine.

Only then may the manager continue with Clippy and broader Rust gates.

## Candidate and review topology

1. manager packet commit (this file and the AIA-048 backlog row);
2. one implementation commit changing exactly the three implementation paths;
3. one result-only commit adding exactly the result path;
4. one fresh independent report-only reviewer commit.

Any implementation-byte change after candidate freeze creates a new subject.
One fresh exact reviewer is sufficient unless the repair introduces a new
security/authority boundary. The reviewer must inspect strict-versus-live call
sites, all churn controls, the full non-Cargo suite, and the manager's live
File Ancestry gate evidence. Optional unrelated hardening is
`nonblocking_improvement` and does not reopen this repair.

## Handoff

Report exact commit/tree/parent/path set; code mechanism; strict/live caller
matrix; tests and counts; unavailable validation; remaining risks; and the
exact manager rerun command. Do not integrate or claim AIA-046, File Ancestry,
or CFR-168 complete.
