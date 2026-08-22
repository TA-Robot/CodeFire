# AI Task Packet

Status: ready

Packet ID: MS-ecosystem-conformance-profile

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Select the smallest existing public contract profile that can honestly receive a maturity label and
bounded conformance corpus for one independent client. Freeze its version, schema/vector set,
extension namespace/critical behavior, compatibility claims, stop rules, and non-claims. Do not
change schemas or author a client.

## Scope

- read-only classification of current public schemas, support/capabilities, conformance fixtures,
  canonical JSON/IDs, version and extension behavior;
- one maturity-labelled profile and bounded corpus candidate;
- this packet only.

## Required Classification

1. Compare current public profile candidates and select exactly one smallest profile with real client value.
2. Define maturity label semantics and what it explicitly does not claim.
3. Freeze exact schema IDs/files, positive/negative vectors, checksums, provenance, unknown-version,
   unknown-critical, and noncritical-extension decisions.
4. Define extension namespace/critical rules and DENY/UNKNOWN versus ALLOW/CLEAN preservation.
5. Define client pass/fail/falsifier and ordered adopt / narrow / stop thresholds before client work.
6. Identify any stale docs/schema/fixture mismatch and classify it `block-m1`, `park`, or `drop`.
7. Classify CFR-171..173 and prior milestone parked findings without pre-admitting them, including the
   product-value parks: six cited JSON identifiers absent from the handed public pack and an
   unconfigured README that can contradict a sealed requirement.

## Exact Write Set

- `docs/development/ai/task-packets/milestone-ecosystem-conformance-profile.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- schema/product/source/test/public-doc edits, corpus generation, or client authoring;
- external organization, standards body, IPR/trademark/governance, release-cycle, or funding claims;
- network publication, Git bridge, telemetry, hosted service, agent platform, or M2–M5 work.

## Handoff

Return exact profile/maturity label/non-claims, schema/vector/checksum set, extension/version decisions,
falsifiers, ordered thresholds, issue classification, first mismatch, `block-m1`, validation, and commit.
Commit only this packet.

