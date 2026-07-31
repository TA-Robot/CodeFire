# Successor Review Recurrence Pilot

This directory holds final manager-owned records for the interim AIA-047
pilot. Do not add empty, example, or pre-approved records: a record is written
only after an exact independent reviewer has stopped and committed its report.

## Immediate application

The pilot is mandatory for the next successor review of:

- the AIA-046 source-keyed Cargo gate repair;
- the development-flow KGI/KPI repair.

Those reviews must not start from an informal “check the previous comments”
prompt. Their repair packets freeze the prior obligations, allowed boundary,
required production routes, anticipated regressions, forbidden shortcuts,
and planned new surfaces. The final record uses
`../schemas/review-recurrence-control.v1.schema.json`.

## Ownership and sequence

1. The manager content-addresses the repair packet before implementation.
2. The implementer returns one exact candidate and identifies every actual
   surface added by the repair, including anything not forecast.
3. An independent report-only reviewer checks prior closure and repair-created
   surface as separate passes. It records observations, effect episodes, and
   causal classes without a finding-count target.
4. The manager waits for terminal worker/process state, recomputes candidate,
   control-overlay, task-packet, report, patch, path, tree, numstat, and digest
   identity, then writes the final record.
5. `check_review_recurrence.py` emits the integration decision. A structurally
   valid `request_changes` record is still `BLOCK`, and schema-only validation
   never authorizes integration.

Nonblocking adjacent work may leave the current repair only through a named
owner, release condition, and manager route authority. A blocking finding,
unreviewed repair-created surface, blocked expansion, missing negative
control, nonterminal validation, live reviewer process, or unavailable
required evidence cannot produce `ALLOW`.

## Why there is no finding quota

A target such as 100 findings rewards duplicate wording and low-value list
filling. This pilot instead uses an obligation-and-surface saturation rule:
all declared prior obligations, production routes, actual repair surfaces,
negative controls, and unavailable evidence must have a disposition. Counts
remain diagnostic outcomes only. Observation count and distinct effect count
are both retained so duplication is visible rather than rewarded.

## Evaluation and promotion

For each Cargo and KGI run, compare:

- review generation and final decision;
- prior obligations closed, left open, or formally routed;
- candidate-created findings and distinct effect episodes;
- planned versus unplanned repair surfaces;
- repair service time, independent review service time, and
  verdict-to-committed-report time;
- unavailable evidence and repeated observations of one effect;
- operator time needed to produce and validate the record.

The pilot is not evidence that the bottleneck is solved. It may become the
default only if the two comparable runs show useful recurrence capture
without quota noise, materially duplicated reporting, false closure, or
integration delay, and a blind causal classification (scheduled once the
Fugu window reopens on or after 2026-08-03) agrees with the manager
classification. Revise or remove the gate if its burden exceeds avoided
rework or if a known recurrence passes it.
