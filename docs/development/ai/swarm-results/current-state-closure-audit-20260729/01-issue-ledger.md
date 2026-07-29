# Current-State Issue Ledger Audit

## Audit Boundary

- Frozen snapshot: `e2a218e015953f082245f36dbc6aee906b837e5a` (`2026-07-29` audit worktree).
- Frozen batch: `BATCH-2026-07-review-portfolio-completion`.
- This report reconstructs current work only. It does not change issue status, product code, the batch scope, or any management source.
- Classification vocabulary:
  - `satisfied`: current-snapshot implementation/evidence directly meets the row.
  - `partial`: a real slice exists, but some required surface, gate, or lifecycle evidence remains.
  - `missing`: no current-snapshot evidence meets the row.
  - `contradicted`: current evidence directly conflicts with the requirement or another authoritative current-state claim.
  - `evidence-not-current`: a claim depends on an absent external artifact, an older run, or evidence not rerun against this snapshot.
- Row identifiers are audit-local. `AC-*` expands an issue Acceptance Criteria bullet; `TC-*` expands a Triage Checklist checkbox.

## Sources Read

The audit read the management hub, lifecycle model, improvement cycle, active burn-down plan, root-cause map, summary ledgers, portfolio disposition ledger, strategy ordering, history, and all eight investigating detail files:

- `docs/development/README.md`
- `docs/development/project-management.md`
- `docs/development/codefire-improvement-cycle.md`
- `docs/development/current-cycle-burn-down-plan.md`
- `docs/development/issue-root-cause-map.json`
- `docs/development/bug-backlog.md`
- `docs/development/code-review-issues-2026-06-06.md`
- `docs/development/bug-issues/README.md`
- `docs/development/code-review-issues/README.md`
- `docs/development/codefire-review-portfolio/README.md`
- `docs/development/codefire-review-portfolio/review-dispositions.json`
- `docs/development/strategy/modernization-roadmap.md`
- `docs/development/strategy/decision-register.md`
- `docs/development/history.md`
- `docs/development/bug-issues/cfb-133.md`
- `docs/development/bug-issues/cfb-136.md`
- `docs/development/code-review-issues/cfr-168.md`
- `docs/development/code-review-issues/cfr-169.md`
- `docs/development/code-review-issues/cfr-170.md`
- `docs/development/code-review-issues/cfr-171.md`
- `docs/development/code-review-issues/cfr-172.md`
- `docs/development/code-review-issues/cfr-173.md`

## Recomputed Open Set

The detail-file recomputation, not a prose count, is authoritative for this report:

```bash
for f in docs/development/bug-issues/cfb-*.md \
         docs/development/code-review-issues/cfr-*.md; do
  status=$(awk -F'|' '/^\| Status \|/{gsub(/^[ \t]+|[ \t]+$/, "", $3); print $3; exit}' "$f")
  printf '%s\t%s\n' "$(basename "$f" .md | tr '[:lower:]' '[:upper:]')" "$status"
done
```

| Family | Detail files | Closed statuses | Investigating | Exact investigating IDs |
|---|---:|---:|---:|---|
| CFB | 137 | 135 `fixed` | 2 | `CFB-133`, `CFB-136` |
| CFR | 175 | 159 `fixed` + 10 `fixed in v0.7` | 6 | `CFR-168`..`CFR-173` |
| **Total** | **312** | **304** | **8** | **the frozen remaining issue set** |

The exact open/investigating count is therefore **8**. This agrees with the management hub, improvement cycle, CFB/CFR detail READMEs, CFR summary, and the frozen-scope list. One active-plan sentence is stale: `current-cycle-burn-down-plan.md:31` says CFB has 136 total / 134 fixed, while the snapshot actually has 137 total / 135 fixed after closed `CFB-137`. The stale total does **not** change the two investigating CFB IDs or the eight-issue total.

## Frozen Dependency DAG

The burn-down plan freezes this root order and forbids promoting a later root past an unmet predecessor:

```text
CFR-168 + CFB-136  bounded work / snapshot-safe validation
        |
        v
CFR-169            proof-enabling ownership seams
        |
        v
CFR-170 + CFB-133  typed postcondition-driven executable actions
        |
        v
CFR-171            claim/proof descriptor
        |
        +----------------------+
        |                      |
        v                      v
CFR-172            CFR-173 product pilot prerequisites also include CFR-168
read-only adoption      (work counters) and CFR-171 (claim descriptor)
        |                      |
        +----------->----------+
                    v
           batch Close / Install / Dogfood / Retrospect
```

Direct dependency facts:

| Root | Predecessor/blocker | Evidence |
|---|---|---|
| `CFR-168` | none among remaining roots; `CFB-136` is its same-root correctness blocker | burn-down order 6; `CFB-136` says it is a CFR-168 Close blocker |
| `CFR-169` | `CFR-168` | burn-down order 7; its audit says not to physically split HTTP before CFR-168 publication semantics settle |
| `CFR-170` + `CFB-133` | `CFR-169` by frozen root order | burn-down order 8; CFB-133 is deduplicated into this root |
| `CFR-171` | `CFR-170` by frozen root order | burn-down order 9 |
| `CFR-172` | safety/contract predecessors through `CFR-171` | issue explicitly reuses `CFR-163`; batch policy puts experiments after safety/contract roots |
| `CFR-173` | `CFR-168` counters and `CFR-171` descriptor; frozen order also places it after `CFR-172` | `cfr-173.md:21-22`; burn-down order 11 |

