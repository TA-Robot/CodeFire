# CFB Issue Details

This directory keeps the audit trail for CodeFire dogfood issues.

Current interpretation:

- File count is not the open issue count.
- `Status: fixed` and `Status: fixed in v0.7` are closed.
- The authoritative current open count is tracked in `docs/development/current-cycle-burn-down-plan.md` and `docs/development/codefire-improvement-cycle.md`.
- As of the 2026-07-30 CFR-168 recovery gate, CFB has 138 tracked issues: 135 fixed and CFB-133/136/138 investigating. CFB-134/135/137 are closed; CFB-133 maps to CFR-170 typed actions, CFB-136 is the admitted CFR-168 validation-cache blocker, and CFB-138 is the admitted CFR-168 default-profile upload-scale blocker. No unrelated open CFB discovery is admitted until the frozen batch closes.

The historical 146-item global burn-down batch was an initial frozen batch size across CFB and CFR issues. It is already closed and must not be read as remaining open work.
