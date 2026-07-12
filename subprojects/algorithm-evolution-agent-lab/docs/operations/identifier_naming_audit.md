# Identifier Naming Audit Operations

This guide describes how to use the identifier naming audit during algorithm development and review.

## Purpose

The naming audit keeps Python identifiers short enough for dense review, CI logs, and CodeFire trace work. Long `REQ-*`, `DES-*`, `CODE-*`, and `TEST-*` atom IDs remain in documentation and trace links; Python identifiers should stay compact.

## Default Policy

- Maximum Python identifier length: 25 characters.
- Audit scope: class names, function names, async function names, argument names, and assigned local names.
- Review output should be bounded to the top findings before posting into review comments.
- Clean output should still be explicit so automation can distinguish "no findings" from missing output.

## Local Workflow

1. Run the naming audit against touched Python files before CodeFire scan.
2. Review the longest names first.
3. Rename only when the shorter name still preserves the research concept.
4. Keep traceability in `# cf-atom:` markers and `codefire.links.yaml`, not in long Python identifiers.
5. Re-run the unit tests and CodeFire verification before commit.

## CI Surfaces

- Use `NameProjCiStatusRun` when CI needs a compact pass/fail JSON payload.
- Use `NameProjCiPackRun` when CI needs every finding for annotation adapters.
- Use `NameProjCiNoteSecRun` when review tooling needs a bounded Markdown section.
- Use `NameProjOpsDoc` when a review comment should include both this workflow guidance and current project findings.

## Review Checklist

- The finding is not just an atom ID that belongs in docs or trace links.
- The proposed shorter name is specific enough for the local module.
- The same concept is named consistently across tests and implementation.
- The rendered Markdown note is bounded and includes an omitted-count marker when needed.
