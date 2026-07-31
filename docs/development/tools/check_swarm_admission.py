#!/usr/bin/env python3
"""Decide whether one more external lane fits the live resource envelope."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


GIB = 1024**3

# Bootstrap reservations include the agent process, its tool servers, and a
# safety margin. Replace them with a measured p95 * 1.5 once a comparable
# cohort exists; they are reservations for one candidate, not session caps.
DEFAULT_RESERVATIONS = {
    "light": 2 * GIB,
    "python-heavy": 3 * GIB,
    "rust-heavy": 4 * GIB,
}

DEFAULT_THRESHOLDS = {
    "available_after_bytes": 6 * GIB,
    "swap_used_bytes": 2 * GIB,
    "disk_available_bytes": 100 * GIB,
    "memory_full_avg10": 0.5,
    "io_full_avg10": 5.0,
    "cpu_some_avg10": 25.0,
    "pid_headroom": 512,
}


def parse_meminfo(text: str) -> dict[str, int]:
    values: dict[str, int] = {}
    for line in text.splitlines():
        parts = line.split()
        if len(parts) < 2 or not parts[0].endswith(":"):
            continue
        multiplier = 1024 if len(parts) >= 3 and parts[2] == "kB" else 1
        values[parts[0][:-1]] = int(parts[1]) * multiplier
    return values


def parse_psi(text: str) -> dict[str, dict[str, float | int]]:
    result: dict[str, dict[str, float | int]] = {}
    for line in text.splitlines():
        parts = line.split()
        if not parts:
            continue
        values: dict[str, float | int] = {}
        for field in parts[1:]:
            key, raw = field.split("=", 1)
            values[key] = int(raw) if key == "total" else float(raw)
        result[parts[0]] = values
    return result


def read_optional_limit(path: Path) -> int | None:
    raw = path.read_text(encoding="utf-8").strip()
    return None if raw == "max" else int(raw)


def effective_cpu_count(cgroup_root: Path) -> float:
    affinity_count = float(
        len(os.sched_getaffinity(0))
        if hasattr(os, "sched_getaffinity")
        else (os.cpu_count() or 1)
    )
    raw = (cgroup_root / "cpu.max").read_text(encoding="utf-8").split()
    if raw[0] == "max":
        return affinity_count
    quota, period = int(raw[0]), int(raw[1])
    return min(affinity_count, quota / period)


def resource_snapshot(
    disk_path: Path = Path("/workspace"),
    proc_root: Path = Path("/proc"),
    cgroup_root: Path = Path("/sys/fs/cgroup"),
) -> dict[str, Any]:
    meminfo = parse_meminfo((proc_root / "meminfo").read_text(encoding="utf-8"))
    memory_max = read_optional_limit(cgroup_root / "memory.max")
    memory_current = int(
        (cgroup_root / "memory.current").read_text(encoding="utf-8").strip()
    )
    cgroup_memory_headroom = (
        None if memory_max is None else max(0, memory_max - memory_current)
    )
    mem_available = meminfo["MemAvailable"]
    effective_available = (
        mem_available
        if cgroup_memory_headroom is None
        else min(mem_available, cgroup_memory_headroom)
    )
    pids_max = read_optional_limit(cgroup_root / "pids.max")
    pids_current = int(
        (cgroup_root / "pids.current").read_text(encoding="utf-8").strip()
    )
    disk = shutil.disk_usage(disk_path)
    load1, load5, load15 = os.getloadavg()
    cpu_count = effective_cpu_count(cgroup_root)
    return {
        "schema": "codefire.swarm_resource_snapshot.v1",
        "captured_at": datetime.now(timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z"),
        "disk_path": str(disk_path.resolve()),
        "mem_available_bytes": mem_available,
        "cgroup_memory_current_bytes": memory_current,
        "cgroup_memory_max_bytes": memory_max,
        "cgroup_memory_headroom_bytes": cgroup_memory_headroom,
        "effective_available_bytes": effective_available,
        "swap_total_bytes": meminfo["SwapTotal"],
        "swap_used_bytes": meminfo["SwapTotal"] - meminfo["SwapFree"],
        "disk_available_bytes": disk.free,
        "effective_cpu_count": cpu_count,
        "load": {"one": load1, "five": load5, "fifteen": load15},
        "load_per_cpu": load1 / cpu_count,
        "psi": {
            kind: parse_psi(
                (proc_root / "pressure" / kind).read_text(encoding="utf-8")
            )
            for kind in ("cpu", "memory", "io")
        },
        "pids_current": pids_current,
        "pids_max": pids_max,
        "pids_headroom": None if pids_max is None else pids_max - pids_current,
    }


def evaluate_admission(
    snapshot: dict[str, Any],
    *,
    resource_class: str,
    reservation_bytes: int,
    provider: str,
    provider_active: int,
    provider_window: int,
    outstanding_reservation_bytes: int = 0,
    provider_backoff: bool = False,
    conflict_reasons: list[str] | None = None,
    thresholds: dict[str, float | int] | None = None,
) -> dict[str, Any]:
    limits = dict(DEFAULT_THRESHOLDS if thresholds is None else thresholds)
    conflicts = [] if conflict_reasons is None else list(conflict_reasons)
    checks: list[dict[str, Any]] = []

    def check(
        check_id: str,
        passed: bool,
        observed: Any,
        limit: Any,
        detail: str,
    ) -> None:
        checks.append(
            {
                "id": check_id,
                "status": "pass" if passed else "block",
                "observed": observed,
                "limit": limit,
                "detail": detail,
            }
        )

    available = int(snapshot["effective_available_bytes"])
    projected = available - outstanding_reservation_bytes - reservation_bytes
    check(
        "projected-memory-floor",
        projected >= int(limits["available_after_bytes"]),
        projected,
        limits["available_after_bytes"],
        "effective available memory after reserving the candidate lane",
    )
    check(
        "swap-use",
        int(snapshot["swap_used_bytes"]) <= int(limits["swap_used_bytes"]),
        snapshot["swap_used_bytes"],
        limits["swap_used_bytes"],
        "current swap use",
    )
    check(
        "workspace-disk-floor",
        int(snapshot["disk_available_bytes"])
        >= int(limits["disk_available_bytes"]),
        snapshot["disk_available_bytes"],
        limits["disk_available_bytes"],
        "current workspace filesystem headroom",
    )
    check(
        "memory-pressure",
        float(snapshot["psi"]["memory"]["full"]["avg10"])
        <= float(limits["memory_full_avg10"]),
        snapshot["psi"]["memory"]["full"]["avg10"],
        limits["memory_full_avg10"],
        "ten-second full memory-stall percentage",
    )
    check(
        "io-pressure",
        float(snapshot["psi"]["io"]["full"]["avg10"])
        <= float(limits["io_full_avg10"]),
        snapshot["psi"]["io"]["full"]["avg10"],
        limits["io_full_avg10"],
        "ten-second full I/O-stall percentage",
    )
    check(
        "cpu-pressure",
        float(snapshot["psi"]["cpu"]["some"]["avg10"])
        <= float(limits["cpu_some_avg10"]),
        snapshot["psi"]["cpu"]["some"]["avg10"],
        limits["cpu_some_avg10"],
        "ten-second runnable CPU-pressure percentage",
    )
    pid_headroom = snapshot["pids_headroom"]
    check(
        "pid-headroom",
        pid_headroom is None or int(pid_headroom) >= int(limits["pid_headroom"]),
        pid_headroom,
        limits["pid_headroom"],
        "cgroup process-ID headroom; unlimited passes",
    )
    check(
        "provider-window",
        provider_active < provider_window,
        provider_active,
        provider_window,
        "active lanes must remain below the current adaptive provider window",
    )
    check(
        "provider-backoff",
        not provider_backoff,
        provider_backoff,
        False,
        "provider overload/rate-limit cooldown must be clear",
    )
    check(
        "conflict-domain",
        not conflicts,
        conflicts,
        [],
        "candidate must not overlap an owned write, integration, or build lock",
    )

    blockers = [item["id"] for item in checks if item["status"] == "block"]
    return {
        "schema": "codefire.swarm_admission.v1",
        "decision": "ALLOW" if not blockers else "BLOCK",
        "basis": "resource-and-provider-window; no fixed total session cap",
        "resource_class": resource_class,
        "reservation_bytes": reservation_bytes,
        "outstanding_reservation_bytes": outstanding_reservation_bytes,
        "provider": {
            "name": provider,
            "active": provider_active,
            "window": provider_window,
            "backoff": provider_backoff,
        },
        "conflict_reasons": conflicts,
        "projected_available_bytes": projected,
        "checks": checks,
        "blockers": blockers,
        "snapshot": snapshot,
    }


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary, path)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Assess one external lane from live resource pressure, a measured "
            "reservation, provider congestion window, and conflict ownership."
        )
    )
    parser.add_argument(
        "--resource-class",
        choices=sorted(DEFAULT_RESERVATIONS),
        required=True,
    )
    parser.add_argument("--reservation-gib", type=float)
    parser.add_argument(
        "--outstanding-reservation-gib",
        type=float,
        default=0.0,
        help=(
            "reservations already admitted but not yet reflected in live "
            "memory pressure"
        ),
    )
    parser.add_argument("--provider", required=True)
    parser.add_argument("--provider-active", type=int, required=True)
    parser.add_argument("--provider-window", type=int, required=True)
    parser.add_argument("--provider-backoff", action="store_true")
    parser.add_argument("--conflict", action="append", default=[])
    parser.add_argument("--disk-path", type=Path, default=Path("/workspace"))
    parser.add_argument("--result", type=Path)
    args = parser.parse_args(argv)
    if args.reservation_gib is not None and args.reservation_gib <= 0:
        parser.error("--reservation-gib must be positive")
    if args.outstanding_reservation_gib < 0:
        parser.error("--outstanding-reservation-gib must not be negative")
    if args.provider_active < 0:
        parser.error("--provider-active must not be negative")
    if args.provider_window < 1:
        parser.error("--provider-window must be positive")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    reservation = (
        DEFAULT_RESERVATIONS[args.resource_class]
        if args.reservation_gib is None
        else int(args.reservation_gib * GIB)
    )
    try:
        snapshot = resource_snapshot(args.disk_path)
        result = evaluate_admission(
            snapshot,
            resource_class=args.resource_class,
            reservation_bytes=reservation,
            provider=args.provider,
            provider_active=args.provider_active,
            provider_window=args.provider_window,
            outstanding_reservation_bytes=int(
                args.outstanding_reservation_gib * GIB
            ),
            provider_backoff=args.provider_backoff,
            conflict_reasons=args.conflict,
        )
        if args.result is not None:
            atomic_write_json(args.result, result)
    except (KeyError, OSError, ValueError) as exc:
        print(f"swarm admission error: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["decision"] == "ALLOW" else 3


if __name__ == "__main__":
    raise SystemExit(main())
