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
| reviewer cannot bind exact base/head/patch | invalid review evidence | regenerate canonical patch and review packet | approval |
| live model call succeeds but has no scored oracle | connectivity/routing evidence only | record as smoke; use historical corpus for quality claims | tier superiority or defect-escape rate |
| expected file/link is absent after document cleanup | possible stale inbound reference | consult deletion manifest and Git recovery revision; promote only the smallest durable requirement | need to resurrect the superseded document |

Escalate to Ultra when the failure implicates user-work loss, identity, sealed commits, path containment, security, canonical deletion, migration, or a completion claim.