Rows marked `blocks later roots` below are root-close rows whose failure keeps every downstream root frozen. Acceptance rows inside a root are also blockers unless the detail file explicitly narrows them.

## Requirement-to-Evidence Matrix

Evidence-path key used by the compact matrix citations:

- `migration.rs` = `crates/codefire-cli/src/migration.rs`
- `migration_apply.rs` = `crates/codefire-cli/src/tests/migration_apply.rs`
- `tests.rs` = `crates/codefire-cli/src/tests.rs`
- `store/lib.rs` = `crates/codefire-store/src/lib.rs`
- `metrics.rs` = `crates/codefire-cli/src/metrics.rs`
- `remote.rs` = `crates/codefire-cli/src/remote.rs`
- `source-data-model-catalog.md` = `docs/development/source-data-model-catalog.md`
- `current-cycle-burn-down-plan.md` = `docs/development/current-cycle-burn-down-plan.md`

### CFB-133 — Migration Action Drift

Current interpretation: the local migration defect is implemented and focused-tested in this snapshot, but the parent CFR-170 schema/root and lifecycle gates remain open. The original installed reproduction artifact `/tmp/codefire-migrate-action-audit.oSUz5T` is absent on this audit host.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFB133-AC-1` | Successful apply with empty `remaining_actions` and no blocker returns no review/apply action. | `migration.rs:578-583`; `migration_apply.rs:176-180`; focused test rerun passed on 2026-07-29. | `satisfied` | No; preserve during CFR-170 integration. |
| `CFB133-AC-2` | Partial/failed apply derives safe recovery only from `remaining_actions`, not consumed plan count. | `migration.rs:622-625`; `migration_apply.rs:183-204`; issue progress records bounded idempotent retry. | `satisfied` | No; preserve during typed-action conversion. |
| `CFB133-AC-3` | Dry-run still returns apply; check returns only state-appropriate read-only action. | `migration.rs:594-619`; decision-table test `migration_next_actions_follow_mode_and_current_postcondition`. | `satisfied` | No. |
| `CFB133-AC-4` | JSON/text planned/applied/remaining audit fields remain. | `migration.rs:550-554`; apply/replay tests retain historical planned/applied and empty remaining. | `satisfied` | No. |
| `CFB133-TC-1` | Reproduction fixture or characterization test exists. | Detail checklist checked; focused decision-table and replay tests exist. The original `/tmp` JSON is absent. | `satisfied` | No; source-controlled regression is current even though installed artifact is not. |
| `CFB133-TC-2` | Existing CFB/CFR duplication checked. | Detail maps it to RVC-034/035 and CFR-170; root-cause map uses `typed-automation-outcome-actions`. | `satisfied` | No. |
| `CFB133-TC-3` | Root fix mapped into frozen batch. | Root-cause map and burn-down order 8 bind `CFB-133` to `CFR-170`. | `satisfied` | No. |
| `CFB133-TC-4` | Implementation, full gate, Install/Dogfood, history update complete. | Implementation and isolated focused gates are documented; detail explicitly says independent review, main integration, Install/Dogfood, and final transition remain. No current installed replay exists. | `partial` | **Blocks CFR-170 close and all later roots.** Integrate under CFR-170, rerun full gates, fresh Install, installed original/recreated replay, canonical Dogfood, independent review, then synchronize issue/map/summary/history. |

### CFB-136 — Snapshot-Safe Validation Reuse

Current interpretation: the central same-inode counterexample and local branch/Store session behavior are reversed in current source. The issue remains correctly open because all consuming surfaces, compatibility, full gate, Install/Dogfood, and Retrospect are incomplete. A canonical data-model sentence still claims in-place rewrites are undetected, contradicting the implemented session path.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFB136-AC-1` | Same-path/same-inode/same-size record mutation is not hidden by a cache hit. | `sealed_validation_session_detects_extension_only_same_metadata_rewrite` and `branch_list_finalization_rejects_same_inode_extension_only_tamper`; the CLI test proves same device, inode, length, and mtime and typed digest-change failure. Both focused tests rerun passed. | `satisfied` | No for Store/local branch; every other reuse surface still belongs to AC-5. |
| `CFB136-AC-2` | Immutable shared DAG validates once and preserves cold result/order/diagnostic. | Store many-head session test passes; branch-list 24-head test asserts shared memoization, exact reads/revalidation, repeat equality, and metrics-neutral domain JSON. | `satisfied` | No for tested branch/Store paths; retain equivalence in remaining surface wiring. |
| `CFB136-AC-3` | Runtime lacking snapshot/lock bypasses cache or fails closed; path/mtime are not authority. | Store/branch use pinned capabilities and final digest/size revalidation; stable-identity-unavailable behavior is documented as no reuse. No complete Rust/Python all-surface matrix is current. | `partial` | **Blocks CFR-168 close.** Prove bypass/fail-closed behavior for Doctor, storage, ancestry, remote, and Python paths. |
| `CFB136-AC-4` | Aggregate hit/miss/revalidated/invalidated counters exist and metrics toggle does not alter semantics. | Store exposes memo hits/misses and revalidated document/byte work; branch metrics project memo/revalidation and metrics-on/off equality is tested. There is no explicit aggregate `invalidated` counter across all surfaces. | `partial` | **Blocks CFR-168 close.** Finish common counter vocabulary/projection, including invalidation, and all-command metrics-neutrality. |
| `CFB136-AC-5` | Full CFR-168 scale, compatibility, Install, Dogfood, Retrospect gate passes before fixed. | Detail and burn-down plan explicitly retain Doctor/storage/ancestry, capability/metrics, Install/Dogfood, and full gate as residuals. | `missing` | **Blocks every later root.** Complete the CFR-168 queue and lifecycle. |
| `CFB136-TC-1` | Classified as the same-root CFR-168 blocker. | Checked in detail; root-cause map uses `decision-equivalent-work-budget`. | `satisfied` | No. |
| `CFB136-TC-2` | Snapshot/cache subject and focused mutation fixture frozen. | ADR-020 plus Store/CLI fixtures define the subject and final revalidation behavior. | `satisfied` | No. |
| `CFB136-TC-3` | Counterexample and same-metadata extension-only fixture fixed in tests. | Source-controlled Store and CLI tests exist and reran successfully. | `satisfied` | No. |
| `CFB136-TC-4` | Root fix, full gate, fresh Install, Dogfood, Retrospect complete. | None of the lifecycle close evidence is current; status remains investigating. | `missing` | **Blocks CFR-168 and every later root.** |

