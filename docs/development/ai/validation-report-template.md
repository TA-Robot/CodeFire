# AI Validation Report Template

This is a handoff shape, not passing evidence. Replace every placeholder; an omitted, skipped, or capability-blocked command must not be reported as passed.

```text
Task packet ID:
Base revision:
Head revision:
Changed-file digest:
Environment/capability artifact:

Validation:
- validation_id:
  exact_command:
  status: passed | failed | blocked
  exit_code:
  duration_ms:
  log_sha256:
  result_summary:
  capability_evidence_path:

Claims verified:
- claim:
  evidence_refs:

Known gaps:
-

Recommended decision: approve | request_changes | block
```

For approval, serialize the equivalent machine-readable record with `schemas/review-packet.v1.schema.json`. The reviewer must bind the exact task digest, committed base/head, canonical patch digest, changed-file set, and every required validation ID.
