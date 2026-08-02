#!/usr/bin/env python3
"""Set a parent-death signal, verify the expected parent, then exec argv."""

from __future__ import annotations

import ctypes
import os
import signal
import sys


PR_SET_PDEATHSIG = 1


def main(argv: list[str]) -> int:
    if len(argv) < 4 or argv[0] != "--parent-pid":
        print(
            "pdeath launcher: expected --parent-pid PID -- COMMAND",
            file=sys.stderr,
        )
        return 2
    try:
        expected_parent = int(argv[1])
    except ValueError:
        print("pdeath launcher: invalid parent PID", file=sys.stderr)
        return 2
    if expected_parent < 1 or argv[2] != "--" or not argv[3]:
        print("pdeath launcher: invalid command boundary", file=sys.stderr)
        return 2

    libc = ctypes.CDLL(None, use_errno=True)
    prctl = libc.prctl
    prctl.argtypes = [
        ctypes.c_int,
        ctypes.c_ulong,
        ctypes.c_ulong,
        ctypes.c_ulong,
        ctypes.c_ulong,
    ]
    prctl.restype = ctypes.c_int
    if prctl(PR_SET_PDEATHSIG, signal.SIGKILL, 0, 0, 0) != 0:
        error = ctypes.get_errno()
        print(
            f"pdeath launcher: prctl failed with errno {error}",
            file=sys.stderr,
        )
        return 126
    if os.getppid() != expected_parent:
        return 125
    try:
        os.execv(argv[3], argv[3:])
    except OSError as exc:
        print(
            f"pdeath launcher: exec failed with errno {exc.errno}",
            file=sys.stderr,
        )
        return 127


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