### CFR-168 — Decision-Equivalent Bounded Work

Current interpretation: this is a very large in-progress implementation, not an unstarted root. Core ledger/counters, indexed Trace evaluation, iterative Store/signature/Python traversal, bounded HTTP response/error transport, local branch session, substantial Doctor accounting, existing-project upload preflight, and many independently reviewed owner slices exist. The issue file and newest history entries repeatedly say production Python publication composition/handler wiring and other surfaces remain frozen. There is no fresh Install, canonical Dogfood, or whole-root independent acceptance.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR168-AC-1` | Selected operations have documented time/memory order and scale fixtures; recursive stack growth removed. | `DecisionWorkLedger` docs; iterative Store 20,000-depth test; iterative Rust/Python signature/sealed traversal; Trace index; deep shared-DAG HTTP tests. Current source docs describe ownership/counters. Coverage is not complete for every selected operation. | `partial` | **Blocks all later roots.** Finish declared operation inventory, complexity statements, and scale fixtures for remaining Doctor/storage/ancestry/HTTP publication paths. |
| `CFR168-AC-2` | Internal reads/validation/materialization stop on budget before output limit. | Core atomic reservations, Store read/hash/revalidation budgets, bounded HTTP writer/parser, object-record admission, Doctor accounting, upload closure/preflight tests exist. Current docs still list unaccounted Doctor summary/active candidates and production Python publication/handler gaps. | `partial` | **Blocks all later roots.** Connect every remaining allocation/read/effect path to the same request ledger before effects/headers. |
| `CFR168-AC-3` | Cache on/off has identical diagnostic/result/order; changed Store subject is not reused. | Branch repeat/metrics-neutrality, Store cold/warm work, same-metadata digest change, branch/opened/Doctor fences provide strong evidence. `source-data-model-catalog.md:197` still says an in-place rewrite is undetected, contradicting current session tests. Full all-surface equivalence is absent. | `partial` | **Blocks all later roots.** Reconcile canonical docs, complete surface parity corpus, and prove no unsafe legacy path remains. |
| `CFR168-AC-4` | x10 work stays within accepted bound; exhaustion is explicit failure/continuation, never partial success. | Typed exit 60 / HTTP 413, exact/+1 tests, many-head counters, deep traversal, response/object limits, and no-mutation preflight exist. No single accepted x10 cross-surface curve, whole-root capability/metrics projection, or installed scale reversal is current. | `partial` | **Blocks all later roots.** Publish accepted bounds/curves, run them at 1x/3x/10x, and prove failure atomicity across local/HTTP/Python surfaces. |
| `CFR168-TC-1` | Resource/performance sources re-audited. | Checked detail, RVC mapping, ADR-020 characterization. | `satisfied` | No. |
| `CFR168-TC-2` | Local closed-issue duplication removed. | Checked detail and portfolio disposition exactly-once map. | `satisfied` | No. |
| `CFR168-TC-3` | Common budget root frozen. | ADR-020 accepted for implementation; root-cause map and task packet bind the shared contract. | `satisfied` | No. |
| `CFR168-TC-4` | Implementation, scale gate, Install/Dogfood, Retrospect complete. | Implementation is extensive but explicitly production-unwired in the newest history; no current full source gate, fresh installed binary, installed reversal, canonical Dogfood, or root-level Retrospect. | `partial` | **Blocks every later root.** Complete and integrate production wiring; then run the complete lifecycle. |

Required evidence-category status for CFR-168/CFB-136:

| Evidence category | Current evidence | Audit conclusion |
|---|---|---|
| Implementation | Core/Store/CLI/Python code and canonical source docs contain many accepted slices. | `partial` — production publication composition/handler and residual consumers remain. |
| Focused tests | Current Store shared-DAG and CLI same-inode tests passed during this audit; hundreds of source-controlled focused tests exist. | `partial` — representative, not the entire frozen matrix. |
| Full tests | Historical component/full counts are recorded up to 2026-07-22. | `evidence-not-current` — no complete snapshot-wide source gate was rerun here. |
| Documentation | ADR-020 and source docs are extensive. | `partial` — at least `source-data-model-catalog.md:197` is stale/contradictory, and residual ownership is explicitly documented. |
| Install | Earlier roots have installed binaries; no CFR-168 closed-source install exists. | `missing`. |
| Dogfood | Earlier canonical Dogfood is healthy; no installed CFR-168 scale/counterexample reversal exists. | `missing`. |
| Independent review | Numerous slice-level `ACCEPT*` decisions are recorded. | `partial` — no whole integrated root acceptance; newest text freezes production wiring. |
| Retrospective | No CFR-168 Retrospect exists. | `missing`. |

### CFR-169 — Architecture / Proof Seams

Current interpretation: prerequisite module extraction and feature-local tests are already substantial, but the exact residual named by the issue remains. Snapshot sizes are still broad (`main.rs` 12,647 lines, `remote.rs` 4,101, `tests.rs` 13,069, Python suite 13,602, Store 20,539). CLI `remote.rs` still owns object-reference traversal while Store separately owns object-kind/reference validation.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR169-AC-1` | Every public command path has uniquely traceable parse→validate→plan→apply→render owners. | `command-source-trace.md` covers commands and many responsibility modules exist, but `main.rs` still owns broad commit/merge/repository flows and the issue audit names remaining vertical slices. | `partial` | **Blocks CFR-170 onward.** Extract only the frozen proof seams, then update trace ownership and checker references. |
| `CFR169-AC-2` | CLI does not redefine object kind/reference/root schema; it uses Store APIs. | Store owns `OBJECT_KINDS` and validators, but `remote.rs::try_for_each_payload_reference` and collection logic still interpret references. | `contradicted` | **Blocks CFR-170 onward.** Move typed traversal/registry to Store and delete duplicate CLI semantics after golden equivalence. |
| `CFR169-AC-3` | Feature regressions live in feature modules; transport/format use integration corpus. | Many feature-local modules exist under `crates/codefire-cli/src/tests/`; large shared Rust/Python suites still retain broad helpers and cases. | `partial` | **Blocks CFR-170 onward.** Move only duplicated invariant fixtures and establish cross-boundary corpus ownership. |
| `CFR169-AC-4` | Behavior/JSON/object bytes are golden-identical before/after, without unrelated changes. | Existing golden/conformance infrastructure exists, but no CFR-169 extraction or before/after golden package has run. | `missing` | **Blocks CFR-170 onward.** Implement each seam in small behavior-preserving commits with byte/envelope/object golden comparisons. |
| `CFR169-TC-1` | Current module/test ownership re-audited. | Detail contains a concrete current-source audit. | `satisfied` | No. |
| `CFR169-TC-2` | Previous maintainability issues deduplicated. | Detail names prior CFRs and avoids line-count-only reopening. | `satisfied` | No. |
| `CFR169-TC-3` | Only proof-enabling scope frozen. | Detail and burn-down plan explicitly freeze limited seams and defer HTTP physical split until CFR-168 settles. | `satisfied` | No. |
| `CFR169-TC-4` | Behavior-preserving extraction and full gate complete. | No root implementation/gate/Install/Dogfood/Retrospect evidence exists. | `missing` | **Blocks CFR-170 onward.** |

