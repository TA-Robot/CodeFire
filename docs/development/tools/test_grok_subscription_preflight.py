"""Billing safety and no-model ACP contract tests (no external services)."""

import copy
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest

from grok_subscription_preflight import PreflightBlocked, assess_billing, check_environment, probe


SAFE = {"config": {
    "creditUsagePercent": 22.0,
    "isUnifiedBillingUser": True,
    "prepaidBalance": {"val": 0},
    "onDemandCap": {"val": 0},
    "onDemandUsed": {"val": 0},
}}


class BillingStateTests(unittest.TestCase):
    def test_provider_environment_overrides_block_without_exposing_values(self):
        check_environment({"DEVCONTAINER_GROK_CLI_VERSION": "1.0.13"})
        for name in ("XAI_API_KEY", "GROK_CLI_CHAT_PROXY_BASE_URL", "GROK_HOME",
                     "DEVCONTAINER_GROK_REAL_BIN", "DEVCONTAINER_AI_CLI_PREFIX"):
            with self.subTest(name=name):
                with self.assertRaises(PreflightBlocked) as caught:
                    check_environment({name: "SECRET"})
                self.assertNotIn("SECRET", str(caught.exception))

    def test_documented_disabled_rule_forms(self):
        for rule in ({}, {"rule": None}, {"rule": {}},
                     {"rule": {"enabled": False}},
                     {"rule": {"topupAmount": {"val": 500}}}):
            with self.subTest(rule=rule):
                self.assertFalse(assess_billing(SAFE, rule)["auto_topup_enabled"])

    def test_missing_and_nonzero_money_always_block(self):
        for field in ("prepaidBalance", "onDemandCap", "onDemandUsed"):
            for value in (None, {}, {"val": 1}, {"val": -1}, {"val": False},
                          {"val": "0"}, {"val": 0.0}):
                with self.subTest(field=field, value=value):
                    billing = copy.deepcopy(SAFE)
                    billing["config"][field] = value
                    with self.assertRaises(PreflightBlocked):
                        assess_billing(billing, {})

    def test_unknown_billing_and_exhaustion_block(self):
        for value in (None, True, "22", -1, 100, 101, float("nan"), float("inf")):
            billing = copy.deepcopy(SAFE)
            billing["config"]["creditUsagePercent"] = value
            with self.subTest(value=value), self.assertRaises(PreflightBlocked):
                assess_billing(billing, {})
        for value in (None, False, 1, "true"):
            billing = copy.deepcopy(SAFE)
            billing["config"]["isUnifiedBillingUser"] = value
            with self.subTest(value=value), self.assertRaises(PreflightBlocked):
                assess_billing(billing, {})
        for value in (None, {}, {"config": []}):
            with self.assertRaises(PreflightBlocked):
                assess_billing(value, {})

    def test_unknown_topup_or_enabled_topup_block(self):
        for value in (None, [], 0, {"error": "unavailable"}, {"raw": "bad"},
                      {"rule": []}, {"rule": {"enabled": True}},
                      {"rule": {"enabled": None}}, {"rule": {"enabled": 0}},
                      {"rule": {"future_enabled_flag": True}},
                      {"rule": {"topupAmount": {"val": "0"}}}):
            with self.subTest(value=value), self.assertRaises(PreflightBlocked):
                assess_billing(SAFE, value)

    def test_evidence_does_not_include_unrelated_private_fields(self):
        billing = copy.deepcopy(SAFE)
        billing["private_identity"] = "must not escape"
        self.assertNotIn("must not escape", json.dumps(assess_billing(billing, {})))


def fake_agent(final_reply=None, mode="normal"):
    """Mock endpoint enforces the exact three messages and forbids sessions."""
    code = f"""
import json, sys, time
expected = ['initialize', '_x.ai/billing', '_x.ai/auto-topup-rule']
responses = [{{'protocolVersion': 1}}, {SAFE!r}, {{}}]
for i, method in enumerate(expected, 1):
    request = json.loads(sys.stdin.readline())
    assert request['method'] == method
    assert request['id'] == i
    if i > 1:
        assert request['params'] == {{}}
    if {mode!r} == 'timeout':
        time.sleep(10)
    if {mode!r} == 'callback':
        print(json.dumps({{'jsonrpc':'2.0','id':99,'method':'session/request_permission','params':{{}}}}), flush=True)
        time.sleep(10)
    response = {{'jsonrpc':'2.0','id':i,'result':responses[i-1]}}
    if i == 3 and {final_reply is not None!r}:
        response = {final_reply!r}
    print(json.dumps(response), flush=True)
assert sys.stdin.readline() == ''
"""
    return (sys.executable, "-u", "-c", code)


