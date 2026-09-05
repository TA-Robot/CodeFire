#!/usr/bin/env python3
"""Read Grok account billing through ACP, without creating a model session.

This is a current account-state check, not a per-request billing lock or an
authentication-routing check. Recheck before each worker; never cache a pass.
Only initialize and two read-only extensions are sent. Credentials, raw replies,
and child stderr are never printed or persisted by this helper.

Empty successful auto-topup objects mean no rule (disabled), per official tests:
https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-pager/src/app/effects/tests.rs#L671
All monetary fields must be explicitly present; unknown data fails closed.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import math
import os
import selectors
import signal
import subprocess
import time


class PreflightBlocked(Exception):
    """A fixed, credential-free explanation of an unverified billing state."""


def check_environment(environment: dict) -> None:
    """Fail closed on provider env overrides; never report their values."""
    for name, value in environment.items():
        if not value or name == "DEVCONTAINER_GROK_CLI_VERSION":
            continue
        if name.startswith(("GROK_", "XAI_", "DEVCONTAINER_GROK_")):
            raise PreflightBlocked("Grok provider environment overrides require review")
    if environment.get("DEVCONTAINER_AI_CLI_PREFIX", "/opt/devcontainer-ai-cli") != "/opt/devcontainer-ai-cli":
        raise PreflightBlocked("Grok binary location override requires review")


def assess_billing(billing: object, topup: object) -> dict:
    """Validate successful extension payloads and return allowlisted evidence."""
    if not isinstance(billing, dict) or not isinstance(billing.get("config"), dict):
        raise PreflightBlocked("missing billing configuration")
    config = billing["config"]
    if config.get("isUnifiedBillingUser") is not True:
        raise PreflightBlocked("unverified unified subscription billing")
    used = config.get("creditUsagePercent")
    if type(used) not in (int, float) or not math.isfinite(used) or not 0 <= used < 100:
        raise PreflightBlocked("subscription allowance unavailable or exhausted")
    evidence = {"usage_percent": used}
    for field, label in (
        ("prepaidBalance", "purchased_credits_cents"),
        ("onDemandCap", "payg_cap_cents"),
        ("onDemandUsed", "payg_used_cents"),
    ):
        amount = config.get(field)
        if not isinstance(amount, dict) or type(amount.get("val")) is not int:
            raise PreflightBlocked("missing or invalid monetary billing field")
        if amount["val"] != 0:
            raise PreflightBlocked("purchased credits or pay-as-you-go detected")
        evidence[label] = 0
    # Accept only the documented no-rule shapes, not arbitrary objects that
    # happen to omit rule (e.g. {"error": ...} or a future response envelope).
    if not isinstance(topup, dict) or set(topup) - {"rule"}:
        raise PreflightBlocked("unrecognized auto-topup response")
    rule = topup.get("rule")
    if rule is not None:
        known = {"enabled", "minBeforeHittingSl", "topupAmount", "maxAmountPerMonth"}
        if not isinstance(rule, dict) or set(rule) - known:
            raise PreflightBlocked("unrecognized auto-topup rule")
        enabled = rule.get("enabled", False)  # proto3 false is omitted
        if type(enabled) is not bool or enabled:
            raise PreflightBlocked("automatic top-up enabled or unknown")
        for name in known - {"enabled"}:
            if name in rule:
                amount = rule[name]
                if amount is not None and (
                    not isinstance(amount, dict)
                    or set(amount) - {"val"}
                    or type(amount.get("val", 0)) is not int
                ):
                    raise PreflightBlocked("invalid auto-topup rule amount")
    evidence["auto_topup_enabled"] = False
    evidence["unified_subscription_billing"] = True
    return evidence


def probe(
    command: tuple[str, ...] = ("grok", "--no-auto-update", "agent", "--no-leader", "stdio"),
    timeout: float = 25,
) -> dict:
    """Use an isolated ACP process and one deadline; kill its process group."""
    if not math.isfinite(timeout) or not 0 < timeout <= 60:
        raise PreflightBlocked("timeout must be greater than zero and at most 60 seconds")
    check_environment(os.environ)
    process = None
    selector = selectors.DefaultSelector()
    pending = b""
    deadline = time.monotonic() + timeout
    try:
        process = subprocess.Popen(
            command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, start_new_session=True,
        )
        selector.register(process.stdout, selectors.EVENT_READ)

        def request(identifier: int, method: str, params: dict) -> object:
            nonlocal pending
            message = {"jsonrpc": "2.0", "id": identifier, "method": method, "params": params}
            process.stdin.write((json.dumps(message) + "\n").encode())
            process.stdin.flush()
            while True:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise PreflightBlocked("billing probe timed out")
                if b"\n" not in pending:
                    if not selector.select(remaining):
                        raise PreflightBlocked("billing probe timed out")
                    chunk = os.read(process.stdout.fileno(), 65536)
                    if not chunk:
                        raise PreflightBlocked("billing probe exited before replying")
                    pending += chunk
                    if len(pending) > 1_048_576:
                        raise PreflightBlocked("billing probe response too large")
                    continue
                line, pending = pending.split(b"\n", 1)
                try:
                    reply = json.loads(line)
                except (ValueError, UnicodeError):
                    raise PreflightBlocked("invalid ACP response") from None
                if not isinstance(reply, dict) or reply.get("jsonrpc") != "2.0":
                    raise PreflightBlocked("invalid ACP envelope")
                if "method" in reply:
                    if "id" in reply:
                        raise PreflightBlocked("unexpected agent request; no action authorized")
                    continue  # Notifications are never executed or printed.
                if type(reply.get("id")) is not int or reply["id"] != identifier:
                    raise PreflightBlocked("unexpected ACP response identifier")
                if "error" in reply or "result" not in reply:
                    raise PreflightBlocked("billing ACP request failed")
                return reply["result"]

        initialized = request(1, "initialize", {
            "protocolVersion": 1, "clientCapabilities": {},
            "clientInfo": {"name": "codefire-billing-preflight", "version": "0.1"},
        })
        if not isinstance(initialized, dict) or initialized.get("protocolVersion") != 1:
            raise PreflightBlocked("unsupported ACP initialization")
        billing = request(2, "_x.ai/billing", {})
        topup = request(3, "_x.ai/auto-topup-rule", {})
        return assess_billing(billing, topup)
    except (OSError, subprocess.SubprocessError):
        raise PreflightBlocked("unable to complete Grok billing probe") from None
    finally:
        selector.close()
        if process is not None:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
            # A reaped leader does not imply its process group is empty. A
            # descendant can ignore TERM and retain pipes after the leader
            # exits, so escalate the group on both wait outcomes.
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                raise PreflightBlocked("billing probe process did not terminate") from None
            finally:
                process.stdin.close()
                process.stdout.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=float, default=25)
    args = parser.parse_args()
    try:
        evidence = probe(timeout=args.timeout)
    except PreflightBlocked as error:
        print(json.dumps({"status": "blocked", "reason": str(error)}))
        return 1
    print(json.dumps({
        "status": "verified", "checked_at": datetime.now(timezone.utc).isoformat(),
        **evidence,
    }))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