### CFR-170 + CFB-133 — Typed Outcomes and Executable Actions

Current interpretation: the CFB-133 migration reducer slice is complete locally. The shared all-command outcome/action contract is not implemented: current `automation_next_action` values remain JSON objects centered on `command`, `description`, and `target`; placeholder commands such as `commit -m <message>` remain machine-looking strings.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR170-AC-1` | Representative command matrix aligns process exit, envelope exit, `ok`, blocking diagnostic, and primary outcome. | Existing envelope regressions and common schema exist; detail says representative all-command matrix remains open. | `partial` | **Blocks CFR-171 onward.** Define one typed vocabulary and run a representative success/warning/blocking/error matrix. |
| `CFR170-AC-2` | Placeholders are not executable actions; executable actions provide argv without shell parsing. | Current actions still expose command strings; `status` can emit `commit -m <message>`. No typed argv schema exists. | `contradicted` | **Blocks CFR-171 onward.** Add versioned typed action request, distinguish templates from executable actions, derive human strings from argv. |
| `CFR170-AC-3` | Stale target/precondition rejects or replans; mutations declare authority and dry-run support. | Some commands independently enforce locks/idempotency/subjects, but next-action objects do not uniformly carry expected precondition, mutation class, authority, or dry-run capability. | `missing` | **Blocks CFR-171 onward.** Implement shared precondition-bound action schema and execution/replan tests. |
| `CFR170-AC-4` | CFB-133 success gives zero actions; partial apply only remaining action. | Current migration code/tests meet this exactly. | `satisfied` | No; preserve as a migration conformance case. |
| `CFR170-TC-1` | Automation drift and CFB-133 reproduced. | Checked; installed reproduction documented, source regression current. | `satisfied` | No. |
| `CFR170-TC-2` | Existing envelope issues deduplicated. | Checked detail/root map. | `satisfied` | No. |
| `CFR170-TC-3` | Typed-action scope frozen. | Required-fix list and root map define it. | `satisfied` | No. |
| `CFR170-TC-4` | Implementation, schema docs, Dogfood, Retrospect complete. | Only migration slice exists; no shared schema, all-command matrix, Install/Dogfood, or Retrospect. | `partial` | **Blocks CFR-171 onward.** |

### CFR-171 — Claim / Proof Descriptor

Current interpretation: predecessor contracts already bind effective policy, coverage, verification subjects, certificate fields, and consistency checks, but there is no single versioned claim descriptor containing proof kind/trust and losslessly projected across all required surfaces.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR171-AC-1` | All surfaces for the same sealed commit return byte/digest-equivalent claim descriptor. | Cross-root subject/policy/coverage consistency exists, but no claim-descriptor type/schema or cross-surface equality corpus exists. | `missing` | **Blocks CFR-172/173.** Define canonical descriptor/vector and project it through verify, certificate, show, review pack, explain. |
| `CFR171-AC-2` | Any subject/policy/coverage/proof-kind field tamper fails sealed validation. | Existing subject/policy/coverage tamper validation covers subsets; `proof_kind` is not a bound descriptor field. | `partial` | **Blocks CFR-172/173.** Add complete tamper table and Store mismatch validation. |
| `CFR171-AC-3` | Legacy does not gain unsupported claims; unknown critical claim version rejects. | General critical-extension/version behavior exists, but no legacy/current claim-descriptor matrix exists. | `missing` | **Blocks CFR-172/173.** Add versioned optional legacy behavior and unknown-critical rejection vectors. |
| `CFR171-AC-4` | Independent minimal consumer identifies claim scope and missing proof without source knowledge. | Strategy requires this; no task packet/result fixture exists for this root. | `missing` | **Blocks CFR-172/173.** Build and independently run clean-room conformance consumer. |
| `CFR171-TC-1` | Claim/certificate gap re-audited. | Checked detail and RVC-001/019 mapping. | `satisfied` | No. |
| `CFR171-TC-2` | Deduplicated from authoritative-decision issues. | Checked detail reuses ADR-013/014 and existing roots. | `satisfied` | No. |
| `CFR171-TC-3` | Downstream proof scope frozen. | Required-fix and root map define the bounded descriptor scope. | `satisfied` | No. |
| `CFR171-TC-4` | Implementation, conformance, Dogfood, Retrospect complete. | No implementation or lifecycle evidence. | `missing` | **Blocks CFR-172/173 and final batch close.** |

