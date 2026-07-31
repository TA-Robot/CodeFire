#!/usr/bin/env python3
"""Reproduce recurrence-v2 record and manifest required-leaf measurements.

The count is deliberately a burden measurement, never an acceptance target.
One leaf is one required scalar in the selected JSON-schema closure.  Required
arrays contribute the configured number of items times the required leaves in
their item schema.  The shape is printed with the result so a reviewer can see
every variable-cardinality assumption.

This program does not classify a byte-bound value as semantically proved.
Obligation completeness, claim quality, validation selection and review
meaning remain human judgements even when the gate proves their exact bytes.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
RECORD_SCHEMA = ROOT / "docs/development/ai/schemas/review-recurrence-control.v2.schema.json"
MANIFEST_SCHEMA = (
    ROOT
    / "docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json"
)


@dataclass(frozen=True)
class Shape:
    name: str
    description: str
    record_arrays: dict[str, int]
    manifest_arrays: dict[str, int]


SHAPES = {
    "minimal": Shape(
        name="minimal",
        description=(
            "one changed path/surface/prior obligation/derivation/validation, "
            "bootstrap authority, eight indispensable artifacts, and one evidence "
            "reference in each nonempty evidence list"
        ),
        record_arrays={
            "/subject/changed_paths": 1,
            "/subject/predecessor_authority/artifact_refs": 1,
            "/artifacts": 8,
            "/path_reviews": 1,
            "/path_reviews/*/surfaces": 1,
            "/path_reviews/*/surfaces/*/evidence_ref_ids": 1,
            "/prior_obligations": 1,
            "/prior_obligations/*/observations/before_ref_ids": 1,
            "/prior_obligations/*/observations/after_ref_ids": 1,
            "/prior_obligations/*/observations/negative_control_ref_ids": 1,
            "/new_findings": 0,
            "/derivations": 1,
            "/derivations/*/evidence/ref_ids": 1,
            "/validations": 1,
        },
        manifest_arrays={
            "/prior_obligations": 1,
            "/prior_obligations/*/closure_validation_ids": 1,
            "/parity_closure_claims": 1,
            "/required_validations": 1,
            "/required_validations/*/argv": 1,
        },
    ),
    "realistic": Shape(
        name="realistic",
        description=(
            "the published comparison shape: eleven changed paths/surfaces, three "
            "prior obligations, one new finding, two derivations, five validations, "
            "seventeen artifacts; validation argv lengths model 4,4,2,2,2 tokens "
            "as an aggregate fourteen-token list burden"
        ),
        record_arrays={
            "/subject/changed_paths": 11,
            "/subject/predecessor_authority/artifact_refs": 1,
            "/artifacts": 17,
            "/path_reviews": 11,
            "/path_reviews/*/surfaces": 1,
            "/path_reviews/*/surfaces/*/evidence_ref_ids": 1,
            "/prior_obligations": 3,
            "/prior_obligations/*/observations/before_ref_ids": 1,
            "/prior_obligations/*/observations/after_ref_ids": 1,
            "/prior_obligations/*/observations/negative_control_ref_ids": 1,
            "/new_findings": 1,
            "/new_findings/*/observations/before_ref_ids": 1,
            "/new_findings/*/observations/after_ref_ids": 1,
            "/new_findings/*/observations/negative_control_ref_ids": 1,
            "/derivations": 2,
            "/derivations/*/evidence/ref_ids": 1,
            "/validations": 5,
        },
        manifest_arrays={
            "/prior_obligations": 3,
            "/prior_obligations/*/closure_validation_ids": 1,
            "/parity_closure_claims": 2,
            "/required_validations": 5,
            # The recursive counter applies one length per repeated row.  Add the
            # nine tokens beyond five one-token argv rows below, transparently.
            "/required_validations/*/argv": 1,
        },
    ),
}


def load_schema(path: Path) -> dict[str, Any]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError(f"schema root is not an object: {path}")
    return raw


def resolve_ref(root: dict[str, Any], ref: str) -> dict[str, Any]:
    if not ref.startswith("#/"):
        raise ValueError(f"only local schema references are supported: {ref}")
    value: Any = root
    for token in ref[2:].split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        value = value[token]
    if not isinstance(value, dict):
        raise ValueError(f"schema reference is not an object: {ref}")
    return value


def required_leaves(
    node: dict[str, Any],
    root: dict[str, Any],
    arrays: dict[str, int],
    path: str = "",
) -> int:
    if "$ref" in node:
        return required_leaves(resolve_ref(root, node["$ref"]), root, arrays, path)
    if node.get("type") == "object" or "properties" in node:
        properties = node.get("properties", {})
        required = node.get("required", [])
        if not isinstance(properties, dict) or not isinstance(required, list):
            raise ValueError(f"malformed object schema at {path or '/'}")
        return sum(
            required_leaves(
                properties[name], root, arrays, f"{path}/{name}"
            )
            for name in required
        )
    if node.get("type") == "array":
        if path not in arrays:
            raise ValueError(f"shape has no array cardinality for {path}")
        count = arrays[path]
        if count < 0:
            raise ValueError(f"negative array cardinality for {path}: {count}")
        if count == 0:
            return 0
        items = node.get("items")
        if not isinstance(items, dict):
            raise ValueError(f"array has no item schema at {path}")
        return count * required_leaves(items, root, arrays, f"{path}/*")
    # const, enum, a scalar type, or a nullable scalar all occupy one leaf.
    return 1


def top_level_breakdown(
    schema: dict[str, Any], arrays: dict[str, int]
) -> dict[str, int]:
    properties = schema.get("properties", {})
    required = schema.get("required", [])
    return {
        name: required_leaves(properties[name], schema, arrays, f"/{name}")
        for name in required
    }


def measure(shape: Shape) -> dict[str, Any]:
    record_schema = load_schema(RECORD_SCHEMA)
    manifest_schema = load_schema(MANIFEST_SCHEMA)
    record = top_level_breakdown(record_schema, shape.record_arrays)
    manifest = top_level_breakdown(manifest_schema, shape.manifest_arrays)
    # Five realistic argv rows contain fourteen tokens in total.  The uniform
    # recursive shape accounts for five; this explicit, labelled delta accounts
    # for the remaining nine without pretending all rows have the same argv.
    argv_delta = 9 if shape.name == "realistic" else 0
    if argv_delta:
        manifest["required_validations"] += argv_delta
    record_total = sum(record.values())
    manifest_total = sum(manifest.values())
    return {
        "measurement": "required scalar leaves; no acceptance threshold or target",
        "shape": shape.name,
        "description": shape.description,
        "record": {"total": record_total, "by_required_property": record},
        "manifest": {"total": manifest_total, "by_required_property": manifest},
        "combined_total": record_total + manifest_total,
        "array_cardinalities": {
            "record": shape.record_arrays,
            "manifest": shape.manifest_arrays,
            "realistic_manifest_argv_extra_tokens": argv_delta,
        },
        "semantic_boundary": (
            "Schema consistency and Git/blob identity do not prove obligation "
            "completeness, parity/closure claim quality, validation selection, "
            "oracle quality, or the meaning of human-authored review text."
        ),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shape", choices=sorted(SHAPES), action="append")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    names = args.shape or list(SHAPES)
    print(json.dumps([measure(SHAPES[name]) for name in names], indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
