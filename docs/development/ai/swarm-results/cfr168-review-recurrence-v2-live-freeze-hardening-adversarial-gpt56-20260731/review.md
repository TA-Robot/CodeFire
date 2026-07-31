# CFR-168 Review Recurrence v2 Live-Freeze Hardening — Final Adversarial Preflight

Reviewer: GPT-5.6 (Codex), independent adversarial report-only lane
`cfr168-review-recurrence-v2-live-freeze-hardening-adversarial-gpt56-20260731`.
Date: 2026-07-31.

This review evaluates the final frozen repair packet, not an implementation.
The lane did not modify schemas, tools, tests, live manifests, anchors, records,
product code, or earlier review evidence. Its only write is this report.

## Exact review identity

| Item | Exact value |
|---|---|
| Target commit | `65beb73dd2632cca69eec44354eadca2d9f53b2f` |
| Target tree | `1d1303a6e1fb28e8991724633e284b0eeb951678` |
| Target parent | `c7ebf5387a7074d55ecd6e067adabf8b5c4eaddd` |
| Target subject | `docs(cfr168): resolve live hardening implementation choices` |
| Packet | `docs/development/ai/task-packets/cfr168-review-recurrence-v2-live-freeze-hardening-20260731.md` |
| Packet SHA-256 | `05ea8c55355ba8674d18be04c0792de2990cdcd981de35c68790ff2c632af20f` |
| Packet-declared implementation base | `00ad88562068a6101a7530b84d5f52b0cff559f0` |

The isolated review worktree was clean at the target. From the prior accepted
adversarial target `fabb77ae44ee95411b5505a90366d031f4b2a6cd` to this target,
the only changed paths are the packet and its earlier primary-preflight report;
no executable gate code changed. I therefore rechecked the full packet and the
four new feasibility clauses, then mapped them back across every earlier
boundary finding.

## Disposition

Unresolved findings: **0**.

The four additions are executable with the allowed standard-library and
repository surfaces. They close implementer choices without weakening or
contradicting the previously accepted current/retained boundary rules. This is
an acceptance of the packet for implementation, not an assertion that the
unchanged pre-repair implementation already satisfies it.

## Four-resolution adversarial check

### 1. Exact I1 claim coverage and derivation routing metrics

**Accepted.** Every required coverage value has one existing authoritative
source: manifest `ManifestClaim` supplies `claim_id`, `kind`, nullable
`obligation_id`, and `locus_path`; record `DerivationRow` supplies the semantic
and evidence fields; the new F1 path boolean supplies the separate Git fact.
The identifier schemas restrict claim and evidence IDs to ASCII, so ordinary
Python sorting implements the packet's ASCII order exactly. `NEXT_ACTIONS`
already contains the eight names now required as zero-preserving count keys.

The result can be built in `recurrence_v2_decision.py`, closed in the new result
schema, and compared by complete retained-result replay. `blocking_derivations`
is independently derivable as the number of coverage rows whose action is not
`NONE`; it does not reuse the obligation-row product-repair metric. This closes
the prior parsed-but-discarded `kind` and `next_action` surfaces and does not
reintroduce last-write-wins behavior because F1c requires duplicate claim IDs
to fail before the manifest/record join.

Required implementation controls remain concrete: shuffled input must produce
the same sorted coverage; every action key, including zero counts, must be
present; a retained action/count/coverage mutation must block; and a non-`NONE`
derivation must increment `blocking_derivations` and remain blocking.

### 2. Exact Cargo and KGI/KPI bootstrap origin maps

**Accepted.** I independently parsed the frozen KGI/KPI twenty-row inventory.
It contains exactly the packet's six generation-2 IDs and fourteen
generation-3 IDs. There are no omitted or extra rows. The Cargo `R1` versus
`R2`–`R5` split is also explicit.

The maps and first-live generations fit the manager-owned `LegacyRoot` model
extension, while manifest-to-record and successor-to-predecessor comparisons
remain in the evidence/decision path. The model, fixture, manifest schema, and
tests are all in the allowed write set. No live manifest or anchor must be
authored by this repair. A map typo, wrong live generation, record/manifest
disagreement, or successor origin drift remains a deterministic blocker.

### 3. Canonical UTC timestamps and lexical POSIX path replay

**Accepted.** The existing runner already emits six fractional digits and a
literal `Z`. A fixed ASCII regular expression followed by Python UTC datetime
parsing rejects alternate offsets/precision, invalid calendar values, and leap
second `:60` while preserving signed forward and backward wall-time
differences.