### CFR-172 — Read-Only Adoption / Mental-Model Experiment

Current interpretation: existing read-only commands, assessment subjects, historical real-repository hash matrices, strategy hypotheses, and dogfood smoke tooling are useful prerequisites. They are not the required bounded experiment: no thin adapter/profile, scored clean-room comprehension result, setup/value measurements, or terminal adopt/iterate/stop decision exists.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR172-AC-1` | Adapter/profile changes no branch/object/open state, proven by before/after hashes. | Earlier roots have real read-only hash matrices; no CFR-172 adapter/profile matrix exists for canonical/configured/plain repositories. | `evidence-not-current` | **Blocks final batch close and, by frozen order, CFR-173 execution.** Recreate the declared three-repository hash matrix with the experiment artifact. |
| `CFR172-AC-2` | Clean-room consumer identifies base/head/worktree/claim kind without private help. | No scored clean-room result; CFR-171 claim descriptor is absent. | `missing` | **Blocks final batch close.** Wait for CFR-171, then run independent task packet. |
| `CFR172-AC-3` | Measure setup time, first useful result, misunderstanding, wrong action, maintenance cost; decide adopt/iterate/stop. | Strategy has hypotheses/falsifiers but no experiment record or measurements. | `missing` | **Blocks final batch close.** Execute bounded cohort and record negative/missing results. |
| `CFR172-AC-4` | If value does not exceed cost, do not promote; record terminal stop. | Decision-register falsifiers support the rule, but no result exists. | `missing` | **Blocks final batch close.** Record the terminal decision without feature rescue. |
| `CFR172-TC-1` | Hypothesis and predecessors re-audited. | Checked detail references prior issues/ADRs and VerifiedOpenSnapshot. | `satisfied` | No. |
| `CFR172-TC-2` | Mutation/adoption duplicates removed. | Checked detail. | `satisfied` | No. |
| `CFR172-TC-3` | Bounded experiment scope frozen. | Required-fix/test plan and strategy hypotheses define it. | `satisfied` | No. |
| `CFR172-TC-4` | Experiment, decision, history/roadmap update complete. | No execution or decision record. | `missing` | **Blocks final batch close and CFR-173 by frozen order.** |

### CFR-173 — Product Falsification Pilot

Current interpretation: historical dogfood logs and existing Fire/verification metrics are input material only. The issue explicitly depends on CFR-168 work counters and CFR-171 claim descriptor. No privacy-preserving cohort protocol, deterministic metric calculator, threshold record, retained negative data, or terminal narrow/stop decision exists.

| Row | Requirement | Evidence at snapshot | Class | Blocks later roots / exact remaining work |
|---|---|---|---|---|
| `CFR173-AC-1` | Replay identical baseline/current cohort; save task/input/result digests and failure logs. | Historical dogfood is rich but not a controlled same-task cohort and lacks the required digest protocol. | `missing` | **Blocks final batch close.** Define tasks after CFR-168/171, execute baseline/current, retain failures. |
| `CFR173-AC-2` | Evaluate Fire by correct action, defect prevention, and lead time, not count. | Existing counts/actionability discussions exist; no deterministic cohort scoring/result. | `missing` | **Blocks final batch close.** Implement metric calculator and classification review. |
| `CFR173-AC-3` | Stay within privacy/retention boundary; missing data is not success. | Strategy/issue states the rule; no evidence format, retention decision, or negative-data fixture exists. | `missing` | **Blocks final batch close.** Freeze local aggregate schema and test missing/failed/negative retention. |
| `CFR173-AC-4` | If threshold missed, narrow/stop rather than add features; record terminal decision. | Decision framework exists, no pilot result/decision. | `missing` | **Blocks final batch close.** Predeclare thresholds/owner, then record adopt/narrow/stop. |
| `CFR173-TC-1` | Product hypothesis/evidence gap re-audited. | Checked detail and RVC-003/004/010 mapping. | `satisfied` | No. |
| `CFR173-TC-2` | Existing metrics/UX issues deduplicated. | Checked detail. | `satisfied` | No. |
| `CFR173-TC-3` | Falsifiable pilot scope frozen. | Required-fix/test plan and strategy decision register define the bounds. | `satisfied` | No. |
| `CFR173-TC-4` | Pilot, stop decision, Retrospect complete. | No pilot execution or decision record. | `missing` | **Blocks final batch close.** |

## Row Classification Summary

The 65 expanded rows classify as follows:

| Class | Rows |
|---|---:|
| `satisfied` | 31 |
| `partial` | 13 |
| `missing` | 18 |
| `contradicted` | 2 |
| `evidence-not-current` | 1 |
| **Total** | **65** |

The two contradictions are deliberate audit findings, not status changes:

1. `CFR169-AC-2`: CLI still owns object-reference traversal despite the acceptance requirement that Store own it.
2. `CFR170-AC-2`: current next actions still expose placeholder/string commands as machine-looking executable actions.

Separately, the snapshot contains current-state documentation drift:

- `current-cycle-burn-down-plan.md:31` has stale CFB totals (136/134 rather than 137/135).
- `source-data-model-catalog.md:197` says validated-file in-place rewrites are not detected, while current session/branch fixtures detect same-inode, same-size, same-mtime digest changes. This is not counted as an additional requirement row, but must be reconciled before CFR-168 Close.

## Per-Issue Evidence Coverage

This table makes the lifecycle evidence requirement explicit for every investigating issue. `current` means source-controlled evidence is present at the audited snapshot; `historical` means only an older recorded run exists; `none` means no completing evidence exists.

| Issue | Implementation | Focused tests | Full gate | Docs | Install | Dogfood | Independent review | Retrospective |
|---|---|---|---|---|---|---|---|---|
| `CFB-133` | current local slice | current; audit rerun pass | historical isolated claim only | current progress/trace | none for this fix | none | none for integrated root | none |
| `CFB-136` | current Store/local-branch slice | current; audit reruns pass | historical component claims only | current issue/ADR; data-model drift remains | none | none | slice reviews only | none |
| `CFR-168` | extensive current partial | extensive current; representative audit reruns | historical evolving-snapshot claims only | extensive current; explicit residuals/drift | none for closed root | none for closed root | many slice accepts; no whole-root accept | none |
| `CFR-169` | prerequisite modules only; root extraction absent | adjacent tests current, root golden absent | none | current audit/scope | not yet applicable | not yet applicable | none | none |
| `CFR-170` | CFB-133 slice only | migration slice current | none for shared contract | issue/progress current; schema absent | none | none | none for shared contract | none |
| `CFR-171` | none | none | none | requirement/strategy only | none | none | none | none |
| `CFR-172` | no experiment artifact | no scored experiment fixture | none | hypothesis/scope only | no experiment binary/profile | older predecessor read-only probes only | no clean-room result | none |
| `CFR-173` | no pilot/calculator artifact | none | none | hypothesis/scope only | not applicable before protocol | historical uncontrolled dogfood only | none | none |

## Prioritized Closure Queue

### P0 — Finish CFR-168 + CFB-136 Before Any Later Root

1. **Complete production wiring and residual consumer accounting.**
   - Finish Python existing/missing upload terminal composition: pinned gate-relative preimage proof, effect/response/publication shared terminal token, typed reducer, and actual handler connection.
   - Finish remaining Doctor summary/active-state/JSON-candidate accounting, storage/ancestry/remote reuse, and any unaccounted branch/object enumeration.
   - Ensure all effects and response headers are preceded by atomic reservation/preflight.
2. **Complete common capability/metrics contract.**
   - Project stable limits/counters/continuation support on all selected commands.
   - Include explicit invalidation/revalidation semantics and prove metrics enabled/disabled domain equivalence.
3. **Reconcile canonical documentation.**
   - Remove the stale in-place-rewrite limitation and document which APIs are snapshot-safe versus intentionally non-reusing.
   - Update source architecture/trace/data-model/known-limitations together and run the source-doc checker.
4. **Run root-level independent review.**
   - Review the integrated production path, not accepted sidecar/prerequisite slices in isolation.
5. **Run Close lifecycle evidence.**
   - Full source gate, fresh isolated Install, installed counterexample/limit/scale reversal, real read-only probes, canonical Algorithm Dogfood, and Retrospect.

Acceptance commands for P0 (focused first, then broad):

```bash
cargo test -p codefire-store sealed_validation_session_detects_extension_only_same_metadata_rewrite -- --nocapture
cargo test -p codefire-store sealed_validation_session_reuses_many_head_shared_dag_and_revalidates_once -- --nocapture
cargo test -p codefire-cli branch_list_finalization_rejects_same_inode_extension_only_tamper -- --nocapture
cargo test -p codefire-cli branch_list_uses_one_session_for_many_head_shared_dag_and_reports_aggregate_work -- --nocapture
cargo test -p codefire-cli doctor_scan_and_candidates_share_one_accounted_recoverable_session -- --nocapture
cargo test -p codefire-cli http_remote_error_preserves_every_resource_budget_field -- --nocapture
python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_python_http_bundle_work_limits_are_atomic \
  tests.test_codefire_cli.CodeFireCliTests.test_python_http_bundle_deep_shared_dag_is_iterative \
  tests.test_codefire_cli.CodeFireCliTests.test_python_signature_history_is_iterative_for_deep_chain \
  tests.test_codefire_cli.CodeFireCliTests.test_python_sealed_validation_is_iterative_for_deep_chain \
  tests.test_codefire_cli.CodeFireCliTests.test_python_http_upload_stages_only_reachable_existing_closure

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
./demo.sh