class TransportTests(unittest.TestCase):
    @unittest.skipUnless(sys.platform == "linux", "descendant reaping uses Linux subreaper")
    def test_term_resistant_descendant_is_killed_after_leader_exits(self):
        # Run the assertion in its own subreaper process, so the deliberately
        # orphaned grandchild is reaped without changing the test runner's
        # process attributes or leaving zombies with the container's PID 1.
        root = Path(__file__).resolve().parents[3]
        scratch = root / "target" / "recovery"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            pid_path = str(Path(directory) / "descendant.pid")
            descendant = (
                "import os, signal\nfrom pathlib import Path\n"
                "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
                f"Path({pid_path!r}).write_text(str(os.getpid()))\n"
                "while True: signal.pause()\n"
            )
            leader = (
                "import subprocess, sys, time\nfrom pathlib import Path\n"
                f"subprocess.Popen([sys.executable, '-c', {descendant!r}], "
                "stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)\n"
                f"while not Path({pid_path!r}).exists(): time.sleep(0.01)\n"
                + fake_agent()[3]
            )
            harness = f"""
import ctypes, os, signal, sys, time
from pathlib import Path
sys.path.insert(0, {str(Path(__file__).resolve().parent)!r})
from grok_subscription_preflight import probe
assert ctypes.CDLL(None).prctl(36, 1, 0, 0, 0) == 0  # PR_SET_CHILD_SUBREAPER
child = None
try:
    probe((sys.executable, '-u', '-c', {leader!r}), timeout=3)
    child = int(Path({pid_path!r}).read_text())
    deadline = time.monotonic() + 2
    while time.monotonic() < deadline:
        pid, status = os.waitpid(child, os.WNOHANG)
        if pid:
            child = None
            assert os.WIFSIGNALED(status) and os.WTERMSIG(status) == signal.SIGKILL
            break
        time.sleep(0.01)
    assert child is None, 'TERM-resistant descendant survived leader exit'
finally:
    if child is None and Path({pid_path!r}).exists():
        # The assertion may have failed before assigning child. Only signal an
        # unreaped child owned by this harness, never a potentially reused PID.
        candidate = int(Path({pid_path!r}).read_text())
        try:
            if os.waitpid(candidate, os.WNOHANG)[0] == 0: child = candidate
        except ChildProcessError:
            pass
    if child is not None:
        os.kill(child, signal.SIGKILL)
        os.waitpid(child, 0)
"""
            process = subprocess.Popen(
                (sys.executable, "-u", "-c", harness),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                start_new_session=True,
            )
            try:
                stdout, stderr = process.communicate(timeout=10)
                self.assertEqual(process.returncode, 0, (stdout + stderr).decode())
            finally:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.communicate()

    def test_only_initialization_and_readonly_extensions(self):
        self.assertEqual(probe(fake_agent(), timeout=3)["usage_percent"], 22)

    def test_error_or_missing_result_cannot_become_no_rule(self):
        for reply in (
            {"jsonrpc": "2.0", "id": 3, "error": {"message": "SECRET"}},
            {"jsonrpc": "2.0", "id": 3},
            {"jsonrpc": "2.0", "id": 3, "result": None},
            {"jsonrpc": "2.0", "id": 99, "result": {}},
        ):
            with self.subTest(reply=reply):
                with self.assertRaises(PreflightBlocked) as caught:
                    probe(fake_agent(reply), timeout=3)
                self.assertNotIn("SECRET", str(caught.exception))

    def test_timeout_and_callback_block_without_execution(self):
        for mode in ("timeout", "callback"):
            with self.subTest(mode=mode), self.assertRaises(PreflightBlocked):
                probe(fake_agent(mode=mode), timeout=0.3)

    def test_invalid_timeout_blocks_before_startup(self):
        for value in (0, -1, 61, float("nan")):
            with self.assertRaises(PreflightBlocked):
                probe(("nonexistent",), timeout=value)


if __name__ == "__main__":
    unittest.main()