The path rule is implementable without inspecting the replay host filesystem:
validate the explicit `/` case, reject non-absolute/control/surrogate/backslash
and empty/dot/dot-dot component forms, require exact `PurePosixPath.as_posix()`
spelling, then compare `PurePosixPath.parts` prefixes. A prototype accepted
`/`, `/repo`, and `/repo/sub`; rejected `//repo`, `/repo/`, `/repo//x`,
`/repo/./x`, `/repo/../x`, and backslash/control forms; treated `/repo` as
containing itself and `/repo/sub`, but not `/repo2`. This directly exercises
the root special case and the component-prefix neighbor that a string-prefix
check would get wrong.

The runner may resolve live paths before emission; retained replay performs
only the frozen lexical check. Thus filesystem relocation or a later symlink
layout cannot reinterpret committed evidence. Cwd equality with the root stays
allowed, while log/result equality, in-root placement, aliases, and neighboring
string-prefix paths remain blocking. None of the earlier timing tolerances,
terminal relations, detail/summary derivations, or current/retained symmetry is
relaxed.

### 4. Closed `live_arm_pin` state object

**Accepted.** The repository's compact schema evaluator was directly exercised
with an object-or-null schema: null and the exact object passed; an extra object
member and a scalar failed. It supports the packet's `type: ["object", "null"]`
shape without adding `oneOf` or another dependency.

The outer object has one closed state enum and semantic validation supplies the
cross-field relation: `unavailable` iff both values are null, and `verified` iff
both exact closed objects are present. This represents diagnostic BLOCK output
without a string/object union while retaining exact F2 anchor and arm identity
for authorization and historical replay. Wrong state/null combinations,
partial objects, extra members, duplicate arms, generation mismatch, and
historical anchor-byte disagreement all remain blockers.

## Full boundary regression pass

| Previously established boundary | Final packet status |
|---|---|
| Git path fact versus human locus judgement; unsafe aliases and probe failures | Still closed by separate booleans, exact manifest path binding, and one shared current/history probe. The new I1 row exposes both facts rather than conflating them. |
| Bootstrap origin ambiguity | Strengthened by the complete named 20-row KGI/KPI map; no catch-all “other fourteen” remains. |
| Duplicate semantic IDs and raw JSON members | Unchanged: semantic keys reject before conversion; strict decoding rejects duplicate members before object construction. New metric joins rely on that uniqueness. |
| Lone surrogates, non-finite/exponent-overflow numbers, excessive nesting, and oversized integers | Unchanged shared structured boundaries cover current, retained, and validator-owned JSON. None of the four additions introduces a permissive decoder. |
| Timed detail/summary, HEAD, path, timing, and terminal contradictions | Strengthened by canonical timestamp and filesystem-independent path rules. Frozen tolerances and the admissible consistent backward-clock case remain intact. |
| Retained result field drift and unknown members | Unchanged closed result schema and complete replay now also own the exact I1 metrics and closed live-pin state. Named machine-local path exclusions remain the only exclusions. |
| Derivation disposition/action mismatch | Strengthened: the route table remains blocking and its exact action is now visible in per-claim coverage and counts. |
| Schema-invalid typed consumption and traceback classes | Unchanged staged decode/schema/typed-use rule. Object/null live-pin schema removes, rather than adds, an unsupported schema branch. |
| Live anchor duplicate/generation/identity ambiguity | Unchanged F2 schema and semantic checks; the result representation now has one closed shape for both states. |
| Negative-control versus timed-validation extraction | Unchanged F3 set rule; none of the feasibility resolutions moves expected-failure evidence into passing timed argv. |

No earlier executable contradiction, partial-authority path, or unstructured
failure class is reopened by the final delta.

## Validation evidence

| Check | Outcome |
|---|---|
| Exact target and packet digest | `65beb73dd2632cca69eec44354eadca2d9f53b2f`; packet SHA-256 matched `05ea8c55…af20f` |
| Compact-schema, timestamp, lexical-path, I1-field/action feasibility probes | `PASS` |
| Frozen KGI/KPI origin table versus packet map | `PASS (20/20)` |
| `python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | `AI acceleration docs check passed.` |
| `python3 docs/development/tools/check_project_docs.py` | `Project docs check passed; 669 Markdown files inventoried.` |
| `python3 docs/development/tools/check_ai_development.py` | All seven reported subchecks passed; `AI development gate passed.` |
| `git diff --check fabb77a..65beb73` | Passed |

The executable probes used `PYTHONDONTWRITEBYTECODE=1` where repository modules
were imported. No implementation suite result is claimed: the target changes a
packet and report only, and the packet deliberately describes work that has not
yet been implemented.

## Residual trust boundary

Semantic claim completeness, honest command execution, and mutate-then-restore
behavior remain explicitly outside the machine proof. The final delta neither
conceals nor widens those declared limits.

OVERALL_VERDICT: ACCEPT