bash -n install.sh
mkdir -p target
CFR168_PREFIX=$(mktemp -d "$PWD/target/cfr168-install.XXXXXX")
./install.sh --prefix "$CFR168_PREFIX"
"$CFR168_PREFIX/bin/codefire" --version
"$CFR168_PREFIX/bin/codefire" capabilities --json
(cd subprojects/algorithm-evolution-agent-lab && \
  PYTHONPATH=src python3 -m unittest discover -s tests -v)
python3 docs/development/tools/run_dogfood_smoke.py \
  --binary "$CFR168_PREFIX/bin/codefire" \
  --path subprojects/algorithm-evolution-agent-lab
```

The install/dogfood command set is a minimum. The root must additionally replay exact/+1 budgets, same-inode corruption, deep/shared graphs, existing/missing HTTP upload, no-partial-response/no-mutation, and configured/canonical read-only hash matrices with the installed binary.

### P1 — CFR-169 Proof Seams

1. Move typed object reference registry/traversal from CLI remote code to Store.
2. Move canonical commit/root builders and version validation to their domain/Store owner.
3. Extract only the remaining lifecycle transaction/workflow seams needed for proof; do not split by line count.
4. Relocate duplicated fixtures into feature-local or cross-runtime conformance owners.
5. Prove byte/envelope/object identity before and after every extraction and synchronize the four canonical source docs.

Acceptance commands:

```bash
rg -n 'try_for_each_payload_reference|collect_object_records_internal' crates/codefire-cli/src/remote.rs
rg -n 'OBJECT_KINDS|validate_.*references|object.*builder' crates/codefire-store/src
python3 docs/development/tools/check_source_docs.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 -m unittest discover -s tests -v
```

Close requires the first two searches to demonstrate single ownership rather than merely still finding both definitions.

### P2 — CFR-170 + CFB-133 Typed Actions

1. Define a versioned outcome/diagnostic/action schema with `argv`, action mode/template status, target subject, expected precondition, mutation class/authority, idempotency, and dry-run support.
2. Make human command strings a projection from typed argv; never mark `<placeholder>` templates executable.
3. Derive actions from authoritative current postconditions and add stale-precondition reject/replan behavior.
4. Use CFB-133 success/replay/partial failure as mandatory conformance cases.
5. Run representative all-command exit/envelope/blocking/outcome matrix, schema compatibility, installed Dogfood, independent review, and Retrospect.

Acceptance commands:

```bash
cargo test -p codefire-cli migration_next_actions_follow_mode_and_current_postcondition -- --nocapture
cargo test -p codefire-cli failed_apply_recovery_action_is_derived_from_remaining_actions -- --nocapture
rg -n '<[^>]+>' crates/codefire-cli/src | rg 'next_action|command|cli_command'
python3 docs/development/tools/check_project_docs.py
cargo test --workspace --locked
python3 -m unittest discover -s tests -v
```

The placeholder search must leave only explicitly typed non-executable templates.

### P3 — CFR-171 Claim Descriptor

1. Freeze canonical descriptor fields/version and golden vectors.
2. Bind descriptor digest into verification/certificate/root validation.
3. Project losslessly through verify, commit, show, review pack, and explain.
4. Add complete one-field tamper table, legacy/current matrix, unknown-critical rejection.
5. Run an independent minimal consumer with no source/private clarification.

Acceptance commands:

```bash
rg -n 'ClaimDescriptor|claim_descriptor|proof_kind' crates schemas docs
test -f schemas/fixtures/claim-descriptor-v1.json
cargo test -p codefire-core
cargo test -p codefire-store
cargo test -p codefire-cli
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
cargo test --workspace --locked
python3 -m unittest discover -s tests -v
```

The symbol/fixture checks must succeed and the package/full gates must include source-controlled descriptor canonical-vector, tamper, legacy/current, cross-surface, and clean-room conformance tests.

### P4 — CFR-172 Read-Only Experiment

After P3 closes:

1. Freeze the non-mutating adapter/profile and comprehension rubric.
2. Run canonical CodeFire, configured CodeFire, and plain existing repository before/after hash matrices.
3. Give the packet to a clean-room consumer; score base/head/worktree/claim-kind interpretation and wrong actions.
4. Record setup time, first useful result, misunderstandings, maintenance cost, missing/negative data.
5. Record terminal `adopt`, `iterate`, or `stop`; do not promote a failed surface.

Acceptance commands/artifacts:

```bash
export CFR172_CANONICAL_REPO="$PWD/subprojects/algorithm-evolution-agent-lab"
: "${CFR172_CONFIGURED_REPO:?set CFR172_CONFIGURED_REPO to the configured CodeFire repository}"
: "${CFR172_PLAIN_REPO:?set CFR172_PLAIN_REPO to the plain existing repository}"
test -x target/cfr172-install/bin/codefire
test -f docs/development/experiments/cfr-172-readonly-task-packet.json
python3 docs/development/experiments/run_cfr172_readonly_experiment.py \
  --binary target/cfr172-install/bin/codefire \
  --canonical-repo "$CFR172_CANONICAL_REPO" \
  --configured-repo "$CFR172_CONFIGURED_REPO" \
  --plain-repo "$CFR172_PLAIN_REPO" \
  --output docs/development/experiments/cfr-172-readonly-result.json
