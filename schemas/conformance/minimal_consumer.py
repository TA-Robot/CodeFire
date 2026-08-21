#!/usr/bin/env python3
"""Independent stdlib-only reader for the CodeFire v1 conformance corpus."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")


def object_decision(value: dict[str, Any]) -> str:
    payload = value.get("payload")
    type_tag = value.get("type")
    if not isinstance(payload, dict) or not isinstance(type_tag, str):
        return "deny"
    digest = hashlib.sha256(type_tag.encode() + b"\0" + canonical_json(payload)).hexdigest()
    if value.get("hash") != digest:
        return "deny"
    object_id = value.get("object_id")
    if object_id not in {f"CF-POLICY-{digest[:12]}", f"CF-POLICY-{digest[:24]}"}:
        return "deny"
    extensions = value.get("extensions", [])
    if not isinstance(extensions, list) or len(extensions) > 128:
        return "deny"
    for extension in extensions:
        if not isinstance(extension, dict):
            return "deny"
        if not isinstance(extension.get("namespace"), str):
            return "deny"
        if not isinstance(extension.get("name"), str):
            return "deny"
        if not isinstance(extension.get("version"), int) or isinstance(extension.get("version"), bool):
            return "deny"
        if not isinstance(extension.get("critical"), bool):
            return "deny"
        if extension["critical"]:
            return "deny"
    return "accept"


def vector_decision(boundary: str, value: Any) -> str:
    if boundary == "remote_branch":
        if not isinstance(value, dict) or value.get("version") != 1:
            return "deny"
        return "accept"
    if boundary == "object_record":
        return object_decision(value)
    if boundary == "canonical_json":
        return "accept"
    return "deny"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    root = args.manifest.parent
    results = []
    for vector in manifest["vectors"]:
        path = root / vector["path"]
        raw = path.read_bytes()
        checksum_ok = hashlib.sha256(raw).hexdigest() == vector["sha256"]
        value = json.loads(raw)
        decision = vector_decision(vector["boundary"], value)
        canonical_sha256 = hashlib.sha256(canonical_json(value)).hexdigest()
        canonical_ok = canonical_sha256 == vector.get("canonical_sha256", canonical_sha256)
        results.append(
            {
                "id": vector["id"],
                "checksum_ok": checksum_ok,
                "decision": decision,
                "expected": vector["expected"],
                "canonical_sha256": canonical_sha256,
                "ok": checksum_ok and canonical_ok and decision == vector["expected"],
            }
        )
    output = {
        "type": "codefire_conformance_result",
        "version": 1,
        "consumer": "independent-python-stdlib-v1",
        "results": results,
        "ok": all(result["ok"] for result in results),
    }
    print(json.dumps(output, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
    return 0 if output["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
