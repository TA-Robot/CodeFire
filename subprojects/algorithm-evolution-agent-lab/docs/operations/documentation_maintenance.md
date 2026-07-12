# Documentation Maintenance Guide

This guide describes how to keep documentation accurate as the algorithm evolution lab grows.

The project should not accumulate disconnected Markdown. Each document needs a role, an owner area, and a maintenance trigger.

## Documentation Layers

| Layer | Directory | Primary Reader | Maintenance Trigger |
|---|---|---|---|
| product/spec | `docs/spec/` | planner or implementer | behavior requirement changes |
| design | `docs/design/` | implementer or reviewer | module responsibility changes |
| operations | `docs/operations/` | human or AI operator | workflow changes |
| experiments | `docs/experiments/` | researcher or reviewer | experiment execution |
| development | `docs/development/` | maintainer | roadmap, history, source shape, or process changes |

## When To Update Docs

Update documentation in the same branch when:

- a new module family appears
- an existing workflow gains or loses a step
- a verification command changes
- a safety or evidence gate changes
- a new experiment record pattern is used
- a CodeFire workaround becomes part of normal operation
- a naming or traceability convention changes

## When Not To Update Docs

Do not update docs just to restate code details that are already obvious from a short function.

Avoid:

- copying function bodies into prose
- generic machine learning tutorials
- speculative roadmaps without owners or triggers
- outdated examples that cannot be run
- navigation links to documents that do not exist

## Required Metadata For New Guides

Every new operations guide should answer:

- What workflow does this guide support?
- Who should read it?
- What are the inputs?
- What are the outputs?
- What commands should be run?
- What are the completion criteria?
- What common mistakes should be avoided?

## Review Checklist

Before committing a documentation change:

- The document has a clear title.
- The entry point links to it.
- The content names concrete project files or commands.
- The guidance does not contradict `AGENTS.md`.
- The guidance does not bypass CodeFire.
- The guidance does not make unevidenced research claims.
- The document has enough structure for skim reading.

## Line Count Policy

The documentation doubling initiative uses line and word counts as measurable progress indicators.

Counts are not a substitute for quality. A batch should increase line count only by adding one of:

- missing workflow steps
- concrete checklists
- examples
- decision tables
- troubleshooting guidance
- ownership and maintenance rules

## Navigation Maintenance

When adding a document:

1. Link it from the nearest directory `README.md`.
2. If it changes operation, link it from `docs/operations/README.md`.
3. If it changes source orientation, update `docs/development/source-code-atlas.md`.
4. If it changes long-horizon plans, update `docs/development/implementation-roadmap.md`.
5. Record durable documentation work in `docs/development/history.md`.

## Staleness Review

Run a staleness pass when:

- a module is renamed
- a CodeFire workflow issue is fixed
- a new verification command becomes standard
- an experiment workflow changes
- the branch workflow changes

Staleness pass checklist:

- search for old command names
- search for old branch paths
- check every operations link
- confirm test command still runs
- confirm CodeFire command examples still match the CLI

