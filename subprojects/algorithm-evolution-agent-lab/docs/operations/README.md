# Operations Handbook

This directory contains operator-facing guides for running the algorithm evolution lab without opening the implementation first.

Use these documents as the first stop when deciding how to plan, run, review, or publish experiment work.

## Guide Map

| Guide | Use When |
|---|---|
| [experiment_lifecycle.md](experiment_lifecycle.md) | Planning or executing a research experiment from objective to archived result |
| [research_cycle_planning.md](research_cycle_planning.md) | Turning retrospective findings into the next cycle's active/review/deferred lanes |
| [evidence_and_claim_review.md](evidence_and_claim_review.md) | Preparing evidence, review objections, limitations, or SOTA claim release checks |
| [codefire_branch_workflow.md](codefire_branch_workflow.md) | Using CodeFire branch-per-feature workflow for this project |
| [identifier_naming_audit.md](identifier_naming_audit.md) | Running the identifier naming audit before review |
| [agent_execution_playbook.md](agent_execution_playbook.md) | Giving a human or AI agent a bounded task and checking that the work closed cleanly |
| [documentation_maintenance.md](documentation_maintenance.md) | Deciding when and how to update docs, examples, indexes, and history |
| [runbook_catalog.md](runbook_catalog.md) | Looking up a focused operating procedure for branch work, experiment governance, evidence review, or docs closeout |

## Operating Principles

- Keep experiments small enough to rerun.
- Treat failed experiments as durable research data.
- Separate hypotheses, plans, runs, evidence, reviews, and claims.
- Do not write SOTA claims until benchmarks, baselines, repeated results, and release gates are complete.
- Use CodeFire to preserve trace links between requirements, design, code, tests, and documentation decisions.
- Treat documentation growth as a planned maintenance stream, not incidental cleanup after code changes.