test -f docs/development/experiments/cfr-172-readonly-decision.md
python3 docs/development/tools/check_project_docs.py
```

The experiment runner, packet, scored result (including before/after hashes), and decision record do not exist at this snapshot and are required deliverables.

### P5 — CFR-173 Product Pilot and Batch Retrospect

After P4 and with P0/P3 outputs available:

1. Freeze same-task baseline/current cohorts and privacy-preserving aggregate schema.
2. Predeclare success/narrow/stop thresholds and decision owner.
3. Run task replay with task/input/result digests and retained failure logs.
4. Score actionable/duplicate/obsolete/false Fire, correct action, prevented defect, lead time, evidence quality, seal recovery, and support cost.
5. Retain missing/failed/negative data; record terminal adopt/narrow/stop.
6. Complete final batch Close synchronization and Retrospect before any new Discovery.

Acceptance commands/artifacts:

```bash
test -f docs/development/experiments/cfr-173-pilot-protocol.md
test -f docs/development/experiments/cfr-173-pilot-result.json
python3 docs/development/experiments/run_cfr173_pilot.py \
  --protocol docs/development/experiments/cfr-173-pilot-protocol.md \
  --output docs/development/experiments/cfr-173-pilot-result.json
python3 -m unittest discover -s tests -p 'test_*pilot*.py' -v
python3 docs/development/tools/check_project_docs.py
(cd subprojects/algorithm-evolution-agent-lab && \
  PYTHONPATH=src python3 -m unittest discover -s tests -v)
