#!/usr/bin/env python3
"""Derived semantics and the integration decision for the recurrence gate v2.

One responsibility: decide.  Every outcome here is derived from the parsed
record, the frozen manifest, the verified chain and the committed timed
evidence — never transcribed by an author.  There is no count target, minimum,
field-count threshold or repair-size threshold anywhere in this module: the only
coverage rule is exact set equality with the Git-derived changed-path set.

`CLOSED_PROVEN` with `next_action: NONE` is the single nonblocking state, and
only when closure is separately proven.  Routing changes the kind of successor
work, never integration authority.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from recurrence_v2_authority import ChainResult
from recurrence_v2_evidence import TimedFacts
from recurrence_v2_model import (
    EVIDENCE_STATES,
    MANAGER_DECISION_ACTIONS,
    NEXT_ACTIONS,
    NONBLOCKING_PAIR,
    STATE_ROUTES,
    Finding,
    ManifestModel,
    RecordModel,
    duplicates,
    error,
)
from recurrence_v2_topology import TopologyFacts


@dataclass(frozen=True)
class RowOutcome:
    row_id: str
    section: str
    evidence_state: str
    next_action: str
    continuity: str
    closure_proven: bool
    blocking: bool
    reasons: tuple[str, ...]


@dataclass
class Decision:
    """Structural findings and integration blockers are separate channels.

    A structurally valid record that honestly records a reproduced defect, a
    failed validation, unavailable derivation evidence, a scope referral or a
    `request_changes` decision has no finding and still cannot authorize
    integration.  Conflating the two would let "valid" read as "approved".
    """

    findings: list[Finding] = field(default_factory=list)
    blockers: list[str] = field(default_factory=list)
    rows: list[RowOutcome] = field(default_factory=list)
    metrics: dict[str, Any] = field(default_factory=dict)
    successor_obligation_ids: tuple[str, ...] = ()
    product_repair_required: bool = False
    blocking_rows: int = 0
    scope_referrals: int = 0


def _observation_consistency(
    where: str,
    negative_control: str,
    control_refs: tuple[str, ...],
    findings: list[Finding],
) -> None:
    """An `unavailable` state must agree with its own observations."""

    if negative_control == "unavailable" and control_refs:
        findings.append(
            error(
                "unavailable-state-with-evidence",
                f"{where}.observations",
                "an unavailable negative control cannot bind evidence references",
            )
        )
    if negative_control in {"killed", "survived"} and not control_refs:
        findings.append(
            error(
                "observed-control-without-evidence",
                f"{where}.observations",
                f"a {negative_control} negative control needs exact evidence",
            )
        )


def _closure_proven(
    where: str,
    row: Any,
    section: str,
    manifest: ManifestModel | None,
    timed: dict[str, TimedFacts],
    chain: ChainResult,
    findings: list[Finding],
) -> tuple[bool, tuple[str, ...]]:
    """Derive whether `CLOSED_PROVEN` is admissible for this row."""

    reasons: list[str] = []
    if section == "new_findings":
        findings.append(
            error(
                "same-generation-closure-of-new-finding",
                where,
                "a finding with no pre-repair manifest mapping cannot be closed in the "
                "generation that found it",
            )
        )
        return False, ("no pre-repair manifest mapping",)
    mapping = (
        manifest.obligation_map().get(row.id, frozenset())
        if manifest is not None
        else frozenset()
    )
    if mapping:
        missing = sorted(identifier for identifier in mapping if identifier not in timed)
        failing = sorted(
            identifier
            for identifier in mapping
            if identifier in timed and not timed[identifier].passing
        )
        if missing:
            findings.append(
                error(
                    "closure-validation-missing",
                    where,
                    f"{row.id} is CLOSED_PROVEN but {missing} have no bound timed v2 result",
                )
            )
            reasons.append(f"missing mapped results {missing}")
        if failing:
            findings.append(
                error(
                    "closure-validation-not-passing",
                    where,
                    f"{row.id} is CLOSED_PROVEN but {failing} did not pass on a clean checkout "
                    "of the bound candidate",
                )
            )
            reasons.append(f"nonpassing mapped results {failing}")
        if row.non_reachability_ref is not None:
            findings.append(
                error(
                    "unexpected-non-reachability-boundary",
                    f"{where}.non_reachability_ref",
                    "a mapped closure does not use the injection-only alternative",
                )
            )
        proven = not missing and not failing
        if proven:
            reasons.append(f"mapped validations passed: {sorted(mapping)}")
        return proven, tuple(reasons)

    # Empty mapping: the exemption exists only if the predecessor v2 record
    # reached by the rooted chain holds this obligation in DEFECT_INJECTION_ONLY.
    # No manifest or record field can assert it, and a bootstrap generation has
    # no predecessor v2 record at all.
    link = chain.predecessor_link
    predecessor_state = link.obligation_states.get(row.id) if link is not None else None
    if predecessor_state != "DEFECT_INJECTION_ONLY":
        findings.append(
            error(
                "empty-closure-mapping-without-injection-predecessor",
                where,
                f"{row.id} has no closure validation and its predecessor state is "
                f"{predecessor_state!r}; the exemption is read only from a "
                "DEFECT_INJECTION_ONLY predecessor row reached by the rooted chain",
            )
        )
        return False, ("empty mapping without an injection-only predecessor",)
    if row.non_reachability_ref is None:
        findings.append(
            error(
                "injection-closure-without-non-reachability-boundary",
                where,
                f"{row.id} may close only by binding the independently accepted "
                "non-reachability boundary",
            )
        )
        return False, ("no bound non-reachability boundary",)
    reasons.append(
        "injection-only predecessor plus a bound non-reachability boundary; the gate attests "
        "that artifact's bytes, role and chronology only"
    )
    return True, tuple(reasons)


def derive(
    record: RecordModel,
    *,
    manifest: ManifestModel | None,
    chain: ChainResult,
    timed: dict[str, TimedFacts],
    topology: TopologyFacts | None,
    predecessor_path_present: dict[str, bool],
) -> Decision:
    decision = Decision()
    findings = decision.findings

    # Derived completeness: a bootstrap authority is always unavailable and can
    # never produce ALLOW; a rooted prior record is verified.
    expected_completeness = (
        "unavailable"
        if record.subject.authority.kind == "legacy_bootstrap_report"
        else "verified"
    )
    if record.predecessor_obligation_completeness != expected_completeness:
        findings.append(
            error(
                "predecessor-completeness-mismatch",
                "record.predecessor_obligation_completeness",
                f"derived {expected_completeness} from the resolved authority kind "
                f"{record.subject.authority.kind}",
            )
        )
    if record.subject.authority.kind == "legacy_bootstrap_report":
        if record.subject.authority.bootstrap_arm is None:
            findings.append(
                error(
                    "bootstrap-arm-missing",
                    "record.subject.predecessor_authority.bootstrap_arm",
                    "a legacy bootstrap authority names its arm",
                )
            )
    elif record.subject.authority.bootstrap_arm is not None:
        findings.append(
            error(
                "bootstrap-arm-on-prior-record-authority",
                "record.subject.predecessor_authority.bootstrap_arm",
                "a prior v2 record authority has no bootstrap arm",
            )
        )

    # Exact changed-path coverage, with one or more scope-dispositioned surface
    # per path.  A surface has no path of its own, so it cannot cover another.
    declared_paths = frozenset(review.path for review in record.path_reviews)
    repeated_paths = duplicates([review.path for review in record.path_reviews])
    if repeated_paths:
        findings.append(
            error("duplicate-path-review", "record.path_reviews", repr(repeated_paths))
        )
    if declared_paths != record.subject.changed_paths:
        findings.append(
            error(
                "path-review-coverage-mismatch",
                "record.path_reviews",
                f"review-only={sorted(declared_paths - record.subject.changed_paths)}, "
                f"changed-only={sorted(record.subject.changed_paths - declared_paths)}",
            )
        )
    surface_ids: list[str] = []
    for index, review in enumerate(record.path_reviews):
        where = f"record.path_reviews[{index}]"
        for surface_index, surface in enumerate(review.surfaces):
            surface_where = f"{where}.surfaces[{surface_index}]"
            surface_ids.append(surface.surface_id)
            if surface.scope_disposition == "referred_to_manager":
                decision.scope_referrals += 1
                if surface.scope_routing_ref is None:
                    findings.append(
                        error(
                            "scope-referral-without-decision-artifact",
                            f"{surface_where}.scope_routing_ref",
                            "a surface referred to the manager binds the routing artifact",
                        )
                    )
                decision.blockers.append(
                    f"{surface_where}: surface {surface.surface_id} is referred to the manager; "
                    "a scope referral routes manager work, never integration authority, and is "
                    "never a silent out-of-scope escape"
                )
            elif surface.scope_routing_ref is not None:
                findings.append(
                    error(
                        "unexpected-scope-routing-artifact",
                        f"{surface_where}.scope_routing_ref",
                        "an in-scope reviewed surface does not carry a manager routing decision",
                    )
                )
    repeated_surfaces = duplicates(surface_ids)
    if repeated_surfaces:
        findings.append(
            error("duplicate-surface-id", "record.path_reviews", repr(repeated_surfaces))
        )

    # Prior obligations and new findings: I5 routing, continuity and closure.
    prior_ids = [row.id for row in record.prior_obligations]
    new_ids = [row.id for row in record.new_findings]
    repeated_rows = duplicates(prior_ids + new_ids)
    if repeated_rows:
        findings.append(
            error("duplicate-row-id", "record", repr(repeated_rows))
        )
    link = chain.predecessor_link
    predecessor_states = dict(link.obligation_states) if link is not None else {}
    predecessor_origins = dict(link.obligation_origins) if link is not None else {}
    state_counts = {state: 0 for state in EVIDENCE_STATES}
    action_counts = {action: 0 for action in NEXT_ACTIONS}
    unavailable_rows = 0
    for where, row in record.rows():
        section = "prior_obligations" if ".prior_obligations[" in where else "new_findings"
        state = row.evidence_state
        action = row.next_action
        if state in state_counts:
            state_counts[state] += 1
        if action in action_counts:
            action_counts[action] += 1
        permitted = STATE_ROUTES.get(state)
        if permitted is None:
            findings.append(
                error("unknown-evidence-state", f"{where}.evidence_state", repr(state))
            )
        elif action not in permitted:
            findings.append(
                error(
                    "next-action-not-routed-for-state",
                    f"{where}.next_action",
                    f"{state} routes to {sorted(permitted)}, not {action!r}",
                )
            )
        _observation_consistency(
            where,
            row.observations.negative_control,
            row.observations.negative_control_ref_ids,
            findings,
        )
        if row.observations.negative_control == "unavailable":
            unavailable_rows += 1
        if action in MANAGER_DECISION_ACTIONS and row.routing_decision_ref is None:
            findings.append(
                error(
                    "manager-routing-without-decision-artifact",
                    f"{where}.routing_decision_ref",
                    f"{action} retains an exact pre-candidate routing artifact",
                )
            )
        if action not in MANAGER_DECISION_ACTIONS and row.routing_decision_ref is not None:
            findings.append(
                error(
                    "unexpected-routing-decision-artifact",
                    f"{where}.routing_decision_ref",
                    "only a manager scope or requirement decision retains a routing artifact",
                )
            )
        continuity = "not_applicable"
        if section == "prior_obligations":
            predecessor_state = predecessor_states.get(row.id)
            if predecessor_state is None:
                continuity = "predecessor_state_unavailable"
            elif predecessor_state == "CLOSED_PROVEN" and state != "CLOSED_PROVEN":
                continuity = "recurred_after_proven_closure"
            elif state == "CLOSED_PROVEN":
                continuity = "closed_this_generation"
            else:
                continuity = "continuous_open"
            expected_origin = predecessor_origins.get(row.id)
            if expected_origin is not None and row.origin_generation != expected_origin:
                findings.append(
                    error(
                        "origin-generation-mismatch",
                        f"{where}.origin_generation",
                        f"the predecessor record places {row.id} at generation {expected_origin}",
                    )
                )
            if row.origin_generation >= record.generation:
                findings.append(
                    error(
                        "non-prior-origin-generation",
                        f"{where}.origin_generation",
                        f"origin {row.origin_generation} must precede {record.generation}",
                    )
                )
            if continuity == "recurred_after_proven_closure":
                if row.closure_evidence_ref is None:
                    findings.append(
                        error(
                            "recurrence-without-prior-closure-evidence",
                            f"{where}.closure_evidence_ref",
                            f"{row.id} recurred after proven closure and must bind the exact "
                            "prior closure evidence",
                        )
                    )
            elif row.closure_evidence_ref is not None:
                findings.append(
                    error(
                        "unexpected-prior-closure-evidence",
                        f"{where}.closure_evidence_ref",
                        f"derived continuity is {continuity}; prior closure evidence is bound "
                        "only for a recurrence after proven closure",
                    )
                )
        closure_proven = False
        reasons: tuple[str, ...] = ()
        if state == "CLOSED_PROVEN":
            closure_proven, reasons = _closure_proven(
                where, row, section, manifest, timed, chain, findings
            )
            if state == "CLOSED_PROVEN" and row.observations.negative_control not in {
                "killed",
                "not_applicable",
            }:
                findings.append(
                    error(
                        "closure-without-negative-control",
                        f"{where}.observations.negative_control",
                        f"{row.observations.negative_control!r} cannot support a proven closure",
                    )
                )
                closure_proven = False
            if not row.observations.before_ref_ids or not row.observations.after_ref_ids:
                findings.append(
                    error(
                        "closure-observation-without-evidence",
                        f"{where}.observations",
                        "a proven closure binds both the before and the after observation",
                    )
                )
                closure_proven = False
        elif section == "prior_obligations" and row.non_reachability_ref is not None:
            findings.append(
                error(
                    "non-reachability-boundary-on-open-row",
                    f"{where}.non_reachability_ref",
                    "an injection-only row is recorded as CLOSED_PROVEN once its boundary is "
                    "independently accepted; it is never silently de-blocked while keeping the "
                    "defect state",
                )
            )
        blocking = not (
            (state, action) == NONBLOCKING_PAIR and closure_proven
        )
        if blocking:
            decision.blocking_rows += 1
        decision.rows.append(
            RowOutcome(
                row_id=row.id,
                section=section,
                evidence_state=state,
                next_action=action,
                continuity=continuity,
                closure_proven=closure_proven,
                blocking=blocking,
                reasons=reasons,
            )
        )
    if link is not None:
        exported = frozenset(link.successor_obligation_ids)
        declared = frozenset(prior_ids)
        if exported != declared:
            findings.append(
                error(
                    "successor-obligation-set-mismatch",
                    "record.prior_obligations",
                    f"predecessor exported {sorted(exported)}, this record carries "
                    f"{sorted(declared)}",
                )
            )

    # I1: manifest-claim scoped predecessor derivation.
    unavailable_derivations = 0
    for index, derivation in enumerate(record.derivations):
        where = f"record.derivations[{index}]"
        present = predecessor_path_present.get(derivation.path)
        if present is not None and present != derivation.locus_existed_in_predecessor:
            findings.append(
                error(
                    "predecessor-locus-existence-contradicted",
                    f"{where}.locus_existed_in_predecessor",
                    f"{derivation.path} "
                    + ("exists" if present else "does not exist")
                    + " in the predecessor tree",
                )
            )
        if derivation.evidence_state == "unavailable":
            unavailable_derivations += 1
            if derivation.evidence_ref_ids:
                findings.append(
                    error(
                        "unavailable-derivation-with-evidence",
                        f"{where}.evidence",
                        "an unavailable derivation cannot bind evidence references",
                    )
                )
            if derivation.disposition != "derivation_unavailable":
                findings.append(
                    error(
                        "unavailable-derivation-disposition-mismatch",
                        f"{where}.disposition",
                        repr(derivation.disposition),
                    )
                )
            decision.blockers.append(
                f"{where}: derivation {derivation.claim_id} has unavailable evidence; it stays in "
                "the denominator and is fail-closed for integration"
            )
        else:
            if not derivation.evidence_ref_ids:
                findings.append(
                    error(
                        "available-derivation-without-evidence",
                        f"{where}.evidence.ref_ids",
                        "an available derivation binds exact evidence",
                    )
                )
            if derivation.disposition == "derivation_unavailable":
                findings.append(
                    error(
                        "available-derivation-marked-unavailable",
                        f"{where}.disposition",
                        "disposition contradicts the available evidence state",
                    )
                )
        if derivation.disposition == "derivation_contradicted":
            decision.blockers.append(
                f"{where}: derivation {derivation.claim_id} contradicts the predecessor behaviour"
            )
    if manifest is not None:
        derivation_by_claim = {row.claim_id: row for row in record.derivations}
        for claim in manifest.claims:
            if claim.obligation_id is None:
                continue
            row = derivation_by_claim.get(claim.claim_id)
            if row is None:
                findings.append(
                    error(
                        "obligation-claim-without-derivation",
                        "record.derivations",
                        f"obligation {claim.obligation_id} declares claim {claim.claim_id} with "
                        "no derivation disposition",
                    )
                )

    # Terminal review state.
    terminal = record.terminalization
    if (
        terminal.state != "complete"
        or terminal.worker_alive
        or terminal.child_processes_active != 0
    ):
        decision.blockers.append(
            "record.terminalization: integration needs a complete review, a stopped worker and "
            "zero active child processes"
        )

    # Timed evidence must pass; the manifest defines the required set, so every
    # bound row is required.
    for validation_id, facts in sorted(timed.items()):
        if not facts.passing:
            decision.blockers.append(
                f"record.validations[{validation_id}]: the committed timed v2 result is not a "
                "pass with exit code zero, no timeout, a complete log and empty clean-before, "
                "clean-after and index-flag states"
            )
    if manifest is not None:
        missing_results = sorted(
            set(manifest.validation_argv()) - set(timed)
        )
        if missing_results:
            findings.append(
                error(
                    "manifest-required-validation-unbound",
                    "record.validations",
                    f"{missing_results} have no bound committed timed v2 result",
                )
            )

    if record.review_decision != "accept":
        decision.blockers.append(
            f"record.review_decision: {record.review_decision!r} is a structurally valid record "
            "and still blocks integration"
        )

    for row in decision.rows:
        if row.blocking:
            decision.blockers.append(
                f"record.{row.section}: {row.row_id} is {row.evidence_state} routed to "
                f"{row.next_action}; only CLOSED_PROVEN with NONE and proven closure is "
                "nonblocking"
            )
    if record.predecessor_obligation_completeness == "unavailable":
        decision.blockers.append(
            "record.predecessor_obligation_completeness: legacy Markdown completeness is not "
            "machine-decidable, so a bootstrap generation retains unavailable in the denominator "
            "and cannot produce machine ALLOW; its retained record seeds the next generation"
        )
    decision.successor_obligation_ids = tuple(sorted(set(prior_ids) | set(new_ids)))
    decision.product_repair_required = any(
        row.next_action == "PRODUCT_REPAIR" for row in decision.rows
    )
    decision.metrics = {
        "i1": {
            "manifest_claims": len(manifest.claims) if manifest is not None else None,
            "derivation_rows": len(record.derivations),
            "rows_unavailable": unavailable_derivations,
            "changed_surface_coverage": {
                "changed_paths": len(record.subject.changed_paths),
                "paths_with_surface": sum(
                    1 for review in record.path_reviews if review.surfaces
                ),
                "surfaces": sum(len(review.surfaces) for review in record.path_reviews),
            },
        },
        "i5": {
            "evidence_state_counts": state_counts,
            "next_action_counts": action_counts,
            "blocking_rows": decision.blocking_rows,
            "unavailable_negative_control_rows": unavailable_rows,
            "product_repair_required": decision.product_repair_required,
            "scope_referrals": decision.scope_referrals,
            "comparison_note": (
                "product-repair generations avoided is measured across reviewed generations; it "
                "is deliberately not the retroactive replay's 6 / 18 candidate-generation "
                "removal metric and must not be compared as though it were"
            ),
        },
        "repair_surface": {
            "paths": len(topology.repair_surface) if topology is not None else None,
            "inserted_lines": topology.inserted_lines if topology is not None else None,
            "deleted_lines": topology.deleted_lines if topology is not None else None,
            "binary_paths": list(topology.binary_paths) if topology is not None else [],
            "repair_commits": len(topology.repair_commits) if topology is not None else None,
            "measurement_only": (
                "the delta is emitted as a measurement; there is no line, file, surface or "
                "repair-size limit"
            ),
        },
        "obligations": {
            "prior": len(record.prior_obligations),
            "new": len(record.new_findings),
            "closed_proven": sum(
                1 for row in decision.rows if row.closure_proven and not row.blocking
            ),
            "predecessor_obligation_completeness": (
                record.predecessor_obligation_completeness
            ),
        },
    }
    return decision
