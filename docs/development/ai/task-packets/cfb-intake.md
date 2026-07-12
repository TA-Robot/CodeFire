# CFB Intake Packet

Status: reusable.

## Objective

Use this packet when dogfooding reveals reproducible CodeFire product friction. It converts an observation into an auditable issue candidate without silently expanding the active frozen batch.

## Scope

- Read the relevant product contract, command output, environment capabilities, and current frozen batch.
- Write only a bounded intake/evidence note unless the manager explicitly assigns authoritative ledger files.
- Preserve exact reproduction data and distinguish product behavior from environment restrictions.

## Non-Goals

- Do not assign a CFB ID from an implementer or dogfood lane.
- Do not amend the active batch implicitly.
- Do not fix the product while evidence intake and root-cause scope remain ambiguous.

## Intake Fields

```text
Observed command:
Target path:
Expected contract:
Observed result:
Exit code:
Reproduction count:
Minimal reproduction:
Evidence digest or artifact path:
Suspected invariant:
User-work risk:
Workaround:
Environment limitation ruled out:
```

## Decision

Choose exactly one disposition:

- **active-batch blocker**: the finding prevents closure of the current frozen batch; ask the manager to assign the next CFB ID and amend the batch;
- **next discovery candidate**: preserve the packet, but do not assign an issue ID until the current batch closes;
- **environment limitation**: record the capability or sandbox restriction and the independent rerun that would distinguish it from a product defect;
- **documentation/tooling friction**: update the canonical development artifact or AIA backlog instead of creating a product issue;
- **not reproducible**: retain the negative result and the exact attempted reproduction.

## Promotion Gate

Before the manager creates `bug-issues/cfb-NNN.md`:

1. the behavior reproduces or has deterministic static evidence;
2. expected behavior cites a product contract, invariant, or explicit UX rule;
3. environment restrictions have been separated from product behavior;
4. scope, severity, and likely root-fix group are stated;
5. a focused regression command is proposed;
6. `bug-backlog.md`, the detail record, current batch if applicable, root-cause map, and history can be updated atomically.

An intake packet is evidence, not permission to mutate the issue ledgers or active batch from an implementer lane.

## Validation

Run the smallest exact reproduction at least twice when safe, record its exit code and output digest, and run:

```bash
python3 docs/development/tools/probe_validation_capabilities.py --json
python3 docs/development/tools/check_project_docs.py
```

If the candidate concerns a command-result envelope, validate schema, command identity, process/payload exit-code agreement, diagnostics, and next actions.

## Handoff

Return the completed intake fields, chosen disposition, exact evidence references, proposed severity/root-fix group, focused regression command, and the manager decision required. State explicitly when no CFB should be created.