```

The metric/retention test modules and pilot decision record do not exist at this snapshot and are required deliverables.

### Final Close Synchronization

Only after P0–P5 acceptance:

1. Change the eight detail statuses to `fixed` or explicit `wontfix`.
2. Synchronize `bug-backlog.md`, `code-review-issues-2026-06-06.md`, both detail READMEs, root-cause map, management hub, active cycle plan, modernization roadmap, known limitations/source docs, and append-only history.
3. Correct the stale CFB totals and remove superseded outside-batch CFB-133 candidate text.
4. Run consistency checks and independently recompute that the open detail count is zero.
5. Record fresh Install, installed Dogfood, independent review, and Retrospect evidence for every root.

```bash
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py

for f in docs/development/bug-issues/cfb-*.md \
         docs/development/code-review-issues/cfr-*.md; do
  awk -F'|' '/^\| Status \|/{gsub(/^[ \t]+|[ \t]+$/, "", $3); if ($3=="open" || $3=="investigating") print FILENAME ": " $3}' "$f"
done
```

The final command must print nothing before the next Discovery begins.

## Root-Close Versus Next-Discovery Work

### Required to close this frozen batch

- All 65 rows above, including lifecycle rows, belong to the eight admitted issues and therefore to root close.
- The incomplete implementation/test/docs/Install/Dogfood/independent-review/Retrospect evidence is not optional follow-up; it is part of the frozen lifecycle contract.
- CFB-133 is no longer merely a discovery candidate: it is an admitted issue under CFR-170. The old outside-batch note in `current-cycle-burn-down-plan.md:317-322` is historical drift and must be removed only during canonical Close synchronization.

### Explicitly outside this frozen batch

- The broader shared diff/merge assessment experiment left from RVC-039 (`current-cycle-burn-down-plan.md:141`).
- Any new RVC/CFB/CFR discovered that does not block one of the frozen outcomes.
- General review-portfolio ideas not promoted to the eleven root owners.
- New marketplace, AI-agent orchestration, GUI, Git-compatible writer, or discretionary feature work.

The management rule remains: a newly observed non-blocker is recorded only as a short next-discovery candidate; it is not assigned a CFB/CFR ID until all eight current issues complete Retrospect.

## Audit Validation

Commands executed against the snapshot during this audit:

| Command | Result |
|---|---|
| detail-file status recomputation across 137 CFB + 175 CFR files | 8 investigating: `CFB-133`, `CFB-136`, `CFR-168`..`CFR-173` |
| `cargo test -p codefire-cli migration_next_actions_follow_mode_and_current_postcondition -- --nocapture` | pass, 1 test |
| `cargo test -p codefire-cli branch_list_finalization_rejects_same_inode_extension_only_tamper -- --nocapture` | pass, 1 test |
| `cargo test -p codefire-store sealed_validation_session_reuses_many_head_shared_dag_and_revalidates_once -- --nocapture` | pass, 1 test |
| `python3 docs/development/tools/check_project_docs.py` before the report existed | pass; 522 Markdown files inventoried |
| `python3 docs/development/tools/check_project_docs.py` after adding the report | pass; 523 Markdown files inventoried |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | pass |

Two earlier commands incorrectly used `--exact` without module-qualified Rust test names and therefore ran zero tests; they are not counted as evidence. The corrected commands above ran the intended tests.

No full workspace gate, fresh Install, installed Dogfood, independent whole-root review, or Retrospect was run for this docs-only audit. Their absence is reflected in the matrix rather than inferred away from historical claims.

## Audit Decision

The frozen batch has **8** exact remaining issues and **65** explicit acceptance/triage rows. The current snapshot is not close-ready:

- CFR-168/CFB-136 contains substantial accepted implementation but still has production wiring, residual surfaces, doc drift, full-gate, Install, Dogfood, independent whole-root review, and Retrospect gaps.
- CFR-169 is audited/frozen but not implemented.
- CFR-170 has only the CFB-133 migration slice; the shared typed executable-action contract is absent.
- CFR-171, CFR-172, and CFR-173 have frozen requirements and predecessor evidence but no completing implementation/experiment lifecycle.

The only valid next action is to finish **P0 CFR-168 + CFB-136**. Promoting or executing later roots beyond predecessor-safe preparatory analysis would violate the frozen dependency order.
