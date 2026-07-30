# AI Development Failure Cookbook

This cookbook maps recurring failure signatures to the smallest safe next action. Preserve the original command, exit code, stderr/stdout digest, environment capabilities, and task packet ID in the handoff.

| Failure signature | Interpret first as | Required next action | Do not claim |
|---|---|---|---|
| socket bind/connect returns `Operation not permitted` only in managed sandbox | environment restriction | rerun the same focused check with approved socket capability; compare exact test set | product regression or passing full gate from a skipped test |
| `cargo` or `rustc` absent | capability gap | run `probe_validation_capabilities.py`; use the documented isolated/pinned environment or report blocked Rust gate | Rust readiness |
| binary exists but provenance is missing/invalid | stale or unbound artifact | run `build_source_bound_rust.py` from clean HEAD and re-run the probe | current-source behavior from mtime alone |
| task base differs from HEAD | stale packet | regenerate context manifest and task base; re-review changed write set | patch applicability |
| dirty path intersects a task write set | ownership conflict | stop the lane; manager reassigns or checkpoints the existing change | safe parallelism |
| command JSON schema/command/exit code disagree | product contract defect | preserve raw output; create a CFB intake packet; add focused envelope test | successful automation merely because process exit was zero |
| timeout with no output | possible work amplification, deadlock, or environment stall | bound reproduction; collect scale/state facts; use an investigation packet | root cause from timeout alone |
| Fugu log contains `codex_models_manager` / `missing field models`, while worker PID and events continue | non-fatal Sakana model-catalog compatibility warning | keep the lane running; decide from `dev-swarm status`, event growth, final return code, and committed deliverable | provider failure from the warning line alone |
| Claude CLI launches but `auth status` disagrees | advisory probe differs from the actual interactive/helper-backed execution path | attempt the isolated worker; preserve its exact exit/log and never reroute automatically to Codex | unavailable Claude from `auth status` alone |
| reviewer cannot bind exact base/head/patch | invalid review evidence | regenerate canonical patch and review packet | approval |
| live model call succeeds but has no scored oracle | connectivity/routing evidence only | record as smoke; use historical corpus for quality claims | tier superiority or defect-escape rate |
| expected file/link is absent after document cleanup | possible stale inbound reference | consult deletion manifest and Git recovery revision; promote only the smallest durable requirement | need to resurrect the superseded document |

## Review-prevention signatures

Apply these checks during Freeze and focused validation, before an exact
candidate enters the independent-review queue.

| Failure signature | Interpret first as | Required next action | Do not claim |
|---|---|---|---|
| focused tests pass but the new reusable API is exercised only by a test helper | production-reachability gap | bind a real entry point, a direct reusable-API case, and an intentionally broken implementation to the same oracle | production closure from helper-only coverage |
| a claimed ordering test stays green after moving the guarded operation past the effect | vacuous or indirect oracle | observe the actual effect boundary, assert the full trace, and prove the deliberate regression fails for the intended reason | ordering merely because a stage label or mock call was observed |
| caller and callee create different budget, deadline, snapshot, response, or cleanup owners | conservation/identity defect | define one affine owner, its claim/release matrix, and exact underflow, overflow, cancellation, and finalize cases | whole-request ownership from equal numeric limits |
| `finalize` charges a category for which the owner never reserved capacity | owner-formula mismatch | enumerate every retained/temporary/control claim, make reservation and release symmetric, and test the reusable API directly | bounded success from a different live-path owner |
| fallible authority reads, projection, encoding, or selection remain after the final fence and before publication | stale-success window | prepare fallible work first, place the final authority/session fence at the true publication boundary, and inject a mutation at each remaining gap | prepublication safety from an earlier revalidation |
| cleanup clears local debt or ownership before the release is confirmed | unretryable cleanup | retain debt until release succeeds, make cleanup idempotent/retryable, and preserve the primary result separately from cleanup status | zero residual from a best-effort `finally` block |
| a sink can expose a prefix and then fail while the contract promises all-or-typed output | publication-contract mismatch | use a supported atomic boundary or represent partial publication explicitly and suppress any second terminal output | atomic publication from pre-encoding alone |
| the same invariant family is rejected on a second surface | missing shared primitive | stop adding surface-local variants; freeze one primitive contract and migrate its consumers through composition packets | closure from another local special case |
| independently developed candidates overlap on a dispatcher, owner, source contract, or generated projection | composition risk | integrate accepted product deltas only on a manager-owned shadow head, run the smallest cross-surface gate after each delta, and independently review the composed revision | whole-tree merge safety from isolated reviews |
| a broad/full gate runs repeatedly while the candidate or its focused oracle is still changing | validation amplification | keep development on deterministic focused gates and run broad/full validation once on a stable composition revision | extra confidence from repeated stale broad runs |
| a review-ready exact candidate waits while a new implementation surface starts | review debt growth | give the next released permit to exact review or review-derived packet freezing | throughput from higher implementation WIP |

Escalate to Ultra when the failure implicates user-work loss, identity, sealed commits, path containment, security, canonical deletion, migration, or a completion claim.
